---
id: morning-brief-prototype
title: "Morning Brief Surface Prototype"
status: draft
owners: [santosh]
last_updated: 2026-04-21
tags: [prototype, morning-brief, surface]
---

# Chief OS Surface Prototype

## Hypothesis

The Work Object surface can make all-day delegated agent work legible in one
place: Work Object View + Brief + HAX Inbox + Omnibar + Ceremony overlay, all
wired to a live chief-core `/v1/*` HTTP API. No mock data in production —
empty API responses render as empty states.

## Run

```bash
npm install
npm run typecheck
npm run build
npm run test
npm run dev
```

All surfaces call `/v1/*` relative paths. The dev server proxies `/v1/*` to
`http://localhost:8080` by default; override with
`VITE_CHIEF_CORE_URL=http://127.0.0.1:4711 npm run dev` to target a legacy
chief-core kernel.

## Surfaces

- `src/surfaces/WorkObjectView.tsx` — all-day POC center bound to `/v1/work/:id`.
- `src/MorningBrief.tsx` — ritual surface bound to `/v1/brief`.
- `src/surfaces/InboxDrawer.tsx` — HAX Inbox, `⌘I`, streams `/v1/inbox/stream`.
- `src/surfaces/Omnibar.tsx` — floating palette, `⌘Space`, `/v1/omnibar/search`.
- `src/surfaces/Ceremony.tsx` — full-screen co-sign with hold-to-confirm ring.
- `src/surfaces/Menubar.tsx` — thin top strip + live spend-today from `/v1/models/cost`.

## Success

- `npm run build` ships clean production bundle.
- `npm run test` passes (vitest + @testing-library/react).
- `⌘I` toggles HAX Inbox, `⌘Space` toggles Omnibar, `Esc` dismisses.
- Ceremony requires a 3-second hold on the copper ring to approve.
- Every surface renders an empty state when the backend is unreachable.

## Cleanup

CLEANUP date: 2026-05-21. Delete this prototype or justify extension in this README.

## Running as native app

The native shell lives in `src-tauri/` and wraps the Vite prototype with Tauri v2. It registers global shortcuts for the omnibar and inbox toggles, applies macOS HUD vibrancy when available, and exposes bundled Inter fonts through the `get_bundled_font` command.

```bash
npm install
npm run tauri:dev
```

Use `npm run tauri:build` for a production native bundle. The Tauri dev server expects Vite at `http://localhost:5173` and runs `npm run dev` automatically.
