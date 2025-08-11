#!/bin/bash
# CI validation script for rust-sitter
# Run this before publishing or in CI to ensure code quality

set -e

echo "=== Rust-Sitter CI Validation ==="
echo

# Format check
echo "→ Checking code formatting..."
cargo fmt --all -- --check || {
    echo "❌ Format check failed. Run 'cargo fmt' to fix."
    exit 1
}
echo "✅ Format check passed"
echo

# Clippy lints
echo "→ Running clippy on tablegen (strict)..."
cargo clippy -p rust-sitter-tablegen -- -D warnings || {
    echo "❌ Clippy found issues. Fix the warnings above."
    exit 1
}
echo "✅ Clippy passed for tablegen (strict mode)"
echo

echo "→ Running clippy on ir (permissive)..."
cargo clippy -p rust-sitter-ir 2>&1 | tee /tmp/clippy-ir.log
if grep -q "error:" /tmp/clippy-ir.log; then
    echo "❌ Clippy found errors in ir. Fix these before release."
    exit 1
else
    echo "✅ Clippy check complete for ir (warnings allowed)"
fi
echo

echo "→ Running clippy on glr-core (permissive)..."
cargo clippy -p rust-sitter-glr-core 2>&1 | tee /tmp/clippy-glr.log
if grep -q "error:" /tmp/clippy-glr.log; then
    echo "❌ Clippy found errors in glr-core. Fix these before release."
    exit 1
else
    echo "✅ Clippy check complete for glr-core (warnings allowed)"
fi
echo

# Tests
echo "→ Running all tests..."
cargo test --workspace || {
    echo "❌ Tests failed. Fix the failing tests above."
    exit 1
}
echo "✅ All tests passed"
echo

# Documentation build
echo "→ Building documentation..."
cargo doc -p rust-sitter-tablegen --no-deps || {
    echo "❌ Documentation build failed."
    exit 1
}
cargo doc -p rust-sitter-ir --no-deps || {
    echo "❌ Documentation build failed."
    exit 1
}
cargo doc -p rust-sitter-glr-core --no-deps || {
    echo "❌ Documentation build failed."
    exit 1
}
echo "✅ Documentation builds successfully"
echo

# Package validation
echo "→ Validating package contents..."
echo "  Checking rust-sitter-ir..."
cargo package -p rust-sitter-ir --list > /dev/null || {
    echo "❌ Package validation failed for rust-sitter-ir"
    exit 1
}

echo "  Checking rust-sitter-glr-core..."
cargo package -p rust-sitter-glr-core --list > /dev/null || {
    echo "❌ Package validation failed for rust-sitter-glr-core"
    exit 1
}

echo "  Checking rust-sitter-tablegen..."
cargo package -p rust-sitter-tablegen --list > /dev/null || {
    echo "❌ Package validation failed for rust-sitter-tablegen"
    exit 1
}
echo "✅ Package contents validated"
echo

# Check for TODO/FIXME comments in release crates
echo "→ Checking for TODO/FIXME comments..."
TODO_COUNT=$(grep -r "TODO\|FIXME" ir/src glr-core/src tablegen/src 2>/dev/null | wc -l || echo 0)
if [ "$TODO_COUNT" -gt 0 ]; then
    echo "⚠️  Found $TODO_COUNT TODO/FIXME comments in release crates:"
    grep -r "TODO\|FIXME" ir/src glr-core/src tablegen/src 2>/dev/null | head -5 || true
    echo "   Consider addressing these before release."
else
    echo "✅ No TODO/FIXME comments found"
fi
echo

echo "=== ✅ All CI checks passed! ==="
echo "Ready for release or merge."