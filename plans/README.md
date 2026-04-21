---
id: plans-readme
title: "Implementation Plans"
status: stable
owners: [santosh]
last_updated: 2026-04-21
tags: [plans, process]
---

# Plans

## Purpose

Step-by-step implementation plans derived from design docs. Each plan corresponds to a concrete block of work (a phase, a milestone, or a single large feature).

## When to write a plan

- You have a design doc (`docs/NN-*.md`) that's accepted.
- You are about to do multi-day work — multiple files, multiple agents, dependencies.
- You want to track progress against explicit steps across sessions.

## When NOT to write a plan

- Single-file edit → just do it.
- Research task → use `docs/research/` instead.
- Open brainstorm → use `docs/ideation/`.

## Format

`YYYY-MM-DD-<slug>-plan.md`. Frontmatter:

```yaml
---
id: plan-<slug>
title: "..."
status: draft | active | complete | abandoned
owners: [handle]
last_updated: YYYY-MM-DD
related: [design-doc-ids]
tags: [...]
---
```

Body structure:

1. **TL;DR** — what we're building, why, in 4 bullets.
2. **Prerequisites** — design docs, ADRs, decisions that must be locked.
3. **Steps** — numbered, each with: goal, files to touch, acceptance criterion.
4. **Parallelizable phases** — which steps can run in parallel (worktree candidates).
5. **Risks** — what could go wrong; mitigations.
6. **Verification** — how we know it's done (tests, demo, manual checks).

## Workflow

1. Write plan.
2. Commit.
3. Open plandb task(s) referencing the plan.
4. Execute via plandb loop (claim → work → done → next).
5. Mark plan `complete` when all plandb tasks done and verification passes.

## Related

- [`../docs/`](../docs/) — source of truth for design
- [`../adr/`](../adr/) — decisions plans depend on
- `../.plandb.db` — task tracking (intentionally in git for this project)
