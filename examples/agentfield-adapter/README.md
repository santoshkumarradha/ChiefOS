---
id: example-agentfield-adapter
title: "AgentField Adapter Example"
status: draft
owners: [santosh]
last_updated: 2026-05-18
related: [headless-chief-node, chief-app-protocol, plan-headless-agent-os-poc]
depends_on: [chief-app-protocol]
tags: [examples, poc, agentfield]
---

# AgentField Adapter Example

## TL;DR

- This is the POC 4B AgentField adapter proof.
- The adapter is user-space; Chief does not import AgentField or choose the agent graph.
- The adapter uses the same Chief HTTP protocol as the simple external orchestrator.
- It optionally detects a local AgentField SDK path, but the E2E gate does not require a running AgentField control plane.

## Contract

Run the gate:

```bash
scripts/poc4-agentfield-adapter.sh
```

The adapter writes an `agentfield_orchestrated_plan` contribution as `app:agentfield-adapter`, opens a Chief Ceremony, and proves the result is visible through Work Object, provenance, and CLI Ceremony state.

## Decisions

1. **Do not make AgentField a kernel dependency.**

   Rationale: AgentField belongs in user-space. Chief should expose protocol substrate to it, not embed its planning model.

2. **Keep the first adapter control-plane-free.**

   Rationale: the POC needs to prove the Chief boundary first. A full AgentField control-plane deployment can be added after the protocol remains stable.
