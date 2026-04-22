use chief_event_log_proto::schema::Event;
use chief_event_log_proto::EventLog;
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use tempfile::tempdir;

fn append_throughput(c: &mut Criterion) {
    let dir = tempdir().expect("tempdir");
    let log = EventLog::open(dir.path()).expect("open log");
    let event = Event::ToolCall {
        agent: "bench-agent".to_string(),
        tool: "bench.tool".to_string(),
        args_hash: [1; 32],
        result_hash: [2; 32],
    };

    let mut group = c.benchmark_group("append");
    group.throughput(criterion::Throughput::Elements(1));
    group.bench_function(BenchmarkId::new("events_per_sec", "fjall"), |b| {
        b.iter(|| log.append(event.clone()).expect("append"));
    });
    group.finish();
}

criterion_group!(benches, append_throughput);
criterion_main!(benches);
