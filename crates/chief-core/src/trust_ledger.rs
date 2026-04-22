//! Per-agent trust accumulator.
//!
//! Every approved card, ceremony, or attestation bumps the score for the
//! responsible agent category. Every denied ceremony, reverted action, or
//! policy violation deducts from it. The scoreboard is in-memory and derived
//! from real events — there are no seeded values.
//!
//! Categories are free-form strings (e.g. `"email"`, `"calendar"`,
//! `"memory"`). The demo Brief picks the top-N by absolute activity.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LedgerDelta {
    Approval,
    Denial,
    Rollback,
    Attestation,
    PolicyViolation,
}

impl LedgerDelta {
    /// Score delta applied when this event is recorded.
    pub fn magnitude(self) -> i32 {
        match self {
            Self::Approval => 1,
            Self::Attestation => 1,
            Self::Denial => -1,
            Self::Rollback => -2,
            Self::PolicyViolation => -3,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LedgerEntry {
    pub category: String,
    pub delta: LedgerDelta,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustBarRow {
    pub category: String,
    /// Monotonically-adjusted score (0-10 display).
    pub score: u8,
    /// Raw sum across events — can be negative.
    pub raw: i32,
    /// Number of approvals.
    pub approvals: u32,
    /// Number of denials/rollbacks/violations.
    pub setbacks: u32,
}

#[derive(Debug, Default)]
struct Accumulator {
    raw: i32,
    approvals: u32,
    setbacks: u32,
}

/// Thread-safe in-memory trust ledger. Cheap to clone.
#[derive(Clone, Default)]
pub struct TrustLedger {
    entries: Arc<RwLock<Vec<LedgerEntry>>>,
}

impl TrustLedger {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn record(&self, category: impl Into<String>, delta: LedgerDelta) {
        let entry = LedgerEntry {
            category: category.into(),
            delta,
            timestamp: Utc::now(),
        };
        let mut guard = self.entries.write().await;
        guard.push(entry);
    }

    pub async fn entries(&self) -> Vec<LedgerEntry> {
        self.entries.read().await.clone()
    }

    /// Snapshot the ledger as trust-bar rows, sorted by total activity
    /// (approvals + setbacks) descending.
    pub async fn snapshot(&self) -> Vec<TrustBarRow> {
        let guard = self.entries.read().await;
        let mut agg: HashMap<String, Accumulator> = HashMap::new();

        for entry in guard.iter() {
            let acc = agg.entry(entry.category.clone()).or_default();
            acc.raw += entry.delta.magnitude();
            match entry.delta {
                LedgerDelta::Approval | LedgerDelta::Attestation => acc.approvals += 1,
                LedgerDelta::Denial | LedgerDelta::Rollback | LedgerDelta::PolicyViolation => {
                    acc.setbacks += 1
                }
            }
        }

        let mut rows: Vec<TrustBarRow> = agg
            .into_iter()
            .map(|(category, acc)| TrustBarRow {
                category,
                score: normalise_score(acc.raw),
                raw: acc.raw,
                approvals: acc.approvals,
                setbacks: acc.setbacks,
            })
            .collect();

        rows.sort_by(|a, b| {
            let a_act = a.approvals + a.setbacks;
            let b_act = b.approvals + b.setbacks;
            b_act.cmp(&a_act).then_with(|| a.category.cmp(&b.category))
        });
        rows
    }

    /// Top-N rows by activity.
    pub async fn top(&self, n: usize) -> Vec<TrustBarRow> {
        self.snapshot().await.into_iter().take(n).collect()
    }
}

/// Map an integer raw score onto the 0-10 display scale, clamping at the
/// edges. The demo UI treats this as a filled-bar indicator.
fn normalise_score(raw: i32) -> u8 {
    // +5 raw = score 10, 0 raw = score 5, -5 raw = score 0.
    let mapped = 5 + raw;
    mapped.clamp(0, 10) as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn empty_ledger_returns_empty_snapshot() {
        let ledger = TrustLedger::new();
        assert!(ledger.snapshot().await.is_empty());
    }

    #[tokio::test]
    async fn approvals_and_denials_aggregate() {
        let ledger = TrustLedger::new();
        ledger.record("email", LedgerDelta::Approval).await;
        ledger.record("email", LedgerDelta::Approval).await;
        ledger.record("email", LedgerDelta::Denial).await;
        let snap = ledger.snapshot().await;
        assert_eq!(snap.len(), 1);
        let row = &snap[0];
        assert_eq!(row.category, "email");
        assert_eq!(row.raw, 1);
        assert_eq!(row.approvals, 2);
        assert_eq!(row.setbacks, 1);
    }

    #[tokio::test]
    async fn top_returns_n_by_activity() {
        let ledger = TrustLedger::new();
        for _ in 0..3 {
            ledger.record("memory", LedgerDelta::Approval).await;
        }
        ledger.record("ops", LedgerDelta::Approval).await;
        let top = ledger.top(1).await;
        assert_eq!(top.len(), 1);
        assert_eq!(top[0].category, "memory");
    }
}
