//! Integration tests — HN Briefer pack under InMemoryConnector.

use chief_sdk::prelude::*;
use hn_briefer::{make_manifest, HnBrieferAgent, HnBrieferPack};
use std::sync::Arc;

#[tokio::test]
async fn test_hn_briefer_agent_full_flow() {
    // Set up the in-memory connector
    let net = Arc::new(InMemoryConnector::new());
    let mem = Arc::new(InMemoryConnector::new());
    let llm = Arc::new(InMemoryConnector::new());
    let events = Arc::new(InMemoryConnector::new());

    // Create context
    let ctx = CapabilityContext::new(net.clone(), mem.clone(), llm, events.clone());

    // Run the agent
    let agent = HnBrieferAgent;
    let result = agent.on_tick(ctx).await;
    assert!(result.is_ok(), "Agent should succeed: {:?}", result.err());

    // Verify Memory Graph writes (should have thought nodes + one card)
    let mem_calls = mem.calls.lock().unwrap();
    let put_node_calls: Vec<_> = mem_calls
        .iter()
        .filter(|c| c.contains("put_node"))
        .collect();
    // Stub provides 2 HN + 2 Substack = 4 items, then 1 card = 5 total nodes
    assert!(
        put_node_calls.len() >= 5,
        "Should have written at least 5 memory nodes (4 items + 1 card), got: {}",
        put_node_calls.len()
    );

    // Verify event emission
    let event_calls = events.calls.lock().unwrap();
    assert!(
        event_calls.iter().any(|c| c.contains("emit")),
        "Should have emitted an event"
    );

    // Verify memory contents
    let memory = mem.memory.lock().unwrap();
    let card_node = memory
        .values()
        .find(|v| v.get("type").and_then(|t| t.as_str()) == Some("card"));
    assert!(
        card_node.is_some(),
        "Should have created a card node in memory"
    );

    if let Some(card) = card_node {
        assert_eq!(
            card.get("source").and_then(|s| s.as_str()),
            Some("HN Briefer"),
            "Card should have source='HN Briefer'"
        );
        assert_eq!(
            card.get("badge").and_then(|b| b.as_str()),
            Some("handled"),
            "Card should have badge='handled'"
        );
    }
}

#[tokio::test]
async fn test_hn_briefer_agent_http_calls() {
    let net = Arc::new(InMemoryConnector::new());
    let mem = Arc::new(InMemoryConnector::new());
    let llm = Arc::new(InMemoryConnector::new());
    let events = Arc::new(InMemoryConnector::new());

    let ctx = CapabilityContext::new(net.clone(), mem.clone(), llm, events.clone());

    let agent = HnBrieferAgent;
    let _ = agent.on_tick(ctx).await;

    // Verify HTTP calls were made to the right hosts
    let net_calls = net.calls.lock().unwrap();
    let hn_calls: Vec<_> = net_calls
        .iter()
        .filter(|c| c.contains("hn.algolia.com") || c.contains("news.ycombinator.com"))
        .collect();
    assert!(
        !hn_calls.is_empty(),
        "Should have made HTTP call to HN/Algolia"
    );

    let substack_calls: Vec<_> = net_calls
        .iter()
        .filter(|c| c.contains("substack.com"))
        .collect();
    assert!(
        !substack_calls.is_empty(),
        "Should have made HTTP call to Substack"
    );
}

#[tokio::test]
async fn test_hn_briefer_agent_llm_scoring() {
    let net = Arc::new(InMemoryConnector::new());
    let mem = Arc::new(InMemoryConnector::new());
    let llm = Arc::new(InMemoryConnector::new());
    let events = Arc::new(InMemoryConnector::new());

    let ctx = CapabilityContext::new(net.clone(), mem.clone(), llm.clone(), events.clone());

    let agent = HnBrieferAgent;
    let result = agent.on_tick(ctx).await;

    // Verify the agent succeeds with LLM scoring via ctx.ai()
    assert!(
        result.is_ok(),
        "Agent should succeed with AI scoring: {:?}",
        result.err()
    );
}

#[tokio::test]
async fn test_pack_lifecycle() {
    let pack = HnBrieferPack;

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
    assert_eq!(manifest.name, "hn-briefer");
    assert_eq!(manifest.version, "0.1.0");

    // All 5 grants are present
    assert_eq!(
        manifest.grants.len(),
        5,
        "Should have exactly 5 grants (net.http x2, mem.write, surface.pane, llm.ai)"
    );

    // Every grant must have non-empty usage_reason
    for grant in &manifest.grants {
        assert!(
            !grant.usage_reason.trim().is_empty(),
            "Grant {:?} has empty usage_reason",
            grant.kind
        );
    }
}

#[test]
fn test_manifest_grants_have_usage_reason() {
    let manifest = make_manifest();

    let mut net_http_count = 0;
    let mut mem_write_count = 0;
    let mut surface_pane_count = 0;
    let mut llm_ai_count = 0;

    for grant in &manifest.grants {
        match &grant.kind {
            CapabilityKind::NetHttp { hosts, .. } => {
                net_http_count += 1;
                assert!(
                    grant.usage_reason.contains("HTTP") || grant.usage_reason.contains("Fetch")
                );
                // First grant should be HN
                if net_http_count == 1 {
                    assert!(hosts.iter().any(|h| h.contains("hn.algolia.com")));
                }
                // Second should be Substack
                if net_http_count == 2 {
                    assert!(hosts.iter().any(|h| h.contains("substack.com")));
                }
            }
            CapabilityKind::MemWrite { types, .. } => {
                mem_write_count += 1;
                assert!(grant.usage_reason.contains("Memory Graph"));
                assert!(types.contains(&"thought".to_string()));
                assert!(types.contains(&"card".to_string()));
            }
            CapabilityKind::SurfacePane { surfaces, .. } => {
                surface_pane_count += 1;
                assert!(grant.usage_reason.contains("Brief"));
                assert!(surfaces.contains(&"morning-brief".to_string()));
            }
            CapabilityKind::LlmAi { .. } => {
                llm_ai_count += 1;
                assert!(grant.usage_reason.contains("score"));
            }
            _ => panic!("Unexpected capability kind in manifest: {:?}", grant.kind),
        }
    }

    assert_eq!(net_http_count, 2, "Should have 2 net.http grants");
    assert_eq!(mem_write_count, 1, "Should have 1 mem.write grant");
    assert_eq!(surface_pane_count, 1, "Should have 1 surface.pane grant");
    assert_eq!(llm_ai_count, 1, "Should have 1 llm.ai grant");
}

#[test]
fn test_manifest_agents() {
    let manifest = make_manifest();
    assert_eq!(manifest.agents.len(), 1);
    assert_eq!(manifest.agents[0], "hn-briefer-agent");
}

#[test]
fn test_manifest_serialization() {
    let manifest = make_manifest();
    let json = serde_json::to_string(&manifest).expect("serialize");
    let restored: PackManifest = serde_json::from_str(&json).expect("deserialize");

    assert_eq!(manifest.name, restored.name);
    assert_eq!(manifest.version, restored.version);
    assert_eq!(manifest.grants.len(), restored.grants.len());
}

#[test]
fn test_no_internal_imports() {
    // This is a compile-time check: if this module can import and use
    // HnBrieferAgent and HnBrieferPack without errors, the public API
    // is correct. The actual check is in src/lib.rs constraints.
    let _ = HnBrieferAgent;
    let _ = HnBrieferPack;
}
