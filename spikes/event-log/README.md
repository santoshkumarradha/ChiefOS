# Chief OS Signed Typed Event Log — Prototype

## Hypothesis

A single Signed Typed Event Log can become the substrate for Chief OS: every tool
call, syscall, agent decision, capability check, and UI event is a typed tuple in
one append-only, content-addressed, signed Merkle chain. Downstream services such
as Memory Graph, Trust Ledger, Search, and Inbox become replayable views over this
one log.

## What This Is

This prototype implements ADR-0005 as a standalone Rust crate:

- typed serde schema for the initial event set
- canonical msgpack event bytes via `rmp-serde`
- blake3 `EventId` values displayed as hex
- append-only `event:<merkle-position>` storage in fjall
- `meta:last_merkle_root`, `meta:height`, and `meta:device_pubkey` metadata
- Ed25519 signatures over `prev_hash || event_bytes || timestamp || device_id`
- full-chain verification that detects tampering anywhere in the log

## Running

```bash
cargo test
cargo clippy --all-targets -- -D warnings
cargo fmt --check
cargo bench
```

## Benchmark Results

Measured on MacBook Pro (Apple Silicon), 2026-04-21:

| Date | Machine | Backend | Throughput | Notes |
|---|---|---|---|---|
| 2026-04-21 | MacBook Pro M-series | fjall | ~900 events/sec | Single-threaded append with crypto and file I/O |

**Analysis:** The measured throughput (~900 events/sec) reflects a single-threaded
sequential append with Ed25519 signing, blake3 hashing, and persistent fjall writes.
Production deployments should:
1. Batch writes (group events before flush)
2. Use concurrent appends with proper locking (tested; see concurrent_append_safety)
3. Profile hot-path crypto (consider SIMD blake3, hardware acceleration where available)
4. Consider async/tokio for I/O-bound scenarios

The chain verification test confirms correctness over concurrent appends (800 events
across 8 threads with full chain validation).

## Success Criteria

- [x] Merkle chain verification with tampering detection
- [x] typed schema for syscall, tool call, agent decision, UI action, and capability events
- [x] concurrent append safety (8 threads × 100 events each verified)
- [x] 64-bit position-based ordering guarantees
- [ ] 10k events/sec throughput (current: ~900 sequential; concurrent mode unlocks higher)

## Event Schema (7 variants)

1. **SyscallWrap** — `{ syscall, fd, args_hash }`
2. **ToolCall** — `{ agent, tool, args_hash, result_hash }`
3. **AgentDecision** — `{ agent, question, choice, rationale_hash }`
4. **UIAction** — `{ surface, action, payload_hash }`
5. **CapabilityIssued** — `{ grant }`
6. **CapabilityRevoked** — `{ grant_id }`
7. **CapabilityCheck** — `{ principal, op, allowed }`

## Device Key Migration

`InMemoryDeviceKey` is prototype-only. Production should implement the
`DeviceKey` trait with a non-exportable platform key:

1. Generate or load a TPM2 / Secure Enclave Ed25519-compatible device identity.
2. Return a stable device id derived from the public key or platform attestation.
3. Sign `full_form` bytes inside the secure boundary.
4. Store the public key and attestation material in event metadata.
5. Keep verification public-key only so historical logs remain externally auditable.

## Cleanup Date

**2026-05-21** — integrate into `chief-kernel`, replace the in-memory key with a
TPM/SE-backed `DeviceKey`, or archive the prototype with ADR-0005 results.

## Related

- [`ADR-0005`](../../adr/0005-signed-typed-event-log.md)
- [`docs/03-chief-kernel.md`](../../docs/03-chief-kernel.md)
