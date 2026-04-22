//! Models configuration — tier→model bindings and provider definitions.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Models configuration stored in `$CHIEF_HOME/models.toml`.
/// Defines tier→model bindings, cloud providers, and user preferences.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ModelsConfig {
    pub tiers: TierBindings,
    pub providers: HashMap<String, ProviderBinding>,
    pub defaults: Defaults,
}

/// Tier→model bindings (Fast and Deep).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TierBindings {
    pub fast: Option<TierBinding>,
    pub deep: Option<TierBinding>,
}

/// A single tier's binding to a concrete model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TierBinding {
    pub kind: BindingKind,
    pub model_id: String,
    pub provider: Option<String>,    // For Cloud: provider identifier
    pub local_path: Option<PathBuf>, // For Local: path to model binary/weights
}

/// Binding type — local or cloud.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BindingKind {
    Local,
    Cloud,
}

/// Cloud provider configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderBinding {
    pub id: String,                  // "anthropic" | "openai" | "google" etc.
    pub api_key_sealed_ref: String,  // Reference to sealed token store
    pub base_url: Option<String>,    // Optional custom endpoint
    pub models_allowed: Vec<String>, // Models allowed from this provider
}

/// User preferences and defaults.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Defaults {
    pub local_only: bool,                // If true, no cloud bindings allowed
    pub daily_cost_usd_cap: Option<f64>, // Soft cap on daily cost
}

impl ModelsConfig {
    /// Load from TOML file.
    pub fn load_from_file(path: &std::path::Path) -> std::io::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        toml::from_str(&content)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))
    }

    /// Save to TOML file.
    pub fn save_to_file(&self, path: &std::path::Path) -> std::io::Result<()> {
        let content = toml::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?;
        std::fs::write(path, content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn models_config_default() {
        let cfg = ModelsConfig::default();
        assert!(cfg.tiers.fast.is_none());
        assert!(cfg.tiers.deep.is_none());
        assert!(cfg.providers.is_empty());
        assert!(!cfg.defaults.local_only);
        assert!(cfg.defaults.daily_cost_usd_cap.is_none());
    }

    #[test]
    fn models_config_round_trip() {
        let cfg = ModelsConfig {
            tiers: TierBindings {
                fast: Some(TierBinding {
                    kind: BindingKind::Local,
                    model_id: "qwen-2.5-3b-q4".to_string(),
                    provider: None,
                    local_path: Some(PathBuf::from("/opt/models/qwen-3b.gguf")),
                }),
                deep: Some(TierBinding {
                    kind: BindingKind::Cloud,
                    model_id: "claude-sonnet-4.6".to_string(),
                    provider: Some("anthropic".to_string()),
                    local_path: None,
                }),
            },
            providers: {
                let mut m = HashMap::new();
                m.insert(
                    "anthropic".to_string(),
                    ProviderBinding {
                        id: "anthropic".to_string(),
                        api_key_sealed_ref: "sealed:anthropic-key".to_string(),
                        base_url: None,
                        models_allowed: vec!["claude-sonnet-4.6".to_string()],
                    },
                );
                m
            },
            defaults: Defaults {
                local_only: false,
                daily_cost_usd_cap: Some(10.0),
            },
        };

        let toml_str = toml::to_string(&cfg).expect("serialize");
        let parsed: ModelsConfig = toml::from_str(&toml_str).expect("deserialize");

        assert_eq!(
            cfg.tiers.fast.as_ref().unwrap().model_id,
            parsed.tiers.fast.as_ref().unwrap().model_id
        );
        assert_eq!(
            cfg.tiers.deep.as_ref().unwrap().model_id,
            parsed.tiers.deep.as_ref().unwrap().model_id
        );
        assert_eq!(
            cfg.defaults.daily_cost_usd_cap,
            parsed.defaults.daily_cost_usd_cap
        );
    }

    #[test]
    fn models_config_file_round_trip() {
        let tmp = NamedTempFile::new().expect("create temp");
        let path = tmp.path().to_path_buf();

        let cfg = ModelsConfig {
            tiers: TierBindings {
                fast: Some(TierBinding {
                    kind: BindingKind::Local,
                    model_id: "test-fast".to_string(),
                    provider: None,
                    local_path: Some(PathBuf::from("/test/fast.gguf")),
                }),
                deep: None,
            },
            providers: HashMap::new(),
            defaults: Defaults::default(),
        };

        cfg.save_to_file(&path).expect("save");
        let loaded = ModelsConfig::load_from_file(&path).expect("load");

        assert_eq!(loaded.tiers.fast.as_ref().unwrap().model_id, "test-fast");
        assert!(loaded.tiers.deep.is_none());
    }
}
