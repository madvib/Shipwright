#!/usr/bin/env sh
set -eu

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT_DIR"

echo "[smoke] running ship migrate"
cargo run -p cli -- migrate

echo "[smoke] listing migrated project entities"
cargo run -p cli -- issue list || true
cargo run -p cli -- spec list || true
cargo run -p cli -- feature list || true
cargo run -p cli -- release list || true

echo "[smoke] verifying required namespace directories"
test -d .ship/project/specs
test -d .ship/project/features
test -d .ship/project/adrs
test -d .ship/project/releases
test -f .ship/ship.db

echo "[smoke] migration smoke checks passed"
