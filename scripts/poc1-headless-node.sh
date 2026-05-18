#!/bin/bash
# scripts/poc1-headless-node.sh — POC 1 headless Chief Node operator gate.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
COMPOSE_FILE="$PROJECT_ROOT/deploy/docker/docker-compose.yml"
PORT="${CHIEF_DEMO_PORT:-18081}"
BASE_URL="http://localhost:$PORT"
WORK_ID="${CHIEF_POC_WORK_ID:-acme-follow-up}"
TMP_DIR="$(mktemp -d)"

cleanup() {
  rm -rf "$TMP_DIR"
}
trap cleanup EXIT

compose() {
  CHIEF_DEMO_PORT="$PORT" docker compose -f "$COMPOSE_FILE" "$@"
}

wait_for_status() {
  echo "Waiting for $BASE_URL/status..."
  for i in $(seq 1 90); do
    if curl -fsS "$BASE_URL/status" >"$TMP_DIR/status.json" 2>/dev/null; then
      return 0
    fi
    if [ "$i" = "90" ]; then
      compose logs chief-os-demo
      return 1
    fi
    sleep 1
  done
}

require_json_field() {
  local file="$1"
  local pattern="$2"
  local description="$3"
  if ! grep -Eq "$pattern" "$file"; then
    echo "Missing expected $description in $file" >&2
    echo "--- $file ---" >&2
    cat "$file" >&2
    exit 1
  fi
}

cd "$PROJECT_ROOT"

echo "==> POC 1: Headless Chief Node"
echo "Port: $PORT"
echo "Work Object: $WORK_ID"

compose up -d --build
wait_for_status

echo "Checking HTTP Work Object projection..."
curl -fsS "$BASE_URL/v1/work/$WORK_ID" >"$TMP_DIR/work-http.json"
require_json_field "$TMP_DIR/work-http.json" '"id"[[:space:]]*:[[:space:]]*"acme-follow-up"' "Work Object id"

echo "Checking HTTP provenance projection..."
curl -fsS "$BASE_URL/v1/work/$WORK_ID/provenance" >"$TMP_DIR/provenance-http.json"
require_json_field "$TMP_DIR/provenance-http.json" 'document-pack|calendar-pack|email-pack|risk-pack|chief-core' "provenance producer"

echo "Checking HTTP inbox projection..."
curl -fsS "$BASE_URL/v1/inbox" >"$TMP_DIR/inbox-http.json"
require_json_field "$TMP_DIR/inbox-http.json" 'ceremony|approval|acme|email' "pending inbox/Ceremony item"

echo "Checking CLI Work Object parity..."
compose exec -T chief-os-demo \
  chief work show "$WORK_ID" --json >"$TMP_DIR/work-cli.json"
require_json_field "$TMP_DIR/work-cli.json" '"id"[[:space:]]*:[[:space:]]*"acme-follow-up"' "CLI Work Object id"

echo "Checking no separate Node/NPM UI runtime is required..."
compose exec -T chief-os-demo sh -lc '
  if command -v node >/dev/null 2>&1 || command -v npm >/dev/null 2>&1; then
    echo "unexpected Node/NPM runtime found in headless runtime image" >&2
    exit 1
  fi
'

echo "POC 1 headless node gate passed."
echo "  status     : $BASE_URL/status"
echo "  work       : $BASE_URL/v1/work/$WORK_ID"
echo "  provenance : $BASE_URL/v1/work/$WORK_ID/provenance"
echo "  inbox      : $BASE_URL/v1/inbox"
echo "  cli        : CHIEF_CORE_URL=$BASE_URL chief work show $WORK_ID --json"
