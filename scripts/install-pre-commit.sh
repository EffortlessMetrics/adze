#!/bin/bash
set -euo pipefail

# Install pre-commit hooks for rust-sitter

echo "rust-sitter Pre-commit Hook Installer"
echo "====================================="
echo ""

# Check for the new robust git hooks system first
if [ -f ".githooks/install.sh" ]; then
    echo "🦀 Robust Git Hooks (Recommended)"
    echo ""
    echo "Found the new robust git hooks system with:"
    echo "  ✓ Partial staging detection"
    echo "  ✓ Targeted formatting (staged files only)"
    echo "  ✓ Full diagnostic output"
    echo "  ✓ Version-controlled hooks"
    echo ""
    echo "Install robust hooks? [Y/n]"
    read -r response
    if [[ -z "$response" || "$response" =~ ^[Yy]$ ]]; then
        .githooks/install.sh
        echo ""
        echo "✅ Robust git hooks installed!"
        echo ""
        echo "You can still install python pre-commit as well if desired."
        echo "Continue with python pre-commit installation? [y/N]"
        read -r python_response
        if [[ ! "$python_response" =~ ^[Yy]$ ]]; then
            echo "Skipping python pre-commit installation."
            echo ""
            echo "To install python pre-commit later:"
            echo "  $0 --python-only"
            exit 0
        fi
    fi
    echo ""
fi

# Python pre-commit installation
echo "📦 Python Pre-commit Framework"
echo ""

if ! command -v pre-commit &> /dev/null; then
    echo "Installing pre-commit..."
    pip install --user pre-commit || pip3 install --user pre-commit
fi

echo "Installing pre-commit hooks..."
pre-commit install

echo "Running pre-commit on all files (first time setup)..."
pre-commit run --all-files || true

echo ""
echo "✅ Python pre-commit hooks installed successfully!"
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
echo ""
echo "Note: If you also installed robust git hooks, they will run in addition to these."