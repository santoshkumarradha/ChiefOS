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
//! ## v0.2 — Agent runtime (ADR-0013)
//!
//! - [`AiBuilder`] — single-shot structured LLM inference (`ctx.ai()`).
//! - [`HarnessBuilder`] — multi-turn tool-using LLM sessions (`ctx.harness()`).
//! - [`Tier`] — closed enum: Fast, Deep.
//! - [`ToolHandle`] — opaque capability handles for harness tools.
//! - [`HarnessTranscript`] — complete record of a harness session.
//!
//! ## Quick start
//!
//! ```ignore
//! use chief_sdk::prelude::*;
//!
//! #[async_trait::async_trait]
//! impl Agent for MyAgent {
//!     async fn on_tick(&self, ctx: CapabilityContext) -> Result<()> {
//!         // Single-shot classification
//!         let result = ctx.ai()
//!             .prompt("Classify this")
//!             .tier(Tier::Fast)
//!             .call::<String>()
//!             .await?;
//!
//!         // Multi-turn reasoning with tools
//!         let transcript = ctx.harness()
//!             .goal("Analyze the data")
//!             .tools(&[ctx.memory_tool(&["thought"])])
//!             .run()
//!             .await?;
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
pub mod ai;
pub mod ai_error;
pub mod capability;
pub mod connectors;
pub mod context;
pub mod error;
pub mod grant;
pub mod harness;
pub mod ingester;
pub mod manifest;
pub mod pack;
pub mod pane;
pub mod prelude;
pub mod tier;
pub mod tool;
pub mod tool_handle;

pub use ai::{AiBackend, AiBuilder};
pub use ai_error::{AiError, HarnessError};
pub use capability::CapabilityKind;
pub use context::CapabilityContext;
pub use error::{Result, SdkError};
pub use grant::Grant;
pub use harness::{
    HarnessBackend, HarnessBuilder, HarnessTranscript, Role, SignedAttestation, Turn, TurnContent,
};
pub use manifest::PackManifest;
pub use tier::Tier;
pub use tool_handle::ToolHandle;
