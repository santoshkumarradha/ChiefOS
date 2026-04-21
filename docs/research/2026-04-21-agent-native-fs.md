---
id: research-agent-fs
title: "Agent-Native Filesystem — focused deep-dive"
status: review
owners: [santosh]
last_updated: 2026-04-21
related: [memory-substrate, research-primitive-rethinks]
tags: [research, filesystem, cas]
---

# Agent-Native Filesystem

## TL;DR

- The broader primitive-rethink scan ([`2026-04-21-ai-native-primitive-rethinks.md`](./2026-04-21-ai-native-primitive-rethinks.md)) already picked **content-addressed filesystem with a human-folder FUSE shim** as top-5 substrate primitive #2. This doc drills in.
- Primary layout: **CAS over blake3** + **property-graph index** (nodes + edges from [`08-memory-substrate.md`](../08-memory-substrate.md)).
- Legacy `~/Downloads/`, `~/Desktop/`, etc. live on as a **read/write FUSE layer** mapping CIDs to path aliases (Plan9 `bind`-style).
- Embedding model choice: **fastembed-rs** (Rust, ort/ONNX, bge-small/bge-m3) for hot-path; **sentence-transformers** in Python for agent-side ad-hoc.

## Two views over one substrate

```
User view (legacy)              Agent view (native)
────────────────                ─────────────────────
~/Downloads/paper.pdf    ←→     mem://file/blake3:a1b2…
                                  type: file
                                  blob_ref: /chief/blobs/a1/b2/…
                                  horizon: short
                                  embedding: [vec]
                                  provenance_ref: prov:…
                                  edges: refers-to, authored-by, cites
```

Same object, two addressing schemes. Writes from either side converge to the same CAS node. Agents never need the path; humans can still see it.

## Capture daemon

Every write to the human-facing FS paths is intercepted by a capture daemon:

1. Compute blake3 CID.
2. If new: write blob to CAS store; extract metadata (MIME, size, lines-if-text).
3. Create or update `mem://file/<cid>` node in Memory Graph.
4. Emit event to signed typed event log (substrate #1).
5. Update FUSE mapping `~/path` → `<cid>`.

The capture daemon is small, Rust, hot-path-optimized. It is the "input" side of the substrate spine.

## Embedding model choice (fast, local, Rust-friendly)

| Option | Language | Quality (MTEB-avg) | Speed | RAM | Notes |
|---|---|---|---|---|---|
| **fastembed-rs** + **bge-small-en-v1.5** | Rust (ort/ONNX) | 62.5 | **Fast** (batch 128/s CPU) | ~400MB | Recommended default. Pure Rust inference. |
| fastembed-rs + **bge-m3** | Rust | 66.3 | Medium | ~1GB | Multilingual; for P0 in non-English contexts |
| fastembed-rs + **nomic-embed-text-v1.5** | Rust | 62.4 | Fast | ~500MB | Matryoshka (variable dim) useful for storage trade-offs |
| Python sentence-transformers | Python | varies | Slower in single-process | — | For ad-hoc agent use via pack |

**Decision:** default to **fastembed-rs + bge-small-en-v1.5** for the capture-daemon hot path; offer **bge-m3** as opt-in for multilingual. Python path available to pack authors.

## Storage topology

```
/chief/blobs/<blake3-prefix>/<cid>      CAS blobs, content-addressed
/chief/graph/                           SQLite: nodes + edges
/chief/embed/                           LanceDB or sqlite-vec vectors
/chief/events/                          FUSE view of signed typed event log
/home/$USER/                            FUSE legacy view → maps to CAS
```

## Retrieval latency budget

From [`requirements/performance.md`](../requirements/performance.md):

- P-004: Memory Graph k=20 retrieval over 100k nodes — p50 ≤ 200ms.
- Omnibar search (P-005) p50 ≤ 500ms end-to-end.

Breakdown (targets):
- Embedding query: ≤ 10ms (fastembed-rs CPU for single input).
- Vector top-K: ≤ 50ms (LanceDB on 100k embeddings, 384-dim).
- BM25 complement: ≤ 50ms (SQLite FTS5).
- Graph-walk reranking: ≤ 50ms (5-hop neighborhood).
- LLM rerank (optional): ≤ 300ms (local Q4 7B).

## Legacy-path compatibility

| Legacy expectation | Chief FS response |
|---|---|
| `cd ~/Downloads && ls` shows files | FUSE returns file names for pinned CIDs |
| User renames a file | Capture daemon emits `rename` event; graph updates `displayed-at` edge; CID unchanged |
| User deletes a file | Tombstone in graph; CAS blob retained for provenance until explicit `chief purge` |
| User copies to an external disk | Copy from `/home/$USER/...` works; CID preserved if destination is another Chief FS, else plain bytes |
| App saves to `~/Documents` | Capture daemon ingests like any other write |

## Open questions (focused)

1. Which CAS inline-engine — `iroh` (BLAKE3-native, modern Rust), custom, or `ipfs-unixfs` (IPFS-compat but heavier)?
2. Do we intercept `open()`/`write()` at the VFS layer (io_uring), at a `fanotify` layer, or at a FUSE boundary? Performance tradeoffs.
3. Inter-device CAS sync protocol: reuse `iroh` peer-to-peer, or bespoke over QUIC?
4. Quota / garbage collection: when do we actually delete CAS blobs? Tombstone + TTL + explicit purge.
5. Retention of embedding vectors across model upgrades: re-embed or lock?

## Follow-up tasks

- [ ] ADR: CAS-over-blake3 with FUSE shim as the primary filesystem layout.
- [ ] ADR: embedding model pinning strategy.
- [ ] Prototype: `chief-fs` capture daemon + FUSE shim (Rust).
- [ ] Update [`08-memory-substrate.md`](../08-memory-substrate.md) to reflect CAS-primary framing.

## Related

- [`2026-04-21-ai-native-primitive-rethinks.md`](./2026-04-21-ai-native-primitive-rethinks.md) — parent research (FS is top-5 pick)
- [`../08-memory-substrate.md`](../08-memory-substrate.md) — Memory Graph schema that this FS feeds
- [`../03-chief-kernel.md`](../03-chief-kernel.md) — capture daemon fits here
