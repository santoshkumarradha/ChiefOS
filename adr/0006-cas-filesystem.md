---
id: adr-0006
title: "0006 — Content-addressed filesystem with human-folder FUSE shim"
status: accepted
authors: [santosh]
created: 2026-04-21
decided: 2026-04-21
supersedes: []
superseded_by: []
axiom_impact: [1, 3, 5]
tags: [filesystem, cas, memory]
---

# 0006 — Content-addressed filesystem with human-folder FUSE shim

## Context

Traditional filesystems (ext4, btrfs, APFS) are human-organized: paths have semantics, filenames are strings, hierarchy is a tree. This is wrong for an AI-native OS:

- Agents don't navigate paths; they cite content.
- Rename / move breaks agent citations.
- Deduplication is after-the-fact.
- Provenance tracking is manual.

At the same time, humans still want `~/Downloads/` to show the PDF they just downloaded.

## Options considered

| Option | Pros | Cons |
|---|---|---|
| Keep legacy FS only | Zero change | Agent citations fragile; no provenance-native FS |
| Rip out legacy FS, agents only | Pure | Humans hate it; breaks every app |
| **CAS primary + FUSE shim for legacy views** | Both paradigms, same bits | Requires capture daemon; some perf cost |
| Dual FS: copy everything to CAS on capture | Isolates concerns | 2× storage; sync complexity |

## Decision

**Chief FS** = content-addressed store (blake3 CIDs) + property-graph index + FUSE layer presenting legacy paths as pinned aliases to CIDs. Every write intercepted by a capture daemon that hashes, indexes, links into Memory Graph, and emits to the substrate event log.

## Rationale

1. Agents cite CIDs; citations survive renames and moves.
2. Provenance is native: every file is a node in the graph from creation.
3. Deduplication is free (same CID ↔ same blob).
4. Legacy apps and human ergonomics preserved via FUSE.
5. Rewind is natural: CID + horizon tombstone + graph edge un-remove.
6. Two-view design means humans can keep saying "~/Downloads" without losing agent-native addressability.

## Consequences

### Positive
- Agent citations stable for the lifetime of content.
- Provenance chain starts at file creation.
- CAS unlocks efficient sync (iroh-style p2p) between devices.
- Rewind is a graph operation, not a shell exercise.

### Negative / accepted costs
- Capture daemon is a new critical path; must be hot-path-optimized (Rust).
- FUSE layer adds latency on metadata ops (mitigated with cache).
- Garbage collection on CAS blobs requires careful policy (tombstone + TTL + explicit purge).
- Some apps with aggressive fsync / rename patterns may surface edge cases.

### Downstream effects
- [`docs/08-memory-substrate.md`](../docs/08-memory-substrate.md) — file nodes are CAS-backed.
- [`docs/07-base-and-hardware.md`](../docs/07-base-and-hardware.md) — storage layout updated.
- [`adr-0005`](./0005-signed-typed-event-log.md) — every capture emits an event.

## Implementation notes

- CAS engine: `iroh` (BLAKE3-native, modern Rust) or fork `ipfs-unixfs`.
- FUSE library: `fuser` crate.
- Capture hooks: start with `fanotify` for broad coverage; consider io_uring path in v1.
- Graph index: SQLite (nodes + edges) + LanceDB (vectors).
- Embedding model for capture metadata: `fastembed-rs + bge-small-en-v1.5` default; `bge-m3` opt-in for multilingual ([`docs/research/2026-04-21-agent-native-fs.md`](../docs/research/2026-04-21-agent-native-fs.md)).
- Layout:
  ```
  /chief/blobs/<prefix>/<cid>
  /chief/graph/   (sqlite)
  /chief/embed/   (lancedb)
  /home/$USER/    (fuse shim)
  ```

## Revisit triggers

- Capture daemon perf falls below P-010 (provenance append p50 ≤ 2ms) — optimize io_uring path.
- FUSE metadata overhead breaks specific app compatibility — whitelist or adopt virtio-fs for critical apps.
- Inter-device sync demand outgrows iroh — consider libp2p or custom QUIC protocol.
