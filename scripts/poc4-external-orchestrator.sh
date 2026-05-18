#!/bin/bash
# scripts/poc4-external-orchestrator.sh — POC 4A bring-your-own orchestrator gate.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
PORT="${CHIEF_POC4_PORT:-18086}"
BASE_URL="http://127.0.0.1:$PORT"
WORK_ID="${CHIEF_POC_WORK_ID:-acme-follow-up}"
TMP_DIR="$(mktemp -d)"
CHIEF_PID=""

cleanup() {
  if [ -n "$CHIEF_PID" ] && kill -0 "$CHIEF_PID" 2>/dev/null; then
    kill "$CHIEF_PID" 2>/dev/null || true
    wait "$CHIEF_PID" 2>/dev/null || true
  fi
  rm -rf "$TMP_DIR"
}
trap cleanup EXIT

require_json_field() {
  local file="$1"
  local pattern="$2"
  local description="$3"
  if ! grep -Eq "$pattern" "$file"; then
    echo "Missing expected $description in $file" >&2
    echo "--- $file ---" >&2
    cat "$file" >&2
    echo "--- chief.log ---" >&2
    cat "$TMP_DIR/chief.log" >&2 || true
    exit 1
  fi
}

json_field() {
  local file="$1"
  local expr="$2"
  python3 - "$file" "$expr" <<'PY'
import json
import sys

data = json.load(open(sys.argv[1]))
value = data
for part in sys.argv[2].split("."):
    value = value[part] if isinstance(value, dict) else value[int(part)]
print(value)
PY
}

wait_for_status() {
  echo "Waiting for $BASE_URL/v1/status..."
  for i in $(seq 1 90); do
    if curl -fsS "$BASE_URL/v1/status" >"$TMP_DIR/status.json" 2>/dev/null; then
      return 0
    fi
    if [ "$i" = "90" ]; then
      cat "$TMP_DIR/chief.log" >&2 || true
      return 1
    fi
    sleep 1
  done
}

cd "$PROJECT_ROOT"

echo "==> POC 4A: External orchestrator on Chief substrate"
echo "Port: $PORT"
echo "Work Object: $WORK_ID"

CHIEF_HOME="$TMP_DIR/state" \
CHIEF_BIND="127.0.0.1:$PORT" \
CHIEF_OS_DIST_PATH="${CHIEF_OS_DIST_PATH:-$PROJECT_ROOT/apps/chief-brief-ui/dist}" \
  cargo run -p chief-core --bin platform_demo_run >"$TMP_DIR/chief.log" 2>&1 &
CHIEF_PID="$!"

wait_for_status

echo "Running external orchestrator process..."
CHIEF_BASE_URL="$BASE_URL" \
CHIEF_WORK_ID="$WORK_ID" \
CHIEF_APP_PRINCIPAL="app:external-orchestrator" \
  python3 "$PROJECT_ROOT/examples/external-orchestrator/app.py" >"$TMP_DIR/orchestrator-output.json"

CEREMONY_ID="$(json_field "$TMP_DIR/orchestrator-output.json" "ceremony.id")"
PAYLOAD_HASH="$(json_field "$TMP_DIR/orchestrator-output.json" "ceremony.payload_hash")"

require_json_field "$TMP_DIR/orchestrator-output.json" '"source"[[:space:]]*:[[:space:]]*"app:external-orchestrator"' "orchestrator source"
require_json_field "$TMP_DIR/orchestrator-output.json" '"orchestrator"[[:space:]]*:[[:space:]]*"external-orchestrator"' "app-owned orchestration plan"
require_json_field "$TMP_DIR/orchestrator-output.json" '"selected_agents"[[:space:]]*:' "selected agents"
require_json_field "$TMP_DIR/orchestrator-output.json" '"authority_state"[[:space:]]*:[[:space:]]*"needs_ceremony"' "Chief authority gate"

echo "Checking Chief substrate projections..."
curl -fsS "$BASE_URL/v1/work/$WORK_ID" >"$TMP_DIR/work-http.json"
curl -fsS "$BASE_URL/v1/work/$WORK_ID/provenance" >"$TMP_DIR/provenance-http.json"
CHIEF_CORE_URL="$BASE_URL" cargo run -q -p chief-cli -- ceremony list --json >"$TMP_DIR/ceremony-cli.json"

require_json_field "$TMP_DIR/work-http.json" '"source"[[:space:]]*:[[:space:]]*"app:external-orchestrator"' "HTTP orchestrator contribution"
require_json_field "$TMP_DIR/provenance-http.json" '"source"[[:space:]]*:[[:space:]]*"app:external-orchestrator"' "provenance orchestrator source"
require_json_field "$TMP_DIR/ceremony-cli.json" "$CEREMONY_ID" "CLI pending Ceremony id"

echo "Approving exact orchestrator payload hash..."
curl -fsS \
  -H 'content-type: application/json' \
  -d "{\"held_ms\":3000,\"payload_hash\":\"$PAYLOAD_HASH\"}" \
  "$BASE_URL/v1/ceremony/$CEREMONY_ID/approve" >"$TMP_DIR/approve.json"
require_json_field "$TMP_DIR/approve.json" '"status"[[:space:]]*:[[:space:]]*"approved"' "approved orchestrator Ceremony"
require_json_field "$TMP_DIR/approve.json" '"target_principal"[[:space:]]*:[[:space:]]*"app:external-orchestrator"' "approved orchestrator target principal"

echo "POC 4A external orchestrator gate passed."
echo "  orchestrator: examples/external-orchestrator/app.py"
echo "  ceremony    : $BASE_URL/v1/ceremony/$CEREMONY_ID"
echo "  work        : $BASE_URL/v1/work/$WORK_ID"
