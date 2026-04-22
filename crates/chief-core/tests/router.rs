//! Integration tests for the model router.

use chief_core::router::{
    check_pack_installable, Arch, BindingKind, Defaults, DeviceCapability, InstallBlockReason,
    ModelRouter, ModelsConfig, PackGrant, TierBinding, TierBindings,
};
use chief_sdk::Tier;
use std::collections::HashMap;
use std::path::PathBuf;

#[test]
fn config_round_trip_toml() {
    let cfg = ModelsConfig {
        tiers: TierBindings {
            fast: Some(TierBinding {
                kind: BindingKind::Local,
                model_id: "qwen-2.5-3b".to_string(),
                provider: None,
                local_path: Some(PathBuf::from("/opt/models/qwen-3b.gguf")),
            }),
            deep: Some(TierBinding {
                kind: BindingKind::Cloud,
                model_id: "claude-sonnet".to_string(),
                provider: Some("anthropic".to_string()),
                local_path: None,
            }),
        },
        providers: {
            let mut m = HashMap::new();
            m.insert(
                "anthropic".to_string(),
                chief_core::router::ProviderBinding {
                    id: "anthropic".to_string(),
                    api_key_sealed_ref: "sealed:key1".to_string(),
                    base_url: Some("https://api.anthropic.com/v1".to_string()),
                    models_allowed: vec!["claude-sonnet".to_string()],
                },
            );
            m
        },
        defaults: Defaults {
            local_only: false,
            daily_cost_usd_cap: Some(5.0),
        },
    };

    let toml_str = toml::to_string(&cfg).expect("serialize");
    let parsed: ModelsConfig = toml::from_str(&toml_str).expect("deserialize");

    assert_eq!(
        cfg.tiers.fast.as_ref().unwrap().model_id,
        parsed.tiers.fast.as_ref().unwrap().model_id
    );
    assert_eq!(
        cfg.tiers.deep.as_ref().unwrap().provider,
        parsed.tiers.deep.as_ref().unwrap().provider
    );
    assert_eq!(
        cfg.defaults.daily_cost_usd_cap,
        parsed.defaults.daily_cost_usd_cap
    );
}

#[test]
fn device_can_run_fast_tier() {
    let dev = DeviceCapability {
        ram_gb: 8,
        cpu_cores: 4,
        has_gpu: false,
        arch: Arch::X86_64,
    };
    assert!(dev.can_run_tier_locally(&Tier::Fast));
}

#[test]
fn device_cannot_run_fast_tier_low_ram() {
    let dev = DeviceCapability {
        ram_gb: 2,
        cpu_cores: 2,
        has_gpu: false,
        arch: Arch::X86_64,
    };
    assert!(!dev.can_run_tier_locally(&Tier::Fast));
}

#[test]
fn device_can_run_deep_tier_apple_silicon() {
    let dev = DeviceCapability {
        ram_gb: 16,
        cpu_cores: 8,
        has_gpu: true,
        arch: Arch::AppleSilicon,
    };
    assert!(dev.can_run_tier_locally(&Tier::Deep));
}

#[test]
fn device_cannot_run_deep_tier_no_gpu_low_ram() {
    let dev = DeviceCapability {
        ram_gb: 16,
        cpu_cores: 8,
        has_gpu: false,
        arch: Arch::X86_64,
    };
    assert!(!dev.can_run_tier_locally(&Tier::Deep));
}

#[test]
fn device_can_run_deep_tier_high_ram_no_gpu() {
    let dev = DeviceCapability {
        ram_gb: 32,
        cpu_cores: 16,
        has_gpu: false,
        arch: Arch::X86_64,
    };
    assert!(dev.can_run_tier_locally(&Tier::Deep));
}

#[test]
fn router_pick_local_fast() {
    let config = ModelsConfig {
        tiers: TierBindings {
            fast: Some(TierBinding {
                kind: BindingKind::Local,
                model_id: "qwen-fast".to_string(),
                provider: None,
                local_path: Some(PathBuf::from("/models/fast.gguf")),
            }),
            deep: None,
        },
        providers: HashMap::new(),
        defaults: Defaults::default(),
    };

    let probe = DeviceCapability {
        ram_gb: 16,
        cpu_cores: 8,
        has_gpu: true,
        arch: Arch::X86_64,
    };

    let router = ModelRouter::new(config, probe);
    let choice = router.pick(Tier::Fast).expect("pick");
    assert_eq!(choice.model_id, "qwen-fast");
    assert_eq!(choice.kind, "local");
}

#[test]
fn router_pick_unbound_tier() {
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
    assert!(router.pick(Tier::Deep).is_err());
}

#[test]
fn router_local_only_blocks_cloud() {
    let config = ModelsConfig {
        tiers: TierBindings {
            fast: Some(TierBinding {
                kind: BindingKind::Cloud,
                model_id: "claude".to_string(),
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
        ram_gb: 8,
        cpu_cores: 4,
        has_gpu: false,
        arch: Arch::X86_64,
    };

    let router = ModelRouter::new(config, probe);
    assert!(router.pick(Tier::Fast).is_err());
}

#[test]
fn check_unbound_tier_blocks_install() {
    let config = ModelsConfig {
        tiers: TierBindings {
            fast: Some(TierBinding {
                kind: BindingKind::Local,
                model_id: "qwen-fast".to_string(),
                provider: None,
                local_path: Some(PathBuf::from("/models/fast.gguf")),
            }),
            deep: None,
        },
        providers: HashMap::new(),
        defaults: Defaults::default(),
    };

    let grants = vec![PackGrant::LlmHarness {
        tier: "deep".to_string(),
    }];

    let result = check_pack_installable("test-pack", &grants, &config);
    assert!(result.is_err());

    if let Err(InstallBlockReason::UnboundTier {
        tier,
        pack_id,
        grant_kind,
    }) = result
    {
        assert_eq!(tier, "deep");
        assert_eq!(pack_id, "test-pack");
        assert_eq!(grant_kind, "llm.harness");
    }
}

#[test]
fn check_bound_tier_allows_install() {
    let config = ModelsConfig {
        tiers: TierBindings {
            fast: Some(TierBinding {
                kind: BindingKind::Local,
                model_id: "qwen-fast".to_string(),
                provider: None,
                local_path: Some(PathBuf::from("/models/fast.gguf")),
            }),
            deep: None,
        },
        providers: HashMap::new(),
        defaults: Defaults::default(),
    };

    let grants = vec![PackGrant::LlmAi {
        tier: "fast".to_string(),
    }];

    let result = check_pack_installable("test-pack", &grants, &config);
    assert!(result.is_ok());
}

#[test]
fn router_pick_cloud_with_provider() {
    let mut providers = HashMap::new();
    providers.insert(
        "anthropic".to_string(),
        chief_core::router::ProviderBinding {
            id: "anthropic".to_string(),
            api_key_sealed_ref: "sealed:key".to_string(),
            base_url: Some("https://api.anthropic.com/v1".to_string()),
            models_allowed: vec!["claude-sonnet".to_string()],
        },
    );

    let config = ModelsConfig {
        tiers: TierBindings {
            fast: None,
            deep: Some(TierBinding {
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
