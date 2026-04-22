//! HTTP adapter for an opencode server subprocess.
//!
//! This file is the chief-core -> opencode wire-protocol client described
//! in ADR-0014. It has three construction modes:
//!
//! - [`OpencodeEngine::connect`] — handshake against an already-running
//!   server at `base_url`. Used when an operator supervises opencode
//!   externally (e.g. via systemd units managed out-of-tree).
//! - [`OpencodeEngine::spawn_real`] — launch and supervise a real opencode
//!   subprocess per ADR-0014 (v0 default for production). Routes through
//!   the platform sandbox (`systemd-nspawn` on Linux, `darwin_stub`
//!   passthrough on macOS).
//! - [`OpencodeEngine::scripted_for_tests`] / [`OpencodeEngine::unchecked_for_tests`]
//!   — in-memory mock paths for unit tests. **Must stay green** — the
//!   broader harness_runtime test suite depends on them and does not
//!   need a real opencode binary.
//!
//! The existing "Mock vs Real" distinction is encoded via the
//! `scripted_steps` option rather than a hard enum split because it lets
//! the HTTP surface and the scripted surface share the same cheap
//! `HarnessEngine` dispatch without pattern-matching on every method.

use super::opencode_supervisor::{OpencodeSupervisor, SupervisorConfig};
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
    /// When `Some`, all HTTP dispatch is bypassed and the engine serves
    /// pre-baked steps from this queue. Only populated by
    /// `scripted_for_tests`.
    scripted_steps: Option<Arc<Mutex<VecDeque<EngineStep>>>>,
}

impl OpencodeEngine {
    /// Connect to an externally-managed opencode server.
    ///
    /// Performs a schema-version handshake but does NOT spawn a child.
    /// Use when opencode runs under some other supervisor and chief-core
    /// is only a client.
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

    /// Launch a real opencode subprocess and wait for it to be ready.
    ///
    /// This is the ADR-0014 v0 default for production. The returned
    /// engine owns the child; dropping it triggers a best-effort SIGKILL
    /// via [`OpencodeSupervisor::Drop`]. Prefer explicit
    /// [`Self::shutdown_supervisor`] so SIGTERM is issued first.
    ///
    /// On Linux, the subprocess runs under `systemd-nspawn`; on macOS it
    /// runs unsandboxed per the ADR-0002 v0 concession (a one-shot
    /// `tracing::warn!` is emitted by the sandbox layer).
    ///
    /// The schema-version handshake is deferred to the first real call —
    /// at spawn time we only verify `/health` responds, to keep the
    /// critical path tight. Callers that need an upfront schema check
    /// can call [`Self::handshake`] after spawn.
    pub async fn spawn_real(config: SupervisorConfig) -> Result<Self, EngineError> {
        let client = Client::new();
        let supervisor = OpencodeSupervisor::spawn_real(&config, &client).await?;
        Ok(Self {
            client,
            supervisor: Arc::new(Mutex::new(supervisor)),
            scripted_steps: None,
        })
    }

    /// Construct without any handshake or child — purely for tests that
    /// want to wire a client to a bespoke mock HTTP server (typically
    /// stood up via `httpmock` in the same test). Production code must
    /// NOT use this path.
    pub fn unchecked_for_tests(base_url: impl Into<String>) -> Self {
        Self {
            client: Client::new(),
            supervisor: Arc::new(Mutex::new(OpencodeSupervisor::connect(base_url))),
            scripted_steps: None,
        }
    }

    /// Construct with a scripted step queue. All HTTP methods are
    /// bypassed; `next_step` pops from the queue and all other methods
    /// are no-ops. This is the mock path that `harness_runtime.rs` tests
    /// consume, and it **must stay green** across refactors.
    pub fn scripted_for_tests(steps: Vec<EngineStep>) -> Self {
        Self {
            client: Client::new(),
            supervisor: Arc::new(Mutex::new(OpencodeSupervisor::connect(
                "http://127.0.0.1:0",
            ))),
            scripted_steps: Some(Arc::new(Mutex::new(steps.into()))),
        }
    }

    /// True when this engine is the scripted-mock variant. Useful for
    /// tests that want to assert they're wired to the mock path.
    pub fn is_mock(&self) -> bool {
        self.scripted_steps.is_some()
    }

    /// Schema-version handshake against the supervised server. Cheap;
    /// callable after [`Self::spawn_real`] if the caller wants to fail
    /// fast on protocol drift before issuing any session traffic.
    pub async fn handshake(&self) -> Result<(), EngineError> {
        if self.scripted_steps.is_some() {
            return Ok(());
        }
        let supervisor = self.supervisor.lock().await;
        supervisor.handshake(&self.client).await
    }

    /// Graceful supervisor shutdown (SIGTERM -> wait -> SIGKILL).
    /// No-op on mock / connect-only variants.
    pub async fn shutdown_supervisor(&self) -> Result<(), EngineError> {
        let mut supervisor = self.supervisor.lock().await;
        supervisor.shutdown().await
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn scripted_mock_still_serves_preseeded_steps() {
        // Guards against accidental breakage of the harness_runtime
        // test matrix by the spawn_real refactor.
        let engine = OpencodeEngine::scripted_for_tests(vec![
            EngineStep::AssistantText {
                content: "hello".into(),
                is_final: false,
            },
            EngineStep::SessionEnded {
                final_output: json!({"ok": true}),
            },
        ]);
        assert!(engine.is_mock());
        let handle = EngineSessionHandle {
            session_id: "s1".into(),
        };
        let s1 = engine.next_step(&handle).await.unwrap();
        assert!(matches!(s1, EngineStep::AssistantText { .. }));
        let s2 = engine.next_step(&handle).await.unwrap();
        assert!(matches!(s2, EngineStep::SessionEnded { .. }));
    }

    #[tokio::test]
    async fn shutdown_supervisor_on_mock_is_noop() {
        let engine = OpencodeEngine::scripted_for_tests(vec![]);
        engine
            .shutdown_supervisor()
            .await
            .expect("mock shutdown is a noop");
    }
}
