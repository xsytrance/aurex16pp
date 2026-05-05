#!/bin/bash
# Aurex-16++ Agent Console — Dashboard Startup Script
# Usage: ./scripts/start-dashboard.sh [port] [recordings-dir]

set -e

PORT="${1:-8080}"
RECORDINGS_DIR="${2:-./recordings}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

cd "$PROJECT_DIR"

# Environment setup
. "$HOME/.cargo/env"
export CARGO_TARGET_DIR=/tmp/aurex-target
export RUSTFLAGS="-C linker=gcc -L /tmp/sdl2-link"

# Ensure SDL2 symlink exists for full builds
mkdir -p /tmp/sdl2-link
ln -sf /usr/lib/x86_64-linux-gnu/libSDL2-2.0.so.0 /tmp/sdl2-link/libSDL2.so 2>/dev/null || true

# Ensure recordings directory exists
mkdir -p "$RECORDINGS_DIR"

# Ensure webapp dist exists
if [ ! -f "webapp/dist/index.html" ]; then
    echo "ERROR: Frontend not built. Build the webapp first."
    exit 1
fi

echo "========================================"
echo "  Aurex-16++ Agent Console"
echo "========================================"
echo "  Port:         $PORT"
echo "  Recordings:   $RECORDINGS_DIR"
echo "  Dashboard:    http://localhost:$PORT"
echo "  API:          http://localhost:$PORT/api"
echo "========================================"
echo ""

# Build if needed
echo "Building server (if needed)..."
cargo build --features server 2>&1 | tail -5

echo ""
echo "Starting server..."
echo ""

# Run server
cargo run --features server -- --server --port "$PORT" --recordings-dir "$RECORDINGS_DIR"
