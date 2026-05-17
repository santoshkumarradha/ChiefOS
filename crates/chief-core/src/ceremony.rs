//! Ceremony store for capability approvals.
//!
//! A Ceremony arises whenever a capability is escalated: an agent requested
//! something its current grants don't cover (e.g. `net.http` to a new host,
//! agent spawning for a previously-unauthorised pack). The broker rejects the
//! op; the orchestrator converts that denial into a `CeremonyItem`, stores it
//! here, and notifies the human via the Inbox.
//!
//! On `approve` (with a ≥ 3s hold-to-confirm), the broker issues the
//! requested grant for real and the ceremony transitions to `Approved`.
//!
//! All state is in-memory — the demo shell restart-resets the ceremony list
//! along with everything else.

use crate::broker::{CapabilityBroker, GrantHandle};
use crate::capability::{CapabilityKind, Grant, PrincipalId};
use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Minimum hold duration (in milliseconds) required for `approve`.
pub const MIN_HOLD_MS: u64 = 3_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CeremonyStatus {
    Pending,
    Approved,
    Denied,
    Expired,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CeremonyEvidence {
    /// Human-readable description of what's being requested.
    pub summary: String,
    /// Structured details — typically the attempted operation.
    pub details: serde_json::Value,
}

#[derive(Debug, Clone, Serialize)]
pub struct CeremonyItem {
    pub id: String,
    pub title: String,
    pub evidence: CeremonyEvidence,
    pub source_agent: String,
    /// Current trust score for the requesting agent (0-10 scale).
    pub trust_context: u8,
    /// Serialized grant that will be issued on approval.
    pub proposed_grant: Grant,
    /// Optional BLAKE3 hash of the exact action payload being approved.
    ///
    /// When present, approval requests must present the same hash. This keeps
    /// Ceremony authorization bound to the draft the human saw instead of to a
    /// mutable UI object.
    pub payload_hash: Option<String>,
    /// Principal that receives the grant on approval.
    pub target_principal: String,
    /// Window (in seconds) during which the approved grant can be rolled
    /// back without friction.
    pub rollback_window_secs: i64,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub status: CeremonyStatus,
    pub resolved_at: Option<DateTime<Utc>>,
    /// Handle of the grant that was issued (populated once approved).
    pub grant_issued: Option<GrantHandle>,
}

#[derive(Debug, Clone)]
pub struct NewCeremony {
    pub title: String,
    pub evidence: CeremonyEvidence,
    pub source_agent: String,
    pub trust_context: u8,
    pub proposed_grant: Grant,
    pub payload_hash: Option<String>,
    pub target_principal: String,
    pub rollback_window: Duration,
    pub ceremony_ttl: Duration,
}

#[derive(Debug, thiserror::Error)]
pub enum CeremonyError {
    #[error("ceremony not found: {0}")]
    NotFound(String),
    #[error("ceremony already resolved: status={0:?}")]
    AlreadyResolved(CeremonyStatus),
    #[error("hold too short: held {held_ms}ms, required {required_ms}ms")]
    HoldTooShort { held_ms: u64, required_ms: u64 },
    #[error("ceremony expired")]
    Expired,
    #[error("payload hash mismatch")]
    PayloadHashMismatch {
        expected: String,
        actual: Option<String>,
    },
    #[error("grant issuance failed: {0}")]
    GrantIssueFailed(String),
}

/// In-memory ceremony store. Cloning is cheap — state lives behind `Arc`.
#[derive(Clone, Default)]
pub struct CeremonyStore {
    inner: Arc<RwLock<HashMap<String, CeremonyItem>>>,
}

impl CeremonyStore {
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a ceremony and return its id.
    pub async fn open(&self, input: NewCeremony) -> CeremonyItem {
        let id = format!("ceremony_{}", uuid::Uuid::new_v4());
        let now = Utc::now();
        let item = CeremonyItem {
            id: id.clone(),
            title: input.title,
            evidence: input.evidence,
            source_agent: input.source_agent,
            trust_context: input.trust_context,
            proposed_grant: input.proposed_grant,
            payload_hash: input.payload_hash,
            target_principal: input.target_principal,
            rollback_window_secs: input.rollback_window.num_seconds(),
            created_at: now,
            expires_at: now + input.ceremony_ttl,
            status: CeremonyStatus::Pending,
            resolved_at: None,
            grant_issued: None,
        };
        let mut guard = self.inner.write().await;
        guard.insert(id, item.clone());
        item
    }

    pub async fn get(&self, id: &str) -> Option<CeremonyItem> {
        let guard = self.inner.read().await;
        guard.get(id).cloned()
    }

    pub async fn list(&self) -> Vec<CeremonyItem> {
        let guard = self.inner.read().await;
        let mut items: Vec<CeremonyItem> = guard.values().cloned().collect();
        items.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        items
    }

    pub async fn list_pending(&self) -> Vec<CeremonyItem> {
        self.list()
            .await
            .into_iter()
            .filter(|c| c.status == CeremonyStatus::Pending)
            .collect()
    }

    /// Approve a ceremony. Requires `held_ms >= MIN_HOLD_MS`. On success,
    /// issues the ceremony's proposed grant via the broker and records the
    /// handle on the ceremony item.
    pub async fn approve(
        &self,
        id: &str,
        held_ms: u64,
        payload_hash: Option<&str>,
        broker: &CapabilityBroker,
    ) -> Result<CeremonyItem, CeremonyError> {
        if held_ms < MIN_HOLD_MS {
            return Err(CeremonyError::HoldTooShort {
                held_ms,
                required_ms: MIN_HOLD_MS,
            });
        }

        // First, take a snapshot to validate without holding the write lock
        // across the broker call.
        let snapshot = {
            let guard = self.inner.read().await;
            guard
                .get(id)
                .cloned()
                .ok_or_else(|| CeremonyError::NotFound(id.to_string()))?
        };

        if snapshot.status != CeremonyStatus::Pending {
            return Err(CeremonyError::AlreadyResolved(snapshot.status));
        }
        if let Some(expected) = snapshot.payload_hash.as_deref() {
            if payload_hash != Some(expected) {
                return Err(CeremonyError::PayloadHashMismatch {
                    expected: expected.to_string(),
                    actual: payload_hash.map(ToOwned::to_owned),
                });
            }
        }
        if Utc::now() > snapshot.expires_at {
            // Mark it expired while we're here.
            let mut guard = self.inner.write().await;
            if let Some(entry) = guard.get_mut(id) {
                if entry.status == CeremonyStatus::Pending {
                    entry.status = CeremonyStatus::Expired;
                    entry.resolved_at = Some(Utc::now());
                }
            }
            return Err(CeremonyError::Expired);
        }

        // Issue the grant via the broker. This is the real capability
        // side-effect; if it fails we surface the error and leave the
        // ceremony in `Pending` (future tries can retry).
        let principal = PrincipalId::from(snapshot.target_principal.as_str());
        let grant = snapshot.proposed_grant.clone();
        let grant_id = broker
            .issue(principal.clone(), grant)
            .await
            .map_err(|e| CeremonyError::GrantIssueFailed(e.to_string()))?;

        let mut guard = self.inner.write().await;
        let entry = guard
            .get_mut(id)
            .ok_or_else(|| CeremonyError::NotFound(id.to_string()))?;
        entry.status = CeremonyStatus::Approved;
        entry.resolved_at = Some(Utc::now());
        entry.grant_issued = Some(GrantHandle {
            id: grant_id,
            principal,
            source: crate::broker::GrantSource::User,
        });
        Ok(entry.clone())
    }

    pub async fn deny(&self, id: &str) -> Result<CeremonyItem, CeremonyError> {
        let mut guard = self.inner.write().await;
        let entry = guard
            .get_mut(id)
            .ok_or_else(|| CeremonyError::NotFound(id.to_string()))?;
        if entry.status != CeremonyStatus::Pending {
            return Err(CeremonyError::AlreadyResolved(entry.status));
        }
        entry.status = CeremonyStatus::Denied;
        entry.resolved_at = Some(Utc::now());
        Ok(entry.clone())
    }
}

/// Default rollback window durations by broad capability category.
///
/// Picked conservatively: anything touching money or payments gets a long
/// window; network expansion grants get a shorter one. These are reference
/// values for the demo and can be overridden when creating ceremonies.
pub fn default_rollback_window(grant: &Grant) -> Duration {
    let has_payment = grant
        .capabilities
        .iter()
        .any(|c| matches!(c, CapabilityKind::PaymentRequest { .. }));
    if has_payment {
        Duration::hours(72)
    } else {
        Duration::hours(24)
    }
}

/// Default TTL for an unanswered ceremony. After this it auto-expires.
pub fn default_ceremony_ttl() -> Duration {
    Duration::hours(24)
}
