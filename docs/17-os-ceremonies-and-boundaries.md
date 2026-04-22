---
id: os-ceremonies
title: "OS Ceremonies & Boundaries — what the OS absorbs"
status: draft
owners: [santosh]
last_updated: 2026-04-21
related: [pack-sdk, security-model, module-system]
depends_on: [security-model]
tags: [sdk, ceremonies, threat-model, agent-autonomy]
---

# OS Ceremonies & Boundaries

Companion to [`40-pack-sdk.md`](./40-pack-sdk.md). Enumerates what the OS absorbs from application authors: operations that carry identity, money, privacy, or trust must be **OS-rendered ceremonies**, not pack-rendered surfaces. Derived from the Mac/iOS pattern (Apple Pay sheet, `ASWebAuthenticationSession`, Share Sheet, File Picker) extended for agent autonomy.

## TL;DR

- Agents take autonomous actions. Some must not be delegable.
- The **pack requests**; the **OS renders** the critical surface; the **OS returns an opaque handle** to the pack. Pack never touches the sensitive primitive.
- Research-backed: every capability-model ecosystem that *let apps render their own login* had credential-theft incidents at scale. Chief OS absorbs the dangerous surfaces as OS ceremonies.
- Threat model: **assume a malicious pack.** If a primitive is attack-vector-shaped in any ecosystem's history, it's absorbed here.

## Layer 1 — Identity & credentials

| Operation | Pack request | OS ceremony renders | Pack receives |
|---|---|---|---|
| OAuth login | `net.oauth2 { providers, scopes }` | Native Tauri surface loads provider's real sign-in page | `SessionHandle` (opaque) |
| Password entry | `credential.prompt { provider }` | OS text input, no custom keyboards, auto-clear clipboard on close | Opaque `CredentialHandle` stored in Keychain |
| API-key paste | `credential.prompt { kind: "api-key" }` | OS ceremony; validates + seals to Secure Enclave | `CredentialHandle` |
| 2FA / TOTP entry | same as password | Secure text input, no paste-visible buffer | Token result only |
| Passkey / WebAuthn | `auth.passkey` | OS-owned ceremony, biometric | Attestation reference |
| Device-key signing | — | `chief-inference` owns all signing | Signed receipt |

## Layer 2 — Money, legal, consequential

| Operation | Ceremony |
|---|---|
| Send wire / ACH / card charge | Ceremony: evidence card (amount, recipient, reversibility window) → 3-sec hold → Secure-Enclave sign → signed receipt through a licensed partner API (Stripe Issuing, Modern Treasury, Increase). |
| E-sign document | Ceremony renders document preview + signing sheet; pack cannot bypass or restyle. |
| Commit / push code to upstream | Ceremony at Region 7/8 thresholds. |
| Delete / disclose / export personal data (GDPR) | OS ceremony with audit entry. |
| High-stakes external message | Region Router triggers ceremony automatically; pack drafts, OS sends. |

## Layer 3 — Pickers & resource handoff (the "Apple Share Sheet" pattern)

Absorbing these eliminates whole malware categories (full-contacts read, full-FS read, cross-app data leak). Every major ecosystem learned this the hard way; we bake it in from day one.

| Absorbed primitive | Replaces (what packs would naively do) | Pack receives |
|---|---|---|
| **Contact picker** (`contact.pick`) | Packs requesting full `READ_CONTACTS` | Opaque contact IDs for only the selected contacts |
| **File picker** (`file.pick`) | Packs requesting broad FS scope | Single-use file handle; expires with session |
| **Recipient picker** | Packs building their own address-book UI | Opaque recipient reference routed through broker |
| **Share sheet** (`share.hand_off`) | Packs implementing custom inter-pack handoff | OS-held blob; destination pack reads via handle; sender never learns destination |
| **Directory picker** | Packs requesting `fs.read { paths: ["**"] }` | Scoped handle, expires with session |
| **Pack picker** | Pack A hard-coding which pack handles a type | User consents to routing; both packs see only the typed payload |

## Layer 4 — Devices & sensors

| Device | OS absorbs |
|---|---|
| Microphone | Always-on indicator while capture is active; pack cannot hide or extend; transient per-session consent; OS-owned audio pipeline |
| Camera | Same |
| Screen capture | Compositor overlays a banner while capture active |
| Location (future) | OS picker (one-time / always / never); pack sees coarse coordinates unless explicitly granted fine |
| USB / HID devices | OS-mediated handle; pack never touches raw HID |

## Layer 5 — Time, cost, attestation

| Primitive | Why OS-owned |
|---|---|
| Clock / time source | Packs can't lie about time (forgeable audit trails = death) |
| LLM cost budget | Pack declares ceiling; OS enforces; pack cannot exceed |
| Attestation signing | Only `chief-inference` signs; packs receive signed objects |
| Trust ledger writes | Pack reads own rows; writes happen via approval rituals |
| Region Router decisions | Deterministic, OS-owned rule table; packs cannot override friction tier |
| Power / battery state | Pack reads; OS controls throttling |

## Layer 6 — Agent-autonomy-specific (novel to Chief OS)

No prior consumer OS has absorbed these because no prior consumer OS has agents.

| Primitive | Absorbed because... |
|---|---|
| **Inter-agent communication** | Cross-pack agent-to-agent traffic without OS mediation = collusion / exfiltration. Goes through broker; both sides declare compatible capabilities. |
| **Sub-agent spawning depth** | Prompt-injected agent spawning 10 sub-agents = confused-deputy bomb. `agent.spawn { max_depth }` declared + enforced. |
| **Ritual invocation** | A ritual is privilege escalation (coordinating multiple agents). Packs can participate in rituals; only OS can *start* them from user intent. |
| **Meta-prompting** | Pack building prompts from untrusted ingested content = prompt injection. `meta.prompt` is **default-deny**. When granted: OS keeps prompt templates reviewed at install; packs parameterise, never provide raw template text. |
| **Memory-graph cross-pack reads** | Pack X reading Pack Y's nodes = exfiltration. Cross-pack edges require user consent ("let X see travel-related things from Y"). |
| **Capability escalation at runtime** | Packs cannot request new capability kinds post-install. Re-install + re-consent required. (Prevents "free-trial-now-demands-your-bank-account.") |
| **Untrusted-input tainting** | Ingested content (emails, web pages, files) carries a taint tag in the Memory Graph. Any action derived from tainted input auto-escalates to a higher friction tier in the Region Router. |
| **Provenance inheritance** | Every action inherits provenance from its triggering input. Auditor can always trace "why did my Chief do this?" back to an external event. |

## Surfaces OS-exclusive (pack code never renders)

- Top status strip (per visual-language.md §13)
- Omnibar (`⌘ Space`)
- HAX Inbox (`⌘ .`)
- Ceremony UI (every Region 7/8 approval)
- Provenance Explorer
- Trust Ledger Viewer
- Quarterly Review
- OAuth login / credential entry
- Share / Contact / File / Directory / Pack pickers
- Permission revocation UI
- Cost / budget dashboard
- Any surface rendering the device key or biometric prompt

## Anti-phishing (pack pretending to be the OS)

| Attack | Mitigation |
|---|---|
| Pack renders a fake Ceremony | Ceremony always dims + locks pack surfaces; once ceremony begins, pack has zero input. OS surfaces carry a machine-visible "Chief-verified" watermark that packs cannot render (pane-sandbox API denies the pixel pattern). |
| Pack fakes OS chrome (status strip, Omnibar) | Top strip is a native OS panel (`NSPanel` / `layer-shell`), not an HTML element; packs can't reach it from their webview. Fonts/typography tokens in the OS chrome are not exported to pack SDK. |
| Pack hijacks a global shortcut | `⌘ Space` / `⌘ .` / `Esc` are registered via `tauri-plugin-global-shortcut` at kernel level; packs can't intercept. |
| Pack mimics a system notification | All pack asks land in HAX Inbox with explicit source-pack label; OS-level notifications (device-key unlock, security event) use a distinct visual + sound palette packs cannot reproduce. |
| Pack silently drains LLM budget | Per-pack `budget_usd_per_day` declared in manifest; OS enforces; visible in Trust Ledger cost pane. |
| Pack exfiltrates via `net.http` | `net.http { hosts: [...] }` is host-allowlisted; no wildcards. A separate `net.wildcard` kind (if ever added) would trigger heavier review + consent. |

## Threat-model cheat sheet

Assume malicious pack. For every operation, ask:

1. **Does the pack render the sensitive surface?** If yes → absorb into OS ceremony.
2. **Does the pack see the sensitive primitive?** (token, password, private key, full address book). If yes → replace with opaque handle.
3. **Can the pack bypass declared scope at runtime?** If yes → the capability kind is mis-designed; fix it.
4. **Could the pack collude with another pack to exceed either's scope?** If yes → mediate through broker with user consent.
5. **Could untrusted content flow through the pack into a high-stakes action without re-friction?** If yes → taint system is not tight enough; propose ADR.

## Dogfood implications

Building Gmail / Calendar / HN-briefer / file-watcher forces us to exercise this boundary for real. Friction surfaced ("it's hard to send a reply as the user when I can't even see their email") is the signal we need more SDK primitives — *never* a reason to bypass ceremonies. Those gaps become ADRs and extensions, mechanically.

## Related

- [`pack-sdk`](./40-pack-sdk.md) — what packs can do, SDK surface, CAN/CANNOT table.
- [`security-model`](./14-security-model.md) — cryptographic backing.
- [`adr/0002-capability-based-security.md`](../adr/0002-capability-based-security.md) — the foundation.
- [`adr/0010-sdk-public-api-stability.md`](../adr/0010-sdk-public-api-stability.md) — SDK as the boundary.
