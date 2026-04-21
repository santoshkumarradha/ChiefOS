//! Ingester trait — data ingestion into Memory Graph.

use crate::error::Result;
use async_trait::async_trait;

/// An ingester: feeds data into the Memory Graph.
#[async_trait]
pub trait Ingester: Send + Sync {
    /// Called when data arrives.
    async fn on_ingest(&self, data: serde_json::Value) -> Result<()>;
}
