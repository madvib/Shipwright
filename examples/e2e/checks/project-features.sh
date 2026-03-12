#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
TMP_ROOT="$ROOT_DIR/examples/e2e/.tmp"
GLOBAL_TMP_ROOT="$ROOT_DIR/examples/e2e/.tmp-global"
RUN_ID="$(date +%Y%m%d%H%M%S)"
WORK_DIR="$TMP_ROOT/project-e2e-$RUN_ID"
HOME_DIR="$GLOBAL_TMP_ROOT/home-$RUN_ID"
ORIG_HOME="${HOME:-}"
KEEP_TMP="${KEEP_TMP:-0}"

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

assert_path_exists() {
  if [[ ! -e "$1" ]]; then
    echo "ASSERTION FAILED: expected path to exist: $1" >&2
    exit 1
  fi
}

assert_path_not_exists() {
  if [[ -e "$1" ]]; then
    echo "ASSERTION FAILED: expected path NOT to exist: $1" >&2
    exit 1
  fi
}

assert_path_not_in_gitignore() {
  local gitignore="$WORK_DIR/.ship/.gitignore"
  if grep -qE "^$1$" "$gitignore"; then
    echo "ASSERTION FAILED: expected '$1' NOT in .ship/.gitignore" >&2
    exit 1
  fi
}

assert_path_in_gitignore() {
  local gitignore="$WORK_DIR/.ship/.gitignore"
  if ! grep -qE "^$1$" "$gitignore"; then
    echo "ASSERTION FAILED: expected '$1' in .ship/.gitignore" >&2
    exit 1
  fi
}

run_ship() {
  (cd "$WORK_DIR" && "$SHIP_BIN" "$@")
}

cleanup() {
  local exit_code=$?
  if [[ -n "${ORIG_HOME:-}" ]]; then
    export HOME="$ORIG_HOME"
  fi
  if [[ "$KEEP_TMP" != "1" ]]; then
    rm -rf "$WORK_DIR"
    rm -rf "$HOME_DIR"
  fi
  return "$exit_code"
}

trap cleanup EXIT
trap 'exit 130' INT TERM

echo "Building CLI binary..."
HOME="$ORIG_HOME" cargo build --manifest-path "$ROOT_DIR/Cargo.toml" -p cli >/dev/null

export HOME="$HOME_DIR"

echo "Initializing isolated test workspace at $WORK_DIR"
(cd "$WORK_DIR" && git init -q && git config user.email "ship-e2e@example.com" && git config user.name "Ship E2E")
out="$(run_ship init .)"
assert_contains "$out" "Initialized and tracked Ship project"

# Canonical Ship structure
assert_path_exists "$WORK_DIR/.ship/vision.md"
assert_path_exists "$WORK_DIR/.ship/ship.toml"
assert_path_exists "$WORK_DIR/.ship/.gitignore"
assert_path_exists "$WORK_DIR/.ship/agents/mcp.toml"
assert_path_exists "$WORK_DIR/.ship/agents/permissions.toml"
assert_path_exists "$WORK_DIR/.ship/agents/skills/task-policy/SKILL.md"
assert_path_exists "$WORK_DIR/.ship/generated"
assert_path_not_exists "$WORK_DIR/.ship/TEMPLATE.md"
assert_path_not_exists "$WORK_DIR/.ship/README.md"

# DB-first model: project markdown namespaces are not pre-seeded at init.
assert_path_not_exists "$WORK_DIR/.ship/project/features"
assert_path_not_exists "$WORK_DIR/.ship/project/specs"
assert_path_not_exists "$WORK_DIR/.ship/project/releases"
assert_path_not_exists "$WORK_DIR/.ship/project/adrs"
assert_path_not_exists "$WORK_DIR/.ship/project/notes"

echo "Validating workflow/status customization..."
run_ship config status add qa >/dev/null
status_out="$(run_ship config status list)"
assert_contains "$status_out" "qa"

echo "Validating mode configuration..."
run_ship mode add planning "Planning Mode" >/dev/null
run_ship mode set planning >/dev/null
mode_out="$(run_ship mode get)"
assert_contains "$mode_out" "Active mode: planning (Planning Mode)"

echo "Validating release workflow..."
run_ship release create "v0.1.0-alpha" >/dev/null
release_list_out="$(run_ship release list)"
assert_contains "$release_list_out" "[upcoming] v0.1.0-alpha"
release_get_out="$(run_ship release get "v0.1.0-alpha.md")"
assert_contains "$release_get_out" "version=v0.1.0-alpha"

echo "Validating feature workflow..."
run_ship feature create "Agent Config UI" --release-id "v0.1.0-alpha" >/dev/null
feature_list_out="$(run_ship feature list)"
assert_contains "$feature_list_out" "[planned] Agent Config UI"
feature_id="$(echo "$feature_list_out" | sed -n 's/.*id=\([^ ]*\)$/\1/p' | head -n 1)"
if [[ -z "${feature_id:-}" ]]; then
  echo "ASSERTION FAILED: expected feature id in `ship feature list` output" >&2
  exit 1
fi
feature_get_out="$(run_ship feature get "$feature_id")"
assert_contains "$feature_get_out" "title = \"Agent Config UI\""
assert_contains "$feature_get_out" "release_id = \"v0.1.0-alpha\""

echo "Validating workspace/session lifecycle..."
run_ship workspace create "feature/agent-config-ui" --type feature --feature "$feature_id" --activate --no-input >/dev/null
workspace_out="$(run_ship workspace list)"
assert_contains "$workspace_out" "[active] feature/agent-config-ui (feature)"

run_ship workspace session start --branch "feature/agent-config-ui" --goal "Validate e2e flow" >/dev/null
session_status_out="$(run_ship workspace session status --branch "feature/agent-config-ui")"
assert_contains "$session_status_out" "[active]"
assert_contains "$session_status_out" "workspace=feature/agent-config-ui"
run_ship workspace session end --branch "feature/agent-config-ui" --summary "e2e check complete" >/dev/null

echo "Validating MCP runtime registration..."
mcp_out="$(run_ship mcp list)"
assert_contains "$mcp_out" "ship — Ship Runtime"

echo "Validating git scope controls..."
# Default: generated/runtime files ignored; ship.toml + core agent config tracked.
assert_path_in_gitignore "generated/"
assert_path_in_gitignore ".tmp-global/"
assert_path_in_gitignore "project/adrs"
assert_path_in_gitignore "project/notes"
assert_path_in_gitignore "project/features"
assert_path_in_gitignore "project/releases"
assert_path_in_gitignore "project/specs"
assert_path_in_gitignore "vision.md"
assert_path_in_gitignore "agents/skills"
assert_path_not_in_gitignore "ship.toml"
assert_path_not_in_gitignore "agents/rules"
assert_path_not_in_gitignore "agents/mcp.toml"
assert_path_not_in_gitignore "agents/permissions.toml"

run_ship git include adrs >/dev/null
assert_path_not_in_gitignore "project/adrs"

echo "Validating event stream visibility..."
events_out="$(run_ship event list --since 0 --limit 100)"
assert_contains "$events_out" "Release.Create"
assert_contains "$events_out" "Feature.Create"
assert_contains "$events_out" "Session.Start"
assert_contains "$events_out" "Session.Stop"

echo "PASS: project feature e2e checks completed"
if [[ "$KEEP_TMP" == "1" ]]; then
  echo "Workspace: $WORK_DIR"
  echo "Global HOME sandbox: $HOME_DIR"
else
  echo "Workspace cleaned up (set KEEP_TMP=1 to retain)"
fi
