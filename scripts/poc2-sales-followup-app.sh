#!/bin/bash
# scripts/poc2-sales-followup-app.sh — POC 2 external Chief app gate.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
PORT="${CHIEF_POC2_PORT:-18083}"
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

echo "==> POC 2: External sales-followup Chief app"
echo "Port: $PORT"
echo "Work Object: $WORK_ID"

CHIEF_HOME="$TMP_DIR/state" \
CHIEF_BIND="127.0.0.1:$PORT" \
CHIEF_OS_DIST_PATH="${CHIEF_OS_DIST_PATH:-$PROJECT_ROOT/apps/chief-brief-ui/dist}" \
  cargo run -p chief-core --bin platform_demo_run >"$TMP_DIR/chief.log" 2>&1 &
CHIEF_PID="$!"

wait_for_status
require_json_field "$TMP_DIR/status.json" '"mode"[[:space:]]*:[[:space:]]*"headless_chief_node"' "headless node mode"

echo "Running external app process..."
CHIEF_BASE_URL="$BASE_URL" \
CHIEF_WORK_ID="$WORK_ID" \
CHIEF_APP_PRINCIPAL="app:sales-followup" \
  python3 "$PROJECT_ROOT/examples/sales-followup-app/app.py" >"$TMP_DIR/app-output.json"

require_json_field "$TMP_DIR/app-output.json" '"source"[[:space:]]*:[[:space:]]*"app:sales-followup"' "app principal attribution"
require_json_field "$TMP_DIR/app-output.json" '"kind"[[:space:]]*:[[:space:]]*"next_best_action"' "app contribution kind"

echo "Checking HTTP Work Object projection..."
curl -fsS "$BASE_URL/v1/work/$WORK_ID" >"$TMP_DIR/work-http.json"
require_json_field "$TMP_DIR/work-http.json" '"source"[[:space:]]*:[[:space:]]*"app:sales-followup"' "HTTP app contribution source"
require_json_field "$TMP_DIR/work-http.json" '"title"[[:space:]]*:[[:space:]]*"Call Acme before sending the draft"' "HTTP app contribution title"

echo "Checking HTTP provenance projection..."
curl -fsS "$BASE_URL/v1/work/$WORK_ID/provenance" >"$TMP_DIR/provenance-http.json"
require_json_field "$TMP_DIR/provenance-http.json" '"kind"[[:space:]]*:[[:space:]]*"contribution_written"' "provenance contribution row"
require_json_field "$TMP_DIR/provenance-http.json" '"source"[[:space:]]*:[[:space:]]*"app:sales-followup"' "provenance app source"

echo "Checking CLI Work Object parity..."
CHIEF_CORE_URL="$BASE_URL" cargo run -q -p chief-cli -- work show "$WORK_ID" --json >"$TMP_DIR/work-cli.json"
require_json_field "$TMP_DIR/work-cli.json" '"source"[[:space:]]*:[[:space:]]*"app:sales-followup"' "CLI app contribution source"

echo "POC 2 sales-followup app gate passed."
echo "  app        : examples/sales-followup-app/app.py"
echo "  work       : $BASE_URL/v1/work/$WORK_ID"
echo "  provenance : $BASE_URL/v1/work/$WORK_ID/provenance"
