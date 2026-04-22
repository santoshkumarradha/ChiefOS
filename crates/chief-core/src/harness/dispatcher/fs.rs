//! Filesystem tool handlers.

use super::{get_string, optional_string, require_grant, DispatchError, DispatchResult};
use crate::capability::{CapabilityKind, Grant, PrincipalId};
use chief_event_log_proto::EventLog;
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

pub(crate) fn read(
    args: &Value,
    principal: &PrincipalId,
    grants: &[Grant],
    handle_scope: Option<&CapabilityKind>,
    event_log: &EventLog,
) -> DispatchResult {
    let path = PathBuf::from(get_string(args, "path")?);
    require_grant(
        event_log,
        principal,
        grants,
        handle_scope,
        "fs.read",
        |capability| matches!(capability, CapabilityKind::FsRead { paths, .. } if path_allowed(paths, &path)),
    )?;

    match fs::read_to_string(&path) {
        Ok(text) => Ok(json!({"status": "ok", "path": path, "text": text})),
        Err(text_err) => match fs::read(&path) {
            Ok(bytes) => Ok(json!({
                "status": "ok",
                "path": path,
                "bytes_hex": hex::encode(bytes),
            })),
            Err(_) => Err(DispatchError::Io(format!(
                "read {}: {text_err}",
                path.display()
            ))),
        },
    }
}

pub(crate) fn write(
    args: &Value,
    principal: &PrincipalId,
    grants: &[Grant],
    handle_scope: Option<&CapabilityKind>,
    event_log: &EventLog,
) -> DispatchResult {
    let path = PathBuf::from(get_string(args, "path")?);
    require_grant(
        event_log,
        principal,
        grants,
        handle_scope,
        "fs.write",
        |capability| matches!(capability, CapabilityKind::FsWrite { paths, .. } if path_allowed(paths, &path)),
    )?;

    let bytes = match optional_string(args, "bytes").or_else(|| optional_string(args, "text")) {
        Some(text) => text.as_bytes().to_vec(),
        None => {
            return Err(DispatchError::InvalidArguments(
                "missing string field `bytes` or `text`".to_string(),
            ))
        }
    };
    atomic_write(&path, &bytes)?;
    Ok(json!({"status": "ok", "path": path, "bytes": bytes.len()}))
}

pub(crate) fn watch(
    args: &Value,
    principal: &PrincipalId,
    grants: &[Grant],
    handle_scope: Option<&CapabilityKind>,
    event_log: &EventLog,
) -> DispatchResult {
    let path = PathBuf::from(get_string(args, "path")?);
    require_grant(
        event_log,
        principal,
        grants,
        handle_scope,
        "fs.watch",
        |capability| matches!(capability, CapabilityKind::FsWatch { paths, .. } if path_allowed(paths, &path)),
    )?;
    Ok(json!({
        "status": "deferred",
        "path": path,
        "deferred_to": "fs.watch notify-crate subscription is optional v0 scope",
    }))
}

fn atomic_write(path: &Path, bytes: &[u8]) -> Result<(), DispatchError> {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent)
        .map_err(|err| DispatchError::Io(format!("create {}: {err}", parent.display())))?;
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("chief-file");
    let tmp_path = parent.join(format!(".{file_name}.chief-tmp-{}", Uuid::new_v4()));
    fs::write(&tmp_path, bytes)
        .map_err(|err| DispatchError::Io(format!("write {}: {err}", tmp_path.display())))?;
    fs::rename(&tmp_path, path).map_err(|err| {
        let _ = fs::remove_file(&tmp_path);
        DispatchError::Io(format!(
            "rename {} to {}: {err}",
            tmp_path.display(),
            path.display()
        ))
    })
}

fn path_allowed(patterns: &[String], requested: &Path) -> bool {
    patterns.iter().any(|pattern| {
        pattern == "*"
            || path_matches(pattern, requested)
            || requested
                .canonicalize()
                .ok()
                .is_some_and(|canonical| path_matches(pattern, &canonical))
    })
}

fn path_matches(pattern: &str, requested: &Path) -> bool {
    let requested = normalize(requested);
    let pattern_path = PathBuf::from(pattern);
    let pattern_normalized = normalize(&pattern_path);

    if let Some(prefix) = pattern
        .strip_suffix("/**")
        .or_else(|| pattern.strip_suffix("/*"))
    {
        return requested.starts_with(normalize(Path::new(prefix)));
    }

    if pattern_normalized == requested {
        return true;
    }

    if pattern_path.is_dir() {
        requested.starts_with(pattern_normalized)
    } else {
        false
    }
}

fn normalize(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        normalized.push(component.as_os_str());
    }
    normalized
}
