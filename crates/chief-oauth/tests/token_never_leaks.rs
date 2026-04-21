//! Test that bearer tokens never appear in SessionHandle serialization.

use chief_oauth::{OAuthBroker, Provider, SessionHandle};
use tempfile::TempDir;

#[test]
fn test_session_handle_contains_no_bearer_token() {
    let handle = SessionHandle::new(
        "sess_abc123".to_string(),
        Provider::Google,
        vec!["gmail.readonly".to_string()],
    );

    let json_str = serde_json::to_string(&handle).expect("serialize handle");

    // Assert: no bearer token patterns in the JSON
    assert!(
        !json_str.contains("ya29"),
        "Google token prefix found in serialized SessionHandle"
    );
    assert!(
        !json_str.contains("gho_"),
        "GitHub token prefix found in serialized SessionHandle"
    );
    assert!(
        !json_str.contains("access_token"),
        "access_token field found in serialized SessionHandle"
    );
    assert!(
        !json_str.contains("Bearer"),
        "Bearer token found in serialized SessionHandle"
    );

    // The handle should only contain: id, provider, scopes
    assert!(json_str.contains("sess_abc123"));
    assert!(json_str.contains("gmail.readonly"));
}

#[tokio::test]
async fn test_sealed_file_contains_no_plaintext_token() {
    use std::fs;

    let tmpdir = TempDir::new().expect("create tempdir");
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

    // Read the sealed file
    let sealed_bytes = fs::read(tmpdir.path().join("tokens.bin")).expect("read tokens.bin");

    // Convert to string for assertions
    let sealed_str = String::from_utf8_lossy(&sealed_bytes);

    // Assert: no plaintext bearer tokens in the encrypted file
    assert!(
        !sealed_str.contains("ya29"),
        "Google token found as plaintext in sealed storage"
    );
    assert!(
        !sealed_str.contains("gho_"),
        "GitHub token found as plaintext in sealed storage"
    );
    assert!(
        !sealed_str.contains("access_token_for_"),
        "token pattern found as plaintext in sealed storage"
    );

    // The file should be mostly high-entropy (binary encrypted data)
    // A very crude check: if the file is mostly-ASCII, it's probably not encrypted
    let ascii_count = sealed_bytes.iter().filter(|&&b| b < 128 && b > 31).count();
    let ratio = ascii_count as f64 / sealed_bytes.len() as f64;
    assert!(
        ratio < 0.3,
        "sealed file appears to be mostly plaintext ({}% ASCII)",
        (ratio * 100.0) as u32
    );
}
