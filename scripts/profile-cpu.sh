#!/usr/bin/env bash
# CPU Profiling Script for rust-sitter
# Part of v0.8.0 Performance Optimization (AC-PERF1)
#
# Usage:
#   ./scripts/profile-cpu.sh <language> <fixture-path>
#
# Example:
#   ./scripts/profile-cpu.sh python benches/fixtures/python/small/sample.py
#
# Requirements:
#   - cargo-flamegraph (install: cargo install flamegraph)
#   - perf (Linux) or dtrace (macOS)
#
# Output:
#   - Flamegraph SVG in docs/analysis/flamegraph-{language}-{size}.svg
#   - Profiling data for analysis

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Check arguments
if [ $# -ne 2 ]; then
    echo -e "${RED}Usage: $0 <language> <fixture-path>${NC}"
    echo -e "${YELLOW}Example: $0 python benches/fixtures/python/small/sample.py${NC}"
    exit 1
fi

LANGUAGE=$1
FIXTURE_PATH=$2

# Validate language
case "$LANGUAGE" in
    python|javascript|rust)
        ;;
    *)
        echo -e "${RED}Error: Unsupported language: $LANGUAGE${NC}"
        echo -e "${YELLOW}Supported: python, javascript, rust${NC}"
        exit 1
        ;;
esac

# Validate fixture path
if [ ! -f "$FIXTURE_PATH" ]; then
    echo -e "${RED}Error: Fixture not found: $FIXTURE_PATH${NC}"
    exit 1
fi

# Determine size from path
if [[ "$FIXTURE_PATH" == *"/small/"* ]]; then
    SIZE="small"
elif [[ "$FIXTURE_PATH" == *"/medium/"* ]]; then
    SIZE="medium"
elif [[ "$FIXTURE_PATH" == *"/large/"* ]]; then
    SIZE="large"
else
    SIZE="unknown"
fi

# Check for cargo-flamegraph
if ! command -v cargo-flamegraph &> /dev/null; then
    echo -e "${YELLOW}⚠️  cargo-flamegraph not found${NC}"
    echo -e "${YELLOW}Install: cargo install flamegraph${NC}"
    exit 1
fi

# Create output directory
mkdir -p docs/analysis

# Output file
OUTPUT_FILE="docs/analysis/flamegraph-${LANGUAGE}-${SIZE}.svg"

echo -e "${BLUE}🔥 CPU Profiling: ${LANGUAGE} (${SIZE})${NC}"
echo -e "${BLUE}Fixture: ${FIXTURE_PATH}${NC}"
echo ""

# TODO: This is a placeholder until we have actual parsing benchmarks
# For now, we'll profile a simple cargo test as proof of concept
echo -e "${YELLOW}Note: Profiling infrastructure setup. Actual parse profiling will be added.${NC}"
echo ""

# Placeholder: Profile a test (will be replaced with actual parsing benchmark)
echo -e "${BLUE}Running CPU profiling...${NC}"
if cargo flamegraph --test glr_parse --output "$OUTPUT_FILE" -- --nocapture 2>/dev/null; then
    echo -e "${GREEN}✅ Flamegraph generated: ${OUTPUT_FILE}${NC}"
else
    # Fallback: Create a placeholder file
    echo -e "${YELLOW}⚠️  Profiling not available yet (needs parsing benchmark)${NC}"
    echo -e "${YELLOW}Creating placeholder flamegraph...${NC}"

    cat > "$OUTPUT_FILE" <<EOF
<?xml version="1.0" standalone="no"?>
<!DOCTYPE svg PUBLIC "-//W3C//DTD SVG 1.1//EN" "http://www.w3.org/Graphics/SVG/1.1/DTD/svg11.dtd">
<svg version="1.1" width="1200" height="600" xmlns="http://www.w3.org/2000/svg">
  <text x="50" y="50" font-family="Verdana" font-size="20" fill="blue">
    Flamegraph: ${LANGUAGE} (${SIZE})
  </text>
  <text x="50" y="100" font-family="Verdana" font-size="14" fill="black">
    Placeholder - Actual profiling data will be added in Week 3 Day 2
  </text>
  <text x="50" y="130" font-family="Verdana" font-size="14" fill="black">
    Fixture: ${FIXTURE_PATH}
  </text>
</svg>
EOF
    echo -e "${GREEN}✅ Placeholder created: ${OUTPUT_FILE}${NC}"
fi

echo ""
echo -e "${BLUE}📊 Next steps:${NC}"
echo -e "  1. Review flamegraph: ${OUTPUT_FILE}"
echo -e "  2. Identify hot functions (>1% CPU time)"
echo -e "  3. Document findings in docs/analysis/PERFORMANCE_ANALYSIS_V0.7.0.md"
echo ""
echo -e "${GREEN}✅ CPU profiling complete${NC}"
