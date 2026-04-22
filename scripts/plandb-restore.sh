#!/usr/bin/env bash
# plandb-restore.sh — rebuild .plandb.db from .plandb/state.sql.
#
# When to run:
#   - Fresh clone (no .plandb.db exists).
#   - Something broke the local db and you want to reset to the repo's
#     canonical state.
#   - After a pull that changed .plandb/state.sql and you want the
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
SRC="${ROOT}/.plandb/state.sql"
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

# FTS5 virtual tables survive .dump as regular table declarations + orphaned
# shadow tables (_data/_idx/_docsize/_config). The restore leaves the vtable
# entry in sqlite_master pointing at shadow tables that never match the
# fts5 file format, so every subsequent query against learnings_fts or
# tasks_fts errors with "vtable constructor failed". Fix: drop the stale
# declarations via writable_schema, VACUUM to clean orphans, recreate the
# vtables cleanly, then rebuild from content. This runs every restore so
# the db is immediately usable for `plandb context`, `plandb search`, etc.
sqlite3 "$DB" <<'SQL'
PRAGMA writable_schema=ON;
DELETE FROM sqlite_master WHERE name IN (
  'learnings_fts','tasks_fts',
  'learnings_fts_data','learnings_fts_idx','learnings_fts_docsize','learnings_fts_config',
  'tasks_fts_data','tasks_fts_idx','tasks_fts_docsize','tasks_fts_config'
);
PRAGMA writable_schema=OFF;
VACUUM;
CREATE VIRTUAL TABLE learnings_fts USING fts5(content, kind, content='learnings', content_rowid='rowid');
CREATE VIRTUAL TABLE tasks_fts     USING fts5(title, description, content='tasks', content_rowid='rowid');
INSERT INTO learnings_fts(learnings_fts) VALUES('rebuild');
INSERT INTO tasks_fts(tasks_fts)         VALUES('rebuild');
SQL

echo "plandb-restore: rebuilt $DB from $SRC"
