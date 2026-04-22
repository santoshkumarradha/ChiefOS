//! Axum route handlers for HTTP API.

use crate::brief::assemble_brief;
use crate::broker::CapabilityDenied;
use crate::capability::{PrincipalId, RequestedOp};
use crate::dispatch::dispatch_intent;
use crate::state::{AppState, BusEvent};
use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json, Router,
};
use chief_oauth::FlowId;
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

// OAuth request/response types
#[derive(Debug, Serialize, Deserialize)]
pub struct OAuthStartRequest {
    pub provider: String,
    pub scopes: Vec<String>,
    pub redirect_uri: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OAuthCompleteRequest {
    pub flow_id: String,
    pub code: String,
    pub state: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OAuthProxyRequest {
    pub session_id: String,
    pub url: String,
    pub method: String,
    pub headers: std::collections::HashMap<String, String>,
    pub body: Option<Vec<u8>>,
}

pub fn router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/oauth/start", axum::routing::post(oauth_start_handler))
        .route(
            "/oauth/complete",
            axum::routing::post(oauth_complete_handler),
        )
        .route("/oauth/proxy", axum::routing::post(oauth_proxy_handler))
        .route("/intent", axum::routing::post(intent_handler))
        .route("/brief", axum::routing::get(brief_handler))
        .route("/approve", axum::routing::post(approve_handler))
        .route("/verify", axum::routing::post(verify_handler))
        .route("/rewind", axum::routing::post(rewind_handler))
        .route("/status", axum::routing::get(status_handler))
        .with_state(state)
}

pub async fn intent_handler(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    Json(req): Json<IntentRequest>,
) -> impl IntoResponse {
    info!("intent: {}", req.text);

    if let Err(denied) = state
        .broker
        .check(
            &principal_for_request(&headers, &state),
            &RequestedOp::agent_spawn("intent_task", 1),
        )
        .await
    {
        return capability_denied_response(denied);
    }

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
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    Json(req): Json<ApproveRequest>,
) -> impl IntoResponse {
    info!(
        "approve: card_id={}, ceremony={}",
        req.card_id, req.ceremony
    );

    if let Err(denied) = state
        .broker
        .check(
            &principal_for_request(&headers, &state),
            &RequestedOp::ceremony_request("approval"),
        )
        .await
    {
        return capability_denied_response(denied);
    }

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
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    Json(req): Json<RewindRequest>,
) -> impl IntoResponse {
    info!("rewind: duration={}", req.duration);

    if let Err(denied) = state
        .broker
        .check(
            &principal_for_request(&headers, &state),
            &RequestedOp::ledger_read("rewind"),
        )
        .await
    {
        return capability_denied_response(denied);
    }

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

// OAuth handlers
pub async fn oauth_start_handler(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    Json(req): Json<OAuthStartRequest>,
) -> impl IntoResponse {
    let principal = principal_for_request(&headers, &state);

    // Check capability if not in dev mode
    if !state.dev_mode {
        if let Err(denied) = state
            .broker
            .check(
                &principal,
                &RequestedOp::net_oauth2(&req.provider, req.scopes.clone()),
            )
            .await
        {
            return capability_denied_response(denied);
        }
    }

    // Parse provider from string
    let provider = match req.provider.as_str() {
        "google" => chief_oauth::Provider::Google,
        "github" => chief_oauth::Provider::Github,
        _ => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "error": "unsupported_provider",
                    "provider": req.provider
                })),
            )
                .into_response();
        }
    };

    match state
        .oauth_broker
        .start_flow(provider, &req.scopes, &req.redirect_uri)
        .await
    {
        Ok(challenge) => {
            let flow_id_str = challenge.flow_id.0.clone();
            info!(
                principal = %principal,
                provider = %req.provider,
                flow_id = %flow_id_str,
                scopes_count = req.scopes.len(),
                "oauth flow started"
            );
            Json(challenge).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "error": "flow_error",
                "details": e.to_string()
            })),
        )
            .into_response(),
    }
}

pub async fn oauth_complete_handler(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    Json(req): Json<OAuthCompleteRequest>,
) -> impl IntoResponse {
    let principal = principal_for_request(&headers, &state);

    let flow_id = FlowId(req.flow_id.clone());
    match state
        .oauth_broker
        .complete_flow(flow_id, &req.code, &req.state)
        .await
    {
        Ok(session_handle) => {
            info!(
                principal = %principal,
                session_id = %session_handle.id,
                provider = %session_handle.provider.name(),
                scopes_count = session_handle.scopes.len(),
                "oauth flow completed"
            );
            Json(session_handle).into_response()
        }
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "completion_error",
                "details": e.to_string()
            })),
        )
            .into_response(),
    }
}

pub async fn oauth_proxy_handler(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    Json(req): Json<OAuthProxyRequest>,
) -> impl IntoResponse {
    let principal = principal_for_request(&headers, &state);

    // Find session
    let sessions = match state.oauth_broker.list().await {
        Ok(sessions) => sessions,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "session_list_error",
                    "details": e.to_string()
                })),
            )
                .into_response();
        }
    };

    let session_meta = match sessions.iter().find(|s| s.handle.id == req.session_id) {
        Some(sm) => sm,
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({
                    "error": "session_not_found",
                    "session_id": req.session_id
                })),
            )
                .into_response();
        }
    };

    // Check capability for the session's provider
    let provider_key = session_meta.handle.provider.name();
    if !state.dev_mode {
        if let Err(denied) = state
            .broker
            .check(&principal, &RequestedOp::net_oauth2(&provider_key, vec![]))
            .await
        {
            return capability_denied_response(denied);
        }
    }

    // Prepare proxy request
    let proxy_req = chief_oauth::ProxyRequest {
        url: req.url.clone(),
        method: req.method.clone(),
        headers: req.headers.clone(),
        body: req.body.clone(),
    };

    match state
        .oauth_broker
        .proxy_request(&session_meta.handle, proxy_req)
        .await
    {
        Ok(resp) => {
            info!(
                principal = %principal,
                session_id = %req.session_id,
                url_host = %req.url,
                status = resp.status,
                "oauth proxy request completed"
            );
            Json(serde_json::json!({
                "status": resp.status,
                "headers": resp.headers,
                "body_size": resp.body.len()
            }))
            .into_response()
        }
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "proxy_error",
                "details": e.to_string()
            })),
        )
            .into_response(),
    }
}

fn principal_for_request(headers: &HeaderMap, state: &AppState) -> PrincipalId {
    if state.dev_mode {
        return PrincipalId::from("dev");
    }

    headers
        .get("x-chief-principal")
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(PrincipalId::from)
        .unwrap_or_else(|| PrincipalId::from("anonymous"))
}

fn capability_denied_response(denied: CapabilityDenied) -> axum::response::Response {
    (
        StatusCode::FORBIDDEN,
        Json(serde_json::json!({
            "error": "capability_denied",
            "kind": denied.kind(),
            "reason": denied.reason().to_string(),
        })),
    )
        .into_response()
}
