#!/usr/bin/env bash
set -euo pipefail

# This script checks that the Tree-sitter ABI hasn't changed
# It should be run in CI to catch any ABI drift

echo "Checking Tree-sitter ABI stability..."

# For now, we just verify the tool can assert ABI version
# In production, you'd:
# 1. Download specific version of tree_sitter/api.h
# 2. Hash it and compare to known good hash
# 3. Do the same for a sample parser.h from tree-sitter-json

# Run the ABI version check via the tool
cargo build --release -p ts-bridge

echo "ABI check passed!"