//! OAuthBroker: the core OAuth session management and token brokering service.
//!
//! The broker holds all tokens in sealed storage. Packs receive opaque
//! SessionHandle values. Only the broker can attach bearer tokens to
//! outbound requests (via proxy_request).

use crate::error::{OAuthError, Result};
use crate::flow::{AuthorizationChallenge, FlowId, PendingFlow};
use crate::providers::Provider;
use crate::proxy::{validate_scope, ProxyRequest, ProxyResponse};
use crate::session::{SessionHandle, SessionMeta, TokenRecord};
use crate::storage::SealedTokenStore;
use base64::{engine::general_purpose, Engine as _};
use chrono::Utc;
use rand::Rng;
use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex};

pub struct OAuthBroker {
    storage: SealedTokenStore,
    pending_flows: Arc<Mutex<HashMap<FlowId, PendingFlow>>>,
    sessions: Arc<Mutex<HashMap<String, SessionMeta>>>,
}

impl OAuthBroker {
    /// Create a new OAuthBroker with sealed token storage.
    pub async fn new(keystore_path: &Path) -> Result<Self> {
        let storage = SealedTokenStore::new(keystore_path)?;
        Ok(OAuthBroker {
            storage,
            pending_flows: Arc::new(Mutex::new(HashMap::new())),
            sessions: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    /// Start an OAuth authorization flow.
    ///
    /// Returns an AuthorizationChallenge containing the auth URL and state.
    /// The OS will render the login ceremony and return the authorization code.
    pub async fn start_flow(
        &self,
        provider: Provider,
        scopes: &[String],
        redirect_uri: &str,
    ) -> Result<AuthorizationChallenge> {
        let scopes = scopes.to_vec();

        // Generate state
        let mut rng = rand::thread_rng();
        let state_bytes: [u8; 32] = rng.gen();
        let state = general_purpose::URL_SAFE_NO_PAD.encode(&state_bytes);

        let flow = PendingFlow::new(provider.clone(), scopes, state.clone(), None);
        let flow_id = flow.flow_id.clone();

        // Build authorization URL (simplified; real implementation uses oauth2 crate)
        let auth_url = match provider {
            Provider::Google => {
                format!(
                    "https://accounts.google.com/oauth/authorize?client_id=&redirect_uri={}&scope={}&state={}",
                    urlencoding::encode(redirect_uri),
                    urlencoding::encode(&flow.scopes.join(" ")),
                    &state
                )
            },
            Provider::Github => {
                format!(
                    "https://github.com/login/oauth?client_id=&redirect_uri={}&scope={}&state={}",
                    urlencoding::encode(redirect_uri),
                    urlencoding::encode(&flow.scopes.join(" ")),
                    &state
                )
            },
            Provider::Custom { auth_url, .. } => {
                return Err(OAuthError::InvalidRequest(
                    "Custom provider auth URL must be used directly".to_string(),
                ))
            }
        };

        let mut flows = self
            .pending_flows
            .lock()
            .map_err(|_| OAuthError::Internal("flow lock poisoned".to_string()))?;
        flows.insert(flow_id.clone(), flow);

        Ok(AuthorizationChallenge::new(flow_id, auth_url, state))
    }

    /// Complete an OAuth authorization flow and obtain a session.
    ///
    /// The OS provides the authorization code and state.
    /// We exchange the code for a token, store it sealed, and return a SessionHandle.
    pub async fn complete_flow(
        &self,
        flow_id: FlowId,
        code: &str,
        state: &str,
    ) -> Result<SessionHandle> {
        let mut flows = self
            .pending_flows
            .lock()
            .map_err(|_| OAuthError::Internal("flow lock poisoned".to_string()))?;
        let flow = flows
            .remove(&flow_id)
            .ok_or_else(|| OAuthError::FlowError("flow not found or expired".to_string()))?;

        if flow.state != state {
            return Err(OAuthError::FlowError("state mismatch".to_string()));
        }

        if flow.is_expired() {
            return Err(OAuthError::FlowError("flow expired".to_string()));
        }

        // In a real implementation, we'd use the oauth2 crate to exchange the code for a token.
        // For now, we'll create a mock token.
        let session_id = format!("sess_{}", flow_id.0);
        let access_token = format!("access_token_for_{}", session_id);
        let refresh_token = Some(format!("refresh_token_for_{}", session_id));

        let token_record = TokenRecord {
            session_id: session_id.clone(),
            access_token,
            refresh_token,
            token_type: "Bearer".to_string(),
            expires_in: Some(3600),
            scope: flow.scopes.join(" "),
        };

        // Seal the token in storage
        self.storage.seal(&token_record)?;

        // Store session metadata
        let handle = SessionHandle::new(session_id.clone(), flow.provider.clone(), flow.scopes);
        let mut sessions = self
            .sessions
            .lock()
            .map_err(|_| OAuthError::Internal("session lock poisoned".to_string()))?;
        sessions.insert(
            session_id,
            SessionMeta {
                handle: handle.clone(),
                created_at: Utc::now(),
                last_used: None,
                expires_at: None,
            },
        );

        Ok(handle)
    }

    /// Proxy an HTTP request with the session's bearer token.
    ///
    /// Validates scope, injects bearer token server-side, executes the request.
    pub async fn proxy_request(
        &self,
        session: &SessionHandle,
        request: ProxyRequest,
    ) -> Result<ProxyResponse> {
        // Validate scope before proxying
        validate_scope(session, &request.url)?;

        // Retrieve the sealed token (in a real broker, this would be cached)
        let _token_record = self.storage.unseal()?;

        // In a real implementation, we'd attach the bearer token and execute the request.
        // For testing, we return a mock response.
        Ok(ProxyResponse {
            status: 200,
            headers: HashMap::new(),
            body: vec![],
        })
    }

    /// Revoke a session (clear the stored token).
    pub async fn revoke(&self, session: &SessionHandle) -> Result<()> {
        let mut sessions = self
            .sessions
            .lock()
            .map_err(|_| OAuthError::Internal("session lock poisoned".to_string()))?;
        sessions.remove(&session.id);

        // Clear sealed storage
        self.storage.clear()?;

        Ok(())
    }

    /// List all active sessions and their metadata.
    pub async fn list(&self) -> Result<Vec<SessionMeta>> {
        let sessions = self
            .sessions
            .lock()
            .map_err(|_| OAuthError::Internal("session lock poisoned".to_string()))?;
        Ok(sessions.values().cloned().collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn broker_creation() {
        let tmpdir = TempDir::new().unwrap();
        let broker = OAuthBroker::new(tmpdir.path()).await.unwrap();
        assert_eq!(broker.list().await.unwrap().len(), 0);
    }

    #[tokio::test]
    async fn start_flow() {
        let tmpdir = TempDir::new().unwrap();
        let broker = OAuthBroker::new(tmpdir.path()).await.unwrap();

        let challenge = broker
            .start_flow(
                Provider::Google,
                &["gmail.readonly".to_string()],
                "http://localhost:8080/callback",
            )
            .await
            .unwrap();

        assert!(challenge.auth_url.contains("accounts.google.com"));
        assert!(!challenge.state.is_empty());
    }
}
