//! Benchmark: attestation signing overhead (target ≤5ms).

use chief_inference::attest::Attestor;
use chief_inference::canon::{CanonicalOutput, CanonicalPrompt};
use chief_inference::device_key::EphemeralDeviceKey;
use chief_inference::Tier;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::sync::Arc;

fn benchmark_attestation(c: &mut Criterion) {
    c.bench_function("attestation_sign_overhead", |b| {
        let key = Arc::new(EphemeralDeviceKey::new());
        let attestor = Attestor::new(key);
        let prompt = CanonicalPrompt::new("hello world test prompt", Some(0.7), Some(0.9), None);
        let output = CanonicalOutput::new("This is a test output from the model.");
        
        b.iter(|| {
            let _ = attestor.attest(
                black_box("local:llama-7b".to_string()),
                black_box(&prompt),
                black_box(&output),
                black_box(None),
                black_box(Tier::Generated),
            );
        });
    });
}

criterion_group!(benches, benchmark_attestation);
criterion_main!(benches);
