//! Surface tool handler.

use super::{require_grant, DispatchError, DispatchResult};
use crate::capability::{CapabilityKind, Grant, PrincipalId};
use chief_event_log_proto::EventLog;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::sync::broadcast;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SurfaceEvent {
    pub surface: String,
    pub region: String,
    pub update: Value,
}

pub(crate) fn pane(
    tx: &broadcast::Sender<SurfaceEvent>,
    args: &Value,
    principal: &PrincipalId,
    grants: &[Grant],
    handle_scope: Option<&CapabilityKind>,
    event_log: &EventLog,
) -> DispatchResult {
    let surface = args
        .get("surface")
        .and_then(Value::as_str)
        .unwrap_or("main")
        .to_string();
    let region = args
        .get("region")
        .and_then(|value| {
            value
                .as_str()
                .map(str::to_string)
                .or_else(|| value.as_u64().map(|region| region.to_string()))
        })
        .unwrap_or_else(|| "0".to_string());
    require_grant(
        event_log,
        principal,
        grants,
        handle_scope,
        "surface.pane",
        |capability| {
            matches!(capability, CapabilityKind::SurfacePane { surfaces, regions, .. }
            if allows(surfaces, &surface) && allows(regions, &region))
        },
    )?;

    let update = args.get("update").cloned().unwrap_or(Value::Null);
    let event = SurfaceEvent {
        surface,
        region,
        update,
    };
    let receivers = tx
        .send(event.clone())
        .map_err(|err| DispatchError::Store(format!("broadcast surface event: {err}")))?;
    Ok(json!({"status": "ok", "event": event, "receivers": receivers}))
}

fn allows(allowed: &[String], requested: &str) -> bool {
    allowed
        .iter()
        .any(|value| value == "*" || value.eq_ignore_ascii_case(requested))
}
