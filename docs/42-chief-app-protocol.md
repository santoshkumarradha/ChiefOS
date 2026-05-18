---
id: chief-app-protocol
title: "Chief App Protocol"
status: draft
owners: [santosh]
last_updated: 2026-05-18
related: [headless-chief-node, memory-substrate, pack-sdk, security-model, plan-headless-agent-os-poc]
depends_on: [headless-chief-node, memory-substrate, security-model]
tags: [developer-platform, app-protocol, poc]
---

# Chief App Protocol

## TL;DR

- A Chief app is user-space software that uses Chief as its OS substrate.
- The app owns domain logic; Chief owns identity, capabilities, memory, provenance, approval, and rewind.
- POC 2 uses the smallest protocol loop: read Work Object, write contribution, inspect provenance.
- This is a protocol contract over existing primitives, not a new kernel primitive.

## Context

The platform proof should show that Chief is useful because developers can build apps on top of it. The app should not be compiled into Chief, import `chief-core`, or receive private kernel entitlements.

This mirrors the OS boundary:

| Traditional OS | Chief OS |
|---|---|
| App uses files, sockets, users, permissions | App uses Work Objects, HTTP/socket, principals, capabilities |
| Kernel does not decide app workflow | Chief does not decide app workflow |
| App writes state through OS APIs | App writes Memory Graph contributions through Chief APIs |
| OS logs and permissions outlive app process | Provenance, Broker checks, Ceremony, and rewind outlive the app process |

## Contract

Minimum POC 2 loop:

```text
GET  /v1/work/:id
POST /v1/work/:id/contributions
GET  /v1/work/:id/provenance
```

POC 3A adds headless authority:

```text
POST /v1/work/:id/contributions   # with ceremony payload
GET  /v1/ceremony
chief ceremony list --json
POST /v1/ceremony/:id/approve
```

POC 3B adds payload rejection and rewind:

```text
POST /v1/ceremony/:id/approve       # mutated hash returns payload_hash_mismatch
POST /v1/work/:id/rewind
GET  /v1/work/:id/provenance
chief work show :id --json
```

POC 4A proves bring-your-own orchestration:

```text
external process selects agents and plan
GET  /v1/work/:id
POST /v1/work/:id/contributions   # orchestrated result plus Ceremony payload
GET  /v1/work/:id/provenance
chief ceremony list --json
```

Request identity:

```http
x-chief-principal: app:<name>
```

Contribution write shape:

```json
{
  "node_type": "finding",
  "kind": "next_best_action",
  "title": "Call Acme before sending the draft",
  "summary": "External app recommendation.",
  "authority_state": "handled",
  "source_refs": ["mem://file/..."],
  "body": {
    "app": "sales-followup-app",
    "recommendation": "call_first"
  }
}
```

Contribution write with Ceremony:

```json
{
  "node_type": "artifact",
  "kind": "proposed_send",
  "title": "Send Acme follow-up after call",
  "summary": "External app drafted an email send that requires Ceremony.",
  "body": {
    "draft_action": "email.send",
    "payload": {
      "to": "maya@acme.example",
      "subject": "Acme follow-up",
      "body": "Confirm clause 4 before sending."
    }
  },
  "ceremony": {
    "title": "Send Acme follow-up after call",
    "summary": "sales-followup app needs Ceremony before sending externally.",
    "category": "email.send",
    "payload": {
      "to": "maya@acme.example",
      "subject": "Acme follow-up",
      "body": "Confirm clause 4 before sending."
    }
  }
}
```

Invariants:

- `node_type` is one of `finding`, `artifact`, or `decision`.
- Chief sets `body.work_object_id` from the URL path.
- Chief checks Broker `mem.write` before writing.
- Chief stores an ordinary Memory Graph node.
- Chief projects the node as a Work Object contribution.
- Provenance attributes the contribution to the app principal.
- If `ceremony` is present, Chief creates a real pending Ceremony and Inbox item.
- Ceremony approval is bound to the exact payload hash Chief returned.
- The app does not coordinate with packs directly.

## Decisions

1. **Apps use protocol first.**

   Rationale: protocol access is the product proof. A developer can use Python, TypeScript, Rust, AgentField, LangGraph, or a shell script as long as they enter through Chief's public boundary.

2. **The OS does not orchestrate.**

   Rationale: an app can choose prompts, agents, tools, task graphs, and workflow. Chief enforces substrate concerns: identity, capability, memory, audit, approval, and rewind.

3. **Contributions are projected, not primitive.**

   Rationale: POC 2 must not add a `contribution` node type. Contributions remain existing Memory Graph nodes linked by `body.work_object_id`.

4. **The first app is intentionally plain.**

   Rationale: [`../examples/sales-followup-app`](../examples/sales-followup-app) uses only Python stdlib HTTP so the demo proves the OS boundary, not SDK ergonomics.

5. **External orchestration is user-space.**

   Rationale: [`../examples/external-orchestrator`](../examples/external-orchestrator) chooses its own local agents and plan. Chief only sees protocol calls, contributions, provenance, and Ceremony.

## Acceptance

- [x] Example app imports no Chief kernel crates.
- [x] Example app reads a Work Object over HTTP.
- [x] Example app writes a contribution over HTTP.
- [x] Work Object and provenance show `app:sales-followup`.
- [x] CLI reads the same contribution as HTTP.
- [x] E2E gate: `scripts/poc2-sales-followup-app.sh`.
- [x] Ceremony gate: `scripts/poc3-external-app-ceremony.sh`.
- [x] Payload + rewind gate: `scripts/poc3-payload-rewind.sh`.
- [x] External orchestrator gate: `scripts/poc4-external-orchestrator.sh`.

## Related

- [`headless-chief-node`](./19-headless-chief-node.md) — product and architecture boundary
- [`memory-substrate`](./13-memory-substrate.md) — Work Object and contribution representation
- [`pack-sdk`](./40-pack-sdk.md) — future SDK contract
- [`plan-headless-agent-os-poc`](../plans/2026-05-18-headless-agent-os-poc-plan.md) — POC ladder
