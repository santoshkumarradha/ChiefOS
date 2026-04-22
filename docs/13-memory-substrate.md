---
id: memory-substrate
title: "Memory Substrate"
status: draft
owners: [santosh]
last_updated: 2026-04-21
related: [chief-kernel, architecture, hax-principles]
depends_on: [architecture]
tags: [memory, graph, storage, retrieval]
---

# Memory Substrate

> **Canonical stack decision:** [ADR-0008](../adr/0008-pure-oss-memory-substrate.md) — pure-OSS composition at the kernel layer (SQLite + sqlite-vec + fastembed-rs + iroh-blobs). Blob CAS alignment: [ADR-0006](../adr/0006-cas-filesystem.md).

## TL;DR

- Memory Graph is a **kernel service**, not a library. One substrate for every pack, every agent, every surface.
- Typed nodes (email, file, person, event, decision, artifact, thought, finding).
- Typed edges (derived-from, replied-to, depends-on, refers-to, authored-by, cites).
- Stable URIs `mem://<type>/<content-hash>` — addressable forever, cite-able by agents.
- Horizon tags drive retrieval: short (today/week), medium (project), long (multi-month), open (identity-level).
- Backend is **pure-OSS by policy**: SQLite + sqlite-vec for nodes+edges+vectors, fastembed-rs for embeddings, iroh-blobs for content-addressed blobs. No startup-led OSS at this layer.

## Why a kernel service

If the memory graph is a library, every pack ships its own half-broken version. If it's a kernel service:

| Benefit | Why it only works centrally |
|---|---|
| One ontology | Packs interoperate: travel pack's flight nodes readable by calendar pack |
| Guaranteed provenance | Every write links to Provenance Log entry; cannot be skipped |
| Capability-gated reads | Broker enforces scope at retrieval time (finance pack cannot read personal email nodes) |
| Unified rollback | One rewind API covers all packs' data |
| Unified encryption, backup, cloud-twin sync | One implementation, one audit |

## Node schema

```yaml
node:
  uri: mem://<type>/<content-hash>
  type: email | file | person | event | decision | artifact | thought | finding | ...
  created_at: <timestamp>
  horizon: short | medium | long | open
  confidence: 0.0–1.0        # agent self-reported; 1.0 for ground truth
  source: <ingester-id> | <agent-id> | <human>
  blob_ref: <optional>       # for large content
  embedding_ref: <optional>  # vector id in sqlite-vec (ADR-0008)
  provenance_ref: <prov-entry-id>
  tombstoned: false          # soft delete for rollback
  body: <typed payload>
```

Payloads per type:

| Type | Payload shape |
|---|---|
| `email` | thread_id, sender, recipients, subject, body_text, body_html, attachments[] |
| `file` | path, mime, size, content_hash, local_or_cloud, line_index |
| `person` | name, handles, email, relationships[] |
| `event` | start, end, attendees, location, agenda |
| `decision` | question, answer, alternatives[], rationale, stakes |
| `artifact` | kind (draft_reply, report, code_patch), body, target (email_id, file_path) |
| `thought` | content, triggered_by (mem_uri), horizon |
| `finding` | content, supports[] (mem_uris), confidence, refuted_by[] |

## Edge schema

```yaml
edge:
  from: <node_uri>
  to: <node_uri>
  kind: derived-from | replied-to | depends-on | refers-to | authored-by | cites | updates | contradicts
  weight: 0.0–1.0             # optional; semantic strength
  created_by: <agent-id>
  provenance_ref: <prov-entry-id>
```

## URI design

`mem://<type>/<content-hash>`

- Type is enum; adding a type requires ADR.
- Content-hash is BLAKE3 of canonical payload; same content → same URI, forever.
- URIs are the lingua franca of citations. When an agent drafts a reply "citing page 14 ¶3," that citation is `mem://file/a1b2.../14/3` — stable, verifiable.
- Never broken by renames. Source path moves happen through edges (`replaces`).

```mermaid
flowchart LR
    F[mem://file/abcd<br/>stanford-paper.pdf]
    T[mem://thought/pqr<br/>"section 14.3 matters"]
    D[mem://decision/xyz<br/>"reply with quote"]
    A[mem://artifact/wvu<br/>draft_reply]
    E[mem://email/stu<br/>sent response]

    F -->|refers-to| T
    T -->|informs| D
    D -->|authored-by| A
    A -->|replied-to| E
```

Diagram also at [`diagrams/memory-schema.mmd`](./diagrams/memory-schema.mmd).

## Horizon tags — retrieval strategy

| Horizon | Retention | Retrieval priority | Typical nodes |
|---|---|---|---|
| short | 30 days | High for today / this week | Inbox items, meeting agendas |
| medium | 1 year | Bounded, project-scoped | Deal threads, research project notes |
| long | indefinite | Decaying but persistent | Ongoing commitments, contracts, relationships |
| open | permanent | Always available | Values, goals, user profile, house rules |

Retrieval rankers weight these differently per surface. Morning Brief privileges short+medium; Quarterly Review privileges long+open; Chat adapts to the query.

## Retrieval API

```
mem.search(
    query: str,
    types: list[NodeType] | None = None,
    horizon: list[Horizon] | None = None,
    time_window: TimeRange | None = None,
    caps: CapabilityContext,           # enforces read scope
    k: int = 20,
    rerank: bool = True,
) -> list[NodeScore]
```

- Default: vector search over embedding + BM25 over text + graph-walk boost (related-to-recent-nodes).
- Caps are mandatory: if the caller lacks read scope for a node type, it is silently filtered out of results (not errored, to avoid oracle-style leaks).
- Rerank: optional LLM rerank for top-K candidates; configurable per surface.

## Why pure-OSS?

At the kernel layer we refuse single-startup-led OSS. A startup-governed dependency is a one-company failure mode: license relicensing, dual-licensing pivots, abandonment, or acquisition all propagate directly into the substrate every agent depends on. Community- or consortium-governed libraries (SQLite, SQLite extensions, ONNX Runtime, Protocol Labs alumni projects) let us customize at OS level without vendor capture. Full rationale + rejected alternatives in [ADR-0008](../adr/0008-pure-oss-memory-substrate.md) and [`research/2026-04-21-pure-oss-memory-substrate.md`](./research/2026-04-21-pure-oss-memory-substrate.md).

Blob-level content addressing aligns with the OS-wide CAS primitive — see [ADR-0006](../adr/0006-cas-filesystem.md) for the filesystem side of the same hash.

## Storage backends

**Principle (directive 2026-04-21, codified in [ADR-0008](../adr/0008-pure-oss-memory-substrate.md)):** at the kernel layer, no single-startup-led OSS. Compose from community- or consortium-governed libraries so we can customize at OS level without vendor capture. Full rationale + rejected alternatives in [`research/2026-04-21-pure-oss-memory-substrate.md`](./research/2026-04-21-pure-oss-memory-substrate.md).

| Component | v0 | Maintainer | License |
|---|---|---|---|
| Nodes + edges | **SQLite** (hand-rolled typed-edge schema) | SQLite Consortium | Public domain |
| Vectors | **sqlite-vec** | Alex Garcia + contributors, no parent co. | Apache-2.0 / MIT |
| Embeddings inference | **fastembed-rs + ort (ONNX Runtime)** | fastembed community; ort via Microsoft ONNX upstream | Apache-2.0 |
| Blobs | **iroh-blobs crate** (not the daemon) — same content-hash plane as [ADR-0006](../adr/0006-cas-filesystem.md) CAS FS | Number 0 + Protocol Labs alumni | Apache-2.0 / MIT |
| Full-text | **SQLite FTS5** | SQLite Consortium | Public domain |
| Python binding | **PyO3** | Community | Apache-2.0 / MIT |
| Startup-led libs at kernel | **Rejected.** (Letta, Mem0, Zep, Chroma-client, SurrealDB, Memgraph, Weaviate, Milvus, Vespa, Meilisearch, Typesense.) Acceptable only as optional pack-level backends. | — | — |

Swap-ability via traits (`VectorIndex`, `BlobStore`, `EmbeddingModel`) preserves future optionality per lane.

All under the encrypted user volume.

## Privacy & encryption

- At rest: LUKS for the entire Chief data volume (v0). Per-file keying (v1+).
- In transit (to cloud twin): all blobs + vectors encrypted with device key before upload; cloud twin sees only ciphertext for sensitive-horizon nodes.
- Sensitive categories (health, finance, personal): never replicate to cloud twin. Local-only. Region Router enforces.

## Sync with cloud twin

- Cloud twin is a stateless compute surface. It receives encrypted blobs + vector queries, returns computed outputs.
- For non-sensitive horizons, an optional encrypted mirror lives in cloud for search/retrieval over a larger context window when locally paging is too slow.
- Mirror is keyed by the device key; cloud provider cannot decrypt.

## Rollback semantics

- Nodes are **append-only**; "delete" is tombstone + Provenance Log entry.
- Edges can be soft-removed; rollback un-removes.
- Rollback API: `mem.rewind(time_t)` → applies all tombstones created after `time_t` in reverse.
- Cross-service: `chief rewind 4h` coordinates Memory Graph rollback + Agent Runtime queue revert + Capability Broker grant revocation.

## Ingester contract

Every ingester (per pack) emits:

```yaml
ingest_event:
  source: <pack-id>/<ingester-id>
  raw_blob: <content-addressed>
  detected_type: <node-type>
  suggested_horizon: <horizon>
  suggested_edges: [...]
```

Kernel normalizes, adds provenance, commits.

## Acceptance

- [ ] Memory Graph URIs stable across pack updates (verified by content-hash test suite).
- [ ] Cap-scoped retrieval filter is oracle-safe (no leak via result count).
- [ ] Rewind API validated end-to-end on files, drafts, sent-but-unshippable actions.
- [ ] Cloud-twin ciphertext-only verified by external pen-test.
- [ ] Vector backend swap (v0 → v1) performed without data migration.

## Open questions

1. Do we ship BLAKE3 (fast, modern) or SHA-256 (broader tooling support)? Bias: BLAKE3.
2. Should embedding model be pinned per node (provenance-stable vectors) or re-embedded on model upgrade?
3. How do we handle "same entity, different form" (same person via email vs LinkedIn) — canonicalize at ingest or at query?
4. Is IPLD compatibility worth the abstraction cost for blobs, or is a flat hash-named tree enough?
5. Retention policy for short-horizon nodes: hard-delete at 30 days, or just drop from retrieval index?

## Related

- [`chief-kernel`](./11-chief-kernel.md) — Memory Graph service contract
- [`security-model`](./14-security-model.md) — encryption + cap-gated reads
- [`local-vs-cloud`](./41-local-vs-cloud.md) — what replicates and what doesn't
- [`hax-principles`](./01-hax-principles.md) — horizon-driven retrieval
