---
id: adr-0004
title: "0004 — 'Machine as fax' legal posture for v0–v1"
status: accepted
authors: [santosh]
created: 2026-04-21
decided: 2026-04-21
supersedes: []
superseded_by: []
axiom_impact: [2, 3]
tags: [legal, regulatory, trust, ceremony]
---

# 0004 — "Machine as fax" legal posture for v0–v1

## Context

An AI-native OS that drafts replies, moves meetings, and eventually signs contracts and wires money invites the legal question: *when the machine acts externally, who is the legal actor?* Liability models, jurisdictional regulations (US ESIGN/UETA, EU eIDAS), and consumer protection all hinge on this.

Three defensible postures:

| Posture | Model | Viability |
|---|---|---|
| Machine as fax | Human signs cryptographically; OS drafts + queues. Human is the legal actor. | Clean for v0/v1, works with ESIGN/UETA/eIDAS |
| Machine as agent of principal | Like broker/attorney. Licensed or disclosed agency. | Requires legal licensing in many jurisdictions |
| Machine as autonomous party | Self-acting entity with its own personhood. | Pure fantasy; no underwriter exists |

## Decision

**"Machine as fax"** for v0 and v1 (at minimum). Every external-effect action requires an explicit, cryptographically signed approval token from the user before it ships. The OS is a drafting + queuing engine; the human is the legal actor.

## Rationale

1. Matches existing e-signature frameworks without novel law.
2. Maps 1:1 to our Capability Broker + Ceremony flow — we already architected for this.
3. Defensible across US + EU + most jurisdictions we care about for P0.
4. Keeps liability on the human, whose choice remains paramount.
5. Does not preclude later evolution — v2+ could evolve to a disclosed-agency model for specific high-stakes categories with partner counsel.

## Consequences

### Positive
- Simple answer to regulators: "our OS does not act; the human does, via cryptographically signed tokens."
- No licensing required for v0 / v1.
- Audit trail is clean: every external action has a human's signature.
- Compatible with existing e-sign integrations (we're an endpoint, not a novel legal fiction).

### Negative / accepted costs
- "Fully autonomous" messaging is impossible; every ship needs an approval, even if friction is near-zero at Region 5.
- Some automation dreams (auto-pay recurring bills without approval) require graduated ceremony patterns, not silent dispatch.

### Downstream effects
- [`docs/14-security-model.md`](../docs/14-security-model.md) — ceremony cryptography is the backbone of this posture.
- [`docs/62-regulatory-posture.md`](../docs/62-regulatory-posture.md) — detailed legal framing.
- Every Region-7/8 action requires a single-use signed token, binding action to human intent.

## Implementation notes

- Ceremony token = single-use, short-lived, binding `(action_id, timestamp, payload_hash)`.
- Token signing via TPM / Secure Enclave or YubiKey — hardware root.
- Token is logged into Provenance Log and cannot be replayed.
- Legal language in onboarding: "Your Chief drafts; you sign. Every external action is your action."

## Revisit triggers

- v2 money / legal evolves beyond simple ceremonies to require standing delegation (e.g., recurring payments without per-cycle approval).
- Courts issue adverse rulings on e-signature under agent-drafted flows.
- A licensed-agency framework for AI agents emerges in a jurisdiction we operate in.
- We enter regulated industries (healthcare, finance-advisory) that require novel postures.
