use chief_mem::{ChiefMem, Horizon, Node, NodeType};
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use std::path::PathBuf;

const NODE_COUNT: usize = 100_000;

fn query_bench(c: &mut Criterion) {
    let dir = std::env::temp_dir().join("chief-mem-query-bench");
    let mem = ChiefMem::open(&dir).expect("open memory graph");
    if mem.node_count().expect("node count") < NODE_COUNT as u64 {
        seed(&mem);
    }

    let mut group = c.benchmark_group("hybrid_query_100k");
    group.sample_size(20);
    for query in ["topic7 owner7", "topic42 owner42", "topic233 shard9"] {
        group.bench_with_input(BenchmarkId::from_parameter(query), query, |bench, query| {
            bench.iter(|| {
                let results = mem.query(None, None, query, 20).expect("query");
                assert!(!results.is_empty());
            });
        });
    }
    group.finish();
}

fn seed(mem: &ChiefMem) {
    for idx in 0..NODE_COUNT {
        let node_type = match idx % 3 {
            0 => NodeType::File,
            1 => NodeType::Email,
            _ => NodeType::Person,
        };
        let horizon = match idx % 4 {
            0 => Horizon::Short,
            1 => Horizon::Medium,
            2 => Horizon::Long,
            _ => Horizon::Open,
        };
        let body = format!("topic{} owner{} shard{}", idx % 256, idx % 128, idx % 32);
        mem.put_node(Node::new(node_type, horizon, "bench", body))
            .expect("insert bench node");
    }
}

#[allow(dead_code)]
fn bench_dir() -> PathBuf {
    std::env::temp_dir().join("chief-mem-query-bench")
}

criterion_group!(benches, query_bench);
criterion_main!(benches);
