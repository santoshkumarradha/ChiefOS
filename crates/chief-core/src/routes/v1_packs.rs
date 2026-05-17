//! Pack install preview routes.
//!
//! Phase 5 needs a real install gate before activating a pack. This route
//! parses an on-disk manifest, verifies the signature placeholder shape, and
//! returns the grants that would be shown to the user before activation.

use crate::state::AppState;
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::post,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new().route("/packs/install-preview", post(install_preview))
}

#[derive(Debug, Clone, Deserialize)]
pub struct InstallPreviewRequest {
    pub manifest_path: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallPreviewResponse {
    pub pack_id: String,
    pub version: String,
    pub description: String,
    pub signature_status: String,
    pub installable: bool,
    pub requested_grants: Vec<GrantPreview>,
    pub agents: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrantPreview {
    pub kind: String,
    pub scope: serde_json::Value,
    pub usage_reason: String,
}

#[derive(Debug, Deserialize)]
struct TomlManifest {
    name: String,
    version: String,
    description: String,
    signature: String,
    #[serde(default)]
    grants: Vec<TomlGrant>,
    #[serde(default)]
    agents: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct TomlGrant {
    kind: String,
    usage_reason: String,
    #[serde(flatten)]
    scope: toml::Table,
}

async fn install_preview(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<InstallPreviewRequest>,
) -> Response {
    match preview_manifest(req.manifest_path) {
        Ok(response) => Json(response).into_response(),
        Err(err) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "pack_manifest_invalid",
                "message": err.to_string(),
            })),
        )
            .into_response(),
    }
}

pub fn preview_manifest(path: PathBuf) -> anyhow::Result<InstallPreviewResponse> {
    let raw = fs::read_to_string(&path)?;
    let manifest: TomlManifest = toml::from_str(&raw)?;
    if !manifest.signature.starts_with("ed25519:") {
        anyhow::bail!("manifest signature must use ed25519 placeholder");
    }
    if manifest
        .grants
        .iter()
        .any(|grant| grant.usage_reason.trim().is_empty())
    {
        anyhow::bail!("all grants must include usage_reason");
    }

    Ok(InstallPreviewResponse {
        pack_id: manifest.name,
        version: manifest.version,
        description: manifest.description,
        signature_status: "placeholder_verified".into(),
        installable: true,
        requested_grants: manifest
            .grants
            .into_iter()
            .map(|grant| GrantPreview {
                kind: grant.kind,
                scope: serde_json::to_value(grant.scope).unwrap_or_else(|_| serde_json::json!({})),
                usage_reason: grant.usage_reason,
            })
            .collect(),
        agents: manifest.agents,
    })
}
