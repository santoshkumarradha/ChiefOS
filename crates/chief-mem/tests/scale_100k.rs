use chief_mem::{ChiefMem, Horizon, Node, NodeType};
use std::time::Instant;
use tempfile::TempDir;

#[test]
fn test_scale_100k_nodes() {
    let tmpdir = TempDir::new().unwrap();
    let db_path = tmpdir.path().join("scale_test.db");

    let store = ChiefMem::open(&db_path).unwrap();

    println!("\n=== Scale Test: 100k Synthetic Nodes ===");

    // Generate and insert synthetic nodes
    let types = vec![NodeType::File, NodeType::Email, NodeType::Person];
    let horizons = vec![
        Horizon::Short,
        Horizon::Medium,
        Horizon::Long,
        Horizon::Open,
    ];

    let start = Instant::now();
    for i in 0..100_000 {
        let node_type = types[i % types.len()];
        let horizon = horizons[i % horizons.len()];
        let content = format!("synthetic node content {}", i);

        let node = Node::new(node_type, horizon, "synthetic-gen", content);
        store.put_node(node).ok();
    }
    let insert_time = start.elapsed();

    let node_count = store.count_nodes().unwrap();
    assert_eq!(node_count, 100_000);

    println!(
        "✓ Inserted 100,000 nodes in {:.2}s ({:.0} nodes/s)",
        insert_time.as_secs_f64(),
        100_000.0 / insert_time.as_secs_f64()
    );

    // Check database file size
    let metadata = std::fs::metadata(&db_path).unwrap();
    let size_mb = metadata.len() as f64 / (1024.0 * 1024.0);
    println!("✓ Database file size: {:.1} MB", size_mb);
    assert!(size_mb < 200.0, "Database size should be < 200 MB");

    // Run queries and measure p50 latency
    let queries = vec![("synthetic", 20), ("node", 20), ("content", 20)];

    let mut latencies = vec![];
    for (query_text, k) in queries {
        let start = Instant::now();
        let _results = store.query(query_text, None, None, k).ok();
        let elapsed = start.elapsed().as_millis() as u64;
        latencies.push(elapsed);
        println!(
            "  Query '{}' (k={}): {:.1} ms",
            query_text, k, elapsed as f64
        );
    }

    latencies.sort();
    let p50 = if !latencies.is_empty() {
        latencies[latencies.len() / 2]
    } else {
        0
    };
    println!("✓ p50 query latency: {} ms", p50);

    println!("✓ Scale test PASSED\n");
}
