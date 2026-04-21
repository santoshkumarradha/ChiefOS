//! Agent trait — autonomous pack code that acts on behalf of the user.

use crate::context::CapabilityContext;
use crate::error::Result;
use async_trait::async_trait;

/// An agent: async code that can read/write via the capability broker.
#[async_trait]
pub trait Agent: Send + Sync {
    /// Called on a schedule or manually.
    async fn on_tick(&self, ctx: CapabilityContext) -> Result<()>;
}
