//! macOS sandbox stub.
//!
//! Per ADR-0002, macOS is an explicit v0 development concession: we run
//! subprocesses **unsandboxed** on Darwin because `systemd-nspawn` is a
//! systemd feature and macOS has no equivalent we want to bet on today
//! (the native `sandbox_init(3)` API is deprecated, the `sandbox-exec`
//! CLI is deprecated, and Apple has not published a replacement).
//!
//! The Capability Broker at L2 remains the authoritative security
//! boundary on all platforms — the sandbox is defense-in-depth, not the
//! primary gate. So running without nspawn on macOS does not weaken the
//! per-action enforcement story; it only weakens the "escape the
//! process" containment story. For a local dev loop that is an
//! accepted trade-off.
//!
//! The warning is emitted **once per process** so developers see it
//! early but it does not spam every test run. A `std::sync::Once`
//! latches on the first `spawn` call.

use std::sync::Once;

use tokio::process::{Child, Command};

use super::policy::SandboxPolicy;
use super::SandboxError;

static WARN_ONCE: Once = Once::new();

pub async fn spawn(mut cmd: Command, policy: SandboxPolicy) -> Result<Child, SandboxError> {
    WARN_ONCE.call_once(|| {
        tracing::warn!("sandbox: macOS dev mode, running unsandboxed per ADR-0002 v0 concession");
    });

    // Even on macOS we honour the policy's env-forwarding contract:
    // `policy.env` is the complete environment for the child. Clearing
    // ambient env keeps behaviour consistent with the Linux path so
    // regressions surface on dev machines, not only in CI.
    cmd.env_clear();
    for (k, v) in &policy.env {
        cmd.env(k, v);
    }

    let child = cmd.spawn().map_err(SandboxError::from)?;
    Ok(child)
}
