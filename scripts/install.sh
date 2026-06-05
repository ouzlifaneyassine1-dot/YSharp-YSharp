#!/usr/bin/env bash
set -euo pipefail

# Y# Install Script — Linux/macOS
# Usage: curl -fsSL https://raw.githubusercontent.com/ouzlifaneyassine1-dot/YSharp-YSharp/master/scripts/install.sh | bash

VERSION="${1:-latest}"
INSTALL_DIR="${2:-/usr/local/bin}"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

detect_platform() {
    local os
    os="$(uname -s | tr '[:upper:]' '[:lower:]')"
    local arch
    arch="$(uname -m)"

    case "$os" in
        linux) os="unknown-linux-gnu" ;;
        darwin) os="apple-darwin" ;;
        *) echo "Unsupported OS: $os"; exit 1 ;;
    esac

    case "$arch" in
        x86_64|amd64) arch="x86_64" ;;
        aarch64|arm64) arch="aarch64" ;;
        *) echo "Unsupported arch: $arch"; exit 1 ;;
    esac

    echo "${arch}-${os}"
}

install_from_source() {
    echo "==> Installing Y# from source..."

    # Check for Rust
    if ! command -v cargo &>/dev/null; then
        echo "==> Rust not found. Installing via rustup..."
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        source "$HOME/.cargo/env"
    fi

    # Check for GCC
    if ! command -v gcc &>/dev/null; then
        echo "==> GCC not found. Please install GCC (build-essential on Debian, Xcode on macOS)."
        echo "    Debian/Ubuntu: sudo apt install build-essential"
        echo "    macOS:         xcode-select --install"
        echo "    Fedora:        sudo dnf install gcc"
        exit 1
    fi

    cd "$PROJECT_DIR/compiler"
    cargo build --release

    sudo mkdir -p "$INSTALL_DIR"
    sudo cp target/release/oys "$INSTALL_DIR/oys"
    sudo cp target/release/yo "$INSTALL_DIR/yo"

    echo "==> Installed: $INSTALL_DIR/oys"
    echo "==> Installed: $INSTALL_DIR/yo"
}

install_from_release() {
    local platform
    platform="$(detect_platform)"
    local url
    local tag

    if [ "$VERSION" = "latest" ]; then
        tag="$(curl -fsSL https://api.github.com/repos/ouzlifaneyassine1-dot/YSharp-YSharp/releases/latest | grep '"tag_name"' | cut -d'"' -f4)"
    else
        tag="$VERSION"
    fi

    url="https://github.com/ouzlifaneyassine1-dot/YSharp-YSharp/releases/download/${tag}/ys-${tag}-${platform}.tar.gz"

    echo "==> Downloading Y# ${tag} for ${platform}..."
    echo "    URL: $url"

    local tmpdir
    tmpdir="$(mktemp -d)"
    cd "$tmpdir"

    if curl -fsSL "$url" -o ys.tar.gz; then
        tar xzf ys.tar.gz
        sudo mkdir -p "$INSTALL_DIR"
        sudo cp oys "$INSTALL_DIR/oys"
        sudo cp yo "$INSTALL_DIR/yo"
        rm -rf "$tmpdir"
        echo "==> Installed: $INSTALL_DIR/oys"
        echo "==> Installed: $INSTALL_DIR/yo"
    else
        echo "==> Binary download failed. Building from source..."
        cd "$PROJECT_DIR"
        install_from_source
    fi
}

main() {
    echo "  Y# (YSharp) v8.0.1 — Oyster Shell"
    echo "  ================================="
    echo ""

    # Try binary release first, fall back to source
    if command -v curl &>/dev/null; then
        install_from_release
    else
        install_from_source
    fi

    echo ""
    echo "==> Y# installed successfully!"
    echo "    Run: oys build myprogram.ys"
}

main
