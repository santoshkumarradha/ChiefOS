//! Chief-Mem: Pure-OSS Memory Graph prototype (ADR-0008)
//!
//! A memory graph implementation using:
//! - SQLite for nodes and edges
//! - BLAKE3 for content hashing
//! - FTS5 for full-text search
//! - Simple CAS directory for blob storage

pub mod edge;
pub mod error;
pub mod node;

use chrono::Utc;
use error::{ChiefMemError, Result};
use rusqlite::{params, Connection, OptionalExtension};
use std::path::{Path, PathBuf};

pub use edge::{Edge, EdgeKind};
pub use node::{format_uri, Horizon, Node, NodeType};

/// Main memory graph store.
pub struct ChiefMem {
    db_path: PathBuf,
    blob_dir: PathBuf,
    conn: Connection,
}

impl ChiefMem {
    /// Open or create a memory graph at the given path.
    pub fn open(path: &Path) -> Result<Self> {
        let db_path = path.to_path_buf();
        let blob_dir = db_path
            .parent()
            .ok_or_else(|| ChiefMemError::InvalidEnum {
                kind: "path",
                value: "parent directory not found".to_string(),
            })?
            .join("blobs");

        std::fs::create_dir_all(&blob_dir)?;

        let conn = Connection::open(&db_path)?;
        conn.execute_batch("PRAGMA journal_mode = WAL; PRAGMA synchronous = NORMAL;")?;

        let mem = ChiefMem {
            db_path,
            blob_dir,
            conn,
        };

        mem.init_schema()?;
        Ok(mem)
    }

    /// Initialize database schema.
    fn init_schema(&self) -> Result<()> {
        let schema_sql = include_str!("schema.sql");
        self.conn.execute_batch(schema_sql)?;
        Ok(())
    }

    /// Store a node and return its mem:// URI.
    pub fn put_node(&self, node: Node) -> Result<String> {
        let content_hash = blake3::hash(node.body.as_bytes());
        let hash_str = content_hash.to_hex().to_string();
        let uri = node::format_uri(node.node_type, &hash_str);
        let now = Utc::now().timestamp();

        self.conn.execute(
            "INSERT OR IGNORE INTO nodes
            (uri, type, content_hash, horizon, created_at, confidence, source, body, blob_ref)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                &uri,
                node.node_type.to_string(),
                &hash_str,
                node.horizon.to_string(),
                now,
                node.confidence,
                &node.source,
                &node.body,
                &node.blob_ref,
            ],
        )?;

        Ok(uri)
    }

    /// Store an edge between two nodes.
    pub fn put_edge(&self, edge: Edge) -> Result<()> {
        let now = Utc::now().timestamp();

        self.conn.execute(
            "INSERT INTO edges
            (src_uri, dst_uri, kind, weight, created_by, created_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                &edge.from,
                &edge.to,
                edge.kind.to_string(),
                edge.weight,
                &edge.created_by,
                now,
            ],
        )?;

        Ok(())
    }

    /// Store a blob and return its BLAKE3 hash.
    pub fn put_blob(&self, data: &[u8]) -> Result<String> {
        let hash = blake3::hash(data);
        let hash_str = hash.to_hex().to_string();
        let blob_path = self.blob_dir.join(&hash_str);

        if !blob_path.exists() {
            std::fs::write(&blob_path, data)?;
        }

        Ok(hash_str)
    }

    /// Query nodes by text with optional filtering.
    pub fn query(
        &self,
        text: &str,
        types: Option<Vec<NodeType>>,
        horizon: Option<Vec<Horizon>>,
        k: usize,
    ) -> Result<Vec<(String, f64)>> {
        // Build WHERE clause for type and horizon filters
        let mut where_parts = vec!["1=1".to_string()];

        if let Some(type_list) = &types {
            if !type_list.is_empty() {
                let type_strs: Vec<String> = type_list.iter().map(|t| format!("'{}'", t)).collect();
                where_parts.push(format!("n.type IN ({})", type_strs.join(",")));
            }
        }

        if let Some(horizon_list) = &horizon {
            if !horizon_list.is_empty() {
                let horizon_strs: Vec<String> =
                    horizon_list.iter().map(|h| format!("'{}'", h)).collect();
                where_parts.push(format!("n.horizon IN ({})", horizon_strs.join(",")));
            }
        }

        let where_clause = where_parts.join(" AND ");

        // BM25 full-text search (FTS5)
        let query_sql = format!(
            "
            SELECT n.uri,
                   rank * COALESCE(n.confidence, 1.0) as score
            FROM texts t
            JOIN nodes n ON t.node_uri = n.uri
            WHERE t.body MATCH ? AND {}
            ORDER BY score DESC
            LIMIT ?
            ",
            where_clause
        );

        let mut stmt = self.conn.prepare(&query_sql)?;
        let results: Vec<(String, f64)> = stmt
            .query_map(params![text, k as i32], |row| {
                Ok((row.get(0)?, row.get(1)?))
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;

        Ok(results)
    }

    /// Get a single node by URI.
    pub fn get_node(&self, uri: &str) -> Result<Option<Node>> {
        let mut stmt = self.conn.prepare(
            "SELECT type, horizon, body, confidence, source, blob_ref
             FROM nodes WHERE uri = ?1",
        )?;

        let node = stmt
            .query_row([uri], |row| {
                let node_type_str: String = row.get(0)?;
                let horizon_str: String = row.get(1)?;

                let node_type = node_type_str
                    .parse()
                    .map_err(|_| rusqlite::Error::InvalidQuery)?;
                let horizon = horizon_str
                    .parse()
                    .map_err(|_| rusqlite::Error::InvalidQuery)?;

                Ok(Node {
                    node_type,
                    horizon,
                    body: row.get(2)?,
                    confidence: row.get(3)?,
                    source: row.get(4)?,
                    blob_ref: row.get(5)?,
                    meta_json: None,
                })
            })
            .optional()?;

        Ok(node)
    }

    /// Get edges from a source node.
    pub fn get_edges_from(&self, uri: &str) -> Result<Vec<Edge>> {
        let mut stmt = self
            .conn
            .prepare("SELECT dst_uri, kind, weight, created_by FROM edges WHERE src_uri = ?1")?;

        let edges: Vec<Edge> = stmt
            .query_map([uri], |row| {
                let kind_str: String = row.get(1)?;
                Ok(Edge {
                    from: uri.to_string(),
                    to: row.get(0)?,
                    kind: kind_str
                        .parse()
                        .map_err(|_| rusqlite::Error::InvalidQuery)?,
                    weight: row.get(2)?,
                    created_by: row.get(3)?,
                })
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;

        Ok(edges)
    }

    /// Count total nodes in the store.
    pub fn count_nodes(&self) -> Result<i64> {
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM nodes WHERE tombstoned = 0",
            [],
            |row| row.get(0),
        )?;
        Ok(count)
    }

    /// Count total edges in the store.
    pub fn count_edges(&self) -> Result<i64> {
        let count: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM edges", [], |row| row.get(0))?;
        Ok(count)
    }

    /// Get database file path.
    pub fn db_path(&self) -> &Path {
        &self.db_path
    }

    /// Get blob directory path.
    pub fn blob_dir(&self) -> &Path {
        &self.blob_dir
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_uri_stability() {
        let body = "test content";
        let hash = blake3::hash(body.as_bytes());
        let hash_str = hash.to_hex().to_string();

        let uri1 = node::format_uri(NodeType::File, &hash_str);
        let uri2 = node::format_uri(NodeType::File, &hash_str);

        assert_eq!(uri1, uri2);
        assert!(uri1.starts_with("mem://file/"));
    }

    #[test]
    fn test_horizon_parse() {
        use std::str::FromStr;
        assert_eq!(Horizon::from_str("short").unwrap(), Horizon::Short);
        assert_eq!(Horizon::from_str("medium").unwrap(), Horizon::Medium);
        assert_eq!(Horizon::from_str("long").unwrap(), Horizon::Long);
        assert_eq!(Horizon::from_str("open").unwrap(), Horizon::Open);
    }

    #[test]
    fn test_edge_kind_display() {
        assert_eq!(EdgeKind::Cites.to_string(), "cites");
        assert_eq!(EdgeKind::DependsOn.to_string(), "depends-on");
    }
}
