---
id: research-firecracker-spike
title: "Firecracker microVM Evaluation Spike"
status: review
owners: [claude-firecracker]
last_updated: 2026-04-22
tags: [sandboxing, virtualization, v1+, firecracker, research]
related: [security-model, chief-kernel]
---

# Firecracker microVM Evaluation Spike

## TL;DR

Firecracker is a **realistic v1 sandbox target** for Chief OS pack agents and the opencode engine subprocess. Published benchmarks give us ~125 ms cold-boot and ~5 ms snapshot-restore for minimal-kernel + minimal-rootfs VMs on x86_64 KVM hosts, with ~5 MB RSS overhead per VM. Steady-state I/O crosses a virtio + vsock boundary rather than a bare-namespace boundary; for pack workloads dominated by HTTP-to-broker round-trips, the added latency is a few hundred microseconds — well inside ADR-0002's "≤5 ms local" broker perf budget.

**Recommendation: proceed to prototype in v1, using snapshot-restore as the default start path and cold-boot only as a first-launch fallback.** The integration cost is non-trivial (kernel + rootfs build, snapshot lifecycle, vsock broker transport), but the security delta over `systemd-nspawn` is exactly what Axiom 2 demands for Region 7/8 actions. The spike did not attempt a working Firecracker boot because the dev host is macOS/arm64 (no KVM); all numbers in this doc are **cited from upstream and third-party publications** and flagged as such.

## Context

ADR-0014 (opencode-subprocess-boundary) §"Future (v1+)" and ADR-0002 (capability-based-security) §Consequences both mention Firecracker as the v1+ isolation layer beneath the Capability Broker. The v0 sandbox (`crates/chief-core/src/sandbox/nspawn.rs`) uses `systemd-nspawn` with `--read-only`, `--private-network`, and `--drop-capability=all`. Namespaces are the isolation primitive. That is enough for v0's threat model (trusted packs on a dev machine) but not for the v1 threat model (untrusted Verified-Community and Community packs acting on behalf of the user with Region 7/8 grants on a shared NixOS host).

This spike answers: **is Firecracker a realistic v1 target, and what should the integration shape look like?** It does **not** build a prototype — Firecracker requires a Linux host with `/dev/kvm`, and the current dev box is macOS/arm64. We rely on published numbers and document honestly what is measured versus cited.

## The questions

1. Is Firecracker a realistic v1 target for pack agents and the opencode engine subprocess, replacing or supplementing `systemd-nspawn`?
2. What is the cold-boot time for a Chief-pack-shaped microVM (minimal rootfs, single Rust or Node workload)?
3. What is the steady-state latency once the VM is running (per-tool-call overhead, vsock round-trip)?
4. What is the right cold-vs-snapshot strategy — should v1 use snapshot-restore for fast starts, or is cold boot fast enough?
5. What does the rootfs need to look like (kernel + initrd/rootfs + pack binary + chief-sdk runtime, approximate size)?
6. What are the integration points with `chief-core` (how does the Rust SDK talk to a pack running inside the microVM)?
7. Recommendation: proceed to prototype in v1, defer further, or reject?

## Cold-boot time

**Cited — not measured here.**

| Source | Workload | Cold-boot wall-time |
|---|---|---|
| Firecracker launch paper (Agache et al., NSDI 2020) | minimal initrd + `init` -> `/bin/sh` | **~125 ms** on x86_64, including VMM init + guest kernel boot to userspace |
| Firecracker README (upstream, 2025) | stock demo kernel + minimal rootfs | "boots and runs a container within ~125 ms" |
| AWS Lambda (Firecracker-backed) | production snapshot-restore | sub-second user-visible cold-start; internal VM boot is amortized |
| Weaveworks `ignite` benchmarks | minimal Alpine VM | ~200-400 ms end-to-end depending on rootfs |
| Kata Containers FC benchmark (2023) | minimal Linux + hello-world | ~150-300 ms |

**Caveats on these numbers:**

- All are x86_64 + KVM. Chief's target laptop/desktop is likely x86_64 + KVM at v1 (NixOS), but macOS/arm64 dev boxes cannot run Firecracker at all; we would need a Linux VM or CI box to measure.
- "Cold-boot" in these benchmarks usually includes VMM spawn + guest kernel boot up to the point `/init` can print something. It does not include fetching an LLM model, opening a SQLite database, or anything the pack actually does. Chief's "Firecracker cold-boot" for a pack = VM boot + chief-sdk handshake + broker vsock setup + pack main. Budget another 50-150 ms on top, honestly.
- Upstream's minimal-init setup is a busybox-based rootfs ~5 MB. A Chief-pack rootfs with our SDK + a pack binary will be larger (see §Rootfs shape); cold-boot grows with rootfs size but only marginally (kernel decompress dominates).

**Fair estimate for a Chief-pack cold-boot on x86_64 + KVM NixOS host, v1:**

- Best case (pre-warmed kernel image in page cache, 5-10 MB rootfs): **~200 ms**
- Realistic case (pack binary ~20-50 MB, first-time kernel load): **~300-500 ms**
- Worst case (cold page cache, larger rootfs with Node.js runtime for opencode): **~600-900 ms**

That is **3-10× slower than `systemd-nspawn`** for a single-shot pack invocation. For long-lived engine subprocesses (opencode session that runs many turns), the per-session cost amortizes and is negligible. For a quick `.ai()` call this overhead is unacceptable — but per ADR-0014, `.ai()` does not go through the engine subprocess, so this is fine.

## Steady-state latency

**Cited — not measured here.**

The relevant number is not throughput; it is the latency chief-core sees when it makes an authorized capability call that the pack issues from inside the microVM.

| Operation | Bare-namespace (nspawn) | Firecracker microVM | Delta |
|---|---|---|---|
| Pack-to-broker RPC | unix socket, ~30-80 µs | vsock, ~100-300 µs | +70-220 µs |
| Pack HTTP call proxied by broker | same as above for authorization round-trip | same | same |
| Pack FS read (in-VM `/workdir`) | host FS via bind mount | virtio-fs or 9p | 2-5× on metadata ops; near-native on bulk read |
| Pack malloc/compute | native | native (KVM) | 0 |

Sources: Firecracker vsock RFC + `virtiofsd` benchmarks + Kata Containers perf analyses (2023-2024).

**Bottom line:** for Chief's workload (pack agents calling the broker over RPC, broker doing the actual I/O on the host), the per-call penalty is well under the 5 ms Capability Broker budget from ADR-0002. The hot path is not pack → in-VM syscall; the hot path is pack → broker-over-vsock → host syscall. vsock adds hundreds of microseconds, not milliseconds.

The case where microVMs hurt is **workloads that do a lot of in-VM FS work**, e.g. a pack that reads a large local corpus. For those, either (a) stage the corpus into the VM image at build time, or (b) use virtio-fs with a host-side daemon and accept the 2-5× metadata overhead. This is a v1 implementation detail, not a blocker.

## Cold vs snapshot

Firecracker supports **snapshot/restore**: a running VM is paused, its memory + device state serialized to disk, and a new VM resumed from that snapshot in ~5 ms on the same host (upstream claim, NSDI 2020 paper).

Snapshot-restore is **significantly faster** than cold-boot for Chief's use case because:

1. Cold boot redoes kernel decompression, device enumeration, and `/init` every time. A Chief pack's first 200 ms is entirely uninteresting — same kernel every launch.
2. Snapshot-restore skips all of that. ~5 ms to map the memory pages in; the pack's `main` continues from where the snapshot was taken (typically right after the pack's `init()` but before it has received its first task).

**Proposed v1 strategy:**

| Scenario | Mechanism | Why |
|---|---|---|
| Pack installed; first-ever invocation | Cold boot; take a **post-init snapshot** on the way out | Pay 300-500 ms once, amortize forever |
| Subsequent invocation of the same pack version | Snapshot restore | 5-20 ms to first pack code |
| Pack version changed (upgrade or downgrade) | Cold boot → new snapshot | Snapshot is per-pack-version; invalidate on change |
| Host reboot | Existing snapshots on disk still valid (memory state frozen) | No rebuild needed |
| opencode engine subprocess | Treat engine as a pack: pre-warm one snapshot per chief-core install | opencode boots Node.js + server; snapshotting post-boot avoids that on every harness session |

**Snapshot hygiene:**

- Snapshots contain the pack's full memory. They must be stored under the same capability discipline as the pack's declared grants. A snapshot of a pack that saw a secret cannot be blindly shared across users or tenants.
- On cosign-signed pack upgrades, invalidate all snapshots for the old version.
- Periodic snapshot refresh (e.g., weekly) to pick up microcode / kernel security updates.

**The one unknown:** snapshot restore requires all the VM's memory pages to be mappable again. For a Chief pack with generous memory grants (e.g. 512 MB for a researcher pack) the snapshot file is 512 MB on disk. Multiply by ~50 installed packs = ~25 GB of snapshot blobs. Manageable, but needs a disk-budget entry in `docs/14-security-model.md`.

## Rootfs shape

A minimal Chief-pack microVM image:

```
chief-pack.img / chief-pack.rootfs/
├── boot/
│   └── (kernel loaded separately via --kernel-image-path)
├── sbin/
│   └── init                   # musl-static pack bootstrap (~200 KB)
├── lib/
│   └── (ld-musl, nothing else for Rust packs; Node for JS packs)
├── usr/
│   └── bin/
│       └── <pack-binary>      # Rust static: 5-20 MB; Node pack + node: 40-80 MB
├── dev/                       # empty, populated by kernel
├── proc/ sys/                 # empty, mounted by kernel
└── workdir/                   # empty, virtio-fs mount point for session scratch
```

**Approximate image sizes:**

| Pack flavor | Rootfs size | Notes |
|---|---|---|
| Pure Rust pack (static musl) | **8-15 MB** | No libc dynamic, no runtime |
| Rust pack with sqlite linked in | **15-25 MB** | For on-VM memory graph queries |
| Node.js pack (e.g. opencode engine) | **60-120 MB** | Node runtime + minimal `node_modules` |
| Python pack (hypothetical) | **100-200 MB** | CPython + minimal stdlib |

The **kernel** is shared across all pack VMs: one `vmlinux.bin` at `/nix/store/<hash>-chief-fc-kernel/vmlinux.bin` built from the Chief-OS NixOS flake. Target size: **~2-4 MB compressed** (Firecracker's own guest kernel config is aggressively minimal: no module support, no filesystems beyond what Chief uses, no extraneous drivers).

**Kernel config requirements:**

- virtio-net, virtio-blk, vsock (mandatory for broker transport)
- 9p or virtio-fs (for host scratch mount)
- no loadable-module support (smaller, faster boot)
- no `modprobe`, no `udev` in userspace (we control device enumeration statically)
- `CONFIG_PRINTK_TIME=y` for boot-time logging, otherwise minimal

**Build pipeline:** each Chief pack is packaged as a NixOS derivation that emits `{kernel, rootfs}` pair + a `manifest.json` listing declared capabilities. Chief's pack-manager signs the pair. The Firecracker VMM is fed `--kernel-image-path` and `--root-drive` at launch.

## Integration with chief-core

```text
                                    host (chief-core Rust process)
                              ┌───────────────────────────────────────┐
                              │  ┌─────────────────────────────────┐  │
                              │  │ Capability Broker (L2 service)  │  │
                              │  │  - grants SQLite                │  │
                              │  │  - policy.rs                    │  │
                              │  └──────────────┬──────────────────┘  │
                              │                 │ vsock CID:1         │
                              │                 │                     │
                              │  ┌──────────────▼──────────────────┐  │
                              │  │ FirecrackerSandbox              │  │
                              │  │  - spawn microVM                │  │
                              │  │  - manage snapshot              │  │
                              │  │  - vsock proxy                  │  │
                              │  └──────────────┬──────────────────┘  │
                              │                 │                     │
                              └─────────────────┼─────────────────────┘
                                                │
                                                │ KVM ioctl + vsock
                                                │
                              ┌─────────────────▼─────────────────────┐
                              │        Firecracker microVM guest       │
                              │  ┌─────────────────────────────────┐  │
                              │  │ chief-sdk runtime (in-VM)       │  │
                              │  │  - vsock RPC client to broker   │  │
                              │  │  - workdir on virtio-fs         │  │
                              │  └──────────────┬──────────────────┘  │
                              │                 │                     │
                              │  ┌──────────────▼──────────────────┐  │
                              │  │ Pack binary (Rust or Node)      │  │
                              │  └─────────────────────────────────┘  │
                              └────────────────────────────────────────┘
```

**Concrete replacement for `sandbox::spawn`:**

The current v0 module (`crates/chief-core/src/sandbox/mod.rs`) exposes:

```rust
pub async fn spawn(cmd: Command, policy: SandboxPolicy) -> Result<Child, SandboxError>;
```

At v1, add a sibling backend and extend the policy:

```rust
pub enum SandboxBackend {
    Nspawn(SandboxPolicy),          // v0 default, kept for fast local dev
    Firecracker(FirecrackerPolicy), // v1 default for pack agents & engine subprocess
}

pub struct FirecrackerPolicy {
    pub kernel_image: PathBuf,          // /nix/store/.../vmlinux.bin
    pub rootfs_image: PathBuf,          // /nix/store/.../chief-pack.rootfs
    pub snapshot: Option<SnapshotRef>,  // Some(_) -> restore; None -> cold boot
    pub memory_mib: u64,
    pub vcpus: u8,
    pub vsock_cid: u32,                 // host-side CID chief-core assigns
    pub workdir_bind: Option<PathBuf>,  // host path for virtio-fs scratch
    pub network: VmNetworkPolicy,       // None | vsock-only | tap-to-broker
}
```

**The `Child` returned is no longer a `tokio::process::Child` over a nspawn subprocess** — it becomes a `FirecrackerChild` wrapping:
- The Firecracker VMM process (still a `tokio::process::Child`).
- The vsock listener on the host side.
- A handle for graceful shutdown (`POST /actions {action_type: SendCtrlAltDel}` via the firecracker API socket, or `SIGTERM` as fallback).

**Pack → broker transport** switches from "broker is on localhost + unix socket" (v0) to **vsock with the host broker listening on CID=2 (hypervisor) and each pack VM using its assigned CID**. vsock is the right primitive here because:

1. It does not consume host port numbers (no conflicts between packs).
2. It has no network namespace — it cannot be routed to the internet, cannot be sniffed by non-host packs.
3. Latency is lower than a TAP-bridged TCP connection (no IP stack at all).
4. It is how AWS Firecracker-backed workloads (Lambda) talk to control plane; well-exercised code path.

The broker's vsock listener replaces the unix-socket listener from v0. The chief-sdk in-VM client switches from `UnixStream::connect("/run/chief/broker.sock")` to `VsockStream::connect(2, BROKER_PORT)`. All other protocol semantics (JSON-RPC, capability handles, attestation) are unchanged.

**opencode engine under Firecracker:** the opencode subprocess runs inside its own microVM. chief-core talks to its OpenAPI server over vsock instead of `127.0.0.1:<port>`. The opencode VMM image is built once per pinned opencode version (per ADR-0014's pinning discipline); snapshot-restore makes per-harness-session startup cheap.

## Security delta vs nspawn

| Threat | nspawn defense (v0) | Firecracker defense (v1) |
|---|---|---|
| Kernel exploit in pack → host kernel compromise | **None** — shared kernel, pack can attack host kernel directly if it has the syscall surface | **Strong** — guest kernel is separate; host kernel exposed to pack only through KVM ioctls, vsock, virtio-blk, virtio-fs |
| Syscall-level sandbox escape (seccomp bypass) | Possible if seccomp policy has a gap | Irrelevant — pack's syscalls go to guest kernel, not host |
| Resource-exhaustion DoS | cgroups; tunable but can starve host | VM memory cap is hard; vCPU cap is hard; no shared page cache |
| Side-channel / timing attacks | Shared page cache, shared scheduler → easier | KVM + per-VM memory → harder but not impossible (Spectre-class still exists) |
| Escape via `/proc` or `/sys` leak | nspawn tries hard but kernel occasionally leaks info | No host `/proc`, no host `/sys` visible to guest at all |
| Network exfil | netns + nftables; correct if policy is correct | Same, plus no TAP device unless policy asks → default is zero network |
| Storage write outside workdir | bind-mount discipline; regressable | virtio-blk is read-only root by default; writable only via virtio-fs mount we control |
| Teardown determinism | kill the nspawn process; some cgroups + mount namespaces may linger | VMM exit is definitive; all guest state is gone (modulo explicit snapshot we choose to keep) |

**The security delta is real and material for Region 7/8 packs.** Namespaces are tested and robust but not a trust boundary in the same class as a hypervisor. For a pack that can spend money, send email on your behalf, or ship code to production, the KVM boundary is what Axiom 2 implicitly promises.

**The cost of the delta:**

- ~300-500 ms cold-boot (mitigated by snapshot → ~5-20 ms most of the time).
- ~5 MB RSS overhead per VM.
- Additional build-system complexity (guest kernel build, rootfs build, snapshot lifecycle).
- vsock transport in both chief-core and chief-sdk (small change, well-bounded).
- Host hardware requirement: KVM available on x86_64 Linux. On arm64 Linux, Firecracker arm64 support is in-tree as of ~2022 but less exercised. On macOS, Firecracker does not run — dev loop stays on nspawn (already the v0 posture).

## Recommendation

**Proceed to prototype in v1.**

Reasoning:
1. The security delta is directly aligned with Axiom 2 and ADR-0002's "process isolation is a layer beneath the broker" posture. nspawn is genuinely weaker.
2. The performance overhead is acceptable given Chief's workload: long-lived engine subprocesses amortize VM boot; snapshot-restore makes per-invocation pack starts ~5-20 ms, which is **faster than nspawn + exec of a Rust binary** in many cases.
3. The rootfs + kernel build fits naturally into the existing NixOS flake discipline. We do not need to pick up a new build system.
4. The integration with chief-core is contained: a new `SandboxBackend::Firecracker` variant, a new vsock listener on the broker, an in-VM chief-sdk transport change. No ripple into pack-facing API.
5. Snapshot-restore is the load-bearing feature that makes this viable. If we could not snapshot, we would defer. We can, so we proceed.

**Sequencing:**

- v1.0 (pack-agent Firecracker): ship FC for Community-tier and Verified-Community packs only; Official packs still run in nspawn for the first release to reduce change surface.
- v1.1 (opencode under Firecracker): once pack-FC is stable, migrate the engine subprocess into FC; this is the harder integration because opencode is TS/Node and the rootfs is larger.
- v2 (full cutover): nspawn retained as "fast local dev" backend only; production always Firecracker.

## Open questions

1. **arm64 production target?** Several target laptops (Apple Silicon dev, and some NixOS laptops on ARM) would need arm64 Firecracker. x86_64 desktop/server NixOS is fine out of the box. Need ADR if arm64 is a v1 commitment.
2. **Snapshot storage and invalidation policy.** Where do snapshots live on disk, what is their disk-budget, when are they refreshed? Must land before v1 prototype begins.
3. **virtio-fs vs 9p for workdir**. virtio-fs is newer and performs better but requires `virtiofsd` running as a sidecar daemon. 9p is simpler but slower. Pick one; not a blocker.
4. **Broker vsock protocol framing.** Today's v0 unix-socket protocol is JSON-RPC newline-delimited. vsock supports the same; no change required — just test that framing holds across the transport.
5. **Does the Capability Broker run on the host or inside a per-pack VM?** Host is the obvious answer (single enforcement point, Axiom 2). This ADR assumes host. Confirm before prototype.
6. **Observability.** The v0 sandbox emits `tracing` spans in-process; under Firecracker, the pack's spans must cross vsock back to chief-core's tracing system. Design a vsock-native tracing forwarder for the chief-sdk.
7. **GPU passthrough.** Chief's inference stack may want GPU on the host. Current Firecracker does not support GPU passthrough. This is a **firm limitation**: inference probably has to continue to run on the host, not inside a pack VM. Packs that need GPU must call a host-side inference service through the broker, not host the model themselves. This is aligned with current chief-inference design.
8. **Cloud Hypervisor as alternative.** Cloud Hypervisor (Intel-led) is a fork-like alternative with broader device support and arm64 parity. If arm64 production is a v1 commitment, CH may be a better target than upstream Firecracker. Revisit in a follow-up spike.

## Why no prototype was built

The dev host for this spike is macOS/arm64 (`Darwin 25.2.0, arm64`). Firecracker requires Linux with `/dev/kvm`. The options were:

1. Spin up a Linux VM on macOS (UTM / Lima / Colima with KVM), install Firecracker, build a rootfs, measure cold-boot. **Estimate: 2-4 hours just to get a clean baseline.**
2. Use a remote Linux dev box over SSH and script it. **Estimate: 1-2 hours, but adds machine-ownership and SSH dependency that this spike doesn't need.**
3. **Skip the prototype, cite published benchmarks honestly.** The numbers from NSDI 2020 + upstream docs + Kata Containers' own measurements are well-validated across three independent sources. The recommendation does not hinge on a number accurate to ±20%; it hinges on "Firecracker boots in hundreds of ms, not seconds" which is uncontested.

Option 3 was selected per this task's "skip if not tractable" instruction. A follow-up task should run option 1 or 2 before the v1 prototype branch lands, so we have house-measured numbers on our actual target hardware before committing to the integration.

## References

- Agache, A. et al. "Firecracker: Lightweight Virtualization for Serverless Applications." *NSDI '20*. [usenix.org/conference/nsdi20/presentation/agache](https://www.usenix.org/conference/nsdi20/presentation/agache) — canonical cold-boot + snapshot-restore numbers.
- Firecracker upstream README: [github.com/firecracker-microvm/firecracker](https://github.com/firecracker-microvm/firecracker)
- Firecracker snapshot design: [docs/snapshotting/snapshot-support.md](https://github.com/firecracker-microvm/firecracker/blob/main/docs/snapshotting/snapshot-support.md)
- Firecracker vsock docs: [docs/vsock.md](https://github.com/firecracker-microvm/firecracker/blob/main/docs/vsock.md)
- Kata Containers perf analyses (2023–2024): [github.com/kata-containers/kata-containers/tree/main/docs/design](https://github.com/kata-containers/kata-containers/tree/main/docs/design)
- Weaveworks `ignite` (Firecracker-based container runner): [github.com/weaveworks-liquidmetal/ignite](https://github.com/weaveworks-liquidmetal/ignite)
- Cloud Hypervisor (arm64-friendly alternative, future spike): [cloud-hypervisor.org](https://www.cloudhypervisor.org/)
- ADR-0014 (opencode-subprocess-boundary) — mentions Firecracker as v1+ path
- ADR-0002 (capability-based-security) — the "process isolation as a layer" posture
- `docs/14-security-model.md` — Sandboxing v0 → v1+ table
- `crates/chief-core/src/sandbox/` — current v0 nspawn implementation
