use thiserror::Error;

pub type Result<T> = std::result::Result<T, SdkError>;

#[derive(Debug, Error)]
pub enum SdkError {
    #[error("capability denied: {kind} ({reason})")]
    CapabilityDenied { kind: &'static str, reason: String },

    #[error("invalid grant: {0}")]
    InvalidGrant(String),

    #[error("invalid manifest: {0}")]
    InvalidManifest(String),

    #[error("connector error: {0}")]
    Connector(String),

    #[error("serialization error: {0}")]
    Serialization(String),

    #[error("ai error: {0}")]
    Ai(#[from] crate::ai_error::AiError),

    #[error("harness error: {0}")]
    Harness(#[from] crate::ai_error::HarnessError),
}
