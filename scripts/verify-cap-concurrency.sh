#!/usr/bin/env bash
set -euo pipefail

echo "=== Cap Concurrency Implementation Verification ==="
echo

echo "1. Scripts created:"
ls -la scripts/preflight.sh scripts/test-capped.sh scripts/verify-cap-concurrency.sh
echo

echo "2. Cargo aliases configured:"
echo "   t2 = test with 2 threads"
echo "   t1 = test with 1 thread  "  
echo "   test-safe = test with safe defaults"
echo "   test-ultra-safe = test with 1 thread"
echo

echo "3. Preflight script test:"
source scripts/preflight.sh
echo

echo "4. Container configuration:"
if [ -f docker-compose.test.yml ]; then
    echo "   ✅ docker-compose.test.yml created"
    echo "   Usage: docker-compose -f docker-compose.test.yml up rust-tests"
else
    echo "   ❌ docker-compose.test.yml missing"
fi
echo

if [ -f .docker/rust.dockerfile ]; then
    echo "   ✅ .docker/rust.dockerfile created"
else
    echo "   ❌ .docker/rust.dockerfile missing"
fi
echo

echo "5. Runtime module:"
if grep -q "concurrency_caps" runtime/src/lib.rs; then
    echo "   ✅ concurrency_caps module added to runtime"
else
    echo "   ❌ concurrency_caps module not found in runtime"
fi
echo

echo "6. CI environment variables:"
if grep -q "RUST_TEST_THREADS: 2" .github/workflows/ci.yml; then
    echo "   ✅ CI configured with concurrency caps"
    echo "   Environment variables:"
    grep -A 8 "Cap Concurrency" .github/workflows/ci.yml | grep ":" | head -8
else
    echo "   ❌ CI concurrency caps not configured"
fi
echo

echo "7. Documentation:"
if grep -q "Cap Concurrency Implementation" CLAUDE.md; then
    echo "   ✅ Implementation documented in CLAUDE.md"
else
    echo "   ❌ Documentation not found in CLAUDE.md"
fi
echo

echo "=== Verification Complete ==="
echo "Usage examples:"
echo "  cargo t2                    # Test with 2 threads"
echo "  scripts/preflight.sh        # Check system pressure"
echo "  scripts/test-capped.sh      # Run capped tests"
echo "  docker-compose -f docker-compose.test.yml up rust-tests"