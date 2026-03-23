#!/usr/bin/env sh
# Ship installer — curl -fsSL https://getship.dev/install.sh | sh
#
# Detects platform/arch, downloads the latest release binary from GitHub,
# and installs it to ~/.ship/bin (or SHIP_INSTALL_DIR).
#
# Environment variables:
#   SHIP_VERSION       — pin a specific version (e.g. "0.1.0"). Default: latest.
#   SHIP_INSTALL_DIR   — installation directory. Default: ~/.ship/bin.
#   SHIP_GITHUB_REPO   — GitHub repo. Default: madvib/ship.
set -eu

REPO="${SHIP_GITHUB_REPO:-madvib/ship}"
INSTALL_DIR="${SHIP_INSTALL_DIR:-$HOME/.ship/bin}"
BINARY_NAME="ship"

# ── Helpers ──────────────────────────────────────────────────────────────────

say() { printf "  %s\n" "$@"; }
err() { printf "error: %s\n" "$@" >&2; exit 1; }

need() {
    if ! command -v "$1" > /dev/null 2>&1; then
        err "need '$1' (command not found)"
    fi
}

# ── Detect platform ─────────────────────────────────────────────────────────

detect_platform() {
    local os arch

    os="$(uname -s)"
    case "$os" in
        Linux*)  os="unknown-linux-gnu" ;;
        Darwin*) os="apple-darwin" ;;
        MINGW*|MSYS*|CYGWIN*) os="pc-windows-msvc" ;;
        *) err "unsupported OS: $os" ;;
    esac

    arch="$(uname -m)"
    case "$arch" in
        x86_64|amd64)  arch="x86_64" ;;
        aarch64|arm64) arch="aarch64" ;;
        *) err "unsupported architecture: $arch" ;;
    esac

    echo "${arch}-${os}"
}

# ── Resolve version ─────────────────────────────────────────────────────────

resolve_version() {
    if [ -n "${SHIP_VERSION:-}" ]; then
        echo "v${SHIP_VERSION#v}"
        return
    fi

    need curl
    local tag
    tag="$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" \
        | grep '"tag_name"' \
        | head -1 \
        | sed 's/.*"tag_name": *"//;s/".*//')"

    if [ -z "$tag" ]; then
        err "could not determine latest version. Set SHIP_VERSION manually."
    fi
    echo "$tag"
}

# ── Download & install ───────────────────────────────────────────────────────

main() {
    need curl
    need uname

    local platform version url archive_name

    printf "\n  Ship installer\n\n"

    platform="$(detect_platform)"
    say "platform: $platform"

    version="$(resolve_version)"
    say "version:  $version"

    archive_name="ship-${version#v}-${platform}"

    # Try .tar.gz first (Linux/macOS), fall back to .zip (Windows).
    local ext="tar.gz"
    case "$platform" in
        *windows*) ext="zip" ;;
    esac

    url="https://github.com/${REPO}/releases/download/${version}/${archive_name}.${ext}"
    say "url:      $url"
    say ""

    local tmpdir
    tmpdir="$(mktemp -d)"
    trap 'rm -rf "$tmpdir"' EXIT

    say "downloading..."
    curl -fsSL "$url" -o "${tmpdir}/archive.${ext}"

    say "extracting..."
    case "$ext" in
        tar.gz)
            tar -xzf "${tmpdir}/archive.tar.gz" -C "$tmpdir"
            ;;
        zip)
            need unzip
            unzip -q "${tmpdir}/archive.zip" -d "$tmpdir"
            ;;
    esac

    # Find the binary — may be at root or in a subdirectory.
    local binary
    binary="$(find "$tmpdir" -name "$BINARY_NAME" -type f | head -1)"
    if [ -z "$binary" ]; then
        binary="$(find "$tmpdir" -name "${BINARY_NAME}.exe" -type f | head -1)"
    fi
    if [ -z "$binary" ]; then
        err "binary not found in archive"
    fi

    mkdir -p "$INSTALL_DIR"
    cp "$binary" "${INSTALL_DIR}/${BINARY_NAME}"
    chmod +x "${INSTALL_DIR}/${BINARY_NAME}"

    say "installed to ${INSTALL_DIR}/${BINARY_NAME}"
    say ""

    # Check if INSTALL_DIR is in PATH.
    case ":${PATH}:" in
        *":${INSTALL_DIR}:"*) ;;
        *)
            say "Add to your shell profile:"
            say ""
            say "  export PATH=\"${INSTALL_DIR}:\$PATH\""
            say ""
            ;;
    esac

    say "run 'ship init' to get started"
    say ""
}

main "$@"
