#!/usr/bin/env bash
# Quick-run the Ship dev container with Podman (ephemeral, --rm).
# For persistent sessions use ship-dev.sh instead.
# Usage: contrib/run.sh [extra podman args]

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

podman run -it --rm \
  --userns=keep-id \
  -v "$REPO_ROOT":/workspaces/ship:Z \
  -v ship-cargo-registry:/home/dev/.cargo/registry:Z \
  -v ship-cargo-git:/home/dev/.cargo/git:Z \
  -v ship-target:/workspaces/ship/target:Z \
  -v ship-pnpm-store:/home/dev/.local/share/pnpm/store:Z \
  -w /workspaces/ship \
  -p 3000:3000 \
  -p 3001:3001 \
  -p 3002:3002 \
  -p 7701:7701 \
  "$@" \
  ship-dev
