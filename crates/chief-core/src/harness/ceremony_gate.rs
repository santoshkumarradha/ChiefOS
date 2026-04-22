//! Mockable external-action Ceremony gate.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CeremonyOutcome {
    Approved,
    Denied { reason: String },
    TimedOut,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DraftedAction {
    pub session_id: String,
    pub call_id: String,
    pub tool_name: String,
    pub arguments: Value,
}

#[async_trait]
pub trait CeremonySurface: Send + Sync {
    async fn request_approval(&self, action: DraftedAction) -> CeremonyOutcome;
}

#[derive(Clone, Default)]
pub struct MockCeremonySurface {
    outcomes: Arc<Mutex<VecDeque<CeremonyOutcome>>>,
    pending: Arc<Mutex<u32>>,
    max_pending: Arc<Mutex<u32>>,
}

impl MockCeremonySurface {
    pub fn new(outcomes: impl IntoIterator<Item = CeremonyOutcome>) -> Self {
        Self {
            outcomes: Arc::new(Mutex::new(outcomes.into_iter().collect())),
            pending: Arc::new(Mutex::new(0)),
            max_pending: Arc::new(Mutex::new(0)),
        }
    }

    pub async fn max_pending_seen(&self) -> u32 {
        *self.max_pending.lock().await
    }
}

#[async_trait]
impl CeremonySurface for MockCeremonySurface {
    async fn request_approval(&self, _action: DraftedAction) -> CeremonyOutcome {
        {
            let mut pending = self.pending.lock().await;
            *pending += 1;
            let mut max_pending = self.max_pending.lock().await;
            *max_pending = (*max_pending).max(*pending);
        }

        let outcome = self
            .outcomes
            .lock()
            .await
            .pop_front()
            .unwrap_or(CeremonyOutcome::Denied {
                reason: "no mock ceremony outcome configured".to_string(),
            });

        let mut pending = self.pending.lock().await;
        *pending = pending.saturating_sub(1);
        outcome
    }
}

pub fn is_external_action(tool_name: &str) -> bool {
    matches!(
        tool_name,
        "share.hand_off" | "payment.request" | "esign.request"
    )
}
