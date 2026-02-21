#!/bin/bash
# Script to run benchmarks locally and check for regressions

set -e

echo "=== Running adze benchmarks ==="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Create results directory
mkdir -p bench-results

# Function to run a specific benchmark
run_benchmark() {
    local bench_name=$1
    echo -e "${YELLOW}Running $bench_name benchmarks...${NC}"
    
    if cargo bench --bench "$bench_name" -- --output-format bencher > "bench-results/${bench_name}.txt" 2>&1; then
        echo -e "${GREEN}✓ $bench_name completed${NC}"
    else
        echo -e "${RED}✗ $bench_name failed${NC}"
        cat "bench-results/${bench_name}.txt"
        return 1
    fi
}

# Run all benchmarks
benchmarks=(
    "incremental_parsing"
    "glr_parser_bench"
    "pure_rust_bench"
    "parser_benchmark"
)

failed=0
for bench in "${benchmarks[@]}"; do
    if ! run_benchmark "$bench"; then
        failed=$((failed + 1))
    fi
done

# Run workspace benchmarks
echo -e "${YELLOW}Running all workspace benchmarks...${NC}"
if cargo bench --workspace -- --output-format bencher > bench-results/all_benchmarks.txt 2>&1; then
    echo -e "${GREEN}✓ All workspace benchmarks completed${NC}"
else
    echo -e "${RED}✗ Workspace benchmarks failed${NC}"
    failed=$((failed + 1))
fi

# Summary
echo
echo "=== Benchmark Summary ==="
if [ $failed -eq 0 ]; then
    echo -e "${GREEN}All benchmarks passed!${NC}"
    echo
    echo "Results saved to bench-results/"
    echo "You can view detailed HTML reports in target/criterion/"
else
    echo -e "${RED}$failed benchmark(s) failed${NC}"
    exit 1
fi

# Optional: Compare with previous results if they exist
if [ -f "bench-results/baseline.json" ]; then
    echo
    echo "=== Comparing with baseline ==="
    # This would use the Python script from the CI workflow
    # python3 scripts/compare_benchmarks.py bench-results/baseline.json bench-results/current.json
fi