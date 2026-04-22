# CLAUDE.md — Development Workflow for AI Agents

**If you are Claude, Gemini, Codex, Cursor, or any other AI agent that landed in this repo: read this first.** Whole workflow is here. PlanDB (`.plandb.db` in this directory, intentionally tracked in git) is the single source of truth for in-flight work and persistent across all agents and sessions.

## Project

**Chief OS** — AI-native operating system built on NixOS. Agents are first-class users; humans are approvers. Currently **pre-v0 (design phase).** No code shipped yet.

## Orientation (read in this order)

1. **[`CLAUDE.md`](./CLAUDE.md)** — you are here
2. **[`AGENTS.md`](./AGENTS.md)** — how docs are organized; where to add what
3. **[`CHARTER.md`](./CHARTER.md)** — 10 axioms; never violate without ADR
4. **[`docs/INDEX.md`](./docs/INDEX.md)** — machine-readable doc map
5. **[`docs/00-north-star.md`](./docs/00-north-star.md)** — wedge, Person Zero, staged vision
6. **[`docs/10-architecture.md`](./docs/10-architecture.md)** — L0–L4
7. **[`docs/50-v0-scope-90-day.md`](./docs/50-v0-scope-90-day.md)** — what ships first
8. **[`docs/51-risks-open-questions.md`](./docs/51-risks-open-questions.md)** — open decisions

## PlanDB — source of truth for all work

Everything you do in this repo is tracked in `.plandb.db`. State persists across sessions, machines, and agents.

### The loop

```bash
# 0. ALWAYS pin PLANDB_DB to our tracked repo db.
#    PlanDB walks up from CWD; from a worktree (labs/chief-os-<slug>/) it would
#    find a different .plandb.db. Pin explicitly — non-negotiable.
export PLANDB_DB=/Users/santoshkumarradha/Documents/agentfield/code/labs/nix/.plandb.db
#    Agents working in a fresh clone should instead compute the root:
#      export PLANDB_DB="$(git -C "$(git rev-parse --show-toplevel 2>/dev/null || echo .)" ls-tree -r --name-only HEAD | grep -m1 '^\.plandb\.db$' >/dev/null && git rev-parse --show-toplevel)/.plandb.db"
#    Or simply: cd into the main repo before running plandb commands.

# 1. Check what's ready
plandb list --status ready --compact

# 2. Set your identity and claim
export PLANDB_AGENT=claude-1           # or codex-1, gemini-1, cursor-<slug>, etc.
plandb task start t-<id> --agent $PLANDB_AGENT    # start a specific task

# 3. Do the work (the task's --description is your spec)

# 4. Record discoveries as you go
plandb context "found that X requires Y" --kind discovery

# 5. Complete
plandb done t-<id> --result '{"pr":"<url>","summary":"..."}'
```

### Persistence model (read this — changed 2026-04-21)

**The binary `.plandb.db` is NOT tracked in git.** It's gitignored. Parallel-worktree writes to a binary file caused merge conflicts and state loss; we fixed that by committing a text SQL dump instead.

- **`.plandb/state.sql`** — authoritative full state (status, results, contexts, events). Committed. Mergeable text. Used by the restore script.
- **`.plandb/template.yaml`** — human-readable sidecar (plandb's native `export`). Graph shape only. **Lossy** — drops status, results, and contexts. Great for PR reviewers to see the task graph at a glance, never used for restore.
- **`.plandb.db`** — local binary, ephemeral, gitignored. Each clone / worktree rebuilds from the SQL dump.

**Why both?** `plandb export` produces clean YAML but intentionally only captures the graph shape — designed as a reusable decomposition template, not a state snapshot. We need full state (is task X done? what was the result? what contexts did agents record?), so we commit the SQL dump too.

#### Scripts

```bash
# One-time per clone: install git hooks (auto-exports on every commit)
scripts/install-hooks.sh

# Manual export (usually not needed — pre-commit hook handles it)
scripts/plandb-export.sh

# Rebuild db from committed text (run on fresh clone or after pull)
scripts/plandb-restore.sh            # refuses if .plandb.db exists
scripts/plandb-restore.sh --force    # replaces (auto-backs-up the old one)
```

#### Pre-commit hook

`scripts/install-hooks.sh` points `git config core.hooksPath` at `.githooks/`. Then on every commit, `.githooks/pre-commit` runs `plandb-export.sh` and auto-stages `.plandb/state.sql` + `.plandb/template.yaml` if their content changed.

Skip in one-off cases with `SKIP_PLANDB_HOOK=1 git commit ...`.

Skip conditions (built in): mid-rebase, mid-merge, mid-cherry-pick — the hook gets out of the way to avoid confusing those state machines.

#### New-session / fresh-clone bootstrap

```bash
git clone git@github.com:santoshkumarradha/ChiefOS.git
cd ChiefOS
scripts/install-hooks.sh              # one-time: activate pre-commit hook
scripts/plandb-restore.sh             # rebuild .plandb.db from committed SQL
export PLANDB_DB="$(pwd)/.plandb.db"  # pin absolute path (see gotcha below)
plandb status --detail                # confirm state
```

On an existing clone, `git pull` pulls the updated SQL; re-run `scripts/plandb-restore.sh --force` if you want the local binary to match exactly.

#### Before opening a PR on main (or before merging)

```bash
# Export the current state so downstream agents pick it up
scripts/plandb-export.sh
git add .plandb/state.sql
git commit -m "chore(plandb): export state"
# Then open/merge the PR as usual.
```

#### PLANDB_DB gotcha (still applies)

PlanDB walks up from your CWD. From a worktree sibling to `labs/nix/`, it would find a wrong `.plandb.db`. **Pin** the absolute path every session:

```bash
export PLANDB_DB=/Users/santoshkumarradha/Documents/agentfield/code/labs/nix/.plandb.db
```

Or `cd` into `labs/nix/` before touching plandb. Without this pin, agents write to the wrong db and state diverges.

### Inspection

```bash
plandb status --detail              # full per-task breakdown
plandb list --status ready          # what's claimable now
plandb list --status running        # what's in flight (maybe by someone else)
plandb critical-path                # what determines total completion time
plandb bottlenecks                  # tasks blocking the most downstream work
plandb ahead --depth 3              # preview what opens up next
plandb show t-<id>                  # details of a specific task
```

### Adaptation

```bash
plandb split --into "A, B, C"                     # split the running task
plandb task insert --after t-a --before t-b ...   # insert a missed step
plandb task amend t-<id> --prepend "NOTE: ..."    # annotate a future task
plandb what-if cancel t-<id>                      # preview cancel impact
```

## Multi-agent / multi-worktree conventions

This project is explicitly designed to be worked on by multiple AI agents in parallel, on git worktrees, across multiple origins.

### Branch naming

| Kind of work | Branch |
|---|---|
| Prototype spike | `proto/<slug>` (e.g. `proto/capability-broker`) |
| Doc edit / research | `docs/<slug>` |
| ADR | `adr/<NNNN-slug>` |
| Bugfix | `fix/<slug>` |
| Feature (post-v0) | `feat/<slug>` |

### Worktree convention

```bash
# One worktree per parallel task
git worktree add ../chief-os-proto-broker -b proto/capability-broker

# Inside the worktree, set your agent identity
export PLANDB_AGENT=codex-broker
cd ../chief-os-proto-broker
plandb task claim t-5296 --agent $PLANDB_AGENT     # the cap-broker task

# Do work, commit, push to origin, open PR, plandb done
git push -u origin proto/capability-broker
gh pr create
plandb done --next --result '{"pr":"<url>"}'
```

### Agent-identity conventions

Use the pattern `<model>-<task-slug>` or `<tool>-<slug>`:

- `claude-cap-broker`
- `codex-region-router`
- `gemini-morning-brief`
- `cursor-hax-inbox`
- `claude-1` / `codex-1` for generic work

This makes `plandb list --agent <name>` searches meaningful.

### Parallelism rule

When `plandb list --status ready` returns 2+ tasks, **spawn sub-agents or worktrees in parallel.** Don't serialize.

## Task descriptions are the spec

Every plandb task has a `--description` that is **self-contained**. If you're picking up a task from a fresh session, the description must have:

1. Hypothesis / goal
2. Files expected (paths)
3. Acceptance criteria
4. Worktree / branch convention
5. Parallel-safety notes
6. References (other docs, ADRs)

If a task description isn't self-contained, amend it before claiming.

## PR process (distributed multi-agent flow)

### One task → one branch → one PR → one merge

```bash
# 0. Set identity + pin plandb db (non-negotiable, see gotcha above)
export PLANDB_AGENT=codex-region-router
export PLANDB_DB=/Users/santoshkumarradha/Documents/agentfield/code/labs/nix/.plandb.db

# 1. Create worktree + branch from main
cd /Users/santoshkumarradha/Documents/agentfield/code/labs/nix
git worktree add ../chief-os-region-router -b proto/region-router
cd ../chief-os-region-router

# 2. Claim + start the task (atomic across agents; PLANDB_DB pinned above)
plandb task claim t-proto-region --agent $PLANDB_AGENT
plandb task start t-proto-region --agent $PLANDB_AGENT

# 3. Do the work. Record discoveries as you go.
plandb context "discovery: nushell crate licensing is MIT" --kind discovery

# 4. Commit using the <type>: format (see below). Co-Authored-By footer required.

# 5. Push branch to origin.
git push -u origin proto/region-router

# 6. Open PR using the template at .github/PULL_REQUEST_TEMPLATE.md
gh pr create --title "proto: Region Router deterministic classifier" \
             --body "..."

# 7. Record the PR URL into plandb
plandb done --next --result '{"pr":"<url>","summary":"..."}'

# 8. Steward (or designated reviewer) merges the PR. After merge:
git worktree remove ../chief-os-region-router
git branch -D proto/region-router       # only after merge
```

### PR review expectations

- Reviewer runs the commands in the task description.
- Every item in the PR template's acceptance checklist is ticked or explicitly waived.
- Axioms box is honest — if an axiom is touched but not listed, review halts.
- `plandb show t-<id>` must match the PR scope.

### Rebasing during long work

If `main` advances while your branch is open:

```bash
cd ../chief-os-<slug>
git fetch origin
git rebase origin/main        # never merge; rebase keeps history linear
git push --force-with-lease    # safe for a branch you own
```

Never force-push `main`. Never merge `main` backwards into a feature branch.

### PR-from-fork (external contributors)

External contributors fork the repo, push to their fork, open PR against `main`. Same template applies. Acceptance gates are identical; they can't write plandb entries from their fork — reviewer writes the plandb follow-up on merge.

## CI (when live)

- `.github/workflows/doc-lint.yml` — frontmatter + internal-link check on PRs touching docs.
- Future:
  - Rust workspace build + test (when `spikes/*/Cargo.toml` or `crates/*/Cargo.toml` lands).
  - TypeScript typecheck + lint (for surfaces).
  - Sigstore / cosign verification on signed pack PRs.
  - PlanDB state consistency check (no orphan deps, no runaway claims).

## Commit message format

```
<type>: <short summary>

<body with context>

Co-Authored-By: <model> <noreply@anthropic.com or provider>
```

Types: `init`, `feat`, `fix`, `refactor`, `docs`, `adr`, `proto`, `test`, `chore`.

Example:
```
proto: chief-mem memory stack with sqlite-vec and fastembed-rs

Implements adr-0008 pure-OSS substrate. 100k synthetic nodes tested at
p50 = 140ms for k=20 retrieval.

Co-Authored-By: Codex <noreply@openai.com>
```

## Restarting work (new session, any agent)

```bash
cd /Users/santoshkumarradha/Documents/agentfield/code/labs/nix
git pull
# Pin the db path (non-negotiable, see gotcha above)
export PLANDB_DB="$(pwd)/.plandb.db"
# If no local db OR you want to reset to repo's canonical state:
[ -f .plandb.db ] || scripts/plandb-restore.sh
plandb status --detail                  # where did we leave off?
plandb list --status running            # anything mid-flight elsewhere?
plandb list --status ready              # what's next?
plandb search "keyword"                 # find relevant context
plandb task claim t-<id> --agent <your-handle> && plandb task start t-<id> --agent <your-handle>
```

All state persists in `.plandb/state.sql` (text, in git). The binary `.plandb.db` is rebuilt from it on demand. If you need to re-read context that was recorded earlier:

```bash
plandb contexts                        # list all context entries
plandb context <c-id>                  # show a specific context entry
plandb search "your query"             # search everything (BM25)
```

## Recording what you learn

Throughout work, record discoveries. They auto-surface for future agents:

```bash
plandb context "discovery: iroh-blobs API changed in v0.9; adjust wrapper" --kind discovery
plandb context "decision: use fastembed-rs default model, not bge-m3" --kind decision
plandb context "pattern: all L2 services expose /health and /metrics" --kind pattern
plandb context "blocker: Wayland compositor XDG portal missing, blocks clipboard proto" --kind blocker
plandb context "constraint: pack grants are closed-enum, must ADR to extend" --kind constraint
```

Kinds are freeform: `discovery` | `decision` | `pattern` | `blocker` | `reference` | `constraint` | `insight`.

## Where things live

| You want to... | Go to... |
|---|---|
| See the axioms | [`CHARTER.md`](./CHARTER.md) |
| Understand layer architecture | [`docs/10-architecture.md`](./docs/10-architecture.md) |
| Find an accepted decision | [`adr/`](./adr/) |
| Add a new design doc | `docs/NN-*.md` + update `docs/INDEX.md` |
| Propose a decision | `adr/NNNN-slug.md` using `adr/template.md` |
| Capture an idea for later | `docs/ideation/YYYY-MM-DD-slug.md` |
| Record a user directive | `docs/directives/README.md` (append) |
| Track what-if research | `docs/research/YYYY-MM-DD-slug.md` |
| Write an implementation plan | `plans/YYYY-MM-DD-slug-plan.md` |
| Spike a risky idea | `spikes/<slug>/` with own README + CLEANUP date |
| Capture a brand decision | `brand/*.md` |
| Track tasks | `.plandb.db` (use `plandb` CLI; see above) |

## Escalating

- **Need a decision you can't make:** add a row to the decisions-needed table in [`docs/51-risks-open-questions.md`](./docs/51-risks-open-questions.md), assign an owner and deadline.
- **Found an axiom that's wrong:** propose an ADR.
- **Architecture shifted:** open an ADR that cites what changed.
- **Directive from steward that cuts across docs:** append to [`docs/directives/README.md`](./docs/directives/README.md).

## Non-negotiables

1. **Never skip PlanDB.** Every non-trivial task tracked there. "Simple" is a rationalization.
2. **Never write prose for prose's sake.** Tables, lists, diagrams > paragraphs.
3. **Never soften an axiom in a doc.** Open an ADR.
4. **Never introduce ambient authority.** Everything through the Capability Broker (in code) or with explicit capability display (in design).
5. **Never duplicate info.** Link instead. If you can't link, add a heading to the target.
6. **Never commit secrets.** `.env`, keys, personal memory-graph dumps — off-limits.
7. **Never move past a failed pre-commit hook with `--no-verify`.** Fix the underlying issue.

## Dispatching multi-model workers (for Claude on the main branch)

If you need parallel execution and are running as Claude Code on the parent machine:

| Worker | CLI | Good for |
|---|---|---|
| Codex | `codex exec --full-auto --skip-git-repo-check -C <worktree> "task"` | Greenfield multi-file, test suites |
| Cursor | `cursor agent -p --force --trust --workspace <worktree> "task"` | Medium edits, refactors |
| Gemini | `gemini -p "task" -y --include-directories <worktree>` | Trivial edits, boilerplate, creative UI |
| Claude sub-agent | Agent tool | Review, security, planning |

See `~/.claude/CLAUDE.md` for full routing guidance. Log dispatches to `~/.claude/routing-log.csv`.

## TL;DR for a new agent

```
1. Read this file, AGENTS.md, CHARTER.md, docs/INDEX.md.
2. `plandb status --detail` — understand state.
3. `plandb go --agent <your-handle>` — claim a task.
4. Do the work following the description as spec.
5. `plandb context "..." --kind discovery` — record findings.
6. Commit with Co-Authored-By. Push. Open PR if branching.
7. `plandb done --next` — hand off to next wave.
```

That's the whole loop.
