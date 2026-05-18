#!/bin/bash
# scripts/poc3-external-app-ceremony.sh — POC 3A external app Ceremony gate.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
PORT="${CHIEF_POC3_PORT:-18084}"
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

echo "==> POC 3A: External app headless Ceremony"
echo "Port: $PORT"
echo "Work Object: $WORK_ID"

CHIEF_HOME="$TMP_DIR/state" \
CHIEF_BIND="127.0.0.1:$PORT" \
CHIEF_OS_DIST_PATH="${CHIEF_OS_DIST_PATH:-$PROJECT_ROOT/apps/chief-brief-ui/dist}" \
  cargo run -p chief-core --bin platform_demo_run >"$TMP_DIR/chief.log" 2>&1 &
CHIEF_PID="$!"

wait_for_status

echo "Running external app in Ceremony mode..."
CHIEF_BASE_URL="$BASE_URL" \
CHIEF_WORK_ID="$WORK_ID" \
CHIEF_APP_PRINCIPAL="app:sales-followup" \
CHIEF_APP_MODE="ceremony" \
  python3 "$PROJECT_ROOT/examples/sales-followup-app/app.py" >"$TMP_DIR/app-output.json"

require_json_field "$TMP_DIR/app-output.json" '"authority_state"[[:space:]]*:[[:space:]]*"needs_ceremony"' "needs_ceremony contribution"
require_json_field "$TMP_DIR/app-output.json" '"source_agent"[[:space:]]*:[[:space:]]*"app:sales-followup"' "app Ceremony source"

CEREMONY_ID="$(json_field "$TMP_DIR/app-output.json" "ceremony.id")"
PAYLOAD_HASH="$(json_field "$TMP_DIR/app-output.json" "ceremony.payload_hash")"

echo "Checking HTTP Ceremony queue..."
curl -fsS "$BASE_URL/v1/ceremony" >"$TMP_DIR/ceremony-http.json"
require_json_field "$TMP_DIR/ceremony-http.json" "$CEREMONY_ID" "pending Ceremony id"
require_json_field "$TMP_DIR/ceremony-http.json" '"source_agent"[[:space:]]*:[[:space:]]*"app:sales-followup"' "pending Ceremony source"

echo "Checking CLI Ceremony parity..."
CHIEF_CORE_URL="$BASE_URL" cargo run -q -p chief-cli -- ceremony list --json >"$TMP_DIR/ceremony-cli.json"
require_json_field "$TMP_DIR/ceremony-cli.json" "$CEREMONY_ID" "CLI pending Ceremony id"
require_json_field "$TMP_DIR/ceremony-cli.json" '"source_agent"[[:space:]]*:[[:space:]]*"app:sales-followup"' "CLI pending Ceremony source"

echo "Approving exact payload hash headlessly..."
curl -fsS \
  -H 'content-type: application/json' \
  -d "{\"held_ms\":3000,\"payload_hash\":\"$PAYLOAD_HASH\"}" \
  "$BASE_URL/v1/ceremony/$CEREMONY_ID/approve" >"$TMP_DIR/approve.json"
require_json_field "$TMP_DIR/approve.json" '"status"[[:space:]]*:[[:space:]]*"approved"' "approved Ceremony status"
require_json_field "$TMP_DIR/approve.json" '"target_principal"[[:space:]]*:[[:space:]]*"app:sales-followup"' "approved app target principal"
require_json_field "$TMP_DIR/approve.json" '"grant_issued"[[:space:]]*:' "issued grant"

echo "POC 3A external app Ceremony gate passed."
echo "  ceremony  : $BASE_URL/v1/ceremony/$CEREMONY_ID"
echo "  work       : $BASE_URL/v1/work/$WORK_ID"
