#!/usr/bin/env bash
# plandb-restore.sh — rebuild .plandb.db from docs/plandb-state.sql.
#
# When to run:
#   - Fresh clone (no .plandb.db exists).
#   - Something broke the local db and you want to reset to the repo's
#     canonical state.
#   - After a pull that changed docs/plandb-state.sql and you want the
#     binary db to match.
#
# Safe: refuses to overwrite an existing .plandb.db unless --force is passed.
#
# Usage:
#   scripts/plandb-restore.sh
#   scripts/plandb-restore.sh --force

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DB="${ROOT}/.plandb.db"
SRC="${ROOT}/docs/plandb-state.sql"
FORCE=0

for arg in "$@"; do
  case "$arg" in
    --force) FORCE=1 ;;
    -h|--help)
      sed -n '2,13p' "$0"
      exit 0
      ;;
    *)
      echo "plandb-restore: unknown arg: $arg" >&2
      exit 2
      ;;
  esac
done

if [ ! -f "$SRC" ]; then
  echo "plandb-restore: source missing: $SRC" >&2
  exit 1
fi

if [ -f "$DB" ] && [ "$FORCE" -ne 1 ]; then
  echo "plandb-restore: refusing to overwrite existing $DB"
  echo "  back up or pass --force to replace."
  exit 1
fi

# Stash existing db as .bak just in case.
if [ -f "$DB" ]; then
  cp "$DB" "$DB.bak.$(date +%s)"
fi

rm -f "$DB"
sqlite3 "$DB" < "$SRC"

echo "plandb-restore: rebuilt $DB from $SRC"
