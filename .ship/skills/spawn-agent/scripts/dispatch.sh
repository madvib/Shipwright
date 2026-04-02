#!/usr/bin/env bash
# Ship dispatch — create a worktree, write a job spec, compile the agent, open a terminal.
# Usage: bash scripts/dispatch.sh --slug <name> --agent <agent> --spec <file> [options]
set -euo pipefail

# ── Defaults ──────────────────────────────────────────────────────────────────
SHIP_GLOBAL_DIR="${SHIP_GLOBAL_DIR:-$HOME/.ship}"
WORKTREE_BASE="${SHIP_WORKTREE_DIR:-${HOME}/dev/ship-worktrees}"
BASE_BRANCH=""
NO_OPEN=false
DRY_RUN=false
CONFIRM="${SHIP_DISPATCH_CONFIRM:-}"
SLUG=""
AGENT=""
SPEC_FILE=""
SHIP_AGENT_MODEL="${SHIP_AGENT_MODEL:-}"

# ── Parse args ────────────────────────────────────────────────────────────────
while [[ $# -gt 0 ]]; do
  case "$1" in
    --slug)     SLUG="$2"; shift 2 ;;
    --agent)    AGENT="$2"; shift 2 ;;
    --spec)     SPEC_FILE="$2"; shift 2 ;;
    --base)     BASE_BRANCH="$2"; shift 2 ;;
    --dir)      WORKTREE_BASE="$2"; shift 2 ;;
    --model)    SHIP_AGENT_MODEL="$2"; shift 2 ;;
    --no-open)  NO_OPEN=true; shift ;;
    --dry-run)  DRY_RUN=true; shift ;;
    --confirm)  CONFIRM=1; shift ;;
    *) echo "Unknown option: $1" >&2; exit 1 ;;
  esac
done

if [[ -z "$SLUG" || -z "$AGENT" || -z "$SPEC_FILE" ]]; then
  echo "Usage: dispatch.sh --slug <name> --agent <agent> --spec <file>" >&2
  echo "  --base <branch>   Branch to fork from (default: current)" >&2
  echo "  --dir <path>      Worktree base dir (default: ~/dev/ship-worktrees)" >&2
  echo "  --no-open         Print launch command instead of opening terminal" >&2
  echo "  --confirm         Show spec and ask y/n before launching" >&2
  echo "  --dry-run         Show what would happen" >&2
  exit 1
fi

if [[ ! -f "$SPEC_FILE" ]]; then
  echo "Spec file not found: $SPEC_FILE" >&2
  exit 1
fi

WORKTREE_PATH="${WORKTREE_BASE}/${SLUG}"
BRANCH_NAME="job/${SLUG}"
BASE_BRANCH="${BASE_BRANCH:-$(git rev-parse --abbrev-ref HEAD)}"

# ── Detect terminal ───────────────────────────────────────────────────────────
detect_terminal() {
  local term="${SHIP_DEFAULT_TERMINAL:-}"
  if [[ -n "$term" ]]; then echo "$term"; return; fi
  if [[ -n "${WT_SESSION:-}" ]]; then echo "wt"; return; fi
  if [[ -n "${TMUX:-}" ]]; then echo "tmux"; return; fi
  case "${TERM_PROGRAM:-}" in
    iTerm.app)       echo "iterm" ;;
    WarpTerminal)    echo "warp" ;;
    vscode)          echo "vscode" ;;
    Apple_Terminal)  echo "manual" ;;
    *)               echo "manual" ;;
  esac
}

TERMINAL=$(detect_terminal)

# ── Dry run ───────────────────────────────────────────────────────────────────
if $DRY_RUN; then
  echo "dispatch (dry run):"
  echo "  slug:            $SLUG"
  echo "  agent:           $AGENT"
  echo "  spec:            $SPEC_FILE"
  echo "  base:            $BASE_BRANCH"
  echo "  worktree:        $WORKTREE_PATH"
  echo "  branch:          $BRANCH_NAME"
  echo "  terminal:        $TERMINAL"
  echo "  SHIP_GLOBAL_DIR: $SHIP_GLOBAL_DIR"
  exit 0
fi

# ── Confirm ───────────────────────────────────────────────────────────────────
if [[ -n "$CONFIRM" ]]; then
  echo "=== Job Spec ==="
  cat "$SPEC_FILE"
  echo ""
  echo "Agent: $AGENT | Worktree: $WORKTREE_PATH | Branch: $BRANCH_NAME"
  read -rp "Dispatch? [y/N] " answer
  if [[ ! "$answer" =~ ^[Yy]$ ]]; then
    echo "Cancelled."
    exit 0
  fi
fi

# ── Create worktree (idempotent) ──────────────────────────────────────────────
mkdir -p "$WORKTREE_BASE"

if [[ -d "$WORKTREE_PATH" ]]; then
  echo "Worktree already exists: $WORKTREE_PATH"
else
  echo "Creating worktree: $WORKTREE_PATH (branch: $BRANCH_NAME from $BASE_BRANCH)"
  git worktree add "$WORKTREE_PATH" -b "$BRANCH_NAME" "$BASE_BRANCH"
fi

# ── Write job spec ────────────────────────────────────────────────────────────
DEST_SPEC="$WORKTREE_PATH/.ship-session/job-spec.md"
mkdir -p "$(dirname "$DEST_SPEC")"
cp "$SPEC_FILE" "$DEST_SPEC"
echo "Job spec written: $DEST_SPEC"

# ── Environment setup (hard stop) ─────────────────────────────────────────────
export SHIP_GLOBAL_DIR

if ! command -v ship &>/dev/null; then
  echo "Error: ship CLI not found. Install ship and ensure it is on PATH." >&2
  exit 1
fi

echo "Compiling agent: ship use $AGENT (SHIP_GLOBAL_DIR=$SHIP_GLOBAL_DIR)"
if ! (cd "$WORKTREE_PATH" && SHIP_GLOBAL_DIR="$SHIP_GLOBAL_DIR" ship use "$AGENT"); then
  echo "Error: 'ship use $AGENT' failed in $WORKTREE_PATH. Agent not launched." >&2
  exit 1
fi

# ── Verify MCP config ─────────────────────────────────────────────────────────
MCP_JSON="$WORKTREE_PATH/.mcp.json"
if [[ ! -f "$MCP_JSON" ]]; then
  echo "Error: .mcp.json not found in $WORKTREE_PATH after 'ship use'. Agent not launched." >&2
  exit 1
fi
if ! (grep -q '"command".*"ship"' "$MCP_JSON" && grep -q '"mcp"' "$MCP_JSON" && grep -q '"serve"' "$MCP_JSON"); then
  echo "Error: .mcp.json missing ship mcp serve config. Agent not launched." >&2
  exit 1
fi
echo "MCP config verified: $MCP_JSON"

# ── Detect provider CLI ────────────────────────────────────────────────────────
PROVIDER_CLI="${SHIP_PROVIDER_CLI:-}"
if [[ -z "$PROVIDER_CLI" ]]; then
  # Read the first provider from the compiled agent config
  if [[ -f "$WORKTREE_PATH/CLAUDE.md" ]]; then
    PROVIDER_CLI="claude"
  elif [[ -f "$WORKTREE_PATH/AGENTS.md" ]]; then
    PROVIDER_CLI="codex"
  elif [[ -f "$WORKTREE_PATH/GEMINI.md" ]]; then
    PROVIDER_CLI="gemini"
  elif [[ -d "$WORKTREE_PATH/.cursor" ]]; then
    PROVIDER_CLI="cursor"
  elif [[ -f "$WORKTREE_PATH/.opencode.json" ]] || [[ -d "$WORKTREE_PATH/.opencode" ]]; then
    PROVIDER_CLI="opencode"
  else
    PROVIDER_CLI="claude"  # fallback
  fi
fi

# ── Open terminal ─────────────────────────────────────────────────────────────
# Build launch command with provider-specific flags
LAUNCH_FLAGS=""
MODEL_FLAG=""
if [[ -n "${SHIP_AGENT_MODEL:-}" ]]; then
  MODEL_FLAG="--model ${SHIP_AGENT_MODEL}"
fi
if [[ "$PROVIDER_CLI" == "claude" ]]; then
  LAUNCH_FLAGS="--dangerously-skip-permissions --dangerously-load-development-channels server:ship ${MODEL_FLAG}"
elif [[ "$PROVIDER_CLI" == "codex" ]]; then
  LAUNCH_FLAGS="${MODEL_FLAG}"
fi
LAUNCH_CMD="cd ${WORKTREE_PATH} && ${PROVIDER_CLI} ${LAUNCH_FLAGS}"

if $NO_OPEN; then
  echo ""
  echo "Launch manually:"
  echo "  $LAUNCH_CMD"
  exit 0
fi

case "$TERMINAL" in
  iterm)
    osascript -e "
      tell application \"iTerm2\"
        tell current window
          set newTab to (create tab with default profile)
          tell current session of newTab
            write text \"$LAUNCH_CMD\"
            set name to \"$SLUG\"
          end tell
        end tell
      end tell" 2>/dev/null || echo "iTerm2 AppleScript failed. Launch manually: $LAUNCH_CMD"
    ;;
  tmux)
    tmux new-window -d -n "$SLUG" "$LAUNCH_CMD" 2>/dev/null || { echo "tmux failed. Launch manually: $LAUNCH_CMD"; exit 1; }
    # Wait for channel confirmation prompt and accept it, then kick job autostart.
    # Loops up to 15s waiting for the prompt before giving up.
    (
      for i in $(seq 1 15); do
        sleep 1
        if tmux capture-pane -t "$SLUG" -p 2>/dev/null | grep -q "I am using this for local development"; then
          tmux send-keys -t "$SLUG" Enter
          sleep 1
          tmux send-keys -t "$SLUG" "Read .ship-session/job-spec.md and execute it autonomously." Enter
          exit 0
        fi
      done
      # No channel prompt found — still kick autostart (no-channels path)
      tmux send-keys -t "$SLUG" "Read .ship-session/job-spec.md and execute it autonomously." Enter
    ) &
    ;;
  warp)
    open -a Warp --args --working-directory "$WORKTREE_PATH" 2>/dev/null || echo "Warp launch failed. Launch manually: $LAUNCH_CMD"
    ;;
  vscode)
    code "$WORKTREE_PATH" 2>/dev/null || echo "VS Code launch failed. Launch manually: $LAUNCH_CMD"
    ;;
  wt)
    # From WSL: wt.exe new-tab opens a new tab in the current WT window.
    # Use bash --login so ~/.bash_profile is sourced (PATH includes ~/.local/bin etc).
    # Detect the running WSL distro name rather than hardcoding.
    WSL_DISTRO="${WSL_DISTRO_NAME:-Ubuntu}"
    wt.exe new-tab --title "$SLUG" -- wsl.exe -d "$WSL_DISTRO" bash --login -c "$LAUNCH_CMD" 2>/dev/null || \
    echo "Windows Terminal launch failed. Launch manually: $LAUNCH_CMD"
    ;;
  *)
    echo ""
    echo "Launch in a new terminal:"
    echo "  $LAUNCH_CMD"
    ;;
esac

echo ""
echo "Dispatched: $SLUG → $AGENT @ $WORKTREE_PATH"
