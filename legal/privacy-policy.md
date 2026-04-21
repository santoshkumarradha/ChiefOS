---
id: privacy-policy
title: "Chief OS — Privacy Policy (draft v0)"
status: draft
owners: [santosh]
last_updated: 2026-04-21
binding: false
review_required: "external counsel before v0 public launch"
tags: [legal, privacy, gdpr, ccpa]
---

# Privacy Policy — Chief OS (draft v0)

> **NOT LEGAL ADVICE. NOT BINDING.** Working draft. External counsel review required before v0 public launch. Placeholders marked `{TBD counsel}`.

## 1. What Chief OS is, and how that shapes privacy

Chief OS is software you install on your device. It is not a cloud service under our operational control. Your data lives on your machine in your Memory Graph (see [docs/08-memory-substrate.md](../docs/08-memory-substrate.md)).

The normal SaaS privacy pattern — "we collect, we process, we store in our cloud" — does not apply here because there is no "we" touching your data on a cloud basis, except in the narrow cases listed in Section 4.

## 2. Data processed on your device

Chief OS processes, on your device:

- Content you explicitly input or connect (emails, calendars, documents, spreadsheets, etc. — only via Capability Packs you install and grant).
- Memory Graph nodes produced by Agents acting on your behalf.
- Signed Inference attestations ([ADR-0009](../adr/0009-signed-inference.md)).
- Signed Event Log entries ([ADR-0005](../adr/0005-signed-typed-event-log.md)).
- Trust Ledger grants and usage records.

This data never leaves your device unless you explicitly authorize an External Action at a Ceremony or grant a Capability Pack `net.http` with specific hosts.

## 3. Data shared with us (by default: none)

By default, Chief OS does not share any of your data with the project maintainers or any cloud service. No telemetry. No crash reports. No usage analytics.

## 4. Optional cloud services

`{TBD counsel: enumerate cloud services once scoped — e.g., optional model-hosted inference for models that don't fit on-device; optional sync across your own devices; optional first-party packs that require cloud legs.}`

Each cloud service, when it exists, will be:

- **Opt-in** — you authorize the cloud leg at a Ceremony on first use.
- **Explicit** — the Ceremony shows exactly what data leaves your device, to which provider, for what purpose.
- **Auditable** — every cloud call is recorded in the Event Log with its cryptographic receipt.
- **Processor** — we act as a GDPR data processor under a Data Processing Addendum, never as a controller.

## 5. Data shared with third parties (Capability Packs)

Capability Packs you install can access your data only to the extent of the capability grants you issue through the Capability Broker.

- Grants are typed, scoped, revocable, and time-limited.
- You see every grant in the Trust Ledger Viewer.
- Revocation is immediate and retroactive (the signed event log remains, but future calls fail).
- Packs communicate with their own external services per the grants they hold — the Chief OS project is not a party to those communications.

## 6. Cryptographic receipts

Every External Action you authorize produces a cryptographic receipt in your local Event Log. These receipts are signed by your device keys (you hold the only private key). They are verifiable by anyone to whom you disclose them — e.g., an auditor, a regulator, or a counterparty.

We cannot issue, revoke, or modify your receipts. They belong to you.

## 7. Children

Chief OS is not directed to children under the age of `{TBD counsel: 13 COPPA / 16 GDPR}`. Do not use it on behalf of a child.

## 8. Your rights

Because your data is on your device:

- **Access / export** — use the built-in export surface; produces a portable Memory Graph archive.
- **Erasure / deletion** — uninstall the OS or delete specific nodes via the Memory Graph surface. Signed receipts remain cryptographic facts (immutable by design) but contain no recoverable personal content beyond what the receipt itself records.
- **Rectification** — edit the underlying node; the event log preserves the history.
- **Portability** — export is standards-friendly (`{TBD counsel: confirm format — JSON-LD / IPFS / other}`) so you can move to another OS without vendor lock-in.

For any optional cloud service (Section 4), the same GDPR / CCPA / APPI rights apply; the relevant surface will route your request appropriately.

## 9. Retention

On-device data is retained until you delete it. The OS does not set a retention policy on your own data.

For optional cloud services, retention policies will be specified in their respective DPAs.

## 10. Security

- Local data is encrypted at rest using platform-standard disk encryption (FileVault, LUKS, BitLocker).
- OAuth tokens are additionally sealed with XChaCha20-Poly1305 ([`crates/chief-oauth/`](../crates/chief-oauth/)).
- Device keys are stored in the platform Secure Enclave / TPM where available.
- Signed Inference attestations provide tamper-evidence on agent outputs.
- Ceremony requires hardware-rooted authentication (biometric or hardware token) for all externally-binding actions.

## 11. Changes

Material changes announced via `CHANGELOG.md` with at least 30 days notice before effective date.

## 12. Contact

`{TBD counsel: designated privacy contact, data protection officer if required, responsible disclosure address.}`

## Related

- [`legal/terms-of-service.md`](./terms-of-service.md)
- [`adr/0002-capability-based-security.md`](../adr/0002-capability-based-security.md)
- [`adr/0004-machine-as-fax-posture.md`](../adr/0004-machine-as-fax-posture.md)
- [`adr/0005-signed-typed-event-log.md`](../adr/0005-signed-typed-event-log.md)
- [`adr/0009-signed-inference.md`](../adr/0009-signed-inference.md)
- [`docs/15-regulatory-posture.md`](../docs/15-regulatory-posture.md)
- [`docs/08-memory-substrate.md`](../docs/08-memory-substrate.md)
