---
id: adr-0016
title: "Kernel principal identity — chief-core holds grants in the same registry as packs"
status: accepted
date: 2026-04-21
deciders: [santosh]
supersedes: []
superseded_by: []
amends: [0002, 0012]
tags: [security, capability, principal, kernel, axiom-2]
---

# ADR-0016 — Kernel Principal Identity & Self-Grants

## Context

[ADR-0012](./0012-no-settings-app.md) §"Security & Privacy — OS-itself grants are visible" committed to showing the kernel's own capabilities in the Security & Privacy surface alongside pack grants. The architecture feasibility review (2026-04-21) flagged the unresolved tension: `chief-core` is not a pack (no Nix flake, no `manifest.toml`), yet the UX model treats it as one. Two consequences follow:

1. **What's the kernel's principal ID?** The Capability Broker ([`adr-0002`](./0002-capability-based-security.md)) keys grants by principal. The kernel must have one.
2. **Who issues grants to the kernel?** If the kernel issues to itself, Axiom 2 ("no ambient authority") is superficially violated — the kernel would have ambient authority over its own grant registry.

Options considered:

| Option | Honest? | Axiom 2 | Machinery | Verdict |
|---|---|---|---|---|
| (A) Hardcode a "kernel" principal with display-only (pretend) grants | No — the kernel has full ambient authority in reality | ✗ | Low | Rejected — dishonest surface |
| (B) Treat the device key as the root-of-trust principal; kernel holds grants on the device key's behalf at boot, verified by a signed boot-attestation | Yes | ✓ with an explicit asterisk | Medium | **Chosen** |
| (C) Issue kernel grants through an out-of-band ceremony at first boot (user co-signs the kernel's self-grants) | Yes, but onerous | ✓ | High | Rejected — too much first-boot friction |

## Decision

**The device's hardware-sealed identity key is the root-of-trust principal. `chief-core` holds grants from this principal in the same Capability Broker registry as every other principal, issued at boot from a signed boot-attestation, and visible in Security & Privacy alongside pack grants.**

### Concrete model

1. **Device identity key.** Generated at first boot in TPM / Secure Enclave (per [`docs/18-base-and-hardware.md`](../docs/18-base-and-hardware.md)). Never leaves secure hardware. Public half is the device's public identity.
2. **Root-of-trust principal.** `urn:chief:device:<pubkey-hash>`. Has one meta-grant: `meta.root_of_trust`, which permits issuing grants to the `urn:chief:kernel` principal.
3. **Kernel principal.** `urn:chief:kernel`. Receives its grants at every boot from the root-of-trust principal via a signed boot-attestation. Grants include:
   - `fs.read { paths: ["/etc/chief/**", "$CHIEF_HOME/**"] }` — own config.
   - `fs.write { paths: ["$CHIEF_HOME/event-log/**", "$CHIEF_HOME/trust-ledger/**", "$CHIEF_HOME/broker.db", ...] }` — own state stores.
   - `net.http { hosts: ["<model-provider-list>"] }` — only if user bound a cloud provider in Controls → Models.
   - `device.tpm { ops: ["sign", "seal", "unseal"] }` — crypto ops for signed inference, event log, ceremony.
   - `mem.read { types: ["*"] }`, `mem.write { types: ["*"] }` — memory graph is kernel-owned infrastructure.
   - ...etc.

4. **Root-of-trust ceremony at first boot.** The very first time Chief OS boots on a device, the root-of-trust principal is created + the kernel's default grant set is issued via a single Ceremony surface interaction (the user physically taps the YubiKey or authenticates via biometric to bind the device key to their identity). This is the only time `meta.root_of_trust` is directly invoked by a human interaction; subsequent boots replay the signed boot-attestation.

5. **Display in Security & Privacy.** The top row of the "By Pack" projection shows "Chief Kernel — device: \<pubkey-hash> — N grants" expandable into the full grant list. Grants can be viewed but not revoked through the normal revoke flow — the kernel cannot be left unfunctional. Attempts to revoke show a link to a dedicated "Reset Chief" ceremony that triggers wipe + reinstall.

6. **Subsequent grant changes.** The kernel's grant list only changes through Ceremony (user-initiated) — e.g., "Let Chief OS reach Anthropic's API" is a Ceremony that adds a `net.http { hosts: ["api.anthropic.com"] }` grant to the kernel. This is the same Ceremony mechanism packs use. No special backdoor.

### Axiom 2 — honesty

[Charter](../CHARTER.md) Axiom 2 says "no ambient authority." This ADR accepts an **explicit asterisk**: the device identity key has ambient authority over its own grants because hardware-sealed roots of trust cannot be bootstrapped otherwise. The alternative — true zero-root bootstrap — is provably impossible for a local-first OS without an external trusted third party.

Honest rephrase of Axiom 2 going forward: "no **software-ambient** authority; hardware-rooted identity is the sole permitted ambient authority, and it is visible, revocable-via-wipe, and attested." A CHARTER amendment will update the wording.

### What this does NOT mean

- It does **not** mean the kernel can do whatever it wants. Every kernel operation checks the Broker against the kernel's declared grants. Revoked grants (via Ceremony) cannot be used even by the kernel — for instance, if the user revokes "cloud inference," the kernel cannot route to a cloud model until a new Ceremony re-grants it.
- It does **not** mean packs see the kernel as a peer they can negotiate with. Packs interact only with `chief-sdk`; the kernel-principal's grants are visible in audit, not addressable by packs.
- It does **not** create a "super-user" like Unix root. The kernel's grants are scoped and auditable; it cannot do anything not in its grant list.

## Consequences

**Positive:**
- Security & Privacy surface is honest — the top row shows what the kernel actually holds, no hidden "infrastructure" category.
- The grant registry is single-source-of-truth for all authority, including the OS itself.
- User can view + audit kernel authority the same way they audit any pack.
- Axiom 2 is made precise rather than left ambiguous.
- Boot attestation path is explicit — verifiable after-the-fact via event log.

**Negative / tradeoffs:**
- `meta.root_of_trust` is a new capability kind that is by construction unique and non-transferable. CapabilityKind enum grows by one → 29 kinds. Closed-enum discipline (ADR-0010) honored: this ADR is the amendment.
- First-boot ceremony adds one step (~30 seconds, user taps YubiKey / passes biometric).
- Kernel-principal implementation carries some machinery that a hardcoded version would avoid. Scope: ~200 LOC in `chief-core` for boot-attestation + grant-replay.

## Charter amendment

Axiom 2 is amended from:

> **"No ambient authority. Every agent operation is mediated by a typed, scoped, revocable capability."**

to:

> **"No software-ambient authority. Every agent operation, including the kernel's own, is mediated by a typed, scoped, auditable capability in the Capability Broker registry. The device's hardware-sealed identity key is the sole permitted root-of-trust ambient authority; it is visible in Security & Privacy, revocable only by device wipe, and replayable via signed boot-attestation."**

A CHARTER PR will follow this ADR.

## CapabilityKind additions

Added to the closed enum in [`docs/40-pack-sdk.md`](../docs/40-pack-sdk.md):

- `meta.root_of_trust` — the one capability only the device identity key can hold. Permits issuing grants to `urn:chief:kernel`. No pack may request this.

Total `CapabilityKind` count: **29** (was 28 after ADR-0013, now +1).

## Related

- [`adr-0002`](./0002-capability-based-security.md) — Capability Broker semantics this ADR extends.
- [`adr-0005`](./0005-signed-typed-event-log.md) — boot-attestation is a typed event log entry.
- [`adr-0009`](./0009-signed-inference.md) — device identity key also signs inference attestations.
- [`adr-0012`](./0012-no-settings-app.md) — Security & Privacy surface shows the results of this model.
- [`CHARTER.md`](../CHARTER.md) — Axiom 2 amendment to follow.
- [`docs/14-security-model.md`](../docs/14-security-model.md) — will be updated to reference this ADR.
- [`docs/18-base-and-hardware.md`](../docs/18-base-and-hardware.md) — TPM / Secure Enclave path already committed.
- [`docs/32-controls-and-policy.md`](../docs/32-controls-and-policy.md) — Security & Privacy implementation.
