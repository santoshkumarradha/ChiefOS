//! Test scope enforcement in proxy requests.

use chief_oauth::{OAuthBroker, Provider, ProxyRequest, SessionHandle};
use tempfile::TempDir;

#[tokio::test]
async fn test_scope_enforcement_denies_wrong_host() {
    let tmpdir = TempDir::new().unwrap();
    let broker = OAuthBroker::new(tmpdir.path()).await.unwrap();

    let session = SessionHandle::new(
        "sess_test".to_string(),
        Provider::Google,
        vec!["gmail.readonly".to_string()],
    );

    // Attempt to proxy a request to an unauthorized host
    let request = ProxyRequest {
        url: "https://evil.example.com/api/data".to_string(),
        method: "GET".to_string(),
        headers: Default::default(),
        body: None,
    };

    let result = broker.proxy_request(&session, request).await;
    assert!(result.is_err());
    match result {
        Err(chief_oauth::OAuthError::ScopeDenied { .. }) => {}
        _ => panic!("expected ScopeDenied error, got {:?}", result),
    }
}

#[tokio::test]
async fn test_scope_enforcement_allows_google_apis() {
    let tmpdir = TempDir::new().unwrap();
    let broker = OAuthBroker::new(tmpdir.path()).await.unwrap();

    let session = SessionHandle::new(
        "sess_test".to_string(),
        Provider::Google,
        vec!["gmail.readonly".to_string()],
    );

    // Request to googleapis.com should succeed (or fail on auth, but not scope)
    let request = ProxyRequest {
        url: "https://www.googleapis.com/gmail/v1/users/me/messages".to_string(),
        method: "GET".to_string(),
        headers: Default::default(),
        body: None,
    };

    let result = broker.proxy_request(&session, request).await;
    // Scope validation must pass for googleapis.com. Downstream errors (no
    // sealed token, no network) are acceptable — we only assert this is
    // NOT a scope denial.
    assert!(!matches!(
        result,
        Err(chief_oauth::OAuthError::ScopeDenied { .. })
    ));
}

#[tokio::test]
async fn test_scope_enforcement_github() {
    let tmpdir = TempDir::new().unwrap();
    let broker = OAuthBroker::new(tmpdir.path()).await.unwrap();

    let session = SessionHandle::new(
        "sess_gh".to_string(),
        Provider::Github,
        vec!["repo".to_string()],
    );

    // Request to api.github.com should pass scope check
    let request = ProxyRequest {
        url: "https://api.github.com/user/repos".to_string(),
        method: "GET".to_string(),
        headers: Default::default(),
        body: None,
    };

    let result = broker.proxy_request(&session, request).await;
    // Scope validation must pass for api.github.com. Downstream errors OK.
    assert!(!matches!(
        result,
        Err(chief_oauth::OAuthError::ScopeDenied { .. })
    ));
}
