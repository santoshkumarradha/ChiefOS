---
id: docs-index
title: "Docs Index"
status: stable
owners: [santosh]
last_updated: 2026-04-21
tags: [meta, navigation]
---

# Docs Index

Machine-readable map of all Chief OS documentation. Update when adding or renaming a doc.

## Core design

| ID | File | Status | Purpose |
|---|---|---|---|
| `charter` | [`../CHARTER.md`](../CHARTER.md) | stable | 10 axioms; constitution |
| `agents-md` | [`../AGENTS.md`](../AGENTS.md) | stable | Entry point for AI agents |
| `north-star` | [`00-north-star.md`](./00-north-star.md) | draft | Mission, P0, wedge, staged vision |
| `hax-principles` | [`01-hax-principles.md`](./01-hax-principles.md) | draft | HAX theory → OS primitives |
| `architecture` | [`02-architecture.md`](./02-architecture.md) | draft | L0–L4 layers, substrate spine, language policy |
| `chief-kernel` | [`03-chief-kernel.md`](./03-chief-kernel.md) | draft | L2 services (7) with swappable harness |
| `module-system` | [`04-module-system.md`](./04-module-system.md) | draft | Capability Packs, Stacks, registry |
| `surfaces` | [`05-surfaces.md`](./05-surfaces.md) | draft | 10 surfaces incl. HAX Inbox + Omnibar + Clipboard Pane |
| `security-model` | [`06-security-model.md`](./06-security-model.md) | draft | Cap-based, hardware-rooted |
| `base-and-hardware` | [`07-base-and-hardware.md`](./07-base-and-hardware.md) | draft | NixOS choice + targets |
| `memory-substrate` | [`08-memory-substrate.md`](./08-memory-substrate.md) | draft | Graph schema, URIs, horizon |
| `local-vs-cloud` | [`09-local-vs-cloud.md`](./09-local-vs-cloud.md) | draft | Hybrid runtime + Model Router swappability |
| `viral-loop` | [`10-viral-loop.md`](./10-viral-loop.md) | draft | Morning Reveal + try-your-own + Stacks |
| `oss-business` | [`11-open-source-business.md`](./11-open-source-business.md) | draft | Licensing, monetization |
| `apple-design` | [`12-apple-design-principles.md`](./12-apple-design-principles.md) | draft | Product guardrails |
| `v0-scope` | [`13-v0-scope-90-day.md`](./13-v0-scope-90-day.md) | draft | 90-day MVP incl. substrate spine |
| `risks-open` | [`14-risks-open-questions.md`](./14-risks-open-questions.md) | draft | Honest punch list |
| `regulatory` | [`15-regulatory-posture.md`](./15-regulatory-posture.md) | draft | Legal posture, compliance |
| `pack-sdk` | [`16-pack-sdk.md`](./16-pack-sdk.md) | draft | Developer contract — public SDK, CapabilityKind closed enum, CAN/CANNOT, dogfood sequence |
| `os-ceremonies` | [`17-os-ceremonies-and-boundaries.md`](./17-os-ceremonies-and-boundaries.md) | draft | What the OS absorbs (OAuth, pickers, payment, e-sign, devices); threat model; anti-phishing |
| `ui-standardization` | [`18-ui-standardization.md`](./18-ui-standardization.md) | draft | `chief-ui` primitive catalog, token system, pack enforcement layers |
| `controls-and-policy` | [`19-controls-and-policy.md`](./19-controls-and-policy.md) | draft | Security & Privacy surface + Controls surface (implementation spec) |
| `agent-runtime` | [`20-agent-runtime.md`](./20-agent-runtime.md) | draft | Two-primitive SDK (`.ai()` + `.harness()`), tier-based routing, engine adapter, harness runtime spec |

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

## Reusable diagrams (mermaid)

| ID | File | Embedded in |
|---|---|---|
| `diag-layers` | [`diagrams/layers.mmd`](./diagrams/layers.mmd) | `architecture` |
| `diag-kernel-services` | [`diagrams/kernel-services.mmd`](./diagrams/kernel-services.mmd) | `chief-kernel`, `architecture` |
| `diag-approval-flow` | [`diagrams/approval-flow.mmd`](./diagrams/approval-flow.mmd) | `security-model`, `hax-principles` |
| `diag-hax-regions` | [`diagrams/hax-regions.mmd`](./diagrams/hax-regions.mmd) | `hax-principles` |
| `diag-pack-lifecycle` | [`diagrams/pack-lifecycle.mmd`](./diagrams/pack-lifecycle.mmd) | `module-system`, `security-model` |
| `diag-night-morning` | [`diagrams/night-morning-loop.mmd`](./diagrams/night-morning-loop.mmd) | `north-star`, `viral-loop` |
| `diag-memory-schema` | [`diagrams/memory-schema.mmd`](./diagrams/memory-schema.mmd) | `memory-substrate` |
| `diag-agent-runtime` | [`diagrams/agent-runtime.mmd`](./diagrams/agent-runtime.mmd) | `architecture`, `agent-runtime` |

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

## Brand & visual

| ID | File | Status | Purpose |
|---|---|---|---|
| `brand-visual-language` | [`brand/visual-language.md`](./brand/visual-language.md) | draft (binding) | Desktop-feel rules: Tauri rendering, typography, color, motion, native integration checklist, anti-patterns. Review-blocker for any UI PR. |
| `brand-typeface` | [`../brand/typeface.md`](../brand/typeface.md) | draft | Inter + Inter Tight + JetBrains Mono picks, bundling rules, v2 upgrade candidates |
| `brand-sound` | [`../brand/sound.md`](../brand/sound.md) | draft | Three-sound palette: ship chime, approval cluck, ceremony tone |
| `brand-motion` | [`../brand/motion.md`](../brand/motion.md) | draft | Seven spring-physics presets; banned motion list |

## Legal (draft, counsel review pending)

| ID | File | Status |
|---|---|---|
| `legal-tos` | [`../legal/terms-of-service.md`](../legal/terms-of-service.md) | draft |
| `legal-privacy` | [`../legal/privacy-policy.md`](../legal/privacy-policy.md) | draft |

## Directories

- [`../plans/`](../plans/) — implementation plans (see `plans/README.md`)
- [`../prototypes/`](../prototypes/) — throwaway code spikes
- [`../brand/`](../brand/) — brand identity
- [`../legal/`](../legal/) — draft legal documents (external counsel review required)

## Other

- `../.plandb.db` — PlanDB task graph (intentionally tracked in git)

## Conventions for this index

- Row-order within a section is stable; append new entries at the bottom of the relevant table.
- `status` must match the doc's frontmatter `status`.
- If a doc is renamed, keep an entry with `status: deprecated` and a link forward.
