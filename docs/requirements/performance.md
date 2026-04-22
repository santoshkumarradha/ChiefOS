---
id: req-performance
title: "Performance Requirements"
status: draft
owners: [santosh]
last_updated: 2026-04-21
related: [non-functional, v0-scope]
tags: [requirements, performance, latency]
---

# Performance Requirements

Append-only. Each requirement has a stable ID `P-NNN`.

## v0 explicit non-goals

- **Boot-time optimization is NOT a v0 requirement.** Explicit user directive.
- No real-time scheduling guarantees.
- No sub-100ms UI responsiveness target.

Focus v0 perf work on: Morning Brief render, Capability Broker hot path, and Omnibar search.

## Latency targets (reference hardware: modern x86_64 or Apple Silicon, 32GB RAM, NVMe)

| ID | Metric | p50 | p99 |
|---|---|---|---|
| P-001 | Morning Brief render from pre-computed state | 3000 ms | 5000 ms |
| P-002 | Capability Broker check (in-proc / socket) | 5 ms | 20 ms |
| P-003 | Capability Broker check (vsock / microVM) | 20 ms | 60 ms |
| P-004 | Memory Graph retrieval (k=20, 100k nodes) | 200 ms | 800 ms |
| P-005 | Omnibar search | 500 ms | 1500 ms |
| P-006 | Local inference — summarization (Qwen Q4 32B, GPU) | 2000 ms | 8000 ms |
| P-007 | Local inference — summarization (CPU fallback) | 10000 ms | 30000 ms |
| P-008 | Cloud twin round-trip (Haiku-tier) | 800 ms | 2500 ms |
| P-009 | Cloud twin round-trip (Sonnet-tier) | 1500 ms | 4000 ms |
| P-010 | Provenance Log append | 2 ms | 10 ms |
| P-011 | Provenance chain reconstruction (depth ≤10) | 50 ms | 200 ms |
| P-012 | Pack install (official, signed, cold) | 10000 ms | 30000 ms |
| P-013 | Rollback (1 generation) | 5000 ms | 15000 ms |
| P-014 | Rewind (4h, files + memory + queues) | 3000 ms | 10000 ms |

## Throughput

| ID | Metric | Target |
|---|---|---|
| P-101 | Concurrent agents (hero-demo) | ≥ 20 at 60fps Live View |
| P-102 | Events / sec into event log | ≥ 10k sustained |
| P-103 | Memory graph writes / sec | ≥ 1k sustained |
| P-104 | Omnibar queries / sec | ≥ 50 sustained (single-user) |
| P-105 | Pack registry lookups / sec (v1) | ≥ 10k concurrent |

## Surface rendering

| ID | Requirement |
|---|---|
| P-201 | Live Agent View maintains 60fps with 100 active agent nodes. |
| P-202 | Morning Brief card animations use physics (spring), not linear tweens. |
| P-203 | Surface state syncs to phone/watch within 2s of change (v1+). |

## Memory footprint

| ID | Metric | Target |
|---|---|---|
| P-301 | L2 kernel services total RSS | ≤ 2GB |
| P-302 | Live Agent View renderer RSS | ≤ 500MB |
| P-303 | Local model (Q4 32B loaded) | ≤ 20GB (fits 32GB RAM dev machine) |
| P-304 | Local model (Q4 13B loaded) | ≤ 10GB (fits 16GB RAM minimum) |

## Power (laptop class)

| ID | Requirement |
|---|---|
| P-401 | Idle background agents: laptop should last ≥ 6 hours on battery with Chief running (but not actively inferring). |
| P-402 | Active inference (local Q4 model) acceptable on AC power; not expected to be sustainable on battery. |
| P-403 | Overnight synthesis scheduled for AC-power windows by default. |

## Benchmarks (CI-enforced where possible)

| ID | Benchmark | Cadence | Kill gate |
|---|---|---|---|
| P-501 | Morning Brief render perf (synthetic 50-card workload) | per PR | > 5s p99 |
| P-502 | Capability Broker microbenchmark (1M checks) | nightly | > 25ms p99 |
| P-503 | Memory Graph retrieval (100k synthetic nodes) | nightly | > 1s p99 |
| P-504 | Local inference throughput (reference prompt set) | weekly | regression > 20% |
| P-505 | Pack install cold start | per release | > 60s p99 |

## Observability

| ID | Requirement |
|---|---|
| P-601 | Per-L2-service latency histograms exposed via event log. |
| P-602 | Morning Brief render trace available for inspection (Performance Inspector). |
| P-603 | Flame-graph generation supported (for internal debugging; user-exposed only in dev mode). |

## Related

- [`non-functional.md`](./non-functional.md) — NFR cross-reference
- [`../50-v0-scope-90-day.md`](../50-v0-scope-90-day.md) — v0 commitments that drive these targets
- [`../51-risks-open-questions.md`](../51-risks-open-questions.md) — risks that touch perf
