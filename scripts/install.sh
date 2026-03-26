#!/usr/bin/env sh
# Ship installer — curl -fsSL https://getship.dev/install.sh | sh
#
# Detects platform/arch, downloads the latest release binary from GitHub,
# and installs it to ~/.ship/bin (or SHIP_INSTALL_DIR).
#
# Usage:
#   curl -fsSL https://getship.dev/install.sh | sh
#   curl -fsSL https://getship.dev/install.sh | sh -s -- --dry-run
#
# Environment variables:
#   SHIP_VERSION       — pin a specific version (e.g. "0.1.0"). Default: latest.
#   SHIP_INSTALL_DIR   — installation directory. Default: ~/.ship/bin.
#   SHIP_GITHUB_REPO   — GitHub repo. Default: madvib/ship.
set -eu

REPO="${SHIP_GITHUB_REPO:-madvib/ship}"
INSTALL_DIR="${SHIP_INSTALL_DIR:-$HOME/.ship/bin}"
BINARY_NAME="ship"
DRY_RUN=0

# ── Argument parsing ─────────────────────────────────────────────────────────

for arg in "$@"; do
    case "$arg" in
        --dry-run) DRY_RUN=1 ;;
        *) printf "error: unknown argument: %s\n" "$arg" >&2; exit 1 ;;
    esac
done

# ── Helpers ──────────────────────────────────────────────────────────────────

say() { printf "  %s\n" "$@"; }
err() { printf "error: %s\n" "$@" >&2; exit 1; }

# Download $1 to $2 using curl or wget.
download() {
    local url="$1"
    local dest="$2"
    if command -v curl > /dev/null 2>&1; then
        if ! curl -fsSL "$url" -o "$dest" 2>/dev/null; then
            printf "error: download failed: %s\n" "$url" >&2
            printf "\n  Download manually: https://github.com/%s/releases/latest\n" "$REPO" >&2
            exit 1
        fi
    elif command -v wget > /dev/null 2>&1; then
        if ! wget -q "$url" -O "$dest" 2>/dev/null; then
            printf "error: download failed: %s\n" "$url" >&2
            printf "\n  Download manually: https://github.com/%s/releases/latest\n" "$REPO" >&2
            exit 1
        fi
    else
        err "neither curl nor wget found. Install one and retry:
    macOS:  brew install curl
    Debian: apt-get install curl
    Alpine: apk add curl"
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
        *)
            err "unsupported OS: $os
  Supported: Linux, macOS (Darwin)
  Open an issue: https://github.com/${REPO}/issues"
            ;;
    esac

    arch="$(uname -m)"
    case "$arch" in
        x86_64|amd64)  arch="x86_64" ;;
        aarch64|arm64) arch="aarch64" ;;
        *)
            err "unsupported architecture: $arch
  Supported: x86_64, aarch64/arm64
  Open an issue: https://github.com/${REPO}/issues"
            ;;
    esac

    echo "${arch}-${os}"
}

# ── Resolve version ─────────────────────────────────────────────────────────

resolve_version() {
    if [ -n "${SHIP_VERSION:-}" ]; then
        echo "v${SHIP_VERSION#v}"
        return
    fi

    local tag
    local api_url="https://api.github.com/repos/${REPO}/releases/latest"
    local tmpfile
    tmpfile="$(mktemp)"
    download "$api_url" "$tmpfile"
    tag="$(grep '"tag_name"' "$tmpfile" | head -1 | sed 's/.*"tag_name": *"//;s/".*//')"
    rm -f "$tmpfile"

    if [ -z "$tag" ]; then
        err "could not determine latest version.
  Set SHIP_VERSION manually, e.g.:
    SHIP_VERSION=0.1.0 curl -fsSL https://getship.dev/install.sh | sh"
    fi
    echo "$tag"
}

# ── Download & install ───────────────────────────────────────────────────────

main() {
    local platform version url archive_name ext tmpdir binary

    printf "\n  Ship installer\n\n"

    platform="$(detect_platform)"
    say "platform: $platform"

    version="$(resolve_version)"
    say "version:  $version"

    archive_name="ship-${platform}"

    ext="tar.gz"
    case "$platform" in
        *windows*) ext="zip" ;;
    esac

    url="https://github.com/${REPO}/releases/download/${version}/${archive_name}.${ext}"
    say "url:      $url"
    say "dest:     ${INSTALL_DIR}/${BINARY_NAME}"
    say ""

    if [ "$DRY_RUN" = "1" ]; then
        say "(dry run — nothing downloaded or installed)"
        say ""
        return
    fi

    tmpdir="$(mktemp -d)"
    trap 'rm -rf "$tmpdir"' EXIT

    say "downloading..."
    download "$url" "${tmpdir}/archive.${ext}"

    say "extracting..."
    case "$ext" in
        tar.gz)
            tar -xzf "${tmpdir}/archive.tar.gz" -C "$tmpdir"
            ;;
        zip)
            if ! command -v unzip > /dev/null 2>&1; then
                err "unzip not found. Install it and retry:
    Debian: apt-get install unzip"
            fi
            unzip -q "${tmpdir}/archive.zip" -d "$tmpdir"
            ;;
    esac

    # Find the binary — may be at root or in a subdirectory.
    binary="$(find "$tmpdir" -name "$BINARY_NAME" -type f | head -1)"
    if [ -z "$binary" ]; then
        binary="$(find "$tmpdir" -name "${BINARY_NAME}.exe" -type f | head -1)"
    fi
    if [ -z "$binary" ]; then
        err "binary not found in archive. Report at: https://github.com/${REPO}/issues"
    fi

    if ! mkdir -p "$INSTALL_DIR" 2>/dev/null; then
        err "cannot create ${INSTALL_DIR}. Try a different install dir:
    SHIP_INSTALL_DIR=~/bin curl -fsSL https://getship.dev/install.sh | sh
  Or install with sudo (not recommended):
    sudo SHIP_INSTALL_DIR=/usr/local/bin curl -fsSL https://getship.dev/install.sh | sh"
    fi

    if ! cp "$binary" "${INSTALL_DIR}/${BINARY_NAME}" 2>/dev/null; then
        err "cannot write to ${INSTALL_DIR}. Try:
    SHIP_INSTALL_DIR=~/bin curl -fsSL https://getship.dev/install.sh | sh"
    fi
    chmod +x "${INSTALL_DIR}/${BINARY_NAME}"

    say "installed to ${INSTALL_DIR}/${BINARY_NAME}"
    say ""

    # Check if INSTALL_DIR is in PATH.
    case ":${PATH}:" in
        *":${INSTALL_DIR}:"*) ;;
        *)
            say "Add ship to your PATH. Choose your shell:"
            say ""
            say "  bash:  echo 'export PATH=\"${INSTALL_DIR}:\$PATH\"' >> ~/.bashrc && source ~/.bashrc"
            say "  zsh:   echo 'export PATH=\"${INSTALL_DIR}:\$PATH\"' >> ~/.zshrc && source ~/.zshrc"
            say "  fish:  fish_add_path ${INSTALL_DIR}"
            say ""
            ;;
    esac

    say "run 'ship init' to get started"
    say ""

    if command -v "${INSTALL_DIR}/${BINARY_NAME}" > /dev/null 2>&1; then
        "${INSTALL_DIR}/${BINARY_NAME}" --version
    fi
}

main "$@"
