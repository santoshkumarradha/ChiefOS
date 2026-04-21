---
id: adr-0001
title: "0001 — NixOS on Linux as the base"
status: accepted
authors: [santosh]
created: 2026-04-21
decided: 2026-04-21
supersedes: []
superseded_by: []
axiom_impact: [3, 5, 7, 10]
tags: [base, kernel, infrastructure]
---

# 0001 — NixOS on Linux as the base

## Context

Chief OS needs a base OS that supports:
- Immutable root and atomic rollback (Axiom 5).
- Content-addressed, reproducible builds for provenance (Axiom 3).
- A radical userland shipped on a boring kernel (Axiom 7).
- Declarative config so house rules and capability grants can be policy-as-code (Axiom 10).
- Wide hardware support for a B2C v0 with a 90-day ship window.
- Community + existing open ecosystem we can extend, not replace.

## Options considered

| Option | Ecosystem | Cap-security native | Reproducibility | Rollback | 90-day ship | Notes |
|---|---|---|---|---|---|---|
| NixOS on Linux | 5 | 3 | 5 | 5 | 4 | Radical userland, boring infra |
| Fuchsia | 1 | 5 | 3 | 3 | 1 | Google-owned, tiny ecosystem, limited HW |
| seL4 | 1 | 5 | 3 | 2 | 1 | Formally verified, no ecosystem |
| FreeBSD + Capsicum | 3 | 4 | 3 | 3 | 2 | Nice cap model, niche base |
| illumos (Zones) | 2 | 4 | 3 | 3 | 2 | Great isolation, tiny ecosystem |
| Custom microkernel | 0 | 5 | 5 | 5 | 0 | Founder suicide for v0 |

## Decision

**NixOS on Linux** as the base for v0 and the foreseeable roadmap.

## Rationale

1. Nix store is content-addressed by construction → provenance primitive at L1.
2. Nix generations = atomic system rollback → reversibility primitive at L1.
3. Linux kernel has vast ecosystem + hardware support; we can ship in 90 days.
4. NixOS flake model makes house rules and capability grants declarative at L1, bridging directly to our L2 Trust Ledger and Capability Broker.
5. Cap-security gap (Linux UID/GID is ambient) is closable at L2 via our Capability Broker + eBPF enforcement. We do not need the kernel to be cap-native.

## Consequences

### Positive
- Immediate HW support across commodity laptops.
- Reproducible, offline-verifiable builds (matters for provenance + regulatory posture).
- Rollback primitive at L1; L2 extends it to memory + agent state.
- Declarative pack installs (flakes) are audit-friendly and signable.

### Negative / accepted costs
- Linux's ambient-authority model fights Axiom 2; we must actively enforce caps at L2.
- NixOS learning curve is real for packagers; we need an SDK to shield pack authors.
- Wayland + newer services on older hardware is occasionally rough; install compat testing required.

### Downstream effects
- [`adr-0002`](./0002-capability-based-security.md) — we must build cap broker to close the L1 gap.
- [`adr-0003`](./0003-wayland-not-x11.md) — Wayland is our surface compositor, implied by security model + NixOS alignment.

## Implementation notes

- Base flake lives at the repo root (future `flake.nix`).
- Three deploy targets (ISO, VM, USB) derived from same flake.
- Lanzaboote provides UEFI Secure Boot + measured boot.
- bootc-on-NixOS tracked as v0 stretch vs. v1 deferred — see [`docs/07-base-and-hardware.md`](../docs/07-base-and-hardware.md).

## Revisit triggers

- Fuchsia ships a usable, OSS, HW-broad edition.
- A Linux cap-native fork lands in mainline (CHERI-on-mainstream HW).
- Our cap-enforcement burden at L2 exceeds 30% of kernel-service dev time at v1.
