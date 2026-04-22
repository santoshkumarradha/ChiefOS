# Chief OS — Docker Demo

One-command live demo. Real AI content in the Brief within ~30 seconds of `docker run`.

**Zero mocks.** The demo binary calls real HN, real OpenRouter, and intentionally triggers a real capability-escalation Ceremony against `*.substack.com`.

## Prerequisites

- Docker 24+ (`docker --version`)
- An OpenRouter API key: https://openrouter.ai/keys (~$0.01 covers many full runs with `openai/gpt-4o-mini`)

## Quickstart

From the repo root:

```bash
# Build + run in one step with docker compose
export OPENROUTER_API_KEY=sk-or-...
docker compose -f deploy/docker/docker-compose.yml up --build

# Open http://localhost:8080
```

Or with raw `docker run`:

```bash
docker build -f deploy/docker/Dockerfile -t chief-os-demo:latest .

docker run -p 8080:8080 \
           -e OPENROUTER_API_KEY=$OPENROUTER_API_KEY \
           -v $(pwd)/chief-state:/var/chief \
           -v $(pwd)/workspace:/workspace \
           chief-os-demo:latest
```

## What you'll see

| ~seconds | Event |
|---|---|
| 0 | Container boots, binary starts, prints `chief-os-demo HTTP listening` |
| 2 | HN briefer tick fires; fetches real top stories from `hn.algolia.com` |
| 5–15 | Real OpenRouter scoring against `openai/gpt-4o-mini` for up to 5 headlines |
| 15–25 | First Card enters `/v1/brief` — real titles, real URLs, real scores |
| 25–30 | Broker denies the pack's attempt at `*.substack.com` → Ceremony card appears |
| ~5 min | File-watcher tick scans `/workspace` — drop a `.md` file to see it summarized |

Browse:

- `http://localhost:8080/` — Morning Brief UI (React bundle from `apps/chief-brief-ui/`)
- `http://localhost:8080/v1/brief` — raw Brief JSON
- `http://localhost:8080/v1/inbox` — flat list of queued + handled cards
- `http://localhost:8080/v1/status` — health / uptime

## The architecture the demo exercises

This is the load-bearing piece: **the Ceremony is not a fixture.** It arises from a real capability escalation.

1. **Narrow grant at boot.** The demo orchestrator issues the `pack:hn-briefer` principal a `net.http` grant for `news.ycombinator.com` + `hn.algolia.com` only.
2. **Pack tries Substack.** Each tick, after scoring HN stories, the pack attempts `*.substack.com` via the broker.
3. **Broker denies.** `broker.check(...)` returns `CapabilityDenied { kind: "net.http", reason: ScopeExceeded }` — this is a real `CapabilityBroker` SQLite lookup.
4. **Event fires.** A `BusEvent::CapabilityCheck { allowed: false, ... }` hits the broadcast channel; the UI can subscribe and render it.
5. **Ceremony card queued.** A second Card of `action_type: "capability_escalation"` is inserted into `state.queued_cards`, complete with the principal, the denied op, and a `grant_narrowing_hint` the user approves.
6. **User approves.** When the user clicks approve (either via UI or `POST /approve`), the `/approve` handler validates a `ceremony.request` capability and records the attestation.

All of this is real. No seeded JSON, no stubbed denial, no synthetic card. Turn off `OPENROUTER_API_KEY` and the binary fails to start. Turn off network and the broker still denies Substack, which means the Ceremony card still appears (what's absent is the scored HN digest).

## Persistence

The `/var/chief` volume contains:

```
/var/chief/
├─ broker/broker.db            # capability grants (SQLite, WAL mode)
├─ memory/                     # memory graph + blobs
├─ oauth/                      # OAuth session sealed storage
├─ provenance/                 # event log + snapshots
├─ secrets/openrouter.key      # mode 0600, sealed storage location
├─ runtime/agents|packs        # reserved for pack runtime
└─ bus/                        # event bus state
```

Remove the volume to reset first-run behavior (the kernel principal rebootstraps and grant state clears).

## Troubleshooting

| Symptom | Fix |
|---|---|
| `ERROR: OPENROUTER_API_KEY env var is required.` | Export the key before `docker run` / `docker compose up`. |
| `listen tcp :8080: address already in use` | Either free 8080 or map a different host port: `-p 9000:8080`. |
| Brief stays at `"Good morning."` after 60s | Inspect logs: `docker logs chief-os-demo`. Common causes: OpenRouter 401 (bad key), rate limit, network block. |
| File-watcher card never appears | Check `/workspace` volume is mounted and has `.md` or `.txt` files. |
| 404 on `/` | Bundle not copied — rebuild with `--no-cache`. The Dockerfile stage 2 must produce `/src/dist/index.html`. |

## How to reset

```bash
docker compose -f deploy/docker/docker-compose.yml down
rm -rf chief-state workspace
docker compose -f deploy/docker/docker-compose.yml up --build
```

## See also

- `crates/chief-core/src/bin/demo_run.rs` — the orchestrator source (commented)
- `deploy/docker/Dockerfile` — three-stage build (rust + node + slim runtime)
- `docs/16-pack-sdk.md` — pack capability model
- `docs/06-security-model.md` — why the Ceremony is load-bearing
