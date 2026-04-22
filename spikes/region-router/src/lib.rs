//! Chief OS Region Router — deterministic HAX region classifier
//!
//! This module implements the deterministic rule-table classifier that maps
//! action metadata + trust ledger snapshots to HAX regions (1-8), surfaces,
//! and friction tiers. No LLM in the hot path; pure rule evaluation.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

pub mod rules;
pub use rules::RuleEngine;

/// Reversibility classification for an action
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReversibilityClass {
    /// Easily undone (e.g., draft email)
    Soft,
    /// Hard to undo but possible (e.g., calendar block on busy day)
    Hard,
    /// Cannot be undone (e.g., contract signature, published post)
    Irreversible,
}

/// Time horizon for commitment — affects which tier of approval is needed
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Horizon {
    /// < 1 week
    Short,
    /// 1 week - 3 months
    Medium,
    /// 3 months - 1 year
    Long,
    /// Open-ended (indefinite commitment)
    Open,
}

/// Legal or binding effect of the action
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LegalEffect {
    /// No legal binding (e.g., internal note)
    None,
    /// Creates draft/proposal (e.g., contract draft)
    Draft,
    /// Legally binding (e.g., signed contract, published commitment)
    Binding,
}

/// Time to detect failure — affects monitoring and rollback window
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TimeToDetect {
    /// Immediate (seconds)
    Seconds,
    /// Minutes
    Minutes,
    /// Hours
    Hours,
    /// Days or longer
    Days,
}

/// Metadata about a proposed agent action for classification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionMetadata {
    /// How easily this action can be undone
    pub reversibility_class: ReversibilityClass,
    /// Monetary impact in USD (0 for non-monetary)
    pub monetary_impact_usd: f64,
    /// Number of affected parties (people, orgs, systems)
    pub affected_parties: u32,
    /// Legal binding effect
    pub legal_effect: LegalEffect,
    /// How quickly a failure would be detectable
    pub time_to_detect: TimeToDetect,
    /// Commitment horizon (how long the action commits us)
    pub horizon: Horizon,
    /// Trust ledger category (e.g., "email", "finance", "calendar")
    pub category: String,
}

/// Snapshot of current Trust Ledger delegation levels per category
/// Maps category -> delegation level (1..=5)
pub type LedgerSnapshot = HashMap<String, u8>;

/// HAX region (1-8) — determines surface and friction tier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[repr(u8)]
pub enum Region {
    /// Ad-hoc chat: low D, low C, short H
    R1 = 1,
    /// Intermediate: low D, low C, medium H
    R2 = 2,
    /// Evidence card: low D, high C, short H
    R3 = 3,
    /// Quarterly review: low D, high C, long H
    R4 = 4,
    /// One-tap queue: high D, low C, short H
    R5 = 5,
    /// Ambient (resting state): high D, low C, long H
    R6 = 6,
    /// Just-in-time ceremony: high D, high C, short H
    R7 = 7,
    /// Graduated autonomy + renegotiation: high D, high C, long H
    R8 = 8,
}

impl Region {
    /// Return numeric region ID (1-8)
    pub fn id(self) -> u8 {
        self as u8
    }
}

/// User-facing surface where this action is presented
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum Surface {
    /// Card in the action queue (auto-dismiss or one-tap)
    QueueCard,
    /// Evidence card in Morning Brief
    EvidenceCard,
    /// Ceremony (hold + biometric + cooldown)
    Ceremony,
    /// Ceremony with co-sign requirement
    CeremonyCoSign,
    /// Automatic action with logging only
    AutoLog,
}

/// Friction tier (0-4) — amount of cognitive/physical effort required
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum FrictionTier {
    /// Auto-send, logged
    Tier0 = 0,
    /// Single tap
    Tier1 = 1,
    /// Evidence card + confirm
    Tier2 = 2,
    /// Ceremony (3s hold + biometric + 60s cooldown)
    Tier3 = 3,
    /// Ceremony + co-sign
    Tier4 = 4,
}

impl FrictionTier {
    /// Return numeric tier ID (0-4)
    pub fn level(self) -> u8 {
        self as u8
    }
}

/// Complete classification result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Classification {
    /// HAX region (1-8)
    pub region: Region,
    /// User-facing surface
    pub surface: Surface,
    /// Friction tier (0-4)
    pub friction_tier: FrictionTier,
    /// Audit template ID for this decision
    pub audit_template: String,
}

/// Error type for classification failures
#[derive(Debug, Error)]
pub enum ClassifyError {
    #[error("Unknown category: {0}")]
    UnknownCategory(String),
    #[error("Invalid delegation level: {0} (must be 1-5)")]
    InvalidDelegation(u8),
    #[error("Rules engine error: {0}")]
    EngineError(String),
    #[error("Missing required metadata")]
    MissingMetadata,
}

/// Classify an action given current metadata and ledger state
///
/// This is the hot-path function: deterministic, no I/O, no LLM.
/// All decisions come from pre-loaded rule tables.
pub fn classify(
    action: &ActionMetadata,
    ledger: &LedgerSnapshot,
) -> Result<Classification, ClassifyError> {
    let engine = RuleEngine::default();
    engine.classify(action, ledger)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_region_ordering() {
        assert!(Region::R1 < Region::R8);
        assert_eq!(Region::R1.id(), 1);
        assert_eq!(Region::R8.id(), 8);
    }

    #[test]
    fn test_friction_tier_ordering() {
        assert!(FrictionTier::Tier0 < FrictionTier::Tier4);
        assert_eq!(FrictionTier::Tier0.level(), 0);
        assert_eq!(FrictionTier::Tier4.level(), 4);
    }
}
