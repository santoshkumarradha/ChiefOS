//! Model backends: local and cloud stubs for inference.

pub mod cloud_stub;
pub mod local;

pub use cloud_stub::CloudAnthropicStub;
pub use local::LocalLlamaCppStub;

use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct BackendId(pub String);

#[derive(Debug, Clone)]
pub struct CapabilitySet {
    pub max_context: usize,
    pub supports_vision: bool,
    pub supports_tools: bool,
}

#[async_trait::async_trait]
pub trait ModelBackend: Send + Sync {
    fn id(&self) -> BackendId;
    fn health(&self) -> Result<()>;
    fn capabilities(&self) -> CapabilitySet;
    async fn infer(&self, prompt: &str, temperature: Option<f32>, top_p: Option<f32>) -> Result<String>;
}
