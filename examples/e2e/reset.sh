#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
WORK_DIR="$ROOT_DIR/examples/e2e"
SHIP_BIN="$ROOT_DIR/target/debug/ship"

echo "Resetting example workspace at $WORK_DIR"
rm -rf "$WORK_DIR/.ship" "$WORK_DIR/.tmp"
mkdir -p "$WORK_DIR/.tmp"

echo "Building latest CLI binary..."
cargo build --manifest-path "$ROOT_DIR/Cargo.toml" -p cli >/dev/null

echo "Seeding fresh .ship data..."
(cd "$WORK_DIR" && "$SHIP_BIN" demo . >/dev/null)

echo "Done. Fresh workspace ready at $WORK_DIR"
