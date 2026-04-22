//! Criterion benchmark for Region Router classification performance
//!
//! Measures p50 (median), p95, p99 latency of the classify() function.
//! Target: p50 < 200 microseconds on modern hardware.

use chief_region_router_proto::*;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::collections::HashMap;

fn bench_classify(c: &mut Criterion) {
    c.bench_function("classify_region_1", |b| {
        let mut ledger = HashMap::new();
        ledger.insert("email".to_string(), 1);

        let action = ActionMetadata {
            reversibility_class: ReversibilityClass::Soft,
            monetary_impact_usd: 50.0,
            affected_parties: 1,
            legal_effect: LegalEffect::None,
            time_to_detect: TimeToDetect::Seconds,
            horizon: Horizon::Short,
            category: "email".to_string(),
        };

        b.iter(|| classify(black_box(&action), black_box(&ledger)))
    });

    c.bench_function("classify_region_5", |b| {
        let mut ledger = HashMap::new();
        ledger.insert("email".to_string(), 5);

        let action = ActionMetadata {
            reversibility_class: ReversibilityClass::Soft,
            monetary_impact_usd: 500.0,
            affected_parties: 1,
            legal_effect: LegalEffect::None,
            time_to_detect: TimeToDetect::Seconds,
            horizon: Horizon::Short,
            category: "email".to_string(),
        };

        b.iter(|| classify(black_box(&action), black_box(&ledger)))
    });

    c.bench_function("classify_region_8", |b| {
        let mut ledger = HashMap::new();
        ledger.insert("contracts".to_string(), 5);

        let action = ActionMetadata {
            reversibility_class: ReversibilityClass::Irreversible,
            monetary_impact_usd: 20000.0,
            affected_parties: 10,
            legal_effect: LegalEffect::Binding,
            time_to_detect: TimeToDetect::Days,
            horizon: Horizon::Open,
            category: "contracts".to_string(),
        };

        b.iter(|| classify(black_box(&action), black_box(&ledger)))
    });

    c.bench_function("classify_worst_case", |b| {
        let mut ledger = HashMap::new();
        ledger.insert("finance".to_string(), 3);

        let action = ActionMetadata {
            reversibility_class: ReversibilityClass::Hard,
            monetary_impact_usd: 2400.0,
            affected_parties: 2,
            legal_effect: LegalEffect::Binding,
            time_to_detect: TimeToDetect::Hours,
            horizon: Horizon::Short,
            category: "finance".to_string(),
        };

        b.iter(|| classify(black_box(&action), black_box(&ledger)))
    });
}

criterion_group!(benches, bench_classify);
criterion_main!(benches);
