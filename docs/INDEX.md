---
id: docs-index
title: "Docs Index"
status: stable
owners: [santosh]
last_updated: 2026-04-22
tags: [meta, navigation]
---

# Docs Index

Machine-readable map of all Chief OS documentation. Update when adding or renaming a doc.

Grouped by topic (numbered ranges):

- `00-09` — overview, design philosophy
- `10-19` — architecture and system design
- `30-39` — surfaces and UI
- `40-49` — developer platform
- `50-59` — deployment and scope
- `60-69` — go-to-market

## Core design (00-09)

| ID | File | Status | Purpose |
|---|---|---|---|
| `charter` | [`../CHARTER.md`](../CHARTER.md) | stable | 10 axioms; constitution |
| `agents-md` | [`../AGENTS.md`](../AGENTS.md) | stable | Entry point for AI agents |
| `north-star` | [`00-north-star.md`](./00-north-star.md) | draft | Mission, Person Zero, wedge, staged vision |
| `hax-principles` | [`01-hax-principles.md`](./01-hax-principles.md) | draft | HAX theory → OS primitives |
| `apple-design` | [`02-apple-design-principles.md`](./02-apple-design-principles.md) | draft | Product design guardrails |

## Architecture and system (10-19)

| ID | File | Status | Purpose |
|---|---|---|---|
| `architecture` | [`10-architecture.md`](./10-architecture.md) | draft | L0–L4 layers, substrate spine, language policy |
| `chief-kernel` | [`11-chief-kernel.md`](./11-chief-kernel.md) | draft | L2 services (with swappable harness) |
| `module-system` | [`12-module-system.md`](./12-module-system.md) | draft | Capability Packs, Stacks, registry |
| `memory-substrate` | [`13-memory-substrate.md`](./13-memory-substrate.md) | draft | Graph schema, URIs, horizon |
| `security-model` | [`14-security-model.md`](./14-security-model.md) | draft | Capability-based, hardware-rooted |
| `agent-runtime` | [`15-agent-runtime.md`](./15-agent-runtime.md) | draft | Two-primitive SDK (`.ai()` + `.harness()`), tier routing, engine adapter |
| `harness-resume-protocol` | [`16-harness-resume-protocol.md`](./16-harness-resume-protocol.md) | draft | Ceremony-interrupted harness sessions |
| `os-ceremonies` | [`17-os-ceremonies-and-boundaries.md`](./17-os-ceremonies-and-boundaries.md) | draft | What the OS absorbs (OAuth, pickers, payment, e-sign, devices); threat model |
| `base-and-hardware` | [`18-base-and-hardware.md`](./18-base-and-hardware.md) | draft | NixOS base + hardware targets |

## Surfaces and UI (30-39)

| ID | File | Status | Purpose |
|---|---|---|---|
| `surfaces` | [`30-surfaces.md`](./30-surfaces.md) | draft | 12 surfaces incl. HAX Inbox, Omnibar, Clipboard Pane |
| `ui-standardization` | [`31-ui-standardization.md`](./31-ui-standardization.md) | draft | `chief-ui` primitive catalog, token system |
| `controls-and-policy` | [`32-controls-and-policy.md`](./32-controls-and-policy.md) | draft | Security & Privacy + Controls surfaces |

## Developer platform (40-49)

| ID | File | Status | Purpose |
|---|---|---|---|
| `pack-sdk` | [`40-pack-sdk.md`](./40-pack-sdk.md) | draft | Developer contract — public SDK, CapabilityKind closed enum |
| `local-vs-cloud` | [`41-local-vs-cloud.md`](./41-local-vs-cloud.md) | draft | Hybrid runtime + Model Router swappability |

## Deployment and scope (50-59)

| ID | File | Status | Purpose |
|---|---|---|---|
| `v0-scope` | [`50-v0-scope-90-day.md`](./50-v0-scope-90-day.md) | draft | 90-day MVP scope |
| `risks-open` | [`51-risks-open-questions.md`](./51-risks-open-questions.md) | draft | Honest punch list |

## Go-to-market (60-69)

| ID | File | Status | Purpose |
|---|---|---|---|
| `viral-loop` | [`60-viral-loop.md`](./60-viral-loop.md) | draft | Morning Reveal + try-your-own + Stacks |
| `oss-business` | [`61-open-source-business.md`](./61-open-source-business.md) | draft | Licensing, monetization |
| `regulatory` | [`62-regulatory-posture.md`](./62-regulatory-posture.md) | draft | Legal posture, compliance |

## Decisions (ADRs)

| ID | File | Status |
|---|---|---|
| `adr-readme` | [`../adr/README.md`](../adr/README.md) | stable |
| `adr-template` | [`../adr/template.md`](../adr/template.md) | stable |
| `adr-0001` | [`../adr/0001-nixos-linux-base.md`](../adr/0001-nixos-linux-base.md) | accepted |
| `adr-0002` | [`../adr/0002-capability-based-security.md`](../adr/0002-capability-based-security.md) | accepted |
| `adr-0003` | [`../adr/0003-wayland-not-x11.md`](../adr/0003-wayland-not-x11.md) | accepted |
| `adr-0004` | [`../adr/0004-machine-as-fax-posture.md`](../adr/0004-machine-as-fax-posture.md) | accepted |
| `adr-0005` | [`../adr/0005-signed-typed-event-log.md`](../adr/0005-signed-typed-event-log.md) | accepted |
| `adr-0006` | [`../adr/0006-cas-filesystem.md`](../adr/0006-cas-filesystem.md) | accepted |
| `adr-0007` | [`../adr/0007-hax-inbox-notifications.md`](../adr/0007-hax-inbox-notifications.md) | accepted |
| `adr-0008` | [`../adr/0008-pure-oss-memory-substrate.md`](../adr/0008-pure-oss-memory-substrate.md) | accepted |
| `adr-0009` | [`../adr/0009-signed-inference.md`](../adr/0009-signed-inference.md) | accepted |
| `adr-0010` | [`../adr/0010-sdk-public-api-stability.md`](../adr/0010-sdk-public-api-stability.md) | accepted |
| `adr-0011` | [`../adr/0011-ui-stack-and-component-library.md`](../adr/0011-ui-stack-and-component-library.md) | accepted |
| `adr-0012` | [`../adr/0012-no-settings-app.md`](../adr/0012-no-settings-app.md) | accepted |
| `adr-0013` | [`../adr/0013-agent-runtime-two-tier-llm.md`](../adr/0013-agent-runtime-two-tier-llm.md) | accepted |
| `adr-0014` | [`../adr/0014-opencode-subprocess-boundary.md`](../adr/0014-opencode-subprocess-boundary.md) | accepted |
| `adr-0015` | [`../adr/0015-chief-os-compositor.md`](../adr/0015-chief-os-compositor.md) | accepted |
| `adr-0016` | [`../adr/0016-kernel-principal-identity.md`](../adr/0016-kernel-principal-identity.md) | accepted |

## Diagrams (mermaid)

| File | Embedded in |
|---|---|
| [`diagrams/layers.mmd`](./diagrams/layers.mmd) | `architecture` |
| [`diagrams/kernel-services.mmd`](./diagrams/kernel-services.mmd) | `chief-kernel`, `architecture` |
| [`diagrams/approval-flow.mmd`](./diagrams/approval-flow.mmd) | `security-model`, `hax-principles` |
| [`diagrams/hax-regions.mmd`](./diagrams/hax-regions.mmd) | `hax-principles` |
| [`diagrams/pack-lifecycle.mmd`](./diagrams/pack-lifecycle.mmd) | `module-system`, `security-model` |
| [`diagrams/night-morning-loop.mmd`](./diagrams/night-morning-loop.mmd) | `north-star`, `viral-loop` |
| [`diagrams/memory-schema.mmd`](./diagrams/memory-schema.mmd) | `memory-substrate` |
| [`diagrams/agent-runtime.mmd`](./diagrams/agent-runtime.mmd) | `architecture`, `agent-runtime` |

## Requirements

| Bucket | File |
|---|---|
| Functional | [`requirements/functional.md`](./requirements/functional.md) |
| Non-functional | [`requirements/non-functional.md`](./requirements/non-functional.md) |
| Security | [`requirements/security.md`](./requirements/security.md) |
| Performance | [`requirements/performance.md`](./requirements/performance.md) |

## Ideation

Append-only log of ideas not yet committed to design.

- [`ideation/README.md`](./ideation/README.md)
- [`ideation/2026-04-21-initial-dump.md`](./ideation/2026-04-21-initial-dump.md)

## Research

External references, competitor analyses, prior art.

- [`research/README.md`](./research/README.md)
- [`research/2026-04-21-oss-landscape-scan.md`](./research/2026-04-21-oss-landscape-scan.md)
- [`research/2026-04-21-ai-native-primitive-rethinks.md`](./research/2026-04-21-ai-native-primitive-rethinks.md)
- [`research/2026-04-21-agent-native-fs.md`](./research/2026-04-21-agent-native-fs.md)
- [`research/2026-04-21-pure-oss-memory-substrate.md`](./research/2026-04-21-pure-oss-memory-substrate.md)
- [`research/2026-04-22-firecracker-spike.md`](./research/2026-04-22-firecracker-spike.md)

## Brand and visual

| ID | File | Status | Purpose |
|---|---|---|---|
| `brand-typeface` | [`../brand/typeface.md`](../brand/typeface.md) | draft | Inter + Inter Tight + JetBrains Mono |
| `brand-sound` | [`../brand/sound.md`](../brand/sound.md) | draft | Three-sound palette |
| `brand-motion` | [`../brand/motion.md`](../brand/motion.md) | draft | Spring-physics presets |

## Legal

| ID | File | Status |
|---|---|---|
| `legal-tos` | [`../legal/terms-of-service.md`](../legal/terms-of-service.md) | draft |
| `legal-privacy` | [`../legal/privacy-policy.md`](../legal/privacy-policy.md) | draft |

## Where code lives

- [`../crates/`](../crates/) — production Rust (kernel services, SDK, CLI)
- [`../packs/`](../packs/) — first-party capability packs
- [`../apps/`](../apps/) — user-facing apps (chief-brief-ui)
- [`../spikes/`](../spikes/) — research prototypes
- [`../tools/`](../tools/) — dev / CI tools
- [`../packages/`](../packages/) — TS npm packages
- [`../deploy/docker/`](../deploy/docker/) — Docker demo image
- [`../nix/`](../nix/), [`../flake.nix`](../flake.nix) — NixOS modules + flake

## State

- `../.plandb/state.sql` — PlanDB task graph (authoritative, tracked in git)
- `../.plandb/template.yaml` — human-readable graph sidecar

## Conventions

- Row-order within a section is stable; append at the bottom.
- `status` must match the doc's frontmatter `status`.
- If a doc is renamed, keep an entry with `status: deprecated` and a forward link.
