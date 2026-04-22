---
id: adr-0011
title: "UI stack: Tauri v2 + Radix + chief-ui component library (binding)"
status: accepted
date: 2026-04-21
deciders: [santosh]
supersedes: []
superseded_by: []
tags: [ui, ux, sdk, frontend, brand]
---

# ADR-0011 — UI Stack & Component Library Standardization

## Context

Chief OS must look and feel like a single coherent operating system, not 50 disparate apps stitched together. macOS achieves this because every app uses AppKit / SwiftUI primitives — `UIButton`, `List`, `NavigationStack`, native menubar — and cannot draw UI that bypasses the HIG-enforced components.

If Chief OS packs can each draw arbitrary HTML/CSS, we get 50 visual identities and the OS fragments into a web-app zoo. The dogfood/third-party parity promise of [`adr-0010`](./0010-sdk-public-api-stability.md) demands the same UI substrate across first-party and third-party packs.

Separately, the visual design language locked in `docs/brand/visual-language.md` and [`screenshots/README.md`](../screenshots/README.md) needs a concrete implementation target — a pinned set of technologies that together deliver the warm-liquid-glass aesthetic across macOS and Linux.

## Build-vs-borrow: do not write chief-ui from scratch

Surveyed the landscape; honest result:

| Option | Verdict |
|---|---|
| **`libadwaita` / GNOME HIG kit** | Real OS kit, GTK-native. Incompatible with a Tauri webview surface; abandoning Tauri for GTK loses our pack-sandbox model. Rejected. |
| **Qt Quick Controls / Kvantum** | Same story, Qt-native. Rejected. |
| **Slint (Rust-native UI)** | Strong Rust-native OS feel, cross-platform. Would require abandoning web-based pack surfaces entirely. Too big a pivot at v0; revisit v2+. |
| **Material Web / Fluent UI / Ant / Carbon** | Fully themed design systems. Re-theming them into the Chief language costs more than starting from primitives; their identity leaks. Rejected. |
| **Mantine / Chakra / NextUI** | Opinionated; theme overrides require fighting their defaults. Rejected. |
| **Radix UI (headless primitives)** | Accessibility-complete, unstyled, small surface. ✅ Foundation layer. |
| **Shadcn UI (source to copy, not import)** | MIT source built on Radix. Not a dependency — we copy the component source into our repo and re-theme aggressively. Gives us 25+ battle-tested component patterns (Dialog, Combobox, Tooltip, Popover, Toast replacement, etc.) for free. ✅ Starter scaffolding. |
| **Park UI (Ark + Panda CSS)** | Similar spirit to Shadcn; newer, less battle-tested. Solid fallback if Shadcn's Tailwind coupling becomes friction. |

**Decision: `chief-ui` = Radix (primitives) + Shadcn source copied and re-themed with Chief tokens (component scaffolding) + Tauri native plugins (OS chrome).** We own the resulting kit; shadcn source is an internal starting point, not a runtime dependency — packs never see it. This saves roughly 200 hours of component work while keeping the kit entirely ours to evolve.

## Decision

**We ship a unified UI stack that ALL surfaces — first-party and third-party packs — must use.** Raw HTML/CSS, arbitrary web fonts, or custom component systems are not permitted inside packs.

| Layer | Technology | Why |
|---|---|---|
| Window chrome + native integration | **Tauri v2** | Real binary, native menubar, OS keyboard shortcuts, native popovers/dialogs/clipboard. Web-wrapped kiosks (Electron, raw Chromium) rejected. |
| Native material (liquid glass) | **macOS:** `tauri-plugin-window-vibrancy` → `NSVisualEffectView`. **Linux (Chief-on-NixOS):** bundled Hyprland + layer-shell + Chief blur preset per [ADR-0015](./0015-chief-os-compositor.md). **Other Linux distros:** CSS `backdrop-filter` fallback, degraded. **Windows:** out of scope at v0. | Real liquid glass on the two platforms we ship; honest fallback elsewhere. |
| Component primitives (unstyled + accessible) | **Radix UI** (React) | Headless, WAI-ARIA-complete. We style them with our own tokens. No opinionated theme to fight. **Radix is a runtime dep; Shadcn is source-vendored — see the row below.** |
| Shadcn UI as **internal scaffolding** (NOT as a runtime dependency) | MIT source copied into `packages/chief-ui-ts/internal/shadcn/`, re-themed with Chief tokens, re-exported only through `chief-ui`'s public entry. Shadcn never appears in the pack-visible import graph. | 25+ battle-tested component patterns (Dialog, Combobox, Tooltip, Popover, Toast, ScrollArea, VisuallyHidden, …) as a starting point — **not** a package dependency, **not** Tailwind-coupled at the public surface. Packs cannot import Shadcn directly because it is not a resolvable module name from their perspective. |
| Design tokens | Hand-rolled CSS custom properties + matching Rust constants, derived verbatim from `docs/brand/visual-language.md`. | Single source of truth. No Material Design / Chakra / Mantine token soup. |
| Fonts | **Inter Tight** (display) + **Inter** (UI) + **JetBrains Mono** (data). Bundled into Tauri binary at build time. | Zero runtime font fetches. No Google Fonts. |
| Icons | **SF Symbols** on macOS via `sfsymbols-react`, **Phosphor Thin** fallback on Linux. Consistent 1.5px stroke weight. | Native-feeling across platforms. |
| Motion | **Framer Motion** with spring physics values locked in `docs/brand/visual-language.md` §7. | Weighty-but-responsive consistent across every surface. |
| Component library | **`@chief-os/ui`** (TypeScript React package) + **`chief-ui`** (Rust companion crate for attributes) — our thin kit wrapping Radix with Chief OS tokens and primitives. | The central choke-point for enforcement. |

**Packs consume `@chief-os/ui` through the Pack SDK. They do not import Radix, Tauri APIs, or raw DOM directly.** Attempting to do so produces a build-time error.

## The `chief-ui` primitive inventory (v0)

Every surface is composed from these. New primitives require ADR amendment.

| Primitive | Use | Renders |
|---|---|---|
| `<Pane>` | Base Liquid Glass panel | Vibrancy material, 10px radius, hairline border |
| `<Row>` | Content row in any feed | Avatar + sender/subject + snippet + badge + timestamp |
| `<SourceAvatar color>` | Small colored square for source-agent | 20px rounded-square, muted palette |
| `<Badge variant>` | HAX-region status pill | `needs-you` (amber) / `review` (blue) / `handled` (green) / `ceremony` (copper) / `anchor` (gray) — **no other variants** |
| `<SectionHeader>` | Time/group divider | Tracked small-caps 11pt, centered |
| `<TabBar chips>` | Count-chip navigation | Used in Brief and in any filtered list |
| `<CommandPill actions>` | Floating bottom toolbar | Thin-line icons: Omnibar / HAX Inbox / Chat by default |
| `<Widget title>` | Right-side widget shell | Small glass card, tracked header |
| `<DotTrail filled/total>` | Trust indicator | 5-dot row |
| `<Sparkline data>` | Activity signal | Compact bar chart |
| `<HoldRing progress>` | Ceremony hold-to-approve | Thin copper arc ring |
| `<Menubar>` / `<MenubarExtra>` | OS-level menubar | Native via Tauri; surfaces declare their menu tree |
| `<CeremonyCard>` | High-stakes approval | Dark takeover, evidence rows, hold-ring |
| `<OmnibarPalette>` | Global semantic search | Modal overlay with input + result list |
| `<InboxDrawer>` | HAX Inbox notification surface | Anchored drawer from menubar |
| `<ChatPane>` | Ad-hoc conversation | Slide-in side panel |

## What packs CAN do

- Compose the primitives above into any layout their pack needs.
- Choose among the locked HAX badge variants. Use them semantically — you don't pick amber because you like the color; you pick `<Badge variant="needs-you">` because the intent is a user-approval ask.
- Declare their own source-agent colors from the approved palette (amber, blue, green, copper, gray, teal, violet, peach — closed set, 8 muted hues).
- Provide short copy.
- Declare icons by semantic name (chief-ui maps to SF Symbols / Phosphor).
- Register menubar items via `<Menubar>`.

## What packs CANNOT do

- Import Radix directly, Tauri APIs directly, or raw DOM.
- Ship custom fonts or load fonts at runtime.
- Render arbitrary HTML or inline SVG for UI chrome.
- Invent new badge variants or colors outside the closed palette.
- Write CSS for color, typography, or motion — only layout-level CSS is allowed on pack-defined surfaces, and only through chief-ui's `<Box>` primitive.
- Style primitives. A `<Badge>` always looks the same.
- Draw their own liquid-glass material — use `<Pane>`.
- Ship toasts or alert modals — use HAX Inbox.

## Enforcement

Three layers of enforcement, earliest wins:

1. **SDK compile-time**: `@chief-os/sdk` exports only chief-ui primitives for UI work; Radix and Tauri are internal to the kit and not re-exported. TypeScript `declare module` aliases that try to access internals fail to typecheck.
2. **Pack build**: the pack-build toolchain (Nix-flake-driven, see [`docs/40-pack-sdk.md`](../docs/40-pack-sdk.md)) scans imports and bans any top-level import not listed in the SDK's public surface. Hard-fails the build.
3. **Runtime / installation**: signed pack manifests declare which primitives they render. The Capability Broker cross-checks at install time. Non-matching packs are rejected.

## First-party packs set the bar

The dogfood sequence — HN briefer, file-watcher brief, Gmail triage, Calendar negotiator — **consumes the same public chief-ui API as external packs**. No internal backdoor. If first-party packs cannot be built with chief-ui alone, the kit is incomplete and we extend it in the open — never by special-casing internals.

## Relationship to other ADRs

- [`adr-0002`](./0002-capability-based-security.md) — Capability Broker gates pack behavior. This ADR gates pack UI.
- [`adr-0003`](./0003-wayland-not-x11.md) — Wayland provides the per-client isolation that makes vibrancy possible on Linux.
- [`adr-0010`](./0010-sdk-public-api-stability.md) — This ADR is the UI extension of that public contract.

## Consequences

**Positive:**
- Every Chief OS pack looks and feels like Chief OS. Users never "enter a third-party site."
- Brand strengthens with every new pack instead of fragmenting.
- Accessibility is guaranteed at the kit level, not per-pack.
- Dark-mode parity, reduce-motion support, and keyboard-nav are automatic.
- When we refine the design system, every pack upgrades for free.

**Negative / tradeoffs:**
- Pack developers have less creative freedom than raw-web. This is the intentional Apple/HIG tradeoff — earned design consistency is worth more than 1000 unique splash screens.
- We must maintain `@chief-os/ui` across platforms. Budget: dedicated maintainer from v1 onward.
- First-party and third-party upgrades are coupled to chief-ui releases.

## Open questions

1. Pack theming — if a developer wants a subtle brand mark in an otherwise-native surface, what (if anything) is allowed? (Open — probably allow a single logo slot inside the Widget header, at chief-ui-controlled size.)
2. Can advanced packs (e.g. a spreadsheet surface) extend chief-ui with their own primitive? (Decision deferred — we will accept PRs post-v0 if primitives demonstrably don't cover a real need. Cf. SwiftUI accepting `.buttonStyle()` customization under strict rules.)
3. Voice UI surfaces (region 1 chat by voice) — how do they compose chief-ui? (Open — likely via a `<VoiceSession>` primitive that wraps the standard Chat pane.)

## References

- [`docs/brand/visual-language.md`](../docs/brand/visual-language.md) — the design specification this ADR implements.
- [`screenshots/`](../screenshots/) — reference mockups showing the locked language.
- [`docs/40-pack-sdk.md`](../docs/40-pack-sdk.md) — pack-level contract.
- [`docs/31-ui-standardization.md`](../docs/31-ui-standardization.md) — implementation details and primitive API reference.
