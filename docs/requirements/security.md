---
id: req-security
title: "Security Requirements"
status: draft
owners: [santosh]
last_updated: 2026-04-21
related: [security-model, non-functional]
tags: [requirements, security, threat-model]
---

# Security Requirements

Append-only. Each requirement has a stable ID `S-NNN`.

## Capability-based enforcement

| ID | Requirement |
|---|---|
| S-001 | Process identity is `(principal, capability-set)`, never `(uid, gid)` alone. |
| S-002 | No syscall bypass for capability-relevant operations. eBPF-LSM enforces at kernel boundary. |
| S-003 | Grants are typed, scoped, expiring, revocable, and non-delegatable. |
| S-004 | Capability Broker is the single enforcement point; logged at every check. |
| S-005 | Adding a new capability kind requires an ADR and a closed-enum bump. |

## Hardware root of trust

| ID | Requirement |
|---|---|
| S-101 | TPM 2.0 or Secure Enclave (hw-virt attested) mandatory. |
| S-102 | User signing key sealed to TPM/SE; never leaves hardware. |
| S-103 | YubiKey supported as secondary signing device. |
| S-104 | Key rotation is an explicit ritual with old-key signs-new-key provenance anchor. |
| S-105 | Recovery via M-of-N Shamir split across trusted contacts; v1 ritual, in-person attestation. |

## Approval cryptography

| ID | Requirement |
|---|---|
| S-201 | Every externally-effective action requires a signed ceremony token. |
| S-202 | Tokens are single-use, short-lived, bound to (action_id, timestamp, payload_hash). |
| S-203 | Token replay returns `REPLAY_DETECTED` from Capability Broker. |
| S-204 | Every consumed token is logged in Provenance Log forever. |
| S-205 | Ceremony UI cannot be impersonated (compositor-enforced surface isolation). |

## Sandboxing

| ID | Requirement |
|---|---|
| S-301 | Each agent runs in its own `systemd-nspawn` + user-namespace (v0). |
| S-302 | Each agent runs in its own `Firecracker` microVM (v1+). |
| S-303 | Per-pack `netns` + nftables allowlist enforced. |
| S-304 | `seccomp-bpf` allowlist narrow per agent role. |
| S-305 | Audio / video / GPU capabilities explicit per-call, mediated via API proxy. |

## Supply chain

| ID | Requirement |
|---|---|
| S-401 | All packs must be signed (ed25519 / cosign). Unsigned installs rejected. |
| S-402 | Signature verification on every install + every update. |
| S-403 | Capability grant display mandatory before install; user taps consent per-kind. |
| S-404 | Pack version bump with grant changes requires re-consent. |
| S-405 | Official tier packs undergo internal + legal review. |
| S-406 | Nix substitutes pinned and signature-verified. |
| S-407 | Reproducible builds required for packs in official tier. |

## Data protection

| ID | Requirement |
|---|---|
| S-501 | All state at rest encrypted (LUKS v0; per-file keying v1). |
| S-502 | All network traffic encrypted (TLS ≥1.3 / QUIC). |
| S-503 | Cloud twin receives only encrypted blobs for sensitive-horizon nodes. |
| S-504 | Sensitive-category retrieval runs on-device only. |
| S-505 | Device key never leaves hardware; used via SE/TPM API. |

## Provenance & audit

| ID | Requirement |
|---|---|
| S-601 | Provenance Log is append-only, Merkle-linked, device-key-signed. |
| S-602 | Retention ≥ 7 years on-device; optional Rekor-compatible mirror. |
| S-603 | External auditor can verify any chain via standard Merkle tooling. |
| S-604 | Provenance entries cannot be deleted; tombstones + rollback only. |
| S-605 | Every capability grant issue / revoke logged. |

## Threat model coverage

| Threat | Mitigation(s) |
|---|---|
| Malicious pack | S-401–S-407, S-301–S-305 |
| Compromised cloud twin | S-503, S-504, S-601 |
| Stolen / lost device | S-101–S-105, S-501, S-505 |
| Supply-chain attack on Nix deps | S-406, S-407 |
| Prompt injection via ingested content | S-001–S-004 (scope-limited caps), Region Router anomaly flag |
| Rogue LLM drift / hallucination | Ceremony gate (S-201–S-205), rollback primitive |
| Physical observer / shoulder-surf | Ceremony redacts by default; reveal gesture required |
| Rogue sysadmin (enterprise v3+) | S-102, S-601 |

## Incident response

| ID | Requirement |
|---|---|
| S-701 | Published vulnerability disclosure process at `chief-os.com/security`. |
| S-702 | Takedown SLA: 24h official-tier pack, 72h community-tier. |
| S-703 | Post-incident provenance-chain reconstruction supported as standard procedure. |
| S-704 | User key rotation tooling available from v0. |

## Related

- [`../06-security-model.md`](../06-security-model.md) — full security architecture
- [`non-functional.md`](./non-functional.md) — NFR cross-reference
- [`../adr/0002-capability-based-security.md`](../adr/0002-capability-based-security.md) — canonical decision
