//! Minimal in-tree Rust engine that drives chief-inference directly.

use super::traits::{
    EngineError, EngineSessionHandle, EngineSessionRequest, EngineStep, HarnessEngine, ToolResult,
};
use crate::router::ModelRouter;
use async_trait::async_trait;
use chief_inference::ModelBackend;
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct CustomEngine {
    router: Arc<ModelRouter>,
    backend: Arc<dyn ModelBackend>,
    sessions: Arc<Mutex<HashMap<String, CustomSession>>>,
}

impl CustomEngine {
    pub fn new(router: Arc<ModelRouter>, backend: Arc<dyn ModelBackend>) -> Self {
        Self {
            router,
            backend,
            sessions: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

#[derive(Debug, Clone)]
struct CustomSession {
    req: EngineSessionRequest,
    transcript: Vec<Value>,
    invalid_json_retried: bool,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum ModelDirective {
    ToolCall {
        call_id: Option<String>,
        tool_name: String,
        arguments: Value,
    },
    Final {
        output: Value,
    },
}

#[async_trait]
impl HarnessEngine for CustomEngine {
    async fn start_session(
        &self,
        req: EngineSessionRequest,
    ) -> Result<EngineSessionHandle, EngineError> {
        let handle = EngineSessionHandle {
            session_id: req.session_id.clone(),
        };
        self.sessions.lock().await.insert(
            handle.session_id.clone(),
            CustomSession {
                req,
                transcript: Vec::new(),
                invalid_json_retried: false,
            },
        );
        Ok(handle)
    }

    async fn next_step(&self, handle: &EngineSessionHandle) -> Result<EngineStep, EngineError> {
        let (prompt, tier) = {
            let sessions = self.sessions.lock().await;
            let session = sessions
                .get(&handle.session_id)
                .ok_or_else(|| EngineError::SessionNotFound(handle.session_id.clone()))?;
            (build_prompt(session), session.req.tier)
        };

        let choice = self
            .router
            .pick(tier)
            .map_err(|err| EngineError::Model(err.to_string()))?;
        let output = self
            .backend
            .infer(&prompt, Some(0.0), Some(1.0))
            .await
            .map_err(|err| EngineError::Model(format!("{}: {err}", choice.model_id)))?;

        let directive = match serde_json::from_str::<ModelDirective>(&output) {
            Ok(value) => value,
            Err(first_err) => {
                let retry_prompt = format!(
                    "{prompt}\n\nYour previous response was invalid JSON: {first_err}. Return ONLY valid JSON."
                );
                let mut sessions = self.sessions.lock().await;
                let session = sessions
                    .get_mut(&handle.session_id)
                    .ok_or_else(|| EngineError::SessionNotFound(handle.session_id.clone()))?;
                if session.invalid_json_retried {
                    return Err(EngineError::InvalidResponse(first_err.to_string()));
                }
                session.invalid_json_retried = true;
                drop(sessions);
                let retry_output = self
                    .backend
                    .infer(&retry_prompt, Some(0.0), Some(1.0))
                    .await
                    .map_err(|err| EngineError::Model(err.to_string()))?;
                serde_json::from_str::<ModelDirective>(&retry_output)
                    .map_err(|err| EngineError::InvalidResponse(err.to_string()))?
            }
        };

        let mut sessions = self.sessions.lock().await;
        let session = sessions
            .get_mut(&handle.session_id)
            .ok_or_else(|| EngineError::SessionNotFound(handle.session_id.clone()))?;
        session.invalid_json_retried = false;

        match directive {
            ModelDirective::ToolCall {
                call_id,
                tool_name,
                arguments,
            } => {
                let call_id =
                    call_id.unwrap_or_else(|| format!("call_{}", session.transcript.len()));
                session.transcript.push(json!({
                    "role": "assistant",
                    "tool_call": {"call_id": call_id, "tool_name": tool_name, "arguments": arguments}
                }));
                Ok(EngineStep::ToolCall {
                    call_id,
                    tool_name,
                    arguments,
                })
            }
            ModelDirective::Final { output } => {
                session
                    .transcript
                    .push(json!({"role": "assistant", "final": output}));
                Ok(EngineStep::SessionEnded {
                    final_output: output,
                })
            }
        }
    }

    async fn submit_tool_result(
        &self,
        handle: &EngineSessionHandle,
        result: ToolResult,
    ) -> Result<(), EngineError> {
        let mut sessions = self.sessions.lock().await;
        let session = sessions
            .get_mut(&handle.session_id)
            .ok_or_else(|| EngineError::SessionNotFound(handle.session_id.clone()))?;
        session.transcript.push(json!({
            "role": "tool",
            "call_id": result.call_id,
            "tool_name": result.tool_name,
            "result": result.result
        }));
        Ok(())
    }

    async fn close_session(&self, handle: EngineSessionHandle) -> Result<(), EngineError> {
        self.sessions.lock().await.remove(&handle.session_id);
        Ok(())
    }
}

fn build_prompt(session: &CustomSession) -> String {
    let tools = serde_json::to_string(&session.req.tools).unwrap_or_else(|_| "[]".to_string());
    let transcript =
        serde_json::to_string(&session.transcript).unwrap_or_else(|_| "[]".to_string());
    format!(
        "You are Chief OS CustomEngine. Goal: {goal}\nAvailable tools: {tools}\nTranscript: {transcript}\nReturn ONLY one JSON object. For a tool call: {{\"type\":\"tool_call\",\"call_id\":\"...\",\"tool_name\":\"...\",\"arguments\":{{...}}}}. For final output: {{\"type\":\"final\",\"output\":{{...}}}}.",
        goal = session.req.goal
    )
}
