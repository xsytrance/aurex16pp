#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

echo "[preflight] formatting check"
cargo fmt -- --check

echo "[preflight] compile check"
cargo check

if [[ "${AUREX_SKIP_AUDIT_LINK:-0}" == "1" ]]; then
  echo "[preflight] skipping cartridge audit run (AUREX_SKIP_AUDIT_LINK=1)"
  exit 0
fi

echo "[preflight] docs-sync check"
cargo run -- --docs-sync-check

echo "[preflight] cartridge audit (json)"
cargo run -- --audit-cartridges --json
