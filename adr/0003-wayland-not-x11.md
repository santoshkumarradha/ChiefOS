---
id: adr-0003
title: "0003 — Wayland as the compositor, not X11"
status: accepted
authors: [santosh]
created: 2026-04-21
decided: 2026-04-21
supersedes: []
superseded_by: []
axiom_impact: [1, 2]
tags: [compositor, surface, security]
---

# 0003 — Wayland as the compositor, not X11

## Context

Chief OS hosts many agent-adjacent processes (packs, surfaces, sandboxed browsers, local model UIs). The compositor's security model directly affects agent safety. X11's design allows any connected client to screenshot, keylog, and inject events into any other window. In an agent environment, that model is catastrophic.

## Options considered

| Option | Per-client isolation | HW acceleration | Agent-compatible | Legacy app support |
|---|---|---|---|---|
| X11 | No — universal snoop | Legacy-tree | Dangerous | Great |
| **Wayland** | Yes | Modern | Safe | Via XWayland in sandbox |
| Custom compositor | Yes | We build it | Depends | None |
| Mir (Canonical) | Yes | Modern | Safe | Limited |

## Decision

**Wayland** is the default compositor. X11 apps run via **XWayland in per-app sandboxes** (nspawn) if ever needed by a legacy pack.

## Rationale

1. Per-client surface isolation is an Axiom-2 requirement; X11 cannot provide it.
2. Wayland is the modern default across NixOS; tooling ecosystem is mature.
3. XWayland handles legacy apps with per-client isolation preserved.
4. HW acceleration story is cleaner for ambient visuals and the Live Agent View.

## Consequences

### Positive
- Agent-hosted apps cannot snoop each other or the user.
- Compositor can enforce approval-ceremony UI is not impersonable.
- Better HiDPI / modern display support.

### Negative / accepted costs
- Some legacy apps need XWayland wrapping; we maintain a small whitelist for official packs.
- Wayland input / clipboard stories are less flexible than X11's (feature, not bug).

### Downstream effects
- [`docs/06-security-model.md`](../docs/06-security-model.md) — surface isolation claim grounded here.
- [`docs/05-surfaces.md`](../docs/05-surfaces.md) — surfaces are Wayland clients.

## Implementation notes

- Compositor choice for v0: `niri` (scrollable tiling) or `hyprland` under evaluation (see [`docs/14-risks-open-questions.md`](../docs/14-risks-open-questions.md) design questions). Default may be our own thin compositor built on `wlroots`.
- XWayland enabled selectively per-pack manifest, never globally.
- Input events to ceremony UI never exposed to other clients, enforced by compositor.

## Revisit triggers

- A demonstrably safer compositor stack emerges.
- XWayland deprecation paths threaten our legacy-app fallback.
- Compositor bug compromises per-client isolation in practice.
