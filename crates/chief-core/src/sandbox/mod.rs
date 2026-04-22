//! Sandbox wrapper for engine and pack agent subprocesses.
//!
//! The sandbox is a defense-in-depth layer beneath the Capability Broker
//! (see ADR-0002, ADR-0014). The Broker performs semantic authorization on
//! every tool call; the sandbox confines a subprocess to an allowlisted view
//! of the filesystem and network so that even a totally compromised engine
//! or pack agent cannot reach the broader host.
//!
//! Two consumer patterns:
//! 1. **Engine subprocess** (opencode, custom engine). Read-only root with
//!    allowlisted RW mounts for a session scratch directory; network
//!    namespace controlled by policy.
//! 2. **Pack agent subprocess**. Same shape, but the policy is derived
//!    from the pack's declared capabilities rather than a fixed template.
//!
//! ## Platform split
//!
//! - **Linux**: real sandbox via `systemd-nspawn` (`nspawn` module).
//! - **macOS**: v0 dev concession per ADR-0002: passthrough to
//!   `tokio::process::Command`. A single `tracing::warn!` is emitted on the
//!   first spawn of each process so operators can see that the dev machine
//!   is not enforcing isolation.
//! - Other OS targets produce a compile error — we do not want to silently
//!   skip isolation on an unknown platform.

mod policy;

#[cfg(target_os = "linux")]
mod nspawn;

#[cfg(target_os = "macos")]
mod darwin_stub;

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
compile_error!("chief-core sandbox module only supports linux and macos targets");

pub use policy::{NetworkPolicy, SandboxPolicy};

use std::io;
use thiserror::Error;
use tokio::process::{Child, Command};

/// Errors surfaced by the sandbox layer.
///
/// These are distinct from generic `io::Error`s so callers can decide
/// whether to retry (transient spawn failure) vs. fail fast (binary missing
/// on the host image — CI misconfiguration).
#[derive(Debug, Error)]
pub enum SandboxError {
    /// `systemd-nspawn` (or equivalent) is not present on the host.
    /// Treat as a host-image bug, not a runtime condition.
    #[error("sandbox backend binary not found: {0}")]
    BinaryNotFound(String),

    /// Policy was malformed — e.g. a bind path that does not exist on host.
    #[error("sandbox policy invalid: {0}")]
    InvalidPolicy(String),

    /// Spawning the child failed (fork/exec level error).
    #[error("sandbox spawn failed: {0}")]
    Spawn(#[from] io::Error),
}

/// Spawn `cmd` under the platform sandbox configured by `policy`.
///
/// The returned `Child` is a standard `tokio::process::Child`. The caller
/// is responsible for waiting on it, streaming its stdio, and propagating
/// exit status. The sandbox takes no position on those — it only wraps
/// the launch.
pub async fn spawn(cmd: Command, policy: SandboxPolicy) -> Result<Child, SandboxError> {
    #[cfg(target_os = "linux")]
    {
        nspawn::spawn(cmd, policy).await
    }
    #[cfg(target_os = "macos")]
    {
        darwin_stub::spawn(cmd, policy).await
    }
}
