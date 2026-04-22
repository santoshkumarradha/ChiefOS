---
id: research-pure-oss-memory
title: "Pure-OSS Memory Substrate"
status: review
owners: [santosh]
last_updated: 2026-04-21
related: [memory-substrate, chief-kernel]
tags: [research, memory, oss, no-vendor-capture]
---

# Pure-OSS Memory Substrate

## TL;DR

At the Chief kernel layer we use **no single-company-captured OSS**. Memory Graph is assembled from community- or consortium-governed components so we can customize at OS level without vendor gravity.

**Picked stack (all pure-OSS, multi-maintainer):**

| Component | Pick | License | Maintainer |
|---|---|---|---|
| Vector index | **sqlite-vec** | Apache-2.0 / MIT | Alex Garcia + broad contributor base, no parent co. |
| Graph store | **SQLite** (hand-rolled typed-edge schema) | Public domain | SQLite Consortium (multi-vendor governance) |
| Blob CAS | **iroh-blobs** (library crate) | Apache-2.0 / MIT | Number 0 + Protocol Labs / IPFS alumni |
| Full-text | **SQLite FTS5** | Public domain | SQLite Consortium |
| Embedding inference | **fastembed-rs + ort (ONNX Runtime)** | Apache-2.0 | fastembed (community) + ort (Microsoft ONNX upstream) |
| Python binding | **PyO3** | Apache-2.0 / MIT | PyO3 Foundation (community) |

**Net:** one `.db` file (SQLite with vec0 + FTS5), one blob dir (iroh-blobs), one embedder. No server processes. No SaaS on critical path.

## Alternatives per component (fallbacks)

| Component | Alternative | Caveat |
|---|---|---|
| Vector | LanceDB (lance format only) | Company-led; safe to depend on `lance` format, not `lancedb` SDK |
| Vector | hnswlib | Pure index only; build persistence yourself |
| Graph | Cozo | Brilliant Datalog, but bus-factor ~1 — use as optional query layer, not substrate |
| Graph | DuckDB | OLAP-shaped; great for analytics over the graph but overkill as primary |
| Blob | gitoxide object store | Git-semantics overloaded; use only for git-interop |
| Full-text | Tantivy (Quickwit-seeded but multi-contrib) | Separate index to sync; choose when FTS5 ranking insufficient |
| Embedding | Candle (HF-backed) | HF-governed but adopted broadly; HF business model is hub/hosting, not framework |
| Embedding | llama.cpp embedding mode | Heavy dep for pure embedding |

## Explicit rejections (single-company capture)

| Project | Parent | Reason |
|---|---|---|
| Letta (MemGPT) | Letta Inc. (a16z) | OSS drives hosted-agent-platform; imposes "agent platform" shape |
| Mem0 | Mem0 (YC + VC) | Managed-API monetization; schema bent toward platform |
| Zep | Zep AI Inc. | Hosted memory is the product; OSS is lead-gen |
| ChromaDB | Chroma Inc. (VC) | Chroma Cloud; SDK increasingly tuned for cloud |
| Weaviate / Milvus / Vespa | Weaviate / Zilliz / Yahoo-spinout | Server-shaped, inappropriate for device |
| Meilisearch / Typesense | Meili SAS / Typesense Inc. | Company-led with hosted offerings |
| SurrealDB (embed) | SurrealDB Ltd. | Single-company; business-source concerns on server |
| Memgraph | Memgraph Ltd. | Company-led, server-shaped, community/enterprise split |

## Integration sketch

One Rust crate (`chief-mem`) opens:

- **One SQLite database** with `sqlite-vec` + FTS5 loaded at `~/.chief/mem/mem.db`.
- **One iroh-blobs store** at `~/.chief/mem/blobs/`.

Schemas:

```sql
-- nodes
CREATE TABLE nodes (
  uri TEXT PRIMARY KEY,          -- mem://<type>/<blake3>
  type TEXT NOT NULL,
  content_hash TEXT NOT NULL,
  horizon TEXT NOT NULL,         -- short | medium | long | open
  created_at INTEGER,
  meta_json TEXT
);

-- edges
CREATE TABLE edges (
  src_uri TEXT, dst_uri TEXT,
  type TEXT NOT NULL,            -- derived-from | replied-to | cites | ...
  weight REAL, meta_json TEXT
);

-- vectors (sqlite-vec virtual table)
CREATE VIRTUAL TABLE vecs USING vec0(
  node_uri TEXT PRIMARY KEY,
  embedding FLOAT[384]
);

-- full-text (SQLite FTS5)
CREATE VIRTUAL TABLE texts USING fts5(
  node_uri UNINDEXED,
  body
);
```

Retrieval is one SQL query joining: vector similarity (sqlite-vec) + FTS5 BM25 + horizon-tag boosts + capability-filter subquery. All in-process.

Python consumers get the same substrate via a **PyO3** shim exposing `open`, `put_node`, `put_edge`, `put_blob`, `query(scope, horizon, text, k)`.

Swap-ability is preserved: every lane is behind a trait in `chief-mem`:

- `VectorIndex` trait: default sqlite-vec; fallback hnswlib + sidecar.
- `BlobStore` trait: default iroh-blobs; fallback plain CAS directory.
- `EmbeddingModel` trait: default fastembed-rs + ort; fallback candle or custom.

Chief FS mounts the graph read-only as a FUSE/virtual tree where paths are `mem://`-derived, so capability-gated retrieval and filesystem traversal share the same authority model.

## Risks

| Risk | Mitigation |
|---|---|
| sqlite-vec is young (v0.x) | Pin version; vendor the extension; keep hnswlib fallback trait impl |
| iroh-blobs API churn | Pin crate version; use only `iroh-blobs` (not `iroh` router/p2p) for kernel |
| fastembed-rs is Qdrant-seeded | Monitor; `ort` + our own model loader is escape hatch |
| Cozo / Surreal bus-factor | Keep off critical path; hand-rolled SQLite schema is boring and unkillable |

## Follow-up actions

- [x] Open ADR [`../../adr/0008-pure-oss-memory-substrate.md`](../../adr/0008-pure-oss-memory-substrate.md).
- [x] Update [`../13-memory-substrate.md`](../13-memory-substrate.md) with confirmed stack.
- [ ] Prototype `chief-mem` crate (plandb `t-future-protos`).
- [ ] Benchmark sqlite-vec vs LanceDB-lance on Chief-class workload.

## Related

- [`../13-memory-substrate.md`](../13-memory-substrate.md) — substrate design
- [`../11-chief-kernel.md`](../11-chief-kernel.md) — Memory Graph service
- [`2026-04-21-agent-native-fs.md`](./2026-04-21-agent-native-fs.md) — filesystem sister doc
- [`2026-04-21-oss-landscape-scan.md`](./2026-04-21-oss-landscape-scan.md) — broader OSS scan
- [`../directives/README.md`](../directives/README.md) — originating directive
