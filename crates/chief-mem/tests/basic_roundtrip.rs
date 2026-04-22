use chief_mem::{ChiefMem, Edge, EdgeKind, Horizon, Node, NodeType};
use tempfile::TempDir;

#[test]
fn test_basic_roundtrip() {
    let tmpdir = TempDir::new().unwrap();
    let db_path = tmpdir.path().join("test.db");

    let store = ChiefMem::open(&db_path).unwrap();

    // Create and store a node
    let node = Node::new(
        NodeType::File,
        Horizon::Medium,
        "test-agent",
        "test content for memory",
    );

    let uri = store.put_node(node.clone()).unwrap();
    assert!(uri.starts_with("mem://file/"));

    // Verify the node was stored
    let retrieved = store.get_node(&uri).unwrap();
    assert!(retrieved.is_some());
    let retrieved_node = retrieved.unwrap();
    assert_eq!(retrieved_node.body, node.body);
    assert_eq!(retrieved_node.source, node.source);

    // Create and store another node
    let node2 = Node::new(
        NodeType::Thought,
        Horizon::Short,
        "test-agent",
        "related thought",
    );
    let uri2 = store.put_node(node2).unwrap();

    // Create an edge between them
    let edge = Edge {
        from: uri.clone(),
        to: uri2.clone(),
        kind: EdgeKind::RefersTo,
        weight: 1.0,
        created_by: "test-agent".to_string(),
    };
    store.put_edge(edge).unwrap();

    // Query edges from first node
    let edges = store.get_edges_from(&uri).unwrap();
    assert_eq!(edges.len(), 1);
    assert_eq!(edges[0].kind, EdgeKind::RefersTo);

    // Verify URI stability: same content -> same URI
    let duplicate = Node::new(
        NodeType::File,
        Horizon::Medium,
        "another-agent",
        "test content for memory",
    );
    let uri_dup = store.put_node(duplicate).unwrap();
    assert_eq!(uri, uri_dup);

    // Count nodes and edges
    let node_count = store.count_nodes().unwrap();
    assert_eq!(node_count, 2);

    let edge_count = store.count_edges().unwrap();
    assert_eq!(edge_count, 1);

    println!("✓ Basic roundtrip test passed");
}

#[test]
fn test_multiple_types() {
    let tmpdir = TempDir::new().unwrap();
    let db_path = tmpdir.path().join("test_types.db");
    let store = ChiefMem::open(&db_path).unwrap();

    let types = vec![
        NodeType::Email,
        NodeType::File,
        NodeType::Person,
        NodeType::Event,
        NodeType::Decision,
        NodeType::Artifact,
        NodeType::Thought,
        NodeType::Finding,
    ];

    for (_i, node_type) in types.iter().enumerate() {
        let node = Node::new(
            *node_type,
            Horizon::Medium,
            "test",
            format!("content for {}", node_type),
        );
        let uri = store.put_node(node).unwrap();
        assert!(uri.contains(&node_type.to_string()));
    }

    assert_eq!(store.count_nodes().unwrap(), types.len() as i64);
    println!("✓ All 8 node types stored successfully");
}
