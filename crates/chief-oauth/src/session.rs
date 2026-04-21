//! Opaque session handle types returned to packs.
//!
//! The bearer token NEVER appears in any type that crosses the API boundary
//! to a pack. Packs only ever hold a `SessionHandle`; the broker holds the
//! corresponding `TokenRecord` in sealed storage.

use crate::providers::Provider;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Opaque handle returned to packs. Contains no bearer token.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionHandle {
    pub id: String,
    pub provider: Provider,
    pub scopes: Vec<String>,
    pub issued_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}

impl SessionHandle {
    /// Construct a new handle with minimal inputs; issued_at defaults to now,
    /// expires_at is left None (set by the broker when it knows the token TTL).
    pub fn new(id: String, provider: Provider, scopes: Vec<String>) -> Self {
        Self {
            id,
            provider,
            scopes,
            issued_at: Utc::now(),
            expires_at: None,
        }
    }

    /// Construct with explicit issued_at / expires_at (used by persistence
    /// reloading).
    #[allow(dead_code)]
    pub(crate) fn with_timestamps(
        id: String,
        provider: Provider,
        scopes: Vec<String>,
        issued_at: DateTime<Utc>,
        expires_at: Option<DateTime<Utc>>,
    ) -> Self {
        Self {
            id,
            provider,
            scopes,
            issued_at,
            expires_at,
        }
    }
}

/// Broker-side metadata about a session. Not exposed to packs as-is; packs
/// only ever see the `handle` field.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionMeta {
    pub handle: SessionHandle,
    pub created_at: DateTime<Utc>,
    /// Most recent time the broker issued a `proxy_request` on behalf of this
    /// session. `None` until first use.
    pub last_used: Option<DateTime<Utc>>,
    /// When the underlying bearer token expires, if known. `None` for
    /// non-expiring tokens or when expiry is not advertised by the provider.
    pub expires_at: Option<DateTime<Utc>>,
    pub updated_at: DateTime<Utc>,
    pub revoked_at: Option<DateTime<Utc>>,
}

/// Broker-internal token record. Held only in sealed storage; never exposed
/// to packs. The bearer token lives here and nowhere else.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TokenRecord {
    pub session_id: String,
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub token_type: String,
    /// Token lifetime in seconds, as advertised by the provider.
    pub expires_in: Option<u64>,
    /// Space-separated scope string from the token response.
    pub scope: String,
}
