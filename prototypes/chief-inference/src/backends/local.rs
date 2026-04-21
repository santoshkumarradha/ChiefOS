//! Local llama.cpp stub: deterministic echo backend (Tier::Generated).

use super::{BackendId, CapabilitySet, ModelBackend};
use anyhow::Result;

pub struct LocalLlamaCppStub {
    model_name: String,
}

impl LocalLlamaCppStub {
    pub fn new(model_name: impl Into<String>) -> Self {
        Self {
            model_name: model_name.into(),
        }
    }
}

#[async_trait::async_trait]
impl ModelBackend for LocalLlamaCppStub {
    fn id(&self) -> BackendId {
        BackendId(format!("local:llama-cpp-{}", self.model_name))
    }
    
    fn health(&self) -> Result<()> {
        Ok(())
    }
    
    fn capabilities(&self) -> CapabilitySet {
        CapabilitySet {
            max_context: 4096,
            supports_vision: false,
            supports_tools: false,
        }
    }
    
    async fn infer(&self, prompt: &str, _temperature: Option<f32>, _top_p: Option<f32>) -> Result<String> {
        Ok(format!("ECHO: {}", prompt))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn local_stub_infer() {
        let backend = LocalLlamaCppStub::new("test");
        let result = backend.infer("hello", None, None).await.unwrap();
        assert_eq!(result, "ECHO: hello");
    }
}
