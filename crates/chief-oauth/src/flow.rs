//! OAuth authorization flow management.

use crate::providers::Provider;
use base64::{engine::general_purpose, Engine as _};
use rand::Rng;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct FlowId(pub String);

impl FlowId {
    pub fn new() -> Self {
        let mut rng = rand::thread_rng();
        let random_bytes: [u8; 16] = rng.gen();
        let encoded = general_purpose::URL_SAFE_NO_PAD.encode(&random_bytes);
        FlowId(format!("flow_{}", encoded))
    }
}

impl Default for FlowId {
    fn default() -> Self {
        Self::new()
    }
}

/// Challenge returned to the OS for the OAuth login ceremony.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorizationChallenge {
    pub flow_id: FlowId,
    pub auth_url: String,
    pub state: String,
}

impl AuthorizationChallenge {
    pub fn new(flow_id: FlowId, auth_url: String, state: String) -> Self {
        AuthorizationChallenge {
            flow_id,
            auth_url,
            state,
        }
    }
}

/// In-flight OAuth flow state (kept in memory by the broker).
#[derive(Debug, Clone)]
pub struct PendingFlow {
    pub flow_id: FlowId,
    pub provider: Provider,
    pub scopes: Vec<String>,
    pub state: String,
    pub pkce_verifier: Option<String>,
    pub created_at: std::time::SystemTime,
}

impl PendingFlow {
    pub fn new(
        provider: Provider,
        scopes: Vec<String>,
        state: String,
        pkce_verifier: Option<String>,
    ) -> Self {
        PendingFlow {
            flow_id: FlowId::new(),
            provider,
            scopes,
            state,
            pkce_verifier,
            created_at: std::time::SystemTime::now(),
        }
    }

    /// Check if flow has expired (default: 10 minutes).
    pub fn is_expired(&self) -> bool {
        self.created_at
            .elapsed()
            .unwrap_or_default()
            .as_secs() > 600
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flow_id_generates() {
        let f1 = FlowId::new();
        let f2 = FlowId::new();
        assert_ne!(f1.0, f2.0);
        assert!(f1.0.starts_with("flow_"));
    }

    #[test]
    fn pending_flow_not_expired_immediately() {
        let flow = PendingFlow::new(
            Provider::Google,
            vec!["gmail.readonly".to_string()],
            "state_xyz".to_string(),
            None,
        );
        assert!(!flow.is_expired());
    }
}
