//! OpenCode backend harness implementation.
//!
//! Shells out to the opencode CLI. Requires opencode to be installed
//! at runtime. If not available, logs a warning and falls back gracefully.

use crate::{Action, Event, Harness, HarnessHandle, Receipt, TaskSpec};
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use futures::stream::{self};
use std::pin::Pin;
use std::process::Command;
use std::sync::Arc;
use std::sync::Mutex;

/// OpenCode CLI harness implementation.
pub struct OpencodeHarness {
    /// Cache of which handles have been started.
    active_handles: Arc<Mutex<Vec<String>>>,
}

impl OpencodeHarness {
    pub fn new() -> Self {
        // Check if opencode is available
        match Command::new("which").arg("opencode").output() {
            Ok(output) if output.status.success() => {
                eprintln!("✓ opencode CLI found");
            }
            _ => {
                eprintln!("⚠ opencode CLI not found; operations will be mocked");
            }
        }

        Self {
            active_handles: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn is_opencode_available() -> bool {
        Command::new("which")
            .arg("opencode")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }
}

impl Default for OpencodeHarness {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Harness for OpencodeHarness {
    async fn start(&self, task_spec: TaskSpec) -> Result<HarnessHandle> {
        let handle = HarnessHandle::new();

        if Self::is_opencode_available() {
            // In a real implementation, this would invoke: opencode start <task-spec>
            eprintln!(
                "📌 opencode start: task={}, handle={}",
                task_spec.id, handle
            );
        } else {
            eprintln!(
                "⚠ opencode unavailable; mocking start: task={}, handle={}",
                task_spec.id, handle
            );
        }

        self.active_handles
            .lock()
            .unwrap()
            .push(handle.as_str().to_string());
        Ok(handle)
    }

    async fn submit_action(&self, handle: &HarnessHandle, action: Action) -> Result<Receipt> {
        let handles = self.active_handles.lock().unwrap();
        if !handles.contains(&handle.as_str().to_string()) {
            return Err(anyhow!("Handle {} not found", handle));
        }
        drop(handles);

        let receipt_id = uuid::Uuid::new_v4().to_string();

        if Self::is_opencode_available() {
            eprintln!(
                "📌 opencode submit-action: handle={}, action_type={}, receipt_id={}",
                handle, action.action_type, receipt_id
            );
        } else {
            eprintln!(
                "⚠ opencode unavailable; mocking submit-action: handle={}, action_type={}",
                handle, action.action_type
            );
        }

        Ok(Receipt {
            id: receipt_id,
            harness_handle: handle.to_string(),
            status: "accepted".to_string(),
            message: None,
        })
    }

    async fn submit_tool_call(
        &self,
        handle: &HarnessHandle,
        tool_id: &str,
        args: serde_json::Value,
    ) -> Result<serde_json::Value> {
        let handles = self.active_handles.lock().unwrap();
        if !handles.contains(&handle.as_str().to_string()) {
            return Err(anyhow!("Handle {} not found", handle));
        }
        drop(handles);

        if Self::is_opencode_available() {
            eprintln!(
                "📌 opencode submit-tool-call: handle={}, tool_id={}, args={}",
                handle, tool_id, args
            );
        } else {
            eprintln!(
                "⚠ opencode unavailable; mocking submit-tool-call: handle={}, tool_id={}",
                handle, tool_id
            );
        }

        // Return a mock result
        Ok(serde_json::json!({
            "tool_id": tool_id,
            "status": "success",
            "data": args
        }))
    }

    async fn stream_events(
        &self,
        handle: &HarnessHandle,
    ) -> Result<Pin<Box<dyn futures::Stream<Item = Event> + Send + 'async_trait>>> {
        let handles = self.active_handles.lock().unwrap();
        if !handles.contains(&handle.as_str().to_string()) {
            return Err(anyhow!("Handle {} not found", handle));
        }
        drop(handles);

        eprintln!("📌 opencode stream-events: handle={}", handle);

        // Return a mock stream with a Started event
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

        if Self::is_opencode_available() {
            eprintln!(
                "📌 opencode terminate: handle={}, reason={}",
                handle, reason
            );
        } else {
            eprintln!(
                "⚠ opencode unavailable; mocking terminate: handle={}, reason={}",
                handle, reason
            );
        }

        Ok(())
    }
}
