#!/usr/bin/env bash
# Test Connectivity Verification Script
# Run this locally to check test harness health

set -euo pipefail

echo "🔍 Test Connectivity Check"
echo "=========================="
echo ""

# Check if cargo-nextest is installed
if ! command -v cargo-nextest &> /dev/null; then
    echo "⚠️  cargo-nextest not found. Installing..."
    cargo install cargo-nextest --locked
fi

# 1. Check for disabled test files
echo "1️⃣  Checking for disabled test files (.rs.disabled)..."
DISABLED_FILES=$(git ls-files '*tests/*.rs.disabled' 2>/dev/null || true)
if [ -n "$DISABLED_FILES" ]; then
    echo "❌ Found disabled test files:"
    echo "$DISABLED_FILES" | sed 's/^/   /'
    echo ""
else
    echo "✅ No disabled test files found"
    echo ""
fi

# 2. Count tests for each feature set
echo "2️⃣  Discovering tests by feature set..."
declare -A FEATURE_COUNTS
FEATURES=("" "--features external_scanners" "--features incremental_glr" "--all-features")
FEATURE_NAMES=("default" "external_scanners" "incremental_glr" "all-features")

for i in "${!FEATURES[@]}"; do
    FEATURE="${FEATURES[$i]}"
    NAME="${FEATURE_NAMES[$i]}"
    echo -n "   $NAME: "
    
    # Use nextest to list tests and count them
    COUNT=$(cargo nextest list --workspace $FEATURE 2>/dev/null | grep -E '^\s+(test|bench)' | wc -l | tr -d ' ')
    FEATURE_COUNTS["$NAME"]=$COUNT
    
    if [ "$COUNT" -eq 0 ]; then
        echo "❌ 0 tests (ERROR: no tests discovered!)"
    else
        echo "✅ $COUNT tests"
    fi
done
echo ""

# 3. Per-crate test counts
echo "3️⃣  Per-crate test discovery (default features)..."
echo "   ┌─────────────────────────────────────┬────────┐"
echo "   │ Crate                               │ Tests  │"
echo "   ├─────────────────────────────────────┼────────┤"

ZERO_TEST_CRATES=""
cargo metadata --no-deps --format-version=1 | jq -r '.packages[].name' | sort | while read CRATE; do
    COUNT=$(cargo nextest list -p "$CRATE" 2>/dev/null | grep -E '^\s+(test|bench)' | wc -l | tr -d ' ')
    printf "   │ %-35s │ %6d │\n" "$CRATE" "$COUNT"
    
    if [ "$COUNT" -eq 0 ]; then
        ZERO_TEST_CRATES="${ZERO_TEST_CRATES}${CRATE}\n"
    fi
done
echo "   └─────────────────────────────────────┴────────┘"
echo ""

# 4. Check for ignored tests
echo "4️⃣  Checking for #[ignore] tests..."
IGNORED_COUNT=$(rg -c '^\s*#\s*\[\s*ignore' --glob '!target' 2>/dev/null | wc -l | tr -d ' ')
if [ "$IGNORED_COUNT" -gt 0 ]; then
    echo "   ⚠️  Found ignored tests in $IGNORED_COUNT files:"
    rg -l '^\s*#\s*\[\s*ignore' --glob '!target' 2>/dev/null | head -10 | sed 's/^/      /'
    TOTAL_IGNORED=$(rg '^\s*#\s*\[\s*ignore' --glob '!target' 2>/dev/null | wc -l | tr -d ' ')
    echo "   Total: $TOTAL_IGNORED ignored test functions"
else
    echo "   ✅ No ignored tests found"
fi
echo ""

# 5. Check for test modules in src/ that might be disconnected
echo "5️⃣  Checking for potential orphaned test modules..."
ORPHANED=0
for f in $(find . -path ./target -prune -o -path './xtask/fixtures' -prune -o -name '*.rs' -path '*/tests/*' -print 2>/dev/null); do
    # Skip files that are clearly integration test entry points
    if basename "$f" | grep -qE '^(main|lib|mod)\.rs$'; then
        continue
    fi
    
    # Check if file has any test functions or modules
    if ! grep -qE '^\s*(#\[test\]|#\[cfg\(test\)\]|fn test_|mod tests)' "$f" 2>/dev/null; then
        if [ $ORPHANED -eq 0 ]; then
            echo "   ⚠️  Potential orphaned files (no test markers found):"
        fi
        echo "      $f"
        ORPHANED=$((ORPHANED + 1))
    fi
done

if [ $ORPHANED -eq 0 ]; then
    echo "   ✅ No orphaned test files detected"
fi
echo ""

# 6. Summary and recommendations
echo "📊 Summary"
echo "=========="
echo ""

TOTAL_DEFAULT="${FEATURE_COUNTS["default"]}"
TOTAL_ALL="${FEATURE_COUNTS["all-features"]}"

echo "• Total tests (default): $TOTAL_DEFAULT"
echo "• Total tests (all features): $TOTAL_ALL"
echo "• Ignored tests: ${TOTAL_IGNORED:-0}"
echo "• Disabled files: $(echo "$DISABLED_FILES" | grep -c '^' 2>/dev/null || echo 0)"
echo ""

# Provide actionable recommendations
if [ -n "$DISABLED_FILES" ] || [ "${TOTAL_DEFAULT}" -eq 0 ] || [ $ORPHANED -gt 0 ]; then
    echo "⚠️  Recommendations:"
    
    if [ -n "$DISABLED_FILES" ]; then
        echo "   • Re-enable disabled test files or remove them"
    fi
    
    if [ "${TOTAL_DEFAULT}" -eq 0 ]; then
        echo "   • CRITICAL: No tests discovered! Check Cargo.toml configurations"
    fi
    
    if [ $ORPHANED -gt 0 ]; then
        echo "   • Review potentially orphaned test files"
    fi
else
    echo "✅ Test infrastructure appears healthy!"
fi

echo ""
echo "💡 Tip: Run this script regularly to catch test disconnections early"
echo "💡 Tip: The CI will fail if any of these checks detect issues"