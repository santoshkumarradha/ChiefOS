---
id: plan-headless-agent-os-poc
title: "Headless Agent OS POC Ladder"
status: active
owners: [santosh]
last_updated: 2026-05-18
related: [headless-chief-node, architecture, chief-kernel, memory-substrate, security-model, agent-runtime, pack-sdk, surfaces, v0-scope, plan-platform-mvp-demo]
tags: [plans, poc, headless, agent-server, platform]
---

# Headless Agent OS POC Ladder

## TL;DR

- The completed Platform MVP demo is **POC 0**: it seeded the Work Object, pack, Ceremony, risk-pack, provenance, rewind, Docker, HTTP, CLI, and UI paths.
- **POC 1** proves Chief can be used as a headless OS-like node without a UI process.
- **POC 2** proves a real app can be built on Chief primitives through public protocol, without importing kernel internals.
- **POC 3** proves why authority, approval, provenance, and rewind must be OS-level substrate, not app-local features.
- **POC 4** proves external agent servers, including AgentField-style apps, can bring their own orchestration while Chief provides identity, capabilities, memory, audit, approval, and rewind.

## Prerequisites

| Dependency | Status | Notes |
|---|---|---|
| [`plan-platform-mvp-demo`](./2026-05-17-platform-mvp-demo-plan.md) | complete | POC 0 substrate seed; deterministic Acme fixture path. |
| [`headless-chief-node`](../docs/19-headless-chief-node.md) | draft | Product-definition source for headless-first Chief Node. |
| [`chief-kernel`](../docs/11-chief-kernel.md) | draft | L2 services and channel-parity target. |
| [`memory-substrate`](../docs/13-memory-substrate.md) | draft | Work Object aggregate and rewind semantics. |
| [`security-model`](../docs/14-security-model.md) | draft | Broker, Ceremony, and payload-bound authority. |

## Boundary

Chief OS is not the semantic orchestrator. It mechanically coordinates resources, state, authority, event delivery, and runtime lifecycle. Planning, agent selection, task decomposition, and domain workflow live in packs, first-party stack code, or external agent servers such as AgentField, LangGraph, CrewAI, AutoGen, or custom services.

The POC ladder must demonstrate this boundary:

- Simple packs work without an orchestrator.
- Complex external agent servers can bring their own orchestration.
- Both use the same Chief protocol boundary.
- Neither bypasses Broker, Memory Graph, Provenance/Event Log, or Ceremony.

## POC 0 — Platform substrate seed

**Status:** complete.

**Question answered:** Can independent packs compose through one OS-owned Work Object instead of integrating with each other?

**What exists now:**

- Deterministic Docker runner.
- `GET /v1/work/acme-follow-up`.
- `chief work show acme-follow-up --json`.
- Document, calendar, email, and risk packs contributing through the platform path.
- Payload-bound Ceremony approval.
- Work Object surface.
- Provenance and rewind routes.

**Remaining use:** POC 0 stays as the regression fixture for POC 1-3.

## POC 1 — Headless Chief Node product proof

**Status:** complete for the POC ladder gate.

**Question answered:** Can Chief be operated like an OS-like server without UI?

**Demo flow:**

```text
docker compose up chief-node
curl /v1/status
curl /v1/work/acme-follow-up
chief work show acme-follow-up --json
curl /v1/work/acme-follow-up/provenance
curl /v1/inbox
```

**Build:**

1. Package the current demo runner as a clearly named headless node path in docs and scripts.
2. Add a `poc1-headless-node` script or make target that starts the deterministic node and runs the operator checks.
3. Add `/v1/status` or equivalent health/status projection showing active services, state directory, demo fixture id, and available channels.
4. Add live user-facing E2E gate using `OPENROUTER_API_KEY` from the environment without committing secrets.
5. Document the operator walkthrough.
6. Ensure the Docker demo can pass with no UI build or UI process.

**Acceptance:**

- [x] One command starts the headless node.
- [x] No UI process is required for the POC 1 gate.
- [x] HTTP and CLI read the same Work Object state.
- [x] Inbox/Ceremony state is inspectable headlessly over HTTP.
- [x] Status endpoint makes it obvious this is an OS node, not a UI demo server.
- [x] Live OpenRouter gate verifies user-facing `/v1/inbox`, `/v1/brief`, and `/` state.
- [x] Existing POC 0 Docker/Work Object regression still passes.

## POC 2 — First real Chief app proof

**Status:** complete for the POC ladder gate.

**Question answered:** Can a developer build an actual Chief app on top of OS primitives instead of linking to kernel internals?

**Demo flow:**

```text
Chief Node runs
external sales-followup app starts as a separate process
app reads Work Object over public HTTP
app writes a recommendation contribution through public HTTP
Work Object, Inbox/Provenance, CLI, and optional UI all show the same app contribution
```

**Build:**

1. Add public contribution-write API if the current write path is too internal.
2. Attribute external app writes to an app principal while still passing through Broker.
3. Add a tiny external app example that imports no `chief-core` internals.
4. Add an E2E script that runs Chief Node plus the external app process: `scripts/poc2-sales-followup-app.sh`.
5. Document the app developer contract: read Work Object, request/use scoped capability, write contribution, inspect provenance.

**Acceptance:**

- [x] External app has no internal crate imports.
- [x] External app reads work state through public protocol.
- [x] External app writes a contribution through public protocol.
- [x] Contribution is persisted as an existing Memory Graph node type, not a new primitive.
- [x] Provenance identifies the external app principal.
- [x] HTTP and CLI show the same app contribution.
- [x] Developer walkthrough exists at [`chief-app-protocol`](../docs/42-chief-app-protocol.md).

## POC 3 — Authority boundary and rewind proof

**Status:** complete for the POC ladder gate.

**Question answered:** Why does this need to be OS-level rather than an app-local agent feature?

**Demo flow:**

```text
sales-followup app proposes Acme send
Chief blocks send
headless Ceremony approval is required
payload A approval cannot authorize payload B
approved action emits receipt
rewind removes active app contribution
provenance remains replayable
downstream state is marked stale
```

**Build:**

1. Add headless Ceremony CLI/API flow for listing and approving pending actions.
2. Add a bypass demo script that approves payload A, mutates payload B, and shows Broker rejection.
3. Add before/after JSON snapshots for rewind using the external app contribution.
4. Add CLI parity:
   - `chief inbox list --json`,
   - `chief ceremony approve ...`,
   - `chief work provenance ... --json`,
   - `chief work rewind ...`.
5. Tighten docs to show that Ceremony is authority substrate, not semantic orchestration.

**Acceptance:**

- [x] Consequential action cannot ship without Broker/Ceremony.
- [x] Approval token is bound to exact payload hash.
- [x] Mutated payload is rejected.
- [x] Rewind changes active Work Object state without deleting event history.
- [x] HTTP and CLI show the same pending Ceremony state.
- [x] HTTP and CLI show the same rewind state.

## POC 4 — Bring-your-own orchestrator proof

**Question answered:** Can another agent system use Chief as its OS substrate while bringing its own orchestration?

**Demo flow:**

```text
Chief Node runs
external AgentField/simple agent-server process starts
external process owns planning and agent selection
external process reads Work Object over public protocol
external process requests scoped capability
external process writes a contribution
Chief logs external principal provenance
high-risk action is still blocked by Ceremony
```

**Build:**

1. Add external principal identity and scoped capability request path for non-pack agent servers.
2. Add a simple external agent-server example that imports no `chief-core` internals.
3. Add AgentField example/adapter once the HTTP contract is stable enough.
4. Add E2E test that runs Chief Node plus an external process.
5. Document the difference between simple packs, app-owned orchestrators, and Chief OS substrate.

**Acceptance:**

- [ ] External process owns semantic orchestration; Chief does not choose agents or plans.
- [ ] External process authenticates as its own principal.
- [ ] External process can read only scoped Work Object state.
- [ ] External process can write a contribution through public protocol.
- [ ] Provenance identifies the external principal.
- [ ] High-risk action is still blocked by Chief, regardless of external orchestrator.

## Dependencies

```text
POC 0 complete
   |
   v
POC 1 headless node
   |
   v
POC 2 first Chief app
   |
   v
POC 3 authority + rewind on app contribution
   |
   v
POC 4 external orchestrator / AgentField
```

POC 1 must come before POC 2 because app developers need a stable headless node contract. POC 3 comes after POC 2 so the authority proof is attached to a real external app workflow, not only a fixture. POC 4 comes last because AgentField/custom orchestrators should prove they are user-space applications on the same contract, not privileged kernel extensions.

## GitHub issue map

| Issue | Depends on | Purpose |
|---|---|---|
| [#71](https://github.com/santoshkumarradha/ChiefOS/issues/71) Tracker: POC 1-4 Chief App Platform Proofs | none | Milestone overview and dependency map. |
| [#72](https://github.com/santoshkumarradha/ChiefOS/issues/72) POC 1A: Headless operator walkthrough + script | POC 0 | One-command headless node proof. |
| [#73](https://github.com/santoshkumarradha/ChiefOS/issues/73) POC 1B: Status endpoint and no-UI gate | #72 | Make node/server nature inspectable. |
| [#74](https://github.com/santoshkumarradha/ChiefOS/issues/74) POC 1C: Live OpenRouter user-facing gate | #73 | Show headless node can produce live user-facing state with real LLM/API IO. |
| [#75](https://github.com/santoshkumarradha/ChiefOS/issues/75) POC 2A: Public Work contribution API | #74 | Let external apps write through protocol, Broker, Memory Graph, and Provenance. |
| [#76](https://github.com/santoshkumarradha/ChiefOS/issues/76) POC 2B: External sales-followup app E2E | #75 | Prove a real app process can use Chief without internal imports. |
| [#77](https://github.com/santoshkumarradha/ChiefOS/issues/77) POC 2C: Chief app developer walkthrough | #76 | Document the small "Swift-like" app contract for Chief. |
| [#78](https://github.com/santoshkumarradha/ChiefOS/issues/78) POC 3A: Headless Ceremony for external app | #76 | Authority proof without UI on the real app workflow. |
| [#79](https://github.com/santoshkumarradha/ChiefOS/issues/79) POC 3B: Payload-bound rejection + rewind for external app | #78 | Show OS-bound approval cannot be reused and rewind preserves history. |
| [#80](https://github.com/santoshkumarradha/ChiefOS/issues/80) POC 4A: External orchestrator E2E | #79 | Prove app-owned orchestration on Chief substrate. |
| [#81](https://github.com/santoshkumarradha/ChiefOS/issues/81) POC 4B: AgentField example adapter | #80 | Prove AgentField can be a user-space app server on the same protocol. |

## Verification

Each issue must land with a real end-to-end check, not only unit tests.

- POC 1 gate: deterministic Docker or local runner plus HTTP and CLI checks.
- Live user-facing gate: `scripts/poc-live-openrouter.sh` with `OPENROUTER_API_KEY` from the environment; this must hit real OpenRouter and user-facing `/v1/inbox`, `/v1/brief`, and `/` surfaces.
- POC 2 gate: Chief Node plus external app process, no internal crate imports, public contribution write, provenance attributed to external principal.
- POC 3 gate: real Ceremony/Broker/Rewind path with HTTP and CLI assertions, using the external app workflow.
- POC 4 gate: Chief Node plus external orchestrator process, no internal crate imports, provenance attributed to external principal.

Before merging the feature branch:

```bash
cargo test -p chief-core --test work_object_e2e --test ceremony_payload_e2e --test work_rewind_e2e
cargo test -p chief-core --bin platform_demo_run
cargo test -p chief-cli
scripts/poc-live-openrouter.sh
scripts/poc2-sales-followup-app.sh
scripts/poc3-external-app-ceremony.sh
scripts/poc3-payload-rewind.sh
CHIEF_DEMO_PORT=18081 docker compose -f deploy/docker/docker-compose.yml up --build -d
curl -fsS http://localhost:18081/v1/work/acme-follow-up
```

Add more exact commands as POC 1-3 implementation adds scripts and external-process tests.
