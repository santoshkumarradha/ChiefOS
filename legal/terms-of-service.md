---
id: terms-of-service
title: "Chief OS — Terms of Service (draft v0)"
status: draft
owners: [santosh]
last_updated: 2026-04-21
binding: false
review_required: "external counsel before v0 public launch"
tags: [legal, regulatory, machine-as-fax]
---

# Terms of Service — Chief OS (draft v0)

> **NOT LEGAL ADVICE. NOT BINDING.** This is a working draft encoding the machine-as-fax posture ([ADR-0004](../adr/0004-machine-as-fax-posture.md)) and the regulatory stance in [docs/15-regulatory-posture.md](../docs/15-regulatory-posture.md). External counsel review is required before v0 public launch. Placeholders are marked `{TBD counsel}`.

## 1. Definitions

- **"Chief OS"** — the operating-system software distributed from this repository, including the kernel, surfaces, and first-party Capability Packs.
- **"You" / "User"** — the natural person or legal entity installing and operating Chief OS on their device.
- **"Agents"** — AI-driven software components that run within Chief OS to perform delegated work on your behalf.
- **"Ceremony"** — the high-friction approval surface (see [docs/05-surfaces.md](../docs/05-surfaces.md) §Ceremony) at which you apply cryptographic authorization to an externally-binding action.
- **"Capability Pack"** — a signed software module loaded into Chief OS under explicit capability grants (see [ADR-0002](../adr/0002-capability-based-security.md)).
- **"External Action"** — any action that communicates with a system outside of your device (sending email, posting to a server, signing a contract, initiating a payment, etc.).

## 2. The machine-as-fax posture

Chief OS operates your agents like a fax machine: a tool of yours that extends your reach, not an independent legal actor.

**2.1.** Every External Action that has legal, financial, or contractual significance must pass through a Ceremony surface where you, the User, apply a cryptographic authorization using a device under your sole control (biometric, YubiKey, or equivalent hardware token).

**2.2.** The cryptographic authorization at the Ceremony is the legally operative act. The Agent is the conduit; you are the actor.

**2.3.** Chief OS records each Ceremony into a signed, append-only event log ([ADR-0005](../adr/0005-signed-typed-event-log.md)) producing non-repudiable evidence of your authorization.

**2.4.** You agree to treat agent-drafted outputs as your own work when you sign them at a Ceremony. Reviewing the evidence presented during the Ceremony is your responsibility.

## 3. ESIGN / UETA / eIDAS compatibility

**3.1.** Where Chief OS is used to execute documents subject to the U.S. Electronic Signatures in Global and National Commerce Act (ESIGN, 15 U.S.C. § 7001 et seq.), the Uniform Electronic Transactions Act (UETA), or the E.U. eIDAS Regulation (No 910/2014):

- Your cryptographic authorization at a Ceremony constitutes an electronic signature with intent under those laws.
- The Ceremony records the associated record retention requirements (timestamp, signer identity assertion, content hash).
- You consent to conduct transactions electronically; you may withdraw consent per Section 10.

**3.2.** `{TBD counsel: eIDAS qualified-signature vs advanced-signature classification for biometric + hardware token combination.}`

**3.3.** Chief OS does not guarantee compliance with jurisdictional notarization or apostille requirements. Those remain your responsibility.

## 4. GDPR / data protection

**4.1. Controller / Processor.** You are the **data controller** for personal data processed on your device by Chief OS. Chief OS distributes software; it does not operate a cloud service that processes your data on a controller basis.

**4.2.** First-party cloud services (if any — `{TBD counsel: declare explicitly when inference is routed to a provider}`) act as **processors** under a Data Processing Addendum, and only upon your explicit opt-in during a Ceremony.

**4.3. On-device processing by default.** Per [docs/09-local-vs-cloud.md](../docs/09-local-vs-cloud.md), agent inference, memory storage, and surface rendering default to on-device. Any cloud leg is shown during the Ceremony that authorizes it.

**4.4. Data subject rights.** Export, deletion, and rectification of your data are supported through surfaces provided by Chief OS; see [docs/08-memory-substrate.md](../docs/08-memory-substrate.md). Chief OS does not retain your data on our infrastructure.

**4.5. Transfers.** No international transfer of personal data occurs unless you explicitly authorize a cloud provider at a Ceremony.

## 5. License

Chief OS is distributed under the GNU Affero General Public License v3.0 or later (AGPL-3.0-or-later). Your use of the software is subject to that license; see [`LICENSE`](../LICENSE).

These Terms of Service govern the relationship between you and the Chief OS project for any first-party cloud services or support offerings not covered by the software license.

## 6. Capability Packs and third-party software

**6.1.** Capability Packs published by third parties are governed by their own license and the capability grants you issue to them through the Capability Broker.

**6.2.** You can revoke any capability grant at any time from the Trust Ledger Viewer. Revocation takes effect immediately for future calls.

**6.3.** Third-party pack authors are independent of the Chief OS project. The project is not liable for their conduct.

## 7. Prohibited uses

You shall not use Chief OS to:

- Perform External Actions on behalf of a natural or legal person who has not authorized you;
- Circumvent the Ceremony surface for actions that ADR-0002 and docs/06-security-model.md classify as requiring one;
- Train, fine-tune, or distill models on Memory Graph content belonging to other parties without their written authorization;
- Export signed provenance records in a manner that misrepresents the cryptographic chain of custody.

## 8. Disclaimers

`{TBD counsel: standard AS-IS, NO WARRANTIES, LIMITATION OF LIABILITY, INDEMNITY language calibrated to AGPL + jurisdictional enforceability.}`

Chief OS is beta software. Agents make mistakes. Always review the evidence presented at the Ceremony before authorizing.

## 9. Changes to these Terms

Material changes will be announced in the [`CHANGELOG`](../CHANGELOG.md) with at least 30 days notice before taking effect. Continued use after the effective date constitutes acceptance.

## 10. Withdrawal of consent

You may withdraw consent to electronic transactions, uninstall Chief OS, and export your Memory Graph at any time from the operating system's standard uninstall flow. Your signed provenance records remain cryptographically verifiable in perpetuity because they are cryptographic facts, not database entries under our control.

## 11. Contact

`{TBD counsel: designated contact for data subject requests, regulatory inquiries, and responsible disclosure.}`

## Related

- [`LICENSE`](../LICENSE) — AGPL-3.0-or-later
- [`adr/0002-capability-based-security.md`](../adr/0002-capability-based-security.md)
- [`adr/0004-machine-as-fax-posture.md`](../adr/0004-machine-as-fax-posture.md)
- [`adr/0005-signed-typed-event-log.md`](../adr/0005-signed-typed-event-log.md)
- [`docs/15-regulatory-posture.md`](../docs/15-regulatory-posture.md)
- [`legal/privacy-policy.md`](./privacy-policy.md)
