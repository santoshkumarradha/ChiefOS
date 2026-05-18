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
- **POC 2** proves why authority, approval, provenance, and rewind must be OS-level substrate, not app-local features.
- **POC 3** proves external agent servers, including AgentField-style apps, can bring their own orchestration while Chief provides identity, capabilities, memory, audit, approval, and rewind.

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
4. Add CLI parity for headless inspection gaps:
   - inbox list,
   - Work Object provenance,
   - pack install preview if not already exposed.
5. Document the operator walkthrough.
6. Ensure the Docker demo can pass with no UI build or UI process.

**Acceptance:**

- [ ] One command starts the headless node.
- [ ] No UI process is required for the POC 1 gate.
- [ ] HTTP and CLI read the same Work Object and provenance state.
- [ ] Inbox/Ceremony state is inspectable headlessly.
- [ ] Status endpoint makes it obvious this is an OS node, not a UI demo server.
- [ ] Existing POC 0 Docker/Work Object regression still passes.

## POC 2 — Authority boundary and rewind proof

**Question answered:** Why does this need to be OS-level rather than an app-local agent feature?

**Demo flow:**

```text
email-pack proposes Acme send
Chief blocks send
headless Ceremony approval is required
payload A approval cannot authorize payload B
approved action emits receipt
rewind removes active contribution
provenance remains replayable
downstream state is marked stale
```

**Build:**

1. Add headless Ceremony CLI/API flow for listing and approving pending actions.
2. Add a bypass demo script that approves payload A, mutates payload B, and shows Broker rejection.
3. Add before/after JSON snapshots for rewind.
4. Add CLI parity:
   - `chief inbox list --json`,
   - `chief ceremony approve ...`,
   - `chief work provenance ... --json`,
   - `chief work rewind ...`.
5. Tighten docs to show that Ceremony is authority substrate, not semantic orchestration.

**Acceptance:**

- [ ] Consequential action cannot ship without Broker/Ceremony.
- [ ] Approval token is bound to exact payload hash.
- [ ] Mutated payload is rejected.
- [ ] Rewind changes active Work Object state without deleting event history.
- [ ] HTTP and CLI show the same approval, provenance, and rewind state.

## POC 3 — External agent server proof

**Question answered:** Can another agent system use Chief as its OS substrate while bringing its own orchestration?

**Demo flow:**

```text
Chief Node runs
external AgentField/simple agent-server process starts
external process reads Work Object over public protocol
external process requests scoped capability
external process writes a contribution
Chief logs external principal provenance
high-risk action is still blocked by Ceremony
```

**Build:**

1. Add public contribution-write API if the current write path is too internal.
2. Add external principal identity and scoped capability request path for non-pack agent servers.
3. Add a simple external agent-server example that imports no `chief-core` internals.
4. Add AgentField example/adapter once the HTTP contract is stable enough.
5. Add E2E test that runs Chief Node plus an external process.
6. Document the difference between simple packs, app-owned orchestrators, and Chief OS substrate.

**Acceptance:**

- [ ] External process has no internal crate imports.
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
POC 2 authority + rewind
   |
   v
POC 3 external agent server
```

POC 1 must come before POC 3 because external agent servers need a stable headless node contract. POC 2 can overlap with POC 1 only for CLI work, but the end-to-end authority demo should use the finalized POC 1 headless runner.

## GitHub issue map

| Issue | Depends on | Purpose |
|---|---|---|
| [#71](https://github.com/santoshkumarradha/ChiefOS/issues/71) Tracker: POC 1-3 Headless Agent OS Proofs | none | Milestone overview and dependency map. |
| [#72](https://github.com/santoshkumarradha/ChiefOS/issues/72) POC 1A: Headless operator walkthrough + script | POC 0 | One-command headless node proof. |
| [#73](https://github.com/santoshkumarradha/ChiefOS/issues/73) POC 1B: Status endpoint and no-UI gate | #72 | Make node/server nature inspectable. |
| [#74](https://github.com/santoshkumarradha/ChiefOS/issues/74) POC 1C: CLI parity for inbox/provenance/pack preview | #72 | Make protocol parity real for headless operation. |
| [#75](https://github.com/santoshkumarradha/ChiefOS/issues/75) POC 2A: Headless Ceremony CLI/API | #74 | Authority proof without UI. |
| [#76](https://github.com/santoshkumarradha/ChiefOS/issues/76) POC 2B: Payload mutation rejection demo | #75 | Show OS-bound approval cannot be reused. |
| [#77](https://github.com/santoshkumarradha/ChiefOS/issues/77) POC 2C: Rewind before/after snapshots | #75 | Show active-state rollback plus retained history. |
| [#78](https://github.com/santoshkumarradha/ChiefOS/issues/78) POC 3A: External principal + scoped capability path | #74 | Let non-pack agent servers enter through protocol. |
| [#79](https://github.com/santoshkumarradha/ChiefOS/issues/79) POC 3B: External simple agent-server E2E | #78 | Prove no internal imports. |
| [#80](https://github.com/santoshkumarradha/ChiefOS/issues/80) POC 3C: AgentField example adapter | #79 | Prove app-owned orchestration on Chief substrate. |

## Verification

Each issue must land with a real end-to-end check, not only unit tests.

- POC 1 gate: deterministic Docker or local runner plus HTTP and CLI checks.
- Live user-facing gate: `scripts/poc-live-openrouter.sh` with `OPENROUTER_API_KEY` from the environment; this must hit real OpenRouter and user-facing `/v1/inbox`, `/v1/brief`, and `/` surfaces.
- POC 2 gate: real Ceremony/Broker/Rewind path with HTTP and CLI assertions.
- POC 3 gate: Chief Node plus external process, no internal crate imports, provenance attributed to external principal.

Before merging the feature branch:

```bash
cargo test -p chief-core --test work_object_e2e --test ceremony_payload_e2e --test work_rewind_e2e
cargo test -p chief-core --bin platform_demo_run
cargo test -p chief-cli
scripts/poc-live-openrouter.sh
CHIEF_DEMO_PORT=18081 docker compose -f deploy/docker/docker-compose.yml up --build -d
curl -fsS http://localhost:18081/v1/work/acme-follow-up
```

Add more exact commands as POC 1-3 implementation adds scripts and external-process tests.
