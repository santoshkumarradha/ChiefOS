//! `POST /v1/ai/generate` — Chief-mediated inference for OS apps.
//!
//! This is not a new orchestration primitive. It exposes the existing
//! `llm.generate` capability through the broker, runs the configured model
//! backend, and signs the prompt/output pair with the device attestor.

use crate::{
    capability::RequestedOp,
    routes::legacy::{capability_denied_response, principal_for_request},
    state::AppState,
};
use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::post,
    Json, Router,
};
use chief_inference::{
    backends::{ModelBackend, OpenRouterBackend},
    canon::{CanonicalOutput, CanonicalPrompt},
    InferenceAttestation, Tier as InferenceTier,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use tracing::warn;

const DEFAULT_OPENROUTER_MODEL: &str = "openai/gpt-4o-mini";

pub fn routes() -> Router<Arc<AppState>> {
    Router::new().route("/ai/generate", post(handler))
}

#[derive(Debug, Clone, Deserialize)]
pub struct GenerateRequest {
    pub prompt: String,
    #[serde(default)]
    pub input: Value,
    #[serde(default)]
    pub schema_name: Option<String>,
    #[serde(default)]
    pub tier: Option<String>,
    #[serde(default)]
    pub max_tokens: Option<u32>,
    #[serde(default)]
    pub temperature: Option<f32>,
    #[serde(default)]
    pub top_p: Option<f32>,
}

#[derive(Debug, Clone, Serialize)]
pub struct GenerateResponse {
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub json: Option<Value>,
    pub provider: String,
    pub model: String,
    pub tier: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    pub attestation: InferenceAttestation,
}

async fn handler(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<GenerateRequest>,
) -> Response {
    if req.prompt.trim().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "empty_prompt",
            })),
        )
            .into_response();
    }

    let (provider, model, backend): (String, String, Arc<dyn ModelBackend>) =
        match openrouter_backend_from_env() {
            Ok(Some((provider, model, backend))) => (provider, model, backend),
            Ok(None) => {
                let id = state.inference_backend.id().0;
                (
                    "local".into(),
                    id.clone(),
                    Arc::clone(&state.inference_backend),
                )
            }
            Err(err) => {
                warn!(error = %err, "OpenRouter backend configuration failed");
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({
                        "error": "llm_backend_configuration_failed",
                    })),
                )
                    .into_response();
            }
        };

    let principal = principal_for_request(&headers, &state);
    if let Err(denied) = state
        .broker
        .check(&principal, &RequestedOp::llm_generate(provider.clone()))
        .await
    {
        return capability_denied_response(denied);
    }

    let prompt = render_prompt(&req);
    let raw = match backend.infer(&prompt, req.temperature, req.top_p).await {
        Ok(raw) => raw,
        Err(err) => {
            warn!(error = %err, model = %model, "LLM generation failed");
            return (
                StatusCode::BAD_GATEWAY,
                Json(serde_json::json!({
                    "error": "llm_generation_failed",
                    "provider": provider,
                    "model": model,
                })),
            )
                .into_response();
        }
    };

    let attestation = match state.attestor.attest(
        model.clone(),
        &CanonicalPrompt::new(&prompt, req.temperature, req.top_p, None),
        &CanonicalOutput::new(&raw),
        None,
        InferenceTier::Generated,
    ) {
        Ok(attestation) => attestation,
        Err(err) => {
            warn!(error = %err, "LLM attestation failed");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "llm_attestation_failed",
                })),
            )
                .into_response();
        }
    };

    Json(GenerateResponse {
        text: raw.clone(),
        json: extract_json(&raw),
        provider,
        model,
        tier: req.tier.unwrap_or_else(|| "fast".into()),
        max_tokens: req.max_tokens,
        attestation,
    })
    .into_response()
}

fn openrouter_backend_from_env() -> anyhow::Result<Option<(String, String, Arc<dyn ModelBackend>)>>
{
    if std::env::var("CHIEF_AI_DISABLE_OPENROUTER").ok().as_deref() == Some("1") {
        return Ok(None);
    }

    if std::env::var("OPENROUTER_API_KEY").ok().is_none() {
        return Ok(None);
    }

    let model = std::env::var("OPENROUTER_MODEL")
        .or_else(|_| std::env::var("CHIEF_OPENROUTER_MODEL"))
        .unwrap_or_else(|_| DEFAULT_OPENROUTER_MODEL.to_string());
    let backend = Arc::new(OpenRouterBackend::from_env(model.clone())?);
    Ok(Some((
        "openrouter".into(),
        format!("openrouter:{model}"),
        backend,
    )))
}

fn render_prompt(req: &GenerateRequest) -> String {
    let mut prompt = req.prompt.trim().to_string();
    if let Some(schema_name) = req.schema_name.as_deref() {
        prompt.push_str("\n\nOutput schema name: ");
        prompt.push_str(schema_name);
    }
    if !req.input.is_null() {
        prompt.push_str("\n\nInput JSON:\n");
        prompt.push_str(&serde_json::to_string_pretty(&req.input).unwrap_or_else(|_| "{}".into()));
    }
    prompt
}

fn extract_json(text: &str) -> Option<Value> {
    serde_json::from_str(text).ok().or_else(|| {
        let start = text.find('{')?;
        let end = text.rfind('}')?;
        if start >= end {
            return None;
        }
        serde_json::from_str(&text[start..=end]).ok()
    })
}
