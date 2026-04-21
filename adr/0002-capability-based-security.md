---
id: adr-0002
title: "0002 — Capability-based security from hardware up"
status: accepted
authors: [santosh]
created: 2026-04-21
decided: 2026-04-21
supersedes: []
superseded_by: []
axiom_impact: [1, 2, 3]
tags: [security, capabilities, trust]
---

# 0002 — Capability-based security from hardware up

## Context

An AI-native OS where agents act on the user's behalf is exactly the environment where ambient-authority security models fail catastrophically. UID/GID-based Linux permissions grant too much to any process; a pack that "needs Gmail access" shouldn't get "can also read your bank tabs." Every incident in the agent era so far (Anthropic bwrap disable, Replit DB wipe, ChatGPT-Operator prompt-injection chains) is a symptom of ambient authority.

## Options considered

| Option | Pros | Cons |
|---|---|---|
| UID/GID + AppArmor profiles | Linux-standard, mature | Not capability-based; static; easy to over-grant |
| seccomp allowlist per process | Kernel-enforced | Syscall-level, not semantic-level; hard to express "gmail:labels=travel*" |
| Linux capabilities (CAP_*) | Kernel-supported | Too coarse (all-or-nothing bits) |
| Landlock / eBPF-LSM | Kernel-enforced, fine-grained | Kernel version dependent; not semantic by default |
| **Capability Broker (L2) + eBPF for high-risk ops** | Semantic caps with narrow scopes; LLM-refusable at call site | We must build and maintain it; perf on hot path |
| gVisor / Kata / Firecracker | Strong isolation | Isolation ≠ cap model; still need semantic gating on top |

## Decision

Build a **Capability Broker at L2** as the single enforcement point for every semantic capability (tool call, memory access, outbound action), with **eBPF-LSM enforcement** for raw syscalls in high-risk categories (fs, net, IPC). Process isolation (nspawn v0, Firecracker v1) is a *layer* beneath the broker, not a substitute.

## Rationale

1. Semantic caps require semantic gating; raw-syscall gating can't express scope like "gmail.send: recipients in allowlist, max 1200 chars."
2. Broker is the natural place for HAX-driven friction tiers (Region Router consults it on every proposed action).
3. eBPF gives kernel-boundary enforcement for the subset of caps that syscalls can express (network hostnames, filesystem paths). Belt and suspenders.
4. Hardware trust root (TPM / Secure Enclave) → user key → broker grants is a clean chain; no gaps.
5. LLM cannot forge a capability (grants are cryptographically bound to approval tokens).

## Consequences

### Positive
- Single enforcement point; one place to audit (Axiom 2).
- Narrow, typed, expiring grants become first-class in pack manifests.
- eBPF enforces even if an agent escapes its nspawn.
- Revocation is instantaneous and atomic.

### Negative / accepted costs
- Every tool call goes through a broker round-trip; perf budget must accept ≤ 5ms local, ≤ 20ms guest.
- We own the capability kind enum; adding new kinds requires an ADR.
- Broker is a high-value attack target; must be hardened and audited.

### Downstream effects
- [`adr-0001`](./0001-nixos-linux-base.md) — Linux ambient authority is why this ADR exists.
- [`docs/06-security-model.md`](../docs/06-security-model.md) — implements this decision.
- [`docs/03-chief-kernel.md`](../docs/03-chief-kernel.md) — Capability Broker is L2 service #2.

## Implementation notes

- Broker is a Rust service (memory safety matters).
- Grants persisted in SQLite; in-memory cache with invalidation on revoke.
- MCP server at kernel boundary for cross-process calls; vsock transport for microVM guests.
- Every check → Provenance Log entry at DEBUG; every deny → INFO.
- Fuzzing + formal lightweight verification (TLA+ model) for v1.

## Revisit triggers

- CHERI lands on mainstream HW; we could move caps to hardware.
- Fuchsia ships a usable OSS edition with native caps.
- eBPF becomes insufficient for our threat model (e.g., kernel bypass via io_uring loopholes).
