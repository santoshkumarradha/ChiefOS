---
id: research-readme
title: "Research — how to use"
status: stable
owners: [santosh]
last_updated: 2026-04-21
tags: [research, process]
---

# Research

## Purpose

Store the outputs of research tasks: OSS scans, competitor analyses, academic citations, prior-art reviews. These are *inputs* to design, not design itself.

## What belongs here

- Scans of external ecosystem (libraries, protocols, tools).
- Competitor / adjacent-product analyses.
- Academic references with relevance notes.
- Deep-dives on a single technical choice.

## What does NOT belong here

- Accepted design — goes in `docs/NN-*.md`.
- Decisions — go in `adr/`.
- Open brainstorms — go in `docs/ideation/`.

## Format

One file per research entry: `YYYY-MM-DD-<slug>.md`. Frontmatter:

```yaml
---
id: research-<slug>
title: "..."
status: in_progress | review | stable
owners: [handle]
last_updated: YYYY-MM-DD
tags: [...]
---
```

Body: TL;DR, methodology, findings (tables preferred), prioritized integration picks, open questions, related cross-links.

## Current entries

- [`2026-04-21-oss-landscape-scan.md`](./2026-04-21-oss-landscape-scan.md) — L1/L2 OSS scan (pending inline write)
- [`2026-04-21-ai-native-primitive-rethinks.md`](./2026-04-21-ai-native-primitive-rethinks.md) — substrate-spine research
- [`2026-04-21-agent-native-fs.md`](./2026-04-21-agent-native-fs.md) — filesystem deep-dive

## Promotion path

When research findings commit Chief OS to a direction:

1. Open an ADR citing the research.
2. Update design docs.
3. Set research frontmatter `status: stable` if findings are locked in.
