//! Host-backed tool dispatcher for chief-core harness sessions.

use super::{HarnessError, ToolDispatcher, ToolInvocation, ToolOutput};
use crate::broker::CapabilityBroker;
use crate::capability::{CapabilityKind, Grant, PrincipalId};
use async_trait::async_trait;
use chief_event_log_proto::schema::Event;
use chief_event_log_proto::EventLog;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::{broadcast, Mutex, RwLock};
use tracing::warn;

pub mod fs;
pub mod mem;
pub mod net;
pub mod notify;
pub mod surface;

#[derive(Clone)]
pub struct HostToolDispatcher {
    broker: Arc<CapabilityBroker>,
    event_log: Arc<EventLog>,
    client: reqwest::Client,
    memory: mem::MemoryStore,
    surface_tx: broadcast::Sender<surface::SurfaceEvent>,
    inbox: notify::InboxStore,
}

impl HostToolDispatcher {
    pub fn new(broker: Arc<CapabilityBroker>, event_log: Arc<EventLog>) -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .expect("static reqwest client configuration is valid");
        Self::with_client(broker, event_log, client)
    }

    pub fn with_client(
        broker: Arc<CapabilityBroker>,
        event_log: Arc<EventLog>,
        client: reqwest::Client,
    ) -> Self {
        let (surface_tx, _) = broadcast::channel(128);
        Self {
            broker,
            event_log,
            client,
            memory: Arc::new(RwLock::new(HashMap::new())),
            surface_tx,
            inbox: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    pub fn subscribe_surface(&self) -> broadcast::Receiver<surface::SurfaceEvent> {
        self.surface_tx.subscribe()
    }

    pub async fn inbox_items(&self) -> Vec<notify::InboxItem> {
        self.inbox.lock().await.iter().cloned().collect()
    }
}

#[async_trait]
impl ToolDispatcher for HostToolDispatcher {
    async fn invoke(&self, invocation: ToolInvocation) -> Result<ToolOutput, HarnessError> {
        let result = self.invoke_value(&invocation).await;
        match result {
            Ok(value) => {
                append_tool_event(&self.event_log, &invocation, &value);
                Ok(ToolOutput { value })
            }
            Err(err @ DispatchError::CapabilityDenied(_)) => {
                let value = err.to_json();
                append_tool_event(&self.event_log, &invocation, &value);
                Err(HarnessError::CapabilityDenied(err.to_string()))
            }
            Err(err) => {
                let value = err.to_json();
                append_tool_event(&self.event_log, &invocation, &value);
                Ok(ToolOutput { value })
            }
        }
    }
}

impl HostToolDispatcher {
    async fn invoke_value(&self, invocation: &ToolInvocation) -> DispatchResult {
        let grants = self
            .broker
            .list(&invocation.principal)
            .await
            .map_err(|err| DispatchError::Store(err.to_string()))?;

        match invocation.tool_name.as_str() {
            "fs.read" => fs::read(
                &invocation.arguments,
                &invocation.principal,
                &grants,
                invocation.handle_scope.as_ref(),
                &self.event_log,
            ),
            "fs.write" => fs::write(
                &invocation.arguments,
                &invocation.principal,
                &grants,
                invocation.handle_scope.as_ref(),
                &self.event_log,
            ),
            "fs.watch" => fs::watch(
                &invocation.arguments,
                &invocation.principal,
                &grants,
                invocation.handle_scope.as_ref(),
                &self.event_log,
            ),
            "net.http" => {
                net::http(
                    &self.client,
                    &invocation.arguments,
                    &invocation.principal,
                    &grants,
                    invocation.handle_scope.as_ref(),
                    &self.event_log,
                )
                .await
            }
            "mem.read" => {
                mem::read(
                    &self.memory,
                    &invocation.arguments,
                    &invocation.principal,
                    &grants,
                    invocation.handle_scope.as_ref(),
                    &self.event_log,
                )
                .await
            }
            "mem.write" => {
                mem::write(
                    &self.memory,
                    &invocation.arguments,
                    &invocation.principal,
                    &grants,
                    invocation.handle_scope.as_ref(),
                    &self.event_log,
                )
                .await
            }
            "surface.pane" => surface::pane(
                &self.surface_tx,
                &invocation.arguments,
                &invocation.principal,
                &grants,
                invocation.handle_scope.as_ref(),
                &self.event_log,
            ),
            "notify.inbox" => {
                notify::inbox(
                    &self.inbox,
                    &invocation.arguments,
                    &invocation.principal,
                    &grants,
                    invocation.handle_scope.as_ref(),
                    &self.event_log,
                )
                .await
            }
            other => Err(DispatchError::Unsupported(format!(
                "unsupported tool {other}"
            ))),
        }
    }
}

pub(crate) type DispatchResult = Result<Value, DispatchError>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum DispatchError {
    CapabilityDenied(String),
    InvalidArguments(String),
    Io(String),
    Http(String),
    Store(String),
    Unsupported(String),
}

impl DispatchError {
    fn kind(&self) -> &'static str {
        match self {
            Self::CapabilityDenied(_) => "CapabilityDenied",
            Self::InvalidArguments(_) => "InvalidArguments",
            Self::Io(_) => "Io",
            Self::Http(_) => "Http",
            Self::Store(_) => "Store",
            Self::Unsupported(_) => "Unsupported",
        }
    }

    fn message(&self) -> &str {
        match self {
            Self::CapabilityDenied(message)
            | Self::InvalidArguments(message)
            | Self::Io(message)
            | Self::Http(message)
            | Self::Store(message)
            | Self::Unsupported(message) => message,
        }
    }

    fn to_json(&self) -> Value {
        json!({
            "status": "error",
            "error": self.kind(),
            "reason": self.message(),
        })
    }
}

impl std::fmt::Display for DispatchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.kind(), self.message())
    }
}

pub(crate) fn require_grant(
    event_log: &EventLog,
    principal: &PrincipalId,
    grants: &[Grant],
    handle_scope: Option<&CapabilityKind>,
    tool_name: &'static str,
    allowed: impl Fn(&CapabilityKind) -> bool,
) -> Result<(), DispatchError> {
    let handle_allows = match handle_scope {
        Some(scope) if scope.name() == tool_name && allowed(scope) => true,
        Some(scope) if scope.name() == tool_name => false,
        Some(_) => false,
        None => true,
    };

    let now = Utc::now();
    let grant_allows = grants.iter().any(|grant| {
        !grant.is_expired(now)
            && grant
                .capabilities
                .iter()
                .any(|capability| capability.name() == tool_name && allowed(capability))
    });
    let accepted = handle_allows && grant_allows;
    append_capability_check(event_log, principal, tool_name, accepted);

    if accepted {
        Ok(())
    } else if !handle_allows {
        Err(DispatchError::CapabilityDenied(format!(
            "{tool_name} handle scope exceeded"
        )))
    } else {
        Err(DispatchError::CapabilityDenied(format!(
            "{tool_name} grant missing or scope exceeded"
        )))
    }
}

pub(crate) fn get_string<'a>(args: &'a Value, field: &str) -> Result<&'a str, DispatchError> {
    args.get(field)
        .and_then(Value::as_str)
        .ok_or_else(|| DispatchError::InvalidArguments(format!("missing string field `{field}`")))
}

pub(crate) fn optional_string<'a>(args: &'a Value, field: &str) -> Option<&'a str> {
    args.get(field).and_then(Value::as_str)
}

fn append_tool_event(event_log: &EventLog, invocation: &ToolInvocation, result: &Value) {
    if let Err(err) = event_log.append(Event::ToolCall {
        agent: invocation.principal.to_string(),
        tool: invocation.tool_name.clone(),
        args_hash: *blake3::hash(&serde_json::to_vec(&invocation.arguments).unwrap_or_default())
            .as_bytes(),
        result_hash: *blake3::hash(&serde_json::to_vec(result).unwrap_or_default()).as_bytes(),
    }) {
        warn!(error = %err, "failed to append tool call event");
    }
}

fn append_capability_check(event_log: &EventLog, principal: &PrincipalId, op: &str, allowed: bool) {
    if let Err(err) = event_log.append(Event::CapabilityCheck {
        principal: principal.to_string(),
        op: op.to_string(),
        allowed,
    }) {
        warn!(error = %err, "failed to append dispatcher capability check");
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolStubStatus {
    pub status: String,
    pub deferred_to: String,
}
