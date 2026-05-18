//! Linux sandbox implementation backed by `systemd-nspawn`.
//!
//! We assemble `systemd-nspawn` CLI flags from the `SandboxPolicy` and
//! exec the requested command inside the container via `--`. The
//! `tokio::process::Command` the caller passes in is decomposed —
//! program, args, current dir, env — and each part is re-plumbed through
//! the nspawn invocation so the child actually sees what the caller
//! intended (not nspawn's own defaults).
//!
//! # Security posture
//!
//! - Root FS is read-only (`--read-only`).
//! - Host ambient env is NOT inherited; only `policy.env` is forwarded.
//! - Kernel capabilities are dropped by default; retained only if the
//!   policy explicitly lists them.
//! - Network namespace is chosen by `NetworkPolicy`:
//!     - `None`          → `--private-network`
//!     - `HostShared`    → no network flag (host netns inherited)
//!     - `Private(...)`  → `--private-network` and a `warn!` that the
//!       userland proxy is not yet implemented; callers wanting the
//!       proxy must check for v1+ explicitly.
//! - `systemd-nspawn` binary presence is verified up front; a missing
//!   binary is surfaced as `SandboxError::BinaryNotFound` so CI can
//!   distinguish "host image bug" from "transient spawn failure."
//!
//! # Debug logging
//!
//! The final argv is emitted at `tracing::debug!`. We deliberately do
//! NOT log the env vector, which may contain secrets passed through
//! `policy.env`. Callers that want to audit env should do so at their
//! own call site with explicit redaction.

use std::ffi::OsString;
use std::path::Path;

use tokio::process::{Child, Command};

use super::policy::{NetworkPolicy, SandboxPolicy};
use super::SandboxError;

/// Absolute path to the nspawn binary on NixOS / typical Linux distros.
/// We look for it by name on `PATH` via `which`-style probe, but keep
/// this constant as the canonical fallback for logging / errors.
const NSPAWN_BINARY: &str = "systemd-nspawn";

/// Verify `systemd-nspawn` is reachable on `PATH`. Returns the resolved
/// path on success. Called once per spawn — cheap enough.
fn probe_nspawn() -> Result<OsString, SandboxError> {
    // `Command::new("systemd-nspawn").arg("--version")` would actually
    // fork; we want a cheaper check. Walk PATH manually.
    if let Some(path) = std::env::var_os("PATH") {
        for dir in std::env::split_paths(&path) {
            let candidate = dir.join(NSPAWN_BINARY);
            if candidate.is_file() {
                return Ok(candidate.into_os_string());
            }
        }
    }
    Err(SandboxError::BinaryNotFound(NSPAWN_BINARY.to_string()))
}

/// Validate host paths in the policy exist. nspawn will reject the mount
/// with a cryptic error if they don't; catching it up front gives a
/// clearer failure mode.
fn validate_paths(policy: &SandboxPolicy) -> Result<(), SandboxError> {
    if !policy.root_dir.is_dir() {
        return Err(SandboxError::InvalidPolicy(format!(
            "root_dir does not exist or is not a directory: {}",
            policy.root_dir.display()
        )));
    }
    for p in policy.ro_binds.iter().chain(policy.rw_binds.iter()) {
        if !p.exists() {
            return Err(SandboxError::InvalidPolicy(format!(
                "bind mount path does not exist: {}",
                p.display()
            )));
        }
    }
    Ok(())
}

/// Format a `--bind=SRC` / `--bind-ro=SRC` argument. nspawn accepts
/// `SRC[:DEST[:OPTS]]`; we always pass `SRC:SRC` so the mount point
/// inside the sandbox matches the host path 1:1. This keeps policy
/// authoring trivial — what you list is what the child sees.
fn bind_arg(flag: &str, path: &Path) -> OsString {
    let mut arg = OsString::from(flag);
    arg.push("=");
    arg.push(path.as_os_str());
    arg.push(":");
    arg.push(path.as_os_str());
    arg
}

/// Build the argv for the nspawn invocation, excluding the binary itself.
/// Returned as `Vec<OsString>` so paths with non-utf8 bytes survive.
fn build_nspawn_args(
    policy: &SandboxPolicy,
    child_cmd: &Command,
) -> Result<Vec<OsString>, SandboxError> {
    let mut args: Vec<OsString> = Vec::new();

    // Quiet nspawn's own chatter; the caller drives their own logging.
    args.push(OsString::from("--quiet"));

    // Read-only root.
    let mut dir_arg = OsString::from("--directory=");
    dir_arg.push(policy.root_dir.as_os_str());
    args.push(dir_arg);
    args.push(OsString::from("--read-only"));

    // No host /etc leaking in (hostname, machine-id, resolv.conf).
    args.push(OsString::from("--register=no"));
    args.push(OsString::from("--keep-unit"));

    // Bind mounts.
    for p in &policy.ro_binds {
        args.push(bind_arg("--bind-ro", p));
    }
    for p in &policy.rw_binds {
        args.push(bind_arg("--bind", p));
    }

    // Network namespace.
    match &policy.network {
        NetworkPolicy::None => args.push(OsString::from("--private-network")),
        NetworkPolicy::HostShared => {
            // Intentionally no flag — nspawn default when neither
            // --private-network nor --network-veth is given is to share
            // the host netns. Comment here rather than an arg keeps the
            // argv minimal and greppable.
        }
        NetworkPolicy::Private(hosts) => {
            tracing::warn!(
                hosts = ?hosts,
                "sandbox: NetworkPolicy::Private requested but userland proxy is not implemented at v0; downgrading to --private-network (no outbound). v1+ will add the proxy."
            );
            args.push(OsString::from("--private-network"));
        }
    }

    // Capabilities: drop all by default, retain listed ones.
    if policy.capabilities.is_empty() {
        args.push(OsString::from("--drop-capability=all"));
    } else {
        // nspawn expects a comma-separated list passed to --capability.
        let joined = policy.capabilities.join(",");
        let mut cap_arg = OsString::from("--capability=");
        cap_arg.push(joined);
        args.push(cap_arg);
    }

    // Forward policy env using nspawn's `--setenv=K=V` flag. Ambient
    // host env is NOT passed — nspawn provides a near-empty env to the
    // child by default.
    for (k, v) in &policy.env {
        if k.contains('=') {
            return Err(SandboxError::InvalidPolicy(format!(
                "env key must not contain '=': {k}"
            )));
        }
        let mut setenv = OsString::from("--setenv=");
        setenv.push(k);
        setenv.push("=");
        setenv.push(v);
        args.push(setenv);
    }

    // If the caller set a working directory, propagate it via --chdir.
    // nspawn's `--chdir=` expects an absolute path inside the sandbox;
    // since our mounts are 1:1, host==sandbox paths line up.
    if let Some(dir) = child_cmd.as_std().get_current_dir() {
        let mut cd = OsString::from("--chdir=");
        cd.push(dir.as_os_str());
        args.push(cd);
    }

    // End of nspawn's own flags; next token is the program to execute.
    args.push(OsString::from("--"));
    args.push(child_cmd.as_std().get_program().to_os_string());
    for a in child_cmd.as_std().get_args() {
        args.push(a.to_os_string());
    }

    Ok(args)
}

/// Spawn `cmd` under `systemd-nspawn` with the given `policy`.
pub async fn spawn(cmd: Command, policy: SandboxPolicy) -> Result<Child, SandboxError> {
    let nspawn_path = probe_nspawn()?;
    validate_paths(&policy)?;

    let args = build_nspawn_args(&policy, &cmd)?;

    tracing::debug!(
        nspawn = ?nspawn_path,
        args = ?args,
        "sandbox: launching systemd-nspawn"
    );

    let mut launcher = Command::new(&nspawn_path);
    launcher.args(&args);
    // Inherit stdio by default; caller may reconfigure via the returned Child.
    // We deliberately do NOT forward the caller's env here — policy.env is the
    // one and only source of truth. nspawn itself needs no env to run.
    launcher.env_clear();

    let child = launcher.spawn().map_err(SandboxError::from)?;
    Ok(child)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn tmp_policy() -> SandboxPolicy {
        // Use `/` as the root — on Linux hosts this will validate as a
        // directory. We don't actually spawn in these unit tests.
        SandboxPolicy::minimal(PathBuf::from("/"))
    }

    #[test]
    fn build_args_has_read_only_and_drop_all() {
        let policy = tmp_policy();
        let cmd = Command::new("/bin/echo");
        let args = build_nspawn_args(&policy, &cmd).unwrap();
        assert!(args.iter().any(|a| a == "--read-only"));
        assert!(args.iter().any(|a| a == "--drop-capability=all"));
        assert!(args.iter().any(|a| a == "--private-network"));
    }

    #[test]
    fn build_args_host_shared_network_emits_no_net_flag() {
        let mut policy = tmp_policy();
        policy.network = NetworkPolicy::HostShared;
        let cmd = Command::new("/bin/echo");
        let args = build_nspawn_args(&policy, &cmd).unwrap();
        assert!(!args.iter().any(|a| a == "--private-network"));
    }

    #[test]
    fn build_args_env_rejects_equals_in_key() {
        let mut policy = tmp_policy();
        policy.env.push(("BAD=KEY".to_string(), "v".to_string()));
        let cmd = Command::new("/bin/echo");
        let err = build_nspawn_args(&policy, &cmd).unwrap_err();
        assert!(matches!(err, SandboxError::InvalidPolicy(_)));
    }

    #[test]
    fn build_args_retains_capabilities_when_listed() {
        let mut policy = tmp_policy();
        policy.capabilities.push("CAP_NET_BIND_SERVICE".to_string());
        let cmd = Command::new("/bin/echo");
        let args = build_nspawn_args(&policy, &cmd).unwrap();
        assert!(args
            .iter()
            .any(|a| a == "--capability=CAP_NET_BIND_SERVICE"));
        assert!(!args.iter().any(|a| a == "--drop-capability=all"));
    }
}
