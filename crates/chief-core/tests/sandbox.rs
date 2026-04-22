//! Integration tests for the `chief_core::sandbox` module.
//!
//! The nspawn-backed tests require a NixOS-ish host with `systemd-nspawn`
//! on `PATH`, root-equivalent privileges, and a usable base image at
//! `/var/empty` or another minimal dir. They are therefore `#[ignore]`-d
//! by default and only executed in the `chief-os-ci` NixOS CI VM.
//!
//! The `policy_builder_smoke` test is platform-agnostic and runs on
//! every `cargo test` invocation, including macOS dev machines. It
//! verifies the type surface compiles and the minimal-policy helper
//! produces the expected default shape.

use std::path::PathBuf;

use chief_core::sandbox::{NetworkPolicy, SandboxPolicy};

#[test]
fn policy_builder_smoke() {
    let root = PathBuf::from("/var/empty");
    let mut policy = SandboxPolicy::minimal(root.clone());

    // Starting policy should be maximally locked down.
    assert_eq!(policy.root_dir, root);
    assert!(policy.ro_binds.is_empty());
    assert!(policy.rw_binds.is_empty());
    assert!(policy.env.is_empty());
    assert!(matches!(policy.network, NetworkPolicy::None));
    assert!(policy.capabilities.is_empty());

    // Compose a non-trivial policy and make sure fields round-trip.
    policy.ro_binds.push(PathBuf::from("/nix/store"));
    policy.rw_binds.push(PathBuf::from("/tmp/session-xyz"));
    policy.env.push(("CHIEF_SESSION".to_string(), "xyz".into()));
    policy.network = NetworkPolicy::HostShared;
    policy.capabilities.push("CAP_NET_BIND_SERVICE".to_string());

    let cloned = policy.clone();
    assert_eq!(policy, cloned);
    assert_eq!(cloned.ro_binds, vec![PathBuf::from("/nix/store")]);
    assert_eq!(cloned.rw_binds, vec![PathBuf::from("/tmp/session-xyz")]);
    assert!(matches!(cloned.network, NetworkPolicy::HostShared));
    assert_eq!(cloned.capabilities, vec!["CAP_NET_BIND_SERVICE"]);

    // The `Private` variant must be constructible even though v0
    // downgrades it at spawn-time.
    let private = NetworkPolicy::Private(vec!["api.example.com".to_string()]);
    match private {
        NetworkPolicy::Private(hosts) => assert_eq!(hosts, vec!["api.example.com"]),
        _ => panic!("Private variant did not round-trip"),
    }
}

// ---------------------------------------------------------------------------
// Linux-only, root-required integration tests. Run manually in CI:
//
//     cargo test -p chief-core --test sandbox -- --ignored
//
// These are the real assertions that the sandbox actually isolates.
// ---------------------------------------------------------------------------

#[cfg(target_os = "linux")]
mod linux {
    use super::*;
    use chief_core::sandbox::spawn;
    use std::process::Stdio;
    use tokio::process::Command;

    fn minimal_root() -> PathBuf {
        // `/var/empty` is a safe, guaranteed-read-only minimal dir on
        // NixOS. Tests that need more should bind additional RO paths.
        PathBuf::from("/var/empty")
    }

    #[tokio::test]
    #[ignore]
    async fn nspawn_runs_echo() {
        let mut policy = SandboxPolicy::minimal(minimal_root());
        // Bind the nix store RO so /bin/echo resolves via the store.
        policy.ro_binds.push(PathBuf::from("/nix/store"));
        policy.ro_binds.push(PathBuf::from("/bin"));

        let mut cmd = Command::new("/bin/echo");
        cmd.arg("hello");
        cmd.stdout(Stdio::piped());

        let mut child = spawn(cmd, policy).await.expect("spawn under nspawn");
        let out = child
            .wait_with_output()
            .await
            .expect("wait for nspawn child");
        assert!(out.status.success(), "echo should exit 0");
        let stdout = String::from_utf8_lossy(&out.stdout);
        assert!(stdout.contains("hello"), "stdout was: {stdout}");
    }

    #[tokio::test]
    #[ignore]
    async fn nspawn_denies_outside_mount() {
        // /etc/shadow is NOT bound into the sandbox. A process inside
        // must fail to read it with EPERM or ENOENT.
        let mut policy = SandboxPolicy::minimal(minimal_root());
        policy.ro_binds.push(PathBuf::from("/nix/store"));
        policy.ro_binds.push(PathBuf::from("/bin"));

        let mut cmd = Command::new("/bin/cat");
        cmd.arg("/etc/shadow");
        cmd.stderr(Stdio::piped());
        cmd.stdout(Stdio::piped());

        let mut child = spawn(cmd, policy).await.expect("spawn under nspawn");
        let out = child
            .wait_with_output()
            .await
            .expect("wait for nspawn child");
        assert!(
            !out.status.success(),
            "cat /etc/shadow should fail inside sandbox"
        );
    }

    #[tokio::test]
    #[ignore]
    async fn nspawn_denies_net_when_private() {
        // NetworkPolicy::None means --private-network. Any outbound
        // attempt should fail.
        let mut policy = SandboxPolicy::minimal(minimal_root());
        policy.ro_binds.push(PathBuf::from("/nix/store"));
        policy.ro_binds.push(PathBuf::from("/bin"));
        assert!(matches!(
            policy.network,
            chief_core::sandbox::NetworkPolicy::None
        ));

        // Try to reach 8.8.8.8 via the ping binary. We expect non-zero
        // exit because no route is reachable inside the private netns.
        let mut cmd = Command::new("/bin/ping");
        cmd.args(["-c", "1", "-W", "1", "8.8.8.8"]);
        cmd.stderr(Stdio::piped());
        cmd.stdout(Stdio::piped());

        let mut child = spawn(cmd, policy).await.expect("spawn under nspawn");
        let out = child
            .wait_with_output()
            .await
            .expect("wait for nspawn child");
        assert!(
            !out.status.success(),
            "ping should fail with NetworkPolicy::None"
        );
    }
}
