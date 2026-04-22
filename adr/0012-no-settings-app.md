---
id: adr-0012
title: "No Settings app — distributed control surfaces with a unified Security & Privacy view"
status: accepted
date: 2026-04-21
deciders: [santosh]
supersedes: []
superseded_by: []
tags: [ux, security, privacy, hax, surfaces]
---

# ADR-0012 — No Settings App

## Context

macOS and iOS ship a unified Settings / System Preferences app. It has ~500 knobs, it is search-driven, and it flattens consequentiality — changing "camera access" (Region 7) and "seconds in menubar" (Region 1) look identical in the UI. For a HAX-first OS where agents act on the user's behalf with real consequence, that flattening is hostile: it trains the user that all controls are equally weighty and reversible, which is exactly the opposite of what HAX demands.

But — the countervailing pressure is also real. Users need a single place to answer "am I safe? what has access to what?" without assembling the answer from six surfaces. That's a legitimate requirement that "no Settings app" initially seemed to reject, and it shouldn't.

This ADR resolves the tension.

## Decision

**Chief OS does not ship a Settings app.** Control is decomposed across surfaces matched to HAX region, with one consolidated **read-dominant** security view and one tight **cosmetic-preferences** surface:

| Surface | Role | HAX region | Writes? |
|---|---|---|---|
| **Security & Privacy** (new) | Universal inventory: who has access to what, what they've done, when | 6 (observe) | Read-dominant; writes cascade to Ceremony or direct-revoke |
| **Trust Ledger Viewer** | Delegation state by category; trust-tier changes | 4, 8 | Yes (via Ceremony) |
| **Controls** (new) | ~30 hand-curated cosmetic / functional preferences | 1–2 | Yes, low-friction |
| **Ceremony** | The only write path for any authority-increasing change | 7, 8 | Yes |
| **Quarterly Review** | 90-day rhythm for big-picture delegation renegotiation | 4, 8 | Yes (via Ceremony) |

The **Omnibar** is the universal entry point — type "permissions," "camera," "privacy," "wallpaper," and you land on the right surface immediately.

## Why `Security & Privacy` (not `Posture`, not `Access`)

Evaluated three names:

| Name | Pros | Cons |
|---|---|---|
| `Posture` | HAX-distinctive, teaches "ongoing supervision" model | jargony; not user-legible on first boot |
| `Access` | compact, matches "what has access to what?" mental model | ambiguous; could be read as accessibility |
| **`Security & Privacy`** | macOS muscle-memory, unambiguous, adoption-curve wins | inheriting a term from the platform we want to transcend |

**Chose `Security & Privacy`.** Users-first trumps distinctiveness-first. The HAX-native meaning is enforced by what the surface *does*, not what it's called.

## Why not a single "Settings" app

| Anti-pattern | How we avoid it |
|---|---|
| 500 flattened knobs | 3 surfaces, each bounded. Controls is capped at ~30 cosmetic items. |
| Consequentiality-blind UI | Ceremony is the only write path above Region 2. Asymmetric by design. |
| Reactive knob-hunting | Ongoing rhythms: Morning Brief, Trust Ledger, Quarterly Review are first-class. |
| Dumping ground for new settings | Controls has a hard size cap. New settings get rejected unless they strictly match its scope. |
| Cosmetic and security at the same-click distance | Physically separate surfaces with different invocation paths. |

## Security & Privacy — the consolidated view

One surface, six projections over the same **signed, typed grant registry**:

1. **By Pack** (default projection) — what every installed pack can do, what it's done recently, revoke inline.
2. **By Capability** — grouped into the 8 pillars below; drill down into the 27-kind closed enum from [`docs/40-pack-sdk.md`](../docs/40-pack-sdk.md).
3. **By Agent** — per-agent activity rollup.
4. **By Data** — data-class lineage ("who has touched my email this week?").
5. **By HAX region** — what's running in Region 7–8 right now.
6. **Recent activity timeline** — signed event-log entries, allow/deny per check.

**OS-itself grants are visible.** The kernel's own holdings (keychain, disk, device keys) are shown alongside pack grants. No hidden "trusted infrastructure" category. The user deserves full transparency over what the OS does on their behalf.

**Read-dominant; writes cascade.** The only writes that originate from Security & Privacy are (a) direct revoke (safety-favoring — no ceremony), and (b) "view history" / "export audit package" (no writes). Any increase in authority jumps to Ceremony.

**Export as first-class.** Every view exports as a cryptographically-signed audit subgraph (JSON-LD + Merkle proofs) and a human-readable markdown summary.

## Controls — the tight preferences surface

Scope cap: **~30 items total, hand-curated.** Wallpaper, sound on/off, keyboard layout, timezone, locale, accessibility toggles, device pickers (Wi-Fi, Bluetooth, audio out). Nothing that has a consequentiality axis. No "privacy" section. No "security" section. No "apps" section.

If a proposed new setting writes anything with user-, data-, or system-consequence, it does not live in Controls — it lives in Trust Ledger, Security & Privacy, or Ceremony as appropriate.

## The 8-pillar capability grouping

The 27-kind `CapabilityKind` closed enum ([`docs/40-pack-sdk.md`](../docs/40-pack-sdk.md)) is too granular as a top-level grouping. We collapse to 8 pillars, each drilling down into its member kinds:

| Pillar | Member CapabilityKinds | User-legible question |
|---|---|---|
| **Data** | `mem.read`, `mem.write`, `mem.link`, `mem.subscribe`, `fs.read`, `fs.write`, `fs.watch`, `ledger.read`, `contact.pick`, `file.pick` | "Who can read my data?" |
| **Network** | `net.http`, `net.ws`, `net.oauth2` | "Who is talking to the internet?" |
| **Inference** | `llm.generate`, `llm.embed` | "Who is calling AI models on my behalf?" |
| **Agents** | `agent.spawn`, `meta.prompt`, `region.route`, `event.emit`, `event.subscribe` | "Who spawns agents or decides how they route?" |
| **Surfaces** | `surface.pane`, `notify.inbox`, `ceremony.request`, `clock.schedule` | "Who draws UI, posts notifications, or schedules work?" |
| **Devices** | `device.mic`, `device.camera`, `screen.capture`, `kbd.read_on_paste`, `kbd.write` | "Who accesses my mic / camera / screen / keyboard?" |
| **External Actions** | `share.hand_off`, `payment.request`, `esign.request` | "Who can send money, share files, or sign documents?" |
| **Tools & Meta** | `tool.invoke`, `tool.register`, `taint.read` | "Who invokes or registers tools, or reads provenance metadata?" |

This pillar grouping is the top-level taxonomy in the By Capability view. Users can drill down to the 27 kinds when needed. The 8 pillars are stable; adding a new pillar requires amending this ADR.

## Safety-favoring asymmetry

A core rule applied across all control surfaces:

| Action | Friction |
|---|---|
| **Grant a new capability** | Ceremony |
| **Increase a grant's scope** | Ceremony |
| **Raise trust tier for a category** | Ceremony (per `docs/30-surfaces.md` §Quarterly Review) |
| **Enroll a new device / key** | Ceremony + biometric |
| **Revoke a grant** | One click. No ceremony. |
| **Tighten a grant's scope** | Inline. No ceremony. |
| **Shrink horizon / add expiry** | Inline. No ceremony. |
| **Wipe device, export data, delete memory** | Ceremony (long-horizon irreversible) |

Rule: **making things safer is frictionless; giving away authority is deliberate.**

## Emergency revoke-all

One chord (TBD — likely `⌃⌘⎋` or menubar "Panic" extra) kills all pack grants and force-sleeps all running agents in a single action. No ceremony (this is safety-favoring). Produces a signed event-log entry; state is restorable via Quarterly Review.

## Invocation paths (summary)

| Want to… | Go to |
|---|---|
| Audit who has access to what | Security & Privacy (by Pack / by Capability) |
| Revoke a pack's grant | Security & Privacy (by Pack) → Revoke |
| See what happened yesterday | Security & Privacy (Recent activity) |
| Increase delegation | Trust Ledger Viewer → Ceremony |
| Change wallpaper, sound, timezone | Controls |
| Pick Wi-Fi / audio output | Status-strip glyph popover |
| Enroll a new YubiKey | Ceremony (triggered from Security & Privacy) |
| Export a signed audit package | Security & Privacy → Export |
| Big-picture boundary renegotiation | Quarterly Review (auto-scheduled) |
| Find anything | Omnibar |

## Consequences

**Positive:**
- No dumping ground. Each surface has a clear purpose and a size cap where relevant.
- Consequentiality is baked into UX friction, not buried in an "Advanced..." submenu.
- Users get a universal read-dominant safety view without re-creating macOS's flattened-knob pattern.
- HAX regions are legible in the UI rather than an abstract theoretical frame.
- "Who has access to what?" is answerable as a single question.

**Negative / tradeoffs:**
- More surfaces to maintain (2 new: Security & Privacy, Controls) for a total of 12.
- First-week users expecting a "Settings" icon will not find one; need onboarding cue ("Try `⌘Space` and type `settings`" → Omnibar routes them).
- `Security & Privacy` as the name is the least HAX-native of the options we considered; accepted tradeoff for user-legibility.

## Open questions (deferred to implementation phase)

1. Exact menubar glyph for Security & Privacy: shield, lock, or something new? Default: shield.
2. Emergency revoke-all chord: specific key combination TBD during keyboard-shortcuts work.
3. Whether the Recent activity timeline is paginated or infinite-scroll. Default: paginated by day.
4. Export package format: lock to JSON-LD + Merkle proofs at v0; markdown summary via templating. Revisit if auditors want a specific standard (e.g., CycloneDX).
5. Per-pack usage history retention: default 90 days, configurable in Controls. (Configurable is a Controls-eligible item because it's a numeric preference, not an authority change.)

## Related

- [`docs/30-surfaces.md`](../docs/30-surfaces.md) — to be updated with new surfaces.
- [`docs/32-controls-and-policy.md`](../docs/32-controls-and-policy.md) — implementation spec for Security & Privacy + Controls.
- [`docs/40-pack-sdk.md`](../docs/40-pack-sdk.md) §CapabilityKind — canonical 27-kind closed enum.
- [`adr/0002-capability-based-security.md`](./0002-capability-based-security.md) — capability mechanism these surfaces project over.
- [`adr/0004-machine-as-fax-posture.md`](./0004-machine-as-fax-posture.md) — user-as-legal-actor, motivates safety-favoring asymmetry.
- [`adr/0010-sdk-public-api-stability.md`](./0010-sdk-public-api-stability.md) — the SDK that exposes grants for Security & Privacy to project over.
- [`adr/0011-ui-stack-and-component-library.md`](./0011-ui-stack-and-component-library.md) — `chief-ui` primitives will add `SecurityPrivacyPane` + `ControlsPane` composites in a follow-up.
