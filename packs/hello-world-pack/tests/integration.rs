//! Integration tests — hello-world pack under InMemoryConnector.

use chief_sdk::prelude::*;
use hello_world_pack::{make_manifest, HelloWorldAgent, HelloWorldPack};
use std::sync::Arc;

#[tokio::test]
async fn test_hello_world_agent_with_in_memory() {
    // Set up the in-memory connector
    let net = Arc::new(InMemoryConnector::new());
    let mem = Arc::new(InMemoryConnector::new());
    let llm = Arc::new(NullConnector::new());
    let events = Arc::new(InMemoryConnector::new());

    // Create context
    let ctx = CapabilityContext::new(net.clone(), mem.clone(), llm, events.clone());

    // Run the agent
    let agent = HelloWorldAgent;
    let result = agent.on_tick(ctx).await;
    assert!(result.is_ok(), "Agent should succeed");

    // Verify Memory Graph write
    let mem_calls = mem.calls.lock().unwrap();
    assert!(mem_calls.iter().any(|c| c.contains("put_node")));

    // Verify event emission
    let event_calls = events.calls.lock().unwrap();
    assert!(event_calls.iter().any(|c| c.contains("emit")));
}

#[tokio::test]
async fn test_pack_lifecycle() {
    let pack = HelloWorldPack;

    // on_init should succeed
    assert!(pack.on_init().await.is_ok());

    // on_enable should succeed
    assert!(pack.on_enable().await.is_ok());

    // on_disable should succeed
    assert!(pack.on_disable().await.is_ok());
}

#[test]
fn test_manifest_creation() {
    let manifest = make_manifest();

    // Basic metadata
    assert_eq!(manifest.name, "hello-world");
    assert_eq!(manifest.version, "0.1.0");

    // Grants are present and valid
    assert_eq!(manifest.grants.len(), 3);
    for grant in &manifest.grants {
        assert!(
            !grant.usage_reason.is_empty(),
            "usage_reason must be non-empty"
        );
    }

    // Agents are declared
    assert_eq!(manifest.agents.len(), 1);
    assert_eq!(manifest.agents[0], "hello-world-agent");
}

#[test]
fn test_manifest_grants_have_usage_reason() {
    let manifest = make_manifest();

    for grant in &manifest.grants {
        match &grant.kind {
            CapabilityKind::NetHttp { .. } => {
                assert!(grant.usage_reason.contains("example.com"));
            }
            CapabilityKind::MemWrite { .. } => {
                assert!(grant.usage_reason.contains("Memory Graph"));
            }
            CapabilityKind::EventEmit { .. } => {
                assert!(grant.usage_reason.contains("events"));
            }
            _ => panic!("Unexpected capability kind"),
        }
    }
}

#[test]
fn test_manifest_serialization() {
    let manifest = make_manifest();
    let json = serde_json::to_string(&manifest).expect("serialize");
    let restored: PackManifest = serde_json::from_str(&json).expect("deserialize");

    assert_eq!(manifest.name, restored.name);
    assert_eq!(manifest.grants.len(), restored.grants.len());
}
