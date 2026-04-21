---
id: ideation-readme
title: "Ideation — how to use"
status: stable
owners: [santosh]
last_updated: 2026-04-21
tags: [ideation, process]
---

# Ideation

## Purpose

Append-only log of ideas not yet committed to design. Use for:

- Sketching possibilities that haven't earned an ADR yet.
- Capturing stray thoughts that might matter later.
- Recording user directives that need cross-doc audit.

## What belongs here

- Future features not scoped to v0/v1/v2.
- Experimental alternatives to current design decisions (still considering).
- User directives captured verbatim, awaiting translation into design.
- Open "what-if" explorations.

## What does NOT belong here

- Accepted design — goes in `docs/NN-*.md` design docs.
- Accepted decisions — go in `adr/`.
- Research with findings — goes in `docs/research/`.
- Concrete tasks — go in plandb.

## Format

One file per ideation entry: `YYYY-MM-DD-<slug>.md`. Append-only.

Frontmatter:

```yaml
---
id: ideation-<slug>
title: "..."
status: open | promoted | archived
owners: [handle]
last_updated: YYYY-MM-DD
tags: [...]
---
```

Body: free-form, but lead with a TL;DR and list cross-references at the end.

## Promotion path

An ideation entry graduates when:

1. An ADR is opened for it (commit to direction), **or**
2. A design doc section is written incorporating it, **or**
3. A plandb task lands it into implementation scope.

Update frontmatter `status: promoted` and link to the new home.

## Archiving

If an idea is rejected or stale:

1. Set `status: archived`.
2. Write a one-line *why-archived* at the bottom.
3. Leave the file for history.

Do not delete.
