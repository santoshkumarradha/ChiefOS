//! Grant definitions — what a pack requests and what the OS enforces.

use crate::capability::CapabilityKind;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A grant: the pack requests a capability kind with typed scope and a usage reason.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Grant {
    /// What the pack needs (mem.read, net.http, etc).
    pub kind: CapabilityKind,

    /// Why the pack needs this, in user-readable terms.
    /// MUST be non-empty. Shown at install-time consent and in the Packs pane forever.
    pub usage_reason: String,

    /// When this grant expires, if ever.
    pub expires_at: Option<DateTime<Utc>>,
}

impl Grant {
    /// Create a new grant with validation.
    /// usage_reason must be non-empty.
    pub fn new(
        kind: CapabilityKind,
        usage_reason: impl Into<String>,
    ) -> crate::error::Result<Self> {
        let reason = usage_reason.into();
        if reason.trim().is_empty() {
            return Err(crate::error::SdkError::InvalidGrant(
                "usage_reason must not be empty".to_string(),
            ));
        }

        Ok(Grant {
            kind,
            usage_reason: reason,
            expires_at: None,
        })
    }

    /// Set expiration on this grant.
    pub fn with_expiration(mut self, expires_at: DateTime<Utc>) -> Self {
        self.expires_at = Some(expires_at);
        self
    }
}
