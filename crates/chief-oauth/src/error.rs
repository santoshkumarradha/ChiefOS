//! Error types for the OAuth broker.

use thiserror::Error;

pub type Result<T> = std::result::Result<T, OAuthError>;

#[derive(Debug, Error)]
pub enum OAuthError {
    #[error("provider config missing for {provider}")]
    ProviderConfigMissing { provider: String },
    #[error("unknown OAuth flow")]
    UnknownFlow,
    #[error("unknown OAuth session")]
    UnknownSession,
    #[error("capability denied: {reason}")]
    CapabilityDenied { reason: String },
    #[error("invalid request: {0}")]
    InvalidRequest(String),
    #[error("flow error: {0}")]
    FlowError(String),
    #[error("scope denied: provider={provider} host={requested_host}")]
    ScopeDenied {
        provider: String,
        requested_host: String,
    },
    #[error("storage error: {0}")]
    StorageError(String),
    #[error("encryption error: {0}")]
    EncryptionError(String),
    #[error("internal error: {0}")]
    Internal(String),
    #[error("token endpoint rejected request: {message}")]
    TokenEndpoint { message: String },
    #[error("invalid sealed token store: {reason}")]
    InvalidStore { reason: String },
    #[error("crypto failure: {reason}")]
    Crypto { reason: String },
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("url parse error: {0}")]
    Url(#[from] url::ParseError),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("invalid header value for {name}")]
    InvalidHeader { name: String },
}
