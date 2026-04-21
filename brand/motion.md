---
id: brand-motion
title: "Motion — spring physics, presets, anti-patterns"
status: draft
owners: [santosh]
last_updated: 2026-04-21
related: [brand-visual-language, apple-design]
tags: [brand, motion, animation]
---

# Motion

> Binding reference. Source of truth for motion tokens is [`docs/brand/visual-language.md`](../docs/brand/visual-language.md) §7.

All motion uses **spring physics** via `framer-motion` (or a Rust equivalent for the native-compositor path). Linear easing is banned. Generic `ease-in-out` is banned. Pages do not transition in Chief OS — surfaces are summoned, not navigated.

## Preset table

Named presets. Packs pick the name; they do not pass raw stiffness/damping.

| Preset | Stiffness | Damping | Mass | Feel | Applied to |
|---|---|---|---|---|---|
| `card-reveal` | 220 | 28 | 1.0 | Firm, settled | Rows appearing in the Brief |
| `trust-grow` | 180 | 22 | 1.0 | Satisfying drift | Trust Ledger bar increments |
| `omnibar-summon` | 260 | 30 | 0.9 | Snappy | Omnibar palette appearance |
| `ceremony-begin` | 120 | 20 | 1.2 | Slow, deliberate, reverent | Ceremony surface takeover |
| `inbox-drawer` | 240 | 28 | 0.95 | Crisp pull-down | HAX Inbox drawer slide |
| `chat-slide-in` | 200 | 26 | 1.0 | Weighty glide | Chat pane slide-in from right edge |
| `hold-ring-progress` | — | — | — | Linear 3s during active hold | Ceremony hold-to-approve ring (linear is intentional for progress feel) |

## Banned motion

- ❌ Linear easing anywhere (except the hold-ring progress indicator).
- ❌ CSS `ease-in-out` / `ease-in` / `ease-out`.
- ❌ Bouncy springs (damping < 20) — feel toy-like, wrong for an OS.
- ❌ "Page transitions" — we are not a web app; there are no pages.
- ❌ Global fade-in on every element mount — feels like a web wrapper.
- ❌ Parallax scrolling.
- ❌ Decorative motion that carries no information.

## Motion-carries-information principle

Every animation must tell the user something:

| Animation | Information conveyed |
|---|---|
| Card reveals downward with slight bounce | "This is new; here it is; notice it" |
| Trust bar grows left to right | "Delegation increased; you earned it overnight" |
| Omnibar snaps in with slight overshoot | "You are now focused here; the system is listening" |
| Ceremony slow-fade to dark | "Everything else is receding; this is what matters now" |

If a motion serves no informational purpose, remove it.

## Reduce-motion accessibility

Respect the OS's `prefers-reduced-motion` preference:

- Springs collapse to instant state transitions.
- Ceremony surface appears without fade — just there.
- The HoldRing still animates its progress arc because that motion IS the information (3-second hold); this is the one exception.
- Never use motion as the sole cue for state change — always pair with color/position/shape.

## Implementation notes

### React (`chief-ui`)

```tsx
import { motion } from 'framer-motion';
import { motionPresets } from '@chief-os/ui/tokens';

<motion.div {...motionPresets.cardReveal}>
  <Row ... />
</motion.div>
```

Presets export typed objects matching framer-motion's API. Raw stiffness/damping/mass is not re-exported.

### Rust / native (future)

For the native-compositor path (post-v0 or partial for Ceremony surface), we replicate the spring values in a small Rust crate (`chief-motion` or embedded in `chief-ui`). Same preset names, same values.

### Timing budget

Total motion duration from trigger to settled state should rarely exceed **400 ms**, except for Ceremony (where slowness is the point, up to ~900 ms).

## Haptics companion (trackpad-equipped hosts)

| Motion | Haptic |
|---|---|
| `card-reveal` | none |
| `trust-grow` (when the bar crosses a threshold) | single light tap at the moment of threshold |
| `omnibar-summon` | subtle ping on appearance |
| `ceremony-begin` | progressive hum during fade-to-dark |
| `hold-ring-progress` | progressive haptic intensifying with arc, firm tap on successful sign |
| Swipe-to-approve on Brief card | light tap on completion |

## Decision log

- **2026-04-21** — Seven preset names locked. Values derived verbatim from `brand/visual-language.md` §7.

## Related

- [`brand/visual-language.md`](./visual-language.md) §7 — motion tokens source
- [`brand/sound.md`](./sound.md) — sound companions
- [`adr/0011-ui-stack-and-component-library.md`](../adr/0011-ui-stack-and-component-library.md) — presets exposed via `chief-ui`
