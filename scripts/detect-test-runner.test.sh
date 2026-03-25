#!/usr/bin/env bash
# Tests for detect-test-runner.sh
# Each test creates a temp project directory with markers, runs detection, asserts output.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DETECT="$SCRIPT_DIR/detect-test-runner.sh"
PASS=0
FAIL=0

assert_eq() {
  local test_name="$1" expected="$2" actual="$3"
  if [[ "$expected" == "$actual" ]]; then
    echo "  PASS: $test_name"
    PASS=$((PASS + 1))
  else
    echo "  FAIL: $test_name"
    echo "    expected: $expected"
    echo "    actual:   $actual"
    FAIL=$((FAIL + 1))
  fi
}

setup_tmp() {
  mktemp -d
}

teardown_tmp() {
  rm -rf "$1"
}

# --- Test 1: Cargo.toml present → cargo test ---
test_cargo() {
  local tmp
  tmp=$(setup_tmp)
  touch "$tmp/Cargo.toml"
  local result
  result=$("$DETECT" "$tmp")
  assert_eq "Cargo.toml → cargo test" "cargo test" "$result"
  teardown_tmp "$tmp"
}

# --- Test 2: package.json with vitest → npx vitest run ---
test_vitest() {
  local tmp
  tmp=$(setup_tmp)
  echo '{"devDependencies":{"vitest":"^1.0.0"}}' > "$tmp/package.json"
  local result
  result=$("$DETECT" "$tmp" 2>/dev/null || true)
  assert_eq "package.json + vitest → npx vitest run" "npx vitest run" "$result"
  teardown_tmp "$tmp"
}

# --- Test 3: package.json with jest → npx jest ---
test_jest() {
  local tmp
  tmp=$(setup_tmp)
  echo '{"devDependencies":{"jest":"^29.0.0"}}' > "$tmp/package.json"
  local result
  result=$("$DETECT" "$tmp" 2>/dev/null || true)
  assert_eq "package.json + jest → npx jest" "npx jest" "$result"
  teardown_tmp "$tmp"
}

# --- Test 4: pyproject.toml with pytest → pytest ---
test_pytest() {
  local tmp
  tmp=$(setup_tmp)
  cat > "$tmp/pyproject.toml" <<'TOML'
[tool.pytest.ini_options]
testpaths = ["tests"]
TOML
  local result
  result=$("$DETECT" "$tmp" 2>/dev/null || true)
  assert_eq "pyproject.toml + pytest → pytest" "pytest" "$result"
  teardown_tmp "$tmp"
}

# --- Test 5: go.mod → go test ./... ---
test_go() {
  local tmp
  tmp=$(setup_tmp)
  echo 'module example.com/foo' > "$tmp/go.mod"
  local result
  result=$("$DETECT" "$tmp" 2>/dev/null || true)
  assert_eq "go.mod → go test ./..." "go test ./..." "$result"
  teardown_tmp "$tmp"
}

# --- Test 6: Makefile with test target → make test ---
test_makefile() {
  local tmp
  tmp=$(setup_tmp)
  cat > "$tmp/Makefile" <<'MAKE'
test:
	echo "running tests"
MAKE
  local result
  result=$("$DETECT" "$tmp" 2>/dev/null || true)
  assert_eq "Makefile with test target → make test" "make test" "$result"
  teardown_tmp "$tmp"
}

# --- Test 7: No markers → exit 1 with "unknown" ---
test_none() {
  local tmp
  tmp=$(setup_tmp)
  local result
  result=$("$DETECT" "$tmp" 2>/dev/null || true)
  assert_eq "no markers → unknown" "unknown" "$result"
  teardown_tmp "$tmp"
}

# --- Test 8: vitest in dependencies (not devDependencies) ---
test_vitest_deps() {
  local tmp
  tmp=$(setup_tmp)
  echo '{"dependencies":{"vitest":"^1.0.0"}}' > "$tmp/package.json"
  local result
  result=$("$DETECT" "$tmp" 2>/dev/null || true)
  assert_eq "vitest in dependencies → npx vitest run" "npx vitest run" "$result"
  teardown_tmp "$tmp"
}

# --- Test 9: vitest takes priority over jest ---
test_vitest_over_jest() {
  local tmp
  tmp=$(setup_tmp)
  echo '{"devDependencies":{"vitest":"^1.0.0","jest":"^29.0.0"}}' > "$tmp/package.json"
  local result
  result=$("$DETECT" "$tmp" 2>/dev/null || true)
  assert_eq "vitest+jest → vitest wins" "npx vitest run" "$result"
  teardown_tmp "$tmp"
}

# --- Test 10: Cargo.toml takes priority over package.json ---
test_cargo_over_node() {
  local tmp
  tmp=$(setup_tmp)
  touch "$tmp/Cargo.toml"
  echo '{"devDependencies":{"vitest":"^1.0.0"}}' > "$tmp/package.json"
  local result
  result=$("$DETECT" "$tmp" 2>/dev/null || true)
  assert_eq "Cargo.toml + package.json → cargo test" "cargo test" "$result"
  teardown_tmp "$tmp"
}

# --- Test 11: Makefile without test target → unknown ---
test_makefile_no_test() {
  local tmp
  tmp=$(setup_tmp)
  cat > "$tmp/Makefile" <<'MAKE'
build:
	echo "building"
MAKE
  local result
  result=$("$DETECT" "$tmp" 2>/dev/null || true)
  assert_eq "Makefile without test target → unknown" "unknown" "$result"
  teardown_tmp "$tmp"
}

# --- Test 12: pyproject.toml without pytest → unknown ---
test_pyproject_no_pytest() {
  local tmp
  tmp=$(setup_tmp)
  cat > "$tmp/pyproject.toml" <<'TOML'
[tool.black]
line-length = 88
TOML
  local result
  result=$("$DETECT" "$tmp" 2>/dev/null || true)
  assert_eq "pyproject.toml without pytest → unknown" "unknown" "$result"
  teardown_tmp "$tmp"
}

# --- Test 13: package.json with no test framework → unknown ---
test_package_no_framework() {
  local tmp
  tmp=$(setup_tmp)
  echo '{"name":"my-app","dependencies":{"express":"^4.0.0"}}' > "$tmp/package.json"
  local result
  result=$("$DETECT" "$tmp" 2>/dev/null || true)
  assert_eq "package.json without test deps → unknown" "unknown" "$result"
  teardown_tmp "$tmp"
}

# --- Test 14: defaults to current directory ---
test_default_cwd() {
  local tmp
  tmp=$(setup_tmp)
  touch "$tmp/Cargo.toml"
  local result
  result=$(cd "$tmp" && "$DETECT" 2>/dev/null || true)
  assert_eq "no arg → uses cwd" "cargo test" "$result"
  teardown_tmp "$tmp"
}

# --- Run ---
echo "detect-test-runner tests"
echo "========================"
test_cargo
test_vitest
test_jest
test_pytest
test_go
test_makefile
test_none
test_vitest_deps
test_vitest_over_jest
test_cargo_over_node
test_makefile_no_test
test_pyproject_no_pytest
test_package_no_framework
test_default_cwd

echo ""
echo "Results: $PASS passed, $FAIL failed"
[[ $FAIL -eq 0 ]] || exit 1
