#!/bin/bash
# scripts/demo-docker.sh — deterministic Platform MVP Docker demo.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
COMPOSE_FILE="$PROJECT_ROOT/deploy/docker/docker-compose.yml"
PORT="${CHIEF_DEMO_PORT:-8080}"

cd "$PROJECT_ROOT"

echo "==> Chief OS Platform MVP Docker Demo"
CHIEF_DEMO_PORT="$PORT" docker compose -f "$COMPOSE_FILE" up -d --build

echo "Waiting for /status..."
for i in $(seq 1 60); do
  if curl -fsS "http://localhost:$PORT/status" >/dev/null 2>&1; then
    break
  fi
  if [ "$i" = "60" ]; then
    docker compose -f "$COMPOSE_FILE" logs chief-os-demo
    exit 1
  fi
  sleep 1
done

echo "Checking Work Object projection..."
curl -fsS "http://localhost:$PORT/v1/work/acme-follow-up" >/dev/null

echo "Checking CLI parity..."
docker compose -f "$COMPOSE_FILE" exec -T chief-os-demo \
  chief work show acme-follow-up --json >/dev/null

echo "Demo ready:"
echo "  UI:   http://localhost:$PORT"
echo "  API:  http://localhost:$PORT/v1/work/acme-follow-up"
echo "  Logs: docker compose -f deploy/docker/docker-compose.yml logs -f chief-os-demo"
