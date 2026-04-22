//! Integration tests for the chief-ebpf enforcer.
//!
//! Tests fall in two buckets:
//!
//! 1. **Cross-platform smoke** — runs on every CI target, verifies that
//!    the crate's public API compiles and that the stub (or real Linux
//!    loader) returns a well-formed error/ok on the dev path.
//!
//! 2. **Linux-only, root-required** — gated behind `#[cfg(target_os =
//!    "linux")]` AND `#[ignore]` so they do not run in `cargo test -p
//!    chief-ebpf` on a non-NixOS host. The NixOS CI step runs them via
//!    `cargo test -p chief-ebpf --ignored` inside a privileged container
//!    where `CAP_BPF` + kernel LSM-BPF are guaranteed.

use chief_ebpf::policy::{EbpfPolicy, NetTarget};
use chief_ebpf::EbpfEnforcer;

// -----------------------------------------------------------------------------
// Cross-platform
// -----------------------------------------------------------------------------

/// Verifies the public types compile and the stub load/detach returns
/// Ok on non-Linux. On Linux without the pre-built BPF artefact, load
/// fails with a specific variant — that is asserted in the loader
/// module's unit tests, so here we only assert the types line up.
#[tokio::test]
async fn policy_builder_smoke() {
    let policy = EbpfPolicy {
        denied_by_default: true,
        allowed_fs_read: vec![std::path::PathBuf::from("/tmp")],
        allowed_fs_write: vec![std::path::PathBuf::from("/tmp")],
        allowed_net_out: vec![NetTarget::exact(
            std::net::IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1)),
            8080,
        )],
    };
    // We don't call `load` here (on Linux it'd need an artefact); we
    // verify field access and `Clone` + `PartialEq` so the DTO remains
    // stable for call-sites that snapshot policies into audit.
    let p2 = policy.clone();
    assert_eq!(policy, p2);
    assert_eq!(policy.allowed_fs_read.len(), 1);
    assert_eq!(policy.allowed_net_out[0].port, Some(8080));
}

/// Pure stub path: on non-Linux, construction succeeds and detach
/// succeeds. This is the test that proves the crate can be linked into
/// every downstream dev target without forcing the whole workspace onto
/// Linux.
#[cfg(not(target_os = "linux"))]
#[tokio::test]
async fn stub_load_and_detach_succeeds() {
    let enforcer = EbpfEnforcer::load(EbpfPolicy::pass_through())
        .await
        .expect("stub load");
    enforcer.detach().await.expect("stub detach");
}

/// Stub must accept a policy update without error.
#[cfg(not(target_os = "linux"))]
#[tokio::test]
async fn stub_update_policy_hotswap() {
    let mut enforcer = EbpfEnforcer::load(EbpfPolicy::pass_through())
        .await
        .expect("load");
    enforcer
        .update_policy(EbpfPolicy::deny_all())
        .await
        .expect("update");
    enforcer.detach().await.expect("detach");
}

// -----------------------------------------------------------------------------
// Linux, root-required — gated behind #[ignore]
// -----------------------------------------------------------------------------

/// Child spawned inside an allowlist-compliant policy must be able to
/// read the permitted path.
///
/// **How to run:**
/// ```text
/// sudo -E cargo test -p chief-ebpf --test integration --ignored \
///     open_allowed_path_succeeds_under_policy
/// ```
///
/// The test assumes:
/// - A pre-built `chief_enforce` BPF object exists at the path in
///   `CHIEF_EBPF_OBJECT` (env var) or the default NixOS CI location.
/// - The process has `CAP_BPF` (or `CAP_SYS_ADMIN` on older kernels).
#[test]
#[cfg(target_os = "linux")]
#[ignore = "requires root + CAP_BPF + pre-built bpf artefact; run on NixOS CI"]
fn open_allowed_path_succeeds_under_policy() {
    // The v1 shape of this test (from docs/06-security-model.md):
    //   1. Create a tempdir T and a file T/ok.txt with content.
    //   2. Build an EbpfPolicy with denied_by_default=true,
    //      allowed_fs_read = [T].
    //   3. Load the enforcer.
    //   4. Spawn a child process (small Rust test helper) that opens
    //      T/ok.txt and reads it.
    //   5. Assert the child exits 0 and prints the expected content.
    // The full wiring lands when the bpf-linker CI step is available;
    // the `#[ignore]` gate keeps this test declared but dormant.
    unimplemented!("enable after bpf-linker CI step lands; see README § ci");
}

/// Child spawned outside the fs allowlist must get `EPERM` when it
/// attempts to read a non-allowlisted file.
#[test]
#[cfg(target_os = "linux")]
#[ignore = "requires root + CAP_BPF + pre-built bpf artefact; run on NixOS CI"]
fn open_denied_path_returns_eperm_under_policy() {
    // Shape:
    //   1. Create two tempdirs: ALLOWED and DENIED.
    //   2. Policy allows ALLOWED only.
    //   3. Child attempts to read DENIED/secret.txt.
    //   4. Assert child exit code reflects EPERM (libc::EPERM = 1).
    unimplemented!("enable after bpf-linker CI step lands; see README § ci");
}

/// Child whose destination IP:port is on the net allowlist must
/// succeed; a destination outside the allowlist must be rejected
/// (ECONNREFUSED in the prototype semantics — see the
/// `chief_enforce.rs` module docs for the v1 EPERM upgrade path).
#[test]
#[cfg(target_os = "linux")]
#[ignore = "requires root + CAP_BPF + pre-built bpf artefact; run on NixOS CI"]
fn connect_allowed_ip_succeeds_connect_denied_ip_rejected() {
    // Shape:
    //   1. Stand up two local TCP listeners: allowed_listener (127.0.0.1:A)
    //      and denied_listener (127.0.0.1:D).
    //   2. Policy: denied_by_default=true, allowed_net_out=[127.0.0.1:A].
    //   3. Spawn a child that connects to both.
    //   4. Assert: A succeeds, D fails with ECONNREFUSED.
    unimplemented!("enable after bpf-linker CI step lands; see README § ci");
}
