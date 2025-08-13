# ts-bridge: Tree-sitter to GLR Runtime Bridge

This tool extracts parse tables from compiled Tree-sitter grammars and converts them into a format suitable for our custom GLR runtime.

## Usage

```bash
# Extract parse tables from a compiled grammar
cargo run -p ts-bridge -- path/to/libtree-sitter-json.so output.json tree_sitter_json

# The output will be a JSON file containing:
# - Symbol names and counts
# - Parse rules
# - Action table (for terminals)
# - Goto table (for non-terminals)
```

## Architecture

The bridge works by:
1. Loading a compiled Tree-sitter grammar (`.so`/`.dll`/`.dylib`)
2. Using FFI to call Tree-sitter's table access functions
3. Extracting the complete parse table data
4. Serializing to JSON for consumption by our runtime

## ABI Stability

We pin to Tree-sitter language version 15 and use CI checks to detect ABI drift.

## Important Notes

- **Incremental parsing**: Not supported in v1. The `TSInputEdit` API for incremental updates is documented but not implemented.
- **Node IDs**: Node IDs are not stable across edits in Tree-sitter. This affects incremental analysis.
- **Field mappings**: Production IDs map to field definitions via `field_map_slices`. This will be implemented in PR2.

## Testing

```bash
# Run parity tests to ensure extraction is correct
cargo test -p ts-bridge
```

## Buffer Sizes

The tool uses a maximum of 64 actions per table cell, which should be sufficient for most grammars. Adjust `MAX_ACTIONS_PER_CELL` if needed.