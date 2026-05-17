#!/bin/sh
# Chief OS demo entrypoint.
# Prepares state dirs and execs the deterministic platform demo binary.
set -e

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
echo "  inspect: chief work show acme-follow-up --json"
echo ""
echo "platform-demo : seeding Acme Work Object and running POC packs"
echo ""

# ────────────────────────────────────────────────────────────────────
# Exec the binary (PID 1 so SIGTERM routes correctly).
# ────────────────────────────────────────────────────────────────────
exec /usr/local/bin/chief-os-demo "$@"
