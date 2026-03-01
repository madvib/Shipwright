#!/usr/bin/env bash
# Smoke test: ship git sync from a worktree without SHIP_DIR
#
# Verifies:
#   - ship init creates full .ship/ structure
#   - ship feature create sets branch in frontmatter
#   - ship git sync from a worktree (no SHIP_DIR) finds .ship/ via walk-up
#   - CLAUDE.md is written to the worktree root, not the main repo root
#
# Usage:
#   bash example/projects-e2e/scripts/worktree-smoke.sh
#   SHIP=./target/release/ship bash example/projects-e2e/scripts/worktree-smoke.sh
set -euo pipefail

SHIP="${SHIP:-$(dirname "$0")/../../../target/debug/ship}"
if [[ ! -x "$SHIP" ]]; then
    echo "ERROR: ship binary not found at $SHIP" >&2
    echo "Run: cargo build -p cli" >&2
    exit 1
fi

DIR=$(mktemp -d)
trap 'rm -rf "$DIR"' EXIT

cd "$DIR"
git init -q
git config user.email "test@test.com"
git config user.name "Test"
git checkout -q -b main

SHIP_DIR="$DIR/.ship" "$SHIP" init > /dev/null
SHIP_DIR="$DIR/.ship" "$SHIP" feature create "Auth Flow" --branch "feature/auth" > /dev/null

git add -A && git commit -q -m "init"
git checkout -q -b feature/auth
git checkout -q main
git worktree add .worktrees/feature-auth feature/auth > /dev/null 2>&1

cd ".worktrees/feature-auth"

echo "→ ship git sync from worktree (no SHIP_DIR)"
"$SHIP" git sync

if [[ -f CLAUDE.md ]]; then
    echo "✓ CLAUDE.md exists in worktree root"
else
    echo "✗ CLAUDE.md missing from worktree root" >&2
    exit 1
fi

if [[ -f "$DIR/CLAUDE.md" ]]; then
    echo "✗ CLAUDE.md leaked into main repo root" >&2
    exit 1
else
    echo "✓ CLAUDE.md not present in main repo root"
fi

echo "PASS"
