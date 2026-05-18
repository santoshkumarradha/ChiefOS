---
id: acme-chief-app-ts
title: "Acme Chief App TypeScript POC"
status: draft
owners: [santosh]
last_updated: 2026-05-18
related: [chief-app-protocol, pack-sdk, headless-chief-node]
depends_on: [chief-app-protocol]
tags: [poc, sdk, app-platform, headless]
---

**TL;DR**
- SDK-only TypeScript app that runs outside `chief-core`.
- Reads the Acme Work Object, calls Chief-mediated `llm.generate`, writes a contribution, and opens Ceremony.
- The app owns planning and agent selection; Chief OS provides authority, memory, provenance, inference, and approval rails.

**Contract**
- Input: `CHIEF_BASE_URL`, `CHIEF_WORK_ID`, `CHIEF_APP_PRINCIPAL`.
- Output: JSON containing the LLM provider/model, selected app agents, contribution URI, and Ceremony ID.
- Invariant: the app imports `@chief-os/sdk` and does not call Chief HTTP routes directly.

**Acceptance**
- `scripts/poc5-sdk-ts-chief-app.sh` passes with `OPENROUTER_API_KEY` in the environment.
- Contribution source is `app:acme-chief-ts`.
- LLM provider is `openrouter`.
- The exact outbound email payload is locked by Ceremony `payload_hash`.
