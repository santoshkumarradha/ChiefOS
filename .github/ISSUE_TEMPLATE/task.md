---
name: Task (PlanDB-linked)
about: A task tracked in PlanDB. GH issue mirrors the plandb task for visibility.
title: "t-<id>: <title>"
labels: ["task"]
---

## PlanDB task

- ID: `t-<id>`
- Kind: `code | research | review | test | shell | generic`
- Priority: `<n>`

## Spec (from PlanDB description)

<!-- Paste the plandb task --description verbatim. -->

## Worktree / branch

- Branch: `proto/<slug>` | `docs/<slug>` | `adr/<slug>` | `fix/<slug>` | `feat/<slug>`
- Worktree path: `../chief-os-<slug>`
- Assigned agent handle: `<model>-<slug>` (set via `PLANDB_AGENT`)

## Parallel-safety

- [ ] Does not touch files any other open task touches.

## Acceptance (copy from plandb task description)

- [ ] ...

## References

- PlanDB: `plandb show t-<id>`
- Docs: [...]
- ADRs: [...]
