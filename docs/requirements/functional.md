---
id: req-functional
title: "Functional Requirements"
status: draft
owners: [santosh]
last_updated: 2026-04-21
related: [v0-scope, chief-kernel, surfaces, module-system]
tags: [requirements, functional]
---

# Functional Requirements

Append-only. Each requirement has a stable ID `F-NNN`. Reference from docs, ADRs, tests.

## Night Handoff → Morning Brief loop

| ID | Requirement | v0 | v1 | v2 |
|---|---|---|---|---|
| F-001 | User can type 30-second intent at bedtime and have Chief commit it to Memory Graph | ✓ | | |
| F-002 | User can speak 30-second intent (whisper.cpp local) | | ✓ | |
| F-003 | Morning Brief renders ≤3s on spacebar tap | ✓ | | |
| F-004 | Morning Brief shows "needs you" vs "handled" lanes | ✓ | | |
| F-005 | Each card has a provenance receipt clickable to chain | ✓ | | |
| F-006 | Trust Ledger viewer shows per-category 1–5 with growth animation | ✓ | | |
| F-007 | Rewind UI scrubs back 1h / 4h / 24h intervals | ✓ | | |
| F-008 | User can approve a low-stakes card with one tap | ✓ | | |
| F-009 | Medium-stakes surface an Evidence Card inline | ✓ | | |
| F-010 | High-stakes surface a Ceremony with 3-sec hold + biometric | ✓ | | |

## Capability Packs

| ID | Requirement | v0 | v1 | v2 |
|---|---|---|---|---|
| F-101 | `chief install <flake-url>` fetches and installs signed pack | ✓ | | |
| F-102 | Install UI displays each capability grant kind explicitly before approval | ✓ | | |
| F-103 | `chief revoke <pack>` atomically removes pack + its grants | ✓ | | |
| F-104 | Packs declare `agents, tools, ingesters, panes, rituals, policies, grants` in a flake | ✓ | | |
| F-105 | Stacks bundle packs + house rules + ledger initialization | ✓ | | |
| F-106 | Pack registry browsable from Surface and CLI | | ✓ | |
| F-107 | Author can publish a signed pack via `chief-sdk publish` | | ✓ | |
| F-108 | Community trust score visible before install | | ✓ | |

## Model Router

| ID | Requirement | v0 | v1 | v2 |
|---|---|---|---|---|
| F-201 | Model Router is a kernel service with a stable `ModelBackend` trait | ✓ | | |
| F-202 | Routing decision considers policy at 5 granularities (system / category / pack / agent / call) | ✓ | | |
| F-203 | Sensitive categories (finance, health, personal, family, legal.private) never route to cloud | ✓ | | |
| F-204 | Adding a backend is a contained change (implement trait + register at boot) | ✓ | | |
| F-205 | User-visible indicator when an op runs on-device | ✓ | | |
| F-206 | Per-category / per-pack routing override configurable | | ✓ | |

## Substrate spine (from research 2026-04-21-ai-native-primitive-rethinks.md)

| ID | Requirement | v0 | v1 | v2 |
|---|---|---|---|---|
| F-301 | Signed typed event log: every action emits a typed, signed tuple | ✓ | | |
| F-302 | Content-addressed FS with blake3 CIDs and property-graph index | ✓ | | |
| F-303 | Human-folder FUSE shim provides legacy `~/` view | ✓ | | |
| F-304 | Clipboard (`kbd://`) bus: typed, content-addressed, agent-watchable | | ✓ | |
| F-305 | HAX Inbox is the single notification surface for all agent asks | ✓ | | |
| F-306 | Omnibar = semantic search over Memory Graph (global hotkey) | ✓ | | |

## Provenance & Rollback

| ID | Requirement | v0 | v1 | v2 |
|---|---|---|---|---|
| F-401 | Every external action signed by user key via ceremony token | ✓ | | |
| F-402 | Provenance Log is append-only + Merkle-linked | ✓ | | |
| F-403 | Provenance Explorer walks any artifact's chain | ✓ | | |
| F-404 | `chief rewind <time>` reverts files + memory + queued actions atomically | ✓ | | |
| F-405 | In-toto v1 statements emitted for every agent action | ✓ | | |
| F-406 | Rekor-compatible mirror available (local or public) | ✓ | | |

## Surfaces

| ID | Requirement | v0 | v1 | v2 |
|---|---|---|---|---|
| F-501 | Morning Brief surface | ✓ | | |
| F-502 | Live Agent View at 60fps with ≥20 agents | ✓ | | |
| F-503 | Provenance Explorer | ✓ | | |
| F-504 | Ceremony UI | ✓ | | |
| F-505 | Chat Pane (typed input) | ✓ | | |
| F-506 | Trust Ledger Viewer | ✓ | | |
| F-507 | HAX Inbox (notification surface) | ✓ | | |
| F-508 | Omnibar | ✓ | | |
| F-509 | Quarterly Review (scaffold at v0, first run v1) | partial | ✓ | |

## Deploy targets

| ID | Requirement | v0 | v1 | v2 |
|---|---|---|---|---|
| F-601 | Bare-metal ISO installer | ✓ | | |
| F-602 | macOS-host VM image (OrbStack/UTM compatible) | ✓ | | |
| F-603 | Bootable USB live image | ✓ | | |
| F-604 | All three targets build from one flake via CI | ✓ | | |
| F-605 | Atomic rollback on failed update | ✓ | | |

## Explicit non-requirements (v0)

- No voice synthesis / speaking (v1).
- No multi-human delegation (v2).
- No money movement (v2).
- No e-signature of binding contracts (v2; drafting yes, signing later).
- No federated Chief-to-Chief communication (v2).
- No enterprise policies / SSO (v3).
- No phone-native companion (v1; web-mirror only at v0).

## Related

- [`../02-architecture.md`](../02-architecture.md) — layer placement
- [`../13-v0-scope-90-day.md`](../13-v0-scope-90-day.md) — v0 commitment
- [`../14-risks-open-questions.md`](../14-risks-open-questions.md) — open questions on scope
