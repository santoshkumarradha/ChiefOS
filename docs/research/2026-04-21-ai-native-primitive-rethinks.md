---
id: research-primitive-rethinks
title: "AI-Native Primitive Rethinks — broad scan"
status: review
owners: [santosh]
last_updated: 2026-04-21
tags: [research, primitives, ai-native, substrate]
---

# AI-Native Primitive Rethinks

## TL;DR

- **5 primitives form the "substrate spine"** of Chief OS and should be baked into v0/v1:
  1. Signed typed event log (the substrate)
  2. Content-addressed filesystem with human-folder shim
  3. Typed, provenanced clipboard (`kbd://`)
  4. HAX approval queue as the notification primitive (`hax://inbox`)
  5. Semantic memory-graph search as omnibar
- **Why together:** #1 is substrate, #2 pins matter to substrate, #3 funnels intent into substrate, #4 is output channel, #5 is read-side UI. Everything else becomes views or policies over this spine.
- Deferred rethinks + "leave boring" catalog at bottom.

## Ranking method

Score = `(Strategic 1–5 × Demo 1–5) / Eng-Cost 1–5`. Higher = bake first.

## Full ranking

| # | Primitive | S | D | C | Score | One-liner |
|---|---|---|---|---|---|---|
| 1 | **Logging / observability** | 5 | 5 | 2 | 12.5 | Signed typed events as substrate; every syscall/tool call is a verifiable tuple. Plan9 `/proc` meets Sigstore. |
| 2 | **Clipboard** | 4 | 5 | 1 | **20.0** | Content-addressed typed pasteboard. Agent watches `kbd://latest`, auto-transforms (PDF → summary → calendar). Cheapest viral demo. |
| 3 | **Filesystem layout** | 5 | 5 | 3 | 8.3 | Primary view = CAS + memory graph (blake3 CIDs, edges). Human folders are a FUSE shim (Plan9 `bind` + Tahoe-LAFS + IPFS UnixFS). |
| 4 | **Notifications** | 4 | 5 | 1 | **20.0** | No popups. HAX approval queue IS the OS notification bus. One pane shows pending agent asks with typed actions. |
| 5 | **Search** | 5 | 5 | 2 | 12.5 | Omnibar = semantic over memory graph. Not Spotlight-over-files; Spotlight-over-agent-state-and-history. |
| 6 | Shell / terminal | 4 | 4 | 3 | 5.3 | Typed command graph (nushell-like). Agents compose without regex-parsing bash. Wrap, don't reinvent. |
| 7 | Session model | 4 | 4 | 2 | 8.0 | Never-logout. Session = persistent thread with Chief. Falls out of #1+#3. |
| 8 | Text editing (CRDT) | 3 | 5 | 2 | 7.5 | Automerge as document layer; live agent+human co-edit. Library, not OS primitive. |
| 9 | DNS / service discovery | 4 | 3 | 2 | 6.0 | `agent://` namespace = capability registry. Fuchsia-style resolution by capability, not hostname. |
| 10 | Authentication | 4 | 3 | 2 | 6.0 | Capability grants via Broker, not passwords. Already planned. |
| 11 | Device drivers | 4 | 3 | 2 | 6.0 | Mic/camera/GPU via Broker time-bound grants. Already planned. |
| 12 | Scheduler | 4 | 2 | 3 | 2.7 | HAX-region-aware priority (long-horizon low-intensity vs short bursts). Hard to demo. |
| 13 | Signals / IPC | 4 | 2 | 3 | 2.7 | MCP + A2A as primary IPC. POSIX signals remain for legacy. Invisible win. |
| 14 | Env vars | 3 | 2 | 1 | 6.0 | Typed, scoped, cap-gated env. Cheap add-on to typed shell. |
| 15 | Network stack | 3 | 3 | 3 | 3.0 | QUIC + vsock + io_uring. Already in earlier research; port. |
| 16 | Media citability | 3 | 4 | 3 | 4.0 | Agents cite `media://<cid>#t=14:22`. Great demo once transcript+index pipeline exists. |
| 17 | Memory mgmt (prefetch) | 3 | 2 | 4 | 1.5 | Agent-aware page prefetch. Kernel-deep; v2+. |
| 18 | Ritual time-scheduling | 3 | 3 | 2 | 4.5 | Multi-scale clocks. Downstream of session + memory. |
| 19 | Mail/cal/browser ingesters | 4 | 4 | 3 | 5.3 | Already in scope per architecture directive. |
| 20 | Window / compositor | 3 | 3 | 3 | 3.0 | Declarative surfaces already chosen. Execute. |
| 21 | Package management | — | — | — | — | Already reimagined (Nix + Capability Packs). Skip. |
| 22 | i18n | 2 | 2 | 2 | 2.0 | Model concern, not OS primitive. |

## Top 5 — bake into v0/v1

### 1. Signed typed event log (substrate for everything)

**What:** Replace syslog/journald with an append-only **blake3-addressed, Sigstore-signed event store** (RocksDB or fjall KV + Parquet cold tier). Every tool call, syscall wrapper, agent decision, and UI action emits a typed tuple:

```
(actor, verb, object_cid, capability_receipt, timestamp, parent_event)
```

Schema defined once in Cap'n Proto or Protobuf. Every other primitive — Memory Graph, search, approval queue, replay — is a **view** over this log.

**Prototype:** Ship a `chief-journald` sidecar in v0 that captures MCP + A2A traffic; expose as a read-only FUSE at `/chief/events/`. Single log makes "replay my morning" and "prove this agent did X" demoable.

**Integration points (our existing docs):**
- [`11-chief-kernel.md`](../11-chief-kernel.md) — Provenance Log grows into this substrate.
- [`14-security-model.md`](../14-security-model.md) — Sigstore signing on every event.
- [`10-architecture.md`](../10-architecture.md) — add as L2 primitive beneath other services.

### 2. Content-addressed filesystem with human-folder shim

**What:** Primary store = **CAS over blake3 + property-graph index** (RocksDB for edges, DuckDB for ad-hoc joins). Human folders are a read/write FUSE layer that pins CIDs to path aliases (Plan9 `bind`-style). Every write goes through a capture daemon that hashes, extracts metadata, links into the graph, and emits an event to #1.

**Prototype:** Fork `ipfs-unixfs` or use `iroh` for CAS; `fuser` crate for the shim. Mount at `/chief/`; `/home/$USER` becomes the legacy view.

**Demo:** "Delete a file. It's still there by CID, and referenced in 3 agent conversations."

**Integration points:**
- [`13-memory-substrate.md`](../13-memory-substrate.md) — this IS the substrate's FS side.
- [`10-architecture.md`](../10-architecture.md) — add as L1/L2 hybrid primitive.

### 3. Typed, provenanced clipboard (`kbd://`)

**What:** Replace X11/Wayland clipboard with a **`kbd://` bus**: copies become CAS entries with MIME + semantic type + source-agent capability receipt + TTL. Agents subscribe and emit derived clips (`kbd://<cid>/summary`, `kbd://<cid>/translated`).

**Implementation:** Wayland protocol extension + small Rust daemon fronting the event log. Legacy `wl-clipboard` compatibility so standard apps see plain text; a sidebar shows typed ancestry and one-click agent transforms.

**Demo:** Copy a paper PDF → paste as calendar events. Or: copy a customer email → paste as a triaged response draft.

**Integration points:**
- [`30-surfaces.md`](../30-surfaces.md) — new surface: Clipboard Pane.
- [`11-chief-kernel.md`](../11-chief-kernel.md) — clipboard as an L2 service.

### 4. HAX approval queue as the notification primitive

**What:** Kill toast/popup notifications entirely. Every async agent ask, permission request, and system alert flows into a single typed queue (`hax://inbox`) with four affordances: **approve, deny, delegate, defer.** Rendered as persistent TUI pane + minimal Wayland overlay. Under the hood, it's a view over event-log entries of type `RequestCapability` or `RequestDecision`.

**Implementation:** Built on top of #1. Render with `ratatui` for TUI and Iced/Slint for Wayland overlay.

**Demo:** Five agents doing work, one pane shows exactly what needs you.

**Integration points:**
- [`30-surfaces.md`](../30-surfaces.md) — major update: this IS the notification surface.
- [`01-hax-principles.md`](../01-hax-principles.md) — the ledger's inbox.

### 5. Semantic memory-graph search as the omnibar

**What:** Default launcher / Spotlight = **embedding search over the memory graph** (people, events, CIDs, conversations, calendar). Not filename glob. Hybrid BM25 + vector + graph-walk ranking over `lancedb` or `qdrant` for vectors and the property graph from #2 for structure.

**Implementation:** Global hotkey opens Iced window backed by `chief-search` daemon that queries event log (#1) + CAS index (#2). Results are typed and dispatch directly into capabilities.

**Demo:** "Find the PDF my sister sent about the cabin last summer" → right answer in one step.

**Integration points:**
- [`30-surfaces.md`](../30-surfaces.md) — new surface: Omnibar.
- [`13-memory-substrate.md`](../13-memory-substrate.md) — retrieval API grows this.

## Why these 5 together

- **#1 is the substrate.** Everything is an event.
- **#2 pins matter to the substrate.** Every file is addressable and linked.
- **#3 funnels ambient user intent into the substrate.** Copy becomes intent.
- **#4 is the output channel back to the human.** HAX queue replaces popups.
- **#5 is the read-side UI.** One search over the entire life.

Everything else in the OS — sessions, scheduler, DNS, drivers, mail, browsers — becomes **views or policies over this spine.**

## Deferred to v2+

| Primitive | Reason |
|---|---|
| Typed shell (nushell-based) | High value; nushell ports well, wrap don't reinvent until v2 |
| Session never-logout | Falls out of #1 + memory graph; feature, not primitive |
| `agent://` DNS namespace | Needs capability registry maturity; v1.5 |
| Capability-gated drivers | Covered by existing Broker plan |
| Auth as capabilities | Same as drivers |
| HAX-region scheduler | Real wins but requires measured workloads; v1 after agents run |
| Media citability | Needs transcription pipeline; v2 |
| MCP/A2A as IPC primitive | Already the plan |
| Typed env vars | Trivial add-on to typed shell |
| Multi-scale time | Downstream of session model |
| Declarative compositor | Already decided; execute |
| Kernel-level prefetch for agent patterns | Measure first; v3 |
| CRDT text editing | Library layer; not OS primitive |

## Leave boring (port as-is)

| Primitive | Why |
|---|---|
| Process model / cgroups / namespaces | Linux right; wrap with capability overlay |
| Init system | systemd or s6; don't touch |
| Network stack (TCP/QUIC/io_uring) | Upstream is where action is |
| Package manager | Nix + Packs already |
| Compositor base (Wayland/Smithay) | Reuse |
| Filesystem (ext4, btrfs) | CAS rides on top |
| Localization / i18n | Model layer, not OS |
| Crypto primitives | Sigstore / ring / rustls |
| Audio/video codec stack | PipeWire + GStreamer; only cap-gate |
| POSIX signals | Keep for legacy; MCP/A2A additive |
| Window-manager internals | Stock compositor; only surface-declaration layer is ours |

## Action items (to be scheduled after architecture incorporation)

- [ ] Add "Signed Typed Event Log" as L2 substrate primitive (beneath all other services). Update [`10-architecture.md`](../10-architecture.md).
- [ ] Grow Provenance Log into the substrate event store. Update [`11-chief-kernel.md`](../11-chief-kernel.md).
- [ ] Add "Chief FS" (CAS + shim) to [`13-memory-substrate.md`](../13-memory-substrate.md) and architecture.
- [ ] Add Clipboard Pane + Omnibar + HAX Inbox to [`30-surfaces.md`](../30-surfaces.md).
- [ ] Open ADRs for #1, #2, #4 (the foundational three).
- [ ] Update [`50-v0-scope-90-day.md`](../50-v0-scope-90-day.md) to list the 5 substrate primitives as v0/v1 non-negotiables.

## Related

- [`../10-architecture.md`](../10-architecture.md) — primary integration target
- [`../13-memory-substrate.md`](../13-memory-substrate.md) — FS + event log converge here
- [`../30-surfaces.md`](../30-surfaces.md) — #3 #4 #5 add new surfaces
- [`2026-04-21-oss-landscape-scan.md`](./2026-04-21-oss-landscape-scan.md) — companion research
