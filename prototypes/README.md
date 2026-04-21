---
id: prototypes-readme
title: "Prototypes — throwaway spikes"
status: stable
owners: [santosh]
last_updated: 2026-04-21
tags: [prototypes, process]
---

# Prototypes

## Purpose

Throwaway code spikes that validate a specific idea before committing to production implementation. Not shipped code.

## Rules

1. **Every prototype has its own directory** under `prototypes/<slug>/`.
2. **Every prototype has a `README.md`** at the root with:
   - What it's validating (the hypothesis).
   - How to run it.
   - What "success" looks like.
   - A CLEANUP date (default: 30 days from creation).
3. **Prototypes are not load-bearing.** Main code does not import from `prototypes/`.
4. **Delete before CLEANUP date or justify extension** in the prototype's README.

## When to write a prototype

- Validating a performance claim (e.g., "fastembed-rs can do 128/s on CPU").
- Testing a protocol (e.g., "MCP over vsock round-trip latency").
- Exploring a library's UX (e.g., "what does authoring a Capability Pack feel like").
- De-risking a specific architecture decision before ADR.

## When NOT to write a prototype

- You have a plan and a design — just build it.
- The risk is speculative — do research instead (`docs/research/`).

## File layout per prototype

```
prototypes/<slug>/
├── README.md          # hypothesis, run, success, cleanup date
├── src/               # code
├── measurements.md    # data collected (if applicable)
└── .gitignore         # prototype-specific
```

## Current prototypes

(None yet. First prototypes target: capability-broker, region-router, morning-brief-render, memory-graph tri-store. See plandb task `t-future-protos`.)

## Parallelization

Prototypes are ideal targets for **codex on git worktrees** (see [`../AGENTS.md`](../AGENTS.md)). Run 4 prototype spikes in parallel, each on a dedicated worktree + codex agent.

## Promotion

A prototype can be promoted if:
- It answers its hypothesis.
- The design doc is updated with findings.
- An implementation plan is opened in `plans/` for the real version.
- The prototype is still deleted or moved to `docs/research/` as evidence.
