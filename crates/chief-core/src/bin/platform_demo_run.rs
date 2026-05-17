//! Deterministic Platform MVP demo runner.
//!
//! Seeds the Acme Work Object, runs all POC packs through `chief-sdk`, opens a
//! pending Ceremony for the drafted reply, then serves the L4 HTTP/UI surface.

use async_trait::async_trait;
use axum::Router;
use calendar_pack::CalendarAgent;
use chief_core::capability::{CapabilityKind, Grant};
use chief_core::ceremony::{CeremonyEvidence, NewCeremony};
use chief_core::inbox::{BadgeVariant, InboxKind, NewInboxItem};
use chief_core::routes::router_with_dist;
use chief_core::state::{AppConfig, BackendKind};
use chief_core::AppState;
use chief_mem::{Edge, EdgeKind, Horizon, Node, NodeType};
use chief_sdk::prelude::*;
use document_pack::DocumentAgent;
use email_pack::EmailAgent;
use risk_pack::RiskAgent;
use serde_json::{json, Value};
use std::collections::HashSet;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::info;

const CONTRACT: &str = include_str!("../../tests/fixtures/platform_demo/acme-contract.md");
const CALENDAR: &str = include_str!("../../tests/fixtures/platform_demo/calendar.json");
const PRIOR_EMAIL: &str = include_str!("../../tests/fixtures/platform_demo/prior-email.json");

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let state_dir = std::env::var("CHIEF_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/tmp/chief-platform-demo"));
    let dist_dir = std::env::var("CHIEF_OS_DIST_PATH").ok().map(PathBuf::from);
    let bind: SocketAddr = std::env::var("CHIEF_BIND")
        .unwrap_or_else(|_| "0.0.0.0:8080".into())
        .parse()?;

    let state = Arc::new(
        AppState::new(AppConfig {
            state_dir: Some(state_dir),
            dev_mode: true,
            backend: BackendKind::Stub,
            model_path: None,
        })
        .await?,
    );

    seed_acme_work_object(&state).await?;
    run_demo_packs(Arc::clone(&state)).await?;
    open_pending_ceremony(&state).await?;

    let app: Router = router_with_dist(Arc::clone(&state), dist_dir);
    let listener = tokio::net::TcpListener::bind(bind).await?;
    info!(%bind, "platform demo HTTP listening");
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await?;
    Ok(())
}

async fn run_demo_packs(state: Arc<AppState>) -> anyhow::Result<()> {
    DocumentAgent
        .on_tick(ctx_for(
            Arc::clone(&state),
            &["artifact", "file"],
            &["finding"],
        ))
        .await?;
    CalendarAgent
        .on_tick(ctx_for(
            Arc::clone(&state),
            &["artifact", "finding", "event"],
            &["finding"],
        ))
        .await?;
    EmailAgent
        .on_tick(ctx_for(
            Arc::clone(&state),
            &["artifact", "finding", "email"],
            &["artifact"],
        ))
        .await?;
    RiskAgent
        .on_tick(ctx_for(
            Arc::clone(&state),
            &["artifact", "finding"],
            &["finding"],
        ))
        .await?;
    Ok(())
}

async fn open_pending_ceremony(state: &AppState) -> anyhow::Result<()> {
    let draft = {
        let mem = state.mem.lock().await;
        mem.nodes_by_type(NodeType::Artifact)?
            .into_iter()
            .find_map(|stored| {
                let body = serde_json::from_str::<Value>(&stored.node.body).ok()?;
                (body.get("kind").and_then(Value::as_str) == Some("draft_reply"))
                    .then_some((stored.uri, body))
            })
    };
    let Some((draft_uri, draft_body)) = draft else {
        anyhow::bail!("email draft not found");
    };

    let payload_hash = payload_hash(&draft_body);
    let ceremony = state
        .ceremonies
        .open(NewCeremony {
            title: "Send Acme follow-up reply".into(),
            evidence: CeremonyEvidence {
                summary: "email-pack drafted the Acme reply and requires Ceremony before send."
                    .into(),
                details: json!({
                    "draft_uri": draft_uri,
                    "payload": draft_body,
                    "payload_hash": payload_hash,
                    "requested_op": "email.send",
                    "principal": "pack:email-pack",
                }),
            },
            source_agent: "email-pack".into(),
            trust_context: 3,
            proposed_grant: Grant::new(vec![CapabilityKind::ceremony_request(
                vec!["email.send".into()],
                "Permit committing the exact Acme follow-up draft approved in Ceremony.",
            )]),
            payload_hash: Some(payload_hash),
            target_principal: "pack:email-pack".into(),
            rollback_window: chrono::Duration::hours(24),
            ceremony_ttl: chrono::Duration::hours(12),
        })
        .await;

    state
        .inbox
        .append(NewInboxItem {
            kind: InboxKind::CeremonyPending,
            title: "Send Acme follow-up reply".into(),
            snippet: "email-pack needs Ceremony before sending externally.".into(),
            source_agent: "pack:email-pack".into(),
            badge: BadgeVariant::Ceremony,
            ceremony_id: Some(ceremony.id),
        })
        .await;
    Ok(())
}

fn payload_hash(value: &Value) -> String {
    let bytes = serde_json::to_vec(value).expect("payload json");
    format!("blake3:{}", blake3::hash(&bytes).to_hex())
}

fn ctx_for(state: Arc<AppState>, read_types: &[&str], write_types: &[&str]) -> CapabilityContext {
    let memory = Arc::new(ScopedKernelMemoryConnector {
        state,
        read_types: read_types.iter().map(|s| s.to_string()).collect(),
        write_types: write_types.iter().map(|s| s.to_string()).collect(),
    });
    let stub = Arc::new(InMemoryConnector::new());
    CapabilityContext::new(stub.clone(), memory, stub.clone(), stub)
}

struct ScopedKernelMemoryConnector {
    state: Arc<AppState>,
    read_types: HashSet<String>,
    write_types: HashSet<String>,
}

#[async_trait]
impl MemoryConnector for ScopedKernelMemoryConnector {
    async fn put_node(&self, node: Value) -> chief_sdk::Result<String> {
        let node_type = node["node_type"]
            .as_str()
            .ok_or_else(|| SdkError::Connector("missing node_type".into()))?;
        if !self.write_types.contains(node_type) {
            return Err(SdkError::Connector(format!(
                "write denied for node_type {node_type}"
            )));
        }
        let node_type = parse_node_type(node_type)?;
        let horizon = node["horizon"]
            .as_str()
            .unwrap_or("medium")
            .parse()
            .map_err(|_| SdkError::Connector("invalid horizon".into()))?;
        let source = node["source"].as_str().unwrap_or("pack:unknown");
        let body = node["body"].clone().to_string();
        let mem = self.state.mem.lock().await;
        mem.put_node(Node::new(node_type, horizon, source, body))
            .map_err(|err| SdkError::Connector(err.to_string()))
    }

    async fn get_node(&self, id: &str) -> chief_sdk::Result<Option<Value>> {
        let mem = self.state.mem.lock().await;
        let Some(node) = mem
            .get_node(id)
            .map_err(|err| SdkError::Connector(err.to_string()))?
        else {
            return Ok(None);
        };
        if !self.read_types.contains(node.node_type.as_str()) {
            return Ok(None);
        }
        Ok(Some(node_to_json(id.to_string(), node)))
    }

    async fn query(&self, filter: Value) -> chief_sdk::Result<Vec<Value>> {
        let requested: HashSet<String> = filter["types"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
            .map(ToOwned::to_owned)
            .collect();
        let allowed: Vec<NodeType> = requested
            .intersection(&self.read_types)
            .map(|ty| parse_node_type(ty))
            .collect::<chief_sdk::Result<Vec<_>>>()?;

        let mem = self.state.mem.lock().await;
        let mut out = Vec::new();
        for node_type in allowed {
            for stored in mem
                .nodes_by_type(node_type)
                .map_err(|err| SdkError::Connector(err.to_string()))?
            {
                out.push(node_to_json(stored.uri, stored.node));
            }
        }
        Ok(out)
    }
}

fn parse_node_type(value: &str) -> chief_sdk::Result<NodeType> {
    value
        .parse()
        .map_err(|_| SdkError::Connector(format!("invalid node type {value}")))
}

fn node_to_json(uri: String, node: Node) -> Value {
    json!({
        "uri": uri,
        "node_type": node.node_type.to_string(),
        "source": node.source,
        "body": serde_json::from_str::<Value>(&node.body).unwrap_or_else(|_| json!(node.body)),
    })
}

async fn seed_acme_work_object(state: &AppState) -> anyhow::Result<String> {
    let mem = state.mem.lock().await;
    let contract_uri = mem.put_node(Node::new(
        NodeType::File,
        Horizon::Medium,
        "fixture:document-pack",
        json!({
            "kind": "fixture_file",
            "path": "fixtures/platform_demo/acme-contract.md",
            "mime": "text/markdown",
            "body": CONTRACT
        })
        .to_string(),
    ))?;
    let calendar_uri = mem.put_node(Node::new(
        NodeType::Event,
        Horizon::Medium,
        "fixture:calendar-pack",
        CALENDAR,
    ))?;
    let prior_email_uri = mem.put_node(Node::new(
        NodeType::Email,
        Horizon::Medium,
        "fixture:email-pack",
        PRIOR_EMAIL,
    ))?;

    let work_uri = mem.put_node(Node::new(
        NodeType::Artifact,
        Horizon::Medium,
        "chief-core:platform-demo-run",
        json!({
            "kind": "work_object",
            "id": "acme-follow-up",
            "title": "Prepare the Acme follow-up",
            "source_refs": [contract_uri, calendar_uri, prior_email_uri],
            "contributions": []
        })
        .to_string(),
    ))?;

    for source in [&contract_uri, &calendar_uri, &prior_email_uri] {
        let _ = mem.put_edge(Edge {
            from: work_uri.clone(),
            to: source.to_string(),
            kind: EdgeKind::DerivedFrom,
            weight: 1.0,
            created_by: "chief-core:platform-demo-run".into(),
        });
    }

    Ok(work_uri)
}
