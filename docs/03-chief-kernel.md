---
id: chief-kernel
title: "Chief Kernel — L2 Services"
status: draft
owners: [santosh]
last_updated: 2026-04-21
related: [architecture, security-model, memory-substrate, hax-principles]
depends_on: [architecture]
tags: [kernel, services, api]
---

# Chief Kernel (L2)

## TL;DR

- 7 services. All expose HTTP + CLI + Unix socket + internal API.
- Agent Runtime hosts harnesses. **Harness interface is swappable** (opencode v0, AgentField / Claude Agent SDK / custom later).
- Capability Broker is the single enforcement point for Axiom 2.
- Region Router is deterministic rule-table code; no LLM in the hot path.
- Provenance Log is append-only, Merkle-tree'd, signed.

## Services overview

```mermaid
flowchart LR
    EB((Event Bus))
    AR[Agent Runtime]
    CB[Capability Broker]
    MG[Memory Graph]
    PL[Provenance Log]
    TLD[Trust Ledger]
    RR[Region Router]

    AR -- propose action --> CB
    CB -- classify --> RR
    CB -- read grants --> TLD
    CB -- log --> PL
    AR -- read/write scoped URIs --> MG
    MG -- emits changes --> EB
    PL -- emits entries --> EB
    TLD -- emits updates --> EB
    AR -- subscribe --> EB
    RR -- reads --> TLD
```

Diagram also at [`diagrams/kernel-services.mmd`](./diagrams/kernel-services.mmd).

## Service-by-service contracts

### 1. Agent Runtime (`chief-runtime`)

| Aspect | Value |
|---|---|
| Purpose | Host long-lived agent processes (harnesses) with lifecycle + isolation |
| Key ops | `runtime.spawn(pack, agent, input, caps)` · `runtime.signal` · `runtime.status` |
| State | Agent process table, active tasks, queued actions |
| Storage | In-memory + SQLite journal |
| Dependencies | Capability Broker, Memory Graph, Provenance Log, Event Bus |
| Harness backend | **Swappable interface.** v0: opencode. Future: AgentField harness, Claude Agent SDK, custom. |

**Harness interface** (the abstraction — ADR-worthy):

```
trait Harness {
    fn start(task_spec: TaskSpec) -> HarnessHandle;
    fn submit_action(handle, action) -> Receipt;
    fn submit_tool_call(handle, tool_id, args) -> Result;
    fn stream_events(handle) -> EventStream;
    fn terminate(handle, reason);
}
```

v0 implementation wraps opencode. Future impls swap by pack config without touching Chief Kernel.

### 2. Capability Broker (`chief-broker`)

| Aspect | Value |
|---|---|
| Purpose | Gate every tool call, memory access, and outbound action against explicit grants |
| Key ops | `broker.grant(principal, capability, scope, expires_at)` · `broker.check(principal, op)` · `broker.revoke` |
| State | Live grants (per principal / pack), grant issuance log |
| Storage | SQLite (authoritative) + in-memory cache |
| Dependencies | Trust Ledger (delegation level gates grant issuance), Provenance Log (every check logged at DEBUG, every deny at INFO) |
| Guarantees | No syscall bypass paths. eBPF enforcement on network + fs for high-risk cats. |

**Grant format:**

```yaml
grant:
  principal: pack:travel-planner/agent:itinerary-synth/rev:abc123
  capability: gmail.send
  scope:
    threads: ["label:travel"]
    max_recipients: 2
    max_body_chars: 1200
  expires_at: 2026-05-21T00:00:00Z
  issued_by: user-approval:ceremony-7f3a
  revocable: true
```

### 3. Memory Graph (`chief-memory`)

See [`memory-substrate`](./08-memory-substrate.md) for schema and URI design.

| Aspect | Value |
|---|---|
| Purpose | Unified, typed, content-addressed graph of everything Chief knows |
| Key ops | `mem.put(node)` · `mem.get(uri)` · `mem.link(from, kind, to)` · `mem.search(query, horizon)` |
| Storage | SQLite (nodes + edges) + FAISS index (embeddings) + blob store (large files) |
| Dependencies | Provenance Log (every write gets a receipt ref) |
| Invariants | URIs are stable forever; nodes are append-only; edges deleteable only via rollback |

### 4. Provenance Log (`chief-provenance`)

| Aspect | Value |
|---|---|
| Purpose | Append-only, Merkle-tree'd, signed log of every agent action, decision, ship |
| Key ops | `prov.append(entry)` · `prov.chain(artifact_id)` · `prov.verify(entry)` |
| Storage | Append-only SQLite + periodic Merkle-root-signed snapshot to disk |
| Dependencies | Device key (L0 TPM) for signing |
| Invariants | No deletes, no rewrites. Retention ≥ 7 years. |

**Entry schema:**

```yaml
prov_entry:
  id: <merkle-path>
  prev: <prev-entry-hash>
  timestamp: 2026-04-21T06:12:03Z
  actor: agent:itinerary-synth / human / system
  op: "read mem://email/xyz" | "propose action/wire-transfer" | "approval-sign" | ...
  inputs: [mem://...]
  outputs: [mem://...]
  rollback_ref: <rollback-window-id>
  signature: <device-key-signature>
```

### 5. Trust Ledger (`chief-trust`)

See [`hax-principles`](./01-hax-principles.md) for semantics.

| Aspect | Value |
|---|---|
| Purpose | Per-category delegation levels (1–5), with auditable write-rules |
| Key ops | `trust.get(category)` · `trust.record_approval(category)` · `trust.record_rollback(category)` · `trust.override(category, level, authority)` |
| Storage | SQLite, append-only event log + materialized view |
| Dependencies | Provenance Log (every change emits entry) |
| Invariants | LLM cannot write. Writes require explicit ritual or override by human. |

### 6. Region Router (`chief-router`)

| Aspect | Value |
|---|---|
| Purpose | Classify proposed actions into HAX regions; route to surface + friction tier |
| Key ops | `router.classify(action_metadata, ledger_snapshot, horizon) -> {region, surface, friction_tier}` |
| Storage | Rules table (YAML, version-controlled) + default weights |
| Dependencies | Trust Ledger (read), action metadata |
| Invariants | **Deterministic. No LLM in the classifier.** Rules changes go through ADR. |

Rules table lives at `chief-router/rules/*.yaml`; shipped packs can declare additional rules that scope only to their own categories.

### 7. Event Bus (`chief-bus`)

| Aspect | Value |
|---|---|
| Purpose | Publish/subscribe for all L2 state changes |
| Key ops | `bus.publish(topic, event)` · `bus.subscribe(topic_glob, filter) -> stream` |
| Storage | In-memory ring buffer + SQLite durable overflow |
| Dependencies | — |
| Invariants | Ordering within topic; at-least-once delivery |

## API channel matrix

| Service | HTTP | CLI | Unix socket | Internal (in-proc) |
|---|---|---|---|---|
| Agent Runtime | ✓ | ✓ | ✓ | ✓ |
| Capability Broker | ✓ | ✓ | ✓ | ✓ |
| Memory Graph | ✓ | ✓ | ✓ | ✓ |
| Provenance Log | ✓ (read-only for remote) | ✓ | ✓ | ✓ |
| Trust Ledger | ✓ (read-only for remote) | ✓ | ✓ | ✓ |
| Region Router | — (called via Broker) | ✓ (inspect) | ✓ | ✓ |
| Event Bus | ✓ (SSE) | ✓ (tail) | ✓ | ✓ |

## Storage layout

```
/var/lib/chief/
├── broker/           (SQLite: grants, check log)
├── memory/
│   ├── nodes.db      (SQLite: typed nodes)
│   ├── edges.db      (SQLite: typed edges)
│   ├── embed.faiss   (vector index)
│   └── blobs/        (content-addressed)
├── provenance/
│   ├── log.db        (append-only)
│   └── snapshots/    (signed Merkle roots)
├── trust/
│   ├── events.db
│   └── view.db       (materialized)
├── runtime/
│   ├── agents/       (per-agent journals)
│   └── packs/        (installed pack state)
└── bus/
    └── overflow.db
```

All under the user's encrypted volume (LUKS v0; Apple-style-per-file v1).

## Acceptance

- [ ] Each service has a contract test suite callable via `chief test <service>`.
- [ ] Channel parity asserted by CI across all 4 channels per service.
- [ ] Harness interface stable enough to swap opencode for a stub implementation in tests.
- [ ] Region Router 100% covered by decision-table unit tests.
- [ ] Provenance Log verified by external tooling (Merkle chain validator).

## Open questions

1. Should the Event Bus be a real message broker (NATS) or in-proc pub/sub for v0? Bias: in-proc.
2. Is FAISS the right vector index for a single-user device? Alternatives: sqlite-vss, LanceDB.
3. Do we ship a privileged "kernel daemon" or run each service as its own systemd user unit?
4. How does the harness interface handle streaming tool outputs (SSE vs. chunked JSON)?
5. What's the default scope for mem-read grants — "everything the pack declared" or "just-in-time narrowing"?

## Related

- [`architecture`](./02-architecture.md) — where these sit in the stack
- [`security-model`](./06-security-model.md) — cryptographic details
- [`memory-substrate`](./08-memory-substrate.md) — Memory Graph schema
- [`hax-principles`](./01-hax-principles.md) — Trust Ledger + Region Router semantics
- [`adr/0002-capability-based-security.md`](../adr/0002-capability-based-security.md) — the cap-model decision
