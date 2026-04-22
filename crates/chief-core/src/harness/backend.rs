//! chief-core backend structs for SDK-facing AI and harness primitives.
//!
//! The SDK request/response DTOs are currently crate-private in `chief-sdk`, so this
//! module exposes equivalent chief-core methods without implementing those traits until
//! the SDK makes its backend DTOs public.

use super::attestation::{hash_json, KernelSigner};
use super::runtime::{HarnessError, HarnessOutcome, HarnessRequest, HarnessRuntime};
use crate::router::ModelRouter;
use chief_inference::ModelBackend;
use chief_sdk::{AiError, SignedAttestation, Tier};
use serde::de::DeserializeOwned;
use serde_json::{json, Value};
use std::sync::Arc;

#[derive(Clone)]
pub struct ChiefCoreHarnessBackend {
    runtime: Arc<HarnessRuntime>,
}

impl ChiefCoreHarnessBackend {
    pub fn new(runtime: Arc<HarnessRuntime>) -> Self {
        Self { runtime }
    }

    pub async fn run_session(&self, req: HarnessRequest) -> Result<HarnessOutcome, HarnessError> {
        self.runtime.run_session(req).await
    }
}

#[derive(Debug, Clone)]
pub struct CoreAiRequest {
    pub prompt: String,
    pub input: Value,
    pub schema_name: String,
    pub tier: Tier,
    pub max_tokens: u32,
}

#[derive(Debug, Clone)]
pub struct CoreAiResponse {
    pub output: Value,
    pub model: String,
    pub tokens_used: u32,
    pub attestation: SignedAttestation,
}

#[derive(Clone)]
pub struct ChiefCoreAiBackend {
    router: Arc<ModelRouter>,
    backend: Arc<dyn ModelBackend>,
    signer: KernelSigner,
}

impl ChiefCoreAiBackend {
    pub fn new(
        router: Arc<ModelRouter>,
        backend: Arc<dyn ModelBackend>,
        signer: KernelSigner,
    ) -> Self {
        Self {
            router,
            backend,
            signer,
        }
    }

    pub async fn call_one(&self, req: &CoreAiRequest) -> Result<CoreAiResponse, AiError> {
        let choice = self
            .router
            .pick(req.tier)
            .map_err(|_| AiError::TierUnavailable { tier: req.tier })?;
        let prompt = format!(
            "{}\n\nInput JSON:\n{}\n\nReturn only JSON for schema {}. Max tokens: {}.",
            req.prompt, req.input, req.schema_name, req.max_tokens
        );
        let raw = self
            .backend
            .infer(&prompt, Some(0.0), Some(1.0))
            .await
            .map_err(|err| AiError::Model(err.to_string()))?;
        let output: Value =
            serde_json::from_str(&raw).map_err(|err| AiError::Schema(err.to_string()))?;
        let attestation_payload = json!({
            "kind": "ai-call",
            "model": choice.model_id,
            "prompt_hash": hash_json(&prompt),
            "output_hash": hash_json(&output),
            "tier": req.tier,
        });
        Ok(CoreAiResponse {
            output,
            model: choice.model_id,
            tokens_used: raw.len() as u32,
            attestation: self.signer.sign_json(&attestation_payload),
        })
    }

    pub async fn call_typed<T: DeserializeOwned>(&self, req: &CoreAiRequest) -> Result<T, AiError> {
        let response = self.call_one(req).await?;
        serde_json::from_value(response.output).map_err(|err| AiError::Schema(err.to_string()))
    }
}
