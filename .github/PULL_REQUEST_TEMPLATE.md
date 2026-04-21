<!--
Chief OS PR template — keep tight. Every section is required.
-->

## What

<!-- One paragraph. What changed and why. -->

## PlanDB reference

- Task: `t-<id>`
- Agent: `<agent-handle>` (e.g. `codex-region-router`)
- Worktree: `proto/<slug>` (or `docs/<slug>`, `adr/<slug>`)

## Axioms touched

<!-- List any of the 10 axioms this PR touches. See CHARTER.md. Empty is fine for docs/proto changes. -->

- [ ] Axiom 1 — AI is primary user
- [ ] Axiom 2 — No ambient authority
- [ ] Axiom 3 — Provenance
- [ ] Axiom 4 — Trust calibrated
- [ ] Axiom 5 — Reversibility
- [ ] Axiom 6 — Local-first
- [ ] Axiom 7 — Boring infra
- [ ] Axiom 8 — OS is protocol
- [ ] Axiom 9 — Demos are product
- [ ] Axiom 10 — HAX as enforcement
- [ ] None — non-load-bearing

If any axiom is touched in a way that amends it: open an ADR, link here.

## Acceptance checklist (from task description)

<!-- Copy the Acceptance section from the plandb task description and tick each item. -->

- [ ] ...

## Parallel-safety

- [ ] This PR touches files no other parallel task touches.
- [ ] If not: resolved conflict paths are listed here → `<paths>`.

## Follow-ups opened (if any)

- plandb task `t-<id>` for ...

## Tests / evidence

<!-- Paste command outputs, screenshots, benchmarks, or provenance log excerpts. -->

```
<output>
```

## Verification

- [ ] Doc frontmatter `last_updated` bumped where relevant.
- [ ] `docs/INDEX.md` updated if docs added/renamed.
- [ ] PlanDB task marked done via `plandb done --next --result '<json>'`.
- [ ] Commit messages follow the `<type>: ...` format with `Co-Authored-By`.
