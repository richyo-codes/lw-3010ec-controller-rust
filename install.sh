#!/usr/bin/env bash
set -euo pipefail

# Install the CLI from this checkout using Cargo's standard install location.
# Usage: ./install.sh [--force]
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

if ! command -v cargo >/dev/null 2>&1; then
    echo "Rust and Cargo are required: https://rustup.rs" >&2
    exit 1
fi

case "${1:-}" in
    "") FORCE=() ;;
    --force) FORCE=(--force) ;;
    *)
        echo "Usage: $0 [--force]" >&2
        exit 2
        ;;
esac

cargo install --path "$ROOT_DIR" --locked "${FORCE[@]}"
echo "Installed lw3010ec-controller. Ensure Cargo's bin directory is on PATH."
