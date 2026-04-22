//! Policy descriptors for eBPF enforcement.
//!
//! An [`EbpfPolicy`] is the closed-form description of what filesystem
//! paths and network endpoints a confined subprocess is allowed to touch.
//! Like `SandboxPolicy` in `chief-core`, it is plain data so it can be
//! constructed from a pack manifest or capability grant, compared in
//! tests, and serialized for audit.
//!
//! ## Design notes
//!
//! - All fields are `pub` — this is a prototype DTO, not an abstract
//!   interface. Constructing one is just struct-literal.
//! - We accept `PathBuf`s for fs allowlists and defer canonicalization
//!   to the loader. The loader normalizes (resolve symlinks, strip
//!   trailing slashes) before pushing to BPF maps so that path prefix
//!   comparison in-kernel is correct.
//! - Network allowlist entries are typed tuples — no CIDR, no DNS. The
//!   prototype enforces the narrowest useful check: exact IPv4 + optional
//!   port. CIDR / v6 / hostname resolution are v1 extensions.
//! - `denied_by_default = true` is the only setting that satisfies the
//!   security model (Axiom 2: no ambient authority). `false` is provided
//!   *only* so development/integration tests can assert that the loader
//!   and BPF program can be attached without enforcement getting in the
//!   way. Production call-sites should pin `true`.

use std::net::IpAddr;
use std::path::PathBuf;

/// Complete enforcement policy for one confined subprocess / pack agent.
///
/// Fields are ordered from highest-level (mode) to most specific
/// (concrete allowlists).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EbpfPolicy {
    /// When `true`, anything not explicitly allowed is denied.
    /// When `false`, the program loads and attaches but every check
    /// returns "allow". Used for the dev/test smoke path; production must
    /// always set `true`.
    pub denied_by_default: bool,

    /// Absolute paths (or path prefixes) that the confined process may
    /// open for *reading*. A path is allowed if it starts with any of
    /// these prefixes after canonicalization.
    pub allowed_fs_read: Vec<PathBuf>,

    /// Absolute paths (or path prefixes) that the confined process may
    /// open for *writing or creation*. Semantics match `allowed_fs_read`.
    pub allowed_fs_write: Vec<PathBuf>,

    /// Network destinations the confined process may `connect(2)` to.
    /// Checked after DNS resolution (the kprobe sees the socketaddr,
    /// not the hostname), so callers must resolve before policy load.
    pub allowed_net_out: Vec<NetTarget>,
}

impl EbpfPolicy {
    /// Deny-all policy. Useful as a baseline that callers layer
    /// allowances on top of.
    pub fn deny_all() -> Self {
        Self {
            denied_by_default: true,
            allowed_fs_read: Vec::new(),
            allowed_fs_write: Vec::new(),
            allowed_net_out: Vec::new(),
        }
    }

    /// Pass-through policy for dev/test. Every check returns "allow".
    /// Not acceptable in production.
    pub fn pass_through() -> Self {
        Self {
            denied_by_default: false,
            allowed_fs_read: Vec::new(),
            allowed_fs_write: Vec::new(),
            allowed_net_out: Vec::new(),
        }
    }
}

/// A single network-out allowlist entry.
///
/// `port = None` means "any destination port on this IP". Callers that
/// need a narrower match should enumerate explicit `(ip, Some(port))`
/// entries.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NetTarget {
    pub ip: IpAddr,
    pub port: Option<u16>,
}

impl NetTarget {
    pub fn exact(ip: IpAddr, port: u16) -> Self {
        Self {
            ip,
            port: Some(port),
        }
    }

    pub fn any_port(ip: IpAddr) -> Self {
        Self { ip, port: None }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;

    #[test]
    fn deny_all_is_denied_by_default() {
        let p = EbpfPolicy::deny_all();
        assert!(p.denied_by_default);
        assert!(p.allowed_fs_read.is_empty());
        assert!(p.allowed_fs_write.is_empty());
        assert!(p.allowed_net_out.is_empty());
    }

    #[test]
    fn pass_through_is_permissive() {
        let p = EbpfPolicy::pass_through();
        assert!(!p.denied_by_default);
    }

    #[test]
    fn net_target_constructors() {
        let ip = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
        let exact = NetTarget::exact(ip, 8080);
        assert_eq!(exact.port, Some(8080));
        let any = NetTarget::any_port(ip);
        assert!(any.port.is_none());
    }

    #[test]
    fn policy_is_comparable() {
        // Struct derives `Eq` so two identical policies compare equal —
        // this is load-bearing for map-diff update logic in the loader.
        let a = EbpfPolicy::deny_all();
        let b = EbpfPolicy::deny_all();
        assert_eq!(a, b);
    }
}
