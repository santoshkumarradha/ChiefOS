//! Engine wire-protocol trait and shared DTOs.

use async_trait::async_trait;
use chief_sdk::Tier;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EngineSessionHandle {
    pub session_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineSessionRequest {
    pub session_id: String,
    pub goal: String,
    pub tier: Tier,
    pub tools: Vec<EngineToolSpec>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EngineToolSpec {
    pub name: String,
    pub description: String,
    pub schema: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum EngineStep {
    AssistantText {
        content: String,
        is_final: bool,
    },
    ToolCall {
        call_id: String,
        tool_name: String,
        arguments: Value,
    },
    SessionEnded {
        final_output: Value,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToolResult {
    pub call_id: String,
    pub tool_name: String,
    pub result: Value,
}

#[derive(Debug, thiserror::Error)]
pub enum EngineError {
    #[error("http error: {0}")]
    Http(String),
    #[error("invalid engine response: {0}")]
    InvalidResponse(String),
    #[error("schema version mismatch: expected {expected}, got {actual}")]
    SchemaVersionMismatch { expected: String, actual: String },
    #[error("session not found: {0}")]
    SessionNotFound(String),
    #[error("subprocess error: {0}")]
    Subprocess(String),
    #[error("model error: {0}")]
    Model(String),
}

#[async_trait]
pub trait HarnessEngine: Send + Sync {
    async fn start_session(
        &self,
        req: EngineSessionRequest,
    ) -> Result<EngineSessionHandle, EngineError>;

    async fn next_step(&self, handle: &EngineSessionHandle) -> Result<EngineStep, EngineError>;

    async fn submit_tool_result(
        &self,
        handle: &EngineSessionHandle,
        result: ToolResult,
    ) -> Result<(), EngineError>;

    async fn close_session(&self, handle: EngineSessionHandle) -> Result<(), EngineError>;
}

#[derive(Clone, Default)]
pub struct EngineRegistry {
    engines: HashMap<String, Arc<dyn HarnessEngine>>,
    default_engine: String,
}

impl EngineRegistry {
    pub fn new(default_engine: impl Into<String>) -> Self {
        Self {
            engines: HashMap::new(),
            default_engine: default_engine.into(),
        }
    }

    pub fn register(mut self, name: impl Into<String>, engine: Arc<dyn HarnessEngine>) -> Self {
        self.engines.insert(name.into(), engine);
        self
    }

    pub fn get(&self, name: Option<&str>) -> Result<Arc<dyn HarnessEngine>, EngineError> {
        let selected = name.unwrap_or(&self.default_engine);
        self.engines
            .get(selected)
            .cloned()
            .ok_or_else(|| EngineError::InvalidResponse(format!("unknown engine: {selected}")))
    }
}
