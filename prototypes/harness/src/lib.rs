//! Chief Harness Prototype — Swappable Harness Interface
//!
//! Validates the hypothesis that the harness interface (Agent Runtime's
//! backend) can be swapped without changing Chief Kernel core.
//!
//! This module defines the Harness trait and supporting types per
//! docs/03-chief-kernel.md (Service #1: Agent Runtime).

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::pin::Pin;
use uuid::Uuid;

pub mod null_impl;
pub mod opencode_impl;

pub use null_impl::NullHarness;
pub use opencode_impl::OpencodeHarness;

/// Unique identifier for a harness instance.
#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct HarnessHandle(String);

impl HarnessHandle {
    pub fn new() -> Self {
        Self(Uuid::new_v4().to_string())
    }

    pub fn from_string(s: String) -> Self {
        Self(s)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for HarnessHandle {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for HarnessHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Task specification for harness startup.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TaskSpec {
    /// Unique task identifier.
    pub id: String,
    /// Human-readable task name.
    pub name: String,
    /// Task description or goal.
    pub description: Option<String>,
    /// Initial input data.
    pub input: serde_json::Value,
}

/// Action submitted to the harness.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Action {
    /// Action type identifier.
    pub action_type: String,
    /// Action parameters.
    pub params: serde_json::Value,
}

/// Receipt confirming an action submission.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Receipt {
    /// Unique receipt ID.
    pub id: String,
    /// Associated harness handle.
    pub harness_handle: String,
    /// Status of the submitted action.
    pub status: String,
    /// Optional message or error details.
    pub message: Option<String>,
}

/// Event emitted by the harness.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Event {
    /// Harness started successfully.
    Started { handle: String },
    /// Action completed.
    ActionCompleted {
        action_id: String,
        result: serde_json::Value,
    },
    /// Tool call returned results.
    ToolCallCompleted {
        tool_id: String,
        result: serde_json::Value,
    },
    /// Harness encountered an error.
    Error { message: String },
    /// Harness terminated.
    Terminated { reason: String },
}

/// Configuration for building harnesses.
#[derive(Clone, Debug, Copy, Eq, PartialEq)]
pub enum HarnessKind {
    /// OpenCode CLI backend (v0 production target).
    Opencode,
    /// Null harness for testing; records all calls in memory.
    Null,
}

/// The core Harness trait — swappable backend for Agent Runtime.
///
/// Per docs/03-chief-kernel.md, the harness interface is the single
/// swap point. v0 uses opencode; v1 could use AgentField, Claude Agent SDK,
/// or custom backends without touching Chief Kernel.
#[async_trait]
pub trait Harness: Send + Sync {
    /// Start a new task and return a handle.
    async fn start(&self, task_spec: TaskSpec) -> Result<HarnessHandle>;

    /// Submit an action to a running harness.
    async fn submit_action(&self, handle: &HarnessHandle, action: Action) -> Result<Receipt>;

    /// Call a tool synchronously and get the result.
    async fn submit_tool_call(
        &self,
        handle: &HarnessHandle,
        tool_id: &str,
        args: serde_json::Value,
    ) -> Result<serde_json::Value>;

    /// Stream events from the harness.
    /// Returns a pinned boxed stream of events.
    async fn stream_events(
        &self,
        handle: &HarnessHandle,
    ) -> Result<Pin<Box<dyn futures::Stream<Item = Event> + Send + 'async_trait>>>;

    /// Terminate a harness with an optional reason.
    async fn terminate(&self, handle: HarnessHandle, reason: &str) -> Result<()>;
}

/// Factory function to build a harness by kind.
pub fn build_harness(kind: HarnessKind) -> Box<dyn Harness> {
    match kind {
        HarnessKind::Opencode => Box::new(OpencodeHarness::new()),
        HarnessKind::Null => Box::new(NullHarness::new()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_harness_handle_creation() {
        let h1 = HarnessHandle::new();
        let h2 = HarnessHandle::new();
        assert_ne!(h1, h2);
    }

    #[test]
    fn test_harness_handle_display() {
        let h = HarnessHandle::from_string("test-123".to_string());
        assert_eq!(h.as_str(), "test-123");
        assert_eq!(h.to_string(), "test-123");
    }

    #[test]
    fn test_task_spec_serialization() {
        let spec = TaskSpec {
            id: "task-1".to_string(),
            name: "test task".to_string(),
            description: Some("test description".to_string()),
            input: serde_json::json!({"key": "value"}),
        };
        let json = serde_json::to_string(&spec).unwrap();
        let deserialized: TaskSpec = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id, spec.id);
    }
}
