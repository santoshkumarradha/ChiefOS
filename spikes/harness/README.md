# Chief Harness Prototype

## Hypothesis

The harness interface (Agent Runtime's backend) is swappable without requiring changes to Chief Kernel core.

## Implementation

A Rust crate (`chief-harness-proto`) that:

1. Defines the `Harness` async trait with 5 methods per `docs/03-chief-kernel.md` Service #1:
   - `start(task_spec) -> HarnessHandle` — begin a task
   - `submit_action(handle, action) -> Receipt` — queue an action
   - `submit_tool_call(handle, tool_id, args) -> Result` — invoke a tool
   - `stream_events(handle) -> Stream<Event>` — subscribe to events
   - `terminate(handle, reason)` — stop the harness

2. **OpencodeHarness** — shells out to the `opencode` CLI. Gracefully degrades if opencode isn't installed (logs warnings, mocks operations).

3. **NullHarness** — in-memory test double that records all operations to a call log for deterministic testing.

4. **Factory function** `build_harness(kind: HarnessKind) -> Box<dyn Harness>` for pluggable initialization.

## Building and Testing

```bash
# Inside prototypes/harness/
cargo test

# Format check
cargo fmt --check

# Lint
cargo clippy --all-targets -- -D warnings
```

### Test Output

Both implementations pass the same test suite (`swap_test.rs`):

- `test_null_harness_swappability` ✓ — validates NullHarness against full sequence
- `test_opencode_harness_swappability` ✓ — validates OpencodeHarness against same sequence
- `test_null_harness_call_recording` ✓ — validates recording fidelity
- `test_factory_builds_both_kinds` ✓ — factory dispatch works for both

## Key Decisions

- **No workspace `Cargo.toml`:** Kept standalone for this prototype. Workspace will be added by a later task.
- **Graceful degradation:** OpencodeHarness doesn't fail if opencode CLI isn't found; it logs and mocks.
- **NullHarness is deterministic:** Call log captures every operation in order, enabling test replay.
- **Trait is async:** All methods are `async` to accommodate real backends with I/O.
- **Streaming trait:** `stream_events` returns a boxed `Stream` to match async runtime patterns.

## Success Criteria

- ✓ Same test suite passes for both OpencodeHarness and NullHarness
- ✓ `cargo test` runs without errors
- ✓ `cargo fmt --check` passes
- ✓ `cargo clippy --all-targets -- -D warnings` passes
- ✓ Zero shared state with other prototypes
- ✓ No hardcoded paths or OS-specific dependencies

## References

- [`docs/03-chief-kernel.md`](../../docs/03-chief-kernel.md) — Service #1: Agent Runtime, harness interface spec
- [`CHARTER.md`](../../CHARTER.md) — Axiom 1 (AI primary user) & Axiom 7 (boring infrastructure)
- [`CLAUDE.md`](../../CLAUDE.md) — project workflow & PlanDB integration

## CLEANUP

**Date:** 2026-05-21

This prototype validates swappability. After acceptance and merge:

1. Results feed into v0 harness selection architecture decision.
2. OpencodeHarness code may be promoted to a real service if opencode is adopted.
3. NullHarness pattern may be reused for testing other L2 services.
4. Crate becomes part of the workspace when multi-crate coordination is ready.

---

*Prototype created 2026-04-21. Tests run via `cargo test` from this directory.*
