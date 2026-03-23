#!/usr/bin/env bash
# ship-dev — launch Ship dev container with Podman (no VSCode)
# Usage: ship-dev [build|start|stop|shell|logs]
#
# Install: cp contrib/ship-dev.sh ~/bin/ship-dev && chmod +x ~/bin/ship-dev
# Or:      alias ship-dev="/path/to/Ship/contrib/ship-dev.sh"
#
# Config via env vars (or edit defaults below):
#   SHIP_REPO          — path to Ship repo (default: ~/dev/Ship)
#   SHIP_CONTAINERFILE — path to Containerfile (default: $SHIP_REPO/.ship-session/Containerfile)

set -euo pipefail

# --- Config ---
REPO_DIR="${SHIP_REPO:-$HOME/dev/Ship}"
CONTAINERFILE="${SHIP_CONTAINERFILE:-$REPO_DIR/contrib/Containerfile}"
IMAGE_NAME="ship-dev"
CONTAINER_NAME="ship"

# --- Volumes (named = persist across rebuilds, bind = host dirs) ---
VOLUMES=(
  # Named volumes — cargo + pnpm caches
  -v shipdev-cargo-registry:/home/dev/.cargo/registry:Z
  -v shipdev-cargo-git:/home/dev/.cargo/git:Z
  -v shipdev-target:/workspaces/ship/target:Z
  -v shipdev-pnpm-store:/home/dev/.local/share/pnpm/store:Z

  # Bind mounts — repo + host config
  -v "$REPO_DIR:/workspaces/ship:Z"
  -v "$HOME/.ship:/home/dev/.ship:Z"
  -v "$HOME/.claude:/home/dev/.claude:Z"
  -v "$HOME/.claude.json:/home/dev/.claude.json:Z"
  -v "$HOME/.config/gh:/home/dev/.config/gh:Z"
  -v "$HOME/.gitconfig:/home/dev/.gitconfig:ro,Z"
)

# Optional host mounts — add only if they exist
[[ -d "$HOME/dev/worktrees" ]]       && VOLUMES+=(-v "$HOME/dev/worktrees:/workspaces/worktrees:Z")
[[ -d "$HOME/dev/ship-internal" ]]   && VOLUMES+=(-v "$HOME/dev/ship-internal:/workspaces/ship-internal:Z")

# Personal dotfiles — mount your configs into the container's XDG dirs.
# Set SHIP_DOTFILES to your dotfiles path, or leave unset to skip.
# Expected structure: starship.toml, helix.toml, bat.conf, etc.
if [[ -n "${SHIP_DOTFILES:-}" && -d "$SHIP_DOTFILES" ]]; then
  [[ -f "$SHIP_DOTFILES/starship.toml" ]] && VOLUMES+=(-v "$SHIP_DOTFILES/starship.toml:/home/dev/.config/starship.toml:ro,Z")
  [[ -f "$SHIP_DOTFILES/helix.toml" ]]    && VOLUMES+=(-v "$SHIP_DOTFILES/helix.toml:/home/dev/.config/helix/config.toml:ro,Z")
  [[ -f "$SHIP_DOTFILES/bat.conf" ]]       && VOLUMES+=(-v "$SHIP_DOTFILES/bat.conf:/home/dev/.config/bat/config:ro,Z")
  [[ -d "$SHIP_DOTFILES/tmux" ]]           && VOLUMES+=(-v "$SHIP_DOTFILES/tmux:/home/dev/.config/tmux:ro,Z")
  [[ -f "$SHIP_DOTFILES/.tmux.conf" ]]     && VOLUMES+=(-v "$SHIP_DOTFILES/.tmux.conf:/home/dev/.tmux.conf:ro,Z")
  [[ -f "$SHIP_DOTFILES/.gitconfig" ]]     && VOLUMES+=(-v "$SHIP_DOTFILES/.gitconfig:/home/dev/.gitconfig:ro,Z")
  [[ -f "$SHIP_DOTFILES/.zshrc" ]]        && VOLUMES+=(-v "$SHIP_DOTFILES/.zshrc:/home/dev/.zshrc:ro,Z")
fi

# --- Ports ---
PORTS=(
  -p 3000:3000   # Ship Studio (Vite dev)
  -p 3001:3001   # Ship Studio (alt)
  -p 3002:3002   # Vite HMR websocket
  -p 6006:6006   # Storybook
  -p 7701:7701   # Ship MCP server (HTTP mode)
)

cmd_build() {
  echo "Building $IMAGE_NAME from $CONTAINERFILE..."
  podman build -f "$CONTAINERFILE" -t "$IMAGE_NAME" "$REPO_DIR"
}

cmd_start() {
  if podman ps -q -f name="^${CONTAINER_NAME}$" 2>/dev/null | grep -q .; then
    echo "Container already running — attaching."
    cmd_shell
    return
  fi

  # Clean up stopped container
  podman rm "$CONTAINER_NAME" 2>/dev/null || true

  # Build if image doesn't exist
  if ! podman image exists "$IMAGE_NAME" 2>/dev/null; then
    cmd_build
  fi

  echo "Starting $CONTAINER_NAME..."
  podman run -d \
    --name "$CONTAINER_NAME" \
    --userns=keep-id \
    "${VOLUMES[@]}" \
    "${PORTS[@]}" \
    -w /workspaces/ship \
    "$IMAGE_NAME" \
    sleep infinity

  # First-run setup: install deps + build if target is empty
  echo "Running first-start setup..."
  podman exec -it "$CONTAINER_NAME" bash -c '
    cd /workspaces/ship
    eval "$($HOME/.local/bin/fnm env)"

    # pnpm install (skip if node_modules looks fresh)
    if [[ ! -d node_modules/.pnpm ]]; then
      echo "Installing JS deps..."
      CI=true pnpm install 2>&1 | tail -3
    fi

    # install ship binary (skip if already on PATH)
    if ! command -v ship &>/dev/null; then
      echo "Building + installing ship..."
      cargo install --path apps/ship-studio-cli 2>&1 | tail -3
    fi
  '

  echo "Attaching tmux..."
  cmd_shell
}

cmd_shell() {
  podman exec -it -w /workspaces/ship "$CONTAINER_NAME" \
    zsh -lc 'tmux -CC -u new-session -A -s ship'
}

cmd_stop() {
  echo "Stopping $CONTAINER_NAME..."
  podman stop "$CONTAINER_NAME" 2>/dev/null
  podman rm "$CONTAINER_NAME" 2>/dev/null
  echo "Done. Volumes preserved — next start will be fast."
}

cmd_exec() {
  podman exec -it -w /workspaces/ship "$CONTAINER_NAME" zsh -lc "$*"
}

cmd_logs() {
  podman logs -f "$CONTAINER_NAME"
}

case "${1:-start}" in
  build) cmd_build ;;
  start) cmd_start ;;
  shell) cmd_shell ;;
  exec)  shift; cmd_exec "$@" ;;
  stop)  cmd_stop  ;;
  logs)  cmd_logs  ;;
  *)
    echo "Usage: ship-dev [build|start|stop|shell|exec|logs]"
    echo ""
    echo "  build  — Build container image from Containerfile"
    echo "  start  — Start container + attach tmux -CC (default)"
    echo "  shell  — Attach to running container's tmux -CC"
    echo "  exec   — Run a command in the container (e.g. ship-dev exec claude)"
    echo "  stop   — Stop container (volumes preserved)"
    echo "  logs   — Tail container logs"
    exit 1
    ;;
esac
