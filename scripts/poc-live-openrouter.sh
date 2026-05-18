#!/bin/bash
# scripts/poc-live-openrouter.sh — live user-facing POC gate using OpenRouter.

set -euo pipefail

if [ -z "${OPENROUTER_API_KEY:-}" ]; then
  echo "OPENROUTER_API_KEY is required in the environment." >&2
  exit 1
fi

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
PORT="${CHIEF_LIVE_POC_PORT:-18082}"
BIND="127.0.0.1:$PORT"
BASE_URL="http://127.0.0.1:$PORT"
TMP_DIR="$(mktemp -d)"
STATE_DIR="$TMP_DIR/state"
WORKSPACE_DIR="$TMP_DIR/workspace"
LOG_FILE="$TMP_DIR/demo_run.log"
PID=""

cleanup() {
  if [ -n "$PID" ] && kill -0 "$PID" >/dev/null 2>&1; then
    kill "$PID" >/dev/null 2>&1 || true
    wait "$PID" >/dev/null 2>&1 || true
  fi
  rm -rf "$TMP_DIR"
}
trap cleanup EXIT

require_json_field() {
  local file="$1"
  local pattern="$2"
  local description="$3"
  if ! grep -Eiq "$pattern" "$file"; then
    echo "Missing expected $description in $file" >&2
    echo "--- $file ---" >&2
    cat "$file" >&2
    echo "--- live runner log tail ---" >&2
    tail -n 120 "$LOG_FILE" >&2 || true
    exit 1
  fi
}

wait_for_http() {
  echo "Waiting for $BASE_URL/v1/status..."
  for _ in $(seq 1 90); do
    if curl -fsS "$BASE_URL/v1/status" >"$TMP_DIR/status.json" 2>/dev/null; then
      return 0
    fi
    sleep 1
  done
  echo "Timed out waiting for live Chief Node." >&2
  tail -n 120 "$LOG_FILE" >&2 || true
  exit 1
}

wait_for_live_openrouter_card() {
  echo "Waiting for live OpenRouter-backed user-facing card..."
  for _ in $(seq 1 150); do
    curl -fsS "$BASE_URL/v1/inbox" >"$TMP_DIR/inbox.json" 2>/dev/null || true
    if grep -Eiq 'OpenRouter|HN top stories' "$TMP_DIR/inbox.json"; then
      return 0
    fi
    sleep 1
  done
  echo "Timed out waiting for OpenRouter-backed inbox card." >&2
  tail -n 160 "$LOG_FILE" >&2 || true
  exit 1
}

cd "$PROJECT_ROOT"
mkdir -p "$STATE_DIR" "$WORKSPACE_DIR"

cat >"$WORKSPACE_DIR/live-poc-note.md" <<'NOTE'
# Chief OS Live OpenRouter POC

This note exists so the live file watcher has real user workspace content.
The POC should use the OpenRouter-backed live runner, not fixture-only tests.
NOTE

echo "==> Live OpenRouter POC"
echo "Bind: $BIND"
echo "State: $STATE_DIR"
echo "Workspace: $WORKSPACE_DIR"
echo "OpenRouter key: present in environment; not printed"

CHIEF_HOME="$STATE_DIR" \
CHIEF_WORKSPACE="$WORKSPACE_DIR" \
CHIEF_BIND="$BIND" \
CHIEF_HN_SCORE_LIMIT="${CHIEF_HN_SCORE_LIMIT:-2}" \
CHIEF_OS_DIST_PATH="${CHIEF_OS_DIST_PATH:-$PROJECT_ROOT/apps/chief-brief-ui/dist}" \
RUST_LOG="${RUST_LOG:-info,reqwest=warn}" \
cargo run -p chief-core --bin demo_run >"$LOG_FILE" 2>&1 &
PID="$!"

wait_for_http
require_json_field "$TMP_DIR/status.json" '"mode"[[:space:]]*:[[:space:]]*"headless_chief_node"' "headless node mode"

wait_for_live_openrouter_card
require_json_field "$TMP_DIR/inbox.json" 'OpenRouter' "OpenRouter-backed inbox evidence"
require_json_field "$TMP_DIR/inbox.json" 'Approve .*substack|Ceremony|ceremony' "real authority/Ceremony prompt"

curl -fsS "$BASE_URL/v1/brief" >"$TMP_DIR/brief.json"
require_json_field "$TMP_DIR/brief.json" 'OpenRouter|HN top stories|substack|Ceremony' "user-facing brief evidence"

curl -fsS "$BASE_URL/" >"$TMP_DIR/index.html" || true
require_json_field "$TMP_DIR/index.html" '<html|Chief' "user-facing surface HTML"

if grep -R --fixed-strings "${OPENROUTER_API_KEY}" "$PROJECT_ROOT" \
  --exclude-dir .git \
  --exclude-dir target \
  --exclude-dir node_modules \
  --exclude-dir deploy/docker/chief-state \
  --exclude-dir deploy/docker/workspace \
  >/dev/null 2>&1; then
  echo "OPENROUTER_API_KEY appeared under the repo tree; refusing to pass." >&2
  exit 1
fi

echo "Live OpenRouter POC gate passed."
echo "  status : $BASE_URL/v1/status"
echo "  inbox  : $BASE_URL/v1/inbox"
echo "  brief  : $BASE_URL/v1/brief"
echo "  ui     : $BASE_URL/"
