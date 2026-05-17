//! End-to-end pack interop test for the Platform MVP demo.
//!
//! This boots real `AppState`, seeds the real Memory Graph, runs three
//! independent pack agents through public `chief-sdk` contexts, then verifies
//! the `/v1/work/:id` projection.

use async_trait::async_trait;
use axum::Router;
use calendar_pack::CalendarAgent;
use chief_core::routes::router_with_dist;
use chief_core::state::{AppConfig, BackendKind};
use chief_core::AppState;
use chief_mem::{Edge, EdgeKind, Horizon, Node, NodeType};
use chief_sdk::prelude::*;
use document_pack::DocumentAgent;
use email_pack::EmailAgent;
use serde_json::{json, Value};
use std::collections::HashSet;
use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;
use tempfile::tempdir;

const CONTRACT: &str = include_str!("fixtures/platform_demo/acme-contract.md");
const CALENDAR: &str = include_str!("fixtures/platform_demo/calendar.json");
const PRIOR_EMAIL: &str = include_str!("fixtures/platform_demo/prior-email.json");

struct TestServer {
    addr: SocketAddr,
    _shutdown: tokio::sync::oneshot::Sender<()>,
    _tmp: tempfile::TempDir,
}

impl TestServer {
    fn url(&self, path: &str) -> String {
        format!("http://{}{}", self.addr, path)
    }
}

async fn spawn(app: Router) -> TestServer {
    let tmp = tempdir().expect("tempdir");
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind");
    let addr = listener.local_addr().expect("local_addr");
    let (tx, rx) = tokio::sync::oneshot::channel();

    tokio::spawn(async move {
        let server = axum::serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .with_graceful_shutdown(async {
            let _ = rx.await;
        });
        let _ = server.await;
    });

    tokio::time::sleep(std::time::Duration::from_millis(20)).await;

    TestServer {
        addr,
        _shutdown: tx,
        _tmp: tmp,
    }
}

#[tokio::test]
async fn three_packs_converge_on_one_work_object_without_direct_coupling() {
    assert_no_forbidden_pack_imports("packs/document-pack/src/lib.rs");
    assert_no_forbidden_pack_imports("packs/calendar-pack/src/lib.rs");
    assert_no_forbidden_pack_imports("packs/email-pack/src/lib.rs");

    let tmp = tempdir().expect("state tempdir");
    let config = AppConfig {
        state_dir: Some(tmp.path().to_path_buf()),
        dev_mode: true,
        backend: BackendKind::Stub,
        model_path: None,
    };
    let state = Arc::new(AppState::new(config).await.expect("init state"));
    seed_acme_work_object(&state).await;

    DocumentAgent
        .on_tick(ctx_for(
            Arc::clone(&state),
            &["artifact", "file"],
            &["finding"],
        ))
        .await
        .expect("document agent");
    CalendarAgent
        .on_tick(ctx_for(
            Arc::clone(&state),
            &["artifact", "finding", "event"],
            &["finding"],
        ))
        .await
        .expect("calendar agent");
    EmailAgent
        .on_tick(ctx_for(
            Arc::clone(&state),
            &["artifact", "finding", "email"],
            &["artifact"],
        ))
        .await
        .expect("email agent");

    let app = router_with_dist(state, None);
    let srv = spawn(app).await;
    let body: Value = reqwest::get(srv.url("/v1/work/acme-follow-up"))
        .await
        .expect("GET work")
        .json()
        .await
        .expect("json");

    let contributions = body["contributions"].as_array().expect("contributions");
    let obligations = contributions
        .iter()
        .filter(|node| node["body"]["kind"] == "obligation")
        .count();
    let slots = contributions
        .iter()
        .filter(|node| node["body"]["kind"] == "candidate_slot")
        .count();
    let drafts = contributions
        .iter()
        .filter(|node| node["body"]["kind"] == "draft_reply")
        .count();

    assert!(obligations >= 3, "expected obligations: {contributions:?}");
    assert!(slots >= 2, "expected candidate slots: {contributions:?}");
    assert_eq!(drafts, 1, "expected one draft reply: {contributions:?}");
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

async fn seed_acme_work_object(state: &AppState) -> String {
    let mem = state.mem.lock().await;
    let contract_uri = mem
        .put_node(Node::new(
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
        ))
        .expect("put contract");
    let calendar_uri = mem
        .put_node(Node::new(
            NodeType::Event,
            Horizon::Medium,
            "fixture:calendar-pack",
            CALENDAR,
        ))
        .expect("put calendar");
    let prior_email_uri = mem
        .put_node(Node::new(
            NodeType::Email,
            Horizon::Medium,
            "fixture:email-pack",
            PRIOR_EMAIL,
        ))
        .expect("put prior email");

    let work_body = json!({
        "kind": "work_object",
        "id": "acme-follow-up",
        "title": "Prepare the Acme follow-up",
        "source_refs": [
            contract_uri,
            calendar_uri,
            prior_email_uri
        ],
        "contributions": []
    })
    .to_string();
    let work_uri = mem
        .put_node(Node::new(
            NodeType::Artifact,
            Horizon::Medium,
            "chief-core:pack-interop-e2e",
            work_body,
        ))
        .expect("put work object");

    for source in [&contract_uri, &calendar_uri, &prior_email_uri] {
        mem.put_edge(Edge {
            from: work_uri.clone(),
            to: source.to_string(),
            kind: EdgeKind::DerivedFrom,
            weight: 1.0,
            created_by: "chief-core:pack-interop-e2e".into(),
        })
        .expect("put source edge");
    }

    work_uri
}

fn assert_no_forbidden_pack_imports(path: &str) {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root");
    let content = std::fs::read_to_string(root.join(path)).expect("read pack source");
    for forbidden in [
        "chief_core",
        "chief_mem",
        "document_pack",
        "calendar_pack",
        "email_pack",
    ] {
        assert!(
            !content.contains(forbidden),
            "{path} must not import or mention forbidden symbol {forbidden}"
        );
    }
}
