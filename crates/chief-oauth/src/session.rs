//! Opaque session handle types returned to packs.

use crate::providers::Provider;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionHandle {
    pub id: String,
    pub provider: Provider,
    pub scopes: Vec<String>,
    pub issued_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}

impl SessionHandle {
    pub(crate) fn new(
        id: String,
        provider: Provider,
        scopes: Vec<String>,
        issued_at: DateTime<Utc>,
        expires_at: Option<DateTime<Utc>>,
    ) -> Self {
        Self { id, provider, scopes, issued_at, expires_at }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionMeta {
    pub handle: SessionHandle,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub revoked_at: Option<DateTime<Utc>>,
}
