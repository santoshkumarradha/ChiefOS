//! Tier enum — two-tier LLM access control (closed enum, ADR-0013).
//!
//! Packs declare which tier of model they need: Fast for cheap inference
//! (classification, formatting), Deep for capable reasoning (synthesis, tool-using).
//! The user's Models configuration in Controls maps each tier to a concrete model.

use serde::{Deserialize, Serialize};

/// Closed enumeration of LLM capability tiers.
/// Adding new variants requires an ADR amendment (see adr/0010-sdk-public-api-stability.md).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum Tier {
    /// Fast tier: cheap inference. v0 default: Qwen 2.5 3B Q4 (local).
    /// Use for: classification, intake, formatting, light reasoning.
    Fast,
    /// Deep tier: capable reasoning. v0 default: Qwen 14B Q4 (local, GPU-permitting).
    /// Use for: synthesis, tool-using agents, long-horizon reasoning.
    Deep,
}

impl Tier {
    /// Human-readable name.
    pub fn as_str(&self) -> &'static str {
        match self {
            Tier::Fast => "fast",
            Tier::Deep => "deep",
        }
    }
}

impl std::fmt::Display for Tier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tier_ser_de() {
        assert_eq!(serde_json::to_string(&Tier::Fast).unwrap(), r#""fast""#);
        assert_eq!(serde_json::to_string(&Tier::Deep).unwrap(), r#""deep""#);
    }

    #[test]
    fn tier_exhaustive_match() {
        let t = Tier::Fast;
        let _ = match t {
            Tier::Fast => "fast",
            Tier::Deep => "deep",
        };
        // This test passes iff match is exhaustive.
    }

    #[test]
    fn tier_unknown_fails() {
        let result: Result<Tier, _> = serde_json::from_str(r#""unknown""#);
        assert!(result.is_err());
    }
}
