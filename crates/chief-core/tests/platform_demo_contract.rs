use chief_core::state::{AppConfig, BackendKind};
use chief_core::AppState;
use chief_mem::{Edge, EdgeKind, Horizon, Node, NodeType};
use serde_json::{json, Value};
use tempfile::tempdir;

const CONTRACT: &str = include_str!("fixtures/platform_demo/acme-contract.md");
const CALENDAR: &str = include_str!("fixtures/platform_demo/calendar.json");
const PRIOR_EMAIL: &str = include_str!("fixtures/platform_demo/prior-email.json");
const EXPECTED: &str = include_str!("fixtures/platform_demo/expected-work-object.json");
const GRANTS: &str = include_str!("fixtures/platform_demo/grants.toml");

#[tokio::test]
async fn fixture_contract_boots_real_state_and_creates_deterministic_work_object() {
    let tmp = tempdir().expect("tempdir");
    let config = AppConfig {
        state_dir: Some(tmp.path().to_path_buf()),
        dev_mode: true,
        backend: BackendKind::Stub,
        model_path: None,
    };
    let state = AppState::new(config).await.expect("boot real AppState");
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
        .expect("put contract node");

    let calendar_uri = mem
        .put_node(Node::new(
            NodeType::Event,
            Horizon::Medium,
            "fixture:calendar-pack",
            CALENDAR,
        ))
        .expect("put calendar node");

    let prior_email_uri = mem
        .put_node(Node::new(
            NodeType::Email,
            Horizon::Medium,
            "fixture:email-pack",
            PRIOR_EMAIL,
        ))
        .expect("put prior email node");

    let expected: Value = serde_json::from_str(EXPECTED).expect("expected json");
    let work_body = json!({
        "kind": "work_object",
        "id": expected["id"],
        "title": expected["title"],
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
            "chief-core:platform-demo-contract",
            work_body.clone(),
        ))
        .expect("put work object");

    let expected_uri = format!(
        "mem://artifact/{}",
        blake3::hash(work_body.as_bytes()).to_hex()
    );
    assert_eq!(work_uri, expected_uri);
    assert!(
        work_uri.starts_with("mem://artifact/"),
        "work object must stay in the Memory Graph artifact namespace: {work_uri}"
    );

    for source in [&contract_uri, &calendar_uri, &prior_email_uri] {
        mem.put_edge(Edge {
            from: work_uri.clone(),
            to: source.to_string(),
            kind: EdgeKind::DerivedFrom,
            weight: 1.0,
            created_by: "chief-core:platform-demo-contract".into(),
        })
        .expect("put source edge");
    }

    let stored = mem
        .get_node(&work_uri)
        .expect("get work object")
        .expect("work object exists");
    let stored_body: Value = serde_json::from_str(&stored.body).expect("stored body json");

    assert_eq!(stored.node_type, NodeType::Artifact);
    assert_eq!(stored_body["kind"], "work_object");
    assert_eq!(stored_body["id"], "acme-follow-up");
    assert_eq!(stored_body["title"], "Prepare the Acme follow-up");
    assert_eq!(
        stored_body["source_refs"].as_array().map(Vec::len),
        Some(expected["source_ref_count"].as_u64().unwrap() as usize)
    );
    assert_eq!(
        stored_body["contributions"].as_array().map(Vec::len),
        Some(0)
    );

    let source_edges = mem.get_edges_from(&work_uri).expect("source edges");
    assert_eq!(source_edges.len(), 3);
    assert!(
        source_edges
            .iter()
            .all(|edge| edge.kind == EdgeKind::DerivedFrom),
        "phase 0 must use only existing edge kinds"
    );
}

#[test]
fn fixture_grants_are_demo_scoped_and_credential_free() {
    let grants: toml::Value = toml::from_str(GRANTS).expect("valid grant toml");
    let grant_list = grants["grants"].as_array().expect("grants array");

    assert_eq!(grant_list.len(), 6);
    for grant in grant_list {
        assert!(
            grant["usage_reason"]
                .as_str()
                .map(|s| !s.trim().is_empty())
                .unwrap_or(false),
            "every grant needs a user-visible usage_reason"
        );
    }

    let lower = GRANTS.to_ascii_lowercase();
    for forbidden in ["oauth", "api_key", "secret", "token", "password"] {
        assert!(
            !lower.contains(forbidden),
            "phase 0 fixtures must not require live credentials: found {forbidden}"
        );
    }
}
