//! Test OAuth session revocation.

use chief_oauth::{OAuthBroker, Provider};
use tempfile::TempDir;

#[tokio::test]
async fn test_revoke_session() {
    let tmpdir = TempDir::new().unwrap();
    let broker = OAuthBroker::new(tmpdir.path()).await.unwrap();

    // Start and complete a flow
    let challenge = broker
        .start_flow(
            Provider::Google,
            &["gmail.readonly".to_string()],
            "http://localhost:8080/callback",
        )
        .await
        .expect("start_flow");

    let session = broker
        .complete_flow(challenge.flow_id, "auth_code_xyz", &challenge.state)
        .await
        .expect("complete_flow");

    // Verify session exists
    let sessions_before = broker.list().await.expect("list sessions");
    assert_eq!(sessions_before.len(), 1);

    // Revoke the session
    broker.revoke(&session).await.expect("revoke session");

    // Verify session is gone
    let sessions_after = broker.list().await.expect("list sessions after revoke");
    assert_eq!(sessions_after.len(), 0);
}

#[tokio::test]
async fn test_revoke_clears_sealed_storage() {
    let tmpdir = TempDir::new().unwrap();
    let broker = OAuthBroker::new(tmpdir.path()).await.unwrap();

    // Create a session
    let challenge = broker
        .start_flow(
            Provider::Github,
            &["repo".to_string()],
            "http://localhost:8080/callback",
        )
        .await
        .expect("start_flow");

    let session = broker
        .complete_flow(challenge.flow_id, "auth_code_xyz", &challenge.state)
        .await
        .expect("complete_flow");

    // Verify token file exists
    let token_file = tmpdir.path().join("tokens.bin");
    assert!(
        token_file.exists(),
        "token file should exist after completing flow"
    );

    // Revoke
    broker.revoke(&session).await.expect("revoke session");

    // Verify token file is deleted
    assert!(
        !token_file.exists(),
        "token file should be deleted after revocation"
    );
}
