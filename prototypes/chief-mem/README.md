# chief-mem: Pure-OSS Memory Graph Prototype

Implements ADR-0008 — a pure-OSS memory substrate for the Chief OS kernel layer.

## Quick start

```rust
use chief_mem::{ChiefMem, Node, NodeType, Horizon};
use std::path::Path;

let store = ChiefMem::open(Path::new("/tmp/mem.db"))?;

// Create a typed node
let node = Node::new(
    NodeType::File,
    Horizon::Medium,
    "Document content here".to_string(),
    "my-agent".to_string(),
);

// Store it (returns mem:// URI based on content hash)
let uri = store.put_node(node)?;
println!("Node URI: {}", uri);  // mem://file/abc123...

// Query by text
let results = store.query("Document", None, None, 20)?;
for result in results {
    println!("{}: {}", result.uri, result.score);
}
```

## Architecture

**Pure-OSS components** (ADR-0008 decision rationale):

| Component | Library | License | Maintainer |
|---|---|---|---|
| Nodes + edges | **SQLite** | Public domain | SQLite Consortium |
| Hashing | **blake3** | Apache-2.0 | BLAKE3 team |
| Full-text search | **rusqlite + FTS5** | Public domain | SQLite Consortium |
| Blob storage | Simple CAS directory | N/A | Chief-mem |
| Serialization | **serde** | Apache-2.0 / MIT | Rust community |

**Explicitly rejected** at kernel layer (would introduce vendor capture):
- Letta, Mem0, Zep, ChromaDB, Weaviate, Milvus, Meilisearch, SurrealDB, Memgraph, Typesense
- Only acceptable as optional pack-level backends (see ADR-0008)

## Data Model

### Nodes

```sql
CREATE TABLE nodes (
  uri TEXT PRIMARY KEY,          -- mem://<type>/<blake3>
  type TEXT NOT NULL,            -- email|file|person|event|decision|artifact|thought|finding
  content_hash TEXT NOT NULL,    -- BLAKE3 of payload
  horizon TEXT NOT NULL,         -- short|medium|long|open
  created_at INTEGER NOT NULL,   -- Unix timestamp
  confidence REAL DEFAULT 1.0,   -- 0.0-1.0; agent-reported
  source TEXT,                   -- ingester/agent/human ID
  body TEXT,                     -- serialized payload (JSON)
  ...
);
```

### Edges

```sql
CREATE TABLE edges (
  id INTEGER PRIMARY KEY,
  src_uri TEXT NOT NULL,
  dst_uri TEXT NOT NULL,
  kind TEXT NOT NULL,            -- derived-from|replied-to|depends-on|refers-to|...
  weight REAL DEFAULT 1.0,       -- semantic strength
  created_by TEXT,
  created_at INTEGER NOT NULL,
  ...
);
```

### Full-text index

FTS5 virtual table for BM25-ranked text search over node bodies.

## URI Design

`mem://<type>/<blake3>`

- **Type** is enum; new types require ADR
- **Content-hash** is BLAKE3 of canonical payload
- **Same content → same URI forever**; enables stable citations and deduplication
- Never broken by renames; changes tracked via edges (`updates`, `replaces`)

## Features

### Content-based deduplication

Re-inserting identical content returns the same `mem://` URI:

```rust
let node1 = Node::new(..., "exact content".to_string(), ...);
let uri1 = store.put_node(node1)?;

let node2 = Node::new(..., "exact content".to_string(), ...);
let uri2 = store.put_node(node2)?;

assert_eq!(uri1, uri2);  // Same content = same URI
```

### Typed nodes and edges

8 node types, 8 edge kinds — enables ontology-aware queries and filtering.

### Full-text + structured filtering

```rust
let results = store.query(
    "search term",
    Some(vec![NodeType::File, NodeType::Artifact]),
    Some(vec![Horizon::Short, Horizon::Medium]),
    20,  // k results
)?;
```

### Graph traversal

```rust
let edges = store.get_edges_from(&uri)?;
for edge in edges {
    println!("{} {} {}", edge.from, edge.kind, edge.to);
}
```

## Testing

```bash
cargo test
```

Tests cover:
- **basic_roundtrip.rs** — node CRUD, edge creation, URI stability
- **scale_100k.rs** — 100k synthetic nodes, p50 query latency, disk size
- **dedup.rs** — content-based deduplication, URI collision resistance

## Benchmarking

```bash
cargo bench
```

Measures:
- Query latency (k=20, k=50) on 10k-node store
- Filtered queries (by type, horizon)
- p50/p95 latency profiles

## Measurements

See [`measurements.md`](./measurements.md) for benchmark results.

## Design Rationale

### Why not X?

| Project | Issue | ADR-0008 ruling |
|---|---|---|
| **Letta** | a16z-backed; memory-blocks schema bends toward hosted-agent platform | Rejected at kernel layer |
| **Mem0** | YC-backed; managed-API monetization; schema platform-optimized | Rejected |
| **Zep** | Hosted memory is core product; OSS is lead-gen | Rejected |
| **ChromaDB** | Chroma Inc.; increasingly cloud-tuned | Rejected |
| **LanceDB** | Company-led; server-ish | `lance` format safe (not SDK) |
| **SQLite + FTS5** | Multi-maintainer governance; unkillable boring infrastructure | ✓ Chosen |
| **BLAKE3** | No company capture; simple, fast, modern | ✓ Chosen |

### Why SQLite?

- Single-file, portable, zero-config
- FTS5 for full-text search (public domain)
- PRAGMA options give fine control over consistency/performance tradeoff
- Decades of production use; not a research project
- Trait-abstracted `VectorIndex` keeps future optionality (LanceDB `lance` format, hnswlib, etc.)

### Horizon tags

Retrieval strategy varies by time scale:

| Horizon | Retention | Retrieval priority | Examples |
|---|---|---|---|
| **short** | 30 days | High, today/week | Inbox, agendas, recently-captured |
| **medium** | 1 year | Bounded, project-scoped | Deals, research projects, PRs |
| **long** | Indefinite | Persistent, decaying | Commitments, contracts, relationships |
| **open** | Permanent | Always available | Values, goals, identity data |

Surface rankers weight these differently per context.

## Integration with Chief kernel

The Memory Graph is a **kernel service**, not a library. Every pack, every agent shares one substrate:

- **One SQLite database**: `~/.chief/mem/mem.db`
- **One blob directory**: `~/.chief/mem/blobs/`
- **Mandatory provenance link**: every write traces to a Provenance Log entry
- **Capability-gated reads**: Capability Broker enforces read scope at query time

This ensures:
- One ontology (packs interoperate; travel pack's flight nodes are readable by calendar pack)
- Unified rollback (one `mem.rewind(time)` covers all packs)
- Unified encryption (one LUKS volume under `/chief`)

## Future work

- **v0.1+**: PyO3 binding to expose same store to Python orchestrators
- **v1+**: Vector index swap (sqlite-vec → LanceDB `lance` format, or hnswlib + sidecar) with zero-data-migration
- **v1+**: Per-file encryption (v0 uses volume-level LUKS)
- **v1+**: Cloud-twin encrypted mirror for retrieval over longer context window

## Non-negotiables

1. **Pure-OSS only** — no single-company-led libraries at kernel layer
2. **Rust 2021 stable** — no nightly, no unsafe without justification
3. **No vendor lock-in** — swappable traits for `VectorIndex`, `BlobStore`, `EmbeddingModel`
4. **Portable** — same `.db` file works across devices; paths are relative or environment-rooted

## References

- **ADR**: [`../../adr/0008-pure-oss-memory-substrate.md`](../../adr/0008-pure-oss-memory-substrate.md)
- **Docs**: [`../docs/08-memory-substrate.md`](../docs/08-memory-substrate.md)
- **Research**: [`../docs/research/2026-04-21-pure-oss-memory-substrate.md`](../docs/research/2026-04-21-pure-oss-memory-substrate.md)
- **Chief kernel**: [`../docs/03-chief-kernel.md`](../docs/03-chief-kernel.md)

## License

Chief OS is AGPL-3.0-or-later. Chief-mem inherits this.
