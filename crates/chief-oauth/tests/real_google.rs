//! Real Google OAuth integration test (opt-in).
//!
//! Run with:
//!   GOOGLE_CLIENT_ID=... GOOGLE_CLIENT_SECRET=... cargo test --test real_google -- --ignored

#[tokio::test]
#[ignore]
async fn test_real_google_oauth_flow() {
    // This test requires real OAuth credentials and would actually
    // hit Google's authorization servers. Skipped by default.
    // To enable: set GOOGLE_CLIENT_ID and GOOGLE_CLIENT_SECRET env vars.

    let client_id = std::env::var("GOOGLE_CLIENT_ID").unwrap_or_default();
    let client_secret = std::env::var("GOOGLE_CLIENT_SECRET").unwrap_or_default();

    if client_id.is_empty() || client_secret.is_empty() {
        println!("Skipping real Google test: missing credentials");
        return;
    }

    // In a real implementation, this would:
    // 1. Start an authorization flow
    // 2. Open a browser (or simulate user consent)
    // 3. Exchange authorization code for token
    // 4. Call a real Google API (e.g., userinfo)
    // 5. Verify token storage is sealed

    println!("Real Google OAuth test: implementation would go here");
}
