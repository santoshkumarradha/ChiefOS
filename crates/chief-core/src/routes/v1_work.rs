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
    pub contributions: Vec<Value>,
    pub provenance: Vec<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkSourceRef {
    pub uri: String,
    pub node_type: String,
    pub source: String,
    pub summary: String,
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
    let contributions = body
        .get("contributions")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();

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
