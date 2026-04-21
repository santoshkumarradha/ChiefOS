# AGENTS.md — Instructions for AI agents working in this repo

This file is the entry point for any AI agent (human-driven or autonomous) collaborating on Chief OS. Read it before you do anything else.

## What this repo is

Chief OS: an AI-native operating system where agents are first-class users and humans are approvers. Pre-v0 — currently design docs only, no code.

## Read-order for new agents

1. [`AGENTS.md`](./AGENTS.md) — you are here
2. [`CLAUDE.md`](./CLAUDE.md) — dev workflow, PlanDB loop, PR process, hooks
3. [`CHARTER.md`](./CHARTER.md) — 10 axioms; every decision must derive from them
4. [`docs/INDEX.md`](./docs/INDEX.md) — machine-readable map of all docs
5. [`docs/00-north-star.md`](./docs/00-north-star.md) — mission, Person Zero, wedge
6. [`docs/02-architecture.md`](./docs/02-architecture.md) — L0–L4 layers
7. [`docs/13-v0-scope-90-day.md`](./docs/13-v0-scope-90-day.md) — what ships first
8. [`docs/14-risks-open-questions.md`](./docs/14-risks-open-questions.md) — what's undecided

## One-time setup (fresh clone)

```bash
scripts/install-hooks.sh     # activate pre-commit hook
scripts/plandb-restore.sh    # rebuild .plandb.db from committed SQL
export PLANDB_DB="$(pwd)/.plandb.db"
```

## Doc conventions

Every doc uses YAML frontmatter:

```yaml
---
id: <kebab-case-id>
title: "Human title"
status: draft | review | stable | deprecated
owners: [handle]
last_updated: YYYY-MM-DD
related: [other-doc-ids]
depends_on: [other-doc-ids]
tags: [area-tags]
---
```

Sections (in order, omit if empty):

1. **TL;DR** — ≤ 4 bullets
2. **Context** — why this doc exists
3. **Contract** — inputs / outputs / invariants
4. **Decisions** — numbered, each with rationale
5. **Diagram** — mermaid where useful
6. **Acceptance** — checklist of done-criteria
7. **Open questions** — numbered
8. **Related** — cross-links

## Where to add new content

| If you are adding… | Put it in… |
|---|---|
| A new system-wide design doc | `docs/NN-<slug>.md` (next number) |
| A decision that changes an axiom or architecture | `adr/NNNN-<slug>.md` (next number, use `adr/template.md`) |
| A future idea, experiment, exploration | `docs/ideation/YYYY-MM-DD-<slug>.md` |
| A concrete requirement (functional / NFR / security / perf) | `docs/requirements/<bucket>.md` (append a row) |
| A reusable diagram referenced from multiple docs | `docs/diagrams/<slug>.mmd` + embed via link |
| A research reference or competitor analysis | `docs/research/<slug>.md` |
| An implementation plan | `plans/YYYY-MM-DD-<slug>-plan.md` |
| A throwaway code spike | `prototypes/<slug>/` with its own `README.md` |
| Brand, naming, visual identity | `brand/<slug>.md` |

## Never do this

- Never write prose for prose's sake. Tables, lists, diagrams > paragraphs.
- Never soften axioms in a doc; propose an ADR instead.
- Never add a feature without checking against Axiom 9 (demos are product): "Does this improve the Morning Reveal or the Stack flywheel?"
- Never introduce ambient authority (Axiom 2). Every call goes through the Capability Broker.
- Never duplicate information. Link instead. If you can't link, the target doc is missing a heading — add one.
- Never commit secrets, API keys, or personal memory-graph dumps.

## Always do this

- Update `docs/INDEX.md` when you add a doc.
- Update `last_updated` in frontmatter when you edit.
- Cross-link aggressively.
- If you make a decision, write an ADR; if the decision is small, link it from the relevant doc.
- When in doubt, ask the steward (Santosh) via a TODO with a question in the doc.

## Stability contract for agents

- **Stable:** doc status `stable` — safe to cite in external material.
- **Review:** ready for human sign-off; link but note "review".
- **Draft:** expect churn; do not cite externally.
- **Deprecated:** do not base new work on; link forward to replacement.

## Minimal "hello, new agent" loop

```
1. Read CHARTER.md → internalize axioms
2. Read docs/INDEX.md → locate relevant areas
3. Read the 3–5 docs most relevant to your task
4. Check docs/14-risks-open-questions.md for known unknowns
5. Draft your change; validate against all 10 axioms
6. If you amend an axiom or core architecture → propose an ADR
7. Update frontmatter + INDEX.md
```
