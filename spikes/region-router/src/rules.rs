//! Deterministic rule-table classifier for HAX regions
//!
//! The rule engine loads YAML-based rules that map action properties
//! to regions, surfaces, and friction tiers. Rules are deterministic:
//! same inputs always produce same outputs.

use crate::*;
use serde::{Deserialize, Serialize};

/// A single rule entry in the decision table
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleEntry {
    /// Minimum delegation level (1-5) required for this rule
    pub min_delegation: u8,
    /// Maximum delegation level (1-5) for this rule
    pub max_delegation: u8,
    /// Reversibility class this rule applies to
    pub reversibility: String, // "soft", "hard", "irreversible"
    /// Horizon this rule applies to
    pub horizon: String, // "short", "medium", "long", "open"
    /// Maximum monetary impact (USD) for this rule
    pub max_stake: f64,
    /// Target HAX region
    pub region: u8,
    /// Target surface
    pub surface: String,
    /// Target friction tier
    pub friction_tier: u8,
    /// Audit template ID
    pub audit_template: String,
}

/// The complete rules table, loaded from YAML
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RulesTable {
    pub version: String,
    pub rules: Vec<RuleEntry>,
}

impl RulesTable {
    /// Load rules from YAML string
    pub fn from_yaml(yaml: &str) -> Result<Self, ClassifyError> {
        serde_yaml::from_str(yaml).map_err(|e| ClassifyError::EngineError(e.to_string()))
    }
}

/// Rule engine with pre-loaded rules
pub struct RuleEngine {
    rules: RulesTable,
}

impl Default for RuleEngine {
    fn default() -> Self {
        // Load default rules
        let yaml = include_str!("../rules/default.yaml");
        let rules = RulesTable::from_yaml(yaml).expect("Failed to parse default rules");
        RuleEngine { rules }
    }
}

impl RuleEngine {
    /// Create a new rule engine with explicit rules
    pub fn new(rules: RulesTable) -> Self {
        RuleEngine { rules }
    }

    /// Classify an action given current metadata and ledger
    pub fn classify(
        &self,
        action: &ActionMetadata,
        ledger: &LedgerSnapshot,
    ) -> Result<Classification, ClassifyError> {
        // Get delegation level for this category
        let delegation = ledger
            .get(&action.category)
            .copied()
            .ok_or_else(|| ClassifyError::UnknownCategory(action.category.clone()))?;

        if !(1..=5).contains(&delegation) {
            return Err(ClassifyError::InvalidDelegation(delegation));
        }

        // Find the best matching rule (rules are ordered by specificity)
        let rule = self
            .rules
            .rules
            .iter()
            .find(|r| self.rule_matches(r, action, delegation))
            .ok_or_else(|| ClassifyError::EngineError("No matching rule found".to_string()))?;

        // Convert region ID to Region enum
        let region = match rule.region {
            1 => Region::R1,
            2 => Region::R2,
            3 => Region::R3,
            4 => Region::R4,
            5 => Region::R5,
            6 => Region::R6,
            7 => Region::R7,
            8 => Region::R8,
            _ => {
                return Err(ClassifyError::EngineError(format!(
                    "Invalid region: {}",
                    rule.region
                )))
            }
        };

        // Convert surface string to Surface enum
        let surface = match rule.surface.as_str() {
            "QueueCard" => Surface::QueueCard,
            "EvidenceCard" => Surface::EvidenceCard,
            "Ceremony" => Surface::Ceremony,
            "CeremonyCoSign" => Surface::CeremonyCoSign,
            "AutoLog" => Surface::AutoLog,
            _ => {
                return Err(ClassifyError::EngineError(format!(
                    "Invalid surface: {}",
                    rule.surface
                )))
            }
        };

        // Convert friction tier to FrictionTier enum
        let friction_tier = match rule.friction_tier {
            0 => FrictionTier::Tier0,
            1 => FrictionTier::Tier1,
            2 => FrictionTier::Tier2,
            3 => FrictionTier::Tier3,
            4 => FrictionTier::Tier4,
            _ => {
                return Err(ClassifyError::EngineError(format!(
                    "Invalid friction tier: {}",
                    rule.friction_tier
                )))
            }
        };

        Ok(Classification {
            region,
            surface,
            friction_tier,
            audit_template: rule.audit_template.clone(),
        })
    }

    /// Check if a rule matches the given action and delegation level
    fn rule_matches(&self, rule: &RuleEntry, action: &ActionMetadata, delegation: u8) -> bool {
        // Check delegation level
        if delegation < rule.min_delegation || delegation > rule.max_delegation {
            return false;
        }

        // Check reversibility
        let reversibility_str = match action.reversibility_class {
            ReversibilityClass::Soft => "soft",
            ReversibilityClass::Hard => "hard",
            ReversibilityClass::Irreversible => "irreversible",
        };
        if rule.reversibility != reversibility_str && rule.reversibility != "*" {
            return false;
        }

        // Check horizon
        let horizon_str = match action.horizon {
            Horizon::Short => "short",
            Horizon::Medium => "medium",
            Horizon::Long => "long",
            Horizon::Open => "open",
        };
        if rule.horizon != horizon_str && rule.horizon != "*" {
            return false;
        }

        // Check monetary impact
        if action.monetary_impact_usd > rule.max_stake && rule.max_stake >= 0.0 {
            return false;
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rules_table_loads() {
        let engine = RuleEngine::default();
        assert!(!engine.rules.rules.is_empty());
    }

    #[test]
    fn test_rule_matching() {
        let engine = RuleEngine::default();
        let mut ledger = LedgerSnapshot::new();
        ledger.insert("email".to_string(), 3);

        let action = ActionMetadata {
            reversibility_class: ReversibilityClass::Soft,
            monetary_impact_usd: 0.0,
            affected_parties: 1,
            legal_effect: LegalEffect::None,
            time_to_detect: TimeToDetect::Seconds,
            horizon: Horizon::Short,
            category: "email".to_string(),
        };

        let result = engine.classify(&action, &ledger);
        assert!(result.is_ok());
    }
}
