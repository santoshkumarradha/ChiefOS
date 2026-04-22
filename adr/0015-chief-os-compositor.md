---
id: adr-0015
title: "Chief OS bundles Hyprland as the shipping Wayland compositor"
status: accepted
date: 2026-04-21
deciders: [santosh]
supersedes: []
superseded_by: []
amends: [0003, 0011]
tags: [compositor, wayland, hyprland, ui, brand]
---

# ADR-0015 — Chief OS Compositor Choice

## Context

[ADR-0003](./0003-wayland-not-x11.md) committed to Wayland. [ADR-0011](./0011-ui-stack-and-component-library.md) promised "native liquid glass" on both macOS (via `NSVisualEffectView`) and Linux (via `layer-shell` + compositor blur). The architecture feasibility review (2026-04-21) found this second claim to be aspirational:

- `tauri-plugin-window-vibrancy` README: "Linux is unsupported."
- Sway: no blur by default.
- GNOME / Mutter: no public blur API; blur is a private implementation detail that can change any release.
- Hyprland: blur works, but only when enabled in config and only for layer-shell surfaces.

On a target NixOS deploy path, unless Chief OS **ships and pins a specific compositor**, the design language fragments across users' installations. "Hope the user's compositor does blur" is not a shipping strategy.

This ADR resolves the choice.

## Decision

**Chief OS on Linux ships and pins Hyprland as the Wayland compositor, bundled into the NixOS flake, with a Chief-specific blur preset baked in.**

### Why Hyprland

| Candidate | Blur support | Per-client isolation | License | Verdict |
|---|---|---|---|---|
| **Hyprland** | ✅ Configurable, works on layer-shell | ✅ Wayland-level (no cross-client snooping) | BSD-3 | **Chosen** |
| Sway | ❌ No default blur; plugins exist but fragment | ✅ Wayland-level | MIT | Rejected: breaks design language by default |
| GNOME / Mutter | ❌ No public blur API | ✅ Wayland-level | GPL-2 | Rejected: too opinionated about workflow; blur is private API |
| KDE Plasma / KWin | ✅ Blur in Wayland session | ✅ Wayland-level | GPL-2/LGPL | Rejected: too heavy, conflicts with "no desktop environment" directive from ADR-0003 |
| Custom Wlroots compositor | ✅ If we write it | ✅ | Ours | Rejected: explicit non-goal at v0 (Axiom 7 — don't invent below L2) |
| COSMIC | Blur arriving | ✅ Wayland-level | MPL-2 | Rejected for v0: too early; revisit v2+ |

Hyprland ships as a nixpkgs flake input; we pin a specific revision and ship it as a NixOS module `chief-compositor.nix`.

### The blur preset: `chief-compositor.nix`

A Nix module that:

1. Installs Hyprland at a pinned version.
2. Ships `hyprland.conf` with Chief blur + rounding + animation defaults matching [`docs/brand/visual-language.md`](../docs/brand/visual-language.md) §6 (shadows/depth) and §7 (motion).
3. Pre-configures Hyprland as a systemd user service to start on boot with the Chief UI as its first layer-shell surface.
4. Disables Hyprland keybinds that conflict with Chief OS keybinds (Omnibar summon, Ceremony hold, etc.).
5. Disables Hyprland's workspace/window-tiling UX by default — Chief OS is single-surface-at-a-time per [`docs/brand/visual-language.md`](../docs/brand/visual-language.md) §14, so we neutralize the tiling-WM ergonomics at the compositor level. Users who want tiling can install a plugin or unlock via Controls.
6. Exposes the blur intensity as a Chief Controls item (a numeric preference — low-consequence, fits Controls contract).

### Updated statement on "native liquid glass"

ADR-0011 §"Native material (liquid glass)" is amended:

| Platform | Native liquid glass delivery |
|---|---|
| **macOS** | `NSVisualEffectView` via `tauri-plugin-window-vibrancy`. Unchanged. |
| **Linux (NixOS, Chief-deployed)** | Hyprland (bundled, pinned) + `layer-shell` + Chief blur preset. Native-equivalent. |
| **Linux (other distributions)** | Not supported by default. CSS `backdrop-filter` fallback. Users on non-Chief-shipping distros accept a degraded visual experience. |
| **Windows** | Out of scope at v0. |

Honest statement: we deliver native liquid glass on **two platforms**: macOS (via Tauri plugin) and **Chief-on-NixOS-with-bundled-Hyprland**. We do not deliver it on arbitrary Linux distributions.

### Version pinning discipline

Hyprland moves fast — multiple releases per year with occasional breaking config changes. We pin to a specific hash in the Chief OS NixOS flake and update only through a deliberate version-bump commit that:
- Tests against the current Chief UI integration.
- Updates `chief-compositor.nix` config for any syntax changes.
- Runs the visual-regression test suite against [`screenshots/01..03.png`](../screenshots/).

Same discipline applies to any eventual Hyprland → Wlroots-native-Chief migration or to a swap to a different compositor (COSMIC revisit in v2+).

## Consequences

**Positive:**
- Design language holds on Linux. Screenshots render the same on deploy as in dev.
- One compositor to test against instead of "whatever the user runs." Bug surface shrinks.
- Controls → Blur intensity becomes a real user-settable item instead of an aspiration.
- Honest statement to users: "Chief OS feels this way because we ship the full stack."

**Negative / tradeoffs:**
- Users with strong compositor preferences cannot run Chief OS natively on their preferred desktop — this is a distribution-level constraint, accepted.
- Hyprland version-bump work is ongoing maintenance load; budget.
- Chief-on-arbitrary-Linux (Arch, Ubuntu) has a degraded visual experience by default. Acceptable because our deploy target is NixOS-Chief, not arbitrary Linux.
- "Axiom 7 — don't invent below L2" pressure: we now own a specific compositor configuration. We vendor Hyprland; we don't fork it. Line held.

## Amendments triggered

- [`adr-0003`](./0003-wayland-not-x11.md) — remains accepted; this ADR further specifies "which Wayland compositor."
- [`adr-0011`](./0011-ui-stack-and-component-library.md) — §"Native material (liquid glass)" row amended to match the table above.
- [`docs/18-base-and-hardware.md`](../docs/18-base-and-hardware.md) — compositor choice row to be updated.
- [`docs/brand/visual-language.md`](../docs/brand/visual-language.md) §6 — blur values become the source of truth for `chief-compositor.nix`.

## Risks

- **Hyprland governance.** Single-maintainer project historically; maintainer has had public conflicts with distributions. Mitigation: pinned version, explicit fork plan in crisis, COSMIC as revisit candidate.
- **Hyprland config breakage.** Version bumps occasionally break config schemas. Mitigation: version-bump PR includes a `chief-compositor.nix` review step; CI tests against current Chief UI.
- **User-compositor preference conflict.** Some Linux users prefer Sway/i3 for reasons. They can still run the Chief UI binary on their compositor; the blur won't work as designed. Accepted tradeoff.

## Related

- [`adr-0003`](./0003-wayland-not-x11.md) — Wayland choice.
- [`adr-0011`](./0011-ui-stack-and-component-library.md) — UI stack this ADR disambiguates.
- [`docs/18-base-and-hardware.md`](../docs/18-base-and-hardware.md) — L1 choices.
- [`docs/brand/visual-language.md`](../docs/brand/visual-language.md) — source of truth for blur + motion values.
