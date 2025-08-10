#!/bin/bash
# Fast test counting script

echo "=== Test Statistics ==="

# Count test functions (fast approximation using grep)
echo -n "Total test functions: "
find crates -name "*.rs" -type f | xargs grep -h "^\s*#\[test\]" 2>/dev/null | wc -l

echo -n "Tokio async tests: "
find crates -name "*.rs" -type f | xargs grep -h "^\s*#\[tokio::test\]" 2>/dev/null | wc -l

echo -n "Property tests: "
find crates -name "*.rs" -type f | xargs grep -h "proptest!" 2>/dev/null | wc -l

echo -n "Integration test files: "
find crates -path "*/tests/*.rs" -type f 2>/dev/null | wc -l

echo -n "Benchmark files: "
find . -path "*/benches/*.rs" -type f 2>/dev/null | wc -l

echo ""
echo "=== Tests by Major Crate ==="
for crate in mergecode-core mergecode-cli mergecode-repl xtask; do
    count=$(find crates/$crate -name "*.rs" -type f 2>/dev/null | xargs grep -h "^\s*#\[test\]" 2>/dev/null | wc -l)
    echo "$crate: $count tests"
done