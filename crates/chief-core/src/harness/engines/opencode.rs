//! HTTP adapter for an opencode server subprocess.

use super::opencode_supervisor::OpencodeSupervisor;
use super::traits::{
    EngineError, EngineSessionHandle, EngineSessionRequest, EngineStep, HarnessEngine, ToolResult,
};
use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct OpencodeEngine {
    client: Client,
    supervisor: Arc<Mutex<OpencodeSupervisor>>,
    scripted_steps: Option<Arc<Mutex<VecDeque<EngineStep>>>>,
}

impl OpencodeEngine {
    pub async fn connect(base_url: impl Into<String>) -> Result<Self, EngineError> {
        let client = Client::new();
        let supervisor = OpencodeSupervisor::connect(base_url);
        supervisor.handshake(&client).await?;
        Ok(Self {
            client,
            supervisor: Arc::new(Mutex::new(supervisor)),
            scripted_steps: None,
        })
    }

    pub fn unchecked_for_tests(base_url: impl Into<String>) -> Self {
        Self {
            client: Client::new(),
            supervisor: Arc::new(Mutex::new(OpencodeSupervisor::connect(base_url))),
            scripted_steps: None,
        }
    }

    pub fn scripted_for_tests(steps: Vec<EngineStep>) -> Self {
        Self {
            client: Client::new(),
            supervisor: Arc::new(Mutex::new(OpencodeSupervisor::connect(
                "http://127.0.0.1:0",
            ))),
            scripted_steps: Some(Arc::new(Mutex::new(steps.into()))),
        }
    }
}

#[async_trait]
impl HarnessEngine for OpencodeEngine {
    async fn start_session(
        &self,
        req: EngineSessionRequest,
    ) -> Result<EngineSessionHandle, EngineError> {
        if self.scripted_steps.is_some() {
            return Ok(EngineSessionHandle {
                session_id: req.session_id,
            });
        }
        #[derive(Deserialize)]
        struct StartResponse {
            session_id: String,
        }
        let base_url = self.supervisor.lock().await.base_url().to_string();
        let response = self
            .client
            .post(format!("{base_url}/session/start"))
            .json(&req)
            .send()
            .await
            .map_err(|err| EngineError::Http(err.to_string()))?
            .error_for_status()
            .map_err(|err| EngineError::Http(err.to_string()))?
            .json::<StartResponse>()
            .await
            .map_err(|err| EngineError::InvalidResponse(err.to_string()))?;
        Ok(EngineSessionHandle {
            session_id: response.session_id,
        })
    }

    async fn next_step(&self, handle: &EngineSessionHandle) -> Result<EngineStep, EngineError> {
        if let Some(steps) = &self.scripted_steps {
            return Ok(steps
                .lock()
                .await
                .pop_front()
                .unwrap_or(EngineStep::SessionEnded {
                    final_output: serde_json::json!({"done": true}),
                }));
        }
        let base_url = self.supervisor.lock().await.base_url().to_string();
        self.client
            .get(format!("{base_url}/session/{}/next", handle.session_id))
            .send()
            .await
            .map_err(|err| EngineError::Http(err.to_string()))?
            .error_for_status()
            .map_err(|err| EngineError::Http(err.to_string()))?
            .json::<EngineStep>()
            .await
            .map_err(|err| EngineError::InvalidResponse(err.to_string()))
    }

    async fn submit_tool_result(
        &self,
        handle: &EngineSessionHandle,
        result: ToolResult,
    ) -> Result<(), EngineError> {
        if self.scripted_steps.is_some() {
            return Ok(());
        }
        let base_url = self.supervisor.lock().await.base_url().to_string();
        self.client
            .post(format!(
                "{base_url}/session/{}/tool-result",
                handle.session_id
            ))
            .json(&result)
            .send()
            .await
            .map_err(|err| EngineError::Http(err.to_string()))?
            .error_for_status()
            .map_err(|err| EngineError::Http(err.to_string()))?;
        Ok(())
    }

    async fn close_session(&self, handle: EngineSessionHandle) -> Result<(), EngineError> {
        if self.scripted_steps.is_some() {
            return Ok(());
        }
        let base_url = self.supervisor.lock().await.base_url().to_string();
        self.client
            .post(format!("{base_url}/session/{}/close", handle.session_id))
            .send()
            .await
            .map_err(|err| EngineError::Http(err.to_string()))?
            .error_for_status()
            .map_err(|err| EngineError::Http(err.to_string()))?;
        Ok(())
    }
}
