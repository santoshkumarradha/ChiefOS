//! HAX inbox notification tool handler.
//!
//! v0 stores `notify.inbox` items in an in-process `VecDeque`. Real integration
//! with `prototypes/hax-inbox/` is deferred to `t-h-inbox-wire`; this handler
//! still enforces priority scope and records dispatch attestations.

use super::{require_grant, DispatchResult};
use crate::capability::{CapabilityKind, Grant, PrincipalId};
use chief_event_log_proto::EventLog;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::Mutex;

pub type InboxStore = Arc<Mutex<VecDeque<InboxItem>>>;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InboxItem {
    pub priority: String,
    pub title: String,
    pub body: String,
}

pub(crate) async fn inbox(
    store: &InboxStore,
    args: &Value,
    principal: &PrincipalId,
    grants: &[Grant],
    handle_scope: Option<&CapabilityKind>,
    event_log: &EventLog,
) -> DispatchResult {
    let priority = args
        .get("priority")
        .and_then(Value::as_str)
        .unwrap_or("normal")
        .to_string();
    require_grant(
        event_log,
        principal,
        grants,
        handle_scope,
        "notify.inbox",
        |capability| matches!(capability, CapabilityKind::NotifyInbox { priorities, .. } if allows(priorities, &priority)),
    )?;

    let item = InboxItem {
        priority,
        title: args
            .get("title")
            .and_then(Value::as_str)
            .unwrap_or("Untitled")
            .to_string(),
        body: args
            .get("body")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string(),
    };
    let mut inbox = store.lock().await;
    inbox.push_back(item.clone());
    Ok(json!({"status": "ok", "queued": inbox.len(), "item": item}))
}

fn allows(allowed: &[String], requested: &str) -> bool {
    allowed
        .iter()
        .any(|value| value == "*" || value.eq_ignore_ascii_case(requested))
}
