#!/usr/bin/env bash
# Ship dev setup — run once on a fresh machine
# Works on Linux and macOS (Intel + Apple Silicon)
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PARENT_DIR="$(dirname "$REPO_ROOT")"

echo "==> Ship dev setup"
echo "    Repo: $REPO_ROOT"
echo ""

# ── 1. Rust ─────────────────────────────────────────────────────────────────
if ! command -v cargo &>/dev/null; then
  echo "==> Installing Rust..."
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --no-modify-path
  source "$HOME/.cargo/env"
fi
echo "==> Rust: $(rustc --version)"

# rust-toolchain.toml will pin the correct version on first cargo invocation
# wasm32 target needed for compiler crate
rustup target add wasm32-unknown-unknown 2>/dev/null || true

# ── 2. wasm-pack ─────────────────────────────────────────────────────────────
if ! command -v wasm-pack &>/dev/null; then
  echo "==> Installing wasm-pack..."
  cargo install wasm-pack --locked
fi
echo "==> wasm-pack: $(wasm-pack --version)"

# ── 3. Node / pnpm ───────────────────────────────────────────────────────────
if ! command -v node &>/dev/null; then
  echo "ERROR: Node.js not found. Install Node 20+ from https://nodejs.org or via nvm/fnm"
  exit 1
fi
echo "==> Node: $(node --version)"

if ! command -v pnpm &>/dev/null; then
  echo "==> Installing pnpm..."
  npm install -g pnpm
fi
echo "==> pnpm: $(pnpm --version)"

# ── 4. Install ship binary from source ───────────────────────────────────────
echo "==> Building + installing ship binary..."
cargo install --path "$REPO_ROOT/apps/ship-studio-cli" --locked --force
echo "==> ship: $(ship --version)"

# ── 5. pnpm install ──────────────────────────────────────────────────────────
echo "==> Installing JS dependencies..."
cd "$REPO_ROOT" && pnpm install

# ── 6. Claude Code plugins (superpowers + tools) ─────────────────────────────
if command -v claude &>/dev/null; then
  echo "==> Installing Claude Code plugins (user scope)..."
  claude plugin install superpowers@claude-plugins-official 2>/dev/null || echo "  superpowers: already installed or unavailable"
  claude plugin install frontend-design@claude-plugins-official 2>/dev/null || echo "  frontend-design: already installed or unavailable"
  claude plugin install rust-analyzer-lsp@claude-plugins-official 2>/dev/null || echo "  rust-analyzer-lsp: already installed or unavailable"
else
  echo "  (claude CLI not found — install Claude Code to get plugin support)"
fi

# ── 7. Worktrees ─────────────────────────────────────────────────────────────
echo ""
echo "==> Setting up worktrees..."
cd "$REPO_ROOT"
git fetch --all --quiet

setup_worktree() {
  local name="$1"
  local branch="$2"
  local dir="$PARENT_DIR/$name"
  if [ -d "$dir" ]; then
    echo "  $name: already exists, pulling..."
    cd "$dir" && git pull --quiet && cd "$REPO_ROOT"
  else
    echo "  $name: creating..."
    git worktree add "$dir" "$branch"
  fi
}

setup_worktree "ship-cli"    "feat/cli-init"
setup_worktree "ship-server" "feat/server-auth"
setup_worktree "ship-web"    "feat/web-import"

# ── 8. Clean stale Claude config ─────────────────────────────────────────────
echo ""
echo "==> Checking Claude config..."
CLAUDE_SETTINGS="$HOME/.claude/settings.json"
if [ -f "$CLAUDE_SETTINGS" ]; then
  # Check for stale MCP entries from old ship binary (the workflow-layer binary)
  if python3 -c "
import json, sys
with open('$CLAUDE_SETTINGS') as f:
    d = json.load(f)
mcp = d.get('mcpServers', {})
stale = [k for k in mcp if 'ship' in k.lower()]
if stale:
    print('STALE: ' + ', '.join(stale))
    sys.exit(1)
" 2>/dev/null; then
    echo "  Claude settings: clean"
  else
    echo "  WARNING: stale MCP entries found in $CLAUDE_SETTINGS"
    echo "  Remove old ship MCP server entries — the new binary uses 'ship mcp' directly"
  fi
fi

echo ""
echo "✓ Setup complete"
echo ""
echo "Worktrees:"
git worktree list
echo ""
echo "Next:"
echo "  cd ../ship-cli    — CLI lane (feat/cli-init)"
echo "  cd ../ship-server — Server lane (feat/server-auth)"
echo "  cd ../ship-web    — Web lane (feat/web-import)"
echo ""
echo "Open each worktree in Claude Code and say: 'Read BRIEF.md then execute all tasks in priority order.'"
