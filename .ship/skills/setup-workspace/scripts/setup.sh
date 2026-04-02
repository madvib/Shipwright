#!/usr/bin/env bash
# Ship workspace setup — configure a multi-agent terminal environment.
# Reads .ship-session/pod.md if it exists, otherwise prompts for agents.
# Usage: bash .ship/skills/setup-workspace/scripts/setup.sh [--pod <file>] [--dry-run]
set -euo pipefail

WORKTREE_BASE="${SHIP_WORKTREE_DIR:-${HOME}/dev/ship-worktrees}"
POD_FILE=".ship-session/pod.md"
DRY_RUN=false
PROJECT_ROOT="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --pod)      POD_FILE="$2"; shift 2 ;;
    --dry-run)  DRY_RUN=true; shift ;;
    *) echo "Unknown option: $1" >&2; exit 1 ;;
  esac
done

# ── Detect terminal ────────────────────────────────────────────────────────────
detect_terminal() {
  local term="${SHIP_DEFAULT_TERMINAL:-}"
  if [[ -n "$term" ]]; then echo "$term"; return; fi
  if [[ -n "${TMUX:-}" ]]; then echo "tmux"; return; fi
  if [[ -n "${WT_SESSION:-}" ]]; then echo "wt"; return; fi
  case "${TERM_PROGRAM:-}" in
    iTerm.app)      echo "iterm" ;;
    WarpTerminal)   echo "warp" ;;
    vscode)         echo "vscode" ;;
    *)              echo "manual" ;;
  esac
}

TERMINAL=$(detect_terminal)

# ── Parse pod.md ───────────────────────────────────────────────────────────────
# Expects YAML-ish front matter:
#   terminal: tmux
#   agents:
#     - name: rust
#       agent: rust-runtime
#       path: /home/user/dev/ship-worktrees/rust-work

declare -a AGENT_NAMES=()
declare -a AGENT_PROFILES=()
declare -a AGENT_PATHS=()

if [[ -f "$POD_FILE" ]]; then
  echo "Reading pod layout: $POD_FILE"
  # Parse agent blocks: lines starting with "    - name:", "      agent:", "      path:"
  current_name=""
  current_agent=""
  current_path=""
  while IFS= read -r line; do
    if [[ "$line" =~ ^[[:space:]]*-[[:space:]]*name:[[:space:]]*(.+)$ ]]; then
      # Save previous block
      if [[ -n "$current_name" && -n "$current_agent" ]]; then
        AGENT_NAMES+=("$current_name")
        AGENT_PROFILES+=("$current_agent")
        AGENT_PATHS+=("${current_path:-$WORKTREE_BASE/$current_name}")
      fi
      current_name="${BASH_REMATCH[1]}"
      current_agent=""
      current_path=""
    elif [[ "$line" =~ ^[[:space:]]*agent:[[:space:]]*(.+)$ ]]; then
      current_agent="${BASH_REMATCH[1]}"
    elif [[ "$line" =~ ^[[:space:]]*path:[[:space:]]*(.+)$ ]]; then
      current_path="${BASH_REMATCH[1]}"
    fi
  done < "$POD_FILE"
  # Save last block
  if [[ -n "$current_name" && -n "$current_agent" ]]; then
    AGENT_NAMES+=("$current_name")
    AGENT_PROFILES+=("$current_agent")
    AGENT_PATHS+=("${current_path:-$WORKTREE_BASE/$current_name}")
  fi
else
  echo "No pod file found at $POD_FILE"
  echo "Create one to save your layout, or pass agent names:"
  echo "  bash .ship/skills/setup-workspace/scripts/setup.sh --pod .ship-session/pod.md"
  echo ""
  echo "Example pod.md:"
  cat << 'EXAMPLE'
agents:
  - name: rust
    agent: rust-runtime
    path: ~/dev/ship-worktrees/rust-work
  - name: web
    agent: web-lane
    path: ~/dev/ship-worktrees/web-work
EXAMPLE
  exit 0
fi

if [[ ${#AGENT_NAMES[@]} -eq 0 ]]; then
  echo "No agents found in $POD_FILE" >&2
  exit 1
fi

echo "Terminal: $TERMINAL"
echo "Agents: ${AGENT_NAMES[*]}"
echo ""

if $DRY_RUN; then
  echo "dry run — would open:"
  for i in "${!AGENT_NAMES[@]}"; do
    echo "  ${AGENT_NAMES[$i]}: cd ${AGENT_PATHS[$i]} && claude --dangerously-skip-permissions"
  done
  exit 0
fi

# ── Open panes ─────────────────────────────────────────────────────────────────
open_tmux() {
  local name="$1" path="$2"
  if tmux list-windows | grep -q "^[0-9]*: ${name}"; then
    echo "  tmux window '$name' already exists — skipping"
    return
  fi
  tmux new-window -d -n "$name" "cd $path && claude --dangerously-skip-permissions --dangerously-load-development-channels server:ship"
  echo "  opened: $name"
}

open_iterm() {
  local name="$1" path="$2"
  osascript -e "
    tell application \"iTerm2\"
      tell current window
        set newTab to (create tab with default profile)
        tell current session of newTab
          write text \"cd $path && claude --dangerously-skip-permissions --dangerously-load-development-channels server:ship\"
          set name to \"$name\"
        end tell
      end tell
    end tell" 2>/dev/null || echo "  iTerm2 failed for $name"
}

open_wt() {
  local name="$1" path="$2"
  wt.exe -w 0 nt --title "$name" -- wsl.exe bash -i -c "cd $path && claude --dangerously-skip-permissions --dangerously-load-development-channels server:ship" 2>/dev/null \
    || echo "  WT failed for $name"
}

open_manual() {
  local name="$1" path="$2"
  echo "  $name:  cd $path && claude --dangerously-skip-permissions --dangerously-load-development-channels server:ship"
}

for i in "${!AGENT_NAMES[@]}"; do
  name="${AGENT_NAMES[$i]}"
  path="${AGENT_PATHS[$i]/#\~/$HOME}"
  echo "Opening: $name → $path"
  case "$TERMINAL" in
    tmux)   open_tmux "$name" "$path" ;;
    iterm)  open_iterm "$name" "$path" ;;
    wt)     open_wt "$name" "$path" ;;
    *)      open_manual "$name" "$path" ;;
  esac
done

echo ""
echo "Pod ready. ${#AGENT_NAMES[@]} agent(s) launched."
