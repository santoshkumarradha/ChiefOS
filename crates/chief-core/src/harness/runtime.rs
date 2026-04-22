//! HarnessRuntime supervisor and chief-core session lifecycle.

use super::attestation::{hash_json, AttestationChain, AttestationVerifier, KernelSigner};
use super::budgets::{BudgetLimits, BudgetState};
use super::ceremony_gate::{is_external_action, CeremonyOutcome, CeremonySurface, DraftedAction};
use super::engines::{
    EngineRegistry, EngineSessionRequest, EngineStep, EngineToolSpec, ToolResult,
};
use super::meta_prompt::{child_result_json, evaluate_meta_prompt, MetaPromptDecision};
use crate::broker::CapabilityBroker;
use crate::capability::{PrincipalId, RequestedOp};
use async_trait::async_trait;
use chief_event_log_proto::schema::Event;
use chief_event_log_proto::EventLog;
use chief_sdk::tool_handle::SessionId;
use chief_sdk::{HarnessTranscript, Role, SignedAttestation, Tier, Turn, TurnContent};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TerminationReason {
    Completed,
    MaxTurns,
    MaxCost,
    MaxWall,
    CapabilityDenied,
    ExternalActionBlocked,
    UserInterrupt,
    EngineError,
    CeremonyDenied,
    CeremonyTimedOut,
}

#[derive(Debug, thiserror::Error)]
pub enum HarnessError {
    #[error("engine error: {0}")]
    Engine(String),
    #[error("tool error: {0}")]
    Tool(String),
}

#[derive(Debug, Clone)]
pub struct HarnessTool {
    pub name: String,
    pub description: String,
    pub schema: Value,
    pub requested_op: Option<RequestedOp>,
}

impl HarnessTool {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: String::new(),
            schema: json!({"type": "object"}),
            requested_op: None,
        }
    }

    pub fn with_op(mut self, requested_op: RequestedOp) -> Self {
        self.requested_op = Some(requested_op);
        self
    }
}

#[derive(Debug, Clone)]
pub struct HarnessRequest {
    pub principal: PrincipalId,
    pub goal: String,
    pub tools: Vec<HarnessTool>,
    pub tier: Tier,
    pub max_turns: u32,
    pub max_cost_usd: f64,
    pub max_wall_secs: u32,
    pub engine: Option<String>,
    pub depth: u8,
}

impl HarnessRequest {
    pub fn new(principal: PrincipalId, goal: impl Into<String>) -> Self {
        Self {
            principal,
            goal: goal.into(),
            tools: Vec::new(),
            tier: Tier::Deep,
            max_turns: 10,
            max_cost_usd: 1.0,
            max_wall_secs: 120,
            engine: None,
            depth: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HarnessOutcome {
    pub transcript: HarnessTranscript,
    pub termination: TerminationReason,
    pub metadata: Value,
}

#[derive(Debug, Clone)]
pub struct HarnessRuntimeConfig {
    pub cost_per_turn_usd: f64,
}

impl Default for HarnessRuntimeConfig {
    fn default() -> Self {
        Self {
            cost_per_turn_usd: 0.001,
        }
    }
}

pub struct HarnessRuntime {
    broker: Arc<CapabilityBroker>,
    event_log: Arc<EventLog>,
    engines: EngineRegistry,
    signer: KernelSigner,
    ceremony: Arc<dyn CeremonySurface>,
    dispatcher: Arc<dyn ToolDispatcher>,
    config: HarnessRuntimeConfig,
}

impl HarnessRuntime {
    pub fn new(
        broker: Arc<CapabilityBroker>,
        event_log: Arc<EventLog>,
        engines: EngineRegistry,
        signer: KernelSigner,
        ceremony: Arc<dyn CeremonySurface>,
        dispatcher: Arc<dyn ToolDispatcher>,
        config: HarnessRuntimeConfig,
    ) -> Self {
        Self {
            broker,
            event_log,
            engines,
            signer,
            ceremony,
            dispatcher,
            config,
        }
    }

    pub fn verifying_key(&self) -> ed25519_dalek::VerifyingKey {
        self.signer.verifying_key()
    }

    pub fn verify_turn_attestation(&self, turn: &Turn, session_id: &SessionId) -> bool {
        let payload = TurnAttestationPayload::from_turn(session_id, turn);
        AttestationVerifier::verify_json(&self.verifying_key(), &payload, &turn.attestation)
    }

    pub async fn run_session(&self, req: HarnessRequest) -> Result<HarnessOutcome, HarnessError> {
        let session_id = new_session_id();
        let session_hex = hex::encode(session_id.0);
        let mut budget = BudgetState::new(BudgetLimits::new(
            req.max_turns,
            req.max_cost_usd,
            req.max_wall_secs,
        ));
        let engine = self
            .engines
            .get(req.engine.as_deref())
            .map_err(|err| HarnessError::Engine(err.to_string()))?;
        let mut turns = Vec::new();
        let mut children = Vec::new();
        let mut chain = AttestationChain::default();
        let mut metadata = json!({"ceremonies": []});

        let _ = self.event_log.append(Event::AgentDecision {
            agent: req.principal.to_string(),
            question: "SessionBorn".to_string(),
            choice: session_hex.clone(),
            rationale_hash: *blake3::hash(req.goal.as_bytes()).as_bytes(),
        });

        let handle = engine
            .start_session(EngineSessionRequest {
                session_id: session_hex.clone(),
                goal: req.goal.clone(),
                tier: req.tier,
                tools: req
                    .tools
                    .iter()
                    .map(|tool| EngineToolSpec {
                        name: tool.name.clone(),
                        description: tool.description.clone(),
                        schema: tool.schema.clone(),
                    })
                    .collect(),
            })
            .await
            .map_err(|err| HarnessError::Engine(err.to_string()))?;

        let termination: TerminationReason;
        let mut final_output = Value::Null;

        loop {
            if let Some(reason) = budget.check() {
                termination = reason;
                break;
            }

            let step = match engine.next_step(&handle).await {
                Ok(step) => step,
                Err(err) => {
                    termination = TerminationReason::EngineError;
                    final_output = json!({"error": err.to_string()});
                    break;
                }
            };

            match step {
                EngineStep::AssistantText { content, is_final } => {
                    push_turn(
                        &mut turns,
                        &mut chain,
                        &self.signer,
                        session_id,
                        Role::Assistant,
                        TurnContent::Text(content.clone()),
                        self.config.cost_per_turn_usd,
                    );
                    if let Some(reason) = budget.record_turn(self.config.cost_per_turn_usd) {
                        termination = reason;
                        break;
                    }
                    if is_final {
                        termination = TerminationReason::Completed;
                        final_output = json!({"text": content});
                        break;
                    }
                }
                EngineStep::SessionEnded {
                    final_output: output,
                } => {
                    termination = TerminationReason::Completed;
                    final_output = output;
                    break;
                }
                EngineStep::ToolCall {
                    call_id,
                    tool_name,
                    arguments,
                } => {
                    push_turn(
                        &mut turns,
                        &mut chain,
                        &self.signer,
                        session_id,
                        Role::Assistant,
                        TurnContent::ToolCall {
                            tool_name: tool_name.clone(),
                            arguments: arguments.clone(),
                        },
                        self.config.cost_per_turn_usd,
                    );
                    if let Some(reason) = budget.record_turn(self.config.cost_per_turn_usd) {
                        termination = reason;
                        break;
                    }

                    let result_value = if tool_name == "meta.prompt" {
                        match evaluate_meta_prompt(req.depth, &arguments) {
                            MetaPromptDecision::Allowed { requested_depth } => {
                                let child =
                                    child_transcript(session_id, requested_depth, &self.signer);
                                let child_turns = child.turns.len() as u32;
                                let child_cost = child.cost_usd;
                                children.push(child);
                                if let Some(reason) = budget.absorb_child(child_turns, child_cost) {
                                    termination = reason;
                                    break;
                                }
                                child_result_json(requested_depth)
                            }
                            MetaPromptDecision::Denied { reason } => {
                                json!({"status": "error", "reason": reason})
                            }
                        }
                    } else if is_external_action(&tool_name) {
                        let action = DraftedAction {
                            session_id: session_hex.clone(),
                            call_id: call_id.clone(),
                            tool_name: tool_name.clone(),
                            arguments: arguments.clone(),
                        };
                        let _ = self.event_log.append(Event::AgentDecision {
                            agent: req.principal.to_string(),
                            question: "CeremonyPending".to_string(),
                            choice: call_id.clone(),
                            rationale_hash: *blake3::hash(
                                &serde_json::to_vec(&action).unwrap_or_default(),
                            )
                            .as_bytes(),
                        });
                        match self.ceremony.request_approval(action).await {
                            CeremonyOutcome::Approved => {
                                append_ceremony_metadata(&mut metadata, "approved", &call_id);
                                json!({"status": "approved"})
                            }
                            CeremonyOutcome::Denied { reason } => {
                                append_ceremony_metadata(&mut metadata, "denied", &call_id);
                                let result = ToolResult {
                                    call_id: call_id.clone(),
                                    tool_name: tool_name.clone(),
                                    result: json!({"status": "denied", "reason": reason}),
                                };
                                let _ = engine.submit_tool_result(&handle, result.clone()).await;
                                push_tool_result_turn(
                                    &mut turns,
                                    &mut chain,
                                    &self.signer,
                                    session_id,
                                    &result,
                                    self.config.cost_per_turn_usd,
                                );
                                termination = TerminationReason::CeremonyDenied;
                                final_output = json!({"status": "denied"});
                                break;
                            }
                            CeremonyOutcome::TimedOut => {
                                append_ceremony_metadata(&mut metadata, "timed_out", &call_id);
                                let result = ToolResult {
                                    call_id: call_id.clone(),
                                    tool_name: tool_name.clone(),
                                    result: json!({"status": "timed_out"}),
                                };
                                let _ = engine.submit_tool_result(&handle, result.clone()).await;
                                push_tool_result_turn(
                                    &mut turns,
                                    &mut chain,
                                    &self.signer,
                                    session_id,
                                    &result,
                                    self.config.cost_per_turn_usd,
                                );
                                termination = TerminationReason::CeremonyTimedOut;
                                final_output = json!({"status": "timed_out"});
                                break;
                            }
                        }
                    } else {
                        let Some(tool) = req.tools.iter().find(|tool| tool.name == tool_name)
                        else {
                            termination = TerminationReason::CapabilityDenied;
                            final_output = json!({"error": "tool not authorized for this session"});
                            break;
                        };

                        if let Some(op) = &tool.requested_op {
                            if let Err(denied) = self.broker.check(&req.principal, op).await {
                                let result = ToolResult {
                                    call_id: call_id.clone(),
                                    tool_name: tool_name.clone(),
                                    result: json!({
                                        "error": "CapabilityDenied",
                                        "reason": denied.to_string()
                                    }),
                                };
                                let _ = engine.submit_tool_result(&handle, result.clone()).await;
                                push_tool_result_turn(
                                    &mut turns,
                                    &mut chain,
                                    &self.signer,
                                    session_id,
                                    &result,
                                    self.config.cost_per_turn_usd,
                                );
                                termination = TerminationReason::CapabilityDenied;
                                final_output = result.result;
                                break;
                            }
                        }

                        let invocation = ToolInvocation {
                            call_id: call_id.clone(),
                            tool_name: tool_name.clone(),
                            arguments: arguments.clone(),
                        };
                        self.dispatcher
                            .invoke(invocation)
                            .await
                            .map_err(|err| HarnessError::Tool(err.to_string()))?
                            .value
                    };

                    let result = ToolResult {
                        call_id,
                        tool_name,
                        result: result_value,
                    };
                    engine
                        .submit_tool_result(&handle, result.clone())
                        .await
                        .map_err(|err| HarnessError::Engine(err.to_string()))?;
                    push_tool_result_turn(
                        &mut turns,
                        &mut chain,
                        &self.signer,
                        session_id,
                        &result,
                        self.config.cost_per_turn_usd,
                    );
                    if let Some(reason) = budget.record_turn(self.config.cost_per_turn_usd) {
                        termination = reason;
                        break;
                    }
                }
            }
        }

        let _ = engine.close_session(handle).await;
        let root = chain.root();
        let final_payload = json!({
            "kind": "harness-transcript",
            "session_id": session_hex,
            "termination": termination,
            "root": hex::encode(root),
            "turns": turns.len(),
            "final_output_hash": hash_json(&final_output),
        });
        let attestation = self.signer.sign_json(&final_payload);
        let _ = self.event_log.append(Event::AgentDecision {
            agent: req.principal.to_string(),
            question: "SessionClose".to_string(),
            choice: format!("{:?}", termination),
            rationale_hash: root,
        });

        Ok(HarnessOutcome {
            transcript: HarnessTranscript {
                session_id,
                turns,
                final_output,
                cost_usd: budget.cost_usd(),
                wall_secs: budget.wall_secs(),
                attestation,
                children,
            },
            termination,
            metadata,
        })
    }
}

#[derive(Debug, Clone)]
pub struct HarnessSession {
    pub session_id: SessionId,
    pub depth: u8,
}

#[derive(Debug, Clone)]
pub struct ToolInvocation {
    pub call_id: String,
    pub tool_name: String,
    pub arguments: Value,
}

#[derive(Debug, Clone)]
pub struct ToolOutput {
    pub value: Value,
}

#[async_trait]
pub trait ToolDispatcher: Send + Sync {
    async fn invoke(&self, invocation: ToolInvocation) -> Result<ToolOutput, HarnessError>;
}

#[derive(Default)]
pub struct EchoToolDispatcher;

#[async_trait]
impl ToolDispatcher for EchoToolDispatcher {
    async fn invoke(&self, invocation: ToolInvocation) -> Result<ToolOutput, HarnessError> {
        Ok(ToolOutput {
            value: json!({
                "status": "ok",
                "tool": invocation.tool_name,
                "arguments": invocation.arguments
            }),
        })
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct TurnAttestationPayload {
    pub kind: &'static str,
    pub session_id: String,
    pub turn_index: u32,
    pub role: Role,
    pub content_hash: String,
    pub cost_usd: f64,
}

impl TurnAttestationPayload {
    pub fn from_turn(session_id: &SessionId, turn: &Turn) -> Self {
        Self {
            kind: "harness-turn",
            session_id: hex::encode(session_id.0),
            turn_index: turn.index,
            role: turn.role,
            content_hash: hash_json(&turn.content),
            cost_usd: turn.cost_usd,
        }
    }
}

fn push_tool_result_turn(
    turns: &mut Vec<Turn>,
    chain: &mut AttestationChain,
    signer: &KernelSigner,
    session_id: SessionId,
    result: &ToolResult,
    cost_usd: f64,
) {
    push_turn(
        turns,
        chain,
        signer,
        session_id,
        Role::Tool,
        TurnContent::ToolResult(result.result.clone()),
        cost_usd,
    );
}

fn push_turn(
    turns: &mut Vec<Turn>,
    chain: &mut AttestationChain,
    signer: &KernelSigner,
    session_id: SessionId,
    role: Role,
    content: TurnContent,
    cost_usd: f64,
) {
    let index = turns.len() as u32;
    let payload = TurnAttestationPayload {
        kind: "harness-turn",
        session_id: hex::encode(session_id.0),
        turn_index: index,
        role,
        content_hash: hash_json(&content),
        cost_usd,
    };
    let attestation: SignedAttestation = signer.sign_json(&payload);
    chain.push(&payload);
    turns.push(Turn {
        index,
        role,
        content,
        attestation,
        wall_ms: 0,
        cost_usd,
    });
}

fn child_transcript(parent_id: SessionId, depth: u8, signer: &KernelSigner) -> HarnessTranscript {
    let mut child_id = parent_id.0;
    child_id[15] = child_id[15].wrapping_add(depth);
    let session_id = SessionId(child_id);
    let mut chain = AttestationChain::default();
    let mut turns = Vec::new();
    push_turn(
        &mut turns,
        &mut chain,
        signer,
        session_id,
        Role::Assistant,
        TurnContent::Text(format!("meta.prompt child depth {depth}")),
        0.001,
    );
    let final_output = json!({"status": "completed", "depth": depth});
    let root = chain.root();
    let attestation = signer.sign_json(&json!({
        "kind": "harness-transcript",
        "session_id": hex::encode(session_id.0),
        "root": hex::encode(root),
        "final_output_hash": hash_json(&final_output),
    }));
    HarnessTranscript {
        session_id,
        turns,
        final_output,
        cost_usd: 0.001,
        wall_secs: 0.0,
        attestation,
        children: Vec::new(),
    }
}

fn append_ceremony_metadata(metadata: &mut Value, status: &str, call_id: &str) {
    if let Some(ceremonies) = metadata.get_mut("ceremonies").and_then(Value::as_array_mut) {
        ceremonies.push(json!({"status": status, "call_id": call_id}));
    }
}

fn new_session_id() -> SessionId {
    SessionId(*Uuid::new_v4().as_bytes())
}
