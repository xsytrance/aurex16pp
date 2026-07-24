#!/bin/bash
# Aurex-16++ Agent Console — Dashboard Startup Script
# Usage: ./scripts/start-dashboard.sh [port] [recordings-dir]

set -e

PORT="${1:-8080}"
RECORDINGS_DIR="${2:-./recordings}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

cd "$PROJECT_DIR"

[ -f "$HOME/.cargo/env" ] && . "$HOME/.cargo/env"

# Ensure recordings directory exists
mkdir -p "$RECORDINGS_DIR"

# Sanity checks
if [ ! -f "webapp/dist/index.html" ]; then
    echo "ERROR: webapp/dist/index.html missing — the dashboard frontend is not present."
    exit 1
fi
if ! command -v ffmpeg >/dev/null; then
    echo "ERROR: ffmpeg not found — required for session recording."
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

# Server build needs no SDL2 — headless + web only
echo "Building server (if needed)..."
cargo build --no-default-features --features server 2>&1 | tail -3

echo ""
echo "Starting server..."
echo ""

exec cargo run --no-default-features --features server -- --server --port "$PORT" --recordings-dir "$RECORDINGS_DIR"
