#!/bin/bash
# scripts/poc5-sdk-ts-chief-app.sh — clean SDK-only TypeScript app gate.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
PORT="${CHIEF_POC5_PORT:-18088}"
BASE_URL="http://127.0.0.1:$PORT"
WORK_ID="${CHIEF_POC_WORK_ID:-acme-follow-up}"
APP_PRINCIPAL="${CHIEF_APP_PRINCIPAL:-app:acme-chief-ts}"
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

if [ -z "${OPENROUTER_API_KEY:-}" ]; then
  echo "OPENROUTER_API_KEY is required in the environment for this live SDK app POC." >&2
  exit 1
fi

cd "$PROJECT_ROOT"

echo "==> POC 5: SDK-only TypeScript Chief app with real LLM"
echo "Port: $PORT"
echo "Work Object: $WORK_ID"
echo "App principal: $APP_PRINCIPAL"
echo "OpenRouter key: present in environment; not printed"

echo "Building TypeScript SDK and app..."
npm ci --prefix "$PROJECT_ROOT/packages/chief-sdk-ts" >/dev/null
npm run build --prefix "$PROJECT_ROOT/packages/chief-sdk-ts" >/dev/null
rm -rf "$PROJECT_ROOT/examples/acme-chief-app-ts/node_modules/@chief-os/sdk"
npm install --no-package-lock --prefix "$PROJECT_ROOT/examples/acme-chief-app-ts" >/dev/null
npm run build --prefix "$PROJECT_ROOT/examples/acme-chief-app-ts" >/dev/null

CHIEF_HOME="$TMP_DIR/state" \
CHIEF_BIND="127.0.0.1:$PORT" \
CHIEF_OS_DIST_PATH="${CHIEF_OS_DIST_PATH:-$PROJECT_ROOT/apps/chief-brief-ui/dist}" \
  cargo run -p chief-core --bin platform_demo_run >"$TMP_DIR/chief.log" 2>&1 &
CHIEF_PID="$!"

wait_for_status

echo "Running SDK-only app..."
CHIEF_BASE_URL="$BASE_URL" \
CHIEF_WORK_ID="$WORK_ID" \
CHIEF_APP_PRINCIPAL="$APP_PRINCIPAL" \
  npm run start --prefix "$PROJECT_ROOT/examples/acme-chief-app-ts" --silent >"$TMP_DIR/app-output.json"

CEREMONY_ID="$(json_field "$TMP_DIR/app-output.json" "ceremony.id")"
PAYLOAD_HASH="$(json_field "$TMP_DIR/app-output.json" "ceremony.payload_hash")"

require_json_field "$TMP_DIR/app-output.json" "\"app\"[[:space:]]*:[[:space:]]*\"$APP_PRINCIPAL\"" "SDK app principal"
require_json_field "$TMP_DIR/app-output.json" '"provider"[[:space:]]*:[[:space:]]*"openrouter"' "real OpenRouter provider"
require_json_field "$TMP_DIR/app-output.json" '"selected_agents"[[:space:]]*:' "LLM selected app agents"
require_json_field "$TMP_DIR/app-output.json" '"contribution_uri"[[:space:]]*:' "Chief contribution URI"
require_json_field "$TMP_DIR/app-output.json" '"payload_hash"[[:space:]]*:[[:space:]]*"blake3:' "Ceremony payload hash"

echo "Checking Chief substrate projections..."
curl -fsS "$BASE_URL/v1/work/$WORK_ID" >"$TMP_DIR/work-http.json"
curl -fsS "$BASE_URL/v1/work/$WORK_ID/provenance" >"$TMP_DIR/provenance-http.json"
CHIEF_CORE_URL="$BASE_URL" cargo run -q -p chief-cli -- ceremony list --json >"$TMP_DIR/ceremony-cli.json"

require_json_field "$TMP_DIR/work-http.json" "\"source\"[[:space:]]*:[[:space:]]*\"$APP_PRINCIPAL\"" "HTTP app contribution source"
require_json_field "$TMP_DIR/provenance-http.json" "\"source\"[[:space:]]*:[[:space:]]*\"$APP_PRINCIPAL\"" "provenance app source"
require_json_field "$TMP_DIR/ceremony-cli.json" "$CEREMONY_ID" "CLI pending Ceremony id"

echo "Approving exact SDK app payload hash..."
curl -fsS \
  -H 'content-type: application/json' \
  -d "{\"held_ms\":3000,\"payload_hash\":\"$PAYLOAD_HASH\"}" \
  "$BASE_URL/v1/ceremony/$CEREMONY_ID/approve" >"$TMP_DIR/approve.json"
require_json_field "$TMP_DIR/approve.json" '"status"[[:space:]]*:[[:space:]]*"approved"' "approved Ceremony status"
require_json_field "$TMP_DIR/approve.json" "\"target_principal\"[[:space:]]*:[[:space:]]*\"$APP_PRINCIPAL\"" "approved app target principal"

if grep -R --fixed-strings "${OPENROUTER_API_KEY}" "$PROJECT_ROOT" \
  --exclude-dir .git \
  --exclude-dir node_modules \
  --exclude-dir target \
  --exclude-dir dist \
  >/dev/null 2>&1; then
  echo "OPENROUTER_API_KEY appeared under the repo tree; refusing to pass." >&2
  exit 1
fi

echo "POC 5 SDK-only TypeScript app gate passed."
echo "  app      : examples/acme-chief-app-ts"
echo "  ceremony : $BASE_URL/v1/ceremony/$CEREMONY_ID"
echo "  work     : $BASE_URL/v1/work/$WORK_ID"
