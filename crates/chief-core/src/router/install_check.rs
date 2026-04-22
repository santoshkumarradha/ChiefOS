//! Install-time tier availability check.
//!
//! When a pack is installed, chief-core verifies that every LLM grant
//! (llm.ai or llm.harness) has a bound model in the user's configuration.
//! If not, installation is blocked with an actionable error.

use super::config::ModelsConfig;
use chief_sdk::CapabilityKind;
use serde::{Deserialize, Serialize};

/// Block reason for pack installation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InstallBlockReason {
    /// Pack requires a tier that is not bound.
    UnboundTier {
        tier: String,
        pack_id: String,
        grant_kind: String, // "llm.ai" | "llm.harness"
    },
}

impl std::fmt::Display for InstallBlockReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InstallBlockReason::UnboundTier {
                tier,
                pack_id,
                grant_kind,
            } => {
                write!(
                    f,
                    "Pack {} requires {}-tier model ({}), which is not configured.",
                    pack_id, tier, grant_kind
                )
            }
        }
    }
}

/// Check whether a pack can be installed given the configuration.
///
/// Returns Ok(()) if all required tiers are bound, or an InstallBlockReason if not.
pub fn check_pack_installable(
    pack_id: &str,
    grants: &[PackGrant],
    config: &ModelsConfig,
) -> Result<(), InstallBlockReason> {
    for grant in grants {
        match grant {
            PackGrant::LlmAi { tier } => {
                let tier_binding = match tier.as_str() {
                    "fast" => config.tiers.fast.as_ref(),
                    "deep" => config.tiers.deep.as_ref(),
                    _ => continue, // Unknown tier, skip
                };

                if tier_binding.is_none() {
                    return Err(InstallBlockReason::UnboundTier {
                        tier: tier.clone(),
                        pack_id: pack_id.to_string(),
                        grant_kind: "llm.ai".to_string(),
                    });
                }
            }
            PackGrant::LlmHarness { tier } => {
                let tier_binding = match tier.as_str() {
                    "fast" => config.tiers.fast.as_ref(),
                    "deep" => config.tiers.deep.as_ref(),
                    _ => continue,
                };

                if tier_binding.is_none() {
                    return Err(InstallBlockReason::UnboundTier {
                        tier: tier.clone(),
                        pack_id: pack_id.to_string(),
                        grant_kind: "llm.harness".to_string(),
                    });
                }
            }
        }
    }
    Ok(())
}

/// Simplified representation of a pack grant for install checking.
#[derive(Debug, Clone)]
pub enum PackGrant {
    LlmAi { tier: String },
    LlmHarness { tier: String },
}

impl PackGrant {
    /// Extract LLM grants from a CapabilityKind.
    pub fn from_capability(cap: &CapabilityKind) -> Option<PackGrant> {
        match cap {
            CapabilityKind::LlmAi { tier, .. } => Some(PackGrant::LlmAi { tier: tier.clone() }),
            CapabilityKind::LlmHarness { tier, .. } => {
                Some(PackGrant::LlmHarness { tier: tier.clone() })
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::router::config::{Defaults, TierBindings};
    use std::collections::HashMap;
    use std::path::PathBuf;

    fn make_config_with_tiers() -> ModelsConfig {
        use crate::router::config::{BindingKind, TierBinding};

        ModelsConfig {
            tiers: TierBindings {
                fast: Some(TierBinding {
                    kind: BindingKind::Local,
                    model_id: "qwen-fast".to_string(),
                    provider: None,
                    local_path: Some(PathBuf::from("/models/qwen-fast.gguf")),
                }),
                deep: None,
            },
            providers: HashMap::new(),
            defaults: Defaults::default(),
        }
    }

    fn make_config_no_tiers() -> ModelsConfig {
        ModelsConfig {
            tiers: TierBindings {
                fast: None,
                deep: None,
            },
            providers: HashMap::new(),
            defaults: Defaults::default(),
        }
    }

    #[test]
    fn check_unbound_deep_tier() {
        let config = make_config_with_tiers();
        let grants = vec![PackGrant::LlmHarness {
            tier: "deep".to_string(),
        }];

        let err = check_pack_installable("test-pack", &grants, &config);
        assert!(err.is_err());

        if let Err(InstallBlockReason::UnboundTier {
            tier,
            pack_id,
            grant_kind,
        }) = err
        {
            assert_eq!(tier, "deep");
            assert_eq!(pack_id, "test-pack");
            assert_eq!(grant_kind, "llm.harness");
        } else {
            panic!("expected UnboundTier");
        }
    }

    #[test]
    fn check_bound_fast_tier() {
        let config = make_config_with_tiers();
        let grants = vec![PackGrant::LlmAi {
            tier: "fast".to_string(),
        }];

        let result = check_pack_installable("test-pack", &grants, &config);
        assert!(result.is_ok());
    }

    #[test]
    fn check_multiple_grants_one_unbound() {
        let config = make_config_with_tiers();
        let grants = vec![
            PackGrant::LlmAi {
                tier: "fast".to_string(),
            },
            PackGrant::LlmHarness {
                tier: "deep".to_string(),
            },
        ];

        let err = check_pack_installable("test-pack", &grants, &config);
        assert!(err.is_err());
    }

    #[test]
    fn check_no_llm_grants() {
        let config = make_config_no_tiers();
        let grants: Vec<PackGrant> = vec![];

        let result = check_pack_installable("test-pack", &grants, &config);
        assert!(result.is_ok());
    }

    #[test]
    fn display_unbound_tier_error() {
        let err = InstallBlockReason::UnboundTier {
            tier: "deep".to_string(),
            pack_id: "my-pack".to_string(),
            grant_kind: "llm.harness".to_string(),
        };

        let msg = err.to_string();
        assert!(msg.contains("my-pack"));
        assert!(msg.contains("deep"));
        assert!(msg.contains("llm.harness"));
    }
}
