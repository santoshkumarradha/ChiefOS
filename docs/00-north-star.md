---
id: north-star
title: "North Star"
status: draft
owners: [santosh]
last_updated: 2026-05-17
related: [charter, v0-scope, viral-loop]
tags: [mission, product, persona]
---

# North Star

## TL;DR

- AI-native OS; agents are first-class workers, humans steer and approve.
- Wedge: one **Work Object** that follows the user all day, with Morning Brief as the daily ritual.
- P0: high-agency prosumer-founders (3–5M globally).
- Goal: let users delegate cross-app work without becoming managers of agent tools.

## The loop

```mermaid
sequenceDiagram
    participant H as Human
    participant C as Chief OS
    participant Ag as Agents (overnight)
    participant L as Trust Ledger

    H->>C: Create or update Work Object
    C->>Ag: Dispatch packs with scoped capabilities
    Ag->>C: Write contributions to Memory Graph
    C->>L: Classify authority state
    H->>C: Work Object View / Brief / Inbox / Ceremony
    C->>L: Update delegation and replayable state
```

## What users uniquely get

Chief OS should be judged by enabled work, not by proof artifacts alone.

| User capability | What it feels like | Why Chief OS is different |
|---|---|---|
| Delegate one outcome across many tools | "Prepare the Acme follow-up" becomes obligations, slots, risk checks, and a draft without app-hopping | Packs compose through one OS-owned Work Object instead of point integrations |
| Keep working while agents keep updating the object | The same object stays live through the day: sources, drafts, risk, approvals, rewind | Surfaces are projections over L2 state; the Work Object is not trapped in a chat or app |
| Add a new capability midstream | Install `risk-pack`; it contributes to existing work without changing document/calendar/email packs | New packs integrate with the OS substrate, not with every other pack |
| Let low-stakes work proceed while high-stakes work stops | Drafting happens automatically; sending waits for Ceremony | Authority is OS-owned and consistent across packs |
| Ask "what changed?" and undo it | Rewind a draft contribution while preserving the history and marking downstream work stale | Rollback is a platform primitive, not a per-app undo button |

The product promise:

> Chief OS turns scattered agent actions into one inspectable, steerable, reversible unit of work.

## Person Zero

**High-agency prosumer-founder.**

| Attribute | Signal |
|---|---|
| Already pays | $200–500/mo across AI + productivity SaaS |
| Tool count | 12+ open tabs, 8 idle dashboards |
| Identity | "Not a manager of my tools" |
| Reads | Every.to, Tiago Forte / David Perell, PG essays |
| Early-adopter index | Rabbit, Humane, Perplexity, Arc, Granola buyer |
| Willingness-to-pay | $49 entry / $149 pro |
| Market size | 3–5M globally |
| LTV horizon | 36+ mo once ledger matures (90d+) |

**Not-P0:** enterprise (v3+), creators (Arc/Notion crowding), mainstream non-technical (trust vocab gap), every-knowledge-worker (too generic).

## Why now

| Forcing function | Evidence |
|---|---|
| Tool-use reliability crossed shipping floor | 2025 harness benchmarks > 90% on bounded tasks |
| 24/7 agent economics feasible | Haiku-tier + local Qwen enable $15–40/mo COGS at scale |
| Category demand proven, no trust floor yet | Rabbit / Humane failed precisely on trust + OS-absence |

## Staged vision

| Stage | Timeline | Scope | Primary metric |
|---|---|---|---|
| v0 — Night Loop | 90 days | Single-human, hardcoded Chief-of-Staff Stack, no money/legal | Hero video views × waitlist signups |
| v1 — Trust Graduates | +6 mo | Public pack API, auto-ship low-stakes, multi-device, renegotiation ritual | Day-30 retention |
| v2 — Money & Legal | +12 mo | Ceremony UI, e-sign, wire rails, federation (Chief-to-Chief) | Autonomous-ship rate |
| v3 — Branded Device | +24 mo | Reference hardware, enterprise-ready trust ledger | Device sell-through + enterprise pilots |

## Success criteria

| Stage | Primary | Secondary | Kill signal |
|---|---|---|---|
| v0 | Hero views × waitlist | Day-7 retention | Install completion < 15% |
| v1 | Day-30 retention | # third-party packs | Ledger growth stalls for > 60% users |
| v2 | Autonomous-ship rate | Federation pairs | Regulatory rollback of posture |
| v3 | Device sell-through | Enterprise pilots | Device flop + weak enterprise demand |

## Explicit non-goals

- Chat-as-primary surface → ChatGPT owns this.
- Coding agent → Cursor owns this.
- Browser → Arc owns this.
- Creator tools → Notion/Granola/Descript own this.
- Branded hardware at v0.
- Enterprise at v0.
- Mobile OS fork — phone is a surface, not a platform.
- On-device training (consumption only at v0).

## Competitive line

| Incumbent | Their territory | Our territory |
|---|---|---|
| ChatGPT / Claude Desktop | Chat | **The night (16 hrs you're not at desk)** |
| Lindy / Zapier Agents | SaaS automation | OS-level data gravity + trust floor |
| Cursor | Coding | Everything non-coding |
| Arc | Browser | Desktop / post-browser |
| Superhuman | Inbox only | Inbox + cal + files + approvals, unified |
| OpenAI Operator | Browser-bot | Local trust floor + provenance + rollback |
| Rabbit / Humane | Hardware gimmicks | Software substrate they lacked |

## North-star metric

**Approved actions per user per week, conditional on Trust-Ledger average ≥ 3/5 across the Chief-of-Staff Stack.**

Captures delegation volume, trust depth, and retention in one number.

## Related

- [`v0-scope`](./50-v0-scope-90-day.md) — concrete MVP
- [`viral-loop`](./60-viral-loop.md) — Morning Reveal + Stack flywheel
- [`hax-principles`](./01-hax-principles.md) — how HAX shapes the loop
