#!/usr/bin/env bash
# Detect the test framework for a project directory.
# Usage: detect-test-runner.sh <project-root>
# Outputs the test command to stdout.
# Exit 1 if no framework detected.

set -euo pipefail

PROJECT_ROOT="${1:-.}"

if [[ -f "$PROJECT_ROOT/Cargo.toml" ]]; then
  echo "cargo test"
  exit 0
fi

if [[ -f "$PROJECT_ROOT/package.json" ]]; then
  if grep -q '"vitest"' "$PROJECT_ROOT/package.json" 2>/dev/null; then
    echo "npx vitest run"
    exit 0
  fi
  if grep -q '"jest"' "$PROJECT_ROOT/package.json" 2>/dev/null; then
    echo "npx jest"
    exit 0
  fi
fi

if [[ -f "$PROJECT_ROOT/pyproject.toml" ]]; then
  if grep -q 'pytest' "$PROJECT_ROOT/pyproject.toml" 2>/dev/null; then
    echo "pytest"
    exit 0
  fi
fi

if [[ -f "$PROJECT_ROOT/go.mod" ]]; then
  echo "go test ./..."
  exit 0
fi

if [[ -f "$PROJECT_ROOT/Makefile" ]]; then
  if grep -q '^test:' "$PROJECT_ROOT/Makefile" 2>/dev/null; then
    echo "make test"
    exit 0
  fi
fi

echo "unknown"
exit 1
