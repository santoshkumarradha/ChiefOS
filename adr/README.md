---
id: adr-readme
title: "ADR Process"
status: stable
owners: [santosh]
last_updated: 2026-04-21
tags: [process, decisions]
---

# Architecture Decision Records

## Purpose

ADRs document decisions that:
- Amend an axiom in [`CHARTER.md`](../CHARTER.md), OR
- Set a layer-spanning technical standard, OR
- Commit to a specific external dependency or protocol, OR
- Reverse a prior decision.

Everything else lives in design docs.

## Numbering

Sequential, 4-digit: `0001-kebab-slug.md`, `0002-...`. Never renumber. Superseded ADRs keep their number and link to the new one.

## Lifecycle

```
proposed → under_review → accepted → (later) superseded | deprecated
```

## Authoring

1. Copy [`template.md`](./template.md) to `NNNN-<slug>.md`.
2. Fill in frontmatter and sections.
3. Open PR with `type: adr` label.
4. Merge requires steward approval (+ at least one non-author reviewer when team grows).

## When NOT to write an ADR

- Describing how something already works → design doc.
- Sketching an idea → `docs/ideation/`.
- Tracking a decision to make later → `docs/14-risks-open-questions.md` → decisions-needed table.

## Relationship to docs

- ADR = *why* a choice was made, *what alternatives existed*, *what changed*.
- Design doc = *how* the thing works today.

If the design doc and an ADR conflict, the ADR wins until a new ADR supersedes it.
