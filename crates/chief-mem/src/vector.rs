use rusqlite::{params, Connection, OptionalExtension};

use crate::embed::{
    decode_dense_embedding, decode_sparse_embedding, encode_dense_embedding,
    encode_sparse_embedding, EMBEDDING_DIM,
};
use crate::error::Result;

pub trait VectorIndex {
    fn put(&self, conn: &Connection, node_uri: &str, embedding: &[f32]) -> Result<()>;
    fn all(&self, conn: &Connection) -> Result<Vec<(String, IndexedEmbedding)>>;
}

#[derive(Debug, Clone)]
pub enum IndexedEmbedding {
    Dense(Vec<f32>),
    Sparse(Vec<(u16, f32)>),
}

#[derive(Debug, Clone)]
pub struct SqliteVectorIndex {
    mode: VectorMode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VectorMode {
    SqliteVec,
    FallbackTable,
}

impl SqliteVectorIndex {
    pub fn migrate(conn: &Connection) -> Result<Self> {
        let mode = match conn.execute(
            "CREATE VIRTUAL TABLE IF NOT EXISTS vecs USING vec0(
              node_uri TEXT PRIMARY KEY,
              embedding FLOAT[384]
            )",
            [],
        ) {
            Ok(_) => VectorMode::SqliteVec,
            Err(_) => {
                conn.execute(
                    "CREATE TABLE IF NOT EXISTS vecs_fallback (
                      node_uri TEXT PRIMARY KEY,
                      embedding_blob BLOB NOT NULL
                    )",
                    [],
                )?;
                VectorMode::FallbackTable
            }
        };
        Ok(Self { mode })
    }

    pub fn mode(&self) -> VectorMode {
        self.mode
    }
}

impl VectorIndex for SqliteVectorIndex {
    fn put(&self, conn: &Connection, node_uri: &str, embedding: &[f32]) -> Result<()> {
        if embedding.len() != EMBEDDING_DIM {
            return Err(crate::error::ChiefMemError::InvalidVectorDimension {
                expected: EMBEDDING_DIM,
                actual: embedding.len(),
            });
        }

        match self.mode {
            VectorMode::SqliteVec => {
                let encoded = encode_dense_embedding(embedding)?;
                conn.execute(
                    "INSERT OR REPLACE INTO vecs(node_uri, embedding) VALUES (?1, ?2)",
                    params![node_uri, encoded],
                )?;
            }
            VectorMode::FallbackTable => {
                let encoded = encode_sparse_embedding(embedding)?;
                conn.execute(
                    "INSERT OR REPLACE INTO vecs_fallback(node_uri, embedding_blob) VALUES (?1, ?2)",
                    params![node_uri, encoded],
                )?;
            }
        }

        let rowid: Option<i64> = conn
            .query_row(
                "SELECT rowid FROM nodes WHERE uri = ?1",
                params![node_uri],
                |row| row.get(0),
            )
            .optional()?;
        if let Some(rowid) = rowid {
            conn.execute(
                "UPDATE nodes SET embedding_ref = ?1 WHERE uri = ?2",
                params![rowid, node_uri],
            )?;
        }
        Ok(())
    }

    fn all(&self, conn: &Connection) -> Result<Vec<(String, IndexedEmbedding)>> {
        match self.mode {
            VectorMode::SqliteVec => {
                let mut stmt = conn.prepare("SELECT node_uri, embedding FROM vecs")?;
                let rows = stmt.query_map([], |row| {
                    let uri: String = row.get(0)?;
                    let encoded: Vec<u8> = row.get(1)?;
                    Ok((uri, encoded))
                })?;
                rows.map(|row| {
                    let (uri, encoded) = row?;
                    Ok((uri, IndexedEmbedding::Dense(decode_dense_embedding(&encoded)?)))
                })
                .collect()
            }
            VectorMode::FallbackTable => {
                let mut stmt =
                    conn.prepare("SELECT node_uri, embedding_blob FROM vecs_fallback")?;
                let rows = stmt.query_map([], |row| {
                    let uri: String = row.get(0)?;
                    let encoded: Vec<u8> = row.get(1)?;
                    Ok((uri, encoded))
                })?;
                rows.map(|row| {
                    let (uri, encoded) = row?;
                    Ok((uri, IndexedEmbedding::Sparse(decode_sparse_embedding(&encoded)?)))
                })
                .collect()
            }
        }
    }
}
