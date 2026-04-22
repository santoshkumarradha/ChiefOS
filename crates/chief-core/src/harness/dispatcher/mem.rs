//! In-process Memory Graph tool handler.
//!
//! v0 intentionally keeps `mem.read` / `mem.write` in this process-local
//! `HashMap`. Wiring to the real `chief-mem` substrate is deferred to
//! `t-h-mem-wire`; this module still enforces capability namespaces before any
//! read or write touches the map.

use super::{get_string, require_grant, DispatchError, DispatchResult};
use crate::capability::{CapabilityKind, Grant, PrincipalId};
use chief_event_log_proto::EventLog;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub type MemoryStore = Arc<RwLock<HashMap<String, Value>>>;

pub(crate) async fn read(
    store: &MemoryStore,
    args: &Value,
    principal: &PrincipalId,
    grants: &[Grant],
    handle_scope: Option<&CapabilityKind>,
    event_log: &EventLog,
) -> DispatchResult {
    let key = get_string(args, "key")?;
    let namespace = namespace_for(key, args)?;
    require_grant(
        event_log,
        principal,
        grants,
        handle_scope,
        "mem.read",
        |capability| matches!(capability, CapabilityKind::MemRead { types, .. } if allows_namespace(types, &namespace)),
    )?;

    let value = store.read().await.get(key).cloned();
    Ok(json!({"status": "ok", "key": key, "value": value}))
}

pub(crate) async fn write(
    store: &MemoryStore,
    args: &Value,
    principal: &PrincipalId,
    grants: &[Grant],
    handle_scope: Option<&CapabilityKind>,
    event_log: &EventLog,
) -> DispatchResult {
    let key = args
        .get("key")
        .and_then(Value::as_str)
        .or_else(|| args.get("type").and_then(Value::as_str))
        .ok_or_else(|| DispatchError::InvalidArguments("missing string field `key`".to_string()))?;
    let namespace = namespace_for(key, args)?;
    require_grant(
        event_log,
        principal,
        grants,
        handle_scope,
        "mem.write",
        |capability| matches!(capability, CapabilityKind::MemWrite { types, .. } if allows_namespace(types, &namespace)),
    )?;

    let value = args.get("value").cloned().unwrap_or(Value::Null);
    store.write().await.insert(key.to_string(), value.clone());
    Ok(json!({"status": "ok", "key": key, "value": value}))
}

fn namespace_for(key: &str, args: &Value) -> Result<String, DispatchError> {
    if let Some(namespace) = args.get("namespace").and_then(Value::as_str) {
        return Ok(namespace.to_string());
    }
    if let Some(namespace) = args.get("type").and_then(Value::as_str) {
        return Ok(namespace.to_string());
    }
    key.split_once('/')
        .or_else(|| key.split_once(':'))
        .map(|(namespace, _)| namespace.to_string())
        .ok_or_else(|| {
            DispatchError::InvalidArguments(
                "memory key must include namespace as `namespace/key` or provide `namespace`"
                    .to_string(),
            )
        })
}

fn allows_namespace(types: &[String], namespace: &str) -> bool {
    types
        .iter()
        .any(|allowed| allowed == "*" || allowed.eq_ignore_ascii_case(namespace))
}
