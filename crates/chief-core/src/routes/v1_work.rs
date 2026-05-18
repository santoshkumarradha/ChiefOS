//! `GET /v1/work/:id` — projection over a Memory Graph Work Object aggregate.
//!
//! A Work Object is product language for an existing `mem://artifact/...` node
//! with `body.kind = "work_object"`. This route does not introduce a new
//! storage namespace or kernel primitive.

use crate::{
    capability::{CapabilityKind, Grant, RequestedOp},
    ceremony::{CeremonyEvidence, CeremonyItem, NewCeremony},
    inbox::{BadgeVariant, InboxKind, NewInboxItem},
    routes::legacy::{capability_denied_response, principal_for_request},
    state::AppState,
};
use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use chief_event_log_proto::schema::Event;
use chief_mem::{EdgeKind, Horizon, Node, NodeType};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::str::FromStr;
use std::sync::Arc;
use tracing::warn;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/work", post(create_work_handler))
        .route("/work/:id", get(handler))
        .route("/work/:id/contributions", post(contribution_handler))
        .route("/work/:id/provenance", get(provenance_handler))
        .route("/work/:id/rewind", post(rewind_handler))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkObjectResponse {
    pub id: String,
    pub uri: String,
    pub title: String,
    pub source_refs: Vec<WorkSourceRef>,
    pub contributions: Vec<WorkContribution>,
    pub provenance: Vec<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkSourceRef {
    pub uri: String,
    pub node_type: String,
    pub source: String,
    pub summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkContribution {
    pub uri: String,
    pub node_type: String,
    pub source: String,
    pub pack: String,
    pub kind: String,
    pub title: String,
    pub summary: String,
    pub authority_state: String,
    pub source_refs: Vec<String>,
    pub body: Value,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateWorkObjectRequest {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub summary: Option<String>,
    #[serde(default)]
    pub body: Value,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RewindRequest {
    pub contribution_uri: String,
    #[serde(default)]
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ContributionWriteRequest {
    #[serde(default = "default_contribution_node_type")]
    pub node_type: String,
    pub kind: String,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub summary: Option<String>,
    #[serde(default)]
    pub authority_state: Option<String>,
    #[serde(default)]
    pub source_refs: Vec<String>,
    #[serde(default)]
    pub body: Value,
    #[serde(default)]
    pub ceremony: Option<ContributionCeremonyRequest>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ContributionCeremonyRequest {
    pub title: String,
    pub summary: String,
    pub category: String,
    pub payload: Value,
    #[serde(default)]
    pub trust_context: Option<u8>,
    #[serde(default)]
    pub target_principal: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ContributionWriteResponse {
    pub work_object_id: String,
    pub contribution: WorkContribution,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ceremony: Option<CeremonyItem>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RewindResponse {
    pub rewound: bool,
    pub work_object_id: String,
    pub contribution_uri: String,
    pub event_id: String,
    pub stale_marked: usize,
}

async fn create_work_handler(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<CreateWorkObjectRequest>,
) -> Response {
    if req.id.trim().is_empty() || req.title.trim().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "invalid_work_object",
            })),
        )
            .into_response();
    }

    let principal = principal_for_request(&headers, &state);
    if let Err(denied) = state
        .broker
        .check(&principal, &RequestedOp::mem_write("artifact"))
        .await
    {
        return capability_denied_response(denied);
    }

    match create_work_object(&state, &headers, req).await {
        Ok(work) => (StatusCode::CREATED, Json(work)).into_response(),
        Err(err) => {
            warn!(error = %err, "work object create failed");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "work_object_create_failed",
                })),
            )
                .into_response()
        }
    }
}

async fn contribution_handler(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<ContributionWriteRequest>,
) -> Response {
    let node_type = match contribution_node_type(&req.node_type) {
        Ok(node_type) => node_type,
        Err(response) => return response,
    };

    let principal = principal_for_request(&headers, &state);
    if let Err(denied) = state
        .broker
        .check(&principal, &RequestedOp::mem_write(node_type.to_string()))
        .await
    {
        return capability_denied_response(denied);
    }

    match write_contribution(&state, &id, &headers, node_type, req).await {
        Ok(Some(response)) => (StatusCode::CREATED, Json(response)).into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "error": "work_object_not_found",
                "id": id,
            })),
        )
            .into_response(),
        Err(err) => {
            warn!(error = %err, id = %id, "work object contribution write failed");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "work_object_contribution_write_failed",
                })),
            )
                .into_response()
        }
    }
}

async fn handler(Path(id): Path<String>, State(state): State<Arc<AppState>>) -> Response {
    match load_work_object(&state, &id).await {
        Ok(Some(work)) => Json(work).into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "error": "work_object_not_found",
                "id": id,
            })),
        )
            .into_response(),
        Err(err) => {
            warn!(error = %err, id = %id, "work object projection failed");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "work_object_projection_failed",
                })),
            )
                .into_response()
        }
    }
}

async fn provenance_handler(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Response {
    match load_work_object(&state, &id).await {
        Ok(Some(work)) => Json(work.provenance).into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "error": "work_object_not_found",
                "id": id,
            })),
        )
            .into_response(),
        Err(err) => {
            warn!(error = %err, id = %id, "work object provenance failed");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "work_object_provenance_failed",
                })),
            )
                .into_response()
        }
    }
}

async fn rewind_handler(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
    Json(req): Json<RewindRequest>,
) -> Response {
    match rewind_contribution(&state, &id, req).await {
        Ok(Some(response)) => Json(response).into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "error": "contribution_not_found",
                "id": id,
            })),
        )
            .into_response(),
        Err(err) => {
            warn!(error = %err, id = %id, "work object rewind failed");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "work_object_rewind_failed",
                })),
            )
                .into_response()
        }
    }
}

async fn load_work_object(
    state: &AppState,
    id: &str,
) -> anyhow::Result<Option<WorkObjectResponse>> {
    let mem = state.mem.lock().await;
    let artifacts = mem.nodes_by_type(NodeType::Artifact)?;

    let Some(work) = artifacts.into_iter().find_map(|stored| {
        let body: Value = serde_json::from_str(&stored.node.body).ok()?;
        let matches = body.get("kind").and_then(Value::as_str) == Some("work_object")
            && body.get("id").and_then(Value::as_str) == Some(id);
        matches.then_some((stored.uri, stored.node, body))
    }) else {
        return Ok(None);
    };

    let (uri, node, body) = work;
    let title = body
        .get("title")
        .and_then(Value::as_str)
        .unwrap_or(id)
        .to_string();
    let mut contributions: Vec<WorkContribution> = body
        .get("contributions")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .enumerate()
        .map(|(idx, contribution)| inline_contribution(idx, contribution))
        .collect();
    contributions.extend(load_contributions(&mem, id)?);

    let mut source_refs = Vec::new();
    for edge in mem.get_edges_from(&uri)? {
        if edge.kind != EdgeKind::DerivedFrom {
            continue;
        }
        if let Some(source_node) = mem.get_node(&edge.to)? {
            source_refs.push(source_ref(edge.to, source_node));
        }
    }

    let provenance = provenance_rows(&mem, id, &uri, &node.source, &contributions)?;

    Ok(Some(WorkObjectResponse {
        id: id.to_string(),
        uri,
        title,
        source_refs,
        contributions,
        provenance,
    }))
}

async fn write_contribution(
    state: &AppState,
    id: &str,
    headers: &HeaderMap,
    node_type: NodeType,
    req: ContributionWriteRequest,
) -> anyhow::Result<Option<ContributionWriteResponse>> {
    let work_exists = {
        let mem = state.mem.lock().await;
        let artifacts = mem.nodes_by_type(NodeType::Artifact)?;
        artifacts.into_iter().any(|stored| {
            serde_json::from_str::<Value>(&stored.node.body)
                .ok()
                .is_some_and(|body| {
                    body.get("kind").and_then(Value::as_str) == Some("work_object")
                        && body.get("id").and_then(Value::as_str) == Some(id)
                })
        })
    };

    if !work_exists {
        return Ok(None);
    }

    let mut body = match req.body {
        Value::Null => Map::new(),
        Value::Object(map) => map,
        other => {
            let mut map = Map::new();
            map.insert("content".into(), other);
            map
        }
    };

    body.insert("kind".into(), Value::String(req.kind));
    body.insert("work_object_id".into(), Value::String(id.to_string()));
    if let Some(title) = req.title {
        body.insert("title".into(), Value::String(title));
    }
    if let Some(summary) = req.summary {
        body.insert("summary".into(), Value::String(summary));
    }
    if let Some(authority_state) = req.authority_state {
        body.insert("authority_state".into(), Value::String(authority_state));
    }
    if !req.source_refs.is_empty() {
        body.insert(
            "source_refs".into(),
            Value::Array(req.source_refs.into_iter().map(Value::String).collect()),
        );
    }

    let source = contribution_source(headers, state);
    let ceremony = if let Some(ceremony_req) = req.ceremony {
        let ceremony = open_contribution_ceremony(state, id, &source, &ceremony_req).await;
        body.insert("requires_ceremony".into(), Value::Bool(true));
        body.insert(
            "authority_state".into(),
            Value::String("needs_ceremony".into()),
        );
        body.insert("ceremony_id".into(), Value::String(ceremony.id.clone()));
        if let Some(payload_hash) = ceremony.payload_hash.clone() {
            body.insert("payload_hash".into(), Value::String(payload_hash));
        }
        Some(ceremony)
    } else {
        None
    };

    let body = Value::Object(body);
    let mem = state.mem.lock().await;
    let uri = mem.put_node(Node::new(
        node_type,
        Horizon::Medium,
        source,
        body.to_string(),
    ))?;
    let Some(stored) = mem.get_node(&uri)? else {
        anyhow::bail!("contribution node was not readable after write");
    };
    let contribution = stored_contribution(uri, stored, body);

    Ok(Some(ContributionWriteResponse {
        work_object_id: id.to_string(),
        contribution,
        ceremony,
    }))
}

async fn create_work_object(
    state: &AppState,
    headers: &HeaderMap,
    req: CreateWorkObjectRequest,
) -> anyhow::Result<WorkObjectResponse> {
    let source = contribution_source(headers, state);
    let mut body = match req.body {
        Value::Null => Map::new(),
        Value::Object(map) => map,
        other => {
            let mut map = Map::new();
            map.insert("content".into(), other);
            map
        }
    };
    body.insert("kind".into(), Value::String("work_object".into()));
    body.insert("id".into(), Value::String(req.id.clone()));
    body.insert("title".into(), Value::String(req.title));
    if let Some(summary) = req.summary {
        body.insert("summary".into(), Value::String(summary));
    }

    let mem = state.mem.lock().await;
    mem.put_node(Node::new(
        NodeType::Artifact,
        Horizon::Medium,
        source,
        Value::Object(body).to_string(),
    ))?;
    drop(mem);

    load_work_object(state, &req.id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("created work object not readable"))
}

async fn rewind_contribution(
    state: &AppState,
    id: &str,
    req: RewindRequest,
) -> anyhow::Result<Option<RewindResponse>> {
    let mem = state.mem.lock().await;
    let contributions = load_contributions(&mem, id)?;
    let Some(target) = contributions
        .into_iter()
        .find(|node| node.uri == req.contribution_uri)
    else {
        return Ok(None);
    };

    if !mem.tombstone_node(&target.uri)? {
        return Ok(None);
    }

    let reason = req.reason.unwrap_or_else(|| "user rewind".into());
    let target_uri = target.uri.clone();
    let payload = serde_json::json!({
        "work_object_id": id,
        "contribution_uri": target_uri,
        "reason": reason,
    });
    let payload_hash = *blake3::hash(payload.to_string().as_bytes()).as_bytes();
    let event_id = state.event_log.append(Event::UIAction {
        surface: "work-object".into(),
        action: "rewind".into(),
        payload_hash,
    })?;

    let stale_marked = mark_downstream_stale(&mem, id, &target_uri)?;
    mem.put_node(Node::new(
        NodeType::Decision,
        Horizon::Medium,
        "chief-core:work-rewind",
        serde_json::json!({
            "kind": "rewind_event",
            "work_object_id": id,
            "target_uri": target_uri,
            "target_kind": target.kind,
            "target_pack": target.pack,
            "target_body": target.body,
            "reason": reason,
            "event_id": event_id.to_hex(),
            "stale_marked": stale_marked
        })
        .to_string(),
    ))?;

    Ok(Some(RewindResponse {
        rewound: true,
        work_object_id: id.to_string(),
        contribution_uri: req.contribution_uri,
        event_id: event_id.to_hex(),
        stale_marked,
    }))
}

fn default_contribution_node_type() -> String {
    "finding".into()
}

fn contribution_node_type(value: &str) -> Result<NodeType, Response> {
    let node_type = NodeType::from_str(value).map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "invalid_contribution_node_type",
                "allowed": ["finding", "artifact", "decision"],
            })),
        )
            .into_response()
    })?;

    if matches!(
        node_type,
        NodeType::Finding | NodeType::Artifact | NodeType::Decision
    ) {
        Ok(node_type)
    } else {
        Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "invalid_contribution_node_type",
                "allowed": ["finding", "artifact", "decision"],
            })),
        )
            .into_response())
    }
}

fn contribution_source(headers: &HeaderMap, state: &AppState) -> String {
    headers
        .get("x-chief-principal")
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| principal_for_request(headers, state).to_string())
}

async fn open_contribution_ceremony(
    state: &AppState,
    work_object_id: &str,
    source: &str,
    req: &ContributionCeremonyRequest,
) -> CeremonyItem {
    let payload_hash = payload_hash(&req.payload);
    let target_principal = req
        .target_principal
        .clone()
        .unwrap_or_else(|| source.to_string());
    let ceremony = state
        .ceremonies
        .open(NewCeremony {
            title: req.title.clone(),
            evidence: CeremonyEvidence {
                summary: req.summary.clone(),
                details: serde_json::json!({
                    "work_object_id": work_object_id,
                    "source": source,
                    "category": req.category,
                    "payload": req.payload,
                    "payload_hash": payload_hash,
                }),
            },
            source_agent: source.to_string(),
            trust_context: req.trust_context.unwrap_or(3),
            proposed_grant: Grant::new(vec![CapabilityKind::ceremony_request(
                vec![req.category.clone()],
                format!(
                    "Permit committing the exact {} payload approved in Ceremony.",
                    req.category
                ),
            )]),
            payload_hash: Some(payload_hash.clone()),
            target_principal,
            rollback_window: chrono::Duration::hours(24),
            ceremony_ttl: chrono::Duration::hours(12),
        })
        .await;

    state
        .inbox
        .append(NewInboxItem {
            kind: InboxKind::CeremonyPending,
            title: req.title.clone(),
            snippet: req.summary.clone(),
            source_agent: source.to_string(),
            badge: BadgeVariant::Ceremony,
            ceremony_id: Some(ceremony.id.clone()),
        })
        .await;
    ceremony
}

fn payload_hash(value: &Value) -> String {
    let bytes = serde_json::to_vec(value).expect("payload json");
    format!("blake3:{}", blake3::hash(&bytes).to_hex())
}

fn source_ref(uri: String, node: Node) -> WorkSourceRef {
    WorkSourceRef {
        uri,
        node_type: node.node_type.to_string(),
        source: node.source,
        summary: summarize_body(&node.body),
    }
}

fn load_contributions(
    mem: &chief_mem::ChiefMem,
    id: &str,
) -> anyhow::Result<Vec<WorkContribution>> {
    let mut out = Vec::new();
    for node_type in [NodeType::Finding, NodeType::Artifact, NodeType::Decision] {
        for stored in mem.nodes_by_type(node_type)? {
            let Ok(body) = serde_json::from_str::<Value>(&stored.node.body) else {
                continue;
            };
            if body.get("kind").and_then(Value::as_str) == Some("work_object") {
                continue;
            }
            if matches!(
                body.get("kind").and_then(Value::as_str),
                Some("rewind_event" | "stale_marker")
            ) {
                continue;
            }
            if body.get("work_object_id").and_then(Value::as_str) != Some(id) {
                continue;
            }
            out.push(stored_contribution(stored.uri, stored.node, body));
        }
    }
    Ok(out)
}

fn provenance_rows(
    mem: &chief_mem::ChiefMem,
    id: &str,
    work_uri: &str,
    work_source: &str,
    contributions: &[WorkContribution],
) -> anyhow::Result<Vec<Value>> {
    let mut rows = vec![serde_json::json!({
        "kind": "memory_node",
        "uri": work_uri,
        "source": work_source,
    })];

    for contribution in contributions {
        rows.push(serde_json::json!({
            "kind": "contribution_written",
            "uri": contribution.uri,
            "source": contribution.source,
            "pack": contribution.pack,
            "contribution_kind": contribution.kind,
            "authority_state": contribution.authority_state,
        }));
    }

    for stored in mem.nodes_by_type(NodeType::Decision)? {
        let Ok(body) = serde_json::from_str::<Value>(&stored.node.body) else {
            continue;
        };
        if body.get("work_object_id").and_then(Value::as_str) != Some(id) {
            continue;
        }
        if !matches!(
            body.get("kind").and_then(Value::as_str),
            Some("rewind_event" | "stale_marker")
        ) {
            continue;
        }
        rows.push(serde_json::json!({
            "kind": body.get("kind").and_then(Value::as_str).unwrap_or("decision"),
            "uri": stored.uri,
            "source": stored.node.source,
            "body": body,
        }));
    }

    Ok(rows)
}

fn mark_downstream_stale(
    mem: &chief_mem::ChiefMem,
    id: &str,
    target_uri: &str,
) -> anyhow::Result<usize> {
    let mut marked = 0;
    for contribution in load_contributions(mem, id)? {
        if !contribution
            .source_refs
            .iter()
            .any(|source_ref| source_ref == target_uri)
        {
            continue;
        }
        mem.put_node(Node::new(
            NodeType::Decision,
            Horizon::Medium,
            "chief-core:work-rewind",
            serde_json::json!({
                "kind": "stale_marker",
                "work_object_id": id,
                "target_uri": contribution.uri,
                "stale_because": target_uri,
            })
            .to_string(),
        ))?;
        marked += 1;
    }
    Ok(marked)
}

fn inline_contribution(_idx: usize, body: Value) -> WorkContribution {
    let kind = body
        .get("kind")
        .and_then(Value::as_str)
        .unwrap_or("inline")
        .to_string();
    let title = contribution_title(&kind, &body);
    let uri = body
        .get("uri")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    WorkContribution {
        uri,
        node_type: "artifact".into(),
        source: "work-object:inline".into(),
        pack: "work-object".into(),
        kind,
        title,
        summary: contribution_summary(&body),
        authority_state: authority_state(&body),
        source_refs: source_refs(&body),
        body,
    }
}

fn stored_contribution(uri: String, node: Node, body: Value) -> WorkContribution {
    let kind = body
        .get("kind")
        .and_then(Value::as_str)
        .unwrap_or("unknown")
        .to_string();
    let title = contribution_title(&kind, &body);
    WorkContribution {
        uri,
        node_type: node.node_type.to_string(),
        pack: pack_name(&node.source),
        source: node.source,
        kind,
        title,
        summary: contribution_summary(&body),
        authority_state: authority_state(&body),
        source_refs: source_refs(&body),
        body,
    }
}

fn pack_name(source: &str) -> String {
    source
        .strip_prefix("pack:")
        .unwrap_or(source)
        .split('/')
        .next()
        .unwrap_or(source)
        .to_string()
}

fn contribution_title(kind: &str, body: &Value) -> String {
    if let Some(title) = body.get("title").and_then(Value::as_str) {
        return title.to_string();
    }
    match kind {
        "obligation" => format!(
            "Obligation {}",
            body.get("ordinal")
                .and_then(Value::as_u64)
                .map(|n| n.to_string())
                .unwrap_or_else(|| "found".into())
        ),
        "candidate_slot" => format!(
            "Candidate slot {}",
            body.get("ordinal")
                .and_then(Value::as_u64)
                .map(|n| n.to_string())
                .unwrap_or_else(|| "found".into())
        ),
        "draft_reply" => body
            .get("subject")
            .and_then(Value::as_str)
            .unwrap_or("Draft reply")
            .to_string(),
        other => other.replace('_', " "),
    }
}

fn contribution_summary(body: &Value) -> String {
    for key in [
        "content",
        "rationale",
        "summary",
        "subject",
        "recipient",
        "body",
    ] {
        if let Some(text) = body.get(key).and_then(Value::as_str) {
            return text.chars().take(220).collect();
        }
    }
    body.to_string().chars().take(220).collect()
}

fn authority_state(body: &Value) -> String {
    if let Some(status) = body.get("authority_state").and_then(Value::as_str) {
        return status.to_string();
    }
    if body.get("shipped").and_then(Value::as_bool) == Some(true) {
        return "shipped".into();
    }
    if body.get("requires_ceremony").and_then(Value::as_bool) == Some(true) {
        return "needs_ceremony".into();
    }
    "handled".into()
}

fn source_refs(body: &Value) -> Vec<String> {
    body.get("source_refs")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(ToOwned::to_owned)
        .collect()
}

fn summarize_body(body: &str) -> String {
    let Ok(value) = serde_json::from_str::<Value>(body) else {
        return body.chars().take(120).collect();
    };

    for key in ["path", "subject", "title", "id"] {
        if let Some(text) = value.get(key).and_then(Value::as_str) {
            return text.to_string();
        }
    }

    value.to_string().chars().take(120).collect()
}
