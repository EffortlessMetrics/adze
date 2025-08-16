#!/usr/bin/env bash
set -euo pipefail

crate="${1:-ts-bridge}"
echo "Building $crate in release mode..."
export CARGO_TARGET_DIR=target
cargo build -p "$crate" --release 2>&1 | tail -5

# Detect OS and set appropriate extension and symbol lister
case "$(uname -s)" in
  Darwin) 
    ext="dylib"
    list="nm -gU --defined-only"
    undef="nm -u"
    ;;
  Linux)  
    ext="so"
    list="nm -g --defined-only"
    undef="nm -u"
    ;;
  MINGW*|MSYS*|CYGWIN*) 
    ext="dll"
    list="objdump -x"
    undef="objdump -x"
    ;;
  *) 
    ext="so"
    list="nm -g --defined-only"
    undef="nm -u"
    ;;
esac

# Find the library file (convert - to _ for library name)
lib_name=$(echo "$crate" | tr '-' '_')
so=$(find target/release -maxdepth 1 -name "lib${lib_name}.${ext}" -o -name "${lib_name}.${ext}" 2>/dev/null | head -n1)

if [ -z "$so" ]; then
    echo "ERROR: Could not find library file for $crate"
    exit 1
fi

echo "→ Checking symbols in $so"
if [[ "$ext" == "dll" ]]; then
    # Windows: use objdump to list exports
    $list "$so" 2>/dev/null | grep -E 'rs_ts_bridge_version|ts_' | head -20 || echo "No matching symbols found"
else
    # Unix: use nm
    $list "$so" 2>/dev/null | grep -E 'rs_ts_bridge_version|ts_' | head -20 || echo "No matching symbols found"
fi

# Check for undefined symbols
echo ""
echo "→ Checking for undefined symbols..."
if [[ "$ext" == "dll" ]]; then
    # Windows: look for imports in objdump output
    $undef "$so" 2>/dev/null | grep -E 'DLL Name:.*ts_|ts_' | head -10 || echo "No undefined ts_ symbols"
else
    # Unix: use nm -u
    $undef "$so" 2>/dev/null | grep -E '^[[:space:]]+U ts_' | head -10 || echo "No undefined ts_ symbols"
fi

echo ""
echo "→ Library info:"
file "$so"

echo ""
echo "OK: Build completed"