# ts-bridge: Tree-sitter to GLR Runtime Bridge

This tool extracts parse tables from compiled Tree-sitter grammars and converts them into a format suitable for our custom GLR runtime.

## Features

- **Production-ready**: Full extraction of Tree-sitter parse tables with ABI guards
- **Feature-gated builds**: Separate development (stub) and production modes
- **ABI stability**: Pinned to Tree-sitter v15 with header hash verification
- **Comprehensive testing**: Parity tests ensure extracted tables match Tree-sitter exactly

## Building

### Production Build (default)
```bash
# Build with real Tree-sitter headers (vendored)
cargo build -p ts-bridge

# Run the ABI verification
cargo run -p ts-bridge --bin tsb-abi-check
```

### Development Build (stub mode)
```bash
# Build with stub headers for development (outputs dummy data)
cargo build -p ts-bridge --features stub-ts

# NOTE: CLI will fail-fast with stub builds to prevent accidental misuse
```

## Usage

### Extract Parse Tables (dynamic loading)
```bash
# Extract from a compiled Tree-sitter grammar library
cargo run -p ts-bridge -- path/to/libtree-sitter-json.so output.json tree_sitter_json

# The output will be a JSON file containing:
# - Symbol names and counts
# - Parse rules with stable IDs
# - Action table (for terminals) 
# - Goto table (for non-terminals)
# - Start symbol detection
```

### Verify ABI Stability
```bash
# Check header hashes and runtime ABI version
./tools/ts-bridge/scripts/abi-hash.sh
```

## Testing

### Basic Tests (always run)
```bash
cargo test -p ts-bridge --test basic
```

### Parity Tests (requires tree-sitter-json)
```bash
# Enable with-grammars feature to link actual Tree-sitter libraries
cargo test -p ts-bridge --features with-grammars -- --nocapture
```

## Architecture

The bridge works by:
1. Loading a compiled Tree-sitter grammar (`.so`/`.dll`/`.dylib`)
2. Using FFI shim to call Tree-sitter's table access functions
3. Extracting complete parse table data with proper type conversions
4. Serializing to JSON for consumption by GLR runtime or static generation

### Key Components

- `ffi/shim.c`: C shim that interfaces with Tree-sitter API
- `src/extract.rs`: Core extraction logic with width checks and buffer safety
- `src/schema.rs`: Data structures for parse table representation
- `ci/vendor/`: Vendored Tree-sitter headers with SHA pinning

### Safety Features

- **Width checks**: All values verified to fit in u16 with debug assertions
- **Dynamic buffer allocation**: Action buffers expand as needed (no truncation)
- **ABI guards**: Runtime version checks prevent silent breakage
- **Feature gates**: Stub builds clearly marked and fail-fast in production

## ABI Stability

We pin to Tree-sitter language version 15 and use multiple layers of protection:
- Vendored headers with SHA-256 hashes
- Runtime ABI version checks via `tsb_language_version()`
- CI script to detect header drift

## Buffer Management

- Default: 32 actions per table cell (`MAX_ACTIONS_PER_CELL`)
- Automatically expands for larger cells (no silent truncation)
- All buffers properly sized based on actual grammar requirements

## Important Notes

- **Incremental parsing**: Not supported in v1 (requires specialized GLR algorithms)
- **External scanners**: Headers defined but implementation deferred to PR2
- **Field mappings**: Production IDs map to fields via `field_map_slices` (PR2)

## Production Checklist

✅ Build without `stub-ts` feature
✅ Run `tsb-abi-check` to verify ABI compatibility  
✅ Execute `abi-hash.sh` to verify header integrity
✅ Run parity tests with actual grammars
✅ Verify extracted JSON contains valid data (non-zero counts)