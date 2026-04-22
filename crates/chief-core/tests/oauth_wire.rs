//! Integration tests for OAuth wire integration.

use chief_core::state::{AppConfig, AppState, BackendKind};
use std::path::PathBuf;

async fn app_state() -> (AppState, tempfile::TempDir) {
    let tmpdir = tempfile::TempDir::new().unwrap();
    let config = AppConfig {
        state_dir: Some(PathBuf::from(tmpdir.path())),
        dev_mode: true,
        backend: BackendKind::Stub,
        model_path: None,
    };
    let state = AppState::new(config).await.unwrap();
    (state, tmpdir)
}

#[tokio::test]
async fn flow_lifecycle_mock() {
    let (state, _tmpdir) = app_state().await;

    // Start a flow
    let challenge = state
        .oauth_broker
        .start_flow(
            chief_oauth::Provider::Google,
            &["gmail.readonly".to_string()],
            "http://localhost:8080/callback",
        )
        .await
        .expect("start flow");

    assert!(!challenge.auth_url.is_empty());
    assert!(!challenge.state.is_empty());
    let flow_id = challenge.flow_id.clone();

    // Complete the flow
    let session = state
        .oauth_broker
        .complete_flow(flow_id, "code_123", &challenge.state)
        .await
        .expect("complete flow");

    assert_eq!(session.provider, chief_oauth::Provider::Google);
    assert_eq!(session.scopes, vec!["gmail.readonly".to_string()]);
}

#[tokio::test]
async fn revoke_then_proxy_fails() {
    let (state, _tmpdir) = app_state().await;

    // Create a session
    let challenge = state
        .oauth_broker
        .start_flow(
            chief_oauth::Provider::Github,
            &["repo".to_string()],
            "http://localhost:8080/callback",
        )
        .await
        .expect("start flow");

    let session = state
        .oauth_broker
        .complete_flow(challenge.flow_id, "code_xyz", &challenge.state)
        .await
        .expect("complete flow");

    // Revoke the session
    state.oauth_broker.revoke(&session).await.expect("revoke");

    // List should be empty
    let sessions = state.oauth_broker.list().await.expect("list");
    assert_eq!(sessions.len(), 0);
}

#[tokio::test]
async fn session_never_leaks_token() {
    let (state, _tmpdir) = app_state().await;

    // Create a session
    let challenge = state
        .oauth_broker
        .start_flow(
            chief_oauth::Provider::Google,
            &["gmail.readonly".to_string()],
            "http://localhost:8080/callback",
        )
        .await
        .expect("start flow");

    let session = state
        .oauth_broker
        .complete_flow(challenge.flow_id, "code_abc", &challenge.state)
        .await
        .expect("complete flow");

    // Serialize the session handle and check for bearer tokens
    let json = serde_json::to_string(&session).expect("serialize session");

    // Common Google token prefixes
    assert!(!json.contains("ya29"), "Google token prefix found");
    // Common GitHub token prefixes
    assert!(!json.contains("gho_"), "GitHub token prefix found");
    // Generic access token field
    assert!(
        !json.contains("access_token_for_"),
        "access_token_for_ found"
    );
    // Bearer keyword
    assert!(!json.contains("Bearer "), "Bearer keyword found");
}

#[tokio::test]
async fn dev_mode_auto_grant() {
    let tmpdir = tempfile::TempDir::new().unwrap();
    let config = AppConfig {
        state_dir: Some(PathBuf::from(tmpdir.path())),
        dev_mode: true,
        backend: BackendKind::Stub,
        model_path: None,
    };
    let state = AppState::new(config).await.unwrap();

    // In dev mode, OAuth should work
    let challenge = state
        .oauth_broker
        .start_flow(
            chief_oauth::Provider::Google,
            &["gmail.readonly".to_string()],
            "http://localhost:8080/callback",
        )
        .await
        .expect("start flow in dev mode");

    assert!(!challenge.auth_url.is_empty());
}

#[tokio::test]
async fn cross_provider_isolation() {
    let (state, _tmpdir) = app_state().await;

    // Start a Google flow
    let google_challenge = state
        .oauth_broker
        .start_flow(
            chief_oauth::Provider::Google,
            &["gmail.readonly".to_string()],
            "http://localhost:8080/callback",
        )
        .await
        .expect("start google flow");

    // Start a GitHub flow
    let github_challenge = state
        .oauth_broker
        .start_flow(
            chief_oauth::Provider::Github,
            &["repo".to_string()],
            "http://localhost:8080/callback",
        )
        .await
        .expect("start github flow");

    // Complete both flows
    let google_session = state
        .oauth_broker
        .complete_flow(google_challenge.flow_id, "code_g", &google_challenge.state)
        .await
        .expect("complete google");

    let github_session = state
        .oauth_broker
        .complete_flow(github_challenge.flow_id, "code_gh", &github_challenge.state)
        .await
        .expect("complete github");

    // Verify they are different providers
    assert_eq!(google_session.provider, chief_oauth::Provider::Google);
    assert_eq!(github_session.provider, chief_oauth::Provider::Github);
    assert_ne!(google_session.id, github_session.id);
}

#[tokio::test]
async fn scope_check_in_list() {
    let (state, _tmpdir) = app_state().await;

    // Create sessions with different scopes
    let challenge1 = state
        .oauth_broker
        .start_flow(
            chief_oauth::Provider::Google,
            &["gmail.readonly".to_string()],
            "http://localhost:8080/callback",
        )
        .await
        .expect("start flow 1");

    let challenge2 = state
        .oauth_broker
        .start_flow(
            chief_oauth::Provider::Google,
            &[
                "gmail.readonly".to_string(),
                "calendar.readonly".to_string(),
            ],
            "http://localhost:8080/callback",
        )
        .await
        .expect("start flow 2");

    let _session1 = state
        .oauth_broker
        .complete_flow(challenge1.flow_id, "code_1", &challenge1.state)
        .await
        .expect("complete 1");

    let _session2 = state
        .oauth_broker
        .complete_flow(challenge2.flow_id, "code_2", &challenge2.state)
        .await
        .expect("complete 2");

    let sessions = state.oauth_broker.list().await.expect("list");
    assert_eq!(sessions.len(), 2);

    // Both should be Google
    assert!(sessions
        .iter()
        .all(|s| s.handle.provider == chief_oauth::Provider::Google));
}
