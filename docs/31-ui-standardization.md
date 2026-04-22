---
id: ui-standardization
title: "UI Standardization (L4 companion to pack-sdk)"
status: draft
owners: [santosh]
last_updated: 2026-04-21
related: [pack-sdk, surfaces, brand-visual-language, adr-0011]
depends_on: [adr-0011, pack-sdk]
tags: [ui, ux, sdk, developer-contract, chief-ui]
---

# UI Standardization

> **Binding.** Every surface in Chief OS — first-party, dogfood, or third-party pack — is composed from the `chief-ui` primitive library. No exceptions. Per [`adr-0011`](../adr/0011-ui-stack-and-component-library.md).

## TL;DR

- **Pack UI = composition of `chief-ui` primitives + Chief OS tokens + locked HAX badge variants.** That's it.
- Packs never import Radix, Tauri, or raw DOM. The kit hides those.
- First-party packs (HN briefer, Gmail triage, etc.) use the exact same public surface as third-party packs — dogfood parity.
- Enforcement is three-layered: SDK types, build-time import scan, install-time manifest check.

## Why "one UI to rule them all"

macOS works because every app inherits from AppKit / SwiftUI. Buttons look like buttons because everyone uses `UIButton`. Menubar integration is automatic. Accessibility is free.

Without this, a multi-pack OS fragments into a zoo: one pack uses Tailwind, another ships its own fonts, a third rolls its own modal pattern. The user experiences 50 visual identities instead of one coherent Chief OS. The brand falls apart.

Solve-once: build one kit, enforce its use, and every new pack strengthens the brand instead of diluting it.

## The primitive inventory (v0)

Detailed catalog of the `chief-ui` components. Each item lists: purpose, prop surface (sketched), and where it is used in the shipping design.

### Structural

#### `<Pane>`
Base Liquid Glass panel — vibrancy material, 10px corner radius, hairline border. All content surfaces start here.

```tsx
<Pane variant="content" tint="warm">
  {children}
</Pane>
```

- `variant`: `content` (Brief main) | `widget` (right-side) | `overlay` (omnibar palette)
- `tint`: `warm` (default) | `dark` (ceremony) | `neutral`

Used in: every surface.

#### `<Menubar>` + `<MenubarExtra>`
OS-level menubar registration. Rendered natively via Tauri. Surfaces declare their menu tree; chief-ui wires it to the OS.

```tsx
<Menubar>
  <MenubarGroup label="Brief">
    <MenubarItem action="review-next">Review next</MenubarItem>
    <MenubarItem action="rewind" shortcut="⌘Z">Rewind</MenubarItem>
  </MenubarGroup>
</Menubar>
```

#### `<TabBar>`
Count-chip horizontal tab bar. Used at top of Brief.

```tsx
<TabBar
  chips={[
    { id: 'all',       label: 'All',        count: 24 },
    { id: 'needs-you', label: 'Needs you',  count: 2, highlighted: true },
    { id: 'handled',   label: 'Handled',    count: 14 },
    { id: 'ceremony',  label: 'Ceremony',   count: 1 },
  ]}
  selected="needs-you"
  onSelect={…}
/>
```

#### `<SectionHeader>`
Time or group divider. Tracked small-caps 11pt.

```tsx
<SectionHeader>TODAY · MORNING</SectionHeader>
```

#### `<CommandPill>`
Floating bottom toolbar. Compact launcher for Omnibar / HAX Inbox / Chat by default; packs can declare their own action slots.

### Content

#### `<Row>`
The universal content row. Avatar + title + snippet + badge + timestamp.

```tsx
<Row
  avatar={<SourceAvatar color="amber" />}
  sender="Calendar Agent"
  subject="Dentist moved to 3pm"
  snippet="was that OK? — rescheduled by Dr. Park's office"
  badge={<Badge variant="needs-you">Needs you</Badge>}
  timestamp="08:03"
  onOpen={…}
/>
```

#### `<SourceAvatar>`
20px rounded-square colored indicator for source agent. Closed palette of 8 muted hues.

- `color`: `amber` | `blue` | `green` | `copper` | `gray` | `teal` | `violet` | `peach`

Intent-linked. Pick the color the agent is intrinsically associated with (Calendar = amber, Negotiator = blue, Email = green, etc.), not arbitrary branding.

#### `<Badge>`
HAX-region status pill. **Closed enum — no custom variants.**

- `variant`: `needs-you` (R3, amber) | `review` (R5, blue) | `handled` (R2, green) | `ceremony` (R8, copper) | `anchor` (R6, gray)

```tsx
<Badge variant="handled">Handled</Badge>
```

### Widgets (right-side pane)

#### `<Widget title>`
Small glass card with tracked header.

```tsx
<Widget title="TRUST">
  <DotTrail label="Email"    filled={4} total={5} color="blue" />
  <DotTrail label="Calendar" filled={4} total={5} color="amber" />
  <DotTrail label="Finance"  filled={1} total={5} color="green" />
</Widget>
```

#### `<DotTrail>`
Five-segment horizontal progress/trust indicator.

#### `<Sparkline>`
Compact bar chart for activity signals (24h agent activity, trust over time).

```tsx
<Sparkline
  data={hourlyActionCounts}
  highlightIndex={currentHour}
  label="47 actions today · 12 agents active"
/>
```

### High-stakes

#### `<HoldRing>`
Copper-accent hold-to-approve circular affordance. Used in ceremony. Progress 0 to 1 during the 3-second hold.

#### `<CeremonyCard>`
Full-screen dark ceremony surface composition. Wraps headline, evidence table, provenance rows, rollback window, and `<HoldRing>` with the reverent layout locked in `docs/brand/visual-language.md` and the `03-ceremony.png` reference.

### Global surfaces (OS-level, invoked by keyboard shortcut)

#### `<OmnibarPalette>`
Modal overlay for global semantic search. Input + result list with typed items (`Memory`, `Card`, `Agent`, `Pack`, `Action`).

#### `<InboxDrawer>`
HAX Inbox anchored drawer. Pulls down from the inbox menubar extra.

#### `<ChatPane>`
Ad-hoc conversation slide-in side panel.

These are owned and rendered by the kernel, not by packs. Packs can post into them via the SDK (e.g., agents emit items into Inbox), but cannot replace or style them.

## Token system

All tokens are CSS custom properties backed by matching Rust constants. Single source of truth in the kit; values derived verbatim from [`brand/visual-language.md`](./brand/visual-language.md).

### Color (dark-first; light in v2+)

```css
:root {
  --chief-bg-0:       #0c0e11;
  --chief-bg-1:       #14171b;
  --chief-bg-2:       #1b1f24;
  --chief-border:     rgba(255,255,255,0.06);

  --chief-fg-primary:   #e8ecf1;
  --chief-fg-secondary: #98a2b0;
  --chief-fg-tertiary:  #5c6470;

  /* HAX region accents — closed set */
  --chief-amber:  #d4a574;  /* needs-you (R3) */
  --chief-blue:   #5294e2;  /* review (R5) */
  --chief-green:  #73c991;  /* handled (R2) */
  --chief-copper: #c68858;  /* ceremony (R8) */
  --chief-gray:   #8a8f99;  /* anchor (R6) */

  /* Source-agent palette — closed set of 8 muted hues */
  --chief-src-amber:  #d4a574;
  --chief-src-blue:   #7a9cc6;
  --chief-src-green:  #8fb093;
  --chief-src-copper: #c68858;
  --chief-src-gray:   #8a8f99;
  --chief-src-teal:   #8ec0b8;
  --chief-src-violet: #9d89b8;
  --chief-src-peach:  #e0b090;
}
```

### Typography

```css
:root {
  --chief-font-display: "Inter Tight", system-ui, sans-serif;
  --chief-font-body:    "Inter",       system-ui, sans-serif;
  --chief-font-mono:    "JetBrains Mono", ui-monospace, monospace;

  --chief-size-headline: 34pt;  /* hero statement, Brief "Good morning." */
  --chief-size-section:  11pt;  /* tracked small-caps time dividers */
  --chief-size-body:     13pt;  /* row subject + snippet */
  --chief-size-meta:     11pt;  /* timestamps, metadata */

  --chief-tracking-section: 0.08em;
}
```

All numeric UI (timestamps, counts, amounts) uses `--chief-font-mono` with `font-variant-numeric: tabular-nums`.

### Space + radius

```css
:root {
  --chief-space-xs: 4px;
  --chief-space-s:  8px;
  --chief-space-m:  12px;
  --chief-space-l:  20px;
  --chief-space-xl: 40px;

  --chief-radius-card:   10px;
  --chief-radius-button: 6px;
  --chief-radius-pill:   999px;
}
```

### Motion

Framer-motion spring tunings are locked in `docs/brand/visual-language.md` §7. The kit exposes them via motion preset names (`card-reveal`, `trust-grow`, `omnibar-summon`, `ceremony-begin`); packs pick the named preset, not arbitrary stiffness/damping.

## What packs CAN do

- Compose primitives from the catalog above into any layout.
- Pick the intent-appropriate `<Badge variant="…">`.
- Pick a `<SourceAvatar color="…">` from the 8-hue palette.
- Provide short copy, including inline italic/bold/code where semantically meaningful.
- Declare menubar items via `<Menubar>`.
- Use the reserved `<Box>` primitive for pack-controlled layout (CSS grid or flex only — no color, type, or motion properties).

## What packs CANNOT do

- Import Radix directly, Tauri APIs directly, or raw DOM.
- Ship custom fonts or load fonts from the network at runtime.
- Render arbitrary HTML or inline SVG for UI chrome.
- Invent badge variants or colors outside the closed palettes.
- Write CSS for `color`, `font-*`, `animation`, `transition`, `box-shadow`, or `backdrop-filter` — these are kit-controlled.
- Style primitives. A `<Badge variant="handled">` always looks the same.
- Draw custom liquid-glass material — use `<Pane>`.
- Ship toast notifications or modal dialogs — use HAX Inbox via `ctx.inbox.post()`.
- Replace, cover, or restyle the Omnibar, HAX Inbox, Chat, or Ceremony surfaces.

## Enforcement (three layers)

### 1. SDK compile-time

`@chief-os/sdk` re-exports `chief-ui` primitives and nothing else for UI. Radix and Tauri are not re-exported. TypeScript `import { foo } from '@chief-os/sdk'` fails to resolve anything not in the public surface. Radix primitives are importable only from `@chief-os/ui/internal` which is outside the SDK's advertised entry points, and pack builds whitelist allowed entry points.

### 2. Pack build-time (hardest enforcement layer)

The pack-build toolchain (Nix-flake-driven, see [`docs/40-pack-sdk.md`](./40-pack-sdk.md)) performs a **static import scan** on every pack before producing a signed artifact:

- Every top-level import MUST be from the allowed list (`@chief-os/sdk`, `@chief-os/ui`, pack-local files, and a small number of well-known utility libraries).
- Any `import { … } from "react-dom"`, `"tauri"`, direct `"@radix-ui/*"`, or arbitrary web font URL → hard build failure with a precise diagnostic.
- CSS files are parsed and forbidden properties (`color`, `font-*`, animation-related) trigger failures.

### 3. Install-time manifest check

Signed pack manifests declare exactly which primitives they render and which HAX badge variants they emit. At install time, the Capability Broker cross-checks declared use against the built bundle. Mismatch → pack refused.

## First-party parity

Every first-party pack — starting with the HN briefer as the Wave 5a dogfood — consumes the **public** `@chief-os/ui` API. No internal backdoor, no special `@chief-os/ui-internal` import for kernel favorites. If the HN briefer cannot be built with the public kit, we extend the kit in the open.

This is how we discover kit gaps early and keep the dogfood → third-party path honest.

## Accessibility guarantees

The kit enforces:

- Full keyboard navigation on every primitive (inherited from Radix).
- WAI-ARIA roles and labels baked into each primitive.
- Respect for the OS's `prefers-reduced-motion` setting — animations collapse to instant when on.
- Respect for the OS's `prefers-color-scheme` (dark-first v0; light v2+).
- Focus rings use the OS-native focus color via Tauri, not CSS `outline`.
- Text-selection color set via Tauri's native color API.

Packs inherit all of this automatically.

## Versioning

- `@chief-os/ui` and `chief-ui` crate version in lockstep with `@chief-os/sdk` per [`adr-0010`](../adr/0010-sdk-public-api-stability.md).
- Adding a primitive: minor bump; pack recompile not required.
- Changing a primitive's prop surface: major bump; requires ADR and a migration path.
- Retiring a primitive: major bump; must ship a replacement primitive with a shim for one full major cycle.

## Open questions

1. **Pack theming** — does a pack get a small logo slot in the Widget header, or stays Chief-typographic only? Lean toward "logo slot inside header, 16px max, kit-controlled size" but TBD.
2. **Primitive extension** — the SwiftUI `.buttonStyle(...)` pattern. Do we accept post-v0 PRs proposing new primitives? Probably yes, with ADR gate.
3. **Voice UI** — how does a voice-invoked surface compose chief-ui? Likely a `<VoiceSession>` wrapper around `<ChatPane>` — scoped later.
4. **Pack preview sandbox** — a lightweight in-dev viewer for pack authors to see their UI on the real chief-ui substrate without full install. Tooling decision, not in ADR-0011 scope.

## Acceptance

Before v0 ships:

- [ ] `@chief-os/ui` package published with v0 primitive catalog above.
- [ ] `chief-ui` Rust crate mirrors type surface.
- [ ] Tauri-plugin-window-vibrancy wired on macOS.
- [ ] Build-time import scan enforced in the pack-build toolchain.
- [ ] HN briefer dogfood pack uses only the public kit.
- [ ] Morning Brief (first-party) migrated to the public kit — no internal backdoors.
- [ ] Reference screenshots in [`/screenshots`](../screenshots/) renderable from the real kit in a visual-diff test (within tolerance).

## Related

- [`adr-0011`](../adr/0011-ui-stack-and-component-library.md) — the decision this document implements.
- [`brand/visual-language.md`](./brand/visual-language.md) — design spec; token values derive from here.
- [`40-pack-sdk.md`](./40-pack-sdk.md) — pack-level developer contract.
- [`30-surfaces.md`](./30-surfaces.md) — what each surface is + does.
- [`02-apple-design-principles.md`](./02-apple-design-principles.md) — product guardrails that motivate this standardization.
- [`screenshots/`](../screenshots/) — reference mockups showing the locked language.
