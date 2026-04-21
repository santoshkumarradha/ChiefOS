#!/usr/bin/env bash
# install-hooks.sh — point git at .githooks/ for this clone.
#
# Run once per fresh clone (or per new worktree if you want hooks there too).
# Sets repo-wide config: core.hooksPath = .githooks

set -euo pipefail

ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT"

chmod +x "$ROOT/.githooks/"* 2>/dev/null || true

git config core.hooksPath .githooks

installed="$(git config core.hooksPath)"
echo "install-hooks: git core.hooksPath = $installed"
echo "install-hooks: hooks now active for this clone."
echo ""
echo "Active hooks:"
ls -1 "$ROOT/.githooks/" | sed 's/^/  /'
