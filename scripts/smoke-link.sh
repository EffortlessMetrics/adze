#!/usr/bin/env bash
set -euo pipefail

crate="${1:-ts-bridge}"
echo "Building $crate in release mode..."
target_dir="${CARGO_TARGET_DIR:-target}"
export CARGO_TARGET_DIR="$target_dir"

if [ -f "tools/$crate/Cargo.toml" ]; then
  cargo build --manifest-path "tools/$crate/Cargo.toml" --release --locked
else
  cargo build -p "$crate" --release --locked
fi

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
    # Try multiple tools in order of preference
    if command -v dumpbin >/dev/null 2>&1; then
      list="dumpbin /exports"
      undef="dumpbin /imports"
    elif command -v llvm-objdump >/dev/null 2>&1; then
      list="llvm-objdump -p"
      undef="llvm-objdump -p"
    else
      list="objdump -x"
      undef="objdump -x"
    fi
    ;;
  *) 
    ext="so"
    list="nm -g --defined-only"
    undef="nm -u"
    ;;
esac

# Find the library file (convert - to _ for library name)
lib_name=$(echo "$crate" | tr '-' '_')
search_dirs=()
for dir in "$target_dir/release" "$target_dir/release/deps"; do
  if [ -d "$dir" ]; then
    search_dirs+=("$dir")
  fi
done

if [ ${#search_dirs[@]} -eq 0 ]; then
  echo "ERROR: No release output directories found under $target_dir"
  exit 1
fi

so=$(
  find "${search_dirs[@]}" -maxdepth 1 -type f \
    \( \
      -name "lib${lib_name}.${ext}" -o \
      -name "${lib_name}.${ext}" -o \
      -name "lib${lib_name}-*.${ext}" -o \
      -name "${lib_name}-*.${ext}" \
    \) \
    2>/dev/null | sort | head -n1
)

# Also check for import library on Windows
if [[ "$ext" == "dll" ]]; then
    imp=$(
      find "${search_dirs[@]}" -maxdepth 1 -type f \
        \( \
          -name "${lib_name}.dll.lib" -o \
          -name "lib${lib_name}.dll.a" -o \
          -name "${lib_name}-*.dll.lib" -o \
          -name "lib${lib_name}-*.dll.a" \
        \) \
        2>/dev/null | sort | head -n1 || true
    )
    [ -n "$imp" ] && echo "→ Found import lib: $imp"
fi

if [ -z "$so" ]; then
    echo "ERROR: Could not find library file for $crate"
    exit 1
fi

echo "→ Checking symbols in $so"
if [[ "$ext" == "dll" ]]; then
    # Windows: use the chosen tool to list exports
    if [[ "$list" == "dumpbin /exports" ]]; then
        $list "$so" 2>/dev/null | tr -d '\r' | grep -E 'rs_ts_bridge_version|ts_' | head -20 || echo "No matching symbols found"
    else
        $list "$so" 2>/dev/null | grep -E 'rs_ts_bridge_version|ts_' | head -20 || echo "No matching symbols found"
    fi
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
