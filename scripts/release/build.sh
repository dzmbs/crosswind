#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/../.."

TARGET="${1:-}"

if [ -n "$TARGET" ]; then
    echo "Building crosswind (release) for target: $TARGET"
    cargo build --release --target "$TARGET"
    BINARY="target/$TARGET/release/crosswind"
else
    echo "Building crosswind (release) for host"
    cargo build --release
    BINARY="target/release/crosswind"
fi

# Windows binary extension
case "${TARGET}" in
    *windows*) BINARY="${BINARY}.exe" ;;
esac

if [ ! -f "$BINARY" ]; then
    echo "ERROR: expected binary not found at $BINARY" >&2
    exit 1
fi

echo "Binary: $BINARY"
