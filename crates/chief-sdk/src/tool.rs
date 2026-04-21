//! Tool trait — typed API wrappers for pack-exposed functionality.

use serde::{Deserialize, Serialize};

/// A typed tool: Req -> Res.
pub trait Tool<Req: Send + Sync, Res: Send + Sync>: Send + Sync {
    fn invoke(&self, req: Req) -> crate::error::Result<Res>;
}

/// Tool metadata for discovery/registration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDescriptor {
    pub id: String,
    pub pack_id: String,
    pub description: String,
    pub input_schema: serde_json::Value,
    pub output_schema: serde_json::Value,
}
