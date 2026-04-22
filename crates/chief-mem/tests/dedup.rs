use chief_mem::{ChiefMem, Horizon, Node, NodeType};
use tempfile::TempDir;

#[test]
fn test_dedup_same_content() {
    let tmpdir = TempDir::new().unwrap();
    let db_path = tmpdir.path().join("dedup_test.db");
    let store = ChiefMem::open(&db_path).unwrap();

    let content = "this is the exact same content for dedup test";

    // Insert same content multiple times
    let mut uris = vec![];
    for i in 0..5 {
        let node = Node::new(
            NodeType::File,
            if i % 2 == 0 {
                Horizon::Short
            } else {
                Horizon::Long
            },
            format!("agent-{}", i),
            content,
        );
        let uri = store.put_node(node).unwrap();
        uris.push(uri);
    }

    // All URIs should be identical (same content = same hash = same URI)
    for (i, uri) in uris.iter().enumerate() {
        assert_eq!(uri, &uris[0], "URI {} differs from URI 0", i);
    }

    // But only one node should exist in the database
    let count = store.count_nodes().unwrap();
    assert_eq!(
        count, 1,
        "Only 1 unique node should exist due to content-based dedup"
    );

    println!(
        "✓ Deduplication test PASSED: {} inserts -> {} unique node",
        uris.len(),
        count
    );
    println!("  URI: {}", uris[0]);
}

#[test]
fn test_dedup_different_content() {
    let tmpdir = TempDir::new().unwrap();
    let db_path = tmpdir.path().join("dedup_diff_test.db");
    let store = ChiefMem::open(&db_path).unwrap();

    // Insert different content
    let mut uris = vec![];
    for i in 0..10 {
        let node = Node::new(
            NodeType::Thought,
            Horizon::Medium,
            "test",
            format!("unique content {}", i),
        );
        let uri = store.put_node(node).unwrap();
        uris.push(uri);
    }

    // All URIs should be different
    for i in 0..uris.len() {
        for j in (i + 1)..uris.len() {
            assert_ne!(
                &uris[i], &uris[j],
                "URIs should differ for different content"
            );
        }
    }

    // All nodes should exist
    let count = store.count_nodes().unwrap();
    assert_eq!(count, 10);

    println!("✓ Different content test PASSED: {} unique nodes", count);
}
