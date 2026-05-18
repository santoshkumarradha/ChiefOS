---
id: example-external-orchestrator
title: "External Orchestrator Example"
status: draft
owners: [santosh]
last_updated: 2026-05-18
related: [headless-chief-node, chief-app-protocol, plan-headless-agent-os-poc]
depends_on: [chief-app-protocol]
tags: [examples, poc, orchestrator]
---

# External Orchestrator Example

## TL;DR

- This is the POC 4A bring-your-own-orchestrator proof.
- The process owns planning and local agent selection.
- Chief provides Work Object state, contribution write, provenance, and Ceremony.

## Contract

Run the gate:

```bash
scripts/poc4-external-orchestrator.sh
```

The example uses only Python stdlib HTTP. It does not import Chief kernel crates or SDK internals.

## Decisions

1. **Keep orchestration outside Chief.**

   Rationale: Chief should provide OS substrate, not choose the semantic workflow.

2. **Still require Ceremony for high-risk action.**

   Rationale: bring-your-own orchestration must not bypass Chief authority.
