---
id: ideation-initial-dump
title: "Initial Ideation Dump"
status: open
owners: [santosh]
last_updated: 2026-04-21
tags: [ideation, dump, future]
---

# Initial Ideation Dump — 2026-04-21

## TL;DR

Stray ideas surfaced during the first brainstorm session. Not yet commitments. Some are promising demos; some are v2+; some may be archived.

## Hardware / physical

- **Branded approval puck** (physical USB/BLE) — held 3s for ceremony, signed by SE on-device. Considered and deferred for v0 (software-only demo chosen). Revisit as v2+ accessory — same shape as AirPods identity-object.
- **Desk-pane ambient display** — always-on e-ink or OLED showing Morning Brief summary. Reference hw at v3.
- **Keyboard shortcut mechanical device** — a single physical "Night Handoff" button. Probably v3 gimmick.

## Surfaces / UX

- **Live Agent View aesthetic options** — cosmic / mission-control / terminal / minimalist. Pick by day 30.
- **Receipt art / provenance-as-object** — signed morning brief as a downloadable aesthetic PNG / NFT-adjacent collectible that users post. Considered and parked as marketing-v1.
- **Chief avatar** — a tiny animated agent face in the corner that breathes. Considered; risk: feels toy-like. Deferred.
- **Ambient soundtrack** — Brian-Eno-like generative audio at night while agents run. v2 flex feature.

## Product features

- **Federation — Chief ↔ Chief** — two users' Chiefs negotiate scheduling, intros, context-sharing. v2.
- **Digital afterlife** — user's Chief persists as a read-only knowledge archive for heirs. v3+ moral question.
- **Chief pairs** — spouses/cofounders share a partial trust ledger. v2.
- **Kid Chief** — a simplified Chief for a child: approvals go to a parent. v3.
- **Chief for Elder Care** — family delegates to help an aging parent. Sensitive; v3 with legal.

## Monetization

- **Pack bounties** — users pay for specific pack requests; bounty board. v1.
- **Signed Stacks as paid products** — creators charge for a curated stack (70/30 split). v1.
- **Chief Studio as a creator platform** — revenue share on published stacks. v1.
- **On-device LoRA fine-tune on your voice** — personalize local model. v3 research.

## Technical moonshots

- **Chief-to-Chief negotiation protocol** — formalize A2A for scheduling and business interactions with counterfoil mechanics. v2.
- **On-device LLM embeddings-only mode** — for users in locked-down environments, run entirely without any remote call. Investigate feasibility.
- **Formal verification of Capability Broker** — TLA+ or Lean spec. Research track, v2+.
- **Federated Sigstore** — Chief-operated public transparency log. If Rekor migration causes pain.

## Ecosystem

- **Chief Academy** — free online course for pack authors. Post-launch.
- **Chief Scholars** — academic access program with research grants. v2.
- **Chief for Education** — teacher dashboards, student Chiefs with guardrails. v3.
- **Open-source governance foundation** — Apache-style if community grows past 100 active maintainers.

## Wild ideas (low-confidence, capture anyway)

- **"Dreaming" — overnight agents run idle-creativity passes** — a 15-minute slot where agents "explore" the memory graph and surface unexpected connections as part of the Morning Brief. Zero instruction, pure serendipity.
- **"Past-self" agent** — an agent persona that roleplays your values from 6 months ago, used during Quarterly Review to check against drift.
- **"Advocate" agent** — a persistent agent whose only job is to argue *against* the user's current plan when drafting a decision. Anti-echo-chamber.
- **"Public Memory Graph lanes"** — certain horizons explicitly shared to a public portfolio (writings, research, code). Discoverable by others' Chiefs.

## User directives captured (for directive-audit pass)

- **Boot-time is not a v0 requirement** — captured in [`../requirements/non-functional.md`](../requirements/non-functional.md) and [`../14-risks-open-questions.md`](../14-risks-open-questions.md).
- **Polyglot language policy** — captured in [`../02-architecture.md`](../02-architecture.md) and NFR.
- **Model routing must be swappable at multiple granularities** — captured in [`../09-local-vs-cloud.md`](../09-local-vs-cloud.md).
- **Rethink all OS primitives for agent-native usage, not just filesystem** — research completed; substrate spine identified; incorporated into architecture + v0 scope + 3 new ADRs.

## Related

- [`../research/2026-04-21-oss-landscape-scan.md`](../research/2026-04-21-oss-landscape-scan.md)
- [`../research/2026-04-21-ai-native-primitive-rethinks.md`](../research/2026-04-21-ai-native-primitive-rethinks.md)
- [`../research/2026-04-21-agent-native-fs.md`](../research/2026-04-21-agent-native-fs.md)
- [`../14-risks-open-questions.md`](../14-risks-open-questions.md)
