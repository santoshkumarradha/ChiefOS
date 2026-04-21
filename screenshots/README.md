---
id: screenshots
title: "Design References (Mockups)"
status: stable
owners: [santosh]
last_updated: 2026-04-21
---

# Design References

Wishful mockups — **not** real implementation captures. Generated from the written spec in [`../docs/brand/visual-language.md`](../docs/brand/visual-language.md) and [`../docs/05-surfaces.md`](../docs/05-surfaces.md) to anchor visual intent before code lands.

## Images

| File | Surface | Source spec |
|---|---|---|
| `01-desktop-hero.png` | Chief OS desktop at resting state (shell = Brief + widgets + command pill) | `docs/brand/visual-language.md` |
| `02-brief-closeup.png` | The Brief panel as product-shot detail on neutral presentation background | `docs/05-surfaces.md` §Morning Brief |
| `03-ceremony.png` | Ceremony surface — high-stakes approval (HAX regions 7/8) | `docs/05-surfaces.md` §Ceremony |

## Locked design language (as of 2026-04-21)

From these three references, the binding visual grammar is:

- **Brief is the shell**, not an app — Finder-like persistent resting state.
- **Single-feed content list with time sections** (TODAY · MORNING / TODAY · OVERNIGHT / YESTERDAY), not a 3-pane Finder layout.
- **Row anatomy**: source-agent avatar (left, muted color square) → sender + subject + snippet → right status-badge pill → tabular mono timestamp.
- **HAX badge palette**: amber = Needs you (R3), blue = Review (R5), green = Handled (R2), copper = Ceremony pending (R8), gray = Anchor (R6).
- **Widgets stack to the right** — TRUST (category dot trails), CEREMONY PENDING (warm copper accent), AGENT ACTIVITY (24h sparkline).
- **Bottom floating command pill** — three thin-line SF-Symbols glyphs: magnifier (Omnibar), inbox tray (HAX Inbox), chat bubble (Chat).
- **Ceremony = full-screen dark takeover**, monochrome with single copper accent.
- **Menubar** — Apple-style translucent, "Chief · Brief · Approvals · Timeline · Window · Help" on the left, small system glyphs + cost + date/time on the right.
- **Wallpaper** — warm abstract, teal → coral, organic shapes.
- **Typography** — Inter Tight for headlines, Inter body at realistic 12–13pt screen size, tabular mono for numerics.
- **Icons** — thin-stroke monochrome SF-Symbols line glyphs, ~1.5px stroke. No emoji. No raster.

## Regeneration

Generated via `gemgen` binary (OpenRouter → `google/gemini-3-pro-image-preview`, Nano Banana Pro) with reference images passed via `-i`. Native model output is 1376×768; upscaled to 3840×2160 (4K UHD) via PIL Lanczos for distribution. If spec changes, regenerate — do not edit images directly.

References used:
- `/Users/santoshkumarradha/Downloads/original-c5adbc69523bd75bfa9311b4897ff336.webp` — email client (Shortwave-style) for list density, section-header, badge-pill, and floating-toolbar patterns.
- `/Users/santoshkumarradha/Downloads/introducing-new-macos-concept-with-elements-from-ipad-os-v0-y0zavfw5hwl91.png` — macOS concept for overall desktop framing, menubar, widget pattern.

## Rules

- These are **design references**, not shipping assets. Do not link from the shipping product.
- If [`../docs/brand/visual-language.md`](../docs/brand/visual-language.md) changes, regenerate so the image never drifts from spec.
- Any PR that ships a real surface must be visually compared against the matching reference here during review.
