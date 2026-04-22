//! Integration tests - File-watcher Briefer pack under InMemoryConnector.

use chief_sdk::prelude::*;
use file_watcher_briefer::{make_manifest, FileWatcherAgent, FileWatcherPack};
use std::sync::Arc;

#[tokio::test]
async fn test_file_watcher_agent_full_flow() {
    let net = Arc::new(InMemoryConnector::new());
    let mem = Arc::new(InMemoryConnector::new());
    let llm = Arc::new(InMemoryConnector::new());
    let events = Arc::new(InMemoryConnector::new());

    let ctx = CapabilityContext::new(net, mem.clone(), llm, events);

    let agent = FileWatcherAgent;
    let result = agent.on_tick(ctx).await;
    assert!(result.is_ok(), "Agent should succeed: {:?}", result.err());

    let mem_calls = mem.calls.lock().unwrap();
    let put_node_calls: Vec<_> = mem_calls
        .iter()
        .filter(|call| call.contains("put_node"))
        .collect();
    assert!(
        put_node_calls.len() >= 4,
        "Should write at least 4 memory nodes (3 files + 1 card), got: {}",
        put_node_calls.len()
    );

    let memory = mem.memory.lock().unwrap();
    let thoughts: Vec<_> = memory
        .values()
        .filter(|node| node.get("type").and_then(|v| v.as_str()) == Some("thought"))
        .collect();
    assert!(
        thoughts.len() >= 3,
        "Should write at least 3 per-file thought nodes, got: {}",
        thoughts.len()
    );

    let card = memory
        .values()
        .find(|node| node.get("type").and_then(|v| v.as_str()) == Some("card"))
        .expect("Should have created an aggregate card node");

    assert_eq!(
        card.get("badge").and_then(|v| v.as_str()),
        Some("handled"),
        "Card should use the handled badge string"
    );
    assert_eq!(
        card.get("source_color").and_then(|v| v.as_str()),
        Some("green"),
        "Card should use the green source avatar color"
    );

    let items = card
        .get("items")
        .and_then(|v| v.as_array())
        .expect("Card should include summarized file items");
    assert_eq!(items.len(), 3, "Stubbed filesystem should produce 3 files");
    assert!(items.iter().all(|item| item
        .get("path")
        .and_then(|v| v.as_str())
        .is_some_and(|path| path.ends_with(".md"))));
}

#[tokio::test]
async fn test_pack_lifecycle() {
    let pack = FileWatcherPack;

    assert!(pack.on_init().await.is_ok());
    assert!(pack.on_enable().await.is_ok());
    assert!(pack.on_disable().await.is_ok());
}

#[test]
fn test_manifest_creation() {
    let manifest = make_manifest();

    assert_eq!(manifest.name, "file-watcher-briefer");
    assert_eq!(manifest.version, "0.1.0");
    assert_eq!(
        manifest.grants.len(),
        5,
        "Should have exactly 5 grants: fs.watch, fs.read, mem.write, surface.pane, llm.ai"
    );

    for grant in &manifest.grants {
        assert!(
            !grant.usage_reason.trim().is_empty(),
            "Grant {:?} has empty usage_reason",
            grant.kind
        );
    }
}

#[test]
fn test_manifest_grants_have_expected_scopes() {
    let manifest = make_manifest();

    let mut fs_watch_count = 0;
    let mut fs_read_count = 0;
    let mut mem_write_count = 0;
    let mut surface_pane_count = 0;
    let mut llm_ai_count = 0;

    for grant in &manifest.grants {
        match &grant.kind {
            CapabilityKind::FsWatch { paths } => {
                fs_watch_count += 1;
                assert_eq!(
                    paths,
                    &vec![
                        "~/Documents/**/*.md".to_string(),
                        "~/Documents/**/*.txt".to_string(),
                    ]
                );
            }
            CapabilityKind::FsRead { paths } => {
                fs_read_count += 1;
                assert_eq!(paths, &vec!["~/Documents/**".to_string()]);
            }
            CapabilityKind::MemWrite { types } => {
                mem_write_count += 1;
                assert!(types.contains(&"thought".to_string()));
                assert!(types.contains(&"card".to_string()));
            }
            CapabilityKind::SurfacePane { surfaces, .. } => {
                surface_pane_count += 1;
                assert!(surfaces.contains(&"morning-brief".to_string()));
            }
            CapabilityKind::LlmAi { max_tokens, tier } => {
                llm_ai_count += 1;
                assert_eq!(*max_tokens, 500);
                assert_eq!(tier, "fast");
            }
            _ => panic!("Unexpected capability kind in manifest: {:?}", grant.kind),
        }
    }

    assert_eq!(fs_watch_count, 1, "Should have 1 fs.watch grant");
    assert_eq!(fs_read_count, 1, "Should have 1 fs.read grant");
    assert_eq!(mem_write_count, 1, "Should have 1 mem.write grant");
    assert_eq!(surface_pane_count, 1, "Should have 1 surface.pane grant");
    assert_eq!(llm_ai_count, 1, "Should have 1 llm.ai grant");
}

#[test]
fn test_manifest_agents() {
    let manifest = make_manifest();
    assert_eq!(manifest.agents.len(), 1);
    assert_eq!(manifest.agents[0], "file-watcher-agent");
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
    let source = include_str!("../src/lib.rs");
    assert!(!source.contains("chief_core"));
    assert!(!source.contains("chief_ui"));
    assert!(!source.contains("chief_oauth"));

    let _ = FileWatcherAgent;
    let _ = FileWatcherPack;
}
