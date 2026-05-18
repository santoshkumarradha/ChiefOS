---
id: example-sales-followup-app
title: "Sales Follow-up App Example"
status: draft
owners: [santosh]
last_updated: 2026-05-18
related: [headless-chief-node, memory-substrate, pack-sdk, plan-headless-agent-os-poc]
depends_on: [headless-chief-node, memory-substrate]
tags: [examples, poc, app-protocol]
---

# Sales Follow-up App Example

## TL;DR

- This is the POC 2 external Chief app.
- It imports no Chief kernel crates and uses only HTTP.
- It reads a Work Object, writes a `finding` contribution, and relies on Chief for Broker, Memory Graph, and Provenance.

## Contract

Inputs:

- `CHIEF_BASE_URL`, default `http://127.0.0.1:18083`
- `CHIEF_WORK_ID`, default `acme-follow-up`
- `CHIEF_APP_PRINCIPAL`, default `app:sales-followup`

Protocol:

```text
GET  /v1/work/:id
POST /v1/work/:id/contributions
GET  /v1/work/:id/provenance
```

Run the end-to-end gate:

```bash
scripts/poc2-sales-followup-app.sh
```

## Decisions

1. **Use Python stdlib only.**

   Rationale: the app should prove the public protocol boundary, not framework integration.

2. **Write a `finding` contribution.**

   Rationale: POC 2 must not invent a `contribution` node type. Chief projects existing Memory Graph nodes as Work Object contributions.
