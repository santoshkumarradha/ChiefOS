# Chief OS — Platform MVP Docker Demo

Deterministic phase-zero demo for the OS-level Work Object platform proof.

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
