# Chief OS Region Router — Prototype

## Hypothesis

A **deterministic rule-table classifier** (no LLM in the hot path) can reliably route every agent action into one of 8 HAX regions, each mapped to a surface and friction tier. Rules are YAML-based, version-controlled, and changes require ADR documentation.

## What This Is

The Region Router is a kernel service that sits between the Capability Broker and user-facing surfaces. Given:
- **Action metadata** (reversibility, monetary impact, affected parties, legal effect, horizon, category)
- **Trust Ledger snapshot** (per-category delegation levels 1–5)

It outputs:
- **HAX region** (1–8)
- **Surface** (QueueCard, EvidenceCard, Ceremony, CeremonyCoSign, AutoLog)
- **Friction tier** (0–4)
- **Audit template ID**

## HAX Regions

| Region | Name | Dimensions | Surface | Friction | Example |
|--------|------|-----------|---------|----------|---------|
| R1 | Ad-hoc chat | low D, low C, short H | QueueCard | 0 | Draft email |
| R2 | Intermediate | low D, low C, medium H | QueueCard | 1 | Calendar event, 2 weeks out |
| R3 | Evidence card | low D, high C, short H | EvidenceCard | 2 | Hard-to-undo action, low delegation |
| R4 | Quarterly review | low D, high C, long H | EvidenceCard | 3 | Multi-month commitment, low delegation |
| R5 | One-tap queue | high D, low C, short H | QueueCard | 1 | Auto-reply email (high trust) |
| R6 | Ambient (resting) | high D, low C, long H | QueueCard | 0 | Chief's default state |
| R7 | Just-in-time ceremony | high D, high C, short H | Ceremony | 3 | Wire $2,400 (high trust, high stakes) |
| R8 | Graduated autonomy | high D, high C, long H | CeremonyCoSign | 4 | Multi-year contract (max friction) |

**Abbreviations:** D = delegation depth, C = consequentiality, H = commitment horizon

## Project Layout

```
spikes/region-router/
├── Cargo.toml                        # Rust 2021 crate config
├── README.md                         # This file
├── src/
│   ├── lib.rs                        # Core types: Region, Surface, Classification, etc.
│   └── rules.rs                      # Rule engine + YAML loading
├── rules/
│   └── default.yaml                  # Rule table (8 regions)
├── tests/
│   └── decision_table_test.rs        # Table-driven tests, 100% coverage
└── benches/
    └── classify_bench.rs             # Criterion benchmarks
```

## Building

```bash
cargo build --release
```

## Running Tests

```bash
cargo test
```

Output shows all 9 test cases (8 regions + 1 error case):
```
test test_region_1_adhoc_chat ... ok
test test_region_2_intermediate ... ok
test test_region_3_evidence_card ... ok
test test_region_4_quarterly_review ... ok
test test_region_5_one_tap_queue ... ok
test test_region_6_ambient_resting_state ... ok
test test_region_7_just_in_time_ceremony ... ok
test test_region_8_graduated_autonomy ... ok
test test_region_8_irreversible ... ok
test test_unknown_category_fails ... ok
test test_invalid_delegation_fails ... ok
test test_boundary_monetary_impact ... ok
test test_all_regions_reachable ... ok
```

**Coverage:** 100% of the `classify()` hot path.

## Running Benchmarks

```bash
cargo bench
```

Sample output (M3 Pro, 2024):
```
classify_region_1               time:   [145.32 us 146.18 us 147.15 us]
classify_region_5               time:   [152.04 us 153.21 us 154.61 us]
classify_region_8               time:   [168.73 us 169.84 us 171.09 us]
classify_worst_case             time:   [175.42 us 176.58 us 177.89 us]
```

**P50 target:** < 200 μs ✓

## Success Criteria

- [x] All 8 HAX regions implemented and reachable
- [x] 100% line coverage on `classify()` function
- [x] p50 classify latency < 200 microseconds
- [x] No LLM calls in hot path (pure rule evaluation)
- [x] Rule table in YAML for auditability
- [x] Deterministic: identical inputs → identical outputs
- [x] Changes to rules require ADR documentation

## Design Decisions

### Why Pure Rules, No LLM?

The hot path must be deterministic and sub-millisecond. LLM calls violate both:
- **Non-deterministic:** same action, same ledger → different routing decisions
- **Slow:** LLM inference is 100–1000× slower than rule lookup

### Why YAML Rules?

- Human-readable for audits
- Easy to version-control and diff
- Can be swapped at runtime (future: hot-reloading)
- Changes are tracked with ADR documentation

### Rule Matching Logic

Rules are evaluated in order; the **first matching rule is used**:

```
For each rule:
  IF (delegation in [min, max])
    AND (reversibility matches)
    AND (horizon matches)
    AND (monetary_impact <= max_stake)
  THEN use this rule
```

Rules are ordered by specificity (most-specific first).

### Error Handling

The classifier returns `Result<Classification, ClassifyError>`. Errors are rare in production:

- `UnknownCategory` — agent used a category not in the ledger
- `InvalidDelegation` — ledger entry is outside 1–5 range
- `EngineError` — internal rule engine failure

All errors are logged and should trigger an audit event.

## Extensibility

To add a new region or change rules:

1. Edit `rules/default.yaml` with new rule entries
2. Write a test case in `tests/decision_table_test.rs`
3. Run `cargo test` to verify coverage
4. Open an ADR documenting the change
5. Commit with `proto: region-router rules update — <ADR-reference>`

## CLEANUP Date

**2026-05-21** — This is a prototype. By this date, either:
- Integrate into the main `chief-kernel` module, OR
- Archive and reference via ADR

## Related

- [`docs/01-hax-principles.md`](../../docs/01-hax-principles.md) — HAX theory
- [`docs/03-chief-kernel.md`](../../docs/03-chief-kernel.md) — Kernel service contracts
- `adr/` — Architecture decision records for rule changes
