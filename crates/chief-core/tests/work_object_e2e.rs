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
