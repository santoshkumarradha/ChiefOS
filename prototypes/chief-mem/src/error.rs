use thiserror::Error;

#[derive(Debug, Error)]
pub enum ChiefMemError {
    #[error("invalid {kind}: {value}")]
    InvalidEnum { kind: &'static str, value: String },
    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("mutex poisoned: {0}")]
    Poisoned(String),
    #[error("node not found: {0}")]
    NodeNotFound(String),
    #[error("invalid vector dimension: expected {expected}, got {actual}")]
    InvalidVectorDimension { expected: usize, actual: usize },
}

pub type Result<T> = std::result::Result<T, ChiefMemError>;
