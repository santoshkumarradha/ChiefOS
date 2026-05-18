#!/bin/bash
# scripts/poc6-downloads-steward.sh — real local-folder Chief app E2E gate.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
PORT="${CHIEF_POC6_PORT:-18089}"
BASE_URL="http://127.0.0.1:$PORT"
WORK_ID="${CHIEF_WORK_ID:-downloads-steward-demo}"
APP_PRINCIPAL="${CHIEF_APP_PRINCIPAL:-app:downloads-steward}"
TMP_DIR="$(mktemp -d)"
DEMO_DIR="$TMP_DIR/Downloads Mess"
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

print_tree() {
  local title="$1"
  echo
  echo "$title"
  python3 - "$DEMO_DIR" <<'PY'
import os
import sys

root = sys.argv[1]
for dirpath, dirnames, filenames in os.walk(root):
    dirnames.sort()
    filenames.sort()
    rel = os.path.relpath(dirpath, root)
    indent = "" if rel == "." else "  " * len(rel.split(os.sep))
    label = os.path.basename(root) if rel == "." else os.path.basename(dirpath)
    print(f"{indent}{label}/")
    for name in filenames:
        print(f"{indent}  {name}")
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
  echo "OPENROUTER_API_KEY is required in the environment for this live filesystem POC." >&2
  exit 1
fi

cd "$PROJECT_ROOT"

echo "==> POC 6: Downloads Steward — real local-folder Chief app"
echo "This demo creates a temporary messy folder, uses Chief OS primitives to"
echo "scan it, asks a real LLM through Chief to plan cleanup, locks the exact"
echo "move manifest behind Ceremony, applies real file moves, then rewinds them."
echo
echo "App install/build:"
echo "  npm ci --prefix packages/chief-sdk-ts"
echo "  npm run build --prefix packages/chief-sdk-ts"
echo "  npm install --no-package-lock --prefix examples/downloads-steward-ts"
echo "  npm run build --prefix examples/downloads-steward-ts"
echo
echo "Port: $PORT"
echo "Work Object: $WORK_ID"
echo "App principal: $APP_PRINCIPAL"
echo "OpenRouter key: present in environment; not printed"

mkdir -p "$DEMO_DIR"
cat >"$DEMO_DIR/receipt_4381.txt" <<'EOF'
Coffee shop receipt total $18.42 paid by card.
EOF
cat >"$DEMO_DIR/acme-contract-final-v3.txt" <<'EOF'
Acme contract draft. Clause 4 requires human confirmation before external send.
EOF
cat >"$DEMO_DIR/bank_statement_april.txt" <<'EOF'
Bank statement for April. Ending balance redacted.
EOF
cat >"$DEMO_DIR/Screenshot 2026-05-18.png" <<'EOF'
fake png bytes for demo
EOF
cat >"$DEMO_DIR/random.zip" <<'EOF'
unknown archive payload
EOF

print_tree "Initial real folder"

echo
echo "Installing and building the SDK app..."
npm ci --prefix "$PROJECT_ROOT/packages/chief-sdk-ts" >/dev/null
npm run build --prefix "$PROJECT_ROOT/packages/chief-sdk-ts" >/dev/null
rm -rf "$PROJECT_ROOT/examples/downloads-steward-ts/node_modules/@chief-os/sdk"
npm install --no-package-lock --prefix "$PROJECT_ROOT/examples/downloads-steward-ts" >/dev/null
npm run build --prefix "$PROJECT_ROOT/examples/downloads-steward-ts" >/dev/null

echo
echo "Starting Chief Core as a headless OS node..."
CHIEF_HOME="$TMP_DIR/state" \
  cargo run -q -p chief-core --bin chief-core -- --bind "127.0.0.1:$PORT" --dev \
  >"$TMP_DIR/chief.log" 2>&1 &
CHIEF_PID="$!"

wait_for_status

echo
echo "Running Downloads Steward app through @chief-os/sdk..."
CHIEF_BASE_URL="$BASE_URL" \
CHIEF_WORK_ID="$WORK_ID" \
CHIEF_APP_PRINCIPAL="$APP_PRINCIPAL" \
CHIEF_DOWNLOADS_ROOT="$DEMO_DIR" \
CHIEF_DOWNLOADS_MODE="apply" \
CHIEF_DOWNLOADS_REWIND="1" \
  npm run start --prefix "$PROJECT_ROOT/examples/downloads-steward-ts" --silent \
  >"$TMP_DIR/app-output.json"

echo
echo "App output summary:"
python3 - "$TMP_DIR/app-output.json" <<'PY'
import json
import sys

data = json.load(open(sys.argv[1]))
print(f"  provider       : {data['llm']['provider']}")
print(f"  model          : {data['llm']['model']}")
print(f"  scanned files  : {data['scanned_files']}")
print(f"  planned moves  : {len(data['plan']['operations'])}")
print(f"  ceremony       : {data['ceremony']['id']}")
print(f"  payload hash   : {data['ceremony'].get('payload_hash')}")
print(f"  applied moves  : {data.get('applied', {}).get('count')}")
print(f"  rewound moves  : {data.get('rewind', {}).get('count')}")
print("  proposed operations:")
for op in data["plan"]["operations"]:
    print(f"    - {op['from']} -> {op['to']}")
PY

require_json_field "$TMP_DIR/app-output.json" "\"app\"[[:space:]]*:[[:space:]]*\"$APP_PRINCIPAL\"" "app principal"
require_json_field "$TMP_DIR/app-output.json" '"provider"[[:space:]]*:[[:space:]]*"openrouter"' "real OpenRouter provider"
require_json_field "$TMP_DIR/app-output.json" '"payload_hash"[[:space:]]*:[[:space:]]*"blake3:' "Ceremony payload hash"
require_json_field "$TMP_DIR/app-output.json" '"applied"[[:space:]]*:' "real filesystem apply receipt"
require_json_field "$TMP_DIR/app-output.json" '"rewind"[[:space:]]*:' "real filesystem rewind receipt"

CEREMONY_ID="$(json_field "$TMP_DIR/app-output.json" "ceremony.id")"

print_tree "Folder after apply + rewind"

echo
echo "Checking Chief OS state..."
curl -fsS "$BASE_URL/v1/work/$WORK_ID" >"$TMP_DIR/work.json"
curl -fsS "$BASE_URL/v1/work/$WORK_ID/provenance" >"$TMP_DIR/provenance.json"
curl -fsS "$BASE_URL/v1/ceremony/$CEREMONY_ID" >"$TMP_DIR/ceremony.json"

require_json_field "$TMP_DIR/work.json" "\"source\"[[:space:]]*:[[:space:]]*\"$APP_PRINCIPAL\"" "Work Object app contribution"
require_json_field "$TMP_DIR/provenance.json" "\"source\"[[:space:]]*:[[:space:]]*\"$APP_PRINCIPAL\"" "provenance app source"
require_json_field "$TMP_DIR/ceremony.json" '"status"[[:space:]]*:[[:space:]]*"approved"' "approved Ceremony detail"

if grep -R --fixed-strings "${OPENROUTER_API_KEY}" "$PROJECT_ROOT" \
  --exclude-dir .git \
  --exclude-dir node_modules \
  --exclude-dir target \
  --exclude-dir dist \
  >/dev/null 2>&1; then
  echo "OPENROUTER_API_KEY appeared under the repo tree; refusing to pass." >&2
  exit 1
fi

echo
echo "POC 6 Downloads Steward gate passed."
echo "  app      : examples/downloads-steward-ts"
echo "  work     : $BASE_URL/v1/work/$WORK_ID"
echo "  ceremony : $BASE_URL/v1/ceremony/$CEREMONY_ID"
