//! Integration tests for Models surface → cloud provider dispatch wiring.
//!
//! See `crates/chief-core/src/router/provider_config.rs` and
//! `crates/chief-core/src/router/picker.rs` for the implementation; this
//! file exercises the public surface and protects against regressions in:
//!
//!  - `ProviderRegistry::bind` / `get` / `default_cloud`
//!  - `ProviderRegistry::resolve_and_seal` (sealed-key fetch)
//!  - `ModelRouter::resolve_cloud_route` (tier → provider route)
//!  - Clean errors when no provider is bound or when the key is missing.
//!
//! Live dispatch (real HTTP to OpenRouter) is in `openrouter_live.rs` and is
//! `#[ignore]`-gated because it spends cents and requires `OPENROUTER_API_KEY`.

use chief_core::router::provider_config::ProviderBinding as CloudBinding;
use chief_core::router::{
    Arch, BindingKind, Defaults, DeviceCapability, InMemorySealedKeyStore, ModelRouter,
    ModelsConfig, OauthKeyHandle, ProviderKind, ProviderRegistry, RouteError, TierBinding,
    TierBindings,
};
use chief_sdk::Tier;
use std::collections::HashMap;
use std::sync::Arc;

fn openrouter_binding() -> CloudBinding {
    CloudBinding {
        provider: ProviderKind::OpenRouter,
        default_model: "openai/gpt-4o-mini".to_string(),
        key_handle: OauthKeyHandle::new("sealed:openrouter"),
        base_url: None,
    }
}

fn anthropic_binding() -> CloudBinding {
    CloudBinding {
        provider: ProviderKind::Anthropic,
        default_model: "claude-sonnet-4.6".to_string(),
        key_handle: OauthKeyHandle::new("sealed:anthropic"),
        base_url: None,
    }
}

fn cloud_tier_config(provider: &str, model: &str) -> ModelsConfig {
    ModelsConfig {
        tiers: TierBindings {
            fast: None,
            deep: Some(TierBinding {
                kind: BindingKind::Cloud,
                model_id: model.to_string(),
                provider: Some(provider.to_string()),
                local_path: None,
            }),
        },
        providers: HashMap::new(),
        defaults: Defaults::default(),
    }
}

fn probe() -> DeviceCapability {
    DeviceCapability {
        ram_gb: 8,
        cpu_cores: 4,
        has_gpu: false,
        arch: Arch::X86_64,
    }
}

// ---------------------------------------------------------------------------
// Registry behavior
// ---------------------------------------------------------------------------

#[test]
fn registry_bind_then_get_returns_binding() {
    let mut reg = ProviderRegistry::new();
    reg.bind(openrouter_binding()).unwrap();

    let got = reg.get(ProviderKind::OpenRouter).expect("bound");
    assert_eq!(got.default_model, "openai/gpt-4o-mini");
    assert_eq!(got.key_handle.as_str(), "sealed:openrouter");
}

#[test]
fn registry_default_cloud_returns_first_bound() {
    let mut reg = ProviderRegistry::new();
    reg.bind(openrouter_binding()).unwrap();

    let default = reg.default_cloud().expect("default");
    assert_eq!(default.provider, ProviderKind::OpenRouter);
}

#[test]
fn registry_default_cloud_persists_across_rebinds() {
    let mut reg = ProviderRegistry::new();
    reg.bind(openrouter_binding()).unwrap();
    reg.bind(anthropic_binding()).unwrap();

    // OpenRouter stays as default even though Anthropic bound later.
    assert_eq!(
        reg.default_cloud().unwrap().provider,
        ProviderKind::OpenRouter
    );
}

#[test]
fn registry_unbind_default_falls_to_next() {
    let mut reg = ProviderRegistry::new();
    reg.bind(openrouter_binding()).unwrap();
    reg.bind(anthropic_binding()).unwrap();
    reg.unbind(ProviderKind::OpenRouter).unwrap();

    assert_eq!(
        reg.default_cloud().unwrap().provider,
        ProviderKind::Anthropic
    );
}

// ---------------------------------------------------------------------------
// Router resolve_cloud_route path
// ---------------------------------------------------------------------------

#[test]
fn resolve_cloud_route_errors_when_no_provider_bound() {
    let config = cloud_tier_config("openrouter", "openai/gpt-4o-mini");
    let router = ModelRouter::new(config, probe());

    let err = router.resolve_cloud_route(Tier::Deep).unwrap_err();
    assert!(matches!(err, RouteError::NoCloudProvider));
}

#[test]
fn resolve_cloud_route_returns_bound_openrouter() {
    let config = cloud_tier_config("openrouter", "openai/gpt-4o-mini");
    let mut reg = ProviderRegistry::new();
    reg.bind(openrouter_binding()).unwrap();

    let router = ModelRouter::new(config, probe()).with_providers(Arc::new(reg));
    let route = router.resolve_cloud_route(Tier::Deep).expect("resolve");

    assert_eq!(route.provider, ProviderKind::OpenRouter);
    assert_eq!(route.model, "openai/gpt-4o-mini");
    assert_eq!(route.key_handle.as_str(), "sealed:openrouter");
}

#[test]
fn resolve_cloud_route_errors_when_tier_provider_not_bound() {
    // Tier is bound to anthropic but the registry has only openrouter.
    let config = cloud_tier_config("anthropic", "claude-sonnet-4.6");
    let mut reg = ProviderRegistry::new();
    reg.bind(openrouter_binding()).unwrap();

    let router = ModelRouter::new(config, probe()).with_providers(Arc::new(reg));
    let err = router.resolve_cloud_route(Tier::Deep).unwrap_err();

    assert!(matches!(
        err,
        RouteError::ProviderNotBound {
            provider: ProviderKind::Anthropic
        }
    ));
}

#[test]
fn resolve_cloud_route_errors_when_tier_names_unknown_provider() {
    let config = cloud_tier_config("google", "gemini-pro");
    let mut reg = ProviderRegistry::new();
    reg.bind(openrouter_binding()).unwrap();

    let router = ModelRouter::new(config, probe()).with_providers(Arc::new(reg));
    let err = router.resolve_cloud_route(Tier::Deep).unwrap_err();

    match err {
        RouteError::UnknownProvider(id) => assert_eq!(id, "google"),
        other => panic!("expected UnknownProvider, got {:?}", other),
    }
}

#[test]
fn resolve_cloud_route_on_local_tier_errors() {
    // Local tier binding — resolve_cloud_route should refuse.
    let config = ModelsConfig {
        tiers: TierBindings {
            fast: None,
            deep: Some(TierBinding {
                kind: BindingKind::Local,
                model_id: "qwen-deep".to_string(),
                provider: None,
                local_path: Some(std::path::PathBuf::from("/models/qwen.gguf")),
            }),
        },
        providers: HashMap::new(),
        defaults: Defaults::default(),
    };
    let mut reg = ProviderRegistry::new();
    reg.bind(openrouter_binding()).unwrap();

    let strong_probe = DeviceCapability {
        ram_gb: 32,
        cpu_cores: 16,
        has_gpu: true,
        arch: Arch::X86_64,
    };
    let router = ModelRouter::new(config, strong_probe).with_providers(Arc::new(reg));
    let err = router.resolve_cloud_route(Tier::Deep).unwrap_err();

    assert!(matches!(err, RouteError::NoCloudProvider));
}

// ---------------------------------------------------------------------------
// Sealed-key fetch
// ---------------------------------------------------------------------------

#[test]
fn resolve_and_seal_returns_key() {
    let mut reg = ProviderRegistry::new();
    reg.bind(openrouter_binding()).unwrap();

    let mut store = InMemorySealedKeyStore::new();
    store.insert(OauthKeyHandle::new("sealed:openrouter"), "sk-or-unit-test");

    let (route, key) = reg
        .resolve_and_seal(ProviderKind::OpenRouter, &store)
        .unwrap();
    assert_eq!(route.provider, ProviderKind::OpenRouter);
    assert_eq!(key, "sk-or-unit-test");
}

#[test]
fn resolve_and_seal_without_key_errors() {
    let mut reg = ProviderRegistry::new();
    reg.bind(openrouter_binding()).unwrap();
    let empty = InMemorySealedKeyStore::new();

    let err = reg
        .resolve_and_seal(ProviderKind::OpenRouter, &empty)
        .unwrap_err();
    match err {
        chief_core::router::ProviderError::ProviderKeyMissing { provider, .. } => {
            assert_eq!(provider, ProviderKind::OpenRouter);
        }
        other => panic!("expected ProviderKeyMissing, got {:?}", other),
    }
}

// ---------------------------------------------------------------------------
// Persistence — keys NEVER appear on disk
// ---------------------------------------------------------------------------

#[test]
fn persisted_registry_never_contains_key_material() {
    let tmp = tempfile::NamedTempFile::new().unwrap();
    let path = tmp.path().to_path_buf();

    let mut reg = ProviderRegistry::new();
    reg.bind(CloudBinding {
        provider: ProviderKind::OpenRouter,
        default_model: "openai/gpt-4o-mini".to_string(),
        key_handle: OauthKeyHandle::new("opaque-handle-xyz"),
        base_url: None,
    })
    .unwrap();

    // Save and re-read the raw file content.
    reg.save(&path).unwrap();
    let content = std::fs::read_to_string(&path).unwrap();

    // Standard API-key prefixes that MUST never appear: OpenAI / OpenRouter start
    // with "sk-"; Anthropic starts with "sk-ant-"; Bearer tokens would start
    // with "Bearer ". We haven't persisted any of these — this asserts it.
    assert!(
        !content.contains("sk-"),
        "api key-like string leaked: {content}"
    );
    assert!(
        !content.contains("Bearer "),
        "bearer token leaked: {content}"
    );
    assert!(
        !content.contains("OPENROUTER_API_KEY"),
        "env var name leaked: {content}"
    );

    // Handle is plaintext (by design) and model id is plaintext (public info).
    assert!(content.contains("opaque-handle-xyz"));
    assert!(content.contains("openai/gpt-4o-mini"));
}
