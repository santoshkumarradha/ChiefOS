---
id: plan-platform-mvp-demo
title: "Platform MVP Demo Plan"
status: draft
owners: [santosh]
last_updated: 2026-05-17
related: [architecture, chief-kernel, module-system, memory-substrate, security-model, agent-runtime, surfaces, pack-sdk, v0-scope]
tags: [plans, demo, mvp, platform, packs]
---

# Platform MVP Demo Plan

## TL;DR

- The demo must prove platform necessity: independent agent packs compose through Chief OS primitives instead of integrating with each other.
- The core object is a shared **Work Object**: one user goal, many pack contributions, one memory/provenance/authority trail.
- The MVP demo is not "Morning Brief only"; it is an all-day OS substrate demo with Brief as one surface.
- Every phase ends with an executable end-to-end test so the demo stays real as it grows.

## Demo thesis

Chief OS should exist because agent software needs OS-owned primitives that apps and frameworks cannot enforce uniformly:

| Primitive | Demo proof | Why app/framework is insufficient |
|---|---|---|
| Memory Graph | Packs contribute to the same Work Object without direct integration | App memories become silos |
| Capability Broker | Each pack gets narrow grants; external action is gated by Ceremony | Framework permissions are voluntary |
| Event Log / Provenance | Every contribution has a signed source chain | Apps can only self-report |
| Surfaces | Human sees one Work Object across Brief, Inbox, Ceremony, and Provenance | App UIs multiply attention surfaces |
| Pack SDK | A newly installed pack contributes immediately to existing work | Point integrations do not scale |

The one-line message:

> Agent apps do not integrate with each other. They integrate with the operating system.

## Primitive audit

This plan must not invent new kernel primitives. "Work Object" is demo/product language for an aggregate over existing primitives.

| Demo concept | Existing primitive used | New primitive? | Rule |
|---|---|---|---|
| Work Object | `mem://artifact/...` node with `body.kind = "work_object"` | No | Do not add `work://` without ADR |
| Contribution | Existing `finding`, `artifact`, or `decision` nodes linked to the Work Object | No | Do not add a `contribution` node type for the demo |
| Work graph | Existing Memory Graph edges: `derived-from`, `depends-on`, `refers-to`, `authored-by`, `cites`, `updates`, `contradicts` | No | Do not add edge kinds without ADR |
| Approval / denial | Capability Broker decision + Provenance Log entry + Ceremony token | No | Do not model authority as UI-local state |
| Work Object View | L4 surface projection over `/v1/work/:id` | No | Surface renders state; it does not own workflow logic |
| Risk pack install | Capability Pack lifecycle + manifest grants | No | No custom plugin path for the demo |

## MVP demo scenario

User drops an Acme contract PDF or markdown fixture into Chief and enters:

```text
Prepare the Acme follow-up.
```

Chief creates one Work Object. This is not a new substrate or URI scheme; it is a typed Memory Graph aggregate:

```text
mem://artifact/acme-follow-up
```

Independent packs contribute through OS primitives only:

| Pack | Reads | Writes | Human-visible result |
|---|---|---|---|
| `document-pack` | local contract file | obligations, risks, cited excerpts | "8 obligations extracted" |
| `calendar-pack` | obligations + calendar fixture | candidate meeting windows | "3 viable follow-up slots" |
| `email-pack` | obligations + meeting windows + prior context | drafted customer reply | "Reply ready for approval" |
| `risk-pack` | existing Work Object after live install | payment/legal/compliance risks | "Net-60 risk detected" |

The packs must not call each other. They communicate only through:

- `mem://...` nodes and edges
- Provenance Log entries and signed receipts
- capability handles
- `surface.pane` cards rendered by OS-owned surfaces

Implementation rule: do not add a new `work://` protocol for the demo. A Work Object is a `mem://artifact/...` node with existing typed edges to sources, findings, draft artifacts, and stale/recomputed descendants. Approvals and denials remain Provenance Log / Broker facts, referenced from the projection; they are not new Memory Graph edge kinds. If the implementation later needs a dedicated Memory Graph node type, edge kind, or URI scheme, that requires the normal ADR discipline.

## Final demo flow

Target length: 5 minutes.

| Beat | What viewer sees | What it proves |
|---|---|---|
| 1 | User creates `mem://artifact/acme-follow-up` from one file and one sentence | Chief has an OS-level work object, not app-local state |
| 2 | Document pack extracts obligations with citations | Pack can write structured memory through public SDK |
| 3 | Calendar and email packs pick up the same Work Object | Independent packs compose without direct integration |
| 4 | Work Object View shows one coherent outcome | Surfaces are projections over OS state |
| 5 | Email send is blocked until Ceremony | External authority is OS-gated |
| 6 | Install `risk-pack`; it contributes to existing object | Platform extensibility, not one-off automation |
| 7 | Provenance/Rewind view verifies or reverts the chain | OS owns the event log and rollback path |

## Phase plan

## End-to-end gate standard

Every phase gate must exercise the real Chief setup available at that phase:

- Boot real `AppState` with an ephemeral state directory for kernel-service tests.
- Use real HTTP routes, CLI calls, or the local socket/internal channel when the phase claims channel parity.
- Use real fixture files, real manifests, real Memory Graph writes, real Broker checks, and real Provenance/Event Log writes where those systems are in scope.
- Use live provider calls only when the phase explicitly needs provider behavior. Secrets such as `OPENROUTER_API_KEY` must come from the environment and must never be committed, logged, or copied into issues.
- Avoid pure unit tests as phase gates. Unit tests are allowed as supporting coverage, but each phase's named E2E test must run the actual integration path.
- Update the architecture/docs in the same phase commit whenever behavior, API shape, route surface, storage shape, or demo contract changes. Do not leave old architecture references for later cleanup unless the phase explicitly records a follow-up.

### Phase 0: Demo contract and fixtures

**Goal:** Lock the demo into one deterministic scenario.

**Files likely touched:**

- `plans/2026-05-17-platform-mvp-demo-plan.md`
- `docs/INDEX.md`
- `deploy/demo-fixtures/` or `crates/chief-core/tests/fixtures/`

**Build:**

- Acme contract fixture as markdown or PDF-derived text.
- Calendar fixture.
- Prior-email fixture.
- Expected Work Object JSON shape.
- Demo grant manifest for each fixture-backed pack.

**End-to-end test:**

```bash
cargo test -p chief-core --test platform_demo_contract
```

Acceptance:

- Fixture load boots real `AppState` with an ephemeral state directory and creates one deterministic `mem://artifact/acme-follow-up`.
- Snapshot JSON contains the same Work Object id, title, source refs, and empty contribution list.
- No fixture requires live Gmail, Calendar, or cloud storage credentials.
- Relevant docs still match the fixture contract and do not reference a new `work://` primitive.

### Phase 1: Work Object memory aggregate

**Goal:** Add the minimum shared object model that packs can independently extend.

**Files likely touched:**

- `crates/chief-core/src/brief.rs`
- `crates/chief-core/src/state.rs`
- `crates/chief-core/src/routes/`
- `crates/chief-cli/src/main.rs`
- `crates/chief-sdk/src/`
- `crates/chief-mem/src/`

**Build:**

- `WorkObject` projection DTO assembled from Memory Graph + Provenance Log state.
- `Contribution` projection DTO assembled from existing `finding`, `artifact`, or `decision` nodes.
- Memory edges use only existing kinds: `derived-from`, `depends-on`, `refers-to`, `authored-by`, `cites`, `updates`, `contradicts`.
- API route: `GET /v1/work/:id`.
- CLI route: `chief work show <id> --json`.
- Shared read handler that can be exposed over the local Unix-socket channel when enabled.

**End-to-end test:**

```bash
cargo test -p chief-core --test work_object_e2e
```

Acceptance:

- Creating a Work Object writes memory nodes and provenance entries.
- `GET /v1/work/acme-follow-up` and `chief work show acme-follow-up --json` return the same graph with sources and zero pack contributions.
- Test does not use mock cross-pack calls.
- The persisted identity is still a `mem://artifact/...` URI; `/v1/work/:id` is a projection route, not a new storage namespace.
- If the Unix-socket channel is not active yet, this phase must either expose the same read handler there or record a narrow channel-parity exception with a follow-up task.
- No new Memory Graph node type, edge kind, URI scheme, or capability kind is introduced by this phase.
- `docs/13-memory-substrate.md` documents the Work Object aggregate shape and projection rule.

### Phase 2: First three packs compose through OS memory

**Goal:** Document, calendar, and email packs produce one coherent outcome without direct coupling.

**Files likely touched:**

- `packs/document-pack/`
- `packs/calendar-pack/`
- `packs/email-pack/`
- `crates/chief-sdk/src/`
- `Cargo.toml`

**Build:**

- Pack A extracts obligations from the contract fixture.
- Pack B reads obligation nodes through a scoped `mem.read` grant and proposes slots from a fixture-backed calendar adapter.
- Pack C reads obligations and slots through a scoped `mem.read` grant and drafts a reply.
- Each pack uses public `chief-sdk` APIs only.

**End-to-end test:**

```bash
cargo test -p chief-core --test pack_interop_e2e
```

Acceptance:

- Running the three packs in any valid order converges to the same Work Object graph.
- No pack imports `chief_core`, `chief_mem`, or another pack.
- Static import scan passes.
- Every cross-pack read is authorized by a scoped Memory Graph grant tied to the Work Object; no broad "read all pack memory" shortcut.
- Every `mem.link` uses an existing edge kind; any desired new edge kind is documented as an ADR candidate, not implemented in the demo path.
- Final Work Object has at least:
  - 3 obligations,
  - 2 proposed slots,
  - 1 drafted reply,
  - source citations for every claim.

### Phase 3: Authority boundary and Ceremony

**Goal:** Show that OS-level authority gates the final external action.

**Files likely touched:**

- `crates/chief-core/src/broker.rs`
- `crates/chief-core/src/routes/v1_ceremony.rs`
- `apps/chief-brief-ui/src/surfaces/Ceremony.tsx`
- `apps/chief-brief-ui/src/surfaces/InboxDrawer.tsx`

**Build:**

- Email pack can draft but not send by default.
- Attempted send creates a pending Ceremony card.
- Ceremony approval token is bound to exact payload hash.
- Payload mutation invalidates approval.
- Trust Ledger / Region Router classify the send attempt as requiring Ceremony; the UI does not make that decision.

**End-to-end test:**

```bash
cargo test -p chief-core --test ceremony_payload_e2e
npm --prefix apps/chief-brief-ui run test -- Ceremony
```

Acceptance:

- Send attempt without grant returns `ceremony_required`.
- Approval for payload A does not authorize payload B.
- UI shows the pending approval and can approve only via hold-to-confirm.
- The approved action flows through Broker verification after Ceremony; the surface never directly ships the action.

### Phase 4: Work Object surface

**Goal:** Give the demo a visual center that is obviously OS-level and not an app dashboard.

**Files likely touched:**

- `apps/chief-brief-ui/src/surfaces/WorkObjectView.tsx`
- `apps/chief-brief-ui/src/api.ts`
- `apps/chief-brief-ui/src/types.ts`
- `apps/chief-brief-ui/src/styles/`
- `crates/chief-core/src/routes/v1_work.rs`
- `docs/30-surfaces.md`

**Build:**

- Central Work Object header.
- Contribution lanes by pack.
- Source/citation column.
- `/v1/work/:id` projection fields for `pack`, `kind`, `title`, `summary`, `source_refs`, and `authority_state`.
- Authority state rendered as handled, blocked, needs Ceremony, shipped.
- Provenance/rewind affordance.

**End-to-end test:**

```bash
npm --prefix apps/chief-brief-ui run test -- WorkObject
npm --prefix apps/chief-brief-ui run build
```

Acceptance:

- UI renders from `/v1/work/:id` only.
- Empty, partial, blocked, and completed states are covered.
- No business logic lives in the UI; the surface renders authority and contribution fields projected by L2.
- Same Work Object state is inspectable through CLI JSON, preserving Axiom 8 channel parity for the demo.
- Surface docs are updated if Work Object View becomes a named or reusable L4 surface.

### Phase 5: Live pack install proves platform extensibility

**Goal:** Install one new pack after the Work Object exists and show it contributes without custom integration.

**Files likely touched:**

- `packs/risk-pack/`
- `crates/chief-core/src/routes/`
- `crates/chief-sdk/src/manifest.rs`
- `docs/40-pack-sdk.md`
- `tools/chief-ts-lint/` if TS pack checks are needed

**Build:**

- `risk-pack` reads existing `mem://artifact/acme-follow-up` through its granted Work Object scope.
- It writes risk findings and links them to existing obligations.
- `POST /v1/packs/install-preview` checks manifest grants, usage reasons, and signature placeholder before activation.

**End-to-end test:**

```bash
cargo test -p chief-core --test live_pack_install_e2e
```

Acceptance:

- Before install, Work Object has no risk contribution.
- After install and tick, Work Object gains risk contribution.
- No existing pack or surface code is changed to know about `risk-pack`.
- The install path displays the risk pack's requested grants before activation, even in the demo.
- Install preview does not grant ambient authority; the pack still runs through `CapabilityContext`.

### Phase 6: Provenance replay and rewind

**Goal:** Make the proof technical but value-oriented: replay the object history and revert one contribution.

**Files likely touched:**

- `crates/chief-core/src/trust_ledger.rs`
- `crates/chief-core/src/routes/v1_trust.rs`
- `crates/chief-core/src/routes/v1_brief.rs`
- `crates/chief-core/src/routes/v1_work.rs`
- `apps/chief-brief-ui/src/surfaces/`
- `crates/chief-mem/src/lib.rs`
- `docs/13-memory-substrate.md`

**Build:**

- `GET /v1/work/:id/provenance`.
- `POST /v1/work/:id/rewind`.
- Rewind tombstones a contribution while preserving replay history as signed event log state plus Memory Graph `decision` nodes.

**End-to-end test:**

```bash
cargo test -p chief-core --test work_rewind_e2e
```

Acceptance:

- Rewind of email draft removes draft contribution from active Work Object.
- Provenance still shows the original draft and the rewind event.
- Downstream affected contributions are marked stale or recomputed.
- No new Memory Graph node type, edge kind, URI scheme, or capability kind is introduced; `rewind_event` and `stale_marker` are `decision` node body kinds.

### Phase 7: One-command demo packaging

**Goal:** Make the whole proof runnable from a clean checkout.

**Files likely touched:**

- `deploy/docker/`
- `crates/chief-core/src/bin/demo_run.rs`
- `apps/chief-brief-ui/README.md`
- `README.md`

**Build:**

- Demo runner seeds fixtures.
- Runs all demo packs.
- Serves Work Object View and Brief.
- Provides deterministic logs and reset command.
- Exposes HTTP and CLI inspection paths for the same Work Object.

**End-to-end test:**

```bash
docker compose -f deploy/docker/docker-compose.yml up --build
curl -fsS http://localhost:8080/v1/work/acme-follow-up
```

Acceptance:

- Clean volume reaches completed `mem://artifact/acme-follow-up` state without manual database edits.
- UI at `http://localhost:8080` shows Work Object, pending Ceremony, and pack contributions.
- Reset removes all generated state and rerun produces same core graph.
- `curl /v1/work/acme-follow-up` and CLI JSON agree on the Work Object graph.
- Local socket inspection returns the same graph when the socket channel is enabled.

## Parallelizable workstreams

| Workstream | Can start after | Notes |
|---|---|---|
| Work Object schema/API | Phase 0 | Blocks packs and UI |
| Fixture design | Phase 0 | Can proceed with schema draft |
| Pack implementations | Phase 1 | Packs should have disjoint directories |
| Work Object UI | Phase 1 | Can use fixture JSON before backend completes |
| Ceremony payload binding | Phase 1 | Independent of specific packs |
| Risk pack install path | Phase 2 | Needs stable manifest and Work Object read |
| Docker packaging | Phase 3 | Can use partial flow, then expand |

## What this demo intentionally does not prove

- Full daily-driver OS install.
- Real Gmail/Calendar OAuth.
- Full marketplace.
- GPU scheduling.
- Multi-device sync.
- Enterprise policy.
- Voice input.

Those are important later, but they distract from the phase-zero platform proof.

## Final verification checklist

- [ ] Four packs contribute to one Work Object through OS primitives only.
- [ ] No pack imports another pack or a kernel-internal crate.
- [ ] Work Object is represented as Memory Graph state (`mem://artifact/...`), not a new storage protocol.
- [ ] Demo introduces no new Memory Graph node types, edge kinds, URI schemes, or capability kinds without ADR.
- [ ] Cross-pack reads are explicitly grant-scoped to the Work Object.
- [ ] At least one external action is blocked until Ceremony.
- [ ] Approval token is payload-bound.
- [ ] Newly installed pack contributes to existing Work Object without UI/backend special-casing.
- [ ] Work Object graph has memory, event-log, capability, and surface projections.
- [ ] Work Object is available through HTTP, CLI JSON, local socket/internal channel, and L4 surface.
- [ ] Rewind changes active state while preserving provenance.
- [ ] Demo runs from a clean checkout with one command.

## Risks

| Risk | Mitigation |
|---|---|
| Demo becomes a workflow app | Keep Work Object state in L2; UI must only project `/v1/work/:id` |
| Work Object becomes an accidental new substrate | Store it as `mem://artifact/...`; any new Memory Graph type or URI scheme requires ADR |
| Packs accidentally couple through Rust imports | Static import scan in every pack E2E test |
| Cross-pack reads weaken capability boundaries | Require Work Object-scoped `mem.read` and `mem.link` grants for every pack contribution |
| Demo needs a new edge/capability kind mid-build | Stop and write an ADR candidate; do not smuggle it into implementation as "demo-only" |
| Scenario feels synthetic | Use realistic contract/calendar/email fixtures, but keep them deterministic |
| Ceremony dominates the story | Use it as one beat; the main proof is pack interoperability through OS primitives |
| Too much to build before showing value | Phase 2 already proves the central thesis in terminal/API form |

## Related

- [`../docs/10-architecture.md`](../docs/10-architecture.md) — layers and L2/L3/L4 boundaries
- [`../docs/11-chief-kernel.md`](../docs/11-chief-kernel.md) — kernel services
- [`../docs/12-module-system.md`](../docs/12-module-system.md) — packs and stacks
- [`../docs/13-memory-substrate.md`](../docs/13-memory-substrate.md) — Memory Graph
- [`../docs/14-security-model.md`](../docs/14-security-model.md) — capability security
- [`../docs/30-surfaces.md`](../docs/30-surfaces.md) — surfaces
- [`../docs/40-pack-sdk.md`](../docs/40-pack-sdk.md) — SDK contract
