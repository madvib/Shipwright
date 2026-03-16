#!/usr/bin/env bash
# prune-stale.sh [idle-hours]
# Prunes git worktrees under .ship/worktrees/ idle longer than N hours.
# Default: 24 hours.

set -euo pipefail

IDLE_HOURS="${1:-24}"
WORKTREES_DIR=".ship/worktrees"
PRUNED=0
SKIPPED=0

if [[ ! -d "$WORKTREES_DIR" ]]; then
  echo "No worktrees directory found at $WORKTREES_DIR"
  exit 0
fi

NOW=$(date +%s)
THRESHOLD=$(( NOW - IDLE_HOURS * 3600 ))

echo "Checking worktrees idle > ${IDLE_HOURS}h..."

for worktree in "$WORKTREES_DIR"/*/; do
  [[ -d "$worktree" ]] || continue
  branch=$(basename "$worktree")

  # Get last modified time of the worktree directory
  if [[ "$(uname)" == "Darwin" ]]; then
    MTIME=$(stat -f %m "$worktree")
  else
    MTIME=$(stat -c %Y "$worktree")
  fi

  if (( MTIME < THRESHOLD )); then
    echo "  pruning: $branch (idle $(( (NOW - MTIME) / 3600 ))h)"
    git worktree remove "$worktree" --force 2>/dev/null && PRUNED=$(( PRUNED + 1 )) || {
      echo "  warning: could not remove $worktree" >&2
      SKIPPED=$(( SKIPPED + 1 ))
    }
  else
    echo "  keeping: $branch (idle $(( (NOW - MTIME) / 3600 ))h)"
    SKIPPED=$(( SKIPPED + 1 ))
  fi
done

# Clean up any leftover worktree refs
git worktree prune 2>/dev/null || true

echo ""
echo "✓ done — pruned: $PRUNED, kept: $SKIPPED"
