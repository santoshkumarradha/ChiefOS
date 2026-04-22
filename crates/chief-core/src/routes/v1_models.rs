//! `GET /v1/models` — report active model tiers and provider bindings.
//!
//! The demo shell uses this to draw the model-tier badge. There's no
//! persisted provider-binding table yet; we expose the one backend wired
//! into `AppState` via `inference_backend` plus a coarse "fast/deep" tier
//! classification derived from the `BackendKind`.

use crate::state::{AppState, BackendKind};
use axum::{extract::State, response::IntoResponse, routing::get, Json, Router};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new().route("/models", get(handler))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelTier {
    pub backend: String,
    pub model: String,
    pub path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelTiers {
    pub fast: Option<ModelTier>,
    pub deep: Option<ModelTier>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderBinding {
    pub provider: String,
    pub tier: String,
    pub active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelsResponse {
    pub tiers: ModelTiers,
    pub bindings: Vec<ProviderBinding>,
}

async fn handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let backend_label = match state.backend {
        BackendKind::Stub => "stub",
        BackendKind::Llama => "llama",
    };
    let model_path = state.model_path.as_ref().map(|p| p.display().to_string());

    // With the stub backend we have no real provider binding — return
    // `tiers.fast = None / deep = None` and `bindings = []` to be honest.
    // When a real backend is wired (llama with a model path), publish it as
    // the "fast" tier; "deep" remains unbound until the routing layer lands.
    let fast = match state.backend {
        BackendKind::Llama => Some(ModelTier {
            backend: backend_label.into(),
            model: model_path.clone().unwrap_or_else(|| "unknown".into()),
            path: model_path.clone(),
        }),
        BackendKind::Stub => None,
    };

    let deep: Option<ModelTier> = None;

    let bindings = match state.backend {
        BackendKind::Llama => vec![ProviderBinding {
            provider: "local-llama".into(),
            tier: "fast".into(),
            active: true,
        }],
        BackendKind::Stub => Vec::new(),
    };

    Json(ModelsResponse {
        tiers: ModelTiers { fast, deep },
        bindings,
    })
}
