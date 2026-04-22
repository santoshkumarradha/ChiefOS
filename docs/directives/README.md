---
id: directives-readme
title: "Directives Log"
status: stable
owners: [santosh]
last_updated: 2026-04-21
tags: [process, directives, audit]
---

# Directives Log

## Purpose

Append-only log of verbal directives from the steward (founder) captured during conversations. Each directive is time-stamped, categorized, and periodically audited against existing docs for needed changes.

## Rules

1. **Append-only.** Never edit past entries; add correction entries if needed.
2. **Time-stamp every entry** (`YYYY-MM-DD HH:MM` when known; date-only otherwise).
3. **Quote or close-paraphrase** the directive so intent is preserved.
4. **Tag** with one of: `scope`, `architecture`, `design`, `ux`, `gtm`, `legal`, `process`.
5. **Track resolution** — each entry gets a `resolution:` field updated as audits happen.

## Entry format

```markdown
### YYYY-MM-DD — <short-title>

**Tag:** `<tag>`
**Directive:** "<quote or paraphrase>"
**Captured-via:** <conversation|message|note>
**Resolution:**
- [ ] <action 1>
- [ ] <action 2>
**Docs touched (or to touch):** [...]
```

## Audit cadence

- **Every 2 weeks (v0):** audit all `open` directives against existing docs. Flag divergence.
- **Before every phase gate (30/60/90):** full sweep; ensure all directives resolved or explicitly deferred.

## Log

### 2026-04-21 — Use plandb for all non-trivial work; add plandb.db to git

**Tag:** `process`
**Directive:** Track all design + implementation work via PlanDB. Add `.plandb.db` to git so state persists across sessions and agents.
**Resolution:**
- [x] PlanDB initialized for chief-os project.
- [x] `.gitignore` updated with explicit `!.plandb.db` negation.
- [x] 28 tasks decomposed with dependencies.
**Docs touched:** `.gitignore`, `AGENTS.md`

### 2026-04-21 — Use codex for parallel worktrees; opencode for AI harness (swappable)

**Tag:** `architecture`
**Directive:** For AI-harness work inside Chief OS, use opencode at v0 but ensure modularity so other harnesses can swap in. Parallelize future implementation via codex on git worktrees.
**Resolution:**
- [x] Harness interface defined as a stable swap point in `docs/11-chief-kernel.md`.
- [x] opencode named as v0 default; swappability requirement captured in `requirements/non-functional.md` (NF-801).
- [ ] Create ADR formalizing the harness interface (tracked as plandb context note).
**Docs touched:** `11-chief-kernel.md`, `requirements/non-functional.md`, `50-v0-scope-90-day.md`

### 2026-04-21 — Use appropriate language per task; no OS-startup-time requirement at v0

**Tag:** `architecture`
**Directive:** Don't constrain to a single language. Pick best tool per task (Rust, Go, TypeScript, Python, Nix). Boot-time optimization is explicitly not a v0 requirement; optimize later.
**Resolution:**
- [x] Polyglot language policy added to `docs/10-architecture.md`.
- [x] Captured as NF-001 through NF-007 in `requirements/non-functional.md`.
- [x] Boot-time non-requirement explicit in `requirements/performance.md`.
**Docs touched:** `10-architecture.md`, `requirements/non-functional.md`, `requirements/performance.md`

### 2026-04-21 — Model routing must be swappable at multiple granularities

**Tag:** `architecture`
**Directive:** Have a modular way to switch between local and cloud models. Who controls the switch is a later design decision, but the architecture MUST allow it.
**Resolution:**
- [x] `Model Router` added as first-class L2 kernel service.
- [x] `ModelBackend` trait defined with 5 routing granularities (system/category/pack/agent/call).
- [x] Captured as F-201–F-206 and baked into diagrams.
**Docs touched:** `10-architecture.md`, `41-local-vs-cloud.md`, `11-chief-kernel.md`, `requirements/functional.md`, `diagrams/kernel-services.mmd`, `diagrams/layers.mmd`

### 2026-04-21 — Filesystem (and more broadly all OS primitives) must be rethought for agent-primary usage

**Tag:** `architecture`
**Directive:** Filesystem was just an example. Think broadly about EVERY OS primitive and whether rethinking it for agent-primary usage unlocks speed, new features, viral demos, or strategic uniqueness.
**Resolution:**
- [x] Background research agent dispatched; returned 5 substrate-spine primitives: signed typed event log, CAS filesystem, typed clipboard, HAX inbox, omnibar.
- [x] Filesystem research deep-dive done (`docs/research/2026-04-21-agent-native-fs.md`) — fastembed-rs + bge-small picked.
- [x] Architecture updated with substrate spine section.
- [x] ADRs 0005, 0006, 0007 opened.
- [x] Surfaces doc updated with 3 new surfaces (HAX Inbox, Omnibar, Clipboard Pane).
- [x] v0 scope updated with substrate-spine non-negotiables.
**Docs touched:** `10-architecture.md`, `30-surfaces.md`, `50-v0-scope-90-day.md`, `research/2026-04-21-ai-native-primitive-rethinks.md`, `research/2026-04-21-agent-native-fs.md`, `adr/0005`, `adr/0006`, `adr/0007`

### 2026-04-21 — Docs must be optimized for AI-agent consumption (mermaid, frontmatter, tight)

**Tag:** `process`
**Directive:** Docs should be agent-consumable: mermaid diagrams, YAML frontmatter, tables over prose, clear index. Future dynamic development benefits from this structure.
**Resolution:**
- [x] All docs use consistent YAML frontmatter (id, title, status, owners, last_updated, related, tags).
- [x] Dense table-first format adopted; mermaid embedded in 3+ architecture docs.
- [x] `docs/INDEX.md` machine-readable map with every doc ID.
- [x] `AGENTS.md` at repo root as agent entry point.
**Docs touched:** all docs adopted new format; `AGENTS.md` created; `docs/INDEX.md` created and updated.

### 2026-04-21 — B2C virality is the primary launch motion; wow-factor features matter more than breadth

**Tag:** `gtm`
**Directive:** Goal is B2C virality and wow-factor. Start small; expand to enterprise later, not now.
**Resolution:**
- [x] Person Zero defined as high-agency prosumer-founder; enterprise explicitly v3+.
- [x] Hero video "Morning Reveal" scripted with 9 beats; every beat proves an OS-only feature.
- [x] "Try Your Own Morning Brief" web funnel scoped as viral loop 2.
- [x] Forkable Stacks scoped as viral loop 3 with 5 influencer-stack seeds at launch.
**Docs touched:** `00-north-star.md`, `60-viral-loop.md`, `61-open-source-business.md`

### 2026-04-21 — Features must honor HAX theory; Apple-grade UX bar

**Tag:** `design`
**Directive:** Everything purpose-thought-about and intentional. AI is main user. HAX principles (delegation × consequentiality × horizon) should be baked in as enforcement, not decoration. Apple-level sophistication.
**Resolution:**
- [x] Axiom 10 in Charter: "HAX maps 1:1 to architectural primitives."
- [x] `docs/01-hax-principles.md` maps 3 HAX dimensions → kernel primitives (Trust Ledger, Region Router, Memory-horizon tags).
- [x] `docs/02-apple-design-principles.md` defines 10 product-design principles with anti-patterns.
- [x] Every surface tagged with HAX region(s) in `docs/30-surfaces.md`.
**Docs touched:** `CHARTER.md`, `01-hax-principles.md`, `02-apple-design-principles.md`, `30-surfaces.md`

### 2026-04-21 — Memory substrate at kernel layer must be pure OSS (no single-company capture)

**Tag:** `architecture`
**Directive:** For memory at the OS kernel layer, use the best **pure-OSS** (multi-maintainer, not startup-led) components rather than a startup's OSS that has gravitational pull toward their hosted platform. Explicitly: Letta is not appropriate at the kernel layer; fine only as an optional pack-level backend.
**Resolution:**
- [x] `docs/11-chief-kernel.md`, `docs/13-memory-substrate.md`, `docs/50-v0-scope-90-day.md` updated to name pure-OSS default (SQLite + sqlite-vec + fastembed-rs + blake3 CAS).
- [x] `docs/research/2026-04-21-oss-landscape-scan.md` updated; Letta withdrawn from kernel-layer recommendation.
- [ ] Follow-up research agent dispatched to audit pure-OSS alternatives across all memory-stack components (vector index, graph store, blob CAS, full-text, embedding inference).
- [ ] After research returns: propose ADR codifying the "no single-company capture at kernel layer" principle.
**Docs touched:** `13-memory-substrate.md`, `11-chief-kernel.md`, `50-v0-scope-90-day.md`, `research/2026-04-21-oss-landscape-scan.md`

### 2026-04-21 — Provable AI is kernel-native (promoted from "demo flex")

**Tag:** `architecture`
**Directive:** Provenance and cryptographic verifiability of AI output must be internal, cleverly designed (not slow, not bloated, not memory-heavy), and woven into the substrate — not an optional add-on or a Chief-branded interface layer.
**Resolution:**
- [x] ADR-0009 opened: Signed Inference as L2 primitive (`chief-inference` service).
- [x] Per-inference attestation ~400 B; Ed25519 signing; BLAKE3 hashes; ~1–2 ms overhead per call; ≤ 5 ms NFR.
- [x] Three-tier claim model (Generated / Co-signed / Custody) prevents implicit over-claiming.
- [x] Docs updated: `10-architecture.md`, `11-chief-kernel.md`, `41-local-vs-cloud.md`, `50-v0-scope-90-day.md`, requirements F-207..F-214, NF-507..NF-508.
- [ ] Implementation prototype (pending — new plandb task).
- [ ] Canonical prompt form adversarial review (open question in `51-risks-open-questions`).
- [ ] Device-key publication path decision (DID vs Sigstore Fulcio vs self-hosted Rekor).

### 2026-04-21 — GPU / hardware scheduling deferred to v1+

**Tag:** `scope`
**Directive:** Kernel-level LLM/GPU scheduling and local GPU inference deferred. Steward lacks GPU for testing; many target users also lack one. v0 targets CPU models + cloud models only. Model Router handles routing in userspace; hardware scheduling can become an add-on later.
**Resolution:**
- [x] `docs/50-v0-scope-90-day.md` out-of-scope table updated.
- [x] `docs/41-local-vs-cloud.md` local stack pinned to CPU (Qwen 2.5 7B Q4 / Llama 3.2 8B Q4 default).
- [x] `docs/requirements/non-functional.md` NF-505 revised (CPU targets).
- [x] Apple Silicon Metal left ambiguous — flagged as decision D10 to lock within 7 days. Lean: Metal is acceptable under v0 constraint since it's a built-in backend, not a discrete GPU.

## Next audit

- Date: 2026-05-05 (2 weeks).
- Owner: steward.
- Scope: full sweep; resolve `[ ]` items; capture new directives from intervening sessions.
