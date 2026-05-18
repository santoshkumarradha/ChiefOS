# Chief OS — Platform MVP Docker Demo

Deterministic phase-zero demo for the OS-level Work Object platform proof.

## POC 1 Headless Node Gate

The next proof treats the Docker runner as a headless Chief Node, not as a UI demo server. It validates the deterministic Work Object over HTTP and CLI without requiring a separate UI process:

```bash
scripts/poc1-headless-node.sh
```

The script defaults to `CHIEF_DEMO_PORT=18081` to avoid common local `8080` conflicts. Override it if needed:

```bash
CHIEF_DEMO_PORT=18082 scripts/poc1-headless-node.sh
```

What it checks:

- `/v1/status` identifies the headless Chief Node mode.
- `/v1/work/acme-follow-up` returns the Work Object projection.
- `/v1/work/acme-follow-up/provenance` returns provenance.
- `/v1/inbox` exposes the pending approval/Ceremony state.
- `chief work show acme-follow-up --json` reads the same node through the CLI.
- The runtime image does not need Node/NPM as a live UI process.

## Live OpenRouter Gate

The deterministic gate above proves the node shape. The live gate uses the real `demo_run` path, real HN data, and real OpenRouter calls from `OPENROUTER_API_KEY` in the environment:

```bash
OPENROUTER_API_KEY=... scripts/poc-live-openrouter.sh
```

The script never prints the key. It uses a temporary `CHIEF_HOME`, removes it on exit, and fails if the key appears under the repo tree.

## Quickstart

```bash
docker compose -f deploy/docker/docker-compose.yml up --build
curl -fsS http://localhost:8080/v1/work/acme-follow-up
```

Open `http://localhost:8080` for the Work Object surface.

If `8080` is busy:

```bash
CHIEF_DEMO_PORT=18081 docker compose -f deploy/docker/docker-compose.yml up --build
curl -fsS http://localhost:18081/v1/work/acme-follow-up
```

## What Runs

On boot, `platform_demo_run`:

- Creates the `mem://artifact/...` Work Object for `acme-follow-up`.
- Seeds the contract, calendar, and prior-email fixtures.
- Runs `document-pack`, `calendar-pack`, `email-pack`, and `risk-pack` through `chief-sdk`.
- Opens a pending Ceremony for the exact email draft payload.
- Serves the React surface and `/v1/*` API on port `8080`.

## Inspection

```bash
curl -fsS http://localhost:8080/v1/work/acme-follow-up
docker compose -f deploy/docker/docker-compose.yml exec chief-os-demo \
  chief work show acme-follow-up --json
```

Both paths read the same `/v1/work/:id` projection.

## Reset

```bash
docker compose -f deploy/docker/docker-compose.yml down
rm -rf deploy/docker/chief-state deploy/docker/workspace
docker compose -f deploy/docker/docker-compose.yml up --build
```

No API keys are required for this deterministic POC.
