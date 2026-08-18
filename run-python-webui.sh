#!/usr/bin/env bash
set -euo pipefail

# ── LW-3010EC WebSerial UI — Python static server ──────────────────
# Serves the WebSerial UI with correct .wasm MIME types.
#
# Usage:
#   ./run-python-webui.sh              # default port (5000)
#   ./run-python-webui.sh 8080         # custom port
#
# The browser talks directly to the PSU via WebSerial; no backend API is used.
# ───────────────────────────────────────────────────────────────────

PORT="${1:-5000}"

# Ensure WASM files exist
if [ ! -f "web/pkg/lw3010ec_core.js" ]; then
    echo "⏳  Building WASM module..."
    cd lw3010ec-core
    rm -rf target/ pkg/
    wasm-pack build --target web --out-name lw3010ec_core
    cp -r pkg/* ../web/pkg/
    cd ..
fi

echo "─────────────────────────────────────────────"
echo "  🌐 Web UI (static) → http://127.0.0.1:$PORT"
echo "  📁 Serving from: $(pwd)/web/"
echo "─────────────────────────────────────────────"

cd web
python3 server.py "$PORT"
