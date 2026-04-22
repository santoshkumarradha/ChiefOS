//! Sandbox policy descriptors.
//!
//! A `SandboxPolicy` is the closed-form description of everything a
//! subprocess is allowed to see and do at the OS layer. The policy is
//! constructed by the caller (harness runtime or pack loader) from the
//! subprocess's declared needs — it is not derived inside the sandbox
//! module itself. The sandbox module only translates this shape into
//! platform-specific launch flags.
//!
//! Fields are plain data (no builders) so the struct is trivially
//! serializable and comparable in tests.

use std::path::PathBuf;

/// Complete policy for one subprocess.
///
/// Semantics:
/// - `root_dir` is mounted read-only as `/` inside the sandbox.
/// - `ro_binds` are additional read-only bind mounts (e.g. pack code,
///   shared model cache).
/// - `rw_binds` are writable bind mounts (e.g. per-session scratch dir).
/// - `env` is the exact set of environment variables forwarded to the
///   child. Nothing else leaks in — the host's ambient env is never
///   inherited by the sandboxed process. Secrets must be passed here
///   explicitly and the caller must ensure they are redacted from any
///   log/debug output.
/// - `network` decides the network namespace posture.
/// - `capabilities` lists Linux kernel capability *names* (e.g.
///   `"CAP_NET_BIND_SERVICE"`) to retain inside the sandbox. Default is
///   empty — drop everything.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SandboxPolicy {
    /// Read-only root filesystem image for the sandbox. Must exist on host.
    pub root_dir: PathBuf,
    /// Extra read-only bind mounts. Host-path -> same-path inside sandbox.
    pub ro_binds: Vec<PathBuf>,
    /// Writable bind mounts (allowlist). Host-path -> same-path inside sandbox.
    pub rw_binds: Vec<PathBuf>,
    /// Environment variables passed through to the child. Ambient host env
    /// is NOT inherited.
    pub env: Vec<(String, String)>,
    /// Network namespace posture.
    pub network: NetworkPolicy,
    /// Linux kernel capabilities to retain. Empty = drop all.
    pub capabilities: Vec<String>,
}

/// Network namespace posture.
///
/// - `None`: fully private network namespace; no loopback to host.
///   Used for pure-compute agents with no net needs.
/// - `HostShared`: share the host's network namespace. Used in v0 for
///   engine subprocesses that must reach the Broker-proxied HTTP.
///   Future work will replace this with a narrower proxy namespace.
/// - `Private(hosts)`: private namespace + a userland proxy that only
///   relays to the named hosts. v1+ feature — at v0 this downgrades to
///   `HostShared` with a `tracing::warn!`, because the userland proxy
///   infrastructure does not exist yet. The variant is defined now so
///   policy-side call sites don't need to change when the proxy lands.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NetworkPolicy {
    /// No network access. `systemd-nspawn --private-network` with no veth.
    None,
    /// Share the host network namespace. No isolation at the net layer;
    /// the Capability Broker remains the authoritative gate.
    HostShared,
    /// Private namespace with userland proxy to allowlisted hosts.
    /// v1+ — at v0 falls back to `HostShared` with a warning.
    Private(Vec<String>),
}

impl SandboxPolicy {
    /// Minimal policy for a read-only sandbox with no network and no
    /// extra binds. Useful as a starting point for tests and as a
    /// defense-in-depth default in production code.
    pub fn minimal(root_dir: PathBuf) -> Self {
        Self {
            root_dir,
            ro_binds: Vec::new(),
            rw_binds: Vec::new(),
            env: Vec::new(),
            network: NetworkPolicy::None,
            capabilities: Vec::new(),
        }
    }
}
