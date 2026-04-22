//! Provider registry — bindings from Models surface → cloud provider dispatch.
//!
//! Implements the Models surface (docs/19-controls-and-policy.md §"Models")
//! binding side: when a user binds a cloud provider via Controls → Models,
//! the CLI / UI persists a `ProviderBinding` entry here. The router consults
//! this registry when resolving `ModelChoice::Cloud { provider, model }`, and
//! `ChiefCoreAiBackend` uses it to fetch the (sealed) API key and dispatch
//! the real HTTP call to the right provider adapter.
//!
//! ## What lives where
//!
//! | File                               | Content              |
//! |------------------------------------|----------------------|
//! | `$CHIEF_HOME/providers.json`       | bindings, keyed refs — NO keys |
//! | `chief-oauth` sealed storage       | actual API keys      |
//!
//! The registry holds only references (`OauthKeyHandle`) into the sealed
//! store. API keys are NEVER persisted in plaintext, NEVER logged, and NEVER
//! travel through structured events.
//!
//! ## Test / production duality
//!
//! Keys live in a `SealedKeyStore` (trait). Production wires this to
//! `chief_oauth::storage::SealedTokenStore`-equivalent; tests can use
//! `InMemorySealedKeyStore` (a plain HashMap) or `EnvOverrideSealedKeyStore`
//! (reads `OPENROUTER_API_KEY` etc. from env for `#[ignore]`-gated live
//! integration tests only).

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use thiserror::Error;

/// Cloud providers Chief OS can dispatch to.
///
/// v0 ships with OpenRouter fully wired; Anthropic and OpenAI are recognized
/// (bindable from the UI) but their dispatch paths return
/// `ProviderDispatchError::NotYetImplemented` until their native adapters land.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderKind {
    OpenRouter,
    Anthropic,
    OpenAI,
}

impl ProviderKind {
    /// Canonical string id used when persisting to JSON.
    pub fn as_str(&self) -> &'static str {
        match self {
            ProviderKind::OpenRouter => "openrouter",
            ProviderKind::Anthropic => "anthropic",
            ProviderKind::OpenAI => "openai",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "openrouter" => Some(ProviderKind::OpenRouter),
            "anthropic" => Some(ProviderKind::Anthropic),
            "openai" => Some(ProviderKind::OpenAI),
            _ => None,
        }
    }
}

impl std::fmt::Display for ProviderKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Opaque reference to an API key stored in sealed storage.
///
/// The string itself is not the key — it is a handle that the sealed store
/// resolves to the actual key material. It can safely appear in config files,
/// log lines, and structured events.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct OauthKeyHandle(pub String);

impl OauthKeyHandle {
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// A single cloud-provider binding persisted to `providers.json`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderBinding {
    pub provider: ProviderKind,
    /// Default model id for this provider (e.g. `openai/gpt-4o-mini` for OpenRouter).
    pub default_model: String,
    /// Opaque reference into sealed storage — never the key itself.
    pub key_handle: OauthKeyHandle,
    /// Optional override base URL (e.g. a self-hosted OpenRouter proxy).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub base_url: Option<String>,
}

/// Errors surfaced when resolving or fetching provider bindings.
#[derive(Debug, Clone, Error, PartialEq, Eq)]
pub enum ProviderError {
    #[error("no cloud provider bound — user must bind one via Controls → Models")]
    NoCloudProvider,

    #[error("provider {provider} is not bound")]
    ProviderNotBound { provider: ProviderKind },

    #[error("API key missing from sealed storage for {provider} (handle {handle:?})")]
    ProviderKeyMissing {
        provider: ProviderKind,
        handle: String,
    },

    #[error("provider {provider} dispatch is not yet implemented in v0")]
    NotYetImplemented { provider: ProviderKind },

    #[error("provider registry io error: {0}")]
    Io(String),

    #[error("provider registry parse error: {0}")]
    Parse(String),
}

/// Default cloud provider preference when multiple are bound.
///
/// v0 has no UI to reorder this — the first binding added becomes default.
/// The persisted registry records the preferred provider explicitly so
/// rebinds / removes don't reshuffle dispatch unpredictably.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProviderRegistry {
    bindings: BTreeMap<String, ProviderBinding>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    default_provider: Option<ProviderKind>,
}

impl ProviderRegistry {
    /// Empty registry (in-memory). Use `load_or_empty` for file-backed.
    pub fn new() -> Self {
        Self::default()
    }

    /// Load from `providers.json`, or return an empty registry if the file
    /// does not exist. Malformed content surfaces as `ProviderError::Parse`.
    ///
    /// This is plaintext config (no keys) — keys live in sealed storage.
    pub fn load_or_empty(path: &Path) -> Result<Self, ProviderError> {
        match std::fs::read_to_string(path) {
            Ok(content) => {
                if content.trim().is_empty() {
                    return Ok(Self::default());
                }
                serde_json::from_str(&content).map_err(|e| ProviderError::Parse(e.to_string()))
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Self::default()),
            Err(e) => Err(ProviderError::Io(e.to_string())),
        }
    }

    /// Persist to `providers.json`. Creates parent directories as needed.
    pub fn save(&self, path: &Path) -> Result<(), ProviderError> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| ProviderError::Io(e.to_string()))?;
        }
        let content =
            serde_json::to_string_pretty(self).map_err(|e| ProviderError::Parse(e.to_string()))?;
        std::fs::write(path, content).map_err(|e| ProviderError::Io(e.to_string()))
    }

    /// Add or replace a provider binding. If no default is set, the first
    /// binding becomes the default.
    pub fn bind(&mut self, binding: ProviderBinding) -> Result<(), ProviderError> {
        let kind = binding.provider;
        self.bindings.insert(kind.as_str().to_string(), binding);
        if self.default_provider.is_none() {
            self.default_provider = Some(kind);
        }
        Ok(())
    }

    /// Remove a binding. If it was the default, the default falls to the
    /// next registered binding (or None if none remain).
    pub fn unbind(&mut self, kind: ProviderKind) -> Result<(), ProviderError> {
        self.bindings.remove(kind.as_str());
        if self.default_provider == Some(kind) {
            self.default_provider = self
                .bindings
                .keys()
                .next()
                .and_then(|k| ProviderKind::parse(k));
        }
        Ok(())
    }

    /// Look up a specific provider's binding.
    pub fn get(&self, kind: ProviderKind) -> Option<&ProviderBinding> {
        self.bindings.get(kind.as_str())
    }

    /// The currently-preferred cloud binding, if any.
    pub fn default_cloud(&self) -> Option<&ProviderBinding> {
        let kind = self.default_provider?;
        self.bindings.get(kind.as_str())
    }

    /// Explicitly set the default provider. Errors if it isn't bound.
    pub fn set_default(&mut self, kind: ProviderKind) -> Result<(), ProviderError> {
        if !self.bindings.contains_key(kind.as_str()) {
            return Err(ProviderError::ProviderNotBound { provider: kind });
        }
        self.default_provider = Some(kind);
        Ok(())
    }

    /// All bindings, sorted by provider id for deterministic iteration.
    pub fn bindings(&self) -> impl Iterator<Item = &ProviderBinding> {
        self.bindings.values()
    }

    /// Suggested path: `$CHIEF_HOME/providers.json`.
    pub fn default_path(chief_home: &Path) -> PathBuf {
        chief_home.join("providers.json")
    }
}

/// Sealed-storage trait for provider API keys.
///
/// Production wires this to `chief_oauth::storage::SealedTokenStore`-equivalent
/// with `XChaCha20-Poly1305` at rest. Tests use `InMemorySealedKeyStore` or
/// `EnvOverrideSealedKeyStore` (latter is `#[ignore]`-gated live-test-only).
///
/// API keys must never leak — implementors must:
///  - never write keys in plaintext to config files
///  - never include keys in `Debug`, `Display`, or structured events
///  - never return keys through error paths (return `ProviderKeyMissing` only)
pub trait SealedKeyStore: Send + Sync {
    /// Resolve a handle to an API key. `None` if the handle is unknown.
    fn get(&self, handle: &OauthKeyHandle) -> Option<String>;
}

/// Plain-memory sealed store for unit tests.
///
/// NOT production — keys are held unencrypted in a HashMap. Do not use
/// outside of tests. (In real usage the trait is wired to the
/// XChaCha20-Poly1305 sealed store in `chief-oauth`.)
#[derive(Debug, Default, Clone)]
pub struct InMemorySealedKeyStore {
    keys: std::collections::HashMap<String, String>,
}

impl InMemorySealedKeyStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, handle: OauthKeyHandle, key: impl Into<String>) {
        self.keys.insert(handle.0, key.into());
    }
}

impl SealedKeyStore for InMemorySealedKeyStore {
    fn get(&self, handle: &OauthKeyHandle) -> Option<String> {
        self.keys.get(&handle.0).cloned()
    }
}

/// Sealed-key store that reads from environment variables.
///
/// Used ONLY in `#[ignore]`-gated live integration tests where the key is
/// already in the developer's env. Maps the provider handle to an env var
/// name. Never ship a production path that reads API keys from env.
#[derive(Debug, Clone)]
pub struct EnvOverrideSealedKeyStore {
    mapping: std::collections::HashMap<String, String>,
}

impl EnvOverrideSealedKeyStore {
    pub fn new() -> Self {
        Self {
            mapping: std::collections::HashMap::new(),
        }
    }

    /// Map a handle string to an env var name.
    pub fn map(mut self, handle: OauthKeyHandle, env_var: impl Into<String>) -> Self {
        self.mapping.insert(handle.0, env_var.into());
        self
    }
}

impl Default for EnvOverrideSealedKeyStore {
    fn default() -> Self {
        Self::new()
    }
}

impl SealedKeyStore for EnvOverrideSealedKeyStore {
    fn get(&self, handle: &OauthKeyHandle) -> Option<String> {
        let env_var = self.mapping.get(&handle.0)?;
        std::env::var(env_var).ok()
    }
}

/// Resolved provider route — what the dispatcher actually needs.
///
/// Separates routing (registry lookup) from sealing (key fetch) from
/// dispatch (HTTP call) so each layer is testable in isolation.
#[derive(Debug, Clone)]
pub struct ResolvedProviderRoute {
    pub provider: ProviderKind,
    pub model: String,
    pub base_url: Option<String>,
    /// Opaque handle — caller passes this to `SealedKeyStore::get` to obtain
    /// the actual key. Never a plaintext key here.
    pub key_handle: OauthKeyHandle,
}

impl ProviderRegistry {
    /// Resolve a specific provider binding into a route, or error if not bound.
    pub fn resolve(&self, kind: ProviderKind) -> Result<ResolvedProviderRoute, ProviderError> {
        let binding = self
            .get(kind)
            .ok_or(ProviderError::ProviderNotBound { provider: kind })?;
        Ok(ResolvedProviderRoute {
            provider: binding.provider,
            model: binding.default_model.clone(),
            base_url: binding.base_url.clone(),
            key_handle: binding.key_handle.clone(),
        })
    }

    /// Resolve the default cloud provider's route. Errors with
    /// `NoCloudProvider` when no binding exists — callers should surface
    /// this directly to the user so they know to visit Controls → Models.
    pub fn resolve_default_cloud(&self) -> Result<ResolvedProviderRoute, ProviderError> {
        let binding = self.default_cloud().ok_or(ProviderError::NoCloudProvider)?;
        Ok(ResolvedProviderRoute {
            provider: binding.provider,
            model: binding.default_model.clone(),
            base_url: binding.base_url.clone(),
            key_handle: binding.key_handle.clone(),
        })
    }

    /// Resolve AND seal: returns `(route, api_key)` where the key is fetched
    /// from the sealed store. Key never touches persistent storage.
    pub fn resolve_and_seal(
        &self,
        kind: ProviderKind,
        sealed: &dyn SealedKeyStore,
    ) -> Result<(ResolvedProviderRoute, String), ProviderError> {
        let route = self.resolve(kind)?;
        let key =
            sealed
                .get(&route.key_handle)
                .ok_or_else(|| ProviderError::ProviderKeyMissing {
                    provider: route.provider,
                    handle: route.key_handle.0.clone(),
                })?;
        Ok((route, key))
    }
}

/// Shareable registry pointer used by the router / backend.
pub type SharedProviderRegistry = Arc<ProviderRegistry>;
pub type SharedSealedKeyStore = Arc<dyn SealedKeyStore>;

#[cfg(test)]
mod tests {
    use super::*;

    fn test_binding(provider: ProviderKind, model: &str, handle: &str) -> ProviderBinding {
        ProviderBinding {
            provider,
            default_model: model.to_string(),
            key_handle: OauthKeyHandle::new(handle),
            base_url: None,
        }
    }

    #[test]
    fn empty_registry_has_no_default() {
        let reg = ProviderRegistry::new();
        assert!(reg.default_cloud().is_none());
        assert!(matches!(
            reg.resolve_default_cloud(),
            Err(ProviderError::NoCloudProvider)
        ));
    }

    #[test]
    fn bind_first_becomes_default() {
        let mut reg = ProviderRegistry::new();
        reg.bind(test_binding(
            ProviderKind::OpenRouter,
            "openai/gpt-4o-mini",
            "sealed:openrouter",
        ))
        .unwrap();

        let default = reg.default_cloud().unwrap();
        assert_eq!(default.provider, ProviderKind::OpenRouter);
        assert_eq!(default.default_model, "openai/gpt-4o-mini");
    }

    #[test]
    fn bind_second_does_not_replace_default() {
        let mut reg = ProviderRegistry::new();
        reg.bind(test_binding(
            ProviderKind::OpenRouter,
            "openai/gpt-4o-mini",
            "sealed:or",
        ))
        .unwrap();
        reg.bind(test_binding(
            ProviderKind::Anthropic,
            "claude-sonnet-4.6",
            "sealed:a",
        ))
        .unwrap();

        assert_eq!(
            reg.default_cloud().unwrap().provider,
            ProviderKind::OpenRouter
        );
    }

    #[test]
    fn set_default_errors_for_unbound() {
        let mut reg = ProviderRegistry::new();
        let err = reg.set_default(ProviderKind::Anthropic).unwrap_err();
        assert!(matches!(err, ProviderError::ProviderNotBound { .. }));
    }

    #[test]
    fn unbind_default_falls_through() {
        let mut reg = ProviderRegistry::new();
        reg.bind(test_binding(
            ProviderKind::OpenRouter,
            "openai/gpt-4o-mini",
            "sealed:or",
        ))
        .unwrap();
        reg.bind(test_binding(
            ProviderKind::Anthropic,
            "claude-sonnet-4.6",
            "sealed:a",
        ))
        .unwrap();
        reg.unbind(ProviderKind::OpenRouter).unwrap();

        assert_eq!(
            reg.default_cloud().unwrap().provider,
            ProviderKind::Anthropic
        );
    }

    #[test]
    fn resolve_and_seal_happy_path() {
        let mut reg = ProviderRegistry::new();
        reg.bind(test_binding(
            ProviderKind::OpenRouter,
            "openai/gpt-4o-mini",
            "sealed:or",
        ))
        .unwrap();

        let mut store = InMemorySealedKeyStore::new();
        store.insert(OauthKeyHandle::new("sealed:or"), "sk-test-123");

        let (route, key) = reg
            .resolve_and_seal(ProviderKind::OpenRouter, &store)
            .unwrap();
        assert_eq!(route.provider, ProviderKind::OpenRouter);
        assert_eq!(route.model, "openai/gpt-4o-mini");
        assert_eq!(key, "sk-test-123");
    }

    #[test]
    fn resolve_and_seal_missing_key() {
        let mut reg = ProviderRegistry::new();
        reg.bind(test_binding(
            ProviderKind::OpenRouter,
            "openai/gpt-4o-mini",
            "sealed:missing",
        ))
        .unwrap();
        let store = InMemorySealedKeyStore::new();

        let err = reg
            .resolve_and_seal(ProviderKind::OpenRouter, &store)
            .unwrap_err();
        assert!(matches!(err, ProviderError::ProviderKeyMissing { .. }));
    }

    #[test]
    fn registry_save_load_round_trip() {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        let path = tmp.path().to_path_buf();

        let mut reg = ProviderRegistry::new();
        reg.bind(test_binding(
            ProviderKind::OpenRouter,
            "openai/gpt-4o-mini",
            "sealed:or",
        ))
        .unwrap();

        reg.save(&path).unwrap();
        let loaded = ProviderRegistry::load_or_empty(&path).unwrap();

        let binding = loaded.default_cloud().unwrap();
        assert_eq!(binding.provider, ProviderKind::OpenRouter);
        assert_eq!(binding.default_model, "openai/gpt-4o-mini");
    }

    #[test]
    fn registry_load_missing_file_is_empty() {
        let path = std::path::PathBuf::from("/tmp/nonexistent-provider-registry-test.json");
        // Ensure it really doesn't exist.
        let _ = std::fs::remove_file(&path);
        let reg = ProviderRegistry::load_or_empty(&path).unwrap();
        assert!(reg.default_cloud().is_none());
    }

    #[test]
    fn persisted_registry_contains_no_keys() {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        let path = tmp.path().to_path_buf();

        let mut reg = ProviderRegistry::new();
        reg.bind(test_binding(
            ProviderKind::OpenRouter,
            "openai/gpt-4o-mini",
            "sealed:opaque-handle",
        ))
        .unwrap();
        reg.save(&path).unwrap();

        let content = std::fs::read_to_string(&path).unwrap();
        // The handle is allowed; an API key MUST NOT appear.
        assert!(
            !content.contains("sk-"),
            "persisted registry leaked an OpenAI/OpenRouter-style API key: {}",
            content
        );
    }

    #[test]
    fn env_override_store_reads_env_var() {
        // Use a uniquely named env var so we don't collide with real keys.
        let env_var = "CHIEF_TEST_PROVIDER_WIRE_KEY_SENTINEL";
        std::env::set_var(env_var, "sk-env-test-value");

        let store =
            EnvOverrideSealedKeyStore::new().map(OauthKeyHandle::new("sealed:env-test"), env_var);

        let got = store.get(&OauthKeyHandle::new("sealed:env-test")).unwrap();
        assert_eq!(got, "sk-env-test-value");

        // Cleanup — keep other tests unaffected.
        std::env::remove_var(env_var);
    }

    #[test]
    fn provider_kind_round_trip_string() {
        for kind in [
            ProviderKind::OpenRouter,
            ProviderKind::Anthropic,
            ProviderKind::OpenAI,
        ] {
            let s = kind.as_str();
            assert_eq!(ProviderKind::parse(s), Some(kind));
        }
        assert_eq!(ProviderKind::parse("unknown"), None);
    }
}
