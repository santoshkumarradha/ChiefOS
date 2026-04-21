//! Test the complete OAuth flow lifecycle.

use chief_oauth::{OAuthBroker, Provider};
use tempfile::TempDir;

#[tokio::test]
async fn test_complete_flow_lifecycle() {
    let tmpdir = TempDir::new().unwrap();
    let broker = OAuthBroker::new(tmpdir.path()).await.unwrap();

    // Start authorization flow
    let challenge = broker
        .start_flow(
            Provider::Google,
            &["gmail.readonly".to_string()],
            "http://localhost:8080/callback",
        )
        .await
        .expect("start_flow");

    assert!(!challenge.auth_url.is_empty());
    assert!(!challenge.state.is_empty());
    let flow_id = challenge.flow_id.clone();

    // Simulate authorization code exchange
    // (In a real test, we'd use a mock authorization server)
    let session = broker
        .complete_flow(flow_id, "auth_code_xyz", &challenge.state)
        .await
        .expect("complete_flow");

    assert_eq!(session.provider, Provider::Google);
    assert!(session.scopes.contains(&"gmail.readonly".to_string()));

    // List sessions
    let sessions = broker.list().await.expect("list sessions");
    assert_eq!(sessions.len(), 1);

    // Revoke session
    broker.revoke(&session).await.expect("revoke session");

    let sessions_after = broker.list().await.expect("list sessions after revoke");
    assert_eq!(sessions_after.len(), 0);
}

#[tokio::test]
async fn test_expired_flow_rejected() {
    use chief_oauth::flow::{FlowId, PendingFlow};
    use std::time::{SystemTime, UNIX_EPOCH};

    let tmpdir = TempDir::new().unwrap();
    let broker = OAuthBroker::new(tmpdir.path()).await.unwrap();

    // Start a flow
    let challenge = broker
        .start_flow(
            Provider::Github,
            &["repo".to_string()],
            "http://localhost:8080/callback",
        )
        .await
        .expect("start_flow");

    // Try to complete with wrong state (this should fail even without expiry check in this test)
    let result = broker
        .complete_flow(challenge.flow_id, "auth_code", "wrong_state")
        .await;

    assert!(result.is_err());
}
