//! End-to-end test for the `/v1/work/:id` projection.

use axum::Router;
use chief_core::routes::router_with_dist;
use chief_core::state::{AppConfig, BackendKind};
use chief_core::AppState;
use chief_mem::{Edge, EdgeKind, Horizon, Node, NodeType};
use serde_json::{json, Value};
use std::net::SocketAddr;
use std::sync::Arc;
use tempfile::tempdir;

const CONTRACT: &str = include_str!("fixtures/platform_demo/acme-contract.md");
const CALENDAR: &str = include_str!("fixtures/platform_demo/calendar.json");
const PRIOR_EMAIL: &str = include_str!("fixtures/platform_demo/prior-email.json");

struct TestServer {
    addr: SocketAddr,
    _shutdown: tokio::sync::oneshot::Sender<()>,
    _tmp: tempfile::TempDir,
}

impl TestServer {
    fn url(&self, path: &str) -> String {
        format!("http://{}{}", self.addr, path)
    }
}

async fn spawn(app: Router) -> TestServer {
    let tmp = tempdir().expect("tempdir");
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
        _shutdown: tx,
        _tmp: tmp,
    }
}

#[tokio::test]
async fn v1_work_returns_memory_graph_projection() {
    let tmp = tempdir().expect("state tempdir");
    let config = AppConfig {
        state_dir: Some(tmp.path().to_path_buf()),
        dev_mode: true,
        backend: BackendKind::Stub,
        model_path: None,
    };
    let state = Arc::new(AppState::new(config).await.expect("init state"));
    seed_acme_work_object(&state).await;

    let app = router_with_dist(state, None);
    let srv = spawn(app).await;

    let body: Value = reqwest::get(srv.url("/v1/work/acme-follow-up"))
        .await
        .expect("GET work")
        .json()
        .await
        .expect("json");

    assert_eq!(body["id"], "acme-follow-up");
    assert_eq!(body["title"], "Prepare the Acme follow-up");
    assert!(
        body["uri"]
            .as_str()
            .unwrap_or_default()
            .starts_with("mem://artifact/"),
        "Work Object must remain a Memory Graph artifact"
    );
    assert_eq!(body["source_refs"].as_array().map(Vec::len), Some(3));
    assert_eq!(body["contributions"].as_array().map(Vec::len), Some(0));
    assert_eq!(body["provenance"].as_array().map(Vec::len), Some(1));
}

#[tokio::test]
async fn v1_work_missing_id_returns_404() {
    let tmp = tempdir().expect("state tempdir");
    let config = AppConfig {
        state_dir: Some(tmp.path().to_path_buf()),
        dev_mode: true,
        backend: BackendKind::Stub,
        model_path: None,
    };
    let state = Arc::new(AppState::new(config).await.expect("init state"));
    let app = router_with_dist(state, None);
    let srv = spawn(app).await;

    let resp = reqwest::get(srv.url("/v1/work/missing"))
        .await
        .expect("GET missing work");
    assert_eq!(resp.status(), reqwest::StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn external_app_can_write_work_contribution_through_public_api() {
    let tmp = tempdir().expect("state tempdir");
    let config = AppConfig {
        state_dir: Some(tmp.path().to_path_buf()),
        dev_mode: true,
        backend: BackendKind::Stub,
        model_path: None,
    };
    let state = Arc::new(AppState::new(config).await.expect("init state"));
    seed_acme_work_object(&state).await;

    let app = router_with_dist(state, None);
    let srv = spawn(app).await;
    let client = reqwest::Client::new();

    let written: Value = client
        .post(srv.url("/v1/work/acme-follow-up/contributions"))
        .header("x-chief-principal", "app:sales-followup")
        .json(&json!({
            "node_type": "finding",
            "kind": "next_best_action",
            "title": "Call Acme before sending the draft",
            "summary": "External app recommends a call because the contract clause changed.",
            "authority_state": "handled",
            "body": {
                "confidence": 0.82,
                "rationale": "Clause 4 changed since the prior email thread."
            }
        }))
        .send()
        .await
        .expect("POST contribution")
        .error_for_status()
        .expect("created")
        .json()
        .await
        .expect("json");

    assert_eq!(written["work_object_id"], "acme-follow-up");
    assert_eq!(written["contribution"]["source"], "app:sales-followup");
    assert_eq!(written["contribution"]["pack"], "app:sales-followup");
    assert_eq!(written["contribution"]["node_type"], "finding");
    assert_eq!(written["contribution"]["kind"], "next_best_action");
    assert_eq!(
        written["contribution"]["body"]["work_object_id"],
        "acme-follow-up"
    );

    let work: Value = client
        .get(srv.url("/v1/work/acme-follow-up"))
        .send()
        .await
        .expect("GET work")
        .error_for_status()
        .expect("ok")
        .json()
        .await
        .expect("json");
    let contributions = work["contributions"].as_array().expect("contributions");
    assert_eq!(contributions.len(), 1);
    assert_eq!(contributions[0]["source"], "app:sales-followup");
    assert_eq!(
        contributions[0]["title"],
        "Call Acme before sending the draft"
    );

    let provenance: Value = client
        .get(srv.url("/v1/work/acme-follow-up/provenance"))
        .send()
        .await
        .expect("GET provenance")
        .error_for_status()
        .expect("ok")
        .json()
        .await
        .expect("json");
    assert!(
        provenance
            .as_array()
            .expect("provenance")
            .iter()
            .any(|row| {
                row["kind"] == "contribution_written" && row["source"] == "app:sales-followup"
            }),
        "provenance should attribute the external app contribution: {provenance:?}"
    );
}

#[tokio::test]
async fn external_app_contribution_write_requires_broker_grant() {
    let tmp = tempdir().expect("state tempdir");
    let config = AppConfig {
        state_dir: Some(tmp.path().to_path_buf()),
        dev_mode: false,
        backend: BackendKind::Stub,
        model_path: None,
    };
    let state = Arc::new(AppState::new(config).await.expect("init state"));
    seed_acme_work_object(&state).await;

    let app = router_with_dist(state, None);
    let srv = spawn(app).await;

    let resp = reqwest::Client::new()
        .post(srv.url("/v1/work/acme-follow-up/contributions"))
        .header("x-chief-principal", "app:sales-followup")
        .json(&json!({
            "node_type": "finding",
            "kind": "next_best_action",
            "title": "Call Acme before sending the draft"
        }))
        .send()
        .await
        .expect("POST contribution");

    assert_eq!(resp.status(), reqwest::StatusCode::FORBIDDEN);
    let body: Value = resp.json().await.expect("json");
    assert_eq!(body["error"], "capability_denied");
    assert_eq!(body["kind"], "mem.write");
}

#[tokio::test]
async fn external_app_contribution_can_open_headless_ceremony() {
    let tmp = tempdir().expect("state tempdir");
    let config = AppConfig {
        state_dir: Some(tmp.path().to_path_buf()),
        dev_mode: true,
        backend: BackendKind::Stub,
        model_path: None,
    };
    let state = Arc::new(AppState::new(config).await.expect("init state"));
    seed_acme_work_object(&state).await;

    let app = router_with_dist(state, None);
    let srv = spawn(app).await;
    let client = reqwest::Client::new();
    let action_payload = json!({
        "to": "maya@acme.example",
        "subject": "Acme follow-up",
        "body": "Confirm clause 4 before sending."
    });

    let written: Value = client
        .post(srv.url("/v1/work/acme-follow-up/contributions"))
        .header("x-chief-principal", "app:sales-followup")
        .json(&json!({
            "node_type": "artifact",
            "kind": "proposed_send",
            "title": "Send Acme follow-up after call",
            "summary": "External app drafted an email send that requires Ceremony.",
            "body": {
                "draft_action": "email.send",
                "payload": action_payload
            },
            "ceremony": {
                "title": "Send Acme follow-up after call",
                "summary": "sales-followup app needs Ceremony before sending externally.",
                "category": "email.send",
                "payload": action_payload,
                "trust_context": 3
            }
        }))
        .send()
        .await
        .expect("POST contribution")
        .error_for_status()
        .expect("created")
        .json()
        .await
        .expect("json");

    assert_eq!(written["contribution"]["source"], "app:sales-followup");
    assert_eq!(written["contribution"]["authority_state"], "needs_ceremony");
    let ceremony_id = written["ceremony"]["id"]
        .as_str()
        .expect("ceremony id")
        .to_string();
    let payload_hash = written["ceremony"]["payload_hash"]
        .as_str()
        .expect("payload hash")
        .to_string();

    let pending: Value = client
        .get(srv.url("/v1/ceremony"))
        .send()
        .await
        .expect("GET ceremony")
        .error_for_status()
        .expect("ok")
        .json()
        .await
        .expect("json");
    assert!(
        pending.as_array().expect("pending").iter().any(|item| {
            item["id"] == ceremony_id && item["source_agent"] == "app:sales-followup"
        }),
        "pending Ceremony should include external app item: {pending:?}"
    );

    let inbox: Value = client
        .get(srv.url("/v1/inbox"))
        .send()
        .await
        .expect("GET inbox")
        .error_for_status()
        .expect("ok")
        .json()
        .await
        .expect("json");
    assert!(
        inbox.as_array().expect("inbox").iter().any(|item| {
            item["ceremony_id"] == ceremony_id && item["source_agent"] == "app:sales-followup"
        }),
        "inbox should include external app Ceremony item: {inbox:?}"
    );

    let approved: Value = client
        .post(srv.url(&format!("/v1/ceremony/{ceremony_id}/approve")))
        .json(&json!({
            "held_ms": 3000,
            "payload_hash": payload_hash
        }))
        .send()
        .await
        .expect("POST approve")
        .error_for_status()
        .expect("approved")
        .json()
        .await
        .expect("json");
    assert_eq!(approved["status"], "approved");
    assert_eq!(
        approved["ceremony"]["target_principal"],
        "app:sales-followup"
    );
    assert!(approved["ceremony"]["grant_issued"].is_object());
}

async fn seed_acme_work_object(state: &AppState) -> String {
    let mem = state.mem.lock().await;
    let contract_uri = mem
        .put_node(Node::new(
            NodeType::File,
            Horizon::Medium,
            "fixture:document-pack",
            json!({
                "kind": "fixture_file",
                "path": "fixtures/platform_demo/acme-contract.md",
                "mime": "text/markdown",
                "body": CONTRACT
            })
            .to_string(),
        ))
        .expect("put contract");
    let calendar_uri = mem
        .put_node(Node::new(
            NodeType::Event,
            Horizon::Medium,
            "fixture:calendar-pack",
            CALENDAR,
        ))
        .expect("put calendar");
    let prior_email_uri = mem
        .put_node(Node::new(
            NodeType::Email,
            Horizon::Medium,
            "fixture:email-pack",
            PRIOR_EMAIL,
        ))
        .expect("put prior email");

    let work_body = json!({
        "kind": "work_object",
        "id": "acme-follow-up",
        "title": "Prepare the Acme follow-up",
        "source_refs": [
            contract_uri,
            calendar_uri,
            prior_email_uri
        ],
        "contributions": []
    })
    .to_string();
    let work_uri = mem
        .put_node(Node::new(
            NodeType::Artifact,
            Horizon::Medium,
            "chief-core:work-object-e2e",
            work_body,
        ))
        .expect("put work object");

    for source in [&contract_uri, &calendar_uri, &prior_email_uri] {
        mem.put_edge(Edge {
            from: work_uri.clone(),
            to: source.to_string(),
            kind: EdgeKind::DerivedFrom,
            weight: 1.0,
            created_by: "chief-core:work-object-e2e".into(),
        })
        .expect("put source edge");
    }

    work_uri
}
