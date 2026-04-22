---
id: oss-business
title: "Open Source & Business Model"
status: draft
owners: [santosh]
last_updated: 2026-04-21
related: [north-star, module-system, v0-scope]
tags: [gtm, licensing, monetization, business]
---

# Open Source & Business Model

## TL;DR

- Kernel + surfaces: **AGPL-3.0-or-later**. Forces competitor forks open; locks in community contributions.
- First-party Capability Packs: **Apache-2.0**. Frictionless reuse drives ecosystem adoption.
- Community packs: author's choice (we publish recommended templates).
- Monetize: **managed cloud twin + pack registry + support**, not the software. Supabase / GitLab / Linux playbook.
- Revenue: $49/mo entry (Chief Plus), $149/mo pro (Chief Studio), enterprise (v3+).

## Licensing map

| Component | License | Why |
|---|---|---|
| Kernel (L2 services, Rust/Go) | AGPL-3.0-or-later | Network-service clause: SaaSified forks must open. Competitor-resistant. |
| Surfaces (L4, TypeScript) | AGPL-3.0-or-later | Keeps the brand experience open but same-pond. |
| Module SDK | Apache-2.0 | Pack authors must freely build commercial packs without AGPL contamination. |
| First-party Capability Packs | Apache-2.0 | Reference quality; adoption > protection at this layer. |
| Community Capability Packs | Author's choice | Documented recommendations; no enforcement. |
| Cloud twin orchestrator (server-side) | AGPL-3.0-or-later | Ensures hosted forks upstream. |
| Legal / regulatory templates (ADRs, policies) | CC-BY-4.0 | Ideas travel; contributions encouraged. |
| Brand (wordmark, logos) | Proprietary | Trademark protection needed. |

AGPL concerns (FAQ):
- **"AGPL scares startups away."** Not for Chief OS adopters (prosumers). For pack authors, Apache-licensed SDK is the touchpoint — they never interact with AGPL. For cloud competitors, that's the point.
- **"AGPL prevents integrations."** False. It affects *modifying* Chief, not *calling* Chief. Integrators use the Apache SDK + the HTTP API.

## Monetization (layered)

### Layer 1: Chief Plus — $49 / user / month

- Managed cloud twin, capped context, managed Rekor-compatible provenance mirror.
- Pack registry browsing + private-pack hosting (limited).
- Standard local-model tier (Qwen 32B).
- Single-user, one device primary + phone/watch mirror.

### Layer 2: Chief Studio — $149 / user / month

- Larger cloud context; priority routing; access to higher-tier models (Claude Sonnet / Opus, GPT top-tier).
- Local model tier: 70B-class with GPU.
- Unlimited private packs; signed-pack distribution tooling.
- Stack hosting + analytics for creators.
- Priority ceremony rituals (e.g., multi-device co-sign, custom friction tiers).

### Layer 3: Chief for Teams (v2+) — per-seat, TBD

- Multi-human delegation within a household or small team.
- Shared Stacks with per-seat Trust Ledgers.
- Co-sign workflows across principals.
- Team registries for private packs.

### Layer 4: Chief Enterprise (v3+)

- Self-hosted cloud twin or private cloud (AWS/GCP/Azure).
- SSO, SCIM, audit exports.
- Policy compliance (HIPAA, SOC 2, ISO 27001 — certification tracked).
- On-prem pack registry with custom tiers.

## Free tier

**Chief Free (no cloud twin):** install Chief OS, run fully local, all first-party packs, limited to local model inference only. Unlimited use, no time-bomb. Trust Ledger works; Provenance Log local-only (no Rekor mirror).

Rationale: the *OS itself* must not be gated by a subscription. Subscription gates cloud compute, not agency. This preserves the open-source ethos and defeats "Chief is just a paid SaaS in trenchcoat."

## Unit economics (v1 target)

| Item | Monthly | Notes |
|---|---|---|
| LLM API cost (Chief Plus P0) | $18 | Haiku-tier retrieval + Claude Sonnet for synthesis |
| LLM API cost (Chief Studio P0) | $48 | Bigger context, heavier models |
| Cloud twin hosting | $3 | Stateless microVMs, autoscale |
| Provenance mirror ops | $1 | Rekor-compatible, low-write load |
| Pack registry | $1 | CDN + lightweight backend |
| Support cost (amortized) | $3 | Self-serve + docs-first |
| **Gross margin (Plus)** | **~50%** | $49 → $24 contribution |
| **Gross margin (Studio)** | **~63%** | $149 → $94 contribution |

Tuning: prompt cache + retrieval compression takes Plus margin toward 60%+ at scale.

## Registry economics

Pack registry model:

| Pack tier | Cost to author | Cost to user | Revenue share to author |
|---|---|---|---|
| Free (permissive license) | $0 | $0 | N/A |
| Paid (community) | Free to list | Per-pack subscription (author sets) | 70% to author, 30% to Chief |
| Paid (official / certified) | Chief review fee | Premium tier | 50/50 split reflecting Chief's liability insurance |
| Enterprise licensed | Negotiated | Enterprise contract | Custom |

Launch with free-only; paid tier goes live at v1 once we have 100+ active packs.

## Community stewardship

- **Maintainer program:** core contributors earn credits toward Studio tier and priority review on their packs.
- **SDK mentorship:** office hours, pack-authoring workshops.
- **Bounty board:** for specific ADR-accepted extensions (new capability kinds, new backends, new surfaces).
- **Code of conduct:** CC-BY-4.0 modified; enforcement via steward + delegated committee at 100+ contributors.

## Competition & moat

| Competitor | What they do | Where Chief OS wins |
|---|---|---|
| ChatGPT / Claude Desktop / Gemini apps | Chat + light tool use | OS-level trust floor, provenance, 24/7 agents, local inference |
| Lindy / Zapier Agents / n8n | SaaS automation | Same data-gravity + no SaaS key custody |
| Windows Copilot / macOS Shortcuts AI | OS-integrated assistants | Not AI-native OS; legacy OS paradigms; less provenance |
| OpenAI Operator / Anthropic Computer Use | Browser bots | Browser only; Chief has OS-wide scope + hardware trust |
| Hugging Face / Ollama local AI | Local model runners | Not an OS, not an agent runtime, no Trust Ledger |

Moat compounds with:
1. Months of accumulated Trust Ledger + Memory Graph (switching cost = restart from 0/5).
2. Public Stack ecosystem with forkable community templates.
3. Provenance receipts accumulating legal + regulatory weight.
4. Hardware trust root making re-issue on a new device require explicit migration ceremony.

## Governance

v0–v1: benevolent stewardship under founder. All ADRs reviewed by steward.

v2+: move to a lightweight foundation (Apache-style) with weighted voting rights for major contributors + first-tier community reviewers.

v3+: optional — if community growth warrants — spin out a Chief Foundation with seats from key enterprise adopters + maintainers.

## Acceptance

- [ ] LICENSE files accurate for every component (AGPL kernel, Apache SDK/packs).
- [ ] Chief Free installs without any subscription gating at v0.
- [ ] Cloud-twin-optional architecture verified (user can opt out of Layer 1+ and still have Chief).
- [ ] Revenue-share calculator for community paid packs defined before v1 marketplace.
- [ ] Trademark registration (wordmark + logo) filed in US + EU before launch.

## Open questions

1. Do we open-source the *demo brand assets* (typeface, sound, visuals) or keep proprietary?
2. Is Chief Free gated by local hardware quality (we can't support users with 8GB RAM well)?
3. Do enterprise customers get their own signed-pack tier or reuse official tier with policy overlays?
4. What's the minimum viable SOC 2 scope for Chief Plus/Studio at 10k+ users?
5. Do we accept crypto payment, or keep fiat-only at v1?

## Related

- [`north-star`](./00-north-star.md) — staged vision
- [`module-system`](./12-module-system.md) — pack registry + Stacks
- [`v0-scope`](./50-v0-scope-90-day.md) — what ships first
- [`regulatory`](./62-regulatory-posture.md) — compliance implications
