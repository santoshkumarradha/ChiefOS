//! Multi-turn tool-using LLM harness via `ctx.harness()` (ADR-0013).
#![allow(private_interfaces)]

use crate::ai_error::HarnessError;
use crate::tier::Tier;
use crate::tool_handle::{SessionId, ToolHandle};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;

/// Builder for a multi-turn harness session.
pub struct HarnessBuilder {
    backend: Arc<dyn HarnessBackend>,
    goal: Option<String>,
    tools: Vec<ToolHandle>,
    tier: Tier,
    max_turns: u32,
    max_cost_usd: f64,
    max_wall_secs: u32,
}

impl HarnessBuilder {
    /// Create a new harness builder (internal use; obtained from `ctx.harness()`).
    pub(crate) fn new(backend: Arc<dyn HarnessBackend>) -> Self {
        Self {
            backend,
            goal: None,
            tools: Vec::new(),
            tier: Tier::Deep,
            max_turns: 10,
            max_cost_usd: 1.0,
            max_wall_secs: 120,
        }
    }

    /// Set the harness goal.
    pub fn goal(mut self, g: impl Into<String>) -> Self {
        self.goal = Some(g.into());
        self
    }

    /// Set the tools available to the harness.
    pub fn tools(mut self, tools: &[ToolHandle]) -> Self {
        self.tools = tools.to_vec();
        self
    }

    /// Set the tier (default: Deep).
    pub fn tier(mut self, t: Tier) -> Self {
        self.tier = t;
        self
    }

    /// Set max turns (default: 10).
    pub fn max_turns(mut self, n: u32) -> Self {
        self.max_turns = n;
        self
    }

    /// Set max cost in USD (default: 1.0).
    pub fn max_cost_usd(mut self, usd: f64) -> Self {
        self.max_cost_usd = usd;
        self
    }

    /// Set max wall-clock seconds (default: 120).
    pub fn max_wall_secs(mut self, s: u32) -> Self {
        self.max_wall_secs = s;
        self
    }

    /// Run the harness and return the transcript.
    pub async fn run(self) -> Result<HarnessTranscript, HarnessError> {
        let req = HarnessRequest {
            goal: self.goal.unwrap_or_default(),
            tools: self.tools,
            tier: self.tier,
            max_turns: self.max_turns,
            max_cost_usd: self.max_cost_usd,
            max_wall_secs: self.max_wall_secs,
        };

        self.backend.run_session(&req).await
    }
}

/// Request to the harness backend.
#[derive(Clone, Debug)]
#[allow(dead_code)]
pub(crate) struct HarnessRequest {
    pub goal: String,
    pub tools: Vec<ToolHandle>,
    pub tier: Tier,
    pub max_turns: u32,
    pub max_cost_usd: f64,
    pub max_wall_secs: u32,
}

/// Trait that chief-core implements to run harness sessions.
#[async_trait]
pub trait HarnessBackend: Send + Sync {
    /// Run a harness session.
    async fn run_session(&self, req: &HarnessRequest) -> Result<HarnessTranscript, HarnessError>;
}

/// Complete transcript of a harness session.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HarnessTranscript {
    pub session_id: SessionId,
    pub turns: Vec<Turn>,
    pub final_output: Value,
    pub cost_usd: f64,
    pub wall_secs: f64,
    pub attestation: SignedAttestation,
    pub children: Vec<HarnessTranscript>,
}

/// A single turn in the harness session.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Turn {
    pub index: u32,
    pub role: Role,
    pub content: TurnContent,
    pub attestation: SignedAttestation,
    pub wall_ms: u32,
    pub cost_usd: f64,
}

/// Speaker role in a turn.
#[derive(Clone, Debug, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    System,
    User,
    Assistant,
    Tool,
}

/// Content of a turn.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum TurnContent {
    #[serde(rename = "text")]
    Text(String),
    #[serde(rename = "tool_call")]
    ToolCall { tool_name: String, arguments: Value },
    #[serde(rename = "tool_result")]
    ToolResult(Value),
}

/// Stub signed attestation for a turn or transcript.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SignedAttestation {
    pub signer: String,
    pub signature: Vec<u8>,
    pub ts: DateTime<Utc>,
}

/// Scope for a harness tool (for meta-prompting).
pub struct HarnessToolScope {
    pub max_turns: u32,
    pub tool_kinds: Vec<String>,
}

/// In-memory stub backend for testing.
pub struct InMemoryHarnessBackend;

#[async_trait]
impl HarnessBackend for InMemoryHarnessBackend {
    async fn run_session(&self, req: &HarnessRequest) -> Result<HarnessTranscript, HarnessError> {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let mut session_id = [0u8; 16];
        rng.fill(&mut session_id);

        let attestation = SignedAttestation {
            signer: "in-memory-stub".to_string(),
            signature: vec![0u8; 64],
            ts: Utc::now(),
        };

        Ok(HarnessTranscript {
            session_id: SessionId(session_id),
            turns: vec![
                Turn {
                    index: 0,
                    role: Role::User,
                    content: TurnContent::Text(req.goal.clone()),
                    attestation: attestation.clone(),
                    wall_ms: 100,
                    cost_usd: 0.001,
                },
                Turn {
                    index: 1,
                    role: Role::Assistant,
                    content: TurnContent::Text("Test response".to_string()),
                    attestation: attestation.clone(),
                    wall_ms: 200,
                    cost_usd: 0.002,
                },
            ],
            final_output: serde_json::json!({"result": "completed"}),
            cost_usd: 0.003,
            wall_secs: 0.3,
            attestation,
            children: Vec::new(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn harness_builder_basic() {
        let backend = Arc::new(InMemoryHarnessBackend);
        let result = HarnessBuilder::new(backend)
            .goal("test goal")
            .tier(Tier::Deep)
            .run()
            .await;
        assert!(result.is_ok());
        let tx = result.unwrap();
        assert!(!tx.turns.is_empty());
    }

    #[test]
    fn harness_builder_tier_default() {
        let backend = Arc::new(InMemoryHarnessBackend);
        let builder = HarnessBuilder::new(backend);
        assert_eq!(builder.tier, Tier::Deep);
    }

    #[test]
    fn role_serde() {
        assert_eq!(
            serde_json::to_string(&Role::Assistant).unwrap(),
            r#""assistant""#
        );
    }
}
