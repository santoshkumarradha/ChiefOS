//! eBPF program source for Chief OS fs + net enforcement.
//!
//! ## IMPORTANT — this file is NOT compiled as part of the `chief-ebpf`
//! userland library.
//!
//! It targets `bpfel-unknown-none` and is built separately by the NixOS
//! CI step (see `README.md`). The file lives in-tree so the userland
//! loader, the policy types, and the BPF programs evolve together in
//! the same PR/commit. To stop rustc from ever trying to compile this
//! file during a normal `cargo build -p chief-ebpf`, the entire body is
//! guarded behind `#[cfg(target_arch = "bpf")]`.
//!
//! ## What this file declares
//!
//! 1. Two BPF maps, written by userland:
//!    - `ALLOWED_FS_READ`  — array-of-strings style (length-prefixed bytes)
//!    - `ALLOWED_NET_OUT`  — hashmap keyed on `(ipv4, port)` tuples
//! 2. Two programs:
//!    - `chief_fs_open_lsm` attaches to the `file_open` LSM hook. Reads
//!      the path of the `struct file`, compares it against
//!      `ALLOWED_FS_READ`, and returns `-EPERM` on mismatch when
//!      `DENIED_BY_DEFAULT` is set.
//!    - `chief_net_connect_kprobe` attaches to `__sys_connect`. Reads
//!      the destination sockaddr from the syscall args and looks up
//!      `(ip, port)` in `ALLOWED_NET_OUT`. On mismatch, it overwrites
//!      the syscall's sockaddr with an unreachable sentinel so the
//!      syscall returns `ECONNREFUSED` — this is the prototype
//!      equivalent of "deny"; a v1 implementation returns -EPERM via an
//!      LSM hook on `socket_connect` instead.
//!
//! ## Build
//!
//! ```text
//! # On a Linux NixOS CI runner with `bpf-linker` available:
//! cargo install bpf-linker
//! cargo rustc \
//!     --manifest-path prototypes/chief-ebpf/Cargo.toml \
//!     --target bpfel-unknown-none \
//!     --release \
//!     -- -C link-arg=--emit=obj
//! # produces target/bpfel-unknown-none/release/chief_enforce.o
//! ```
//!
//! The userland loader (`loader.rs`) then loads that `.o` at runtime.
//!
//! ## Why this is a Rust file (not .bpf.c)
//!
//! aya's eBPF toolchain is pure Rust. Keeping the program in Rust lets
//! us share types and constants with userland (through a shared
//! `#[repr(C)]` header module in a future refactor) and avoids pulling
//! in `libbpf` / `clang` C toolchain deps. If in practice the aya-ebpf
//! hook coverage for LSM is insufficient on the target kernel, the
//! fallback is to rewrite this file as `chief_enforce.bpf.c` and switch
//! the userland dep to `libbpf-rs` — see README § "loader choice".

#![cfg(target_arch = "bpf")]
#![no_std]
#![no_main]
// These are placeholder declarations. The real wiring lives in the
// follow-on task (see README). The shape below documents the intended
// map + program surface so CI can build-test the eBPF half as soon as
// `bpf-linker` is wired into the NixOS image. Everything inside the
// `cfg(target_arch = "bpf")` guard is invisible on macOS/x86_64, so
// `cargo build -p chief-ebpf` ignores this file entirely.

// Intentionally left minimal until the bpf-linker CI step lands. The
// cfg guard above means this file is effectively blank for userland
// builds — all we need for v0 is the loader + policy + stub.
