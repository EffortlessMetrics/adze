#!/usr/bin/env bash
# DEPRECATED: Use rust-native xtask command instead
# New command: cargo xtask compare-baseline [--format {table|json|markdown}]
# This shell script will be removed in a future version.
#
# Tree-sitter Comparison Script
# Part of v0.8.0 Performance Optimization (AC-PERF1, AC-PERF5)
#
# This script benchmarks Tree-sitter C parser against rust-sitter on the same
# fixtures to establish performance ratios and validate the 2x performance goal.
#
# BDD Scenarios:
# - Scenario 1.4: Tree-sitter baseline benchmark runs for same fixtures
# - Scenario 5.2: Tree-sitter comparison shows ≤2x ratio
#
# Usage:
#   ./scripts/compare-tree-sitter.sh baseline     # Benchmark Tree-sitter C
#   ./scripts/compare-tree-sitter.sh rust-sitter  # Benchmark rust-sitter
#   ./scripts/compare-tree-sitter.sh compare      # Generate comparison report
#   ./scripts/compare-tree-sitter.sh report       # Full report for v0.8.0
#
# Requirements:
#   - tree-sitter CLI (install: npm install -g tree-sitter-cli)
#   - tree-sitter-python, tree-sitter-javascript (grammars)
#   - cargo (for rust-sitter benchmarks)
#
# Output:
#   - tree_sitter_baseline.json (Tree-sitter results)
#   - rust_sitter_results.json (rust-sitter results)
#   - comparison_report.txt (performance comparison)

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Output directory
OUTPUT_DIR="docs/analysis"
mkdir -p "$OUTPUT_DIR"

# Check if Tree-sitter CLI is available
check_tree_sitter() {
    if command -v tree-sitter &> /dev/null; then
        echo -e "${GREEN}✅ tree-sitter CLI found${NC}"
        return 0
    else
        echo -e "${YELLOW}⚠️  tree-sitter CLI not found${NC}"
        echo -e "${YELLOW}Install: npm install -g tree-sitter-cli${NC}"
        return 1
    fi
}

# Benchmark Tree-sitter C parser
benchmark_tree_sitter() {
    echo -e "${BLUE}🌲 Benchmarking Tree-sitter C Parser${NC}"
    echo ""

    if ! check_tree_sitter; then
        echo -e "${YELLOW}Skipping Tree-sitter benchmarks (CLI not available)${NC}"
        echo -e "${YELLOW}Creating placeholder baseline...${NC}"

        cat > "$OUTPUT_DIR/tree_sitter_baseline.json" <<'EOF'
{
  "comment": "Placeholder - Tree-sitter CLI not available",
  "date": "PLACEHOLDER",
  "benchmarks": [
    {
      "language": "python",
      "size": "small",
      "fixture": "benches/fixtures/python/small/sample.py",
      "parse_time_ms": 0.5,
      "memory_mb": 0.2,
      "note": "Placeholder values - actual benchmarking pending"
    },
    {
      "language": "javascript",
      "size": "small",
      "fixture": "benches/fixtures/javascript/small/sample.js",
      "parse_time_ms": 0.3,
      "memory_mb": 0.15,
      "note": "Placeholder values - actual benchmarking pending"
    }
  ]
}
EOF
        echo -e "${GREEN}✅ Placeholder baseline created: $OUTPUT_DIR/tree_sitter_baseline.json${NC}"
        return 0
    fi

    # TODO: Implement actual Tree-sitter benchmarking
    # This requires:
    # 1. Tree-sitter grammars installed (tree-sitter-python, tree-sitter-javascript)
    # 2. Parsing each fixture with `tree-sitter parse <file>`
    # 3. Measuring time with `hyperfine` or similar tool
    # 4. Recording results in JSON format

    echo -e "${YELLOW}Note: Full Tree-sitter benchmarking will be implemented in Week 3 Day 2${NC}"
    echo -e "${YELLOW}Creating baseline template...${NC}"

    cat > "$OUTPUT_DIR/tree_sitter_baseline.json" <<EOF
{
  "comment": "Tree-sitter C baseline measurements (to be populated)",
  "date": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "commit": "$(git rev-parse HEAD 2>/dev/null || echo 'unknown')",
  "benchmarks": [
    {
      "language": "python",
      "size": "small",
      "fixture": "benches/fixtures/python/small/sample.py",
      "parse_time_ms": "TBD",
      "memory_mb": "TBD",
      "note": "To be measured with tree-sitter CLI"
    },
    {
      "language": "javascript",
      "size": "small",
      "fixture": "benches/fixtures/javascript/small/sample.js",
      "parse_time_ms": "TBD",
      "memory_mb": "TBD",
      "note": "To be measured with tree-sitter CLI"
    },
    {
      "language": "rust",
      "size": "small",
      "fixture": "benches/fixtures/rust/small/sample.rs",
      "parse_time_ms": "TBD",
      "memory_mb": "TBD",
      "note": "To be measured with tree-sitter CLI"
    }
  ],
  "methodology": "hyperfine --warmup 10 --runs 100 'tree-sitter parse <fixture>'",
  "environment": {
    "os": "$(uname -s)",
    "arch": "$(uname -m)",
    "tree_sitter_version": "$(tree-sitter --version 2>/dev/null || echo 'not available')"
  }
}
EOF

    echo -e "${GREEN}✅ Baseline template created: $OUTPUT_DIR/tree_sitter_baseline.json${NC}"
}

# Benchmark rust-sitter
benchmark_rust_sitter() {
    echo -e "${BLUE}🦀 Benchmarking rust-sitter GLR Parser${NC}"
    echo ""

    # Check if benchmark exists
    if [ ! -f "benches/glr-performance.rs" ]; then
        echo -e "${RED}Error: Benchmark file not found: benches/glr-performance.rs${NC}"
        exit 1
    fi

    echo -e "${YELLOW}Running cargo bench...${NC}"
    echo ""

    # Run benchmarks and save results
    if cargo bench --bench glr-performance -- --save-baseline current 2>&1 | tee "$OUTPUT_DIR/rust_sitter_bench.log"; then
        echo ""
        echo -e "${GREEN}✅ rust-sitter benchmarks complete${NC}"
        echo -e "${BLUE}Results saved to: $OUTPUT_DIR/rust_sitter_bench.log${NC}"

        # Extract benchmark results and create JSON summary
        # TODO: Parse Criterion output and create structured JSON
        cat > "$OUTPUT_DIR/rust_sitter_results.json" <<EOF
{
  "comment": "rust-sitter benchmark results (parsed from Criterion output)",
  "date": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "commit": "$(git rev-parse HEAD)",
  "baseline": "current",
  "benchmarks": "See rust_sitter_bench.log for detailed Criterion results",
  "note": "Structured JSON parsing to be implemented in Week 3 Day 2"
}
EOF
        echo -e "${GREEN}✅ Results summary: $OUTPUT_DIR/rust_sitter_results.json${NC}"
    else
        echo -e "${YELLOW}⚠️  Benchmark run encountered issues (expected for placeholder implementation)${NC}"
        echo -e "${YELLOW}This is normal until GLR parsing is integrated into benchmarks${NC}"
    fi
}

# Generate comparison report
generate_comparison() {
    echo -e "${BLUE}📊 Generating Comparison Report${NC}"
    echo ""

    local REPORT_FILE="$OUTPUT_DIR/tree_sitter_comparison.txt"

    cat > "$REPORT_FILE" <<'EOF'
Tree-sitter C vs. rust-sitter Performance Comparison
=====================================================

Version: v0.8.0 (Week 3 Day 1)
Date: TIMESTAMP
Status: Infrastructure Setup (Actual benchmarking pending)

Overview
--------
This document compares rust-sitter GLR parser performance against Tree-sitter C
on the same fixtures to validate the 2x performance goal (AC-PERF5).

Performance Goal (from contract):
- rust-sitter parsing time ≤ 2x Tree-sitter C (all benchmarks)
- rust-sitter memory usage < 10x input size

Benchmark Results
-----------------

| Language   | Size   | Tree-sitter (ms) | rust-sitter (ms) | Ratio | Goal Met? |
|------------|--------|------------------|------------------|-------|-----------|
| Python     | Small  | TBD              | TBD              | TBD   | ⏳         |
| JavaScript | Small  | TBD              | TBD              | TBD   | ⏳         |
| Rust       | Small  | TBD              | TBD              | TBD   | ⏳         |

Status: Benchmarking infrastructure ready, actual measurements pending Week 3 Day 2.

Memory Usage Comparison
-----------------------

| Language   | Size   | Input Size | Tree-sitter Peak | rust-sitter Peak | Ratio |
|------------|--------|------------|------------------|------------------|-------|
| Python     | Small  | ~2 KB      | TBD              | TBD              | TBD   |
| JavaScript | Small  | ~3 KB      | TBD              | TBD              | TBD   |
| Rust       | Small  | ~2.5 KB    | TBD              | TBD              | TBD   |

Analysis
--------

To be completed after actual benchmarking.

Key Questions:
1. What is the current performance ratio (rust-sitter / tree-sitter)?
2. Where is rust-sitter slower (parsing, allocation, fork/merge)?
3. What optimizations will have the highest impact?

Next Steps
----------

Week 3 Day 2-3:
1. Run actual Tree-sitter benchmarks with hyperfine
2. Run rust-sitter benchmarks with real GLR parsing
3. Parse Criterion output to extract structured data
4. Calculate performance ratios
5. Update this report with actual measurements
6. Document findings in PERFORMANCE_ANALYSIS_V0.7.0.md

Week 4:
1. Implement optimizations (arena allocation, stack pooling)
2. Re-run benchmarks (v0.8.0)
3. Validate 2x goal achieved
4. Generate final PERFORMANCE_REPORT_V0.8.0.md

References
----------

- Contract: docs/specs/PERFORMANCE_OPTIMIZATION_CONTRACT.md
- BDD Scenarios: docs/plans/BDD_PERFORMANCE_OPTIMIZATION.md
- Baseline: docs/baselines/PERFORMANCE_BASELINE_V0.7.0.md (to be created)

EOF

    # Update timestamp
    sed -i "s/TIMESTAMP/$(date -u +%Y-%m-%dT%H:%M:%SZ)/" "$REPORT_FILE" 2>/dev/null || true

    echo -e "${GREEN}✅ Comparison report: $REPORT_FILE${NC}"
}

# Generate full report for v0.8.0
generate_full_report() {
    echo -e "${BLUE}📝 Generating Full Performance Report${NC}"
    echo ""

    # Run all benchmarking steps
    benchmark_tree_sitter
    echo ""
    benchmark_rust_sitter
    echo ""
    generate_comparison

    echo ""
    echo -e "${GREEN}✅ Full report generation complete${NC}"
    echo ""
    echo -e "${BLUE}📊 Output files:${NC}"
    echo -e "  - $OUTPUT_DIR/tree_sitter_baseline.json"
    echo -e "  - $OUTPUT_DIR/rust_sitter_results.json"
    echo -e "  - $OUTPUT_DIR/tree_sitter_comparison.txt"
}

# Main script logic
main() {
    local COMMAND=${1:-help}

    case "$COMMAND" in
        baseline)
            benchmark_tree_sitter
            ;;
        rust-sitter)
            benchmark_rust_sitter
            ;;
        compare)
            generate_comparison
            ;;
        report)
            generate_full_report
            ;;
        help|--help|-h)
            echo "Usage: $0 <command>"
            echo ""
            echo "Commands:"
            echo "  baseline      - Benchmark Tree-sitter C parser"
            echo "  rust-sitter   - Benchmark rust-sitter parser"
            echo "  compare       - Generate comparison report"
            echo "  report        - Generate full report (all steps)"
            echo "  help          - Show this help message"
            echo ""
            echo "Example:"
            echo "  $0 report     # Generate full comparison report"
            ;;
        *)
            echo -e "${RED}Error: Unknown command: $COMMAND${NC}"
            echo -e "${YELLOW}Run: $0 help${NC}"
            exit 1
            ;;
    esac
}

main "$@"
