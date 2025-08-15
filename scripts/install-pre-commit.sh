#!/bin/bash
set -euo pipefail

# Install pre-commit hooks for rust-sitter

if ! command -v pre-commit &> /dev/null; then
    echo "Installing pre-commit..."
    pip install --user pre-commit || pip3 install --user pre-commit
fi

echo "Installing pre-commit hooks..."
pre-commit install

echo "Running pre-commit on all files (first time setup)..."
pre-commit run --all-files || true

echo ""
echo "✅ Pre-commit hooks installed successfully!"
echo ""
echo "The following checks will run automatically before each commit:"
echo "  • cargo fmt (code formatting)"
echo "  • cargo clippy (linting)"
echo "  • Test connectivity checks"
echo "  • Contract tripwires (EOF & error stats)"
echo "  • Table sanity invariant tests"
echo ""
echo "To run manually: pre-commit run --all-files"
echo "To skip temporarily: git commit --no-verify"