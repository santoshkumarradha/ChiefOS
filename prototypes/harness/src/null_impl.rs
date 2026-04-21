//! Null harness for testing.
//!
//! Records all calls in memory without executing them. Enables deterministic
//! testing of harness swappability by comparing call sequences.

use crate::{Action, Event, Harness, HarnessHandle, Receipt, TaskSpec};
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use futures::stream::{self};
use serde::{Deserialize, Serialize};
use std::pin::Pin;
use std::sync::Arc;
use std::sync::Mutex;

/// A single recorded harness operation.
#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub enum CallRecord {
    Start { handle: String, task_id: String },
    SubmitAction { handle: String, action_type: String },
    SubmitToolCall { handle: String, tool_id: String },
    StreamEvents { handle: String },
    Terminate { handle: String, reason: String },
}

/// Null harness that records all operations for testing.
pub struct NullHarness {
    /// All recorded operations, in order.
    pub call_log: Arc<Mutex<Vec<CallRecord>>>,
    /// Active harness handles.
    active_handles: Arc<Mutex<Vec<String>>>,
}

impl NullHarness {
    pub fn new() -> Self {
        Self {
            call_log: Arc::new(Mutex::new(Vec::new())),
            active_handles: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Retrieve the current call log (for testing/inspection).
    pub fn call_log(&self) -> Vec<CallRecord> {
        self.call_log.lock().unwrap().clone()
    }

    /// Clear the call log.
    pub fn clear_log(&self) {
        self.call_log.lock().unwrap().clear();
    }
}

impl Default for NullHarness {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Harness for NullHarness {
    async fn start(&self, task_spec: TaskSpec) -> Result<HarnessHandle> {
        let handle = HarnessHandle::new();
        self.call_log.lock().unwrap().push(CallRecord::Start {
            handle: handle.to_string(),
            task_id: task_spec.id.clone(),
        });
        self.active_handles.lock().unwrap().push(handle.to_string());
        Ok(handle)
    }

    async fn submit_action(&self, handle: &HarnessHandle, action: Action) -> Result<Receipt> {
        let handles = self.active_handles.lock().unwrap();
        if !handles.contains(&handle.to_string()) {
            return Err(anyhow!("Handle {} not found", handle));
        }
        drop(handles);

        self.call_log
            .lock()
            .unwrap()
            .push(CallRecord::SubmitAction {
                handle: handle.to_string(),
                action_type: action.action_type.clone(),
            });

        Ok(Receipt {
            id: uuid::Uuid::new_v4().to_string(),
            harness_handle: handle.to_string(),
            status: "accepted".to_string(),
            message: None,
        })
    }

    async fn submit_tool_call(
        &self,
        handle: &HarnessHandle,
        tool_id: &str,
        _args: serde_json::Value,
    ) -> Result<serde_json::Value> {
        let handles = self.active_handles.lock().unwrap();
        if !handles.contains(&handle.to_string()) {
            return Err(anyhow!("Handle {} not found", handle));
        }
        drop(handles);

        self.call_log
            .lock()
            .unwrap()
            .push(CallRecord::SubmitToolCall {
                handle: handle.to_string(),
                tool_id: tool_id.to_string(),
            });

        Ok(serde_json::json!({
            "tool_id": tool_id,
            "status": "success"
        }))
    }

    async fn stream_events(
        &self,
        handle: &HarnessHandle,
    ) -> Result<Pin<Box<dyn futures::Stream<Item = Event> + Send + 'async_trait>>> {
        let handles = self.active_handles.lock().unwrap();
        if !handles.contains(&handle.to_string()) {
            return Err(anyhow!("Handle {} not found", handle));
        }
        drop(handles);

        self.call_log
            .lock()
            .unwrap()
            .push(CallRecord::StreamEvents {
                handle: handle.to_string(),
            });

        let events = vec![Event::Started {
            handle: handle.to_string(),
        }];

        Ok(Box::pin(stream::iter(events)))
    }

    async fn terminate(&self, handle: HarnessHandle, reason: &str) -> Result<()> {
        let mut active = self.active_handles.lock().unwrap();
        if let Some(pos) = active.iter().position(|h| h == handle.as_str()) {
            active.remove(pos);
        } else {
            return Err(anyhow!("Handle {} not found", handle));
        }
        drop(active);

        self.call_log.lock().unwrap().push(CallRecord::Terminate {
            handle: handle.to_string(),
            reason: reason.to_string(),
        });

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_null_harness_basic_flow() {
        let harness = NullHarness::new();
        let spec = TaskSpec {
            id: "test-1".to_string(),
            name: "test".to_string(),
            description: None,
            input: serde_json::json!({}),
        };

        let handle = harness.start(spec).await.unwrap();
        let log = harness.call_log();
        assert_eq!(log.len(), 1);

        harness.terminate(handle, "test").await.unwrap();
        let log = harness.call_log();
        assert_eq!(log.len(), 2);
    }
}
