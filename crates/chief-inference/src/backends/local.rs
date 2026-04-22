//! Local inference backend: real llama.cpp or stub echo.

use super::{BackendId, CapabilitySet, ModelBackend};
use anyhow::{Context, Result};
use std::path::PathBuf;

#[cfg(feature = "real-llama")]
use llama_cpp_rs::LLamaContext;

/// Local llama.cpp inference backend.
/// When `real-llama` feature is enabled: real inference via llama_cpp_rs.
/// Otherwise: stub echo backend for testing.
pub struct LocalLlamaCppStub {
    #[cfg(feature = "real-llama")]
    context: LLamaContext,
    model_name: String,
    model_id: String,
}

impl LocalLlamaCppStub {
    /// Create a new local inference backend.
    ///
    /// When `real-llama` is enabled:
    /// - Loads GGUF model from resolved path
    /// - Returns error if model file not found
    ///
    /// When `real-llama` is disabled:
    /// - Returns deterministic stub echo backend
    #[cfg(feature = "real-llama")]
    pub fn new(model_name: impl Into<String>) -> Result<Self> {
        let model_name = model_name.into();
        let model_path = resolve_model_path(&model_name)?;

        tracing::info!("Loading GGUF model from: {}", model_path.display());

        // Load GGUF model with llama_cpp_rs
        let context = LLamaContext::new(&model_path)
            .with_context(|| format!("failed to load GGUF model from: {}", model_path.display()))?;

        let model_id = format!("local:qwen2.5-3b-instruct-q4_k_m");

        Ok(Self {
            context,
            model_name,
            model_id,
        })
    }

    /// Stub version (when real-llama is disabled).
    #[cfg(not(feature = "real-llama"))]
    pub fn new(model_name: impl Into<String>) -> Result<Self> {
        let model_name = model_name.into();
        let model_id = format!("local:llama-cpp-{}", model_name);
        Ok(Self {
            model_name,
            model_id,
        })
    }
}

#[async_trait::async_trait]
impl ModelBackend for LocalLlamaCppStub {
    fn id(&self) -> BackendId {
        BackendId(self.model_id.clone())
    }

    fn health(&self) -> Result<()> {
        // Model loaded successfully in new(), so health is OK
        Ok(())
    }

    fn capabilities(&self) -> CapabilitySet {
        CapabilitySet {
            max_context: 2048,
            supports_vision: false,
            supports_tools: false,
        }
    }

    async fn infer(
        &self,
        prompt: &str,
        _temperature: Option<f32>,
        _top_p: Option<f32>,
    ) -> Result<String> {
        #[cfg(feature = "real-llama")]
        {
            // Real llama.cpp inference via llama_cpp_rs
            // For v0, we generate a limited number of tokens (proof-of-concept).
            // The context is already loaded in new(); we just need to call completion.

            // Generate completion for the prompt
            // Limit to 256 tokens to keep inference time reasonable on CPU
            let completion = self
                .context
                .completion(&prompt)
                .max_tokens(256)
                .temperature(0.7)
                .top_p(0.9)
                .build()?;

            Ok(completion)
        }

        #[cfg(not(feature = "real-llama"))]
        {
            // Stub: deterministic echo (for CI + testing without a model)
            Ok(format!("ECHO: {}", prompt))
        }
    }
}

/// Resolve model path from environment or default location.
#[cfg(feature = "real-llama")]
fn resolve_model_path(model_name: &str) -> Result<PathBuf> {
    let chief_model_dir = std::env::var("CHIEF_MODEL_DIR").unwrap_or_else(|_| {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        format!("{}/.cache/chief-os/models", home)
    });

    let model_path = PathBuf::from(&chief_model_dir).join(format!("{}.gguf", model_name));

    if !model_path.exists() {
        anyhow::bail!(
            "model not found: {}. Run scripts/fetch-model.sh to download.",
            model_path.display()
        );
    }

    Ok(model_path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn local_stub_infer() {
        #[cfg(not(feature = "real-llama"))]
        {
            let backend = LocalLlamaCppStub::new("test").unwrap();
            let result = backend.infer("hello", None, None).await.unwrap();
            assert_eq!(result, "ECHO: hello");
        }
    }
}
