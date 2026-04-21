---
id: adr-0008
title: "0008 — Pure-OSS Memory Substrate at the kernel layer"
status: accepted
authors: [santosh]
created: 2026-04-21
decided: 2026-04-21
supersedes: []
superseded_by: []
axiom_impact: [1, 7]
tags: [memory, oss, no-vendor-capture]
---

# 0008 — Pure-OSS Memory Substrate at the kernel layer

## Context

Earlier design considered Letta (a16z-backed startup-OSS) as a reference Memory Graph backend. Steward directive 2026-04-21 rejected this at the kernel layer: an OS-level substrate must not inherit gravitational pull toward any single company's hosted platform. Startup-backed OSS tends to bend schema, scope, and API toward their SaaS business model over time.

## Options considered

| Option | Pros | Cons |
|---|---|---|
| Letta as reference Memory Graph backend | Feature-rich; memory-blocks abstraction | Company-led; platform gravity; schema opinionates; AGPL-ish risks |
| Mem0 / Zep / Chroma-Inc | Working libraries | Same "startup-OSS-as-SaaS-funnel" pattern |
| LanceDB SDK | Strong engineering | LanceDB Inc. monetization trajectory; server-ish |
| **Pure-OSS stack (sqlite-vec + SQLite + iroh-blobs + FTS5 + fastembed-rs/ort)** | Multi-maintainer, embedded, no company capture | sqlite-vec is young; we own more assembly |
| Server memory stores (Weaviate, Milvus, Vespa, Memgraph) | Feature parity | Wrong shape for single-user device; company-led |
| Building our own | Total control | Founder-suicide |

## Decision

Assemble the Memory Graph from community- or consortium-governed components:

- **Vector index:** `sqlite-vec` (Apache-2.0 / MIT, Alex Garcia + contributors).
- **Graph store:** SQLite (public domain, SQLite Consortium).
- **Blob CAS:** `iroh-blobs` crate (Apache-2.0 / MIT, Number 0 + Protocol Labs alumni).
- **Full-text:** SQLite FTS5 (public domain).
- **Embedding inference:** `fastembed-rs` + `ort` (ONNX Runtime bindings) — Apache-2.0, multi-vendor.
- **Python binding:** PyO3 (Apache-2.0 / MIT, community).

Startup-led memory libraries (Letta, Mem0, Zep, Chroma-client, SurrealDB, Memgraph, Weaviate, Milvus, Vespa, Meilisearch, Typesense) are **rejected at the kernel layer**. Acceptable only as optional **pack-level** memory backends, not as the substrate.

## Rationale

1. Axiom 1 ("AI is the primary user") + Axiom 7 ("boring infrastructure") demand we own the substrate without vendor roadmap risk.
2. SQLite + its extensions are the archetype of unkillable boring infrastructure.
3. Embedded-only reduces attack surface and simplifies single-user-device deployment.
4. Trait-based swap-ability per lane (VectorIndex, BlobStore, EmbeddingModel) preserves future optionality.
5. Pack authors who want Letta/Mem0/etc. can still use them at the pack layer; the kernel stays clean.

## Consequences

### Positive
- No single-vendor roadmap risk at the kernel.
- Memory Graph bench-benchmarks are reproducible (same SQLite file portable across devices).
- Embedded stack has zero-config dev experience.
- Trait interfaces keep alternative backends plug-compatible.

### Negative / accepted costs
- sqlite-vec is young (v0.x); we carry pinning + fallback discipline.
- iroh-blobs API churn; pin to a crate version.
- More assembly than adopting a platform lib.
- Pack authors who want "memory blocks" must adopt Letta/Mem0 themselves as pack deps.

### Downstream effects
- [`docs/08-memory-substrate.md`](../docs/08-memory-substrate.md) — storage-backends table updated.
- [`docs/13-v0-scope-90-day.md`](../docs/13-v0-scope-90-day.md) — Memory Graph line updated.
- [`docs/03-chief-kernel.md`](../docs/03-chief-kernel.md) — Memory Graph service description updated.
- Opens: benchmark sqlite-vec vs lance-embedded at a future checkpoint.

## Implementation notes

- One Rust crate: `chief-mem`.
- One SQLite database (`~/.chief/mem/mem.db`) with vec0 + FTS5.
- One blob directory (`~/.chief/mem/blobs/`) using `iroh-blobs`.
- Traits: `VectorIndex`, `BlobStore`, `EmbeddingModel` — each swappable.
- Default embeddings: `fastembed-rs + bge-small-en-v1.5` (see [`adr-0006`](./0006-cas-filesystem.md)).
- Python surface via PyO3 shim.

## Revisit triggers

- sqlite-vec maintainer disappears or project stalls (> 6mo no commits).
- A genuine consortium-governed memory stack emerges that unifies components.
- Benchmarks show sqlite-vec can't scale past 500k nodes (V1 concern).
- Pack authors universally want Letta/Mem0/Zep — in which case expose a sanctioned pack-SDK wrapper, never a kernel dependency.
