//! Subprocess supervisor for the opencode HTTP server.
//!
//! Two construction paths:
//!
//! 1. [`OpencodeSupervisor::connect`] — no child, just records the base URL of
//!    an externally-managed opencode server. Used by unit tests (the
//!    `scripted_for_tests` / `unchecked_for_tests` paths on
//!    [`super::opencode::OpencodeEngine`]) and by deployments where opencode
//!    is managed outside chief-core.
//!
//! 2. [`OpencodeSupervisor::spawn_real`] — the real path per ADR-0014. Picks a
//!    free loopback port, launches an opencode server subprocess under the
//!    platform sandbox (`systemd-nspawn` on Linux, `darwin_stub` passthrough
//!    on macOS dev), and blocks until the HTTP `/health` endpoint responds.
//!    The returned supervisor owns the child and is responsible for
//!    tear-down.
//!
//! # Drop safety
//!
//! The `Drop` impl spawns a detached tokio task that issues a SIGKILL via
//! `tokio::process::Child::start_kill`. This guarantees no zombie child
//! survives a panic or an early return from the runtime, even when the
//! caller forgets to call [`OpencodeSupervisor::shutdown`] first.
//!
//! # Graceful shutdown
//!
//! [`OpencodeSupervisor::shutdown`] issues SIGTERM (best-effort, via the
//! host `kill(1)` utility so we don't pick up a new `libc` dep), waits up
//! to `graceful_secs` for the child to reap, and then escalates to SIGKILL.

use super::traits::EngineError;
use crate::sandbox::{NetworkPolicy, SandboxError, SandboxPolicy};
use reqwest::Client;
use serde::Deserialize;
use std::net::{SocketAddr, TcpListener};
use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio::process::{Child, Command};
use tokio::time::sleep;

/// Engine HTTP wire-protocol schema version this build of chief-core speaks.
///
/// Opencode returns its implemented schema version at `GET /schema-version`.
/// A mismatch at handshake is a hard failure — per ADR-0014 we pin the
/// opencode version at each chief-core release and refuse to run against a
/// drifted server rather than risk silent protocol divergence.
pub const EXPECTED_SCHEMA_VERSION: &str = "chief-harness-v0";

/// Configuration for a real opencode subprocess.
///
/// All paths are resolved once at spawn; the supervisor does not re-read
/// them later. `opencode_binary` defaults to `opencode` (PATH lookup via
/// the spawned shell env). On NixOS CI this resolves to the flake-pinned
/// derivation; on macOS dev it resolves to whatever the user has on PATH
/// (typically `npm install -g opencode`).
#[derive(Debug, Clone)]
pub struct SupervisorConfig {
    /// Path or name of the opencode binary. Name-only values rely on
    /// PATH lookup at spawn time.
    pub opencode_binary: PathBuf,
    /// Path or name of the node binary (opencode's runtime). Same PATH
    /// semantics as `opencode_binary`. Kept explicit so the sandbox
    /// policy can bind it RO separately.
    pub node_binary: PathBuf,
    /// Workspace root that the opencode session will serve. Bound RW
    /// into the Linux sandbox; left as-is on macOS passthrough.
    pub workspace_dir: PathBuf,
    /// Max seconds to wait for the `/health` probe to succeed after
    /// spawn. Polled at 100ms backoff.
    pub timeout_ready_secs: u64,
    /// Max seconds to wait between SIGTERM and SIGKILL in
    /// [`OpencodeSupervisor::shutdown`]. Keeping this short (2s default)
    /// avoids stalls when a session ends abnormally.
    pub graceful_shutdown_secs: u64,
}

impl Default for SupervisorConfig {
    fn default() -> Self {
        Self {
            opencode_binary: PathBuf::from("opencode"),
            node_binary: PathBuf::from("node"),
            // CWD is a reasonable default for local dev loops; production
            // callers should override with the per-session scratch dir.
            workspace_dir: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            timeout_ready_secs: 5,
            graceful_shutdown_secs: 2,
        }
    }
}

/// Subprocess handle + wire-protocol endpoint.
///
/// Two shapes:
/// - `child = Some(_)` when this supervisor owns the opencode subprocess
///   (the `spawn_real` path).
/// - `child = None` when pointing at an externally managed URL (the
///   `connect` path used in tests and out-of-process deployments).
pub struct OpencodeSupervisor {
    binary: PathBuf,
    child: Option<Child>,
    base_url: String,
    graceful_shutdown_secs: u64,
}

impl OpencodeSupervisor {
    /// Point at an existing opencode server at `base_url`; do NOT launch a
    /// child process. Used by tests and by deployments that run opencode
    /// under an external supervisor.
    pub fn connect(base_url: impl Into<String>) -> Self {
        Self {
            binary: PathBuf::from("opencode"),
            child: None,
            base_url: base_url.into(),
            graceful_shutdown_secs: 2,
        }
    }

    /// Legacy simple spawn retained for parity with tests that construct
    /// a supervisor directly with a known `SocketAddr`. Does **not** route
    /// through the sandbox — prefer [`Self::spawn_real`] everywhere except
    /// targeted subprocess-supervision unit tests.
    pub async fn spawn(binary: impl Into<PathBuf>, addr: SocketAddr) -> Result<Self, EngineError> {
        let binary = binary.into();
        let child = Command::new(&binary)
            .arg("serve")
            .arg("--host")
            .arg(addr.ip().to_string())
            .arg("--port")
            .arg(addr.port().to_string())
            .env_clear()
            .env("PATH", std::env::var("PATH").unwrap_or_default())
            .spawn()
            .map_err(|err| EngineError::Subprocess(err.to_string()))?;
        Ok(Self {
            binary,
            child: Some(child),
            base_url: format!("http://{addr}"),
            graceful_shutdown_secs: 2,
        })
    }

    /// Real opencode subprocess launch per ADR-0014.
    ///
    /// Steps:
    /// 1. Ask the OS for a free loopback port by binding `:0` and reading it back.
    ///    We drop the listener before spawning, accepting the tiny TOCTOU race
    ///    (another process could grab the port in between). Retrying on bind
    ///    failure would push this over 30 LOC for a risk that is effectively
    ///    zero on a dev laptop / CI VM. Revisit if seen in practice.
    /// 2. Build `tokio::process::Command` with `serve --host 127.0.0.1 --port <p>`
    ///    and the caller-provided workspace. stdio is captured so callers can
    ///    stream it into their own logs without fighting the child's own
    ///    terminal probing.
    /// 3. On Linux, route through [`crate::sandbox::spawn`] with a policy that
    ///    mounts the nix store, node, and the workspace dir; network posture
    ///    is `HostShared` so the loopback HTTP reaches chief-core.
    ///    On macOS, the sandbox layer passthroughs the command unchanged (see
    ///    `sandbox::darwin_stub`) — a single tracing warn is emitted once per
    ///    process there.
    /// 4. Poll `GET http://127.0.0.1:$PORT/health` at 100ms intervals until
    ///    either success or the timeout; timeout yields a typed error and we
    ///    tear the child down before returning so the caller does not leak a
    ///    zombie. A mid-probe child-exit is also surfaced.
    pub async fn spawn_real(config: &SupervisorConfig, http: &Client) -> Result<Self, EngineError> {
        let port = pick_free_loopback_port()
            .map_err(|err| EngineError::Subprocess(format!("port alloc: {err}")))?;
        let addr: SocketAddr = ([127, 0, 0, 1], port).into();
        let base_url = format!("http://{addr}");

        let mut cmd = Command::new(&config.opencode_binary);
        cmd.arg("serve")
            .arg("--host")
            .arg("127.0.0.1")
            .arg("--port")
            .arg(port.to_string())
            .current_dir(&config.workspace_dir)
            // stdio: piped so operators can attach chief-core's own logger
            // to the child's output. We deliberately don't drop it on the
            // floor (inherited) — that would tangle the child's TTY detection
            // with chief-core's own stdio.
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .stdin(std::process::Stdio::null());

        let policy = build_sandbox_policy(config);

        let child = crate::sandbox::spawn(cmd, policy)
            .await
            .map_err(sandbox_err_to_engine)?;

        let mut supervisor = Self {
            binary: config.opencode_binary.clone(),
            child: Some(child),
            base_url,
            graceful_shutdown_secs: config.graceful_shutdown_secs,
        };

        // Readiness probe. If the child dies mid-probe we surface that as
        // Subprocess error, not a generic timeout, so CI logs make the
        // cause unambiguous.
        match supervisor
            .wait_ready(http, Duration::from_secs(config.timeout_ready_secs))
            .await
        {
            Ok(()) => Ok(supervisor),
            Err(err) => {
                // Tear down the child we just spawned before propagating;
                // otherwise a readiness failure leaks a zombie until Drop
                // fires, which racing with the caller's own cleanup can
                // deadlock a test runner.
                let _ = supervisor.shutdown().await;
                Err(err)
            }
        }
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    pub fn binary(&self) -> &PathBuf {
        &self.binary
    }

    /// Schema-version handshake per ADR-0014. The server's declared
    /// schema must match [`EXPECTED_SCHEMA_VERSION`] exactly; we do not
    /// do semver-range negotiation at v0 because the protocol is still
    /// evolving.
    pub async fn handshake(&self, client: &Client) -> Result<(), EngineError> {
        #[derive(Deserialize)]
        struct Handshake {
            schema_version: String,
        }

        let response = client
            .get(format!("{}/schema-version", self.base_url))
            .send()
            .await
            .map_err(|err| EngineError::Http(err.to_string()))?;
        let handshake = response
            .json::<Handshake>()
            .await
            .map_err(|err| EngineError::InvalidResponse(err.to_string()))?;
        if handshake.schema_version != EXPECTED_SCHEMA_VERSION {
            return Err(EngineError::SchemaVersionMismatch {
                expected: EXPECTED_SCHEMA_VERSION.to_string(),
                actual: handshake.schema_version,
            });
        }
        Ok(())
    }

    /// Fast liveness check: is the child still running and does its
    /// `/health` endpoint respond 2xx? Intended for integration into a
    /// future watchdog; the adapter does NOT call this in the hot path.
    pub async fn liveness_check(&mut self, client: &Client) -> Result<(), EngineError> {
        if let Some(child) = &mut self.child {
            if let Some(status) = child
                .try_wait()
                .map_err(|err| EngineError::Subprocess(err.to_string()))?
            {
                return Err(EngineError::Subprocess(format!(
                    "opencode exited with {status}"
                )));
            }
        }
        client
            .get(format!("{}/health", self.base_url))
            .send()
            .await
            .map_err(|err| EngineError::Http(err.to_string()))?
            .error_for_status()
            .map_err(|err| EngineError::Http(err.to_string()))?;
        Ok(())
    }

    /// Same shape as [`Self::liveness_check`] but immutable. Exposed so
    /// callers that only hold an `&OpencodeSupervisor` (e.g. through a
    /// `Mutex::lock().await`) can health-probe without bumping the lock
    /// to a write guard.
    pub async fn health(&self, client: &Client) -> Result<(), EngineError> {
        client
            .get(format!("{}/health", self.base_url))
            .send()
            .await
            .map_err(|err| EngineError::Http(err.to_string()))?
            .error_for_status()
            .map_err(|err| EngineError::Http(err.to_string()))?;
        Ok(())
    }

    /// Graceful SIGTERM → wait `graceful_shutdown_secs` → SIGKILL.
    ///
    /// Implemented without a new `libc`/`nix` dep: SIGTERM is issued via
    /// the host `kill(1)` utility (always present on Linux/macOS), and
    /// SIGKILL falls back to `tokio::process::Child::start_kill` which
    /// already sends SIGKILL on Unix.
    ///
    /// Idempotent: calling a second time on an already-reaped child is a
    /// no-op.
    pub async fn shutdown(&mut self) -> Result<(), EngineError> {
        let Some(child) = self.child.as_mut() else {
            return Ok(());
        };

        // Try to reap quickly if already exited.
        if let Some(_status) = child
            .try_wait()
            .map_err(|err| EngineError::Subprocess(err.to_string()))?
        {
            self.child = None;
            return Ok(());
        }

        // SIGTERM via host kill(1). Best-effort: if kill is missing or the
        // pid has already been reaped we just fall through to SIGKILL.
        if let Some(pid) = child.id() {
            let _ = std::process::Command::new("kill")
                .arg("-TERM")
                .arg(pid.to_string())
                .status();
        }

        // Wait up to graceful_shutdown_secs for the child to exit, polling
        // at 100ms. If it exits cleanly we return Ok.
        let deadline =
            tokio::time::Instant::now() + Duration::from_secs(self.graceful_shutdown_secs);
        loop {
            if let Some(_status) = child
                .try_wait()
                .map_err(|err| EngineError::Subprocess(err.to_string()))?
            {
                self.child = None;
                return Ok(());
            }
            if tokio::time::Instant::now() >= deadline {
                break;
            }
            sleep(Duration::from_millis(100)).await;
        }

        // Escalate to SIGKILL. `kill()` on tokio::process::Child awaits
        // the reap so we are guaranteed no zombie on return.
        child
            .kill()
            .await
            .map_err(|err| EngineError::Subprocess(err.to_string()))?;
        self.child = None;
        Ok(())
    }

    /// Legacy hard-kill retained for compatibility. Prefer
    /// [`Self::shutdown`] in new code.
    pub async fn kill(&mut self) -> Result<(), EngineError> {
        if let Some(child) = &mut self.child {
            child
                .kill()
                .await
                .map_err(|err| EngineError::Subprocess(err.to_string()))?;
        }
        self.child = None;
        Ok(())
    }

    /// Poll `/health` at 100ms intervals until it succeeds or `timeout`
    /// elapses. Separate from [`Self::health`] because it also watches
    /// the child for early exit — "opencode refused to start" and
    /// "opencode is still booting" need different error surfaces.
    async fn wait_ready(&mut self, client: &Client, timeout: Duration) -> Result<(), EngineError> {
        let deadline = tokio::time::Instant::now() + timeout;
        let probe_url = format!("{}/health", self.base_url);

        loop {
            // Early-exit detection: if the child is gone, don't keep polling.
            if let Some(child) = &mut self.child {
                if let Some(status) = child
                    .try_wait()
                    .map_err(|err| EngineError::Subprocess(err.to_string()))?
                {
                    return Err(EngineError::Subprocess(format!(
                        "opencode exited during startup with {status}"
                    )));
                }
            }

            // Cheap HEAD-style GET; opencode /health is a flat 200 OK.
            if let Ok(resp) = client.get(&probe_url).send().await {
                if resp.status().is_success() {
                    return Ok(());
                }
            }

            if tokio::time::Instant::now() >= deadline {
                return Err(EngineError::Subprocess(format!(
                    "opencode did not become ready within {}s at {}",
                    timeout.as_secs(),
                    self.base_url
                )));
            }
            sleep(Duration::from_millis(100)).await;
        }
    }
}

impl Drop for OpencodeSupervisor {
    /// Detach a best-effort SIGKILL task so a panic or early return from
    /// the runtime can never leak a zombie opencode child. We cannot
    /// `.await` here; the detached task runs on whatever tokio runtime
    /// is active when the drop fires. If no runtime is active (rare —
    /// non-tokio test harness), the child leaks for the remainder of
    /// the process lifetime; that is accepted as a debug-only footgun
    /// rather than a production risk.
    fn drop(&mut self) {
        let Some(mut child) = self.child.take() else {
            return;
        };
        // We already consumed the child; if it's already reaped this is
        // a no-op. If it's still running this sends SIGKILL (Unix) /
        // TerminateProcess (Windows, not a supported target here).
        if let Ok(handle) = tokio::runtime::Handle::try_current() {
            handle.spawn(async move {
                let _ = child.start_kill();
                // Best-effort reap; if try_wait is Pending we let the OS
                // deal with it rather than blocking the executor.
                let _ = child.wait().await;
            });
        } else {
            // No runtime — fall back to a synchronous start_kill. Child
            // becomes a zombie until the parent exits; acceptable given
            // this path only fires in non-async contexts.
            let _ = child.start_kill();
        }
    }
}

/// Build the sandbox policy for an opencode subprocess.
///
/// Rationale per ADR-0014:
/// - RO root at `/` so basic system libs resolve on Linux.
/// - RO bind of the nix store so node/opencode derivations resolve.
/// - RO bind of the node + opencode binaries' parent dirs (best-effort).
/// - RW bind of the workspace so opencode can read source and write
///   its scratch state.
/// - `NetworkPolicy::HostShared`: v0 concession. opencode's loopback
///   HTTP server must be reachable by chief-core on the same host, and
///   we have not yet built the userland proxy that would let us pin
///   an `allow-loopback-only` policy. The Capability Broker remains
///   the real gate; this is defense-in-depth, not the primary boundary.
/// - Drop all kernel capabilities; opencode is a pure userland JS runtime.
fn build_sandbox_policy(config: &SupervisorConfig) -> SandboxPolicy {
    let mut policy = SandboxPolicy::minimal(PathBuf::from("/"));

    // Nix store: where the node + opencode derivations live on NixOS CI.
    let nix_store = Path::new("/nix/store");
    if nix_store.is_dir() {
        policy.ro_binds.push(nix_store.to_path_buf());
    }

    // Best-effort binary parent binds. If the caller passed a name
    // (`opencode`, `node`) rather than an absolute path, we can't bind a
    // parent — PATH lookup inside the sandbox will pick up from /nix/store.
    for bin in [&config.opencode_binary, &config.node_binary] {
        if bin.is_absolute() {
            if let Some(parent) = bin.parent() {
                if parent.is_dir() && !policy.ro_binds.iter().any(|p| p == parent) {
                    policy.ro_binds.push(parent.to_path_buf());
                }
            }
        }
    }

    // Workspace: opencode writes under this; must be RW.
    if config.workspace_dir.is_dir() {
        policy.rw_binds.push(config.workspace_dir.clone());
    }

    policy.network = NetworkPolicy::HostShared;

    // Forward a minimal env. NODE_ENV=production silences dev-mode chatter;
    // HOME lets opencode locate its config without blowing up on a missing
    // home dir inside the sandbox.
    policy.env.push(("NODE_ENV".into(), "production".into()));
    if let Ok(home) = std::env::var("HOME") {
        policy.env.push(("HOME".into(), home));
    }
    // Keep the child's PATH narrow: nix profile bins + nix store wrappers.
    // If the host has a custom PATH we do NOT forward it — we don't want
    // surprise binaries resolving differently inside vs outside the sandbox.
    policy.env.push((
        "PATH".into(),
        "/run/current-system/sw/bin:/usr/bin:/bin".into(),
    ));

    policy
}

/// Ask the OS for a free loopback port. We bind `127.0.0.1:0`, read
/// the assigned port, and drop the listener. There is a tiny TOCTOU
/// window before the child rebinds — accepted for v0 per the spawn-path
/// comment.
fn pick_free_loopback_port() -> std::io::Result<u16> {
    let listener = TcpListener::bind("127.0.0.1:0")?;
    let port = listener.local_addr()?.port();
    drop(listener);
    Ok(port)
}

/// Normalize sandbox errors into the harness engine error vocabulary.
/// Keeping this conversion local avoids leaking the sandbox module into
/// the engine trait.
fn sandbox_err_to_engine(err: SandboxError) -> EngineError {
    EngineError::Subprocess(format!("sandbox: {err}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_has_sane_defaults() {
        let cfg = SupervisorConfig::default();
        assert_eq!(cfg.opencode_binary, PathBuf::from("opencode"));
        assert_eq!(cfg.node_binary, PathBuf::from("node"));
        assert_eq!(cfg.timeout_ready_secs, 5);
        assert_eq!(cfg.graceful_shutdown_secs, 2);
    }

    #[test]
    fn pick_free_loopback_port_returns_nonzero() {
        let p = pick_free_loopback_port().expect("allocate port");
        assert!(p > 0);
    }

    #[test]
    fn sandbox_policy_for_spawn_real_is_read_only_with_workspace_rw() {
        let tmp = tempfile::tempdir().unwrap();
        let cfg = SupervisorConfig {
            opencode_binary: PathBuf::from("opencode"),
            node_binary: PathBuf::from("node"),
            workspace_dir: tmp.path().to_path_buf(),
            timeout_ready_secs: 1,
            graceful_shutdown_secs: 1,
        };
        let policy = build_sandbox_policy(&cfg);
        // Workspace must be in rw_binds so opencode can write.
        assert!(policy.rw_binds.iter().any(|p| p == tmp.path()));
        // Network posture must be HostShared — see policy doc above.
        assert!(matches!(policy.network, NetworkPolicy::HostShared));
        // No kernel capabilities retained.
        assert!(policy.capabilities.is_empty());
        // PATH is forwarded explicitly, not inherited from host.
        assert!(policy.env.iter().any(|(k, _)| k == "PATH"));
    }

    #[tokio::test]
    async fn shutdown_on_connect_only_supervisor_is_noop() {
        // A `connect`-constructed supervisor owns no child. shutdown
        // must not surprise-spawn anything or fail.
        let mut sup = OpencodeSupervisor::connect("http://127.0.0.1:1");
        sup.shutdown()
            .await
            .expect("shutdown on no-child supervisor");
    }
}
