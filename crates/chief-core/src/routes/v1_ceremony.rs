//! Ceremony HTTP endpoints.
//!
//! * `GET  /v1/ceremony`              — list pending ceremonies
//! * `GET  /v1/ceremony/{id}`         — fetch a ceremony by id
//! * `POST /v1/ceremony/{id}/approve` — approve (requires held_ms >= 3000)
//! * `POST /v1/ceremony/{id}/deny`    — deny

use crate::ceremony::{CeremonyError, CeremonyItem, CeremonyStatus};
use crate::state::AppState;
use crate::trust_ledger::LedgerDelta;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::info;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/ceremony", get(list_handler))
        .route("/ceremony/:id", get(detail_handler))
        .route("/ceremony/:id/approve", post(approve_handler))
        .route("/ceremony/:id/deny", post(deny_handler))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApproveRequest {
    pub held_ms: u64,
    #[serde(default)]
    pub payload_hash: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ApproveResponse {
    pub status: &'static str,
    pub ceremony: CeremonyItem,
}

#[derive(Debug, Serialize)]
pub struct DenyResponse {
    pub status: &'static str,
    pub ceremony: CeremonyItem,
}

async fn list_handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let items = state.ceremonies.list_pending().await;
    Json(items)
}

async fn detail_handler(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match state.ceremonies.get(&id).await {
        Some(item) => Json(item).into_response(),
        None => not_found(&id),
    }
}

async fn approve_handler(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(req): Json<ApproveRequest>,
) -> impl IntoResponse {
    info!(ceremony_id = %id, held_ms = req.held_ms, "ceremony approve requested");

    match state
        .ceremonies
        .approve(&id, req.held_ms, req.payload_hash.as_deref(), &state.broker)
        .await
    {
        Ok(ceremony) => {
            // Trust ledger: approving a ceremony is a strong positive signal
            // for the requesting agent's primary category. We use a coarse
            // category derived from the ceremony title prefix — the demo can
            // refine this later.
            let category = derive_category(&ceremony);
            state
                .trust_ledger
                .record(category, LedgerDelta::Approval)
                .await;

            Json(ApproveResponse {
                status: "approved",
                ceremony,
            })
            .into_response()
        }
        Err(CeremonyError::HoldTooShort {
            held_ms,
            required_ms,
        }) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "hold_too_short",
                "held_ms": held_ms,
                "required_ms": required_ms,
            })),
        )
            .into_response(),
        Err(CeremonyError::NotFound(_)) => not_found(&id),
        Err(CeremonyError::AlreadyResolved(s)) => (
            StatusCode::CONFLICT,
            Json(serde_json::json!({
                "error": "already_resolved",
                "status": status_label(s),
            })),
        )
            .into_response(),
        Err(CeremonyError::Expired) => (
            StatusCode::GONE,
            Json(serde_json::json!({
                "error": "expired",
            })),
        )
            .into_response(),
        Err(CeremonyError::PayloadHashMismatch { expected, actual }) => (
            StatusCode::CONFLICT,
            Json(serde_json::json!({
                "error": "payload_hash_mismatch",
                "expected": expected,
                "actual": actual,
            })),
        )
            .into_response(),
        Err(CeremonyError::GrantIssueFailed(msg)) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "error": "grant_issue_failed",
                "details": msg,
            })),
        )
            .into_response(),
    }
}

async fn deny_handler(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match state.ceremonies.deny(&id).await {
        Ok(ceremony) => {
            let category = derive_category(&ceremony);
            state
                .trust_ledger
                .record(category, LedgerDelta::Denial)
                .await;
            Json(DenyResponse {
                status: "denied",
                ceremony,
            })
            .into_response()
        }
        Err(CeremonyError::NotFound(_)) => not_found(&id),
        Err(CeremonyError::AlreadyResolved(s)) => (
            StatusCode::CONFLICT,
            Json(serde_json::json!({
                "error": "already_resolved",
                "status": status_label(s),
            })),
        )
            .into_response(),
        Err(other) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "error": other.to_string(),
            })),
        )
            .into_response(),
    }
}

fn not_found(id: &str) -> axum::response::Response {
    (
        StatusCode::NOT_FOUND,
        Json(serde_json::json!({
            "error": "ceremony_not_found",
            "id": id,
        })),
    )
        .into_response()
}

fn status_label(s: CeremonyStatus) -> &'static str {
    match s {
        CeremonyStatus::Pending => "pending",
        CeremonyStatus::Approved => "approved",
        CeremonyStatus::Denied => "denied",
        CeremonyStatus::Expired => "expired",
    }
}

/// Use the ceremony's source_agent name as a trust category. This is a
/// pragmatic default — the real system will tag ceremonies with an explicit
/// category via `CapabilityKind`.
fn derive_category(ceremony: &CeremonyItem) -> String {
    // Source-agent names look like `hn-briefer`, `inbox-briefer`, etc.
    // Use the first segment (before the first `-`) as the category so
    // counts aggregate sensibly.
    ceremony
        .source_agent
        .split('-')
        .next()
        .unwrap_or(&ceremony.source_agent)
        .to_string()
}
