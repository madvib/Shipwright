#!/usr/bin/env bash
# =============================================================================
# Ship Story: Multi-Provider
#
# Jordan uses different AI clients for different parts of the dev workflow:
# Gemini for broad planning and research, Claude for implementation, Codex for
# code review. Ship manages all three from a single config source.
#
# Usage:
#   bash examples/demos/multi-provider/story.sh [--skip-build]
# =============================================================================
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
DEMOS_TMP="$ROOT_DIR/examples/demos/.tmp"
RUN_ID="$(date +%Y%m%d%H%M%S)"
WORK_DIR="$DEMOS_TMP/multi-provider-$RUN_ID"
HOME_DIR="$WORK_DIR/.home"
ORIG_HOME="${HOME:-}"
KEEP_TMP="${KEEP_TMP:-0}"
SHIP_BIN="${SHIP_BIN_OVERRIDE:-$ROOT_DIR/target/debug/ship}"
if ! "$SHIP_BIN" --version &>/dev/null 2>&1; then
  SHIP_BIN="$(which ship 2>/dev/null || echo ship)"
fi

# ── Colors ────────────────────────────────────────────────────────────────────
BOLD='\033[1m'
DIM='\033[2m'
CYAN='\033[0;36m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
MAGENTA='\033[0;35m'
RESET='\033[0m'

scene() {
  echo ""
  echo -e "${MAGENTA}${BOLD}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${RESET}"
  echo -e "${MAGENTA}${BOLD}  Scene $1: $2${RESET}"
  echo -e "${MAGENTA}${BOLD}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${RESET}"
  echo ""
}

narrate() {
  echo -e "${DIM}  ▸ $1${RESET}"
}

run_ship() {
  echo -e "${YELLOW}  \$${RESET} ship $*"
  (cd "$WORK_DIR" && "$SHIP_BIN" "$@")
}

cleanup() {
  local exit_code=$?
  if [[ -n "${ORIG_HOME:-}" ]]; then
    export HOME="$ORIG_HOME"
  fi
  if [[ "$KEEP_TMP" != "1" ]]; then
    rm -rf "$WORK_DIR"
  fi
  return "$exit_code"
}

trap cleanup EXIT
trap 'exit 130' INT TERM

# ── Bootstrap ─────────────────────────────────────────────────────────────────

SKIP_BUILD=false
for arg in "$@"; do
  [[ "$arg" == "--skip-build" ]] && SKIP_BUILD=true
done

if [[ "$SKIP_BUILD" == false ]]; then
  echo -e "${BOLD}Building Ship CLI...${RESET}"
  HOME="$ORIG_HOME" cargo build --manifest-path "$ROOT_DIR/Cargo.toml" -p cli -q
fi

mkdir -p "$WORK_DIR" "$HOME_DIR"
export HOME="$HOME_DIR"

# ── Story begins ──────────────────────────────────────────────────────────────

echo ""
echo -e "${BOLD}┌─────────────────────────────────────────────────┐${RESET}"
echo -e "${BOLD}│  Ship Story: Multi-Provider                     │${RESET}"
echo -e "${BOLD}│                                                 │${RESET}"
echo -e "${BOLD}│  Jordan uses Gemini for planning, Claude for    │${RESET}"
echo -e "${BOLD}│  implementation, Codex for review.              │${RESET}"
echo -e "${BOLD}└─────────────────────────────────────────────────┘${RESET}"

# ── Scene 1: Initialize Project ───────────────────────────────────────────────

scene 1 "Initializing the project"

narrate "Ship init sets up the project scaffold. Provider config is separate"
narrate "from planning artifacts — both live in .ship/ but providers are"
narrate "managed globally or per-project."
echo ""

run_ship init .

# ── Scene 2: Discover Providers ───────────────────────────────────────────────

scene 2 "Detecting installed AI providers"

narrate "Ship knows about Claude (claude), Codex (codex), and Gemini (gemini)."
narrate "It looks for their binaries on PATH and reports what it finds."
echo ""

run_ship providers detect
echo ""

narrate "Full provider status (installed + version + available models):"
run_ship providers list

# ── Scene 3: Connect Providers to Project ─────────────────────────────────────

scene 3 "Connecting providers to this project"

narrate "By default, Ship tracks which providers are enabled per project."
narrate "Jordan connects all installed providers so they all get exports."
echo ""

# Connect whichever providers are actually installed
narrate "Connecting detected providers..."
run_ship providers connect claude 2>/dev/null || narrate "  (claude not installed — skipping)"
run_ship providers connect codex 2>/dev/null || narrate "  (codex not installed — skipping)"
run_ship providers connect gemini 2>/dev/null || narrate "  (gemini not installed — skipping)"
echo ""

run_ship providers list

# ── Scene 4: Create Workflow Modes ────────────────────────────────────────────

scene 4 "Creating workflow modes"

narrate "This is the key insight: modes are PROVIDER-AGNOSTIC."
narrate "Jordan defines what each mode does (capabilities, context, tools)"
narrate "and Ship translates that config for whichever provider is running."
echo ""

narrate "Planning mode: broad thinking, no code execution tools"
run_ship mode add roadmap "Planning & Research"
echo ""

narrate "Implementation mode: full access, optimized for coding"
run_ship mode add develop "Implementation"
echo ""

narrate "Review mode: read-only tools, focused on analysis"
run_ship mode add analysis "Code Review"
echo ""

run_ship mode list

# ── Scene 5: Add Project-Specific Skills ──────────────────────────────────────

scene 5 "Creating workflow-specific skills"

narrate "Skills provide the agent with focused context for each mode."
narrate "Rather than dumping everything into every chat, each mode gets"
narrate "exactly the context it needs."
echo ""

narrate "Planning skill: architecture context and decision history"
run_ship skill create planning-context \
  "Planning Context" \
  --content "You are helping plan software architecture. Focus on: system design, trade-offs, scalability, and alignment with project goals. Reference existing ADRs before suggesting new approaches."
echo ""

narrate "Review skill: code quality guidelines"
run_ship skill create review-standards \
  "Review Standards" \
  --content "You are performing code review. Enforce: no unused imports, test coverage for new public functions, consistent error handling, and documentation for public APIs. Be direct and constructive."
echo ""

run_ship skill list

# ── Scene 6: Set Up a Release and Features ────────────────────────────────────

scene 6 "Planning the release"

narrate "With providers and modes configured, Jordan plans the work."
echo ""

run_ship release create "v1.0.0"
echo ""
RELEASE_FILE="$(find "$WORK_DIR/.ship/project/releases" -maxdepth 2 -name 'v1.0.0*.md' -print | head -n 1)"
RELEASE_ID="$(basename "$RELEASE_FILE")"

run_ship feature create "REST API Layer" --release-id "$RELEASE_ID"
run_ship feature create "Authentication Service" --release-id "$RELEASE_ID"
run_ship feature create "Frontend Dashboard" --release-id "$RELEASE_ID"
echo ""

run_ship feature list

# ── Scene 7: Mode-Switching Day in the Life ───────────────────────────────────

scene 7 "A day in the life: mode switching"

narrate "Morning: Jordan switches to planning mode to design the API layer."
run_ship mode set roadmap
echo ""
run_ship mode get
echo ""
narrate "  → Jordan opens Claude/Gemini, which now has the planning-context skill"
narrate "  → Discusses API design, documents decisions as ADRs"
echo ""

run_ship adr create "REST over GraphQL for v1.0" "REST chosen for simplicity and broader tooling support"
run_ship adr create "Postgres as primary datastore" "Postgres selected for ACID compliance and rich querying"
echo ""

narrate "Afternoon: switch to implementation mode, start coding."
run_ship mode set develop
echo ""
run_ship mode get
echo ""
narrate "  → Full tool access, AI helps write and refactor code"
narrate "  → Issues created and tracked in Ship"
echo ""

run_ship issue create "Implement /api/v1/tasks CRUD" "GET, POST, PUT, DELETE endpoints"
run_ship issue create "Add OpenAPI spec generation" "Auto-generate from route handlers"
echo ""

narrate "End of day: review mode to check the work."
run_ship mode set analysis
echo ""
run_ship mode get
echo ""
narrate "  → Read-only tools, review-standards skill active"
narrate "  → AI reviews diff, flags issues, suggests improvements"

# ── Scene 8: Export to All Providers ─────────────────────────────────────────

scene 8 "Exporting config to all connected providers"

narrate "When Jordan's setup changes — new MCP server, updated skill, new mode —"
narrate "they export to all providers in one go. Ship translates its unified"
narrate "format to each provider's native config schema."
echo ""

for provider in claude codex gemini; do
  narrate "Exporting to $provider..."
  run_ship mcp export "$provider" 2>/dev/null \
    || narrate "  ($provider not installed — export skipped)"
  echo ""
done

# ── Scene 9: Show the Config Landscape ───────────────────────────────────────

scene 9 "Config landscape overview"

narrate "At any time Jordan can see the full picture:"
echo ""

echo -e "${BOLD}  Modes:${RESET}"
run_ship mode list
echo ""

echo -e "${BOLD}  Skills:${RESET}"
run_ship skill list
echo ""

echo -e "${BOLD}  ADRs (decisions captured):${RESET}"
run_ship adr list
echo ""

echo -e "${BOLD}  Providers:${RESET}"
run_ship providers list
echo ""

echo -e "${BOLD}  Event log (all activity):${RESET}"
run_ship event list --since 0 --limit 20

# ── Fin ───────────────────────────────────────────────────────────────────────

echo ""
echo -e "${GREEN}${BOLD}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${RESET}"
echo -e "${GREEN}${BOLD}  Story complete.${RESET}"
echo ""
echo -e "  ${DIM}Key takeaway: modes are provider-agnostic. One config, many clients.${RESET}"
echo ""
if [[ "$KEEP_TMP" == "1" ]]; then
  echo -e "  Workspace at: ${BOLD}$WORK_DIR${RESET}"
  echo ""
  echo -e "  Try mode switching:"
  echo -e "  ${DIM}cd $WORK_DIR && ship mode set planning && ship mode get${RESET}"
else
  echo -e "  ${DIM}Workspace cleaned up. Set KEEP_TMP=1 to keep artifacts.${RESET}"
fi
echo ""
