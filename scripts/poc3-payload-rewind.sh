#!/bin/bash
# scripts/poc3-payload-rewind.sh — POC 3B payload-bound approval + rewind gate.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
PORT="${CHIEF_POC3B_PORT:-18085}"
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

assert_rewind_state() {
  local work_file="$1"
  local provenance_file="$2"
  local contribution_uri="$3"
  python3 - "$work_file" "$provenance_file" "$contribution_uri" <<'PY'
import json
import sys

work = json.load(open(sys.argv[1]))
provenance = json.load(open(sys.argv[2]))
uri = sys.argv[3]

active = [node for node in work["contributions"] if node["uri"] == uri]
if active:
    raise SystemExit(f"rewound contribution still active: {active}")

rewind = [
    row for row in provenance
    if row.get("kind") == "rewind_event" and row.get("body", {}).get("target_uri") == uri
]
if not rewind:
    raise SystemExit(f"missing rewind_event for {uri}: {provenance}")
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

echo "==> POC 3B: Payload-bound approval + rewind"
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

CEREMONY_ID="$(json_field "$TMP_DIR/app-output.json" "ceremony.id")"
PAYLOAD_HASH="$(json_field "$TMP_DIR/app-output.json" "ceremony.payload_hash")"
CONTRIBUTION_URI="$(json_field "$TMP_DIR/app-output.json" "contribution.uri")"

echo "Rejecting mutated payload hash..."
MUTATED_HASH="${PAYLOAD_HASH%?}0"
if [ "$MUTATED_HASH" = "$PAYLOAD_HASH" ]; then
  MUTATED_HASH="${PAYLOAD_HASH%?}1"
fi
HTTP_STATUS="$(curl -sS -o "$TMP_DIR/mismatch.json" -w '%{http_code}' \
  -H 'content-type: application/json' \
  -d "{\"held_ms\":3000,\"payload_hash\":\"$MUTATED_HASH\"}" \
  "$BASE_URL/v1/ceremony/$CEREMONY_ID/approve")"
if [ "$HTTP_STATUS" != "409" ]; then
  echo "Expected payload mismatch HTTP 409, got $HTTP_STATUS" >&2
  cat "$TMP_DIR/mismatch.json" >&2
  exit 1
fi
require_json_field "$TMP_DIR/mismatch.json" '"error"[[:space:]]*:[[:space:]]*"payload_hash_mismatch"' "payload hash mismatch"

echo "Approving exact payload hash..."
curl -fsS \
  -H 'content-type: application/json' \
  -d "{\"held_ms\":3000,\"payload_hash\":\"$PAYLOAD_HASH\"}" \
  "$BASE_URL/v1/ceremony/$CEREMONY_ID/approve" >"$TMP_DIR/approve.json"
require_json_field "$TMP_DIR/approve.json" '"status"[[:space:]]*:[[:space:]]*"approved"' "approved Ceremony status"

echo "Rewinding the external app contribution..."
curl -fsS \
  -H 'content-type: application/json' \
  -d "{\"contribution_uri\":\"$CONTRIBUTION_URI\",\"reason\":\"POC 3B rewind external app contribution\"}" \
  "$BASE_URL/v1/work/$WORK_ID/rewind" >"$TMP_DIR/rewind.json"
require_json_field "$TMP_DIR/rewind.json" '"rewound"[[:space:]]*:[[:space:]]*true' "rewind success"

curl -fsS "$BASE_URL/v1/work/$WORK_ID" >"$TMP_DIR/work-after.json"
curl -fsS "$BASE_URL/v1/work/$WORK_ID/provenance" >"$TMP_DIR/provenance-after.json"
assert_rewind_state "$TMP_DIR/work-after.json" "$TMP_DIR/provenance-after.json" "$CONTRIBUTION_URI"

echo "Checking CLI Work Object after rewind..."
CHIEF_CORE_URL="$BASE_URL" cargo run -q -p chief-cli -- work show "$WORK_ID" --json >"$TMP_DIR/work-cli.json"
require_json_field "$TMP_DIR/work-cli.json" '"kind"[[:space:]]*:[[:space:]]*"rewind_event"' "CLI replayable rewind event"

echo "POC 3B payload + rewind gate passed."
echo "  contribution: $CONTRIBUTION_URI"
echo "  ceremony    : $BASE_URL/v1/ceremony/$CEREMONY_ID"
echo "  work        : $BASE_URL/v1/work/$WORK_ID"
