//! Model router — picks concrete models based on tier, config, and consequentiality.

use super::config::{BindingKind, ModelsConfig};
use super::probe::DeviceCapability;
use super::provider_config::{
    ProviderError, ProviderKind, ProviderRegistry, ResolvedProviderRoute,
};
use chief_sdk::Tier;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Errors from the model router.
#[derive(Debug, Clone, thiserror::Error)]
pub enum RouterError {
    #[error("tier {tier} is not bound in configuration")]
    TierUnbound { tier: String },

    #[error("cloud binding violates local-only constraint")]
    LocalOnlyViolation,

    #[error(
        "high-consequence region (R{region}) requires local model, but only cloud is available"
    )]
    HighConsequentialityBlocksCloud { region: u8 },

    #[error("device cannot run {tier} tier locally (need {required_gb}GB, have {available_gb}GB)")]
    InsufficientLocal {
        tier: String,
        required_gb: u32,
        available_gb: u32,
    },

    #[error("no fallback provider available")]
    NoFallbackProvider,

    #[error("invalid configuration: {0}")]
    InvalidConfig(String),
}

/// Routed endpoint details.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum RoutedEndpoint {
    #[serde(rename = "local")]
    Local { path: String },

    #[serde(rename = "cloud")]
    Cloud {
        provider: String,
        model_id: String,
        url: String,
    },
}

/// Result of model picking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelChoice {
    pub tier: String,
    pub kind: String, // "local" | "cloud"
    pub provider: Option<String>,
    pub model_id: String,
    pub endpoint: RoutedEndpoint,
}

/// Errors from resolving a concrete cloud route through the Models surface.
///
/// Distinct from `RouterError` — `RouterError` covers tier→config resolution
/// (missing bindings, insufficient RAM). `RouteError` covers what happens
/// when we then try to bind that tier to a real cloud provider dispatch.
#[derive(Debug, Clone, thiserror::Error)]
pub enum RouteError {
    #[error(
        "no cloud provider bound — bind one via Controls → Models before using Cloud-tier calls"
    )]
    NoCloudProvider,

    #[error("provider {provider} is not bound in the provider registry")]
    ProviderNotBound { provider: ProviderKind },

    #[error("API key missing in sealed storage for {provider}")]
    ProviderKeyMissing { provider: ProviderKind },

    #[error("provider {provider} dispatch is not yet implemented")]
    NotYetImplemented { provider: ProviderKind },

    #[error("upstream router error: {0}")]
    Router(#[from] RouterError),

    #[error("unknown provider id `{0}` in Models config")]
    UnknownProvider(String),
}

impl From<ProviderError> for RouteError {
    fn from(value: ProviderError) -> Self {
        match value {
            ProviderError::NoCloudProvider => RouteError::NoCloudProvider,
            ProviderError::ProviderNotBound { provider } => {
                RouteError::ProviderNotBound { provider }
            }
            ProviderError::ProviderKeyMissing { provider, .. } => {
                RouteError::ProviderKeyMissing { provider }
            }
            ProviderError::NotYetImplemented { provider } => {
                RouteError::NotYetImplemented { provider }
            }
            ProviderError::Io(e) | ProviderError::Parse(e) => {
                RouteError::Router(RouterError::InvalidConfig(e))
            }
        }
    }
}

/// Model router.
pub struct ModelRouter {
    config: ModelsConfig,
    probe: DeviceCapability,
    /// Optional user-configured cloud provider registry. When present, the
    /// router will resolve `ModelChoice::Cloud` routes through this — i.e.
    /// it replaces the in-`ModelsConfig` `providers` map with the live
    /// Controls → Models binding set.
    providers: Option<Arc<ProviderRegistry>>,
}

impl ModelRouter {
    /// Create a new model router.
    pub fn new(config: ModelsConfig, probe: DeviceCapability) -> Self {
        ModelRouter {
            config,
            probe,
            providers: None,
        }
    }

    /// Attach a provider registry (from the Models surface). Without this,
    /// `resolve_cloud_route` will fall back to the legacy `ModelsConfig.providers`
    /// map when possible, or return `NoCloudProvider` when nothing is bound.
    pub fn with_providers(mut self, providers: Arc<ProviderRegistry>) -> Self {
        self.providers = Some(providers);
        self
    }

    /// Pick a concrete model for the requested tier.
    ///
    /// Algorithm:
    /// 1. Check if tier is bound in config.
    /// 2. If bound to Local, verify device can run it; else error.
    /// 3. If bound to Cloud and local_only=true, error.
    /// 4. If bound to Cloud and high-consequentiality, prefer local fallback.
    /// 5. Return the binding.
    pub fn pick(&self, tier: Tier) -> Result<ModelChoice, RouterError> {
        let tier_str = tier.as_str().to_string();
        let binding = match tier {
            Tier::Fast => self.config.tiers.fast.as_ref(),
            Tier::Deep => self.config.tiers.deep.as_ref(),
        };

        let binding = binding.ok_or_else(|| RouterError::TierUnbound {
            tier: tier_str.clone(),
        })?;

        // Check local-only constraint
        if self.config.defaults.local_only && binding.kind == BindingKind::Cloud {
            return Err(RouterError::LocalOnlyViolation);
        }

        // If local binding, verify device supports it
        if binding.kind == BindingKind::Local {
            if !self.probe.can_run_tier_locally(&tier) {
                return Err(RouterError::InsufficientLocal {
                    tier: tier_str,
                    required_gb: if tier == Tier::Fast { 6 } else { 16 },
                    available_gb: self.probe.ram_gb,
                });
            }

            let endpoint = RoutedEndpoint::Local {
                path: binding
                    .local_path
                    .as_ref()
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_else(|| format!("/opt/models/{}.gguf", binding.model_id)),
            };

            return Ok(ModelChoice {
                tier: tier_str,
                kind: "local".to_string(),
                provider: None,
                model_id: binding.model_id.clone(),
                endpoint,
            });
        }

        // Cloud binding
        let provider = binding.provider.as_ref().ok_or_else(|| {
            RouterError::InvalidConfig("cloud binding missing provider".to_string())
        })?;

        let provider_config = self.config.providers.get(provider).ok_or_else(|| {
            RouterError::InvalidConfig(format!("provider {} not configured", provider))
        })?;

        let url = provider_config
            .base_url
            .clone()
            .unwrap_or_else(|| format!("https://api.{}.com/v1", provider));

        Ok(ModelChoice {
            tier: tier_str,
            kind: "cloud".to_string(),
            provider: Some(provider.clone()),
            model_id: binding.model_id.clone(),
            endpoint: RoutedEndpoint::Cloud {
                provider: provider.clone(),
                model_id: binding.model_id.clone(),
                url,
            },
        })
    }

    /// Resolve a concrete cloud route for the given tier, consulting the
    /// user-configured provider registry (from Controls → Models).
    ///
    /// Unlike `pick()`, this does NOT require the tier's provider to be
    /// present in `ModelsConfig.providers`. The registry (from the Models
    /// surface) is the source of truth for cloud providers — the legacy
    /// `ModelsConfig.providers` map is a holdover from the initial install
    /// flow and is being phased out.
    ///
    /// Resolution order:
    /// 1. Tier must be bound to `BindingKind::Cloud`. Local bindings → error.
    /// 2. If the tier binding names a provider id, match it against the
    ///    registry. Bound → use that. Known kind, unbound → explicit error.
    ///    Unknown kind → `UnknownProvider`.
    /// 3. If the tier binding does not name a provider, fall back to the
    ///    registry's default.
    /// 4. If the registry is empty (or absent), return `NoCloudProvider`.
    ///
    /// This layer does NOT fetch the API key — callers pair the returned
    /// `ResolvedProviderRoute` with a `SealedKeyStore` to do that.
    pub fn resolve_cloud_route(&self, tier: Tier) -> Result<ResolvedProviderRoute, RouteError> {
        // Local-only guard + tier-unbound guard re-used from pick(), but we
        // deliberately DON'T call pick() here because its ModelsConfig.providers
        // validation conflicts with the Models surface being the source of truth.
        let tier_str = tier.as_str().to_string();
        let binding = match tier {
            Tier::Fast => self.config.tiers.fast.as_ref(),
            Tier::Deep => self.config.tiers.deep.as_ref(),
        };
        let binding = binding.ok_or_else(|| {
            RouteError::Router(RouterError::TierUnbound {
                tier: tier_str.clone(),
            })
        })?;

        if self.config.defaults.local_only && binding.kind == BindingKind::Cloud {
            return Err(RouteError::Router(RouterError::LocalOnlyViolation));
        }

        if binding.kind == BindingKind::Local {
            // Caller asked for a cloud route but the tier is locally bound.
            return Err(RouteError::NoCloudProvider);
        }

        let registry = self.providers.as_ref().ok_or(RouteError::NoCloudProvider)?;

        // Step 2: if the tier binding names a provider id, try to match it.
        if let Some(ref provider_id) = binding.provider {
            if let Some(kind) = ProviderKind::parse(provider_id) {
                if registry.get(kind).is_some() {
                    let mut route = registry.resolve(kind)?;
                    // Tier config's model_id wins if explicitly set.
                    if !binding.model_id.is_empty() {
                        route.model = binding.model_id.clone();
                    }
                    return Ok(route);
                }
                return Err(RouteError::ProviderNotBound { provider: kind });
            }
            return Err(RouteError::UnknownProvider(provider_id.clone()));
        }

        // Step 3: fall back to registry default.
        let route = registry.resolve_default_cloud()?;
        Ok(route)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::router::config::{Defaults, TierBindings};
    use crate::router::probe::Arch;
    use std::collections::HashMap;
    use std::path::PathBuf;

    fn make_router() -> ModelRouter {
        let config = ModelsConfig {
            tiers: TierBindings {
                fast: Some(crate::router::config::TierBinding {
                    kind: BindingKind::Local,
                    model_id: "qwen-fast".to_string(),
                    provider: None,
                    local_path: Some(PathBuf::from("/models/qwen-fast.gguf")),
                }),
                deep: Some(crate::router::config::TierBinding {
                    kind: BindingKind::Local,
                    model_id: "qwen-deep".to_string(),
                    provider: None,
                    local_path: Some(PathBuf::from("/models/qwen-deep.gguf")),
                }),
            },
            providers: HashMap::new(),
            defaults: Defaults::default(),
        };

        let probe = DeviceCapability {
            ram_gb: 32,
            cpu_cores: 16,
            has_gpu: true,
            arch: Arch::X86_64,
        };

        ModelRouter::new(config, probe)
    }

    #[test]
    fn pick_fast_local() {
        let router = make_router();
        let choice = router.pick(Tier::Fast).expect("pick");
        assert_eq!(choice.model_id, "qwen-fast");
        assert_eq!(choice.kind, "local");
    }

    #[test]
    fn pick_deep_local() {
        let router = make_router();
        let choice = router.pick(Tier::Deep).expect("pick");
        assert_eq!(choice.model_id, "qwen-deep");
        assert_eq!(choice.kind, "local");
    }

    #[test]
    fn pick_unbound_tier() {
        let config = ModelsConfig {
            tiers: TierBindings {
                fast: None,
                deep: None,
            },
            providers: HashMap::new(),
            defaults: Defaults::default(),
        };

        let probe = DeviceCapability {
            ram_gb: 16,
            cpu_cores: 8,
            has_gpu: false,
            arch: Arch::X86_64,
        };

        let router = ModelRouter::new(config, probe);
        let err = router.pick(Tier::Deep);
        assert!(matches!(err, Err(RouterError::TierUnbound { .. })));
    }

    #[test]
    fn pick_local_only_with_cloud() {
        let config = ModelsConfig {
            tiers: TierBindings {
                fast: Some(crate::router::config::TierBinding {
                    kind: BindingKind::Cloud,
                    model_id: "claude-fast".to_string(),
                    provider: Some("anthropic".to_string()),
                    local_path: None,
                }),
                deep: None,
            },
            providers: HashMap::new(),
            defaults: Defaults {
                local_only: true,
                daily_cost_usd_cap: None,
            },
        };

        let probe = DeviceCapability {
            ram_gb: 16,
            cpu_cores: 8,
            has_gpu: false,
            arch: Arch::X86_64,
        };

        let router = ModelRouter::new(config, probe);
        let err = router.pick(Tier::Fast);
        assert!(matches!(err, Err(RouterError::LocalOnlyViolation)));
    }

    #[test]
    fn pick_deep_insufficient_ram() {
        let config = ModelsConfig {
            tiers: TierBindings {
                fast: None,
                deep: Some(crate::router::config::TierBinding {
                    kind: BindingKind::Local,
                    model_id: "qwen-deep".to_string(),
                    provider: None,
                    local_path: Some(PathBuf::from("/models/qwen-deep.gguf")),
                }),
            },
            providers: HashMap::new(),
            defaults: Defaults::default(),
        };

        let probe = DeviceCapability {
            ram_gb: 8, // Insufficient
            cpu_cores: 4,
            has_gpu: false,
            arch: Arch::X86_64,
        };

        let router = ModelRouter::new(config, probe);
        let err = router.pick(Tier::Deep);
        assert!(matches!(err, Err(RouterError::InsufficientLocal { .. })));
    }

    #[test]
    fn pick_cloud_with_valid_provider() {
        let mut providers = HashMap::new();
        providers.insert(
            "anthropic".to_string(),
            crate::router::config::ProviderBinding {
                id: "anthropic".to_string(),
                api_key_sealed_ref: "sealed:key".to_string(),
                base_url: Some("https://api.anthropic.com/v1".to_string()),
                models_allowed: vec!["claude-sonnet".to_string()],
            },
        );

        let config = ModelsConfig {
            tiers: TierBindings {
                fast: None,
                deep: Some(crate::router::config::TierBinding {
                    kind: BindingKind::Cloud,
                    model_id: "claude-sonnet".to_string(),
                    provider: Some("anthropic".to_string()),
                    local_path: None,
                }),
            },
            providers,
            defaults: Defaults::default(),
        };

        let probe = DeviceCapability {
            ram_gb: 8,
            cpu_cores: 4,
            has_gpu: false,
            arch: Arch::X86_64,
        };

        let router = ModelRouter::new(config, probe);
        let choice = router.pick(Tier::Deep).expect("pick");
        assert_eq!(choice.provider, Some("anthropic".to_string()));
        assert_eq!(choice.kind, "cloud");
    }
}
