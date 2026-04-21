//! Integration tests asserting SDK contract invariants.

use chief_sdk::prelude::*;
use serde_json::json;

#[test]
fn test_closed_enum_invariance() {
    let _net_http = CapabilityKind::NetHttp {
        hosts: vec!["example.com".to_string()],
        methods: vec!["GET".to_string()],
    };
    let _mem_write = CapabilityKind::MemWrite {
        types: vec!["finding".to_string()],
    };
}

#[test]
fn test_usage_reason_enforcement() {
    let result = Grant::new(
        CapabilityKind::NetHttp {
            hosts: vec!["example.com".to_string()],
            methods: vec!["GET".to_string()],
        },
        "",
    );
    assert!(result.is_err());

    let grant = Grant::new(
        CapabilityKind::NetHttp {
            hosts: vec!["example.com".to_string()],
            methods: vec!["GET".to_string()],
        },
        "Fetch top stories each morning",
    );
    assert!(grant.is_ok());
}

#[test]
fn test_in_memory_connector_roundtrip() {
    let conn = InMemoryConnector::new();
    let node = json!({"type": "finding", "content": "test"});
    let rt = tokio::runtime::Runtime::new().unwrap();

    let id = rt.block_on(conn.put_node(node.clone())).expect("put_node");
    assert!(!id.is_empty());

    let retrieved = rt
        .block_on(conn.get_node(&id))
        .expect("get_node")
        .expect("node exists");
    assert_eq!(retrieved, node);

    let calls = conn.calls.lock().unwrap();
    assert!(calls.iter().any(|c| c.contains("put_node")));
    assert!(calls.iter().any(|c| c.contains("get_node")));
}

#[test]
fn test_null_connector_records_calls() {
    let conn = NullConnector::new();
    let rt = tokio::runtime::Runtime::new().unwrap();

    rt.block_on(conn.emit("test:topic", json!({"data": 1})))
        .expect("emit");
    rt.block_on(conn.subscribe("test:topic"))
        .expect("subscribe");

    let calls = conn.calls.lock().unwrap();
    assert_eq!(calls.len(), 2);
    assert!(calls[0].contains("emit"));
    assert!(calls[1].contains("subscribe"));
}

#[test]
fn test_grant_narrowing() {
    let grant = Grant::new(
        CapabilityKind::NetHttp {
            hosts: vec!["example.com".to_string()],
            methods: ["GET", "POST"].iter().map(|s| s.to_string()).collect(),
        },
        "Fetch and post to example.com",
    )
    .expect("valid grant");

    let json_str = serde_json::to_string(&grant).expect("serialize");
    let deserialized: Grant = serde_json::from_str(&json_str).expect("deserialize");
    assert_eq!(grant.usage_reason, deserialized.usage_reason);
}

#[test]
fn test_manifest_creation() {
    let manifest = PackManifest::new("test-pack", "0.1.0", "ed25519:signature123")
        .with_description("A test pack")
        .with_grants(vec![Grant::new(
            CapabilityKind::NetHttp {
                hosts: vec!["example.com".to_string()],
                methods: vec!["GET".to_string()],
            },
            "Fetch data",
        )
        .expect("valid grant")])
        .with_agents(vec!["test-agent".to_string()])
        .with_panes(vec!["test-pane".to_string()]);

    assert_eq!(manifest.name, "test-pack");
    assert_eq!(manifest.version, "0.1.0");
    assert_eq!(manifest.grants.len(), 1);
    assert_eq!(manifest.agents.len(), 1);
    assert_eq!(manifest.panes.len(), 1);

    let json_str = serde_json::to_string(&manifest).expect("serialize");
    let deserialized: PackManifest = serde_json::from_str(&json_str).expect("deserialize");
    assert_eq!(deserialized.name, manifest.name);
    assert_eq!(deserialized.grants.len(), 1);
}

#[test]
fn test_capability_metadata() {
    let http = CapabilityKind::NetHttp {
        hosts: vec!["example.com".to_string()],
        methods: vec!["GET".to_string()],
    };
    assert_eq!(http.description(), "Make HTTP requests");
    assert_eq!(http.danger_level(), Danger::Medium);

    let oauth = CapabilityKind::NetOAuth2 {
        providers: vec!["github".to_string()],
        scopes: vec!["user:email".to_string()],
    };
    assert_eq!(oauth.description(), "Use OAuth2 authentication");
    assert_eq!(oauth.danger_level(), Danger::High);

    let payment = CapabilityKind::PaymentRequest {
        partner: "stripe".to_string(),
        max_usd: 100,
    };
    assert_eq!(payment.danger_level(), Danger::Critical);
}
