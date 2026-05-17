//! End-to-end Ceremony payload binding test for the Platform MVP demo.

use chief_core::capability::{CapabilityKind, Grant, PrincipalId, RequestedOp};
use chief_core::ceremony::{CeremonyEvidence, NewCeremony};
use chief_core::routes::router_with_dist;
use chief_core::state::{AppConfig, BackendKind};
use chief_core::AppState;
use chrono::Duration;
use serde_json::{json, Value};
use std::net::SocketAddr;
use std::sync::Arc;
use tempfile::tempdir;

struct TestServer {
    addr: SocketAddr,
    state: Arc<AppState>,
    _shutdown: tokio::sync::oneshot::Sender<()>,
    _tmp: tempfile::TempDir,
}

impl TestServer {
    fn url(&self, path: &str) -> String {
        format!("http://{}{}", self.addr, path)
    }
}

async fn spawn_default() -> TestServer {
    let tmp = tempdir().expect("tempdir");
    let config = AppConfig {
        state_dir: Some(tmp.path().to_path_buf()),
        dev_mode: false,
        backend: BackendKind::Stub,
        model_path: None,
    };
    let state = Arc::new(AppState::new(config).await.expect("init state"));
    let app = router_with_dist(Arc::clone(&state), None);
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
    tokio::time::sleep(std::time::Duration::from_millis(20)).await;

    TestServer {
        addr,
        state,
        _shutdown: tx,
        _tmp: tmp,
    }
}

#[tokio::test]
async fn ceremony_approval_is_bound_to_exact_payload_hash() {
    let srv = spawn_default().await;
    let principal = "pack:email-pack";
    let draft_payload = json!({
        "recipient": "maya@acme.example",
        "subject": "Re: Acme pilot kickoff",
        "body": "Confirm kickoff and ask for security contact."
    });
    let draft_hash = payload_hash(&draft_payload);
    let mutated_hash = payload_hash(&json!({
        "recipient": "mallory@example.invalid",
        "subject": "Re: Acme pilot kickoff",
        "body": "Confirm kickoff and ask for security contact."
    }));

    let grant = Grant::new(vec![CapabilityKind::ceremony_request(
        vec!["email.send".into()],
        "Permit committing the exact Acme follow-up draft approved in Ceremony.",
    )]);
    let ceremony = srv
        .state
        .ceremonies
        .open(NewCeremony {
            title: "Send Acme follow-up reply".into(),
            evidence: CeremonyEvidence {
                summary: "email-pack drafted a reply to Maya at Acme.".into(),
                details: json!({
                    "payload": draft_payload,
                    "payload_hash": draft_hash,
                }),
            },
            source_agent: "email-pack".into(),
            trust_context: 3,
            proposed_grant: grant,
            payload_hash: Some(draft_hash.clone()),
            target_principal: principal.into(),
            rollback_window: Duration::hours(24),
            ceremony_ttl: Duration::hours(1),
        })
        .await;

    let denied = srv
        .state
        .broker
        .check(
            &PrincipalId::from(principal),
            &RequestedOp::ceremony_request("email.send"),
        )
        .await;
    assert!(denied.is_err(), "send must be denied before Ceremony");

    let client = reqwest::Client::new();
    let mismatch = client
        .post(srv.url(&format!("/v1/ceremony/{}/approve", ceremony.id)))
        .json(&json!({
            "held_ms": 3500u64,
            "payload_hash": mutated_hash,
        }))
        .send()
        .await
        .expect("approve mismatch");
    assert_eq!(mismatch.status(), reqwest::StatusCode::CONFLICT);
    let mismatch_body: Value = mismatch.json().await.expect("mismatch json");
    assert_eq!(mismatch_body["error"], "payload_hash_mismatch");

    let ok = client
        .post(srv.url(&format!("/v1/ceremony/{}/approve", ceremony.id)))
        .json(&json!({
            "held_ms": 3500u64,
            "payload_hash": draft_hash,
        }))
        .send()
        .await
        .expect("approve exact payload");
    assert_eq!(ok.status(), reqwest::StatusCode::OK);

    let allowed = srv
        .state
        .broker
        .check(
            &PrincipalId::from(principal),
            &RequestedOp::ceremony_request("email.send"),
        )
        .await;
    assert!(
        allowed.is_ok(),
        "send should be allowed after exact approval"
    );
}

fn payload_hash(value: &Value) -> String {
    let bytes = serde_json::to_vec(value).expect("payload json");
    format!("blake3:{}", blake3::hash(&bytes).to_hex())
}
