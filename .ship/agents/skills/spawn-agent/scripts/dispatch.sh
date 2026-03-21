#!/usr/bin/env bash
set -euo pipefail

# dispatch.sh — Idempotent job dispatch to a git worktree
# Creates worktree, writes job spec, shows spec for approval, compiles agent, opens terminal.
#
# Usage:
#   bash scripts/dispatch.sh --slug <slug> --agent <agent> --spec <spec-file> [OPTIONS]
#
# Options:
#   --slug <slug>        Worktree/branch name (e.g. jsonc-config)
#   --agent <agent>      Ship agent profile (e.g. rust-compiler, default)
#   --spec <file>        Path to job-spec.md to copy into worktree
#   --base <branch>      Base branch to fork from (default: current branch)
#   --dir <path>         Worktree base directory (default: ~/dev/ship-worktrees)
#   --no-open            Skip terminal auto-open, just print the launch command
#   --dry-run            Show what would happen without doing anything
#   --confirm            Show spec and ask y/n before launching agent
#   --help               Show this help
#
# Environment variables:
#   SHIP_DEFAULT_TERMINAL   Terminal to open: wt, iterm, tmux, gnome, vscode, manual
#                           Auto-detected if unset.
#   SHIP_DISPATCH_CONFIRM   Set to 1 to always confirm before launching agent.
#                           Equivalent to --confirm on every call.
#   SHIP_WORKTREE_DIR       Default worktree base directory.
#                           Overridden by --dir flag.
#
# Examples:
#   bash scripts/dispatch.sh --slug jsonc-config --agent rust-compiler --spec /tmp/spec.md
#   bash scripts/dispatch.sh --slug docs --agent default --spec specs/docs.md --no-open
#   SHIP_DISPATCH_CONFIRM=1 bash scripts/dispatch.sh --slug tui --agent cli-lane --spec s.md

SLUG=""
AGENT=""
SPEC=""
BASE=""
WORKTREE_DIR=""
NO_OPEN=false
DRY_RUN=false
CONFIRM=false

usage() {
    sed -n '3,/^$/s/^# \?//p' "$0"
    exit "${1:-0}"
}

die() { echo "Error: $*" >&2; exit 1; }

while [[ $# -gt 0 ]]; do
    case "$1" in
        --slug)    SLUG="$2"; shift 2 ;;
        --agent)   AGENT="$2"; shift 2 ;;
        --spec)    SPEC="$2"; shift 2 ;;
        --base)    BASE="$2"; shift 2 ;;
        --dir)     WORKTREE_DIR="$2"; shift 2 ;;
        --no-open) NO_OPEN=true; shift ;;
        --dry-run) DRY_RUN=true; shift ;;
        --confirm) CONFIRM=true; shift ;;
        --help)    usage 0 ;;
        *)         die "Unknown option: $1. Use --help for usage." ;;
    esac
done

# Apply preferences: flag > env var > ship config > default
if ! $CONFIRM && [[ "${SHIP_DISPATCH_CONFIRM:-}" == "1" ]]; then
    CONFIRM=true
elif ! $CONFIRM && command -v ship &>/dev/null; then
    [[ "$(ship config get dispatch.confirm 2>/dev/null)" == "true" ]] && CONFIRM=true
fi

[[ -n "$SLUG" ]]  || die "--slug is required."
[[ -n "$AGENT" ]] || die "--agent is required."
[[ -n "$SPEC" ]]  || die "--spec is required."
[[ -f "$SPEC" ]]  || die "Spec file not found: $SPEC"

# Resolve defaults: flag > env var > ship config > hardcoded
BASE="${BASE:-$(git rev-parse --abbrev-ref HEAD)}"
if [[ -z "$WORKTREE_DIR" ]]; then
    WORKTREE_DIR="${SHIP_WORKTREE_DIR:-}"
    if [[ -z "$WORKTREE_DIR" ]] && command -v ship &>/dev/null; then
        WORKTREE_DIR="$(ship config get worktrees.dir 2>/dev/null || true)"
    fi
    WORKTREE_DIR="${WORKTREE_DIR:-$HOME/dev/ship-worktrees}"
fi

BRANCH="job/$SLUG"
WORKTREE="$WORKTREE_DIR/$SLUG"

# --- Dry run ---
if $DRY_RUN; then
    echo "Would dispatch:"
    echo "  slug:     $SLUG"
    echo "  branch:   $BRANCH"
    echo "  worktree: $WORKTREE"
    echo "  agent:    $AGENT"
    echo "  spec:     $SPEC"
    echo "  base:     $BASE"
    echo "  confirm:  $CONFIRM"
    _t="${SHIP_DEFAULT_TERMINAL:-}"
    [[ -z "$_t" ]] && command -v ship &>/dev/null && _t="$(ship config get terminal.program 2>/dev/null || true)"
    echo "  terminal: ${_t:-auto}"
    exit 0
fi

# --- Step 1: Create worktree (idempotent) ---
if [[ -d "$WORKTREE" ]]; then
    echo "Worktree exists: $WORKTREE (skipping create)"
else
    if git rev-parse --verify "$BRANCH" &>/dev/null; then
        echo "Branch $BRANCH exists, attaching worktree..."
        git worktree add "$WORKTREE" "$BRANCH"
    else
        echo "Creating worktree $WORKTREE on new branch $BRANCH from $BASE..."
        git worktree add -b "$BRANCH" "$WORKTREE" "$BASE"
    fi
fi

# --- Step 2: Write job spec (idempotent) ---
SESSION_DIR="$WORKTREE/.ship-session"
SPEC_DEST="$SESSION_DIR/job-spec.md"

mkdir -p "$SESSION_DIR"

if [[ -f "$SPEC_DEST" ]]; then
    if cmp -s "$SPEC" "$SPEC_DEST"; then
        echo "Job spec unchanged (skipping write)"
    else
        cp "$SPEC" "$SPEC_DEST"
        echo "Job spec updated: $SPEC_DEST"
    fi
else
    cp "$SPEC" "$SPEC_DEST"
    echo "Job spec written: $SPEC_DEST"
fi

# --- Step 3: Confirm (if requested) ---
if $CONFIRM; then
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "  Job: $SLUG"
    echo "  Agent: $AGENT"
    echo "  Branch: $BRANCH"
    echo "  Worktree: $WORKTREE"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""
    # Show spec title + acceptance criteria count
    title=$(head -1 "$SPEC" | sed 's/^# //')
    criteria=$(grep -c '^\- \[ \]' "$SPEC" 2>/dev/null || echo 0)
    mode=$(grep -A1 '^## Mode' "$SPEC" | tail -1 | tr -d ' ')
    echo "  Title: $title"
    echo "  Mode: ${mode:-autonomous}"
    echo "  Acceptance criteria: $criteria"
    echo ""
    # Show scope section if present
    scope=$(sed -n '/^## Scope/,/^## /{ /^## Scope/d; /^## /d; p; }' "$SPEC" | head -15)
    if [[ -n "$scope" ]]; then
        echo "  Scope:"
        echo "$scope" | sed 's/^/    /'
        echo ""
    fi
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    printf "  Launch agent? [y/N/e(dit spec)] "
    read -r answer </dev/tty
    case "$answer" in
        y|Y) echo "  Launching..." ;;
        e|E)
            echo "  Edit the spec at: $SPEC_DEST"
            echo "  Re-run this command when ready."
            exit 0
            ;;
        *)
            echo "  Aborted. Worktree preserved at: $WORKTREE"
            echo "  Re-run with --confirm to try again, or without to skip confirmation."
            exit 0
            ;;
    esac
fi

# --- Step 4: Compile agent config ---
echo "Compiling agent config: ship use $AGENT"
(cd "$WORKTREE" && ship use "$AGENT")

# --- Step 5: Open terminal ---
LAUNCH_CMD="cd $WORKTREE && claude ."

if $NO_OPEN; then
    echo ""
    echo "Dispatched $SLUG"
    echo "  worktree: $WORKTREE"
    echo "  agent:    $AGENT"
    echo "  launch:"
    echo ""
    echo "  $LAUNCH_CMD"
    echo ""
    exit 0
fi

# Resolve terminal: env var > ship config > auto-detect
TERMINAL="${SHIP_DEFAULT_TERMINAL:-}"
if [[ -z "$TERMINAL" ]] && command -v ship &>/dev/null; then
    TERMINAL="$(ship config get terminal.program 2>/dev/null || true)"
fi
TERMINAL="${TERMINAL:-auto}"

if [[ "$TERMINAL" == "auto" ]]; then
    if [[ -n "${WT_SESSION:-}" ]]; then
        TERMINAL="wt"
    elif [[ -n "${TMUX:-}" ]]; then
        TERMINAL="tmux"
    elif [[ "${TERM_PROGRAM:-}" == "iTerm.app" ]]; then
        TERMINAL="iterm"
    elif [[ "${TERM_PROGRAM:-}" == "Apple_Terminal" ]]; then
        TERMINAL="apple-terminal"
    elif [[ "${TERM_PROGRAM:-}" == "vscode" ]]; then
        TERMINAL="vscode"
    elif command -v gnome-terminal &>/dev/null; then
        TERMINAL="gnome"
    else
        TERMINAL="manual"
    fi
fi

opened=false

case "$TERMINAL" in
    wt)
        wt.exe -w 0 nt --title "$SLUG" -d "$(wslpath -w "$WORKTREE")" wsl.exe bash -c "claude ." 2>/dev/null && opened=true
        ;;
    tmux)
        tmux new-window -n "$SLUG" -c "$WORKTREE" "claude ." && opened=true
        ;;
    iterm)
        osascript -e "
            tell application \"iTerm2\"
                tell current window
                    create tab with default profile
                    tell current session
                        write text \"cd $WORKTREE && claude .\"
                    end tell
                end tell
            end tell
        " 2>/dev/null && opened=true
        ;;
    apple-terminal)
        osascript -e "
            tell application \"Terminal\"
                do script \"cd $WORKTREE && claude .\"
            end tell
        " 2>/dev/null && opened=true
        ;;
    vscode)
        # VS Code integrated terminal — open folder in new window
        code "$WORKTREE" 2>/dev/null && opened=true
        ;;
    gnome)
        gnome-terminal --tab --title="$SLUG" -- bash -c "cd $WORKTREE && claude .; exec bash" 2>/dev/null && opened=true
        ;;
    manual)
        # Explicit skip — user wants to paste manually
        ;;
esac

if $opened; then
    echo ""
    echo "Dispatched $SLUG → $TERMINAL"
    echo "  worktree: $WORKTREE"
    echo "  agent:    $AGENT"
else
    echo ""
    echo "Dispatched $SLUG"
    echo "  worktree: $WORKTREE"
    echo "  agent:    $AGENT"
    echo "  launch:"
    echo ""
    echo "  $LAUNCH_CMD"
    echo ""
fi
