#!/bin/sh
# Chief OS demo entrypoint.
# Validates required env, prepares state dirs, execs the demo binary.
set -e

# ────────────────────────────────────────────────────────────────────
# Required env
# ────────────────────────────────────────────────────────────────────
if [ -z "${OPENROUTER_API_KEY:-}" ]; then
  echo ""
  echo "ERROR: OPENROUTER_API_KEY env var is required."
  echo ""
  echo "Example:"
  echo "  docker run -p 8080:8080 \\"
  echo "             -e OPENROUTER_API_KEY=sk-or-... \\"
  echo "             -v \$(pwd)/chief-state:/var/chief \\"
  echo "             chief-os-demo:latest"
  echo ""
  echo "Get a key: https://openrouter.ai/keys"
  echo ""
  exit 1
fi

# ────────────────────────────────────────────────────────────────────
# State directories
# ────────────────────────────────────────────────────────────────────
CHIEF_HOME="${CHIEF_HOME:-/var/chief}"
mkdir -p \
  "$CHIEF_HOME/broker" \
  "$CHIEF_HOME/memory/blobs" \
  "$CHIEF_HOME/oauth" \
  "$CHIEF_HOME/provenance/snapshots" \
  "$CHIEF_HOME/trust" \
  "$CHIEF_HOME/runtime/agents" \
  "$CHIEF_HOME/runtime/packs" \
  "$CHIEF_HOME/bus" \
  "$CHIEF_HOME/secrets"
chmod 700 "$CHIEF_HOME/secrets" 2>/dev/null || true

# Workspace volume — the file-watcher pack scans this directory.
mkdir -p /workspace

# ────────────────────────────────────────────────────────────────────
# Preflight log
# ────────────────────────────────────────────────────────────────────
echo "chief-os-demo: starting"
echo "  bind   : 0.0.0.0:8080"
echo "  state  : $CHIEF_HOME"
echo "  dist   : ${CHIEF_OS_DIST_PATH:-/usr/local/share/chief-os/dist}"
echo "  browse : http://localhost:8080"
echo ""
echo "hn-briefer : running immediately, re-runs every 15 min"
echo "file-watch : scanning /workspace every 5 min"
echo ""

# ────────────────────────────────────────────────────────────────────
# Exec the binary (PID 1 so SIGTERM routes correctly).
# ────────────────────────────────────────────────────────────────────
exec /usr/local/bin/chief-os-demo "$@"
