//! Chief OS OAuth broker.
//!
//! Packs request OAuth capability through the OS, then receive an opaque
//! [`SessionHandle`]. Bearer tokens stay inside the broker and are injected
//! only after provider host and scope validation.

pub mod broker;
pub mod error;
pub mod flow;
pub mod providers;
pub mod proxy;
pub mod session;
pub mod storage;

pub use broker::OAuthBroker;
pub use error::{OAuthError, Result};
pub use flow::{AuthorizationChallenge, FlowId, OAuthClientConfig};
pub use providers::{CustomProvider, Provider};
pub use proxy::{ProxyRequest, ProxyResponse};
pub use session::{SessionHandle, SessionMeta};
pub use storage::SealedTokenStore;
