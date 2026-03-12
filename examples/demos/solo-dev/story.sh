#!/usr/bin/env bash
# =============================================================================
# Ship Story: Solo Developer
#
# Alex is an indie dev building "TaskFlow" — a lightweight task management
# SaaS. This script walks through the complete Ship workflow: planning a
# release, defining features and specs, creating issues, configuring an AI
# agent, running a session, and shipping.
#
# Usage:
#   bash examples/demos/solo-dev/story.sh [--skip-build]
# =============================================================================
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
DEMOS_TMP="$ROOT_DIR/examples/demos/.tmp"
RUN_ID="$(date +%Y%m%d%H%M%S)"
WORK_DIR="$DEMOS_TMP/solo-dev-$RUN_ID"
HOME_DIR="$WORK_DIR/.home"
ORIG_HOME="${HOME:-}"
KEEP_TMP="${KEEP_TMP:-0}"
SHIP_BIN="${SHIP_BIN_OVERRIDE:-$ROOT_DIR/target/debug/ship}"
# Fall back to system ship if the debug binary is missing or stale
if ! "$SHIP_BIN" --version &>/dev/null 2>&1; then
  SHIP_BIN="$(which ship 2>/dev/null || echo ship)"
fi

# ── Colors ────────────────────────────────────────────────────────────────────
BOLD='\033[1m'
DIM='\033[2m'
CYAN='\033[0;36m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
RESET='\033[0m'

scene() {
  echo ""
  echo -e "${CYAN}${BOLD}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${RESET}"
  echo -e "${CYAN}${BOLD}  Scene $1: $2${RESET}"
  echo -e "${CYAN}${BOLD}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${RESET}"
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
echo -e "${BOLD}│  Ship Story: Solo Developer                     │${RESET}"
echo -e "${BOLD}│                                                 │${RESET}"
echo -e "${BOLD}│  Alex is building TaskFlow, a task management   │${RESET}"
echo -e "${BOLD}│  SaaS. Working alone with Claude as AI pair.    │${RESET}"
echo -e "${BOLD}└─────────────────────────────────────────────────┘${RESET}"

# ── Scene 1: Project Init ─────────────────────────────────────────────────────

scene 1 "Initializing the project"

narrate "Alex starts fresh. Ship initializes a .ship/ directory alongside the codebase."
narrate "Planning artifacts are committed to git; execution state stays local."
echo ""

run_ship init .

narrate ""
narrate "Ship has created the full project scaffold:"
echo ""
echo -e "${DIM}  .ship/"
echo "  ├── project/"
echo "  │   ├── features/     ← committed"
echo "  │   ├── releases/     ← committed"
echo "  │   ├── adrs/         ← committed"
echo "  │   ├── notes/        ← committed"
echo "  │   ├── vision.md     ← committed"
echo "  │   └── specs/        ← committed"
echo "  ├── issues/           ← local only"
echo "  ├── agents/           ← committed (modes, skills, rules)"
echo "  └── events.ndjson     ← local only (append-only log)"
echo -e "${RESET}"

# ── Scene 2: Plan the Release ─────────────────────────────────────────────────

scene 2 "Planning the v0.1.0 release"

narrate "Alex creates the first release milestone. This becomes the anchor for"
narrate "all features and specs in this cycle."
echo ""

run_ship release create "v0.1.0"
echo ""

narrate "Check the release list:"
run_ship release list

# Capture the release filename for linking
RELEASE_FILE="$(find "$WORK_DIR/.ship/project/releases" -maxdepth 2 -name 'v0.1.0*.md' -print | head -n 1)"
RELEASE_ID="$(basename "$RELEASE_FILE")"

# ── Scene 3: Define Features ──────────────────────────────────────────────────

scene 3 "Breaking the release into features"

narrate "Each feature is a meaningful chunk of user value. Alex identifies three"
narrate "core areas for v0.1.0: auth, the core task board, and team collaboration."
echo ""

run_ship feature create "User Authentication" --release-id "$RELEASE_ID"
echo ""
run_ship feature create "Task Board" --release-id "$RELEASE_ID"
echo ""
run_ship feature create "Team Collaboration" --release-id "$RELEASE_ID"
echo ""

narrate "Feature overview:"
run_ship feature list

AUTH_FILE="$(find "$WORK_DIR/.ship/project/features" -maxdepth 1 -name 'user-authentication*.md' -print | head -n 1)"
AUTH_ID="$(basename "$AUTH_FILE")"

# ── Scene 4: Write a Spec ─────────────────────────────────────────────────────

scene 4 "Writing a spec for User Authentication"

narrate "Specs define the technical approach before implementation. This one"
narrate "captures the auth architecture decisions and constraints."
echo ""

run_ship spec create "Authentication Architecture"
echo ""

SPEC_FILE="$(find "$WORK_DIR/.ship/project/specs" -maxdepth 2 -name 'authentication-architecture*.md' -print | head -n 1)"
SPEC_ID="$(basename "$SPEC_FILE")"

narrate "Spec list:"
run_ship spec list

# ── Scene 5: Create Work Items ────────────────────────────────────────────────

scene 5 "Creating issues for the auth feature"

narrate "Issues are local execution items — they don't clutter git history"
narrate "but are still tracked in the event stream."
echo ""

run_ship issue create "Implement JWT middleware" "Validate tokens on all authenticated routes"
run_ship issue create "Add Google OAuth flow" "OAuth2 redirect + callback + token exchange"
run_ship issue create "Add email/password login" "bcrypt hashing, rate limiting, forgot-password flow"
run_ship issue create "Session management" "Refresh tokens, logout, concurrent session limits"
echo ""

narrate "Current issue backlog:"
run_ship issue list

# ── Scene 6: Create an ADR ────────────────────────────────────────────────────

scene 6 "Recording an architecture decision"

narrate "Alex decides to use JWT over session cookies. This decision goes into"
narrate "an ADR so future Alex (and the AI) always knows the reasoning."
echo ""

run_ship adr create "Use JWT for authentication" \
  "JWT chosen over session cookies for stateless, scalable auth across services"

echo ""
narrate "ADR list:"
run_ship adr list

# ── Scene 7: Configure the Agent ─────────────────────────────────────────────

scene 7 "Setting up the AI agent"

narrate "Ship manages agent config independently of the AI provider."
narrate "Alex creates a focused mode for auth work — no distractions."
echo ""

narrate "Create a 'auth-focused' mode that restricts to relevant tools:"
run_ship mode add auth-focused "Authentication Focus Mode"
echo ""

narrate "Set it as the active mode:"
run_ship mode set auth-focused
echo ""

run_ship mode get
echo ""

narrate "Create a project skill so the agent always has TaskFlow context:"
run_ship skill create taskflow-context \
  "TaskFlow Project Context" \
  --content "TaskFlow is a SaaS task management app. Stack: Next.js, Postgres, Redis. Auth: JWT + OAuth2. See .ship/project/ for full planning context."
echo ""

narrate "Skill list:"
run_ship skill list
echo ""

narrate "Detect which AI providers are installed:"
run_ship providers detect
echo ""
run_ship providers list

# ── Scene 8: Export Agent Config ─────────────────────────────────────────────

scene 8 "Exporting agent config to Claude"

narrate "Ship exports its unified config (modes, skills, rules, MCP servers)"
narrate "into Claude's native config format. One source of truth, many clients."
echo ""

run_ship mcp export claude 2>/dev/null || narrate "(Claude not installed — skipping export)"
echo ""

# ── Scene 9: Start a Session ──────────────────────────────────────────────────

scene 9 "Running a workspace session"

narrate "Alex creates a workspace for the auth feature and starts a session."
narrate "Sessions track AI execution time, cost, and progress."
echo ""

run_ship workspace create "feature/user-auth" 2>/dev/null || true
echo ""

run_ship session start 2>/dev/null || narrate "(session start requires an active workspace)"
echo ""

narrate "Log progress mid-session:"
run_ship log "Implemented JWT validation middleware. All routes now require Bearer token." 2>/dev/null || true
run_ship log "Google OAuth flow complete. Callback handler tested against staging." 2>/dev/null || true
echo ""

run_ship session status 2>/dev/null || true
echo ""

narrate "End the session:"
run_ship session end 2>/dev/null || true

# ── Scene 10: Project State ───────────────────────────────────────────────────

scene 10 "Reviewing project state"

narrate "At any point Alex can review the full project state:"
echo ""

echo -e "${BOLD}  Release status:${RESET}"
run_ship release list
echo ""

echo -e "${BOLD}  Feature status:${RESET}"
run_ship feature list
echo ""

echo -e "${BOLD}  Active mode:${RESET}"
run_ship mode get
echo ""

echo -e "${BOLD}  Recent events:${RESET}"
run_ship event list --since 0 --limit 15

# ── Fin ───────────────────────────────────────────────────────────────────────

echo ""
echo -e "${GREEN}${BOLD}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${RESET}"
echo -e "${GREEN}${BOLD}  Story complete.${RESET}"
echo -e "${GREEN}${BOLD}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${RESET}"
echo ""
if [[ "$KEEP_TMP" == "1" ]]; then
  echo -e "  Workspace preserved at:"
  echo -e "  ${BOLD}$WORK_DIR${RESET}"
  echo ""
  echo -e "  To explore:"
  echo -e "  ${DIM}cd $WORK_DIR && ship release list${RESET}"
  echo -e "  ${DIM}cd $WORK_DIR && ship feature list${RESET}"
  echo -e "  ${DIM}cd $WORK_DIR && ship event list --since 0 --limit 50${RESET}"
else
  echo -e "  ${DIM}Workspace cleaned up. Set KEEP_TMP=1 to keep artifacts.${RESET}"
fi
echo ""
