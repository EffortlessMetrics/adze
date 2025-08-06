#!/bin/bash

# Build script for WASM demo
set -e

echo "Building rust-sitter WASM demo..."

# Install wasm-pack if not already installed
if ! command -v wasm-pack &> /dev/null; then
    echo "Installing wasm-pack..."
    curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
fi

# Build the WASM module
echo "Building WASM module..."
wasm-pack build --target web --out-dir pkg

echo "Build complete! To run the demo:"
echo "1. Start a local web server (e.g., python3 -m http.server 8000)"
echo "2. Open http://localhost:8000 in your browser"