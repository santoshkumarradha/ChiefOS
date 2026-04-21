use axum::body::to_bytes;
use axum::extract::State;
use axum::http::{HeaderMap, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use chief_core::broker::DenialReason;
use chief_core::capability::{CapabilityKind, Grant, HttpMethod, PrincipalId, RequestedOp};
use chief_core::routes::{
    approve_handler, intent_handler, rewind_handler, ApproveRequest, IntentRequest, RewindRequest,
};
use chief_core::state::{AppConfig, BackendKind};
use chief_core::AppState;
use chrono::{Duration, Utc};
use serde_json::Value;
use std::path::Path;
use std::sync::Arc;
use tempfile::{tempdir, TempDir};

struct TestApp {
    _dir: TempDir,
    state: Arc<AppState>,
}

impl TestApp {
    async fn new(dev_mode: bool) -> Self {
        let dir = tempdir().expect("temp state dir");
        let state = state_at(dir.path(), dev_mode).await;
        Self { _dir: dir, state }
    }
}

async fn state_at(path: &Path, dev_mode: bool) -> Arc<AppState> {
    Arc::new(
        AppState::new(AppConfig {
            state_dir: Some(path.to_path_buf()),
            dev_mode,
            backend: BackendKind::Stub,
            model_path: None,
        })
        .await
        .expect("init state"),
    )
}

fn headers(principal: &str) -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(
        "x-chief-principal",
        HeaderValue::from_str(principal).expect("valid principal header"),
    );
    headers
}

fn no_headers() -> HeaderMap {
    HeaderMap::new()
}

async fn intent_response(state: Arc<AppState>, headers: HeaderMap) -> Response {
    intent_handler(
        headers,
        State(state),
        Json(IntentRequest {
            text: "draft an email to alice@example.com".to_string(),
        }),
    )
    .await
    .into_response()
}

async fn approve_response(state: Arc<AppState>, headers: HeaderMap) -> Response {
    approve_handler(
        headers,
        State(state),
        Json(ApproveRequest {
            card_id: "card_missing".to_string(),
            ceremony: true,
        }),
    )
    .await
    .into_response()
}

async fn rewind_response(state: Arc<AppState>, headers: HeaderMap) -> Response {
    rewind_handler(
        headers,
        State(state),
        Json(RewindRequest {
            duration: "4h".to_string(),
        }),
    )
    .await
    .into_response()
}

async fn response_json(response: Response) -> Value {
    let bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("read body");
    serde_json::from_slice(&bytes).expect("json body")
}

fn agent_spawn_grant(pack_id: &str) -> Grant {
    Grant::new(vec![CapabilityKind::agent_spawn(
        vec![pack_id.to_string()],
        1,
        "test agent spawn",
    )])
}

#[tokio::test]
async fn missing_grant_returns_403_no_grant() {
    let app = TestApp::new(false).await;

    let response = intent_response(Arc::clone(&app.state), no_headers()).await;
    assert_eq!(response.status(), StatusCode::FORBIDDEN);

    let body = response_json(response).await;
    assert_eq!(body["error"], "capability_denied");
    assert_eq!(body["kind"], "agent.spawn");
    assert_eq!(body["reason"], "NoGrant");
}

#[tokio::test]
async fn scope_out_of_range_host_allowlist_mismatch_is_403_scope_exceeded() {
    let app = TestApp::new(false).await;
    let principal = PrincipalId::from("agent:net");
    app.state
        .broker
        .issue(
            principal.clone(),
            Grant::new(vec![CapabilityKind::net_http(
                vec!["api.allowed.test".to_string()],
                vec![HttpMethod::Get],
                "test http scope",
            )]),
        )
        .await
        .expect("issue grant");

    let denied = app
        .state
        .broker
        .check(
            &principal,
            &RequestedOp::net_http("api.blocked.test", HttpMethod::Get),
        )
        .await
        .expect_err("host should exceed scope");

    assert_eq!(denied.kind(), "net.http");
    assert_eq!(denied.reason(), &DenialReason::ScopeExceeded);
    assert_eq!(denied.http_status_code(), 403);
}

#[tokio::test]
async fn expired_grant_returns_403_expired() {
    let app = TestApp::new(false).await;
    let principal = PrincipalId::from("agent:expired");
    app.state
        .broker
        .issue(
            principal.clone(),
            agent_spawn_grant("intent_task").expires_at(Utc::now() - Duration::minutes(1)),
        )
        .await
        .expect("issue expired grant");

    let response = intent_response(Arc::clone(&app.state), headers(principal.as_str())).await;
    assert_eq!(response.status(), StatusCode::FORBIDDEN);

    let body = response_json(response).await;
    assert_eq!(body["reason"], "Expired");
}

#[tokio::test]
async fn valid_scoped_grant_allows_intent() {
    let app = TestApp::new(false).await;
    let principal = PrincipalId::from("agent:intent");
    app.state
        .broker
        .issue(principal.clone(), agent_spawn_grant("intent_task"))
        .await
        .expect("issue grant");

    let response = intent_response(Arc::clone(&app.state), headers(principal.as_str())).await;
    assert_eq!(response.status(), StatusCode::OK);

    let body = response_json(response).await;
    assert!(body["cards_queued"].as_u64().unwrap_or_default() > 0);
}

#[tokio::test]
async fn grants_are_isolated_by_principal() {
    let app = TestApp::new(false).await;
    app.state
        .broker
        .issue(
            PrincipalId::from("agent:alice"),
            agent_spawn_grant("intent_task"),
        )
        .await
        .expect("issue grant");

    let response = intent_response(Arc::clone(&app.state), headers("agent:bob")).await;
    assert_eq!(response.status(), StatusCode::FORBIDDEN);

    let body = response_json(response).await;
    assert_eq!(body["reason"], "NoGrant");
}

#[tokio::test]
async fn dev_mode_unknown_principal_is_allowed_by_dev_grant() {
    let app = TestApp::new(true).await;

    let response = intent_response(Arc::clone(&app.state), headers("unknown:principal")).await;
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn revoke_takes_effect_immediately() {
    let app = TestApp::new(false).await;
    let principal = PrincipalId::from("agent:revoked");
    let grant_id = app
        .state
        .broker
        .issue(principal.clone(), agent_spawn_grant("intent_task"))
        .await
        .expect("issue grant");

    let response = intent_response(Arc::clone(&app.state), headers(principal.as_str())).await;
    assert_eq!(response.status(), StatusCode::OK);

    app.state
        .broker
        .revoke(grant_id)
        .await
        .expect("revoke grant");

    let response = intent_response(Arc::clone(&app.state), headers(principal.as_str())).await;
    assert_eq!(response.status(), StatusCode::FORBIDDEN);

    let body = response_json(response).await;
    assert_eq!(body["reason"], "NoGrant");
}

#[tokio::test]
async fn grants_persist_across_restart() {
    let dir = tempdir().expect("temp state dir");
    let principal = PrincipalId::from("agent:persisted");

    {
        let state = state_at(dir.path(), false).await;
        state
            .broker
            .issue(principal.clone(), agent_spawn_grant("intent_task"))
            .await
            .expect("issue grant");
        assert_eq!(
            state
                .broker
                .list(&principal)
                .await
                .expect("list grants")
                .len(),
            1
        );
    }

    let restarted = state_at(dir.path(), false).await;
    assert_eq!(
        restarted
            .broker
            .list(&principal)
            .await
            .expect("list restarted grants")
            .len(),
        1
    );

    let response = intent_response(Arc::clone(&restarted), headers(principal.as_str())).await;
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn approve_and_rewind_are_enforced() {
    let app = TestApp::new(false).await;

    let approve = approve_response(Arc::clone(&app.state), headers("agent:no-ceremony")).await;
    assert_eq!(approve.status(), StatusCode::FORBIDDEN);
    let approve_body = response_json(approve).await;
    assert_eq!(approve_body["kind"], "ceremony.request");

    let rewind = rewind_response(Arc::clone(&app.state), headers("agent:no-ledger")).await;
    assert_eq!(rewind.status(), StatusCode::FORBIDDEN);
    let rewind_body = response_json(rewind).await;
    assert_eq!(rewind_body["kind"], "ledger.read");
}
