//! Model router — picks concrete models based on user configuration, device capability, and tier.
//!
//! This module implements ADR-0013 Phase 2:
//! - Models configuration (`$CHIEF_HOME/models.toml`)
//! - Tier→model bindings (Fast and Deep)
//! - Device capability probing (RAM, CPU, GPU, arch)
//! - Model picking algorithm with consequentiality guards
//! - Install-time tier availability check

pub mod config;
pub mod install_check;
pub mod picker;
pub mod probe;
pub mod provider_config;

pub use config::{BindingKind, Defaults, ModelsConfig, ProviderBinding, TierBinding, TierBindings};
pub use install_check::{check_pack_installable, InstallBlockReason, PackGrant};
pub use picker::{ModelChoice, ModelRouter, RouteError, RoutedEndpoint, RouterError};
pub use probe::{Arch, DeviceCapability};
pub use provider_config::{
    EnvOverrideSealedKeyStore, InMemorySealedKeyStore, OauthKeyHandle,
    ProviderBinding as CloudProviderBinding, ProviderError, ProviderKind, ProviderRegistry,
    ResolvedProviderRoute, SealedKeyStore, SharedProviderRegistry, SharedSealedKeyStore,
};
