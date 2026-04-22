-- Memory Graph Schema (ADR-0008)
-- Pure-OSS stack: SQLite + sqlite-vec + FTS5

-- Nodes table: typed memory atoms
CREATE TABLE IF NOT EXISTS nodes (
  uri TEXT PRIMARY KEY,              -- mem://<type>/<blake3>
  type TEXT NOT NULL,                -- email | file | person | event | decision | artifact | thought | finding
  content_hash TEXT NOT NULL,        -- BLAKE3 hash of canonical payload
  horizon TEXT NOT NULL CHECK (horizon IN ('short', 'medium', 'long', 'open')),
  created_at INTEGER NOT NULL,       -- Unix timestamp
  confidence REAL DEFAULT 1.0,       -- 0.0-1.0; agent self-reported
  source TEXT,                       -- ingester-id | agent-id | human
  blob_ref TEXT,                     -- optional hash for large content
  embedding_id INTEGER,              -- reference to vector embedding
  provenance_ref TEXT,               -- link to provenance log
  tombstoned INTEGER DEFAULT 0,      -- soft delete flag
  body TEXT,                         -- serialized payload (JSON)
  meta_json TEXT,                    -- additional metadata
  UNIQUE(type, content_hash)
);

-- Edges table: typed relationships between nodes
CREATE TABLE IF NOT EXISTS edges (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  src_uri TEXT NOT NULL,
  dst_uri TEXT NOT NULL,
  kind TEXT NOT NULL CHECK (kind IN (
    'derived-from', 'replied-to', 'depends-on', 'refers-to',
    'authored-by', 'cites', 'updates', 'contradicts'
  )),
  weight REAL DEFAULT 1.0,           -- semantic strength
  created_by TEXT,
  provenance_ref TEXT,
  created_at INTEGER NOT NULL,
  meta_json TEXT,
  FOREIGN KEY(src_uri) REFERENCES nodes(uri) ON DELETE CASCADE,
  FOREIGN KEY(dst_uri) REFERENCES nodes(uri) ON DELETE CASCADE
);

-- Full-text search index (FTS5)
CREATE VIRTUAL TABLE IF NOT EXISTS texts USING fts5(
  node_uri UNINDEXED,
  body,
  content=nodes,
  content_rowid=rowid
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_nodes_type ON nodes(type);
CREATE INDEX IF NOT EXISTS idx_nodes_horizon ON nodes(horizon);
CREATE INDEX IF NOT EXISTS idx_nodes_created_at ON nodes(created_at);
CREATE INDEX IF NOT EXISTS idx_nodes_source ON nodes(source);
CREATE INDEX IF NOT EXISTS idx_edges_src ON edges(src_uri);
CREATE INDEX IF NOT EXISTS idx_edges_dst ON edges(dst_uri);
CREATE INDEX IF NOT EXISTS idx_edges_kind ON edges(kind);
CREATE INDEX IF NOT EXISTS idx_edges_created_at ON edges(created_at);
