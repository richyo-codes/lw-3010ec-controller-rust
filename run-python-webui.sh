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

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PORT="${1:-5000}"

if ! [[ "$PORT" =~ ^[0-9]+$ ]] || (( PORT < 1 || PORT > 65535 )); then
    echo "Port must be an integer between 1 and 65535." >&2
    exit 2
fi

if ! command -v python3 >/dev/null 2>&1; then
    echo "python3 is required to serve the WebSerial UI." >&2
    exit 1
fi

# Ensure WASM files exist
if [ ! -f "$ROOT_DIR/web/pkg/lw3010ec_core.js" ]; then
    echo "⏳  Building WASM module..."
    "$ROOT_DIR/scripts/build-webui.sh"
fi

echo "─────────────────────────────────────────────"
echo "  🌐 Web UI (static) → http://127.0.0.1:$PORT"
echo "  📁 Serving from: $ROOT_DIR/web/"
echo "─────────────────────────────────────────────"

cd "$ROOT_DIR/web"
python3 server.py "$PORT"
