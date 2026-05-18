---
id: downloads-steward-ts
title: "Downloads Steward TypeScript POC"
status: draft
owners: [santosh]
last_updated: 2026-05-18
related: [chief-app-protocol, pack-sdk, headless-chief-node]
depends_on: [chief-app-protocol]
tags: [poc, sdk, filesystem, ceremony]
---

**TL;DR**
- SDK-only Chief app for cleaning a real local folder.
- Uses Chief filesystem scan, Chief-mediated LLM planning, Work Object contribution, Ceremony, apply, and rewind.
- Demo script uses a temporary folder so it is safe while still exercising real filesystem operations.

**Install**

```bash
npm ci --prefix packages/chief-sdk-ts
npm run build --prefix packages/chief-sdk-ts
npm install --no-package-lock --prefix examples/downloads-steward-ts
npm run build --prefix examples/downloads-steward-ts
```

**Run**

```bash
OPENROUTER_API_KEY=... scripts/poc6-downloads-steward.sh
```

**Contract**
- Input: `CHIEF_BASE_URL`, `CHIEF_APP_PRINCIPAL`, `CHIEF_DOWNLOADS_ROOT`, `CHIEF_WORK_ID`.
- Output: JSON containing the scan, LLM provider/model, proposed move manifest, Ceremony ID, apply receipt, and rewind receipt.
- Invariant: the app imports `@chief-os/sdk` and does not call Chief HTTP routes directly.
