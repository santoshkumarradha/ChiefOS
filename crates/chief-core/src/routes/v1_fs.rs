//! `POST /v1/fs/*` — scoped filesystem operations for app POCs.
//!
//! This routes through existing filesystem capability kinds. It is deliberately
//! narrow: scan a folder, apply an approved move manifest, and rewind that
//! receipt. No delete operation is exposed in this POC.

use crate::{
    capability::RequestedOp,
    ceremony::CeremonyStatus,
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
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    fs,
    path::{Path, PathBuf},
    sync::Arc,
};
use tracing::warn;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/fs/scan", post(scan_handler))
        .route("/fs/apply", post(apply_handler))
        .route("/fs/rewind", post(rewind_handler))
}

#[derive(Debug, Clone, Deserialize)]
pub struct ScanRequest {
    pub root: String,
    #[serde(default = "default_max_entries")]
    pub max_entries: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct ScanResponse {
    pub root: String,
    pub entries: Vec<FsEntry>,
}

#[derive(Debug, Clone, Serialize)]
pub struct FsEntry {
    pub relative_path: String,
    pub kind: String,
    pub size_bytes: u64,
    pub extension: Option<String>,
    pub preview: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FsMoveOperation {
    pub from: String,
    pub to: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ApplyRequest {
    pub root: String,
    pub operations: Vec<FsMoveOperation>,
    pub ceremony_id: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ApplyResponse {
    pub applied: Vec<FsMoveOperation>,
    pub receipt: FsReceipt,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FsReceipt {
    pub root: String,
    pub operations: Vec<FsMoveOperation>,
    pub undo_operations: Vec<FsMoveOperation>,
    pub payload_hash: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RewindRequest {
    pub receipt: FsReceipt,
}

#[derive(Debug, Clone, Serialize)]
pub struct RewindResponse {
    pub rewound: Vec<FsMoveOperation>,
}

async fn scan_handler(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<ScanRequest>,
) -> Response {
    let principal = principal_for_request(&headers, &state);
    if let Err(denied) = state
        .broker
        .check(&principal, &RequestedOp::fs_read(req.root.clone()))
        .await
    {
        return capability_denied_response(denied);
    }

    match scan_folder(&req.root, req.max_entries) {
        Ok(response) => Json(response).into_response(),
        Err(err) => bad_fs_response("fs_scan_failed", err),
    }
}

async fn apply_handler(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<ApplyRequest>,
) -> Response {
    let principal = principal_for_request(&headers, &state);
    if let Err(denied) = state
        .broker
        .check(&principal, &RequestedOp::fs_write(req.root.clone()))
        .await
    {
        return capability_denied_response(denied);
    }

    let payload = manifest_payload(&req.root, &req.operations);
    let payload_hash = payload_hash(&payload);
    let Some(ceremony) = state.ceremonies.get(&req.ceremony_id).await else {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "error": "ceremony_not_found",
                "id": req.ceremony_id,
            })),
        )
            .into_response();
    };
    if ceremony.status != CeremonyStatus::Approved {
        return (
            StatusCode::CONFLICT,
            Json(serde_json::json!({
                "error": "ceremony_not_approved",
                "status": ceremony.status,
            })),
        )
            .into_response();
    }
    if ceremony.payload_hash.as_deref() != Some(payload_hash.as_str()) {
        return (
            StatusCode::CONFLICT,
            Json(serde_json::json!({
                "error": "payload_hash_mismatch",
                "expected": ceremony.payload_hash,
                "actual": payload_hash,
            })),
        )
            .into_response();
    }

    match apply_moves(&req.root, &req.operations) {
        Ok(applied) => {
            let undo_operations = applied
                .iter()
                .rev()
                .map(|op| FsMoveOperation {
                    from: op.to.clone(),
                    to: op.from.clone(),
                })
                .collect();
            Json(ApplyResponse {
                applied: applied.clone(),
                receipt: FsReceipt {
                    root: req.root,
                    operations: applied,
                    undo_operations,
                    payload_hash,
                },
            })
            .into_response()
        }
        Err(err) => bad_fs_response("fs_apply_failed", err),
    }
}

async fn rewind_handler(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<RewindRequest>,
) -> Response {
    let principal = principal_for_request(&headers, &state);
    if let Err(denied) = state
        .broker
        .check(&principal, &RequestedOp::fs_write(req.receipt.root.clone()))
        .await
    {
        return capability_denied_response(denied);
    }

    match apply_moves(&req.receipt.root, &req.receipt.undo_operations) {
        Ok(rewound) => Json(RewindResponse { rewound }).into_response(),
        Err(err) => bad_fs_response("fs_rewind_failed", err),
    }
}

fn scan_folder(root: &str, max_entries: usize) -> anyhow::Result<ScanResponse> {
    let root_path = canonical_root(root)?;
    let mut entries = Vec::new();
    scan_dir(&root_path, &root_path, max_entries.min(250), &mut entries)?;
    entries.sort_by(|a, b| a.relative_path.cmp(&b.relative_path));
    Ok(ScanResponse {
        root: root_path.display().to_string(),
        entries,
    })
}

fn scan_dir(
    root: &Path,
    dir: &Path,
    max_entries: usize,
    entries: &mut Vec<FsEntry>,
) -> anyhow::Result<()> {
    if entries.len() >= max_entries {
        return Ok(());
    }

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let metadata = entry.metadata()?;
        let relative_path = relative_string(root, &path)?;
        if metadata.is_dir() {
            scan_dir(root, &path, max_entries, entries)?;
            continue;
        }
        if !metadata.is_file() {
            continue;
        }
        entries.push(FsEntry {
            extension: path
                .extension()
                .and_then(|value| value.to_str())
                .map(|value| value.to_ascii_lowercase()),
            relative_path,
            kind: "file".into(),
            size_bytes: metadata.len(),
            preview: preview_file(&path),
        });
        if entries.len() >= max_entries {
            break;
        }
    }
    Ok(())
}

fn apply_moves(root: &str, operations: &[FsMoveOperation]) -> anyhow::Result<Vec<FsMoveOperation>> {
    let root_path = canonical_root(root)?;
    let mut applied = Vec::new();
    for op in operations {
        let from = safe_join(&root_path, &op.from)?;
        if !from.exists() {
            warn!(from = %from.display(), "skipping missing file operation source");
            continue;
        }
        let to = safe_join_create_parent(&root_path, &op.to)?;
        fs::rename(&from, &to)?;
        applied.push(op.clone());
    }
    Ok(applied)
}

fn canonical_root(root: &str) -> anyhow::Result<PathBuf> {
    let path = PathBuf::from(root);
    let canonical = path.canonicalize()?;
    if !canonical.is_dir() {
        anyhow::bail!("root is not a directory: {}", canonical.display());
    }
    Ok(canonical)
}

fn safe_join(root: &Path, relative: &str) -> anyhow::Result<PathBuf> {
    if relative.starts_with('/') || relative.contains("..") {
        anyhow::bail!("unsafe relative path: {relative}");
    }
    let path = root.join(relative);
    if !path.starts_with(root) {
        anyhow::bail!("path escaped root: {relative}");
    }
    Ok(path)
}

fn safe_join_create_parent(root: &Path, relative: &str) -> anyhow::Result<PathBuf> {
    let path = safe_join(root, relative)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    Ok(path)
}

fn relative_string(root: &Path, path: &Path) -> anyhow::Result<String> {
    Ok(path
        .strip_prefix(root)?
        .to_string_lossy()
        .replace(std::path::MAIN_SEPARATOR, "/"))
}

fn preview_file(path: &Path) -> Option<String> {
    let extension = path.extension().and_then(|value| value.to_str())?;
    if !matches!(extension, "txt" | "md" | "json" | "csv") {
        return None;
    }
    let bytes = fs::read(path).ok()?;
    let text = String::from_utf8_lossy(&bytes);
    Some(text.chars().take(280).collect())
}

fn manifest_payload(root: &str, operations: &[FsMoveOperation]) -> Value {
    serde_json::json!({
        "root": root,
        "operations": operations,
    })
}

fn payload_hash(value: &Value) -> String {
    let bytes = serde_json::to_vec(value).expect("payload json");
    format!("blake3:{}", blake3::hash(&bytes).to_hex())
}

fn bad_fs_response(error: &str, err: anyhow::Error) -> Response {
    (
        StatusCode::BAD_REQUEST,
        Json(serde_json::json!({
            "error": error,
            "details": err.to_string(),
        })),
    )
        .into_response()
}

fn default_max_entries() -> usize {
    100
}
