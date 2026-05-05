#!/bin/bash
# Aurex-16++ Agent Console — Environment Setup Script
# Run once before first use

set -e

echo "Setting up Aurex-16++ Agent Console environment..."

# Install Rust if missing
if ! command -v cargo &> /dev/null; then
    echo "Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    . "$HOME/.cargo/env"
fi

# Ensure SDL2 runtime library is available
if [ ! -f /usr/lib/x86_64-linux-gnu/libSDL2-2.0.so.0 ]; then
    echo "WARNING: SDL2 runtime library not found. Interactive mode may not work."
fi

# Create SDL2 symlink for linker
mkdir -p /tmp/sdl2-link
ln -sf /usr/lib/x86_64-linux-gnu/libSDL2-2.0.so.0 /tmp/sdl2-link/libSDL2.so 2>/dev/null || true

# Verify ffmpeg
if ! command -v ffmpeg &> /dev/null; then
    echo "ERROR: ffmpeg is required for recording but not found."
    exit 1
fi

echo ""
echo "Environment ready!"
echo ""
echo "Next steps:"
echo "  1. Start dashboard: ./scripts/start-dashboard.sh"
echo "  2. Or run headless:  cargo run --no-default-features --features agent -- --headless --frames 60"
echo "  3. Or run agent:     cargo run --no-default-features --features agent -- --agent --max-frames 120"
echo ""
