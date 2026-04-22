//! Error types for AI and Harness operations (ADR-0013).

use crate::tier::Tier;
use thiserror::Error;

/// Errors from `ctx.ai()` operations.
#[derive(Debug, Error)]
pub enum AiError {
    #[error("capability denied: {reason}")]
    CapabilityDenied { reason: String },

    #[error("tier {tier} unavailable on this system")]
    TierUnavailable { tier: Tier },

    #[error("max_tokens clamped: requested {requested}, allowed {allowed}")]
    MaxTokensClamped { requested: u32, allowed: u32 },

    #[error("model error: {0}")]
    Model(String),

    #[error("budget exceeded: {0}")]
    Budget(String),

    #[error("schema parse failed: {0}")]
    Schema(String),
}

/// Errors from `ctx.harness()` operations.
#[derive(Debug, Error)]
pub enum HarnessError {
    #[error("capability denied: {reason}")]
    CapabilityDenied { reason: String },

    #[error("tier {tier} unavailable on this system")]
    TierUnavailable { tier: Tier },

    #[error("max turns exceeded: {turns} > {limit}")]
    MaxTurnsExceeded { turns: u32, limit: u32 },

    #[error("max cost exceeded: ${cost:.2} > ${limit:.2}")]
    MaxCostExceeded { cost: f64, limit: f64 },

    #[error("max wall time exceeded: {wall}s > {limit}s")]
    MaxWallExceeded { wall: u32, limit: u32 },

    #[error("engine error: {0}")]
    Engine(String),

    #[error("tool error: {0}")]
    Tool(String),
}
