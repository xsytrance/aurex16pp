#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

DOC_ARCH="docs/architecture.md"
DOC_CANON="docs/ai_handoff_canon.md"

shell_docs_sync_check() {
  echo "[preflight] docs-sync check (shell fallback)"

  local -a arch_markers=(
    "--generate-runtime-baseline"
    "--docs-sync-check"
    "Launch stage: pending"
    "Launch stage: validating"
    "Launch stage: ready"
  )

  for marker in "${arch_markers[@]}"; do
    if ! rg -n --fixed-strings -- "$marker" "$DOC_ARCH" >/dev/null; then
      echo "[preflight] docs-sync failed: missing '$marker' in $DOC_ARCH" >&2
      exit 2
    fi
  done

  if ! rg -n --fixed-strings -- "Audio diagnostics baseline artifact generation is required" "$DOC_CANON" >/dev/null; then
    echo "[preflight] docs-sync failed: missing baseline discipline line in $DOC_CANON" >&2
    exit 2
  fi

  echo "[preflight] docs-sync check passed (shell fallback)"
}

echo "[preflight] formatting check"
cargo fmt -- --check

echo "[preflight] compile check"
cargo check

if [[ "${AUREX_SKIP_AUDIT_LINK:-0}" == "1" ]]; then
  shell_docs_sync_check
  echo "[preflight] skipping cartridge audit run (AUREX_SKIP_AUDIT_LINK=1)"
  exit 0
fi

echo "[preflight] docs-sync check"
cargo run -- --docs-sync-check

echo "[preflight] cartridge audit (json)"
cargo run -- --audit-cartridges --json
