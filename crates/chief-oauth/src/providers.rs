//! OAuth provider definitions and coarse provider allowlists.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Provider {
    Google,
    Github,
    Custom(CustomProvider),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CustomProvider {
    pub id: String,
    pub auth_url: String,
    pub token_url: String,
    pub revoke_url: Option<String>,
    pub allowed_hosts: Vec<String>,
}

impl Provider {
    pub fn key(&self) -> String {
        match self {
            Self::Google => "google".to_string(),
            Self::Github => "github".to_string(),
            Self::Custom(provider) => format!("custom:{}", provider.id),
        }
    }

    pub fn display_name(&self) -> &str {
        match self {
            Self::Google => "Google",
            Self::Github => "GitHub",
            Self::Custom(provider) => provider.id.as_str(),
        }
    }

    pub fn auth_url(&self) -> &str {
        match self {
            Self::Google => "https://accounts.google.com/o/oauth2/v2/auth",
            Self::Github => "https://github.com/login/oauth/authorize",
            Self::Custom(provider) => provider.auth_url.as_str(),
        }
    }

    pub fn token_url(&self) -> &str {
        match self {
            Self::Google => "https://oauth2.googleapis.com/token",
            Self::Github => "https://github.com/login/oauth/access_token",
            Self::Custom(provider) => provider.token_url.as_str(),
        }
    }

    pub fn revoke_url(&self) -> Option<&str> {
        match self {
            Self::Google => Some("https://oauth2.googleapis.com/revoke"),
            Self::Github => None,
            Self::Custom(provider) => provider.revoke_url.as_deref(),
        }
    }

    pub fn allows_host(&self, host: &str) -> bool {
        match self {
            Self::Google => host == "googleapis.com" || host.ends_with(".googleapis.com"),
            Self::Github => host == "api.github.com",
            Self::Custom(provider) => provider
                .allowed_hosts
                .iter()
                .any(|allowed| host == allowed || host.ends_with(&format!(".{allowed}"))),
        }
    }

    /// Short string identifier for error messages, logs, and event records.
    pub fn name(&self) -> String {
        self.key()
    }

    /// Coarse host allowlist used by `proxy_request` to enforce scope before
    /// attaching the bearer token. Packs cannot widen this list — only the
    /// provider config controls it.
    pub fn allowed_hosts(&self) -> Vec<String> {
        match self {
            Self::Google => vec![
                "googleapis.com".to_string(),
                "www.googleapis.com".to_string(),
                "gmail.googleapis.com".to_string(),
                "calendar.googleapis.com".to_string(),
                "people.googleapis.com".to_string(),
                "oauth2.googleapis.com".to_string(),
            ],
            Self::Github => vec!["api.github.com".to_string(), "github.com".to_string()],
            Self::Custom(provider) => provider.allowed_hosts.clone(),
        }
    }
}
