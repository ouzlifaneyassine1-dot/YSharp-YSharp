#!/usr/bin/env bash
set -euo pipefail

# Y# Build Script — Linux/macOS
# Usage: ./scripts/build.sh [target]

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
TARGET="${1:-native}"

cd "$PROJECT_DIR/compiler"

echo "==> Building Y# compiler for $TARGET..."
cargo build --release

echo "==> Copying binaries..."
mkdir -p "$PROJECT_DIR/dist"
cp target/release/oys "$PROJECT_DIR/dist/oys"
cp target/release/yo "$PROJECT_DIR/dist/yo"

echo "==> Testing..."
"$PROJECT_DIR/dist/oys" build "$PROJECT_DIR/testprog/hello.ys"
"$PROJECT_DIR/dist/oys" build --easy "$PROJECT_DIR/testprog/hello_easy.yse"
"$PROJECT_DIR/dist/oys" test "$PROJECT_DIR/testprog"

echo "==> Build complete!"
echo "    Binary: dist/oys"
echo "    Run:     dist/oys build myprogram.ys"
