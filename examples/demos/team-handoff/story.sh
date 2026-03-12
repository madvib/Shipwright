#!/usr/bin/env bash
# =============================================================================
# Ship Story: Team Handoff
#
# Sam (senior dev) has built up months of project context in Ship: planning
# artifacts, agent config, rules, skills, and MCP server setup. A new
# contributor, Alex, joins. This story shows how Ship makes that handoff
# immediate — Alex clones the repo and is productive with full AI context
# from the first session.
#
# Usage:
#   bash examples/demos/team-handoff/story.sh [--skip-build]
# =============================================================================
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
DEMOS_TMP="$ROOT_DIR/examples/demos/.tmp"
RUN_ID="$(date +%Y%m%d%H%M%S)"

# Two separate workspace roots — Sam's and Alex's (simulates git clone)
SAM_DIR="$DEMOS_TMP/handoff-sam-$RUN_ID"
ALEX_DIR="$DEMOS_TMP/handoff-alex-$RUN_ID"
SAM_HOME="$SAM_DIR/.home"
ALEX_HOME="$ALEX_DIR/.home"
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
BLUE='\033[0;34m'
RESET='\033[0m'

scene() {
  echo ""
  echo -e "${BLUE}${BOLD}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${RESET}"
  echo -e "${BLUE}${BOLD}  Scene $1: $2${RESET}"
  echo -e "${BLUE}${BOLD}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${RESET}"
  echo ""
}

narrate() {
  echo -e "${DIM}  ▸ $1${RESET}"
}

as_sam() {
  echo -e "${CYAN}  [Sam]${RESET} ${YELLOW}\$${RESET} ship $*"
  (cd "$SAM_DIR" && HOME="$SAM_HOME" "$SHIP_BIN" "$@")
}

as_alex() {
  echo -e "${GREEN}  [Alex]${RESET} ${YELLOW}\$${RESET} ship $*"
  (cd "$ALEX_DIR" && HOME="$ALEX_HOME" "$SHIP_BIN" "$@")
}

cleanup() {
  local exit_code=$?
  if [[ -n "${ORIG_HOME:-}" ]]; then
    export HOME="$ORIG_HOME"
  fi
  if [[ "$KEEP_TMP" != "1" ]]; then
    rm -rf "$SAM_DIR" "$ALEX_DIR"
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

mkdir -p "$SAM_DIR" "$SAM_HOME" "$ALEX_HOME"

# ── Story begins ──────────────────────────────────────────────────────────────

echo ""
echo -e "${BOLD}┌─────────────────────────────────────────────────┐${RESET}"
echo -e "${BOLD}│  Ship Story: Team Handoff                       │${RESET}"
echo -e "${BOLD}│                                                 │${RESET}"
echo -e "${BOLD}│  Sam (senior) → Alex (new contributor)          │${RESET}"
echo -e "${BOLD}│  Institutional knowledge, made executable.      │${RESET}"
echo -e "${BOLD}└─────────────────────────────────────────────────┘${RESET}"

# =============================================================================
# PART 1: SAM BUILDS UP CONTEXT
# =============================================================================

echo ""
echo -e "${CYAN}${BOLD}  ═══════════════════════════════════════════════════"
echo -e "  Part 1: Sam's project (two months of work)"
echo -e "  ═══════════════════════════════════════════════════${RESET}"

# ── Scene 1: Established Project ─────────────────────────────────────────────

scene 1 "Sam's established project state"

narrate "Sam initializes a git repo (simulating an existing project with history)."
echo ""

(cd "$SAM_DIR" && git init -q && git config user.email "sam@example.com" && git config user.name "Sam Dev")
as_sam init .
echo ""

narrate "Several months of planning artifacts now exist:"
as_sam release create "v0.1.0"
as_sam release create "v0.2.0"
as_sam release create "v1.0.0"
echo ""

RELEASE_V1="$(find "$SAM_DIR/.ship/project/releases" -maxdepth 2 -name 'v1.0.0*.md' -print | head -n 1)"
RELEASE_V1_ID="$(basename "$RELEASE_V1")"

as_sam feature create "Core API" --release-id "$RELEASE_V1_ID"
as_sam feature create "Admin Dashboard" --release-id "$RELEASE_V1_ID"
as_sam feature create "Webhook System" --release-id "$RELEASE_V1_ID"
echo ""

narrate "Key architecture decisions are captured:"
as_sam adr create "Postgres over MongoDB" "Postgres selected for ACID compliance and team expertise"
as_sam adr create "Event sourcing for audit trail" "Immutable event log required for compliance and audit history"
as_sam adr create "REST over GraphQL for v1" "REST chosen for simplicity and broader tooling support"
echo ""

as_sam spec create "Webhook Delivery Architecture"
as_sam spec create "Admin Permission Model"
echo ""

narrate "Project state after two months:"
as_sam release list
echo ""
as_sam feature list

# ── Scene 2: Sam Writes the Project Rules ────────────────────────────────────

scene 2 "Sam writes rules — the team's coding standards"

narrate "Rules apply to EVERY agent session, regardless of who runs it."
narrate "They encode the team's standards so the AI never goes off-script."
narrate "(Rules are markdown files in .ship/agents/rules/ — committed to git)"
echo ""

# Write rule files directly
mkdir -p "$SAM_DIR/.ship/agents/rules"

cat > "$SAM_DIR/.ship/agents/rules/coding-standards.md" <<'RULE'
# Coding Standards

## Language
- TypeScript only. No JavaScript files in src/.
- Strict mode. No `any` types except at external boundaries.

## Testing
- Every new public function must have at least one unit test.
- Integration tests for all API endpoints.
- Use vitest for unit tests, supertest for API tests.

## Error handling
- Throw typed errors (use src/errors/ classes).
- Never swallow exceptions silently.
- All async routes must have error boundaries.

## Database
- Use the query builder (Kysely). No raw SQL strings.
- All migrations must be reversible.
- Never query in a loop — use batch operations.
RULE

cat > "$SAM_DIR/.ship/agents/rules/git-conventions.md" <<'RULE'
# Git Conventions

## Commits
- Conventional commits: feat/fix/chore/docs/refactor/test
- Scope required for feat/fix: feat(auth): ...
- No "WIP" or "temp" commits on main.

## Branches
- Feature branches: feature/<id>-<slug>
- Bug fixes: fix/<id>-<slug>
- No long-lived branches. Merge to main within 3 days.

## PRs
- PR title must match the issue title.
- At least one approval before merge.
- All CI checks green before merge.
RULE

cat > "$SAM_DIR/.ship/agents/rules/security.md" <<'RULE'
# Security Rules

## NEVER
- Hardcode secrets, API keys, or tokens in source files.
- Log request bodies in production (may contain PII).
- Use eval() or similar dynamic code execution.
- Disable CORS or CSP headers.

## ALWAYS
- Validate and sanitize all user input at the API boundary.
- Use parameterized queries for all database operations.
- Rate-limit all public endpoints.
- Check authorization before every database write.
RULE

narrate "Rules written:"
ls "$SAM_DIR/.ship/agents/rules/"
echo ""
narrate "These rules will apply to every agent session — Sam's and Alex's."

# ── Scene 3: Sam Creates Project Skills ──────────────────────────────────────

scene 3 "Sam creates skills for onboarding and focused work"

narrate "Skills are the 'system prompt' for each mode. Sam creates two:"
narrate "  1. onboarding-context — full project orientation for new contributors"
narrate "  2. api-development    — focused skill for the Core API feature"
echo ""

as_sam skill create onboarding-context \
  "Project Onboarding Context" \
  --content "$(cat <<'SKILL'
You are working on a B2B SaaS API platform. Key facts:

**Stack**: Node.js + TypeScript, Postgres, Redis, Kysely ORM, Vitest tests, deployed on Railway.

**Architecture**: Event-sourced audit trail (events.ndjson), REST API (no GraphQL yet), webhook delivery system in progress.

**Key ADRs**: Postgres over MongoDB (performance + transactions), REST over GraphQL for v1 (simplicity), event sourcing for audit (compliance requirement).

**Current focus**: v1.0.0 milestone — Core API, Admin Dashboard, Webhook System.

**Never do**: Raw SQL strings (use Kysely), any types (use typed errors), skip tests for public functions.

See .ship/project/ for full planning context and .ship/agents/rules/ for coding standards.
SKILL
)"
echo ""

as_sam skill create api-development \
  "Core API Development" \
  --content "You are implementing the Core API feature. Focus areas: REST endpoint design, Kysely query patterns, typed error handling, and test coverage. Reference the existing src/api/ structure and match established patterns. When in doubt, look at a similar existing endpoint before writing new code."
echo ""

as_sam skill list

# ── Scene 4: Sam Sets Up Modes ────────────────────────────────────────────────

scene 4 "Sam configures workflow modes"

narrate "Modes define capability bundles. Sam sets up three:"
narrate "  onboarding  — full context, useful for orientation"
narrate "  api-work    — focused on Core API implementation"
narrate "  review      — read-only analysis mode"
echo ""

as_sam mode add onboarding "Project Onboarding"
as_sam mode add api-work "API Development"
as_sam mode add review "Code Review"
echo ""

as_sam mode list

# ── Scene 5: Sam Adds MCP Servers ────────────────────────────────────────────

scene 5 "Sam registers MCP servers"

narrate "Sam has a few MCP servers configured. They're stored in"
narrate ".ship/agents/mcp.toml and committed to git."
narrate "(Adding via the config file directly to demonstrate the TOML format)"
echo ""

mkdir -p "$SAM_DIR/.ship/agents"
cat >> "$SAM_DIR/.ship/agents/mcp.toml" <<'MCP'

[mcp.servers.filesystem]
id = "filesystem"
name = "Filesystem"
command = "npx"
args = ["-y", "@modelcontextprotocol/server-filesystem", "."]
scope = "project"
server_type = "stdio"
disabled = false

[mcp.servers.filesystem.env]

[mcp.servers.github]
id = "github"
name = "GitHub"
command = "npx"
args = ["-y", "@modelcontextprotocol/server-github"]
scope = "project"
server_type = "stdio"
disabled = false

[mcp.servers.github.env]
GITHUB_TOKEN = "${GITHUB_TOKEN}"
MCP

as_sam mcp list
echo ""
narrate "MCP servers are committed — Alex gets them automatically on clone."

# ── Scene 6: Sam Commits and Exports ─────────────────────────────────────────

scene 6 "Sam commits the project state and exports agent config"

narrate "Everything in .ship/ (except issues/ and events.ndjson) is committed."
narrate "Sam also writes a handoff note and exports agent config."
echo ""

(cd "$SAM_DIR" && git add .ship/ && git commit -q -m "chore: prepare project handoff — add rules, skills, modes, MCP config")

narrate "What's committed:"
(cd "$SAM_DIR" && git log --oneline)
echo ""

narrate "Sam exports to Claude before wrapping up:"
as_sam mcp export claude 2>/dev/null || narrate "  (Claude not installed — skipping export demo)"
echo ""

narrate "Writing a handoff note for Alex:"
cat > "$SAM_DIR/.ship/project/notes/handoff-for-alex.md" <<'NOTE'
+++
title = "Handoff for Alex"
created = "2026-03-07T00:00:00Z"
updated = "2026-03-07T00:00:00Z"
author = "sam"
tags = ["onboarding", "handoff"]
+++

# Handoff for Alex

Welcome to the project! Here's what you need to know to get started fast.

## First steps

1. Run `ship providers detect` to connect your AI clients
2. Run `ship mode set onboarding` — this activates the onboarding skill
3. Open Claude (or your preferred provider) and ask it to orient you to the codebase
4. Run `ship feature list` and `ship release list` to see what's in flight
5. Pick up an issue from the Core API feature: `ship issue list`

## Project conventions

All coding standards, git conventions, and security rules are in
`.ship/agents/rules/`. They apply automatically to every agent session.

## Key contacts

- Infra questions → ping @ops-channel
- Product questions → the spec files in `.ship/project/specs/`
- Architecture decisions → `.ship/project/adrs/`

## Active work

The Core API feature is in progress. Focus there first.
NOTE

as_sam note list

# =============================================================================
# PART 2: ALEX ONBOARDS
# =============================================================================

echo ""
echo -e "${GREEN}${BOLD}  ═══════════════════════════════════════════════════"
echo -e "  Part 2: Alex joins the project"
echo -e "  ═══════════════════════════════════════════════════${RESET}"

scene 7 "Alex clones the repo and activates Ship"

narrate "Alex simulates a git clone by copying Sam's committed .ship/ state."
narrate "In a real workflow, this is just: git clone <repo>"
echo ""

mkdir -p "$ALEX_DIR"
# Simulate git clone: copy committed .ship/ state
cp -r "$SAM_DIR/.ship" "$ALEX_DIR/.ship"
(cd "$ALEX_DIR" && git init -q && git config user.email "alex@example.com" && git config user.name "Alex Dev")
(cd "$ALEX_DIR" && cp -r "$SAM_DIR/.ship" . && git add .ship/ 2>/dev/null || true)

narrate "Alex initializes their local Ship state (registers the project):"
as_alex projects list 2>/dev/null || true
as_alex init . 2>/dev/null || narrate "  (project already initialized via clone)"
echo ""

narrate "Workflow modes are team config — Alex registers them from the project README:"
narrate "  (In practice: run once after cloning. Modes live in local state, not git)"
as_alex mode add onboarding "Project Onboarding" 2>/dev/null || narrate "  (mode already exists)"
as_alex mode add api-work "API Development" 2>/dev/null || narrate "  (mode already exists)"
as_alex mode add review "Code Review" 2>/dev/null || narrate "  (mode already exists)"
echo ""

narrate "Alex immediately sees the full project context:"
echo ""
echo -e "${BOLD}  Releases:${RESET}"
as_alex release list
echo ""
echo -e "${BOLD}  Features in flight:${RESET}"
as_alex feature list
echo ""
echo -e "${BOLD}  ADRs (architecture decisions):${RESET}"
as_alex adr list

# ── Scene 8: Alex Gets Up to Speed ───────────────────────────────────────────

scene 8 "Alex activates the onboarding mode and gets productive"

narrate "Alex sets the onboarding mode — this activates the onboarding-context"
narrate "skill in their AI session. The AI now has full project orientation"
narrate "without Alex having to paste anything."
echo ""

as_alex mode set onboarding
as_alex mode get
echo ""

narrate "Alex connects their AI providers:"
as_alex providers detect
echo ""

narrate "Export Sam's config to Alex's local Claude (same config, new machine):"
as_alex mcp export claude 2>/dev/null || narrate "  (Claude not installed — skipping)"
echo ""

narrate "Alex reads the handoff note:"
as_alex note list
echo ""

# ── Scene 9: Alex Picks Up Work ──────────────────────────────────────────────

scene 9 "Alex picks up the first issue"

narrate "Alex switches to the api-work mode and picks up an issue:"
as_alex mode set api-work
as_alex mode get
echo ""

narrate "Creating Alex's first issue — linked to the Core API feature:"
CORE_API="$(find "$ALEX_DIR/.ship/project/features" -maxdepth 1 -name 'core-api*.md' -print | head -n 1)"
CORE_API_ID="$(basename "${CORE_API:-}")"

as_alex issue create "Implement /api/v1/webhooks CRUD" "Implement create/list/delete for webhook subscriptions. See webhook architecture spec for data model."
echo ""

as_alex issue list
echo ""

narrate "Alex starts a session, logs progress, runs their AI session with full"
narrate "project context already loaded via the active mode and skill."
echo ""

as_alex session start 2>/dev/null || narrate "  (session requires active workspace — skipping)"
as_alex log "Started webhook endpoint implementation. Following existing patterns in src/api/tasks/." 2>/dev/null || true
as_alex session end 2>/dev/null || true

# ── Scene 10: What Alex Got for Free ─────────────────────────────────────────

scene 10 "What Alex inherited automatically"

narrate "Let's see everything Alex has access to on day one:"
echo ""

echo -e "${BOLD}  Rules (apply to every AI session):${RESET}"
for f in "$ALEX_DIR/.ship/agents/rules/"*.md; do
  echo -e "  ${DIM}  • $(basename "$f")${RESET}"
done
echo ""

echo -e "${BOLD}  Skills (focused agent context):${RESET}"
as_alex skill list
echo ""

echo -e "${BOLD}  Modes (workflow presets):${RESET}"
as_alex mode list
echo ""

echo -e "${BOLD}  MCP servers (tools registered):${RESET}"
as_alex mcp list
echo ""

echo -e "${BOLD}  Planning context:${RESET}"
as_alex event list --since 0 --limit 8

# ── Fin ───────────────────────────────────────────────────────────────────────

echo ""
echo -e "${GREEN}${BOLD}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${RESET}"
echo -e "${GREEN}${BOLD}  Story complete.${RESET}"
echo ""
echo -e "  ${DIM}Sam → Alex: zero-friction. All context committed, all config shared.${RESET}"
echo ""
if [[ "$KEEP_TMP" == "1" ]]; then
  echo -e "  Sam's workspace:  ${BOLD}$SAM_DIR${RESET}"
  echo -e "  Alex's workspace: ${BOLD}$ALEX_DIR${RESET}"
  echo ""
  echo -e "  Compare them:"
  echo -e "  ${DIM}cd $ALEX_DIR && ship feature list && ship mode list && ship skill list${RESET}"
else
  echo -e "  ${DIM}Workspaces cleaned up. Set KEEP_TMP=1 to keep artifacts.${RESET}"
fi
echo ""
