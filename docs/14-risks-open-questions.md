---
id: risks-open
title: "Risks & Open Questions"
status: draft
owners: [santosh]
last_updated: 2026-04-21
related: [v0-scope, architecture, security-model, regulatory]
depends_on: [v0-scope]
tags: [risks, open-questions, decisions-needed]
---

# Risks & Open Questions

## TL;DR

- Honest punch list. Every risk either has a mitigation, a fallback, or escalation to a decision we need to make explicitly.
- Decisions-needed at bottom — those block progress until resolved.

## Technical risks

| # | Risk | Blast radius | Mitigation | Fallback | Status |
|---|---|---|---|---|---|
| T1 | Boot time > 10s on reference HW | Breaks the demo's opening beat | CI gate; trim systemd units; lazy-init non-critical services | Ship at 15s and document target | open |
| T2 | Local-model (Qwen/Llama Q4) output quality embarrassing vs Claude in demo | Video credibility | Confine local model to triage/summarization; NEVER generation in demo | Cloud-only for demo, local-only as off-screen claim | open |
| T3 | Morning Brief "live render" perceived as fake | Tech Twitter flogs us | Make render a real re-layout over pre-computed cards; publish technical writeup | Re-cut demo with explicit "overnight synthesis, morning re-render" caption | open |
| T4 | bootc-on-NixOS integration cost > budget | Day-90 slip | Parallelize via codex worktree; fall back to Nix generations + OSTree snapshots | Ship without bootc; add at v1 | open |
| T5 | Pack sandbox escape (nspawn not strong enough) | Core security claim broken | eBPF enforcement on net+fs; pen-test before launch | Accelerate Firecracker microVMs to v0 | open |
| T6 | Wayland breaks critical apps (legacy) | User frustration | XWayland in isolated sandbox for legacy packs | Whitelist approved legacy apps only | monitored |
| T7 | Memory Graph scale at 12+ months (millions of nodes) | Retrieval latency | sqlite-vec → LanceDB/Tantivy migration path planned | Aggressive horizon-based pruning | v1 concern |
| T8 | eBPF compatibility across kernel versions | Install failures | Pin kernel via Nix flake; test matrix | Skip eBPF enforcement on non-matching kernels, block install | open |
| T9 | MCP spec churn during v0 build | Integration rework | Pin to dated spec revision; maintain transport shim | Revert to Streamable-HTTP mainline | monitored |
| T10 | Sigstore Rekor v1 → v2 migration lands mid-build | Provenance regressions | Abstract client; test both v1 and v2 | Local-only mirror if public Rekor unstable | monitored |

## Product risks

| # | Risk | Blast radius | Mitigation | Fallback |
|---|---|---|---|---|
| P1 | OS install friction too high for Person Zero | Low conversion | VM image + bootable USB paths; try-your-own web preview as first funnel | Push cloud-rendered preview as the primary acquisition for 6 months |
| P2 | Cold-start: first week feels empty (no memory graph yet) | High churn day-7 | Pre-bootstrap with Gmail + Calendar backfill on first run | Guided "first 48h" tour that triggers meaningful actions |
| P3 | Trust Ledger growth feels gamified / hollow | Narrative fails | Ground growth in real approvals; visible "unlocked" moments tied to actual capability | Hide the number; surface unlocks as prose moments only |
| P4 | Hero demo video perceived as staged | Viral loop fizzles | Shoot on reference HW with no post-production compositing; release raw footage with audit | Redo demo on a subscriber's machine post-launch |
| P5 | Stacks don't emerge; community doesn't fork | Secondary viral loop dies | Seed with 5 influencer Stacks on day 90 | Accept that Stacks are a v1 bet; lean on Morning Reveal alone for v0 |

## Supply-chain risks (packs)

| # | Risk | Mitigation |
|---|---|---|
| S1 | Malicious pack with over-broad grant accepted by rushed user | Capability display mandatory + per-kind tap-to-approve + trust score + official tier |
| S2 | Dependency confusion attack on pack registry | Signed packs + pinned hashes + reproducible Nix builds |
| S3 | Pack exfiltrates via a granted capability (e.g., uses granted `gmail.send` for phishing) | eBPF anomaly detection; ceremony for ship > N recipients; red-team audit of official packs |
| S4 | Pack updates silently escalate grants | Version bump requires re-consent if grants differ |

## Regulatory / legal risks

| # | Risk | Mitigation |
|---|---|---|
| R1 | Jurisdiction holds Chief OS liable when agent sends a message | "Machine as fax" posture — human cryptographically signs; OS is a drafting tool (see [`regulatory`](./15-regulatory-posture.md)) |
| R2 | EU DMA / AI Act imposes unexpected obligations on "AI system" | Consult counsel; publish transparency reports; design for auditability (we're already there) |
| R3 | Apple / Google sandbox restrictions limit browser pack capabilities on companion devices | Desktop is the anchor; companion devices are read-only mirrors (v1) |
| R4 | SEC / FINRA classify agent-assisted trading as advisory | Block trading/advice packs from official tier; community packs carry disclaimers |

## Business risks

| # | Risk | Mitigation |
|---|---|---|
| B1 | COGS at scale exceeds $40/user | Optimize retrieval (80%+ of cost is model; retrieval tuning cuts tokens 50%+); reserved capacity with providers; Haiku-tier for classification |
| B2 | OpenAI / Anthropic ship "Operator-like" product in next 6 mo | Our moat is the OS trust floor + local inference — they cannot match without their own OS; accelerate the "impossible in a webapp" messaging |
| B3 | NixOS community friction over us shipping a "non-standard" distro | Upstream improvements; respect the brand; position as "Chief OS, built on NixOS" not a fork |
| B4 | GTM stalls: Person Zero doesn't evangelize | Backup channel: paid partnerships with 5 creators to publish their Stacks on launch |

## Open questions

### Architecture

1. Does the cloud twin get its own kernel-service process or reuse local broker via remote proxy?
2. Is the Event Bus an in-proc pub/sub or NATS/Redpanda even in v0? (Bias: in-proc.)
3. Does the Provenance Log mirror to public Rekor, a self-hosted Rekor, or both?
4. Do we support dual-boot at v0?

### Product

5. Default trust-ledger N thresholds per category? (How many approvals → level-up.)
6. Delegation decay: if a category is unused for 30 days, does the level drop?
7. Cross-category coupling: should `finance` require `calendar ≥ 3/5` as a "presence-of-life" signal?
8. Live Agent View in v0 or v1?
9. Voice input in v0 or v1?
10. Companion phone app: native iOS/Android, or web-only mirror?

### Design

11. Typeface: custom commission, licensed (Pitch/Söhne/Inter-adjacent), or open-source (Inter, JetBrains Mono for mono)?
12. Default sound palette: who composes it? (Licensing matters.)
13. Live Agent View visual style: cosmic / mission-control / terminal / minimalist? (Pick one by day 30.)

### Supply chain

14. Official pack tier: insured by whom (first-party, third-party policy, self)?
15. Who runs the pack registry operationally at day 90? (Same team or outsourced?)

### Legal

16. Which jurisdictions do we explicitly serve at v0? (Probably US + EU; do we block others?)
17. Do we register an "AI system" designation in EU AI Act if that category applies?

## Decisions needed BEFORE v0 build starts

| # | Decision | Owner | Deadline |
|---|---|---|---|
| D1 | Pick agent harness SDK version (opencode X.Y.Z pinned) + ADR | Kernel eng | Day 5 |
| D2 | Confirm pure-OSS memory stack (sqlite-vec + fastembed-rs + SQLite + blake3 CAS) after follow-up research returns | Agent eng | Day 7 |
| D3 | Pick MCP transport shim strategy: custom vs. wait-for-spec | Kernel eng | Day 10 |
| D4 | Pick bootc-on-NixOS vs. Nix-generations-only for v0 | Infra eng + founder | Day 10 |
| D5 | Pick Sigstore deployment: public Rekor vs self-hosted vs both | Security eng | Day 12 |
| D6 | Pick typeface and sound partner | Design + founder | Day 14 |
| D7 | Pick live-render implementation strategy for Morning Brief | Surfaces eng | Day 14 |
| D8 | Pick Person Zero target count for day-90 private beta | Founder | Day 5 |

## Related

- [`v0-scope`](./13-v0-scope-90-day.md) — what we're trying to deliver
- [`security-model`](./06-security-model.md) — threat model context
- [`regulatory`](./15-regulatory-posture.md) — legal posture
- [`research/2026-04-21-oss-landscape-scan.md`](./research/2026-04-21-oss-landscape-scan.md) — OSS options evaluated
