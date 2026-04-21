---
id: regulatory
title: "Regulatory Posture"
status: draft
owners: [santosh]
last_updated: 2026-04-21
related: [security-model, oss-business, v0-scope]
tags: [legal, regulatory, compliance]
---

# Regulatory Posture

## TL;DR

- v0–v1 posture: **"Machine as fax."** Chief drafts + queues; human cryptographically signs. Human is the legal actor. ([adr-0004](../adr/0004-machine-as-fax-posture.md))
- E-signature compatible with **ESIGN/UETA (US)** and **eIDAS (EU)**.
- Money rails (v2+) via licensed partners (Stripe Issuing, Plaid, Modern Treasury). We never move money directly.
- KYC / AML obligations attach to the **human**, not the agent.
- EU AI Act: operate as "limited risk" at v0; reassess if category shifts.

## Legal posture: "Machine as fax"

Every externally-effective action requires a cryptographically signed approval token from the human, produced by hardware (TPM / Secure Enclave / YubiKey). The OS is a drafting + queuing engine; the human is the legal actor.

Technical backing:
- Approval tokens are single-use, short-lived, bound to `(action_id, timestamp, payload_hash)`.
- Every token is logged in the Provenance Log.
- Token cannot be replayed; broker journals consumed tokens.

This posture is defensible across US + EU + most jurisdictions we care about for P0.

## E-signature compatibility

### United States — ESIGN Act (2000) + UETA

| Requirement | Chief OS answer |
|---|---|
| Intent to sign | Ceremony UI shows payload; user's deliberate hold-or-biometric indicates intent |
| Consent to electronic records | Obtained at onboarding; opt-out returns user to manual-only use |
| Association of signature with record | Token bound to `(action_id, payload_hash)` via hardware-backed key |
| Record retention | Provenance Log retention ≥ 7 years |
| Non-repudiation | Hardware-bound signatures + Merkle-linked log |

### European Union — eIDAS

- **Simple Electronic Signature (SES):** Chief's default. Sufficient for most private contracts.
- **Advanced Electronic Signature (AES):** Achievable when ceremony uses a qualified signing key (YubiKey + specific CA). We can support this at v1+.
- **Qualified Electronic Signature (QES):** Requires a Qualified Trust Service Provider (QTSP). Partner integration, not a v0 feature.

**v0 commitment:** SES out of the box. AES available with user-provided qualified key.

### Jurisdictions out of scope at v0

India, Japan, China, Brazil, and most of APAC have their own e-sign frameworks. We block signups from those jurisdictions at v0 and add them as legal review completes. Do not claim compliance we haven't verified.

## Money rails (v2+)

We do not move money. Ever. We **draft** financial actions and **orchestrate** user-approved flows through licensed partners:

| Flow | Partner (candidate) |
|---|---|
| Card issuance / purchase auth | Stripe Issuing, Marqeta |
| ACH / wire origination | Modern Treasury, Increase |
| Bank account read | Plaid (US), TrueLayer / Tink (EU) |
| International remit | Wise, Airwallex |
| Crypto custody | Not at v0; external wallets via OAuth if at all |

All money flows require a ceremony with a signed token. Partners verify the user identity; Chief is the drafting layer.

**Liability model:** partner holds the money-transmission license; Chief is a pass-through drafting tool. The human is the actor of record.

## KYC / AML

- **The human** is the subject of KYC, performed by the financial partner at account linkage.
- Chief does not store or revalidate KYC artifacts; these live with the partner.
- For v2 money actions, Chief attaches the user's partner-issued identity reference to the signed token, so the audit trail shows "user X (verified by partner Y) approved action Z."

## EU AI Act (2024) — v0 classification

| Tier | Applies? |
|---|---|
| Prohibited | No. |
| High-risk | No (we do not operate in biometric ID, law enforcement, critical infra, education scoring, employment, essential services, law/justice, immigration). |
| Limited-risk | **Yes.** Chief must be transparent: users know they are interacting with AI; outputs are disclosed as AI-generated. |
| Minimal-risk | N/A (we are above minimal). |

**v0 obligations:**
- Clear labeling of AI-generated drafts (visible in Morning Brief).
- Transparent about what agents are running and with what data access.
- Logging sufficient for audit (Provenance Log exceeds what's required).

**Reassess annually.** If Chief moves into regulated verticals (healthcare advice, legal advice, employment), the classification may shift. ADR required if so.

## HIPAA (US healthcare data)

- v0: **not HIPAA-covered.** We do not offer BAAs or accept PHI as a covered entity.
- Users are advised to treat health categories as local-only; our sensitive-category routing enforces this at the architecture level.
- If a user runs a health pack that ingests PHI, that pack is responsible for informing them of HIPAA status.
- Enterprise (v3) may include a BAA-covered tier.

## GDPR (EU personal data)

| Article | Chief response |
|---|---|
| Art. 6 (lawfulness) | User is the controller; Chief processes under explicit consent + contract. |
| Art. 15 (access) | Provenance Log export gives the user full audit trail. |
| Art. 17 (erasure) | Rollback + CAS tombstone + explicit `chief purge --category X` command. |
| Art. 20 (portability) | Memory Graph exports as JSON + signed bundle. |
| Art. 25 (privacy by design) | Capability-based security + local-first is PbD architecture. |
| Art. 32 (security) | Encrypted at rest (LUKS), encrypted in transit, hardware-backed keys. |

## Data Residency

- v0: user's device is the source of truth. No data residency question on local side.
- Cloud twin: operate twins in regions per user's jurisdiction (US, EU) by v0; APAC post-launch.
- Encrypted replica: same region as user's primary jurisdiction.

## Liability insurance

- v0: E&O policy for Chief OS as a software publisher. ~$2M coverage minimum.
- v1: Upgrade to include pack-registry liability if we offer signed official packs.
- v2: Cybersecurity rider as money/legal features come online.

## Dispute & takedown policy

- Any legal request for user data routed through a published process at `chief-os.com/legal`.
- We provide only what's ours to provide — Provenance Log entries for the account in question. User data is end-to-end under their key.
- Takedown of malicious packs: 24-hour SLA on official tier; 72-hour on community tier with signed abuse reports.

## Ongoing regulatory cadence

| Cadence | Action | Owner |
|---|---|---|
| Quarterly | Review EU AI Act guidance updates | Legal + founder |
| Bi-annual | Third-party security audit | Security eng |
| Annual | SOC 2 Type II audit (post v1) | Security + ops |
| Per-launch-region | Local counsel review | Legal |

## Acceptance

- [ ] "Machine as fax" posture codified in Terms of Service by v0.
- [ ] ESIGN/UETA/eIDAS compliance documented by day 60.
- [ ] GDPR DPA template ready before EU launch.
- [ ] E&O insurance bound before v0 public launch.
- [ ] Partner LOIs for money rails signed before v1 announce.
- [ ] EU AI Act limited-risk transparency UI audited by external counsel.

## Open questions

1. Do we offer HIPAA BAA at enterprise tier, or refuse until clarity on agent-in-the-loop medical liability?
2. Which jurisdictions get a v0 launch and which are blocked?
3. Do we require users to link a verified identity (through a partner) before enabling high-stakes categories, even for v0?
4. What's the minimum viable cookie / tracking disclosure for `chief-os.com` (GDPR + ePrivacy)?
5. Are we subject to AI liability directive drafts in EU? Counsel review needed.

## Related

- [`security-model`](./06-security-model.md) — cryptographic backing
- [`oss-business`](./11-open-source-business.md) — commercial obligations
- [`v0-scope`](./13-v0-scope-90-day.md) — v0 compliance scope
- [`adr/0004-machine-as-fax-posture.md`](../adr/0004-machine-as-fax-posture.md) — canonical decision
