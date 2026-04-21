---
id: morning-brief-prototype
title: "Morning Brief Surface Prototype"
status: draft
owners: [santosh]
last_updated: 2026-04-21
tags: [prototype, morning-brief, surface]
---

# Morning Brief Surface Prototype

## Hypothesis

The Morning Reveal can make delegated agent work legible in one cold render: two human decisions, fourteen handled receipts, and a trust ledger that visibly moves from prior state to current state.

## Run

```bash
npm install
npm run typecheck
npm run build
npm run dev
```

The app reads only from `mock/brief-state.json`. It performs no HTTP requests.

## Success

- Spacebar or the reveal button starts the ceremony.
- Cards enter with staggered spring motion.
- Trust ledger bars spring from previous values to current values.
- Handled cards open a receipt panel with provenance.
- Needs-you review buttons open an Evidence Card modal stub.
- Tab, Arrow keys, Enter, and Esc operate the surface.
- Cold render remains below 3 seconds on a normal local Vite build.

## Cleanup

CLEANUP date: 2026-05-21. Delete this prototype or justify extension in this README.

## Running as native app

The native shell lives in `src-tauri/` and wraps the Vite prototype with Tauri v2. It registers global shortcuts for the omnibar and inbox toggles, applies macOS HUD vibrancy when available, and exposes bundled Inter fonts through the `get_bundled_font` command.

```bash
npm install
npm run tauri:dev
```

Use `npm run tauri:build` for a production native bundle. The Tauri dev server expects Vite at `http://localhost:5173` and runs `npm run dev` automatically.
