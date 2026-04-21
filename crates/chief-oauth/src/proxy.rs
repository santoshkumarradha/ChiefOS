//! Request proxying with scope enforcement and bearer injection.

use crate::error::{OAuthError, Result};
use crate::session::SessionHandle;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use url::Url;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyRequest {
    pub url: String,
    pub method: String,
    pub headers: HashMap<String, String>,
    pub body: Option<Vec<u8>>,
}

#[derive(Debug, Clone)]
pub struct ProxyResponse {
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
}

/// Validate that a request URL is within the session's authorized scopes.
pub fn validate_scope(session: &SessionHandle, request_url: &str) -> Result<()> {
    let parsed_url = Url::parse(request_url)
        .map_err(|e| OAuthError::InvalidRequest(format!("invalid URL: {}", e)))?;

    let host = parsed_url
        .host_str()
        .ok_or_else(|| OAuthError::InvalidRequest("URL missing host".to_string()))?;

    let allowed_hosts = session.provider.allowed_hosts();

    // Check if the request host matches any of the provider's allowed hosts
    let is_allowed = allowed_hosts.iter().any(|pattern| {
        if pattern.starts_with("*.") {
            host.ends_with(&pattern[1..])
        } else {
            host.ends_with(pattern) || host == pattern
        }
    });

    if !is_allowed && !allowed_hosts.is_empty() {
        return Err(OAuthError::ScopeDenied {
            provider: session.provider.name().to_string(),
            requested_host: host.to_string(),
        });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::providers::Provider;

    #[test]
    fn scope_validation_google_api() {
        let session = SessionHandle::new(
            "sess_test".to_string(),
            Provider::Google,
            vec!["gmail.readonly".to_string()],
        );

        assert!(validate_scope(&session, "https://www.googleapis.com/gmail/v1/users/me").is_ok());
        assert!(validate_scope(&session, "https://googleapis.com/calendar/v3").is_ok());
    }

    #[test]
    fn scope_validation_reject_wrong_host() {
        let session = SessionHandle::new(
            "sess_test".to_string(),
            Provider::Google,
            vec!["gmail.readonly".to_string()],
        );

        let result = validate_scope(&session, "https://evil.example.com/api");
        assert!(result.is_err());
        match result {
            Err(OAuthError::ScopeDenied { .. }) => {}
            _ => panic!("expected ScopeDenied error"),
        }
    }

    #[test]
    fn scope_validation_github() {
        let session = SessionHandle::new(
            "sess_gh".to_string(),
            Provider::Github,
            vec!["repo".to_string()],
        );

        assert!(validate_scope(&session, "https://api.github.com/user/repos").is_ok());
        assert!(validate_scope(&session, "https://github.com/user").is_ok());
    }
}
