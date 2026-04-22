---
id: adr-0005
title: "0005 — Signed Typed Event Log as the substrate"
status: accepted
authors: [santosh]
created: 2026-04-21
decided: 2026-04-21
supersedes: []
superseded_by: []
axiom_impact: [1, 3, 8]
tags: [substrate, provenance, observability]
---

# 0005 — Signed Typed Event Log as the substrate

## Context

Initial design had Provenance Log as one of seven L2 services. Research ([`docs/research/2026-04-21-ai-native-primitive-rethinks.md`](../docs/research/2026-04-21-ai-native-primitive-rethinks.md)) reframes: the signed typed event log isn't *one* service — it's the **substrate** on which every other L2 service is a view. Every tool call, syscall, agent decision, capability check, and UI event is a typed tuple in one log.

Traditional syslog / journald is unstructured text, unsigned, unaudited. It fails Axiom 3 (unforgeable provenance) and Axiom 8 (protocol not UI).

## Options considered

| Option | Pros | Cons |
|---|---|---|
| Keep Provenance Log as one-of-seven services; let others log as they wish | Simple | Axiom 3 violation; no unified replay; audit chain broken |
| Typed event log per service | Structured | Multiple logs; cross-service reconstruction hard |
| **Single Signed Typed Event Log as substrate** | Unified, auditable, replayable; every service is a view | We must implement and harden this carefully; schema discipline required |

## Decision

Implement a **single Signed Typed Event Log** as the substrate of Chief OS. Every L2 service publishes typed events into it; every downstream service (Memory Graph, Trust Ledger, Search, Inbox) is a view over it. Signatures are Sigstore-compatible in-toto v1 statements.

## Rationale

1. Collapses the "seven services + seven logs" problem into "seven services, one log, seven views."
2. Enables "replay my morning" as a first-class operation (critical demo beat).
3. Provenance chain is native, not bolted on.
4. Unifies observability (no more "check broker logs + memory logs + router logs").
5. Schema discipline (Cap'n Proto / Protobuf) prevents log rot.

## Consequences

### Positive
- One audit surface (Provenance Explorer) for everything.
- Rewind primitive becomes log-replay from a timestamp.
- External auditors can verify via standard Merkle tooling.
- Every service gets "free" observability.

### Negative / accepted costs
- Schema governance becomes critical; new event types require coordination.
- Hot-path services must avoid ballooning event size.
- Storage grows continuously; retention + compaction policy needed.

### Downstream effects
- Supersedes the simple "Provenance Log" service scope in [`docs/11-chief-kernel.md`](../docs/11-chief-kernel.md) — Provenance Log is the custodian of the substrate.
- Enables [`adr-0007`](./0007-hax-inbox-notifications.md) (HAX Inbox as a view over pending-decision events).
- Informs [`docs/13-memory-substrate.md`](../docs/13-memory-substrate.md) (Memory Graph is a denormalized view).

## Implementation notes

- Storage: RocksDB (hot) + Parquet (cold tier) at `/var/lib/chief/events/`.
- Addressing: blake3 CID per event.
- Schema: Cap'n Proto (`schemas/event.capnp`), versioned per-event-type.
- Signature: in-toto v1 statement per event, bundled into periodic Merkle roots.
- FUSE view mounted at `/chief/events/` for read-only structured access.
- Retention: 7+ years on-device; external mirror per user policy.

## Revisit triggers

- Storage grows beyond pragmatic per-device size (tune compaction + cold-tier policy).
- Schema churn exceeds deprecation-window discipline (formalize schema governance).
- A fundamentally different substrate emerges (e.g., a ledger protocol we can adopt).
