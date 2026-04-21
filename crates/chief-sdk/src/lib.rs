//! # Chief OS Public SDK
//!
//! This is the canonical developer contract for building Chief OS packs.
//! All packs — first-party and third-party — use only this public surface.
//!
//! **Kernel internals are NOT re-exported here.** If you find yourself needing a kernel crate,
//! that's a signal the SDK is missing a primitive (file an ADR; don't bypass).
//!
//! ## Key types
//!
//! - [`CapabilityContext`] — the ONLY object through which packs interact with the OS.
//! - [`CapabilityKind`] — closed enum of what packs can request.
//! - [`Grant`] — a pack's request for a capability with scope narrowing and usage reason.
//! - [`Agent`] — autonomous pack code that acts on a schedule.
//! - [`Pane`](pane::PaneDescriptor) — declarative UI surface.
//!
//! ## Quick start
//!
//! ```ignore
//! use chief_sdk::prelude::*;
//!
//! #[async_trait::async_trait]
//! impl Agent for MyAgent {
//!     async fn on_tick(&self, ctx: CapabilityContext) -> Result<()> {
//!         // Fetch via broker-enforced HTTP
//!         let data = ctx.net().http_get("https://example.com").await?;
//!
//!         // Write via broker-enforced Memory Graph
//!         ctx.memory().put_node(data).await?;
//!
//!         Ok(())
//!     }
//! }
//! ```
//!
//! ## Versioning
//!
//! This SDK is semver-versioned. Breaking changes require major version bumps + deprecation windows.
//! See [`adr/0010-sdk-public-api-stability.md`](../../adr/0010-sdk-public-api-stability.md).

pub mod agent;
pub mod capability;
pub mod connectors;
pub mod context;
pub mod error;
pub mod grant;
pub mod ingester;
pub mod manifest;
pub mod pack;
pub mod pane;
pub mod prelude;
pub mod tool;

pub use error::{Result, SdkError};
pub use grant::Grant;
pub use manifest::PackManifest;
