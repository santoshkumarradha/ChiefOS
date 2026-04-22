---
id: base-and-hardware
title: "Base & Hardware"
status: draft
owners: [santosh]
last_updated: 2026-04-21
related: [architecture, security-model, local-vs-cloud]
depends_on: [architecture]
tags: [nixos, linux, hardware, deployment]
---

# Base & Hardware

## TL;DR

- **Base: NixOS on Linux.** Chosen over Fuchsia, seL4, FreeBSD, illumos, custom.
- Wayland, not X11. microVMs (Firecracker) at v1+, systemd-nspawn at v0.
- Three deploy targets from one flake: bare-metal ISO, macOS-host VM image, bootable USB.
- TPM / Secure Enclave mandatory. YubiKey optional.
- Boot target: power-on → Morning Brief in ≤ 10 seconds.

## Base choice — scored

| Base | Ecosystem | Cap-security (native) | Reproducibility | Rollback | Ship-in-90d | Notes |
|---|---|---|---|---|---|---|
| **NixOS / Linux** | 5 | 3 (add via broker + eBPF) | **5** | **5** | **4** | Our pick. Radical userland on boring infra. |
| Fuchsia | 1 | 5 | 3 | 3 | 1 | Google-owned; tiny ecosystem; HW support limited. |
| seL4 | 1 | 5 | 3 | 2 | 1 | Formally verified; no ecosystem. v5+ research. |
| FreeBSD + Capsicum | 3 | 4 | 3 | 3 | 2 | Capsicum is lovely; base niche. |
| illumos (Zones) | 2 | 4 | 3 | 3 | 2 | Great isolation; tiny ecosystem. |
| Custom microkernel | 0 | 5 | 5 | 5 | 0 | Founder-suicide. |

**Why NixOS wins, mapped to charter:**

| Axiom | NixOS answer |
|---|---|
| 3 Provenance | Nix store content-addressed by construction; reproducible builds |
| 5 Reversibility | Generations = atomic system rollback |
| 7 Boring infra | Linux works on everything; HW support is real |
| 10 HAX-as-enforcement | Flakes = declarative policy; user's house rules ARE a flake |

**Cap-security gap closed at L2** via our Capability Broker + eBPF enforcement on network + fs for high-risk categories. See [`security-model`](./14-security-model.md).

## macOS considered and rejected

- Sandbox model fights us; no way to share state between apps cleanly.
- No immutable root; no atomic rollback equivalent to Nix generations.
- Apple can sandbox/ban us at any moment; trust floor cannot be ours if Apple controls the floor below.
- macOS *can* be a **client** — a phone/laptop/watch surface consuming Chief OS state. It cannot be the base.

## Wayland over X11

| Dim | X11 | Wayland |
|---|---|---|
| Per-client surface isolation | No (any client can screenshot/keylog any other) | Yes |
| Security model age | 1984 | 2008+ |
| HW acceleration | Legacy driver maze | Modern |
| Agent-compatible | Dangerous (universal snooping) | Safe |

Non-negotiable for agent-native. X11 apps can still run in isolated XWayland sandboxes for legacy packs, but the default surface compositor is Wayland.

## Sandboxing (v0 → v1+)

| v0 | v1+ |
|---|---|
| `systemd-nspawn` + user namespace | Firecracker microVM per agent |
| OverlayFS per-pack | virtio-fs per microVM |
| Per-pack netns + nftables allowlist | microVM + vsock to broker only |
| seccomp-bpf allowlist | seccomp + microVM boundary |

Why microVMs (v1+): container escape = total compromise. Firecracker boots in ≤125ms, memory-light, agent-lifetime-shaped. Pack-level isolation becomes hardware-level.

## Deploy targets (all from one Nix flake)

```nix
outputs.chief-os.{
  packages.x86_64-linux.iso            # bare-metal installer
  packages.x86_64-linux.vm             # QCOW2 for macOS/Linux hosts (OrbStack, UTM, QEMU)
  packages.x86_64-linux.usb            # bootable live USB image
  packages.aarch64-linux.{iso,vm,usb}  # ARM64 (Apple Silicon via UTM, RPi, cloud)
}
```

- **Bare-metal ISO:** for converts. Installs Chief OS as the primary OS.
- **VM image:** first trial surface. Ships as OrbStack/UTM-compatible QCOW2. Human can run Chief as a VM inside macOS/Linux while keeping their main OS untouched.
- **Bootable USB:** conference-demo-friendly. Boot the USB on any laptop, experience Chief for an hour, shut down, original OS untouched.

**Ship target at v0:** all three. The VM image is the primary onboarding path for Person Zero; ISO and USB for enthusiasts.

## Hardware requirements

| Component | v0 minimum | Recommended |
|---|---|---|
| CPU | x86_64 with AVX2, or ARM64 with Neon | Recent Zen/Intel/Apple Silicon |
| RAM | 16 GB | 32 GB (local model + cloud twin + overnight agents) |
| Storage | 128 GB SSD | 512 GB+ (blobs grow with horizon) |
| TPM / Secure Enclave | **Mandatory** | TPM 2.0, or Apple SE via virtualization attestation |
| GPU | Optional but recommended for local model | Nvidia with CUDA or Apple Metal |
| Network | Ethernet or Wi-Fi 6 | — |
| YubiKey / FIDO2 | Optional, recommended | Yes, for Ceremonies |

## Boot sequence

```
power-on
  → firmware (UEFI)
  → systemd-boot → measure PCRs → unlock LUKS (TPM-sealed)
  → initramfs → Chief Kernel systemd target
      ⇨ Agent Runtime
      ⇨ Capability Broker
      ⇨ Memory Graph
      ⇨ Provenance Log
      ⇨ Trust Ledger
      ⇨ Region Router
      ⇨ Event Bus
  → Wayland compositor
  → Morning Brief surface renderer
(total target: ≤10s post-firmware)
```

No login screen. No greeter. No desktop environment. The Morning Brief *is* the login, gated by biometric on first surface interaction of the day.

## Updates & rollback

- Nix flake bumps produce a new generation.
- `chief update` fetches, builds offline, stages as a pending generation.
- Reboot switches; if the new generation fails health check (e.g., kernel services don't come up), auto-rollback to previous generation.
- User can pin or roll back at any time: `chief rollback` → atomic.

## Firmware & boot attestation

- UEFI Secure Boot with our own signing keys (user can opt out).
- Measured boot with TPM PCRs; Chief Kernel attests its own state in the Provenance Log at boot.
- Any unexpected PCR → "tampered boot" warning on Morning Brief and all ceremonies blocked until investigated.

## Acceptance

- [ ] Boot target met: ≤10s power-on → Morning Brief on reference hardware.
- [ ] All three deploy targets build from one flake via CI.
- [ ] Rollback on failed update verified in CI.
- [ ] TPM binding tested on x86_64 and aarch64.
- [ ] Installer supports TPM-auto-enroll LUKS keys.

## Open questions

1. Do we ship our own signed UEFI keys, or require user MOK enrollment? (Lean toward MOK for flexibility.)
2. On Apple Silicon, what's the path to TPM-equivalent key sealing inside UTM? (Investigate Apple's virt attestation.)
3. Do we support dual-boot (Chief OS + user's Windows/macOS on separate partitions) at v0 or defer?
4. What's the CI matrix for hardware certification (Framework, ThinkPad, MacBook M-series via UTM, a few Dells, Steam Deck for lolz)?
5. Is ZFS worth the complexity for snapshot layering, or is BTRFS + Nix generations enough?

## Related

- [`architecture`](./10-architecture.md) — L0/L1 placement
- [`security-model`](./14-security-model.md) — TPM, attestation, boot measurement
- [`local-vs-cloud`](./41-local-vs-cloud.md) — local runtime requirements
- [`adr/0001-nixos-linux-base.md`](../adr/0001-nixos-linux-base.md) — canonical decision
- [`adr/0003-wayland-not-x11.md`](../adr/0003-wayland-not-x11.md) — compositor choice
