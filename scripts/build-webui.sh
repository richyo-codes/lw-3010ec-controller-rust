#!/usr/bin/env bash
set -euo pipefail

# Build the browser-facing protocol module into the static WebSerial site.
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CORE_DIR="$ROOT_DIR/lw3010ec-core"
OUT_DIR="$ROOT_DIR/web/pkg"

if ! command -v wasm-pack >/dev/null 2>&1; then
    echo "wasm-pack is required. Install it with: cargo install wasm-pack" >&2
    exit 1
fi

wasm-pack build "$CORE_DIR" \
    --target web \
    --out-name lw3010ec_core \
    --out-dir "$OUT_DIR"
