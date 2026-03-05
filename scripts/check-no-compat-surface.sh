#!/usr/bin/env bash
set -euo pipefail

status=0

fail() {
  echo "FAIL: $1" >&2
  status=1
}

echo "[check-no-compat-surface] scanning CLI/MCP surfaces"

# 1) Clap aliases are not allowed for compatibility paths.
if rg -n --pcre2 "(visible_alias|\\balias\\s*=)" \
  crates/cli/src/surface.rs \
  core/cli-framework/src/lib.rs >/tmp/ship-no-compat-alias.txt; then
  cat /tmp/ship-no-compat-alias.txt >&2
  fail "clap aliases detected; remove compatibility aliases."
fi

# 2) Top-level `migrate` command must not exist (dev-only under `ship dev migrate`).
commands_block="$(
  awk '
    /pub enum Commands[[:space:]]*{/ { in_block=1; next }
    in_block && /^}/ { in_block=0; exit }
    in_block { print }
  ' crates/cli/src/surface.rs
)"
if printf "%s\n" "$commands_block" | rg -n "^[[:space:]]*Migrate[[:space:]]*\\{" >/tmp/ship-no-compat-top-level.txt; then
  cat /tmp/ship-no-compat-top-level.txt >&2
  fail "top-level migrate command found; use `ship dev migrate` only."
fi

# 3) Explicit legacy routing in CLI handlers should be absent.
if rg -n --pcre2 "\\bCommands::Migrate\\b" crates/cli/src/app.rs >/tmp/ship-no-compat-route.txt; then
  cat /tmp/ship-no-compat-route.txt >&2
  fail "legacy command routing found in CLI handlers."
fi

if [[ "$status" -eq 0 ]]; then
  echo "[check-no-compat-surface] OK"
fi

exit "$status"
