//! Single-shot structured LLM inference via `ctx.ai()` (ADR-0013).
#![allow(private_interfaces)]

use crate::ai_error::AiError;
use crate::tier::Tier;
use async_trait::async_trait;
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::Value;
use std::sync::Arc;

/// Builder for a single-shot structured LLM call.
///
/// Chain methods and call `.call()` or `.stream()` to invoke.
/// Example:
/// ```ignore
/// let result = ctx.ai()
///     .prompt("Classify this email")
///     .input(&email_text)
///     .schema::<IntakeResult>()
///     .tier(Tier::Fast)
///     .max_tokens(500)
///     .call::<IntakeResult>()
///     .await?;
/// ```
pub struct AiBuilder {
    backend: Arc<dyn AiBackend>,
    prompt: Option<String>,
    input: Option<Value>,
    schema_name: Option<String>,
    tier: Tier,
    max_tokens: u32,
}

impl AiBuilder {
    /// Create a new AI builder (internal use; obtained from `ctx.ai()`).
    pub(crate) fn new(backend: Arc<dyn AiBackend>) -> Self {
        Self {
            backend,
            prompt: None,
            input: None,
            schema_name: None,
            tier: Tier::Fast,
            max_tokens: 1024,
        }
    }

    /// Set the prompt.
    pub fn prompt(mut self, p: impl Into<String>) -> Self {
        self.prompt = Some(p.into());
        self
    }

    /// Set the input to classify/process.
    pub fn input<T: Serialize>(mut self, v: &T) -> Self {
        self.input = Some(serde_json::to_value(v).unwrap_or(Value::Null));
        self
    }

    /// Set the expected output schema.
    pub fn schema<T: DeserializeOwned + schemars::JsonSchema>(mut self) -> Self {
        self.schema_name = Some(std::any::type_name::<T>().to_string());
        self
    }

    /// Set the tier (default: Fast).
    pub fn tier(mut self, t: Tier) -> Self {
        self.tier = t;
        self
    }

    /// Set max tokens (default: 1024, clamped by grant).
    pub fn max_tokens(mut self, n: u32) -> Self {
        self.max_tokens = n;
        self
    }

    /// Execute the call and return structured output.
    pub async fn call<T: DeserializeOwned>(self) -> Result<T, AiError> {
        let req = AiRequest {
            prompt: self.prompt.unwrap_or_default(),
            input: self.input.unwrap_or(Value::Null),
            schema_name: self.schema_name.unwrap_or_default(),
            tier: self.tier,
            max_tokens: self.max_tokens,
        };

        let resp = self.backend.call_one(&req).await?;
        serde_json::from_value::<T>(resp.output).map_err(|e| AiError::Schema(e.to_string()))
    }

    /// Execute the call and stream partial tokens as raw values.
    pub async fn stream(
        self,
    ) -> Result<
        Box<dyn futures::stream::Stream<Item = Result<Value, AiError>> + Unpin + Send>,
        AiError,
    > {
        let req = AiRequest {
            prompt: self.prompt.unwrap_or_default(),
            input: self.input.unwrap_or(Value::Null),
            schema_name: self.schema_name.unwrap_or_default(),
            tier: self.tier,
            max_tokens: self.max_tokens,
        };

        self.backend.stream_one(&req).await
    }
}

/// Request to the AI backend.
#[derive(Clone, Debug)]
#[allow(dead_code)]
pub(crate) struct AiRequest {
    pub prompt: String,
    pub input: Value,
    pub schema_name: String,
    pub tier: Tier,
    pub max_tokens: u32,
}

/// Response from the AI backend.
#[derive(Clone, Debug)]
#[allow(dead_code)]
pub(crate) struct AiResponse {
    pub output: Value,
    pub model: String,
    pub tokens_used: u32,
}

/// Trait that chief-core implements to actually run AI calls.
/// Packs never use this directly; they use AiBuilder.
#[async_trait]
pub trait AiBackend: Send + Sync {
    /// Execute a single AI call.
    async fn call_one(&self, req: &AiRequest) -> Result<AiResponse, AiError>;

    /// Stream tokens from an AI call.
    async fn stream_one(
        &self,
        req: &AiRequest,
    ) -> Result<
        Box<dyn futures::stream::Stream<Item = Result<Value, AiError>> + Unpin + Send>,
        AiError,
    >;
}

/// In-memory stub backend for testing.
pub struct InMemoryAiBackend;

#[async_trait]
impl AiBackend for InMemoryAiBackend {
    async fn call_one(&self, _req: &AiRequest) -> Result<AiResponse, AiError> {
        // Return a simple string response that works for String deserialization.
        // For structured types, return a JSON object.
        let output = serde_json::json!("test response");
        Ok(AiResponse {
            output,
            model: "in-memory-stub".to_string(),
            tokens_used: 42,
        })
    }

    async fn stream_one(
        &self,
        _req: &AiRequest,
    ) -> Result<
        Box<dyn futures::stream::Stream<Item = Result<Value, AiError>> + Unpin + Send>,
        AiError,
    > {
        use futures::stream::{self, StreamExt};
        let stream = stream::once(async { Ok(serde_json::json!("test")) });
        Ok(Box::new(stream.boxed()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn ai_builder_basic() {
        let backend = Arc::new(InMemoryAiBackend);
        let result = AiBuilder::new(backend)
            .prompt("test")
            .tier(Tier::Deep)
            .call::<String>()
            .await;
        assert!(result.is_ok());
    }

    #[test]
    fn ai_builder_tier_default() {
        let backend = Arc::new(InMemoryAiBackend);
        let builder = AiBuilder::new(backend);
        assert_eq!(builder.tier, Tier::Fast);
    }
}
