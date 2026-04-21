---
id: surfaces
title: "Surfaces (L4)"
status: draft
owners: [santosh]
last_updated: 2026-04-21
related: [architecture, chief-kernel, hax-principles, apple-design]
depends_on: [architecture, chief-kernel]
tags: [ui, ux, surfaces, hax]
---

# Surfaces (L4)

## TL;DR

- 7 surfaces. Each is a declarative projection over L2 state — no business logic.
- Default surface post-boot: **Morning Brief.**
- Friction tier rendered by surface type: Queue Card (tier 1) → Evidence Card (tier 2) → Ceremony (tiers 3–4).
- Apple-style guardrails: one way to do each thing, rituals not workflows, typography as substance.

## Surface inventory

| Surface | Region(s) | Purpose | Primary input |
|---|---|---|---|
| Morning Brief | 3, 5, 6 | Daily synthesis + approval queue | Spacebar / touch |
| **HAX Inbox** | 3, 5, 7, 8 | **Primary notification surface.** Replaces all popups. | Keyboard / tap |
| **Omnibar** | 1, 5 | **Global semantic search over Memory Graph.** | Global hotkey |
| **Clipboard Pane** | 1, 5 | `kbd://` typed clipboard with agent transforms. | Auto / gesture |
| Live Agent View | 6 (observe) | Cinematic real-time agent activity | Scroll / filter |
| Provenance Explorer | 3, 4 | Walk back any artifact's chain | Click / keystroke |
| Trust Ledger Viewer | 4, 8 | See + adjust delegation | Tap (read) / ritual (write) |
| Ceremony | 7, 8 | High-stakes approvals with evidence + friction | Hold / biometric / co-sign |
| Chat Pane | 1, 5 | Ad-hoc "ask / tell Chief" | Voice / typing |
| Quarterly Review | 4, 8 | Renegotiation ritual every 90d | Guided walk |

Three new surfaces (**HAX Inbox, Omnibar, Clipboard Pane**) come from the substrate-spine research ([`research/2026-04-21-ai-native-primitive-rethinks.md`](./research/2026-04-21-ai-native-primitive-rethinks.md)). They are first-class, not add-ons.

## Morning Brief — the flagship

```
┌────────────────────────────────────────────────────────────────────┐
│  Tuesday, April 21                              Trust avg: 3.8/5  │
│                                                                    │
│  Good morning. 2 things need you. 14 are done.                     │
│                                                                    │
│  ── NEEDS YOU ──                                                   │
│  ⚠  Dentist moved to 3pm. Was that OK?           [ Keep ] [ Undo ] │
│  🔍 Stanford PDF reply (cites p.14 ¶3)           [ Review ]        │
│                                                                    │
│  ── HANDLED (tap for receipt) ──                                   │
│  ✓  Acme call → Thursday 2:30, confirmed both sides                │
│  ✓  v3 → staging 06:12, 47 tests green                             │
│  ✓  11 emails triaged (9 auto-replied, 2 queued)                   │
│  ✓  3 invoices filed → spreadsheet                                 │
│                                                                    │
│  ── CATEGORIES ──                                                  │
│  Email drafts     ███░░ 3/5   (+1 to unlock auto-send < 40w)       │
│  Calendar         ████░ 4/5   unlocked overnight ✨                 │
│  Finance          █░░░░ 1/5                                        │
│                                                                    │
│  ── LOCAL / CLOUD ──                                               │
│  🔒 Finance summary ran on-device. Nothing left your machine.       │
└────────────────────────────────────────────────────────────────────┘
```

**Behavior:**

- Renders on spacebar tap (post-boot default surface).
- Cards are live-synthesized from pre-computed agent outputs (≤3s re-layout). Honest: synthesis happens overnight; only the render is "live."
- Trust Ledger bar increments animate visibly when thresholds hit.
- Tap a "handled" card → receipt view (signed, with citations).
- Tap "Review" → Evidence Card (tier 2) for higher-stakes items.
- Scrubber at bottom (hidden by default): drag to rewind.

**Data contract:** reads `mem://digest/<date>` composed from `mem://action/<id>` nodes proposed by agents overnight, filtered by Region Router into needs-you vs handled lanes.

## Live Agent View — the cinematic one

Used for the hero demo (night handoff). Shows agents as nodes, tool calls as edges lighting up, files/URLs opening and closing. Designed to be screenshottable at any moment.

- Nodes: agents (colored by Stack), tools (shape by kind).
- Edges: tool calls (color by region — low stakes green, high stakes amber).
- Overlay: current capability grants in flight.
- eBPF-powered (see [`chief-kernel`](./03-chief-kernel.md) event bus + observability).

Not a debugger. It is **theater of agency** — the aesthetic version of `top -H`. Must hold up at 60fps even with 100 agents.

## Provenance Explorer

A clickable graph over Provenance Log entries.

- Entry point: any card, any file, any ship.
- Reveals: agent chain, sources consumed, alternatives considered and rejected, confidence, Merkle verification status.
- One-key shortcut (`P` from anywhere) → explorer for the focused artifact.
- Exportable: sign and share a subgraph as an audit artifact (for legal, partners, auditors).

## Trust Ledger Viewer

A horizontal stacked-bar strip by category, with per-category timeline:

```
 email/triage        ██████░░░░  4/5    (since 2026-01-15)
 calendar/schedule   ████░░░░░░  2/5
 finance/under-100   █░░░░░░░░░  1/5    (capped by regulation)
 legal/review        ██░░░░░░░░  2/5
```

Tap a category → timeline of approvals, rollbacks, and growth triggers. Writes happen only through rituals (approval-triggered auto-grow, Quarterly Review, or manual override with biometric).

## Ceremony

The Region 7/8 approval surface.

- Triggered by Capability Broker when Region Router selects friction tier 3 or 4.
- Payload: proposed action, evidence card, rollback window, affected parties, monetary impact, alternatives considered.
- Interaction: hold spacebar 3s OR tap YubiKey → Secure Enclave biometric gate → signed token returned.
- Token is single-use, short-lived, bound to the action.
- Full ceremony is recorded (frame-by-frame signatures) into Provenance Log.

Visual: monochrome, minimal, reverent. The *opposite* of a spammy modal. If the user cancels, the action is vetoed cleanly and the Trust Ledger records a down-vote in that category.

## Chat Pane

Ad-hoc interaction. Voice or typing. Region 1/5 work.

- Invoked by global hotkey (`Cmd+Space` equivalent).
- Reads and writes `mem://conversation/<id>`.
- Agents proposed from chat go through the same Broker path as any other action.
- Chat is **not** the default surface — Morning Brief is. Chat is an accelerator, not the home.

## Quarterly Review

A scheduled ritual. Triggered on the 90-day anniversary of setup (and every 90 days after).

Shape:

1. **Open:** Chief summarizes the season. "Here's what I did, what you approved, what you vetoed, what I learned."
2. **Middle:** Walk through each category. User confirms / lifts / lowers delegation. Chief surfaces anomalies ("you vetoed 40% of my travel drafts — want me to change approach, or hand that category back?").
3. **Close:** Chief writes a flake diff to `~/chief/house-rules.nix`, proposes it for sign, user approves → provenance anchor written (*"Renegotiated 2026-07-21"*).

This is a ritual, not a settings screen. Duration: 10–20 minutes. Modeled on season retrospectives, not product onboarding.

## Apple-style guardrails applied

| Principle | Surface implication |
|---|---|
| One way to do each thing | Single approval surface (Ceremony). Single "what happened today" (Morning Brief). Single "ask Chief" (Chat). |
| Hide complexity, reveal power | `Inspect` key on anything surfaces full JSON + provenance. Normies never need it. |
| Defaults are the product | Boot → Morning Brief, no onboarding wizard. |
| Rituals not workflows | Morning Brief, Night Handoff, Quarterly Review have openings / middles / closings. |
| Typography as substance | Morning Brief uses type hierarchy as IA. Ship a custom typeface or license Pitch/Söhne-tier. |
| Sound as chrome | Ship chime (soft), approval-needed cluck, ceremony tone. Defaults on. |
| Continuity | Pane state syncs to phone/watch from day one. No sync → no ship. |
| Physics over animation | Cards have weight; rewind has inertia. Not linear easing on opacity. |

## Accessibility

- Every surface usable with keyboard only. `?` reveals shortcuts.
- High-contrast mode that preserves brand; not a separate stylesheet.
- Voice input parity on every surface (not just Chat).
- Screen-reader ARIA on all semantic regions.

## Acceptance

- [ ] Morning Brief renders in ≤3s on modern hardware from pre-computed state.
- [ ] Ceremony cannot be bypassed (no keyboard shortcut skips the hold or biometric).
- [ ] Every surface is a declarative view; business logic lives in L2.
- [ ] All 7 surfaces have keyboard-only parity with mouse/touch.
- [ ] Live Agent View maintains 60fps with 100 active agents.
- [ ] Sync between laptop, phone, watch within 2s of state change (v1+).

## Open questions

1. Does Live Agent View ship in v0 or is it v1? (Bias: v0 — it's the hero demo.)
2. What's the typeface? Custom commission vs. license vs. open-source (e.g., Inter-adjacent)?
3. Do we ship a native phone companion at v0 or web-only for the morning-brief-on-phone use case?
4. Should the Provenance Explorer be a separate surface or an overlay invoked from any card?
5. How is the Quarterly Review scheduled relative to calendar context (e.g., avoid a busy week)?

## Related

- [`hax-principles`](./01-hax-principles.md) — which region maps to which surface
- [`chief-kernel`](./03-chief-kernel.md) — what the surfaces read
- [`apple-design`](./12-apple-design-principles.md) — deeper design guardrails
- [`viral-loop`](./10-viral-loop.md) — Morning Brief as the shareable artifact
