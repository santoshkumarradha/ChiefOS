//! `GET /v1/work/:id` — projection over a Memory Graph Work Object aggregate.
//!
//! A Work Object is product language for an existing `mem://artifact/...` node
//! with `body.kind = "work_object"`. This route does not introduce a new
//! storage namespace or kernel primitive.

use crate::state::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use chief_mem::{EdgeKind, Node, NodeType};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use tracing::warn;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new().route("/work/:id", get(handler))
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

    let provenance = vec![serde_json::json!({
        "kind": "memory_node",
        "uri": uri,
        "source": node.source,
    })];

    Ok(Some(WorkObjectResponse {
        id: id.to_string(),
        uri: provenance[0]["uri"]
            .as_str()
            .unwrap_or_default()
            .to_string(),
        title,
        source_refs,
        contributions,
        provenance,
    }))
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
            if body.get("work_object_id").and_then(Value::as_str) != Some(id) {
                continue;
            }
            out.push(stored_contribution(stored.uri, stored.node, body));
        }
    }
    Ok(out)
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
