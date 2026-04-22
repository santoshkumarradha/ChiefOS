//! OpenRouter cloud inference backend (real HTTP).
//!
//! Implements `ModelBackend` by POSTing to OpenRouter's chat/completions endpoint.
//! This is the first backend that actually reaches a real cloud provider — prior
//! cloud backends (cloud_stub.rs) were canned-response placeholders.
//!
//! API key is read from `OPENROUTER_API_KEY` at construction. Tests that exercise
//! this backend should be gated with `#[ignore]` so CI doesn't incur cost; invoke
//! them manually with `cargo test -- --ignored openrouter`.

use super::{BackendId, CapabilitySet, ModelBackend};
use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use std::time::Duration;

const OPENROUTER_URL: &str = "https://openrouter.ai/api/v1/chat/completions";

pub struct OpenRouterBackend {
    api_key: String,
    model: String,
    http: reqwest::Client,
}

impl OpenRouterBackend {
    /// Construct from env. Requires `OPENROUTER_API_KEY` to be set.
    pub fn from_env(model: impl Into<String>) -> Result<Self> {
        let api_key = std::env::var("OPENROUTER_API_KEY").context("OPENROUTER_API_KEY not set")?;
        Self::new(api_key, model)
    }

    pub fn new(api_key: impl Into<String>, model: impl Into<String>) -> Result<Self> {
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(60))
            .build()?;
        Ok(Self {
            api_key: api_key.into(),
            model: model.into(),
            http,
        })
    }
}

#[derive(Serialize)]
struct ChatRequest<'a> {
    model: &'a str,
    messages: Vec<ChatMessage<'a>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
}

#[derive(Serialize)]
struct ChatMessage<'a> {
    role: &'a str,
    content: &'a str,
}

#[derive(Deserialize, Debug)]
struct ChatResponse {
    choices: Vec<ChatChoice>,
    #[serde(default)]
    error: Option<ChatError>,
}

#[derive(Deserialize, Debug)]
struct ChatChoice {
    message: ChatResponseMessage,
}

#[derive(Deserialize, Debug)]
struct ChatResponseMessage {
    #[serde(default)]
    content: String,
}

#[derive(Deserialize, Debug)]
struct ChatError {
    message: String,
    #[allow(dead_code)]
    #[serde(default)]
    code: Option<serde_json::Value>,
}

#[async_trait::async_trait]
impl ModelBackend for OpenRouterBackend {
    fn id(&self) -> BackendId {
        BackendId(format!("openrouter:{}", self.model))
    }

    fn health(&self) -> Result<()> {
        // We don't ping on health() — that would burn quota. Construction already
        // validated the API key exists; real liveness is proven by a successful infer.
        Ok(())
    }

    fn capabilities(&self) -> CapabilitySet {
        CapabilitySet {
            // Conservative defaults; OpenRouter models vary widely. Real per-model
            // capability resolution is a v1 concern.
            max_context: 8192,
            supports_vision: false,
            supports_tools: true,
        }
    }

    async fn infer(
        &self,
        prompt: &str,
        temperature: Option<f32>,
        top_p: Option<f32>,
    ) -> Result<String> {
        let body = ChatRequest {
            model: &self.model,
            messages: vec![ChatMessage {
                role: "user",
                content: prompt,
            }],
            temperature,
            top_p,
            max_tokens: Some(512),
        };

        let resp = self
            .http
            .post(OPENROUTER_URL)
            .bearer_auth(&self.api_key)
            .header(
                "HTTP-Referer",
                "https://github.com/santoshkumarradha/ChiefOS",
            )
            .header("X-Title", "Chief OS integration test")
            .json(&body)
            .send()
            .await
            .context("openrouter POST failed")?;

        let status = resp.status();
        let text = resp.text().await.context("read openrouter response body")?;

        if !status.is_success() {
            return Err(anyhow!(
                "openrouter returned {}: {}",
                status,
                text.chars().take(500).collect::<String>()
            ));
        }

        let parsed: ChatResponse = serde_json::from_str(&text).with_context(|| {
            format!(
                "parse openrouter response: {}",
                &text[..text.len().min(500)]
            )
        })?;

        if let Some(err) = parsed.error {
            return Err(anyhow!("openrouter API error: {}", err.message));
        }

        let content = parsed
            .choices
            .first()
            .ok_or_else(|| anyhow!("openrouter returned no choices"))?
            .message
            .content
            .clone();

        Ok(content)
    }
}
