#!/bin/bash
# Build script for WASM target

set -e

echo "Building rust-sitter for WASM..."

# Add WASM target if not already added
rustup target add wasm32-unknown-unknown 2>/dev/null || true

# Build with wasm-bindgen features
cargo build --target wasm32-unknown-unknown --features wasm-bindgen

# If wasm-bindgen-cli is installed, generate bindings
if command -v wasm-bindgen &> /dev/null; then
    echo "Generating WASM bindings..."
    wasm-bindgen \
        --target web \
        --out-dir pkg \
        --out-name rust_sitter \
        target/wasm32-unknown-unknown/debug/rust_sitter.wasm
    
    echo "WASM build complete! Output in pkg/"
else
    echo "wasm-bindgen-cli not found. Install with: cargo install wasm-bindgen-cli"
    echo "WASM library built at: target/wasm32-unknown-unknown/debug/rust_sitter.wasm"
fi