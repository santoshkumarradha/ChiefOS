---
id: brand-typeface
title: "Typeface — specimens, licensing, bundling"
status: draft
owners: [santosh]
last_updated: 2026-04-21
related: [brand-visual-language, apple-design]
tags: [brand, typography]
---

# Typeface

> Binding reference. Source of truth for type tokens is [`docs/brand/visual-language.md`](../docs/brand/visual-language.md) §3. This document records the specific families chosen, licensing posture, bundling rules, and v1 upgrade path.

## v0 picks (bundled in binary)

| Role | Family | License | Rationale |
|---|---|---|---|
| Display (headlines, statement type) | **Inter Tight** | SIL OFL 1.1 (free) | Tight apertures, confident at large sizes, excellent numerics |
| UI (all body, labels, navigation) | **Inter** | SIL OFL 1.1 (free) | Apple-adjacent, excellent at small sizes, full OpenType feature set |
| Mono (timestamps, code, hashes, provenance chains) | **JetBrains Mono** | OFL 1.1 (free) | Ligatures on for code (`=>`, `->`, `==`), tabular numerals, wide language coverage |

## v1 upgrade candidates (paid / commissioned)

| Role | Candidate | License tier | Why consider |
|---|---|---|---|
| Display | **Söhne** by Klim Foundry | Desktop per-user | Calmer than Inter Tight at large sizes; ships with the "computer calm" feel. ~$3-5k/team. |
| Display | **Pitch** by Klim Foundry | Desktop per-user | Editorial, distinctive, strong numeric set. |
| Display | **Commissioned custom** | Commission + exclusive use | ~$15-50k. Only if Chief OS brand warrants its own voice — decision deferred to post-PMF. |
| Mono | **Berkeley Mono** by Berkeley Graphics | Commercial per-seat | Crisper than JetBrains Mono, more opinionated. |

**Default stance:** stick with Inter + Inter Tight + JetBrains Mono through v0 and v1. Revisit at v2 once there's budget and clear brand differentiation need.

## Bundling rules

1. **Never fetch fonts over HTTP at runtime.** All type bundled into the Tauri binary at build time.
2. **No Google Fonts link tag.** Ever.
3. Subsetting: ship only Latin + numerals at v0; expand ranges only as localization demands (CJK, Arabic, Cyrillic are v1+).
4. **Font files location:** `packages/chief-ui-ts/assets/fonts/` — loaded via CSS `@font-face` referencing the bundled path.
5. **License attribution:** include full OFL-1.1 text and per-family attributions in `packages/chief-ui-ts/assets/fonts/LICENSES.md`.

## OpenType features (via `chief-ui` tokens)

| Context | Features on | Why |
|---|---|---|
| Body UI text | `ss01` (alternate digits if defined), kerning | Default legibility |
| Timestamps / numerics | `tnum` (tabular numerals), `zero` (slashed zero) | Changing numbers don't reflow; zero distinguishable from O |
| Display headlines | Optical sizing if available; kerning | Crisper at large sizes |
| Mono / code | `calt` (contextual alternates), `liga` (ligatures) | `=>`, `->`, `==`, `!=` render as glyphs |

## Size system (derived from `brand/visual-language.md` §3)

| Token | Size | Use |
|---|---|---|
| `--chief-size-headline` | 34pt | Hero statement (Brief "Good morning.") |
| `--chief-size-section` | 11pt, tracked 0.08em | Small-caps time dividers |
| `--chief-size-body` | 13pt | Row subject + snippet |
| `--chief-size-meta` | 11pt | Timestamps, metadata |
| Mono runtime | 11pt | Numeric displays, hashes |

## Specimen screenshots

See [`../screenshots/01-desktop-hero.png`](../screenshots/01-desktop-hero.png), [`../screenshots/02-brief-closeup.png`](../screenshots/02-brief-closeup.png), and [`../screenshots/03-ceremony.png`](../screenshots/03-ceremony.png) for typography in-context.

## Decision log

- **2026-04-21** — Locked Inter + Inter Tight + JetBrains Mono for v0 and v1. Paid alternatives considered and deferred to v2 per budget/PMF gating. Decision affirms ADR-0011 which encodes the same choice at the technical stack level.

## Related

- [`brand/visual-language.md`](./visual-language.md) §3 — source of truth for type tokens
- [`docs/12-apple-design-principles.md`](../docs/12-apple-design-principles.md) — "typography as substance" principle
- [`adr/0011-ui-stack-and-component-library.md`](../adr/0011-ui-stack-and-component-library.md) — where font bundling is enforced
