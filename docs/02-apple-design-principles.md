---
id: apple-design
title: "Apple-Grade Design Principles"
status: draft
owners: [santosh]
last_updated: 2026-04-21
related: [surfaces, viral-loop, hax-principles]
tags: [design, ux, guardrails]
---

# Apple-Grade Design Principles

## TL;DR

- 10 principles. Non-negotiable guardrails for every surface, every ritual, every moment.
- The goal is not "use SF Symbols." It is Apple's *intentionality bar* applied to an AI-native OS.

## The 10 principles

### 1. One way to do each thing

Single approval surface (Ceremony). Single Morning Brief. Single Chat. Single Pack governance pane. No "advanced settings" where a second way sneaks in.

**Guardrail:** If two surfaces or flows would do the same thing, merge them or kill one.

### 2. Hide complexity, reveal power

Normies live on Morning Brief. Power users press `I` on any element to open the inspector (raw JSON, provenance chain, capability grants, memory URIs). Same state, two views.

**Guardrail:** Every user-visible item has an inspector. Every inspector opens from a single keystroke. No buried menus.

### 3. Defaults are the product

Boot lands on Morning Brief with the Chief-of-Staff Stack pre-active. Zero config to "holy shit." Users can customize, but nothing requires them to.

**Guardrail:** First-time install cannot ask more than **2 questions** (pair device with phone; connect Gmail). Everything else is opinionated.

### 4. Rituals not workflows

Night Handoff, Morning Brief, Quarterly Review are rituals: *opening / middle / closing*. Not multi-step wizards. Not optional.

**Guardrail:** Any feature called a "workflow" is not shippable until it has a ritual shape (why now, what you're doing, done).

### 5. Typography as substance

Morning Brief uses type hierarchy as information architecture. Cards read as a newspaper front, not a dashboard.

**Guardrail:** Ship a custom commissioned typeface or license Pitch/Söhne/Sentinel-tier. Default system fonts are *not acceptable* for v0 hero surfaces.

### 6. Sound as chrome

Three tones: ship-sent chime (soft), approval-needed cluck (clear), ceremony-begin tone (reverent). Defaults on; one toggle to mute all.

**Guardrail:** Commission sound design before day 60. No stock WAVs.

### 7. Continuity

Pane state syncs to phone and watch within 2 seconds. State flows with the human, not the device.

**Guardrail:** If a surface doesn't sync, it doesn't ship. No "phone version 2.0 coming soon."

### 8. Physics over animation

Cards have weight. Rewind has inertia. Approval ripples outward like water. No linear easing on opacity.

**Guardrail:** Review every animation for physics grounding. If it's a "fade-in," it shouldn't be.

### 9. Demos are product

Every feature is judged by: *does it improve the Morning Reveal or the Stack flywheel?* If not, v2+.

**Guardrail:** No feature merges without a demo shot. Features ship with video clips.

### 10. Privacy as UX

The "on-device" state is as legible as the battery icon. When sensitive work runs, users *see* it. When data doesn't leave the machine, users *feel* it.

**Guardrail:** Every cross-trust-boundary operation has a visible indicator. Not a settings flag.

## Anti-principles (what we refuse)

| Anti-pattern | Why we refuse |
|---|---|
| Badges for badges' sake | Gamification theater. Trust Ledger is earned, not gamified beyond meaningful milestones. |
| Tooltips as documentation | If it needs a tooltip to understand, redesign. |
| "Are you sure?" modals | See [`01-hax-principles.md`](./01-hax-principles.md) verification efficiency. |
| Onboarding carousels | Learn by doing, not by reading screens. |
| Settings pages with 12 tabs | Violates principle 1. |
| Notifications without HAX routing | Every notification is a queued approval with a region. |
| Skeuomorphism for its own sake | Physics yes; wood-grain leather no. |

## Design review cadence

| Cadence | Scope | Who |
|---|---|---|
| Daily (async) | Active surface work | Design + surfaces engineer |
| Weekly (sync) | Ritual integrity, typography, sound | Design + founder |
| Before each gate (30/60/90) | Principle compliance audit | Full team |
| Pre-launch | Hero demo dry-runs | Full team + 3 external reviewers |

## The Apple test

Before shipping any surface, ask:

1. Would Apple ship this? If not, why does Chief OS?
2. Is there a single sentence that describes what this does? If not, cut until yes.
3. Can a 70-year-old use this? (Chief OS is for prosumer-founders, but clarity travels.)
4. Does it hold up at 4K on a 2026-spec OLED MacBook? (Because that's where it'll be shown.)

## Acceptance

- [ ] Every surface has a style-guide entry with a specimen screenshot.
- [ ] Type specimen and sound palette locked by day 30.
- [ ] Inspector (`I` key) works on every element in Morning Brief.
- [ ] Continuity across laptop + phone mirror verified by day 60.
- [ ] External design review by day 80 (pre-launch).

## Open questions

1. Commissioned typeface (custom) vs. license (Pitch / Söhne / Sentinel)?
2. Sound partner — bespoke composer or licensed library?
3. Who owns the "Apple test" — founder, design lead, or external advisor?
4. Where's the style guide hosted — in-repo (`brand/`) or external Figma?
5. Can we afford an external design-ops reviewer for the demo week?

## Related

- [`surfaces`](./30-surfaces.md) — implementation of these principles
- [`viral-loop`](./60-viral-loop.md) — demo as product, principle 9
- [`hax-principles`](./01-hax-principles.md) — verification efficiency
