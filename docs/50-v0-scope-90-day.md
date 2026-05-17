---
id: v0-scope
title: "v0 — 90-Day MVP Scope"
status: draft
owners: [santosh]
last_updated: 2026-05-17
related: [north-star, chief-kernel, security-model, base-and-hardware, viral-loop]
depends_on: [chief-kernel, security-model, base-and-hardware]
tags: [scope, mvp, milestones]
---

# v0 Scope — 90-Day MVP

## TL;DR

- Ship one coherent loop: **Work Object → pack contributions → authority gate → rewind**, with Morning Brief as one ritual surface.
- Single human, single laptop, deterministic Chief-of-Staff Stack proof.
- All 7 OS-only capabilities present; quality over breadth.
- 4 external OSS integrations accelerate 6+ months of eng: **Sigstore/Rekor, opencode, MCP-over-vsock, bootc/Lanzaboote, Agent-Sandbox CRD schema.** (See [`research`](./research/2026-04-21-oss-landscape-scan.md).)
- Demo-ready: a viewer can see one user goal become a multi-pack Work Object that stays inspectable through HTTP, CLI, and UI.

## What ships

### Core loop

1. **Work Object creation** — one human outcome becomes `mem://artifact/...` state.
2. **Pack contribution** — independent packs add obligations, slots, risks, and drafts through `chief-sdk`.
3. **All-day surface** — Work Object View shows current state; Morning Brief summarizes ritual checkpoints.
4. **Approval flow** — external authority moves through Ceremony, gated by Broker/Region Router.
5. **Rewind** — contribution-level rewind removes active state while preserving replay history.

### Platform MVP POC

The phase-zero demo now exists as the deterministic Docker path:

```bash
docker compose -f deploy/docker/docker-compose.yml up --build
curl -fsS http://localhost:8080/v1/work/acme-follow-up
```

It proves the v0 shape with fixture-backed packs:

| Pack | Contribution |
|---|---|
| `document-pack` | Contract obligations |
| `calendar-pack` | Candidate follow-up slots |
| `email-pack` | Draft reply requiring Ceremony before send |
| `risk-pack` | Payment/compliance risk after install |

This POC is intentionally narrower than the full 90-day scope, but it demonstrates the core OS claim: packs compose through Chief OS-owned memory, authority, provenance, and surfaces.

### 7 OS-only capabilities (non-negotiable)

| # | Capability | v0 implementation |
|---|---|---|
| 1 | Hardware-bound approval root | TPM/SE + YubiKey optional; ceremony token single-use |
| 2 | Local filesystem gravity | Chief FS: CAS over blake3 + FUSE legacy shim at `~/`; agents cite `mem://file/<cid>` |
| 3 | Local-inference fallback | **Qwen/Llama Q4 7B–13B, CPU-only for v0** (GPU deferred to v1+); used for sensitive categories |
| 4 | OS-level rewind | NixOS generations + Memory Graph tombstones + agent-queue journal |
| 5 | Capability-sandboxed agents | `systemd-nspawn` per agent + eBPF enforcement on net/fs |
| 6 | OS-owned Chromium | Single instance, cookie store owned by OS, agents drive via CDP |
| 7 | Provenance graph | Sigstore-compatible: in-toto v1 statements + Rekor-style local log + cosign-signed receipts |
| 8 | **Signed Inference** | `chief-inference` mediates every model call; Ed25519 attestation per call; tier labels (Generated / Co-signed / Custody). ADR-0009. |

### 5 substrate-spine primitives (non-negotiable, from research)

See [`research/2026-04-21-ai-native-primitive-rethinks.md`](./research/2026-04-21-ai-native-primitive-rethinks.md). The 5 must be present in v0 even if basic:

| # | Primitive | v0 scope |
|---|---|---|
| 1 | Signed Typed Event Log | Append-only, blake3-addressed, Sigstore-signed. RocksDB/fjall backing. Mounted as `/chief/events/` FUSE. |
| 2 | Chief FS (CAS + FUSE shim) | blake3 CAS + property-graph index + FUSE legacy view at `~/`. Capture daemon intercepts writes. |
| 3 | `kbd://` Clipboard | v0 scaffold. Typed clipboard with one auto-transform (PDF → summary). Full agent subscribers in v1. |
| 4 | HAX Inbox | Replaces all popup notifications. Queue of typed approvals with 4 affordances (approve/deny/delegate/defer). |
| 5 | Omnibar | Semantic search over Memory Graph. Global hotkey. Hybrid BM25 + vector + graph-walk. |

### Kernel services (minimum viable)

| Service | v0 completeness |
|---|---|
| Agent Runtime | opencode as harness (swappable interface), systemd-nspawn isolation, SQLite journal |
| Capability Broker | Typed grants, eBPF-enforced on high-risk caps; **MCP server at kernel boundary, vsock transport** |
| Memory Graph | Node/edge schema, `mem://` URIs, **pure-OSS stack: SQLite + sqlite-vec + fastembed-rs + content-addressed blobs.** No single-vendor dependency at the kernel layer (research in flight). |
| Provenance Log | Append-only, Merkle-chained, **in-toto v1 statements + Rekor-compatible mirror** |
| Trust Ledger | Per-category 1–5, auditable write rules, no LLM writes |
| Region Router | Rule-table classifier, 100% unit test coverage |
| Event Bus | In-proc pub/sub (v0); NATS deferred |

### Surfaces (minimum viable)

| Surface | v0 |
|---|---|
| Morning Brief | Yes, polished |
| Live Agent View | Yes, **hero-demo-ready** |
| Provenance Explorer | Yes, with Sigstore verification UI |
| Ceremony | Yes, 3-second hold + biometric |
| Chat Pane | Yes (voice deferred to v1) |
| Trust Ledger Viewer | Yes (read + growth animation) |
| Quarterly Review | Scaffold only; first real run is v1 at 90-day mark |

### First-party Chief-of-Staff Stack (hardcoded at v0)

| Pack | Purpose |
|---|---|
| `inbox-triage` | Gmail OAuth read-only; classify + propose drafts |
| `calendar-negotiator` | Google Calendar OAuth; move meetings, propose slots |
| `daily-brief` | Aggregate: compose the Morning Brief from pack outputs |
| `finance-watcher` | Local-inference only; watches numbers in files, never sends |
| `fs-ingester` | Watches `~/Downloads/`, `~/Desktop/`, `~/ops/` |
| `browser-navigator` | Drives the OS-owned Chromium via CDP |

## OSS integrations (v0 accelerators)

From [`research/2026-04-21-oss-landscape-scan.md`](./research/2026-04-21-oss-landscape-scan.md):

| Integration | Role in v0 | Risk |
|---|---|---|
| **Sigstore (cosign + Rekor + in-toto v1)** | Provenance Log emits in-toto statements; Rekor-style local mirror; cosign signs every ship and every pack. Axiom 3 made real, standards-aligned, court-admissible. | Rekor v1→v2 transition in flight; pin client abstraction. |
| **opencode** | Primary agent harness behind swappable interface. | MIT license; stable. |
| ~~**Letta**~~ | **Rejected at kernel layer.** Startup-led (a16z-backed); OSS has gravitational pull toward hosted platform. Acceptable only as **optional pack-level** backend for agents that want "memory blocks" specifically, never as the Memory Graph substrate. | See directives log; pure-OSS audit in flight. |
| **MCP over vsock + QUIC** | Capability Broker is an MCP server at kernel boundary; local agents use vsock, remote cloud twin uses QUIC. | MCP spec is Streamable-HTTP mainline; maintain a transport shim. |
| **bootc + OSTree + Lanzaboote** | Atomic transactional updates; measured boot chained into provenance; Morning Brief shows "last night's diff + rollback." | bootc-on-NixOS is not paved; this seam IS the moat — eng invest worth it. |
| **Agent Sandbox CRD schema** (K8s SIG) | Port the schema as declarative contract for Agent Runtime; no Kubernetes, but schema-compatible so devs with Agent-Sandbox YAML drop in instantly. | Schema young (<1yr); pin a tagged minor and track. |
| **sqlite-vec + fastembed-rs + SQLite + blake3 CAS** | Pure-OSS Memory Graph stack. No single-vendor dependency at the kernel. Pack-level memory libraries (Letta, Mem0, etc.) optional via SDK. | Apache-2.0 / MIT / public domain. Stable. |

## Out of scope at v0

- Money / wires / cards (v2).
- E-signature / legal binding (v2).
- Federation (Chief ↔ Chief) (v2).
- Pack marketplace / community registry (v1).
- Voice input (v1).
- Phone / watch surfaces beyond read-only mirror (v1).
- Multi-user / multi-human delegation (v2).
- Enterprise trust ledger / policies (v3).
- Branded hardware / physical puck (v3).
- Dual-boot installer / Windows compat (v1 or later).
- COSMIC DE / Niri tiling compositor (v2 surface story).
- **GPU acceleration for local inference** (v1+ per steward directive 2026-04-21). v0 runs CPU-only local models + cloud models. Rationale: many target users lack GPUs; simplifies v0 shipping surface.
- **Kernel-level LLM/GPU scheduling** (v1+ add-on). Model Router handles policy in userspace; kernel-level GPU resource scheduling deferred.
- **KV-cache-as-process-state** (v2+ research track). Checkpointable agent "mind" requires inference-engine-specific support; parked in ideation.

## Milestones

### Day 30 — "Night Loop runs locally"

- NixOS image builds; boot-to-Morning-Brief ≤ 10s.
- Capability Broker + Memory Graph + Provenance Log + Trust Ledger + Region Router services alive.
- opencode harness wired behind swappable interface.
- Sigstore emission on every agent action.
- 1 pack (inbox-triage) working end-to-end in sandbox.
- **Gate:** Can a hand-typed "read my test inbox and triage" produce a signed Morning Brief with 10+ cards?

### Day 60 — "Stack complete, demo rehearsable"

- All 6 Chief-of-Staff packs installed + coordinated.
- Live Agent View renders at 60fps with ≥ 20 active agents.
- Rewind primitive (`chief rewind 4h`) works end-to-end.
- Local-inference fallback demonstrated with network cable pulled.
- MCP-over-vsock path hot (< 5ms for in-process, < 20ms for guest agents).
- **Gate:** Morning Reveal hero video shot end-to-end on reference hardware without post-production magic.

### Day 90 — "Launch-ready"

- All 3 deploy targets build from one flake (ISO, VM, USB).
- Web "Try Your Own Morning Brief" preview live.
- Pack signing + install consent flow polished.
- Quarterly Review scheduled but not yet run.
- Waitlist + landing page online.
- **Gate:** An invited external user completes one full Night Handoff → Morning Brief cycle on their own machine.

## Team requirements (rough)

| Role | Headcount at v0 | Primary scope |
|---|---|---|
| Kernel / systems engineer (Rust) | 2 | Capability Broker, Event Bus, sandbox |
| NixOS ops / infra | 1 | Flake, 3 deploy targets, CI image builds |
| Agent harness + memory engineer | 1 | opencode integration, Memory Graph (pure-OSS stack) |
| Provenance / security engineer | 1 | Sigstore, in-toto, ceremony cryptography, eBPF |
| Surfaces engineer (UI + rendering) | 1 | Morning Brief, Live Agent View, Ceremony, Wayland compositor config |
| Brand + product design | 1 | Typography, sound, demo storyboard, landing page |
| PM / founder | 1 | Distribution, waitlist, creator Stacks, press |

Parallelizable across codex-on-worktree for independent workstreams (see [`AGENTS.md`](../AGENTS.md) and plandb entries).

## Risks (top 5, full list in [`risks-open`](./51-risks-open-questions.md))

1. **Boot time breaches 10s** → gate in CI; trim systemd units; move more to lazy init.
2. **Local-model quality embarrassing next to Claude** → confine local model to triage + summarization; never generation in demo.
3. **Morning Brief "live render" seen as fake** → make re-synthesis on render real (re-layout pre-computed cards); document it; post a technical writeup.
4. **bootc-on-NixOS engineering cost higher than budgeted** → v0 can fall back to Nix generations + OSTree-style snapshots; full bootc integration can slip to v1.
5. **Pack sandbox escape** → eBPF enforcement as belt-and-suspenders; consider Firecracker earlier if threat appears.

## Acceptance (ship criteria)

- [ ] Day-90 gate met: external user completes Night Handoff → Morning Brief unsupervised.
- [ ] Morning Reveal hero video shot and approved.
- [ ] All 7 OS-only capabilities demonstrable in the hero video.
- [ ] Sigstore verification passes on every artifact shown in the demo.
- [ ] No feature in v0 that doesn't improve the Morning Reveal or Stack flywheel (Axiom 9).

## Related

- [`north-star`](./00-north-star.md) — staged vision
- [`chief-kernel`](./11-chief-kernel.md) — what's implemented at v0
- [`security-model`](./14-security-model.md) — Sigstore integration detail
- [`viral-loop`](./60-viral-loop.md) — Morning Reveal + try-your-own preview
- [`research/2026-04-21-oss-landscape-scan.md`](./research/2026-04-21-oss-landscape-scan.md) — OSS integrations source
