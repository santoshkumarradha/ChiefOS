//! Chief OS eBPF enforcement prototype.
//!
//! This crate is the **defense-in-depth** layer that sits BELOW
//! `systemd-nspawn` (see `crates/chief-core/src/sandbox/`). Its job is to
//! ensure that the filesystem- and network-capability decisions made by the
//! Capability Broker are enforced by the kernel even if the userland
//! sandbox (`nspawn`, namespace masks, seccomp) is misconfigured or
//! bypassed.
//!
//! Per `docs/06-security-model.md`, acceptance gate:
//!
//! > eBPF enforces fs + net caps even if nspawn/namespace is misconfigured.
//!
//! ## Shape
//!
//! - [`policy::EbpfPolicy`] — plain-data description of what a confined
//!   subprocess is allowed to open on the filesystem and which
//!   `(ip, port)` network endpoints it is allowed to connect to.
//! - [`loader::EbpfEnforcer`] — the runtime object that
//!   loads a bpf program, pushes the policy into BPF maps, and attaches
//!   the program to the appropriate hooks.
//! - The eBPF program source (`src/ebpf/chief_enforce.rs`) is referenced
//!   but **not compiled as part of this crate's userland build**. It
//!   targets `bpfel-unknown-none` and is compiled separately on Linux CI
//!   via `bpf-linker`. See `README.md` for the build incantation. The
//!   userland loader loads the pre-built object file at runtime.
//!
//! ## Hooks
//!
//! | Subsystem | Hook         | Kind    | Semantics |
//! |-----------|--------------|---------|-----------|
//! | fs        | `file_open`  | LSM     | Returns `-EPERM` when the path does not start with an allowlisted prefix. |
//! | net       | `__sys_connect` | kprobe | Returns a rejection when the destination `(ip, port)` is outside the allowlist. |
//!
//! ## Platform split
//!
//! - **Linux**: real loader using `aya` (pure Rust, no `libbpf` C deps).
//! - **macOS / other**: no-op stub with a `tracing::warn!` on construction.
//!   The stub keeps dev-on-macOS unblocked; it also keeps the crate's
//!   public API identical across platforms so downstream code can depend
//!   on `chief-ebpf` unconditionally.
//!
//! ## Limitations (v0)
//!
//! - The LSM hook covers `file_open` only; `openat2`-via-io_uring and
//!   memory-mapped path mutations are not yet covered.
//! - The kprobe on `__sys_connect` covers IPv4 TCP only; IPv6, UDP, and
//!   raw sockets are deferred.
//! - Policy is pushed once at attach time; hot-swap via
//!   [`loader::EbpfEnforcer::update_policy`] is supported but assumes a
//!   single-writer contract.
//! - No CO-RE portability: programs are compiled for the kernel running
//!   on the NixOS CI image. Porting to an arbitrary host kernel is
//!   explicitly deferred.

pub mod loader;
pub mod policy;

pub use loader::EbpfEnforcer;
pub use policy::{EbpfPolicy, NetTarget};
