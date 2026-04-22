---
id: brand-visual-language
title: "Visual Language — feel like an OS, not a web app"
status: draft
owners: [santosh]
last_updated: 2026-04-21
related: [apple-design, surfaces, north-star]
tags: [brand, ui, ux, desktop, anti-saas]
---

# Visual Language

**Binding reference** for every Chief OS surface. Any UI work that violates these is a regression and must be challenged in review.

## TL;DR — the one thing that matters

Chief OS must **feel like a desktop operating system**, not a web application, not a SaaS dashboard. Inspiration: macOS Big Sur / Sonoma polish × Linux-minimalist themes (Nord / Adwaita / Arc) × Apple Vision Pro's disappearing chrome. Explicitly NOT Notion / Linear / Slack aesthetic — those are SaaS apps. We are an **OS**.

Below are concrete, enforceable decisions that prevent the "web-app smell."

## 1. Rendering substrate: Tauri, not Chromium-kiosk

| Option | Decision |
|---|---|
| Pure web page served over HTTP + Chromium --kiosk | ❌ rejected — inherits browser's web-ness: scrollbars, focus rings, text selection all feel "page"-like |
| **Tauri v2 wrapping our React UI** | ✅ **default for v0** — one native binary, native window decorations, native menubar integration, OS-level keyboard shortcuts, native context menus, native drag-and-drop, native file dialogs |
| Native from scratch (Iced/Slint) | ⏳ considered for v1 if Tauri proves inadequate |

### Why Tauri specifically

- Produces a **real binary** (`chief-ui`) that the OS treats like a native app. Window chrome is OS-chrome, not web-chrome.
- WebView layer (WebKit on Linux/macOS, WebView2 on Windows) handles our React code, but **ALL OS-level interactions go through Tauri's Rust bindings** — keyboard shortcuts, clipboard, notifications, menubar, tray, file system, etc.
- Keyboard shortcuts are registered via Tauri's global shortcut API, NOT `document.addEventListener` — so `⌘ Space` for Omnibar works at the OS level, not as a web-page event.

## 2. Window model

| Aspect | Decision |
|---|---|
| At boot (NixOS live/install) | **Fullscreen, borderless, no window decorations.** Morning Brief fills the screen. Wayland compositor (Cage or custom) runs this as the root surface. |
| In dev / windowed mode | Standard OS window decorations (traffic lights on macOS-alike, close button on Linux-alike). NO custom window controls. |
| Multi-window | Not supported at v0. One surface at a time. Summoned overlays (Omnibar, HAX Inbox) are modal overlays in the same window, not separate windows. |

## 3. Typography — the #1 tell for "web app vs OS"

Rules:

- **No Google Fonts over HTTP.** Ever. Fonts are bundled into the Tauri binary at build time.
- **Default weights at OS-app sizes**, not web-doc sizes:
  - Body: **13–14 pt** (not 16 pt browser default).
  - UI text: 11–12 pt.
  - Headlines (Morning Brief "2 things need you"): 32–40 pt, tight tracking.
  - No 18px body everywhere — that screams "marketing landing page."

### Typefaces

| Role | v0 choice | v1 target |
|---|---|---|
| Display (headlines) | **Inter Tight** (SIL OFL, free) | **Pitch** by Klim or **Söhne** by Klim (licensed) OR commission |
| UI (all body + labels) | **Inter** (SIL OFL) | Same; Inter is our long-term floor |
| Mono (code, hashes, provenance chains) | **JetBrains Mono** or **Berkeley Mono** (licensed) | Berkeley Mono if budget allows |

Features: ligatures on for mono (`=>`, `->`, `==`), off for UI. Tabular numerals enforced for any UI that shows changing numbers (trust-ledger bar, cost counter, time). Optical sizing on where available.

### Anti-patterns

- ❌ `<h1>` stacked with `<h2>` stacked with `<p>` — doesn't feel like an OS; feels like a blog.
- ❌ Line heights > 1.6 for UI text. OS apps use 1.2–1.4.
- ❌ Underlined links. OS apps don't have "links" — they have buttons or affordances.
- ❌ Google Fonts link tag in HTML. Bundled fonts or nothing.

## 4. Color system

Dark-first (OS-style). Nord / macOS Sonoma dark pallete-adjacent, warmer than pure black.

| Token | Value | Use |
|---|---|---|
| `--bg-0` | `#0c0e11` | Base background (near-black, slightly warm) |
| `--bg-1` | `#14171b` | Surface raised (cards, overlays) |
| `--bg-2` | `#1b1f24` | Surface raised 2× (modals over overlays) |
| `--border-subtle` | `rgba(255,255,255,0.06)` | Card borders — barely visible |
| `--fg-primary` | `#e8ecf1` | Body text (not pure white — hurts eyes) |
| `--fg-secondary` | `#98a2b0` | Labels, timestamps |
| `--fg-tertiary` | `#5c6470` | Metadata, provenance |
| `--accent` | `#5294e2` | Primary action (restrained blue, Nord-ish) |
| `--accent-warm` | `#c68858` | "Needs you" warning (muted copper) |
| `--success` | `#73c991` | "Handled / verified" (muted green) |
| `--danger` | `#e05a5a` | "Blocked / rolled back" (muted red) |
| `--on-device` | `#8ec0b8` | Local-inference indicator (distinct teal) |
| `--cloud` | `#8498c4` | Cloud-inference indicator (distinct blue-gray) |

Accent use: **restrained.** Mac apps don't have blue everywhere. One or two primary-blue touches per screen max. The OS feels calm when colors are scarce.

Light mode: v2. Not v0.

## 5. Spatial system

| Token | Value | Rule |
|---|---|---|
| `--space-xs` | 4 px | inside tightly-related glyphs |
| `--space-s` | 8 px | label ↔ value pairs |
| `--space-m` | 12 px | within a card |
| `--space-l` | 20 px | card ↔ card |
| `--space-xl` | 40 px | section ↔ section |
| `--radius-card` | 10 px | cards and overlays |
| `--radius-button` | 6 px | buttons |
| `--radius-pill` | 999 px | status pills, trust ledger segments |

No harsh 90° corners, no iOS-rounded (18–20 px) corners. 10 px is the "Mac app" sweet spot.

## 6. Shadows & depth

- Shadows are **low opacity, soft, large blur.** Never harsh.
  - Card: `box-shadow: 0 1px 2px rgba(0,0,0,0.4), 0 12px 32px rgba(0,0,0,0.25)`.
- Overlays (Omnibar, HAX Inbox popover) use **OS-native vibrancy/blur** via Tauri where the platform supports it (macOS: `NSVisualEffectView` via `window-vibrancy` crate; Linux/Wayland: `layer-shell` + compositor blur where possible; fallback: solid `--bg-2`).
- **No CSS `backdrop-filter: blur(20px)`** as the primary blur — looks web-ish. Tauri's platform-native vibrancy is the real deal.

## 7. Motion

All motion uses **spring physics** via `framer-motion`'s spring transitions (or Tauri-native motion where available). Tuned for "weighty-but-responsive":

| Motion type | Stiffness | Damping | Mass | Feel |
|---|---|---|---|---|
| Card reveal | 220 | 28 | 1 | firm, settled |
| Trust-ledger bar grow | 180 | 22 | 1 | satisfying drift |
| Omnibar summon | 260 | 30 | 0.9 | snappy |
| Ceremony begin | 120 | 20 | 1.2 | slow, deliberate, reverent |

**Banned:**
- ❌ Linear easing anywhere.
- ❌ Generic `ease-in-out` CSS transitions.
- ❌ Bouncy springs (damping < 20) — feel toy-like.
- ❌ "Page transitions" — we're not a web app; we have surface invocations, not page navigations.

## 8. Scrollbars (major web-app tell)

- **Overlay, thin, autohide.** macOS-style. Never persistent scroll gutters.
- On Linux: custom CSS (`::-webkit-scrollbar` width 8 px, thumb `--fg-tertiary`, track transparent, appears on scroll, fades after 1.5 s).
- On macOS host (dev): use the OS-native overlay scrollbars via `-webkit-overflow-scrolling: touch` and the default appearance.

## 9. Text selection, focus, context menus

- **Focus rings**: Tauri sets the native focus ring color. No CSS `outline: 2px solid blue` defaults — let the OS/WebView render focus natively.
- **Text selection color**: match `--accent` at 30% opacity. Tauri can set the native selection color.
- **Context menus**: use Tauri's `contextmenu` API (`tauri-plugin-context-menu`), NOT CSS/React-rendered menus. This is a massive OS-feel tell.
- **Text selection behavior**: double-click for word, triple-click for line/paragraph — browsers usually get this right, but we check on each platform.

## 10. Icons

- **Phosphor** (MIT) or **Lucide** (ISC) at v0. Consistent stroke width (1.5 px at 16 px icon size).
- **No emoji** in core UI chrome. (Emojis OK inside user-provided content like chat or card bodies.)
- Status-strip glyphs (network dot, model-tier dot, battery) are **custom SVG pieces** at 14 px, hand-tuned — NOT library icons. This is the one place we can afford pixel-perfect custom work.

## 11. Sound

- Three system sounds, commissioned or carefully licensed at v1 (v0 can use placeholder from royalty-free libs):
  1. **Ship chime** — one-shot, bell-adjacent, 200 ms, ~-24 dB. Plays on every shipped approval.
  2. **Approval-needed cluck** — wood-block adjacent, 80 ms, ~-28 dB. Plays when a new item lands in HAX Inbox.
  3. **Ceremony-begin tone** — a held drone, 900 ms, ~-30 dB, subtle. Plays when Region 7/8 ceremony surface appears.
- Played via Tauri's sound API (native audio output) — NOT `<audio>` tags. Native audio is instant; web-audio can be 100+ ms latent.
- Master off-switch in Settings (discoverable via Omnibar: "sound").

## 12. Haptics (trackpad-equipped hosts)

- Ceremony hold: progressive haptic during the 3-second hold, culminating in a firm tap on sign.
- Swipe-to-approve: light tap on completion.
- Via Tauri's haptic plugin where supported (macOS trackpads, some Linux devices).

## 13. The thin top status strip (the ONLY persistent OS chrome)

28 px tall. Always visible except during Ceremony (which takes over the full screen).

```
┌────────────────────────────────────────────────────────────────────┐
│ Mon · 14 Apr · 14:23   ●●○ 3 agents · trust 3.8      🟢 ◐ $0.42 ✓ 🔋 │  ← 28px
└────────────────────────────────────────────────────────────────────┘
```

**Rendering:**
- Rendered by Tauri as a **native OS-level overlay panel** (macOS: `NSPanel` attached to the main window's status layer; Linux/Wayland: `layer-shell` top strip).
- Transparent background with vibrancy (platform-native blur).
- NOT an HTML `<header>` element sitting on top of the surface. It's a real OS bar.
- Glyph popovers (tap network dot → picker) appear as native OS popovers with vibrancy, not as absolutely-positioned `<div>`s.

**Contents, left to right:**
- Day + date (weight 400, `--fg-secondary`, 11 pt).
- Time (weight 500, `--fg-primary`, 11 pt, tabular nums).
- [20 px gap]
- Agent count + status dots (animated pulse when active).
- "trust 3.8" tiny indicator.
- [20 px gap, mostly empty space]
- Network status dot (tap → Wi-Fi picker popover, native vibrancy).
- Model tier glyph (tap → which model, switch).
- Cost today (small; hide by default unless > threshold; user can pin).
- Device trust status (✓ or ! glyph; tap → YubiKey prompt).
- Battery (laptop only).

Each right-side item is interactive; no text beyond glyphs. Think macOS Sonoma's Control Center — minimal, glanceable.

## 14. Surfaces vs windows

| Concept | Web-app pattern | Chief OS pattern |
|---|---|---|
| Change surface | URL navigation | **Keyboard shortcut** (⌘ + letter) or gesture, summoning a surface as a full takeover |
| Go back | Browser back | `Esc` always returns to Morning Brief |
| Multiple things at once | Tabs / split panes | None at v0. One surface at a time. |
| History | URL bar | Timeline scrub on Morning Brief / Provenance Explorer |

## 15. Native integration checklist (per surface)

Every Tauri-rendered surface MUST:

- [ ] Register its keyboard shortcuts through Tauri global shortcut API, not DOM listeners.
- [ ] Use Tauri's `contextmenu` API for any right-click.
- [ ] Use native file dialogs for any file pick/save.
- [ ] Use native notifications for rare critical alerts (device-key-unlock, low-battery).
- [ ] Use native audio for sound effects.
- [ ] Use native clipboard APIs (Tauri clipboard plugin).
- [ ] Use tabular numerals for any changing number.
- [ ] Respect the OS's "reduce motion" accessibility preference — animations collapse to instant on.
- [ ] Respect the OS's font size / zoom preference within reason.

## 16. Explicit anti-patterns (review-blockers)

If any of these show up in a PR, block the review and ask for a fix:

- ❌ Web-fonts loaded from Google / Adobe / a CDN.
- ❌ Toast notifications (use HAX Inbox).
- ❌ Hamburger menu.
- ❌ Footer with navigation links.
- ❌ "Powered by" badges.
- ❌ Loading spinners borrowed from Bootstrap/Tailwind examples.
- ❌ Tooltips on every button (documentation belongs in surfaces, not hover states).
- ❌ Modal dialogs with "Cancel / OK" in a row — use Ceremony pattern or Inbox item instead.
- ❌ `<a href>` tags (we have no URLs).
- ❌ Dark-mode as an afterthought (we ARE dark-first; light-mode is the afterthought, v2+).

## 17. Inspiration references

Study these before designing any new surface:

| Product | Study for | Not for |
|---|---|---|
| macOS Sonoma | typography, motion, blur/vibrancy, menubar discipline | color palette (too safe) |
| Apple Vision Pro | chrome disappearance, text-as-UI | we don't do AR |
| Nord / Niri / Hyprland on Linux | color restraint, tiling discipline | we don't tile |
| Arc browser (RIP) | bold typography moments, gradients-as-accents | it's a browser |
| Rewind.ai desktop app | provenance-as-interface, timeline scrub | over-featured |
| Soulver | mono-font elegance, minimal chrome | scope |
| Ivory / Tapbots apps | weighty Apple-adjacent polish on Linux-feasible stack | iOS-only |

Study these AS NEGATIVE examples:

| Product | Why NOT |
|---|---|
| Notion | infinitely-scrollable dashboard; toolbar-heavy |
| Linear | beautifully designed but obviously a webapp (still feels like a browser tab) |
| Slack | chrome on chrome on chrome |
| Electron-based Discord | web-wrapper anti-pattern |
| Windows 11 settings | icon soup, context menus everywhere |

## Acceptance (review checklist)

Before any Morning Brief / surface PR lands on main:

- [ ] Runs as a Tauri binary, not a browser URL.
- [ ] Fonts bundled in the binary, not fetched over HTTP.
- [ ] Native focus rings (not CSS `outline`).
- [ ] Native scrollbars (autohide overlay).
- [ ] Native context menus on right-click.
- [ ] Top status strip is a native panel, not an HTML header.
- [ ] All motion uses spring physics with values in this doc's table.
- [ ] No web-app anti-patterns from §16 present.
- [ ] OS reduce-motion preference respected.
- [ ] 13–14 pt body text; no 16 pt browser default.
- [ ] Color tokens match §4 palette exactly.

## Open questions

1. Tauri v2 WebView performance on Linux/Wayland under a minimal Cage compositor — needs measuring.
2. Typeface licensing: free (Inter only) vs. commissioned (~$15k+) vs. licensed Klim/Commercial (~$3–5k/team).
3. Native popover API availability on Linux/Wayland for the status-strip glyph popovers — may need layer-shell + custom compositor work.
4. Haptics on Linux — limited hardware coverage; keep as macOS-only polish for v0.

## Related

- [`apple-design`](../02-apple-design-principles.md) — the product-design guardrails this doc implements visually.
- [`surfaces`](../30-surfaces.md) — what each surface is and does.
- [`north-star`](../00-north-star.md) — why "it feels like an OS" matters for B2C virality.
