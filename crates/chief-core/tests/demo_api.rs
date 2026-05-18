//! Integration tests for the `/v1/*` demo-shell API.
//!
//! These tests boot a real `AppState` onto an ephemeral tempdir, bind a real
//! HTTP listener to an OS-assigned port, and exercise each route with
//! `reqwest`. There is zero mocking: empty state returns empty arrays,
//! approvals issue real broker grants, SSE delivers real broadcast events.

use chief_core::capability::{CapabilityKind, Grant, HttpMethod, PrincipalId, RequestedOp};
use chief_core::ceremony::{CeremonyEvidence, NewCeremony};
use chief_core::inbox::NewInboxItem;
use chief_core::routes::router_with_dist;
use chief_core::state::{AppConfig, BackendKind};
use chief_core::trust_ledger::LedgerDelta;
use chief_core::AppState;
use chrono::Duration;
use futures_util::StreamExt;
use serde_json::Value;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use tempfile::tempdir;

struct TestServer {
    addr: SocketAddr,
    state: Arc<AppState>,
    _shutdown: tokio::sync::oneshot::Sender<()>,
    // Keep the tempdir alive for the lifetime of the server.
    _tmp: tempfile::TempDir,
    _dist_tmp: Option<tempfile::TempDir>,
}

impl TestServer {
    fn url(&self, path: &str) -> String {
        format!("http://{}{}", self.addr, path)
    }
}

async fn spawn(dist_dir: Option<PathBuf>, dist_tmp: Option<tempfile::TempDir>) -> TestServer {
    let tmp = tempdir().expect("tempdir");
    let config = AppConfig {
        state_dir: Some(tmp.path().to_path_buf()),
        dev_mode: true,
        backend: BackendKind::Stub,
        model_path: None,
    };
    let state = Arc::new(AppState::new(config).await.expect("init state"));

    let app = router_with_dist(state.clone(), dist_dir);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind");
    let addr = listener.local_addr().expect("local_addr");

    let (tx, rx) = tokio::sync::oneshot::channel();

    tokio::spawn(async move {
        let server = axum::serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .with_graceful_shutdown(async {
            let _ = rx.await;
        });
        let _ = server.await;
    });

    // Small delay so the listener is accepting connections.
    tokio::time::sleep(std::time::Duration::from_millis(20)).await;

    TestServer {
        addr,
        state,
        _shutdown: tx,
        _tmp: tmp,
        _dist_tmp: dist_tmp,
    }
}

async fn spawn_default() -> TestServer {
    spawn(None, None).await
}

#[tokio::test]
async fn v1_brief_empty_state_returns_empty_arrays() {
    let srv = spawn_default().await;
    let body: Value = reqwest::get(srv.url("/v1/brief"))
        .await
        .expect("GET brief")
        .json()
        .await
        .expect("json");

    assert!(body["greeting"].is_string(), "greeting is a string");
    assert_eq!(
        body["needs_you"].as_array().map(Vec::len),
        Some(0),
        "needs_you empty"
    );
    assert_eq!(
        body["handled"].as_array().map(Vec::len),
        Some(0),
        "handled empty"
    );
    assert_eq!(
        body["provenance"].as_array().map(Vec::len),
        Some(0),
        "provenance empty"
    );
    assert_eq!(
        body["trust"].as_array().map(Vec::len),
        Some(0),
        "trust empty"
    );
    assert!(
        body["signed_by"]
            .as_str()
            .unwrap()
            .starts_with("Chief · Ed25519:"),
        "signed_by has correct prefix"
    );
}

#[tokio::test]
async fn v1_status_describes_headless_node_contract() {
    let srv = spawn_default().await;
    let body: Value = reqwest::get(srv.url("/v1/status"))
        .await
        .expect("GET status")
        .json()
        .await
        .expect("json");

    assert_eq!(body["api_version"].as_str(), Some("v1"));
    assert_eq!(body["node"]["mode"].as_str(), Some("headless_chief_node"));
    assert_eq!(body["node"]["ui_runtime_required"].as_bool(), Some(false));
    assert_eq!(
        body["demo"]["fixture_work_object_id"].as_str(),
        Some("acme-follow-up")
    );

    let channels = body["channels"].as_array().expect("channels array");
    assert!(
        channels
            .iter()
            .any(|channel| channel["name"] == "http" && channel["status"] == "active"),
        "http channel active"
    );
    assert!(
        channels
            .iter()
            .any(|channel| channel["name"] == "cli" && channel["status"] == "active"),
        "cli channel active"
    );
}

#[tokio::test]
async fn v1_inbox_append_and_list() {
    let srv = spawn_default().await;

    for title in ["first", "second", "third"] {
        srv.state
            .inbox
            .append(NewInboxItem {
                title: title.into(),
                snippet: format!("snippet for {title}"),
                source_agent: "test-agent".into(),
                ..Default::default()
            })
            .await;
        tokio::time::sleep(std::time::Duration::from_millis(2)).await;
    }

    let body: Value = reqwest::get(srv.url("/v1/inbox"))
        .await
        .expect("GET inbox")
        .json()
        .await
        .expect("json");
    let items = body.as_array().expect("array");
    assert_eq!(items.len(), 3);
    // Reverse chronological: newest (third) first.
    assert_eq!(items[0]["title"].as_str(), Some("third"));
    assert_eq!(items[1]["title"].as_str(), Some("second"));
    assert_eq!(items[2]["title"].as_str(), Some("first"));
}

#[tokio::test]
async fn v1_inbox_sse_receives_appended_item() {
    let srv = spawn_default().await;

    let client = reqwest::Client::new();
    let resp = client
        .get(srv.url("/v1/inbox/stream"))
        .send()
        .await
        .expect("GET sse");
    assert!(resp.status().is_success());

    let mut stream = resp.bytes_stream();

    // Consume the immediate `ready` hello event.
    let hello = stream
        .next()
        .await
        .expect("hello chunk")
        .expect("hello bytes");
    let hello_text = std::str::from_utf8(&hello).unwrap().to_string();
    assert!(
        hello_text.contains("event: ready"),
        "got hello: {hello_text}"
    );

    // Now append.
    srv.state
        .inbox
        .append(NewInboxItem {
            title: "live".into(),
            snippet: "live payload".into(),
            source_agent: "test".into(),
            ..Default::default()
        })
        .await;

    // Collect bytes until we see the payload.
    let mut buf = String::new();
    let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(5);
    while tokio::time::Instant::now() < deadline {
        let chunk = tokio::time::timeout(std::time::Duration::from_secs(2), stream.next()).await;
        match chunk {
            Ok(Some(Ok(bytes))) => {
                buf.push_str(std::str::from_utf8(&bytes).unwrap_or(""));
                if buf.contains("event: inbox") && buf.contains("\"title\":\"live\"") {
                    return;
                }
            }
            Ok(Some(Err(err))) => panic!("stream err: {err}"),
            Ok(None) => break,
            Err(_) => break,
        }
    }
    panic!("did not receive inbox event; buffer was: {buf}");
}

#[tokio::test]
async fn v1_omnibar_search_empty_memory_returns_empty() {
    let srv = spawn_default().await;
    let client = reqwest::Client::new();
    let body: Value = client
        .post(srv.url("/v1/omnibar/search"))
        .json(&serde_json::json!({ "q": "nothing" }))
        .send()
        .await
        .expect("POST search")
        .json()
        .await
        .expect("json");
    assert_eq!(body.as_array().map(Vec::len), Some(0));
}

#[tokio::test]
async fn v1_omnibar_search_matches_inbox_title() {
    let srv = spawn_default().await;
    srv.state
        .inbox
        .append(NewInboxItem {
            title: "Approve migration plan".into(),
            snippet: "details".into(),
            source_agent: "migration-agent".into(),
            ..Default::default()
        })
        .await;

    let client = reqwest::Client::new();
    let body: Value = client
        .post(srv.url("/v1/omnibar/search"))
        .json(&serde_json::json!({ "q": "migration" }))
        .send()
        .await
        .expect("POST search")
        .json()
        .await
        .expect("json");
    let hits = body.as_array().expect("array");
    assert!(!hits.is_empty(), "expected inbox hit, got {body}");
    assert_eq!(hits[0]["source"], "inbox");
}

#[tokio::test]
async fn v1_ceremony_lifecycle_approve() {
    let srv = spawn_default().await;

    // Seed a ceremony by pushing one into the store directly — this mirrors
    // what the broker will do when it denies a grant that requires ceremony.
    let target_principal = "hn-briefer".to_string();
    let grant = Grant::new(vec![CapabilityKind::net_http(
        vec!["news.ycombinator.com".to_string()],
        vec![HttpMethod::Get],
        "approved via ceremony",
    )]);
    let seeded = srv
        .state
        .ceremonies
        .open(NewCeremony {
            title: "Approve net.http to news.ycombinator.com".into(),
            evidence: CeremonyEvidence {
                summary: "hn-briefer needs to fetch front page".into(),
                details: serde_json::json!({"host": "news.ycombinator.com"}),
            },
            source_agent: target_principal.clone(),
            trust_context: 5,
            proposed_grant: grant,
            payload_hash: None,
            target_principal: target_principal.clone(),
            rollback_window: Duration::hours(24),
            ceremony_ttl: Duration::hours(24),
        })
        .await;

    // Before approval: the principal cannot perform the requested op.
    let denied = srv
        .state
        .broker
        .check(
            &PrincipalId::from(target_principal.as_str()),
            &RequestedOp::net_http("news.ycombinator.com", HttpMethod::Get),
        )
        .await;
    assert!(denied.is_err(), "expected denial before ceremony approval");

    // Fetch via HTTP to confirm detail endpoint works.
    let detail_url = srv.url(&format!("/v1/ceremony/{}", seeded.id));
    let detail: Value = reqwest::get(&detail_url)
        .await
        .expect("GET detail")
        .json()
        .await
        .expect("json");
    assert_eq!(detail["id"], seeded.id);
    assert_eq!(detail["status"], "pending");

    // Approve with a sufficient hold.
    let client = reqwest::Client::new();
    let resp = client
        .post(srv.url(&format!("/v1/ceremony/{}/approve", seeded.id)))
        .json(&serde_json::json!({ "held_ms": 3500u64 }))
        .send()
        .await
        .expect("POST approve");
    assert_eq!(resp.status().as_u16(), 200, "expected 200 OK");
    let body: Value = resp.json().await.expect("json");
    assert_eq!(body["status"], "approved");
    assert!(
        body["ceremony"]["grant_issued"].is_object(),
        "grant_issued should be populated"
    );

    // After approval: the broker now allows the op (real grant issued).
    let allowed = srv
        .state
        .broker
        .check(
            &PrincipalId::from(target_principal.as_str()),
            &RequestedOp::net_http("news.ycombinator.com", HttpMethod::Get),
        )
        .await;
    assert!(
        allowed.is_ok(),
        "broker should allow after approval: {allowed:?}"
    );
}

#[tokio::test]
async fn v1_ceremony_approve_short_hold_rejected() {
    let srv = spawn_default().await;

    let grant = Grant::new(vec![CapabilityKind::net_http(
        vec!["*".into()],
        vec![HttpMethod::Get],
        "test",
    )]);
    let seeded = srv
        .state
        .ceremonies
        .open(NewCeremony {
            title: "test".into(),
            evidence: CeremonyEvidence {
                summary: "test".into(),
                details: Value::Null,
            },
            source_agent: "test".into(),
            trust_context: 0,
            proposed_grant: grant,
            payload_hash: None,
            target_principal: "test-principal".into(),
            rollback_window: Duration::hours(1),
            ceremony_ttl: Duration::hours(1),
        })
        .await;

    let client = reqwest::Client::new();
    let resp = client
        .post(srv.url(&format!("/v1/ceremony/{}/approve", seeded.id)))
        .json(&serde_json::json!({ "held_ms": 500u64 }))
        .send()
        .await
        .expect("POST approve");
    assert_eq!(resp.status().as_u16(), 400);
    let body: Value = resp.json().await.expect("json");
    assert_eq!(body["error"], "hold_too_short");
    assert_eq!(body["required_ms"], 3000);
}

#[tokio::test]
async fn v1_trust_ledger_empty_returns_empty() {
    let srv = spawn_default().await;
    let body: Value = reqwest::get(srv.url("/v1/trust-ledger"))
        .await
        .expect("GET ledger")
        .json()
        .await
        .expect("json");
    assert_eq!(body.as_array().map(Vec::len), Some(0));
}

#[tokio::test]
async fn v1_trust_ledger_after_real_events() {
    let srv = spawn_default().await;
    srv.state
        .trust_ledger
        .record("email", LedgerDelta::Approval)
        .await;
    srv.state
        .trust_ledger
        .record("email", LedgerDelta::Approval)
        .await;
    srv.state
        .trust_ledger
        .record("memory", LedgerDelta::Denial)
        .await;

    let body: Value = reqwest::get(srv.url("/v1/trust-ledger"))
        .await
        .expect("GET ledger")
        .json()
        .await
        .expect("json");
    let rows = body.as_array().expect("array");
    assert_eq!(rows.len(), 2);
    // Activity tie: email has 2 approvals = activity 2, memory has 1 denial.
    // email should come first (2 events > 1 event).
    assert_eq!(rows[0]["category"], "email");
    assert_eq!(rows[0]["approvals"], 2);
    assert_eq!(rows[1]["category"], "memory");
    assert_eq!(rows[1]["setbacks"], 1);
}

#[tokio::test]
async fn v1_models_unbound_returns_none() {
    let srv = spawn_default().await;
    let body: Value = reqwest::get(srv.url("/v1/models"))
        .await
        .expect("GET models")
        .json()
        .await
        .expect("json");
    // Stub backend => tiers.fast = None, deep = None, bindings = [].
    assert!(body["tiers"]["fast"].is_null());
    assert!(body["tiers"]["deep"].is_null());
    assert_eq!(body["bindings"].as_array().map(Vec::len), Some(0));
}

#[tokio::test]
async fn v1_ai_generate_uses_brokered_stub_without_openrouter() {
    std::env::set_var("CHIEF_AI_DISABLE_OPENROUTER", "1");
    let srv = spawn_default().await;
    let client = reqwest::Client::new();
    let body: Value = client
        .post(srv.url("/v1/ai/generate"))
        .json(&serde_json::json!({
            "prompt": "Return a short app plan.",
            "tier": "fast",
            "input": {"work_object_id": "acme-follow-up"},
        }))
        .send()
        .await
        .expect("POST ai generate")
        .json()
        .await
        .expect("json");

    assert_eq!(body["provider"].as_str(), Some("local"));
    assert!(
        body["model"]
            .as_str()
            .unwrap_or_default()
            .starts_with("local:llama-cpp-"),
        "model={}",
        body["model"]
    );
    assert!(
        body["text"]
            .as_str()
            .unwrap_or_default()
            .contains("Return a short app plan."),
        "text={}",
        body["text"]
    );
    assert!(body["attestation"]["id"].is_array());
}

#[tokio::test]
async fn v1_static_placeholder_when_dist_missing() {
    let srv = spawn_default().await;
    let resp = reqwest::get(srv.url("/")).await.expect("GET root");
    assert_eq!(resp.status().as_u16(), 200);
    let ct = resp
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    assert!(ct.starts_with("text/plain"), "content-type {ct}");
    let body = resp.text().await.expect("text");
    assert!(
        body.contains("chief-core is running"),
        "placeholder text, got: {body}"
    );
}

#[tokio::test]
async fn v1_static_serves_index_when_dist_present() {
    let dist_tmp = tempdir().expect("dist tmp");
    let index_path = dist_tmp.path().join("index.html");
    tokio::fs::write(&index_path, "<!doctype html><title>demo</title><h1>hi</h1>")
        .await
        .expect("write index");
    let srv = spawn(Some(dist_tmp.path().to_path_buf()), Some(dist_tmp)).await;

    let resp = reqwest::get(srv.url("/")).await.expect("GET root");
    assert_eq!(resp.status().as_u16(), 200);
    let body = resp.text().await.expect("text");
    assert!(body.contains("<h1>hi</h1>"), "got: {body}");
}

#[tokio::test]
async fn v1_brief_signed_by_includes_device_fingerprint() {
    let srv = spawn_default().await;
    let body: Value = reqwest::get(srv.url("/v1/brief"))
        .await
        .expect("GET brief")
        .json()
        .await
        .expect("json");
    let signed_by = body["signed_by"].as_str().expect("string");
    let fingerprint = srv.state.device_fingerprint();
    assert!(
        signed_by.ends_with(&fingerprint),
        "signed_by={signed_by}, fingerprint={fingerprint}"
    );
}

#[tokio::test]
async fn v1_ceremony_detail_404_for_unknown() {
    let srv = spawn_default().await;
    let resp = reqwest::get(srv.url("/v1/ceremony/ceremony_nonexistent"))
        .await
        .expect("GET");
    assert_eq!(resp.status().as_u16(), 404);
}

#[tokio::test]
async fn legacy_routes_still_work() {
    // Backward-compat: the v0 /status endpoint must still be reachable.
    let srv = spawn_default().await;
    let resp = reqwest::get(srv.url("/status")).await.expect("GET status");
    assert!(resp.status().is_success());
}
