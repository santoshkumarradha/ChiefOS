//! Pack trait — lifecycle hooks for pack initialization.

use crate::error::Result;
use async_trait::async_trait;

/// A pack's lifecycle hooks.
#[async_trait]
pub trait Pack: Send + Sync {
    /// Called when the pack is initialized.
    async fn on_init(&self) -> Result<()> {
        Ok(())
    }

    /// Called when the pack is enabled (user grants permissions or manually enables).
    async fn on_enable(&self) -> Result<()> {
        Ok(())
    }

    /// Called when the pack is disabled.
    async fn on_disable(&self) -> Result<()> {
        Ok(())
    }
}
