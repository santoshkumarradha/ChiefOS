//! Axum route handlers for HTTP API.

use crate::brief::assemble_brief;
use crate::dispatch::dispatch_intent;
use crate::state::{AppState, BusEvent};
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json, Router};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct IntentRequest {
    pub text: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IntentResponse {
    pub intent_id: String,
    pub cards_queued: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BriefResponse {
    pub date: String,
    pub needs_you: Vec<String>,
    pub handled: Vec<String>,
    pub trust_ledger: std::collections::HashMap<String, u8>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApproveRequest {
    pub card_id: String,
    pub ceremony: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApproveResponse {
    pub shipped: bool,
    pub attestation_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VerifyRequest {
    pub tier: String,
    pub ok: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RewindRequest {
    pub duration: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RewindResponse {
    pub reverted_count: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StatusResponse {
    pub services: std::collections::HashMap<String, String>,
    pub uptime_s: i64,
    pub version: String,
}

pub fn router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/intent", axum::routing::post(intent_handler))
        .route("/brief", axum::routing::get(brief_handler))
        .route("/approve", axum::routing::post(approve_handler))
        .route("/verify", axum::routing::post(verify_handler))
        .route("/rewind", axum::routing::post(rewind_handler))
        .route("/status", axum::routing::get(status_handler))
        .with_state(state)
}

pub async fn intent_handler(
    State(state): State<Arc<AppState>>,
    Json(req): Json<IntentRequest>,
) -> impl IntoResponse {
    info!("intent: {}", req.text);

    let intent_id = state.next_id("intent");
    let cards = match dispatch_intent(&req.text, &intent_id, &state).await {
        Ok(c) => c,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": e.to_string()
                })),
            )
                .into_response();
        }
    };

    let count = cards.len();

    {
        let mut queue = state.queued_cards.lock().await;
        for card in cards {
            let _ = state.event_tx.send(BusEvent::CardQueued {
                card_id: card.id.clone(),
            });
            queue.push_back(card);
        }
    }

    Json(IntentResponse {
        intent_id,
        cards_queued: count,
    })
    .into_response()
}

pub async fn brief_handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let brief = assemble_brief(&state).await;
    Json(brief).into_response()
}

pub async fn approve_handler(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ApproveRequest>,
) -> impl IntoResponse {
    info!(
        "approve: card_id={}, ceremony={}",
        req.card_id, req.ceremony
    );

    let card = {
        let mut queue = state.queued_cards.lock().await;
        queue
            .iter()
            .position(|c| c.id == req.card_id)
            .and_then(|i| queue.remove(i))
    };

    let card = match card {
        Some(c) => c,
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({
                    "error": "card not found"
                })),
            )
                .into_response();
        }
    };

    let attestation_id = Uuid::new_v4().to_string();

    {
        let mut handled = state.handled_cards.lock().await;
        let mut approved_card = card.clone();
        approved_card.approved_at = Some(chrono::Utc::now());
        handled.push(approved_card);
    }

    let _ = state.event_tx.send(BusEvent::CardApproved {
        card_id: card.id.clone(),
        attestation_id: attestation_id.clone(),
    });

    Json(ApproveResponse {
        shipped: true,
        attestation_id,
    })
    .into_response()
}

pub async fn verify_handler(
    State(_state): State<Arc<AppState>>,
    Json(_req): Json<VerifyRequest>,
) -> impl IntoResponse {
    Json(serde_json::json!({
        "tier": "Generated",
        "ok": true
    }))
    .into_response()
}

pub async fn rewind_handler(
    State(state): State<Arc<AppState>>,
    Json(req): Json<RewindRequest>,
) -> impl IntoResponse {
    info!("rewind: duration={}", req.duration);

    let mut handled = state.handled_cards.lock().await;
    let reverted_count = handled.len();

    for card in handled.drain(..) {
        let _ = state
            .event_tx
            .send(BusEvent::ActionReverted { card_id: card.id });
    }

    Json(RewindResponse { reverted_count }).into_response()
}

pub async fn status_handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let mut services = std::collections::HashMap::new();
    services.insert("memory".to_string(), "ok".to_string());
    services.insert("event_log".to_string(), "ok".to_string());
    services.insert("harness".to_string(), "ok".to_string());
    services.insert("router".to_string(), "ok".to_string());
    services.insert("inference".to_string(), "ok".to_string());

    Json(StatusResponse {
        services,
        uptime_s: state.uptime_secs(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
    .into_response()
}
