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
6. **[`docs/02-architecture.md`](./docs/02-architecture.md)** — L0–L4
7. **[`docs/13-v0-scope-90-day.md`](./docs/13-v0-scope-90-day.md)** — what ships first
8. **[`docs/14-risks-open-questions.md`](./docs/14-risks-open-questions.md)** — open decisions

## PlanDB — source of truth for all work

Everything you do in this repo is tracked in `.plandb.db`. State persists across sessions, machines, and agents.

### The loop

```bash
# 1. Check what's ready
plandb list --status ready --compact

# 2. Claim a task (you MUST set PLANDB_AGENT to your identity)
export PLANDB_AGENT=claude-1           # or codex-1, gemini-1, cursor-1, etc.
plandb go                               # auto-picks next ready task
# OR
plandb task claim t-<id> --agent $PLANDB_AGENT    # claim specific task

# 3. Do the work (the task's --description is your spec)

# 4. Record discoveries as you go
plandb context "found that X requires Y" --kind discovery

# 5. Complete + auto-claim next
plandb done --next --result '{"what":"changed"}'
```

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
plandb status --detail                  # where did we leave off?
plandb list --status running            # anything mid-flight elsewhere?
plandb list --status ready              # what's next?
plandb search "keyword"                 # find relevant context
plandb go --agent <your-handle>         # claim next ready task
```

All state persists in `.plandb.db`. If you need to re-read context that was recorded earlier:

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
| Understand layer architecture | [`docs/02-architecture.md`](./docs/02-architecture.md) |
| Find an accepted decision | [`adr/`](./adr/) |
| Add a new design doc | `docs/NN-*.md` + update `docs/INDEX.md` |
| Propose a decision | `adr/NNNN-slug.md` using `adr/template.md` |
| Capture an idea for later | `docs/ideation/YYYY-MM-DD-slug.md` |
| Record a user directive | `docs/directives/README.md` (append) |
| Track what-if research | `docs/research/YYYY-MM-DD-slug.md` |
| Write an implementation plan | `plans/YYYY-MM-DD-slug-plan.md` |
| Spike a risky idea | `prototypes/<slug>/` with own README + CLEANUP date |
| Capture a brand decision | `brand/*.md` |
| Track tasks | `.plandb.db` (use `plandb` CLI; see above) |

## Escalating

- **Need a decision you can't make:** add a row to the decisions-needed table in [`docs/14-risks-open-questions.md`](./docs/14-risks-open-questions.md), assign an owner and deadline.
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
