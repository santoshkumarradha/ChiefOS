//! chief-core backend structs for SDK-facing AI and harness primitives.
//!
//! The SDK request/response DTOs are currently crate-private in `chief-sdk`, so this
//! module exposes equivalent chief-core methods without implementing those traits until
//! the SDK makes its backend DTOs public.
//!
//! ## Cloud provider dispatch (t-h-provider-wire)
//!
//! When a user binds a cloud provider via Controls → Models, the router resolves
//! `ModelChoice::Cloud` via the [`ProviderRegistry`]. `ChiefCoreAiBackend` then
//! fetches the API key from a [`SealedKeyStore`] (never from env; env only for
//! `#[ignore]`-gated live tests) and dispatches to the appropriate provider
//! adapter.
//!
//! v0 ships with:
//!   - OpenRouter — fully wired via `chief_inference::backends::OpenRouterBackend`
//!   - Anthropic  — stubbed; returns `AiError::Model("provider not yet implemented: anthropic")`
//!   - OpenAI     — stubbed; returns `AiError::Model("provider not yet implemented: openai")`
//!
//! The routing/wiring exists for all three; only the last-mile HTTP adapter is
//! missing for Anthropic / OpenAI. When they land, drop them in by replacing the
//! `NotYetImplemented` arm with a real client construction call.

use super::attestation::{hash_json, KernelSigner};
use super::runtime::{HarnessError, HarnessOutcome, HarnessRequest, HarnessRuntime};
use crate::router::{
    ModelRouter, ProviderKind, ResolvedProviderRoute, RouteError, SharedProviderRegistry,
    SharedSealedKeyStore,
};
use chief_inference::backends::OpenRouterBackend;
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

/// Default dispatch backend selection. When a tier resolves to a local
/// binding we use the fixed `default_local_backend` passed at construction.
/// When it resolves to a cloud binding we build an `OpenRouterBackend` (or
/// similar, once Anthropic/OpenAI adapters land) on the fly from the sealed
/// API key. The dispatch backend is never cached because the key handle and
/// model id are part of the per-call identity.
#[derive(Clone)]
pub struct ChiefCoreAiBackend {
    router: Arc<ModelRouter>,
    default_local_backend: Arc<dyn ModelBackend>,
    signer: KernelSigner,
    providers: Option<SharedProviderRegistry>,
    sealed: Option<SharedSealedKeyStore>,
}

impl ChiefCoreAiBackend {
    /// Construct without cloud provider wiring. Behavior matches legacy
    /// chief-core: all inference (including "cloud"-labeled tiers) goes
    /// through the `backend` handed in at construction.
    pub fn new(
        router: Arc<ModelRouter>,
        backend: Arc<dyn ModelBackend>,
        signer: KernelSigner,
    ) -> Self {
        Self {
            router,
            default_local_backend: backend,
            signer,
            providers: None,
            sealed: None,
        }
    }

    /// Attach the Controls → Models provider registry + sealed API-key store.
    /// After this is called, `Cloud`-tier picks dispatch through the registry
    /// instead of `default_local_backend`.
    pub fn with_providers(
        mut self,
        providers: SharedProviderRegistry,
        sealed: SharedSealedKeyStore,
    ) -> Self {
        self.providers = Some(providers);
        self.sealed = Some(sealed);
        self
    }

    pub async fn call_one(&self, req: &CoreAiRequest) -> Result<CoreAiResponse, AiError> {
        let prompt = format!(
            "{}\n\nInput JSON:\n{}\n\nReturn only JSON for schema {}. Max tokens: {}.",
            req.prompt, req.input, req.schema_name, req.max_tokens
        );

        // Resolution order:
        //  1. If a provider registry is attached AND the tier resolves to a cloud
        //     route, dispatch via the Models surface → sealed-key → provider adapter
        //     chain. This path bypasses the legacy pick() path that requires the
        //     (now-deprecated) ModelsConfig.providers map to be populated.
        //  2. Otherwise fall back to pick() + default_local_backend — matches
        //     the legacy (pre-provider-wire) behavior.
        let (dispatch_model_id, raw) = if self.providers.is_some() && self.sealed.is_some() {
            match self.router.resolve_cloud_route(req.tier) {
                Ok(_) => {
                    let (route, key) = self.resolve_cloud(req.tier)?;
                    let model_id = route.model.clone();
                    let raw = self.infer_cloud(&route, &key, &prompt).await?;
                    (model_id, raw)
                }
                Err(RouteError::NoCloudProvider) | Err(RouteError::Router(_)) => {
                    // Tier isn't bound cloud, or is locally bound. Use fallback.
                    self.call_local(&prompt, req.tier).await?
                }
                Err(e) => return Err(AiError::Model(e.to_string())),
            }
        } else {
            self.call_local(&prompt, req.tier).await?
        };

        let output: Value =
            serde_json::from_str(&raw).map_err(|err| AiError::Schema(err.to_string()))?;
        let attestation_payload = json!({
            "kind": "ai-call",
            "model": dispatch_model_id,
            "prompt_hash": hash_json(&prompt),
            "output_hash": hash_json(&output),
            "tier": req.tier,
        });
        Ok(CoreAiResponse {
            output,
            model: dispatch_model_id,
            tokens_used: raw.len() as u32,
            attestation: self.signer.sign_json(&attestation_payload),
        })
    }

    pub async fn call_typed<T: DeserializeOwned>(&self, req: &CoreAiRequest) -> Result<T, AiError> {
        let response = self.call_one(req).await?;
        serde_json::from_value(response.output).map_err(|err| AiError::Schema(err.to_string()))
    }

    /// Local-dispatch path: pick() + default_local_backend. Returns
    /// (model_id, raw_response).
    async fn call_local(&self, prompt: &str, tier: Tier) -> Result<(String, String), AiError> {
        let choice = self
            .router
            .pick(tier)
            .map_err(|_| AiError::TierUnavailable { tier })?;
        let raw = self
            .default_local_backend
            .infer(prompt, Some(0.0), Some(1.0))
            .await
            .map_err(|err| AiError::Model(err.to_string()))?;
        Ok((choice.model_id, raw))
    }

    /// Resolve a cloud route + sealed key pair for the given tier.
    ///
    /// Returns a clean `AiError` variant for every failure path — callers
    /// never see a raw `RouteError` and never see the API key.
    fn resolve_cloud(&self, tier: Tier) -> Result<(ResolvedProviderRoute, String), AiError> {
        let providers = self
            .providers
            .as_ref()
            .ok_or_else(|| AiError::Model(RouteError::NoCloudProvider.to_string()))?;
        let sealed = self.sealed.as_ref().ok_or_else(|| {
            AiError::Model("cloud provider registry set but no sealed key store wired".to_string())
        })?;

        // Update the router's view of providers so resolve_cloud_route works.
        // (ModelRouter was constructed with an optional registry; if it wasn't
        // attached there, we still have the registry in-hand here.)
        let route = match self.router.resolve_cloud_route(tier) {
            Ok(r) => r,
            Err(RouteError::NoCloudProvider) => providers
                .resolve_default_cloud()
                .map_err(|e| AiError::Model(RouteError::from(e).to_string()))?,
            Err(e) => return Err(AiError::Model(e.to_string())),
        };

        let key = sealed.get(&route.key_handle).ok_or_else(|| {
            AiError::Model(
                RouteError::ProviderKeyMissing {
                    provider: route.provider,
                }
                .to_string(),
            )
        })?;

        Ok((route, key))
    }

    /// Dispatch a completed prompt to the right provider adapter.
    ///
    /// OpenRouter is the only fully-wired path in v0. Anthropic / OpenAI
    /// return `AiError::Model("provider not yet implemented: ...")` — this
    /// proves the wiring is live and tells us exactly where to plug the
    /// next adapter in.
    async fn infer_cloud(
        &self,
        route: &ResolvedProviderRoute,
        api_key: &str,
        prompt: &str,
    ) -> Result<String, AiError> {
        match route.provider {
            ProviderKind::OpenRouter => {
                let backend = OpenRouterBackend::new(api_key, route.model.clone())
                    .map_err(|e| AiError::Model(format!("openrouter client: {e}")))?;
                backend
                    .infer(prompt, Some(0.0), Some(1.0))
                    .await
                    .map_err(|e| AiError::Model(format!("openrouter infer: {e}")))
            }
            ProviderKind::Anthropic => Err(AiError::Model(
                RouteError::NotYetImplemented {
                    provider: ProviderKind::Anthropic,
                }
                .to_string(),
            )),
            ProviderKind::OpenAI => Err(AiError::Model(
                RouteError::NotYetImplemented {
                    provider: ProviderKind::OpenAI,
                }
                .to_string(),
            )),
        }
    }
}
