---
id: req-non-functional
title: "Non-Functional Requirements"
status: draft
owners: [santosh]
last_updated: 2026-04-21
related: [v0-scope, architecture, apple-design]
tags: [requirements, nfr, quality]
---

# Non-Functional Requirements

Append-only. Each requirement has a stable ID `NF-NNN`.

## Language policy (polyglot)

| ID | Requirement |
|---|---|
| NF-001 | Pick the right language per layer. No mandatory single language. |
| NF-002 | L2 kernel services (Broker, Router, Provenance, Trust Ledger, Model Router) in **Rust** (memory safety, fuzz-friendly). |
| NF-003 | L2 network-heavy services (cloud-twin bridge, MCP transport shims, Event Bus) in **Go**. |
| NF-004 | L3 SDK in **TypeScript + Python**. |
| NF-005 | L4 surfaces in **TypeScript** (Tauri/Wry + React or Svelte). |
| NF-006 | Agents / ingesters in **Python or TypeScript**. |
| NF-007 | L1 glue in **Nix + Bash**. |

## Observability & provenance

| ID | Requirement |
|---|---|
| NF-101 | Every agent action emits a typed event to the signed typed event log (substrate primitive #1). |
| NF-102 | Provenance Log retention ≥ 7 years; Merkle-linked; externally verifiable. |
| NF-103 | Every capability check produces a DEBUG event; every deny produces an INFO event. |
| NF-104 | eBPF-based observability for kernel-level actions (net, fs, process). |
| NF-105 | Live Agent View at 60fps with ≥20 simultaneous agents. |

## Reliability

| ID | Requirement |
|---|---|
| NF-201 | A single L2 service failure degrades but does not crash the OS. Watchdog restarts with backoff. |
| NF-202 | `chief update` atomically switches generations; auto-rollback on failed health check. |
| NF-203 | Cloud twin failures degrade non-sensitive work; sensitive work unaffected. |
| NF-204 | Rewind operation is atomic across files, memory, queued actions. No partial rewinds. |

## Security

| ID | Requirement |
|---|---|
| NF-301 | No ambient authority. Every call goes through Capability Broker. |
| NF-302 | Hardware-bound approval tokens single-use, short-lived, non-replayable. |
| NF-303 | All persistent data encrypted at rest (LUKS v0; per-file keying v1). |
| NF-304 | All network traffic encrypted in transit. |
| NF-305 | Cloud twin receives only encrypted blobs for sensitive-horizon nodes. |
| NF-306 | All packs signed; installs without valid signature rejected. |
| NF-307 | Capability grants are typed, scoped, expiring, revocable. |

## Privacy

| ID | Requirement |
|---|---|
| NF-401 | Source of truth lives on the user's device. Cloud twin is stateless. |
| NF-402 | Sensitive categories (finance, health, personal, family, legal.private) never leave the device. |
| NF-403 | Encrypted cloud replica is opt-in and off-by-default for sensitive categories. |
| NF-404 | User-visible "on-device" indicator whenever sensitive ops run. |
| NF-405 | GDPR Art. 15/17/20 operations supported via CLI + Surface (access, erasure, portability). |

## Performance

| ID | Requirement |
|---|---|
| NF-501 | Morning Brief renders ≤3s on reference hardware from pre-computed state. |
| NF-502 | Capability Broker check: p50 ≤5ms local, p50 ≤20ms guest (microVM). |
| NF-503 | Memory Graph retrieval: p50 ≤200ms for k=20 over 100k nodes. |
| NF-504 | Omnibar search result: p50 ≤500ms. |
| NF-505 | Local-inference latency for summarization (CPU v0): p50 ≤10s on reference CPU (Qwen 2.5 7B Q4); ≤2s on GPU when GPU lands in v1. |
| NF-506 | Cloud-twin round-trip: p50 ≤800ms end-to-end. |
| NF-507 | **Signed Inference overhead ≤ 5 ms per call** (BLAKE3 hash + Ed25519 sign + event-log append). ADR-0009. |
| NF-508 | Attestation storage ≤ 500 B per inference call (struct + provider_attest when present). |

## Explicit non-performance requirements (v0)

- **No boot-time optimization required at v0.** Explicit non-goal per user directive. Optimize later.
- No real-time scheduling guarantees.
- No sub-100ms UI responsiveness target (Apple-parity is a v1 push).
- **No GPU acceleration for local inference at v0.** Explicit deferral per directive 2026-04-21. v0 targets CPU + cloud only. GPU scheduling / fractional allocation is a v1+ concern.
- No kernel-level LLM/GPU resource scheduling at v0.

## Scalability

| ID | Requirement |
|---|---|
| NF-601 | Single user, single device (v0). |
| NF-602 | Memory Graph scales to 500k nodes without retrieval regression > 50% (v1 target). |
| NF-603 | Pack registry serves 10k concurrent lookups (v1). |
| NF-604 | Cloud twin fleet auto-scales; stateless design enables horizontal scaling. |

## Usability

| ID | Requirement |
|---|---|
| NF-701 | First-time install asks ≤2 questions (pair phone, connect Gmail). |
| NF-702 | Every surface keyboard-navigable (full parity with mouse/touch). |
| NF-703 | High-contrast mode preserves brand; not a separate stylesheet. |
| NF-704 | Voice input parity on every surface (v1). |
| NF-705 | Screen-reader ARIA on all semantic regions. |

## Extensibility

| ID | Requirement |
|---|---|
| NF-801 | Harness interface swappable (opencode v0; AgentField / Claude Agent SDK / custom later) without touching L2 services. |
| NF-802 | Adding a new capability kind requires an ADR but is a contained change. |
| NF-803 | Adding a new Model Router backend requires only implementing the `ModelBackend` trait and registering. |
| NF-804 | Adding a new memory node type or edge kind requires an ADR. |
| NF-805 | Pack format stable across minor versions; breaking changes require deprecation window. |

## Maintainability

| ID | Requirement |
|---|---|
| NF-901 | L2 services independently testable; each ships a contract test suite. |
| NF-902 | Channel parity (HTTP / CLI / socket / surface) asserted in CI per service. |
| NF-903 | Region Router 100% covered by decision-table unit tests. |
| NF-904 | Pack format documented with schema + examples; changelog maintained. |

## Related

- [`functional.md`](./functional.md) — functional complement
- [`security.md`](./security.md) — security depth
- [`performance.md`](./performance.md) — performance depth
- [`../10-architecture.md`](../10-architecture.md) — layer placement
- [`../02-apple-design-principles.md`](../02-apple-design-principles.md) — usability guardrails
