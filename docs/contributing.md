---
id: contributing
title: "Contributing"
status: draft
owners: [santosh]
last_updated: 2026-04-22
related: [security-model, base-and-hardware]
depends_on: [security-model, base-and-hardware]
tags: [contributing, ci, testing, nixos]
---

# Contributing

## TL;DR

- CI currently runs a provisional NixOS-equivalent Linux environment.
- Tests belong to one of three groups: `nixos-only`, `both`, or `mac-only`.
- Linux-only tests use `#[cfg(target_os = "linux")]` and `_nixos_only` names.
- Before opening a PR, run `nix develop` and then `cargo test`.

## Context

Chief OS is designed for NixOS on Linux.
macOS is a development host and client surface, not the base platform.
The CI pipeline therefore has one job: keep normal Rust code portable while
making Linux-only security and runtime behavior explicit.

This document defines how contributors mark tests so reviewers can tell
whether a failure is a product bug, a missing host feature, or an expected
platform boundary.

The current GitHub Actions job is provisional.
It runs on `ubuntu-latest`, installs Nix, enters the repo dev shell, and runs:

```bash
cargo build --workspace
cargo test -p chief-sdk -- --nocapture
```

That is real Linux coverage.
It is not yet a full NixOS VM boot test.
The v1 upgrade goal is to run the same suite inside a NixOS VM built from the
flake.

## Test Matrix

Use this table before adding or moving tests.

| Tier | Runs on | Purpose | Examples |
|---|---|---|---|
| `nixos-only` | Linux CI and NixOS dev shells | Kernel, sandbox, filesystem, and Linux service contracts | eBPF capability enforcement, `systemd-nspawn`, FUSE, namespaces, nftables, Wayland headers |
| `both` | Linux CI and macOS dev hosts | Portable Rust contracts that must work everywhere | SDK schema generation, serialization, pack manifest validation, pure capability data types |
| `mac-only` | macOS dev hosts only | Host integration or developer convenience that is not Chief OS base behavior | local editor scripts, macOS client-surface experiments, Apple-specific packaging checks |

Default to `both`.
Move a test to `nixos-only` only when it needs a Linux kernel feature, Linux
userspace service, or NixOS-style runtime boundary.
Move a test to `mac-only` only when it is explicitly about macOS as a client or
developer host.

## Marking Convention

Every Linux-only Rust test must be compiled only on Linux:

```rust
#[cfg(target_os = "linux")]
#[test]
fn ebpf_capability_enforcement_nixos_only() {
    // ...
}
```

Async tests use the same platform gate:

```rust
#[cfg(target_os = "linux")]
#[tokio::test]
async fn nspawn_pack_sandbox_nixos_only() {
    // ...
}
```

The function name must also end in `_nixos_only`.
The suffix is intentionally redundant with `#[cfg(target_os = "linux")]`.
The attribute controls compilation.
The suffix makes `cargo test` output, CI logs, and review comments readable.

Do not hide Linux-only behavior behind broad helper names.
Prefer:

```rust
#[cfg(target_os = "linux")]
fn fuse_overlay_mount_nixos_only() {}
```

Avoid:

```rust
fn filesystem_test() {}
```

The reviewer should not need to inspect the body to understand the platform
contract.

## Linux-Only Areas

Mark new tests as `nixos-only` when they touch any of these areas:

- eBPF programs or eBPF-backed filesystem and network enforcement.
- `systemd-nspawn`, user namespaces, network namespaces, or cgroups.
- FUSE, OverlayFS, bind mounts, mount namespaces, or Linux permission probes.
- nftables, netlink, seccomp-bpf, or low-level process sandboxing.
- Wayland compositor assumptions that require Linux headers or runtime state.
- TPM, measured boot, or boot-attestation tests that depend on Linux tooling.
- NixOS module evaluation, NixOS VM images, generations, or rollback behavior.

If a test uses one of these mechanisms indirectly through a helper, the test is
still Linux-only.
Mark the test, not just the helper.

## Portable Tests

Portable tests are the default.
Keep SDK and pack contracts in `both` unless the test needs Linux.

Good `both` candidates:

- Capability schema serialization.
- Manifest parsing.
- Stable error codes.
- Pack signing data structures.
- Provenance record hashing when no kernel service is required.
- Region routing and policy decisions with pure fixtures.

Portable tests must not depend on:

- `/proc`, `/sys`, `/run`, or Linux-only paths.
- systemd being present.
- root privileges.
- a particular shell outside the dev shell contract.
- local machine state outside the repository.

## macOS-Only Tests

Use `mac-only` sparingly.
Chief OS does not treat macOS as the base.

Valid macOS-only work includes:

- Client-surface behavior that explicitly targets macOS.
- Developer tooling that only exists for macOS contributors.
- Experiments for VM packaging on Apple Silicon.

Do not mark a product-runtime test `mac-only` because it happens to pass on a
laptop.
If it is part of Chief OS base behavior, make it run in Linux CI or mark it as a
known gap.

## Local CI

Run the same path locally before opening a PR:

