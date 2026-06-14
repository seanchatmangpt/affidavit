#!/usr/bin/env bash
#
# web-dev.sh — start the Next.js dev server for the self-contained web app.
#
# The web app under web/ is fully self-contained (Node 22, Next.js 15) and
# needs none of the sibling Rust crates. This is the fastest inner loop in
# the repo.
#
# Usage:  scripts/web-dev.sh        (run from anywhere)
#         scripts/web-dev.sh -p 4000  (extra args pass through to `next dev`)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

if ! command -v npm >/dev/null 2>&1; then
  echo "ERROR: npm not found. Install Node 22 (see scripts/bootstrap.sh)." >&2
  exit 1
fi

if [ ! -d "$REPO_ROOT/web/node_modules" ]; then
  echo "==> web/node_modules missing; installing first (npm ci)"
  ( cd "$REPO_ROOT/web" && npm ci )
fi

echo "==> starting Next.js dev server (cd web && npm run dev)"
echo "    press Ctrl-C to stop."
cd "$REPO_ROOT/web"
exec npm run dev -- "$@"
