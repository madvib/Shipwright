#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
TMP_ROOT="$ROOT_DIR/example/projects-e2e/.tmp"
RUN_ID="$(date +%Y%m%d%H%M%S)"
WORK_DIR="$TMP_ROOT/project-e2e-$RUN_ID"
HOME_DIR="$WORK_DIR/home"
ORIG_HOME="${HOME:-}"

mkdir -p "$WORK_DIR" "$HOME_DIR"

SHIP_BIN="$ROOT_DIR/target/debug/ship"

assert_contains() {
  local haystack="$1"
  local needle="$2"
  if [[ "$haystack" != *"$needle"* ]]; then
    echo "ASSERTION FAILED: expected output to contain: $needle" >&2
    echo "--- output ---" >&2
    echo "$haystack" >&2
    exit 1
  fi
}

run_ship() {
  (cd "$WORK_DIR" && "$SHIP_BIN" "$@")
}

echo "Building CLI binary..."
HOME="$ORIG_HOME" cargo build --manifest-path "$ROOT_DIR/Cargo.toml" -p cli >/dev/null

export HOME="$HOME_DIR"

echo "Initializing isolated test workspace at $WORK_DIR"
out="$(run_ship init .)"
assert_contains "$out" "Initialized and tracked Ship project"
if [[ ! -d "$WORK_DIR/.ship/features" ]]; then
  echo "ASSERTION FAILED: expected .ship/features directory" >&2
  exit 1
fi
if [[ ! -d "$WORK_DIR/.ship/releases" ]]; then
  echo "ASSERTION FAILED: expected .ship/releases directory" >&2
  exit 1
fi
if [[ ! -f "$WORK_DIR/.ship/templates/FEATURE.md" ]]; then
  echo "ASSERTION FAILED: expected .ship/templates/FEATURE.md template" >&2
  exit 1
fi
if [[ ! -f "$WORK_DIR/.ship/templates/RELEASE.md" ]]; then
  echo "ASSERTION FAILED: expected .ship/templates/RELEASE.md template" >&2
  exit 1
fi
if [[ ! -f "$WORK_DIR/.ship/specs/vision.md" ]]; then
  echo "ASSERTION FAILED: expected seeded .ship/specs/vision.md" >&2
  exit 1
fi
if [[ ! -f "$WORK_DIR/.ship/events.ndjson" ]]; then
  echo "ASSERTION FAILED: expected .ship/events.ndjson event stream file" >&2
  exit 1
fi

echo "Validating workflow/status customization..."
run_ship config status add qa >/dev/null
status_out="$(run_ship config status list)"
assert_contains "$status_out" "qa"

echo "Validating mode configuration..."
run_ship mode add planning "Planning Mode" >/dev/null
run_ship mode set planning >/dev/null
mode_out="$(run_ship mode get)"
assert_contains "$mode_out" "Active mode: planning (Planning Mode)"

echo "Validating spec workflow baseline..."
run_ship spec create "Agent Config Spec" >/dev/null
spec_list_out="$(run_ship spec list)"
assert_contains "$spec_list_out" "[draft] Agent Config Spec"
spec_file="$(find "$WORK_DIR/.ship/specs" -maxdepth 1 -name 'agent-config-spec*.md' -print | head -n 1)"
if [[ -z "${spec_file:-}" ]]; then
  echo "ASSERTION FAILED: expected generated spec file in .ship/specs" >&2
  exit 1
fi
spec_get_out="$(run_ship spec get "$(basename "$spec_file")")"
assert_contains "$spec_get_out" "title = \"Agent Config Spec\""

echo "Validating release workflow..."
run_ship release create "v0.1.0-alpha" >/dev/null
release_list_out="$(run_ship release list)"
assert_contains "$release_list_out" "[planned] v0.1.0-alpha"
release_file="$(find "$WORK_DIR/.ship/releases" -maxdepth 1 -name 'v0-1-0-alpha*.md' -print | head -n 1)"
if [[ -z "${release_file:-}" ]]; then
  echo "ASSERTION FAILED: expected generated release file in .ship/releases" >&2
  exit 1
fi
release_get_out="$(run_ship release get "$(basename "$release_file")")"
assert_contains "$release_get_out" "version = \"v0.1.0-alpha\""

echo "Validating feature workflow..."
run_ship feature create "Agent Config UI" --release "$(basename "$release_file")" --spec "$(basename "$spec_file")" >/dev/null
feature_list_out="$(run_ship feature list)"
assert_contains "$feature_list_out" "[active] Agent Config UI"
feature_file="$(find "$WORK_DIR/.ship/features" -maxdepth 1 -name 'agent-config-ui*.md' -print | head -n 1)"
if [[ -z "${feature_file:-}" ]]; then
  echo "ASSERTION FAILED: expected generated feature file in .ship/features" >&2
  exit 1
fi
feature_get_out="$(run_ship feature get "$(basename "$feature_file")")"
assert_contains "$feature_get_out" "title = \"Agent Config UI\""
assert_contains "$feature_get_out" "release = \"$(basename "$release_file")\""
assert_contains "$feature_get_out" "spec = \"$(basename "$spec_file")\""

echo "Validating MCP workflow parity..."
HOME="$ORIG_HOME" cargo test --manifest-path "$ROOT_DIR/Cargo.toml" -p mcp mcp_release_feature_flow_emits_events >/dev/null

echo "Validating git scope controls..."
# Default policy: issues local; adrs/features committed.
if ! grep -Eq '^issues$' "$WORK_DIR/.ship/.gitignore"; then
  echo "ASSERTION FAILED: expected issues to be gitignored by default" >&2
  exit 1
fi
if ! grep -Eq '^events.ndjson$' "$WORK_DIR/.ship/.gitignore"; then
  echo "ASSERTION FAILED: expected events.ndjson to be gitignored by default" >&2
  exit 1
fi
if grep -Eq '^adrs$' "$WORK_DIR/.ship/.gitignore"; then
  echo "ASSERTION FAILED: expected adrs to be committed by default" >&2
  exit 1
fi
if grep -Eq '^features$' "$WORK_DIR/.ship/.gitignore"; then
  echo "ASSERTION FAILED: expected features to be committed by default" >&2
  exit 1
fi
if grep -Eq '^releases$' "$WORK_DIR/.ship/.gitignore"; then
  echo "ASSERTION FAILED: expected releases to be committed by default" >&2
  exit 1
fi

run_ship git exclude adrs >/dev/null
if ! grep -Eq '^adrs$' "$WORK_DIR/.ship/.gitignore"; then
  echo "ASSERTION FAILED: expected adrs to be gitignored in .ship/.gitignore" >&2
  exit 1
fi

echo "Validating issue CRUD baseline..."
run_ship issue create "Workflow issue" "Validate end-to-end flow" >/dev/null
issues_out="$(run_ship issue list)"
assert_contains "$issues_out" "[backlog] workflow-issue.md"
events_out="$(run_ship event list --since 0 --limit 100)"
assert_contains "$events_out" "Issue.Create"
assert_contains "$events_out" "Release.Create"
assert_contains "$events_out" "Feature.Create"

echo "Validating filesystem ingest flow..."
cat > "$WORK_DIR/.ship/specs/manual-sync.md" <<'EOF'
+++
title = "Manual Sync"
status = "draft"
created = "2026-02-25T00:00:00Z"
updated = "2026-02-25T00:00:00Z"
author = ""
tags = []
+++

## Overview

Manual edit for ingest verification.
EOF
ingest_out="$(run_ship event ingest)"
assert_contains "$ingest_out" "Ingested"
events_after_ingest="$(run_ship event list --since 0 --limit 200)"
assert_contains "$events_after_ingest" "[filesystem]"
assert_contains "$events_after_ingest" "Spec.Create manual-sync.md"

echo "PASS: project feature e2e checks completed"
echo "Workspace: $WORK_DIR"
