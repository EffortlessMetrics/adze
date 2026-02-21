# CLI Reference

Complete reference for the adze command-line interface.

## Installation

Install the CLI tool:

```bash
cargo install adze-cli
```

Or build from source:

```bash
git clone https://github.com/hydro-project/adze
cd adze
cargo build --release -p adze-cli
```

## Global Options

### `--verbose, -v`

Enable verbose output showing detailed processing information.

```bash
adze --verbose parse grammar.rs input.txt
```

### `--help, -h`

Show help information for commands.

```bash
adze --help
adze parse --help
```

## Commands

### `init`

Create a new adze grammar project.

**Usage:**
```bash
adze init <NAME> [OPTIONS]
```

**Arguments:**
- `<NAME>` - Name of the grammar project

**Options:**
- `--output, -o <DIR>` - Output directory (default: current directory)

**Example:**
```bash
# Create a new JSON grammar
adze init json-parser

# Create in specific directory
adze init json-parser --output ~/projects/
```

**Generated Structure:**
```
json-parser/
├── Cargo.toml
├── build.rs
├── src/
│   ├── lib.rs
│   └── grammar.rs
├── tests/
│   └── basic.rs
├── examples/
└── README.md
```

### `build`

Build grammar parsers from adze grammar definitions.

**Usage:**
```bash
adze build [PATH] [OPTIONS]
```

**Arguments:**
- `[PATH]` - Path to grammar file or directory (default: current directory)

**Options:**
- `--watch, -w` - Watch for changes and rebuild automatically

**Examples:**
```bash
# Build grammar in current directory
adze build

# Build specific grammar file
adze build src/grammar.rs

# Watch for changes
adze build --watch
```

**Watch Mode:**
Automatically rebuilds when `.rs` files change. Useful during grammar development.

### `parse`

Parse input files using adze grammars.

**Usage:**
```bash
adze parse <GRAMMAR> <INPUT> [OPTIONS]
```

**Arguments:**
- `<GRAMMAR>` - Grammar file path, or library path when using `--dynamic`
- `<INPUT>` - Input file to parse

**Options:**
- `--format, -f <FORMAT>` - Output format: `tree` (default), `json`, `sexp`, `dot`
- `--dynamic` - Use dynamic loading from shared library
- `--symbol <SYMBOL>` - Symbol name for dynamic loading (default: "language")

**Static Parsing Examples:**
```bash
# Parse with tree output (default)
adze parse grammar.rs input.txt

# Parse with JSON output
adze parse grammar.rs input.txt --format json

# Parse with S-expression output  
adze parse grammar.rs input.txt --format sexp
```

**Dynamic Loading Examples:**
```bash
# Parse JSON file with tree-sitter-json
adze parse --dynamic libtree-sitter-json.so input.json

# Use custom symbol name
adze parse --dynamic libmy-lang.so input.txt --symbol tree_sitter_mylang

# JSON output for tooling
adze parse --dynamic libpython.so script.py --format json
```

**Output Formats:**

**Tree Format** (human-readable):
```
Parsed successfully. Root symbol: document, nodes: 127
Input size: 1024 bytes
```

**JSON Format** (machine-readable):
```json
{
  "status": "ok",
  "root_symbol": "document", 
  "nodes": 127
}
```

**S-Expression Format** (Lisp-style):
```lisp
(document (statement (expression (number "42"))))
```

**Error Output:**
```json
{
  "status": "error",
  "errors": 3,
  "message": "Parse tree contains errors"
}
```

### `test`

Run tests for adze grammars.

**Usage:**
```bash
adze test [PATH] [OPTIONS]
```

**Arguments:**
- `[PATH]` - Path to grammar directory (default: current directory)

**Options:**
- `--update, -u` - Update test snapshots

**Examples:**
```bash
# Run tests
adze test

# Update snapshots
adze test --update
```

Uses `cargo test` internally with `insta` snapshot testing.

### `doc`

Generate documentation from grammar files.

**Usage:**
```bash
adze doc <GRAMMAR> [OPTIONS]
```

**Arguments:**
- `<GRAMMAR>` - Path to grammar file

**Options:**
- `--output, -o <FILE>` - Output file (default: stdout)

**Example:**
```bash
# Output to console
adze doc src/grammar.rs

# Save to file
adze doc src/grammar.rs --output docs/grammar.md
```

Extracts documentation from `///` comments in grammar files.

### `check`

Validate grammar syntax without full compilation.

**Usage:**
```bash
adze check <GRAMMAR>
```

**Arguments:**
- `<GRAMMAR>` - Path to grammar file

**Example:**
```bash
adze check src/grammar.rs
```

**Output:**
```
✅ Grammar syntax is valid
```

Fast validation for CI/CD pipelines and editors.

### `stats`

Show statistics about grammar files.

**Usage:**
```bash
adze stats <GRAMMAR>
```

**Arguments:**
- `<GRAMMAR>` - Path to grammar file

**Example:**
```bash
adze stats src/grammar.rs
```

**Output:**
```
📊 Grammar statistics:
  Lines: 245
  Rules: 28
  Leaf rules: 15
  Repeat rules: 8
```

Useful for tracking grammar complexity and growth.

## Dynamic Loading

### Supported Formats

Dynamic loading supports standard Tree-sitter library formats:

**Linux:**
- `.so` files (shared objects)
- Example: `libtree-sitter-json.so`

**macOS:**
- `.dylib` files (dynamic libraries)  
- Example: `libtree-sitter-json.dylib`

**Windows:**
- `.dll` files (dynamic link libraries)
- Example: `tree-sitter-json.dll`

### Common Library Locations

**System Package Locations:**
```bash
# Ubuntu/Debian
/usr/lib/x86_64-linux-gnu/libtree-sitter-*.so

# Fedora/CentOS
/usr/lib64/libtree-sitter-*.so

# macOS (Homebrew)
/opt/homebrew/lib/libtree-sitter-*.dylib

# macOS (MacPorts)
/opt/local/lib/libtree-sitter-*.dylib
```

**Language-Specific Locations:**
```bash
# Node.js installations
node_modules/tree-sitter-*/bindings/node/

# Python installations  
~/.local/lib/python*/site-packages/tree_sitter_*/

# Cargo target directory
target/release/deps/libtree_sitter_*.so
```

### Symbol Names

Common symbol naming patterns:

```bash
# Standard pattern: tree_sitter_{language}
tree_sitter_json
tree_sitter_python  
tree_sitter_javascript
tree_sitter_rust
tree_sitter_cpp

# Alternative patterns
language                    # Generic name
tree_sitter_{lang}_language # Extended format
get_language               # Function-style name
```

Use `nm -D library.so | grep tree_sitter` to find available symbols.

## FFI Safety and Security

### Input Validation

- **Library Path**: Existence and readability checks
- **Input Files**: Size limits (100MB), UTF-8 validation
- **Symbol Names**: Alphanumeric + underscore validation
- **Memory Safety**: Pointer null checks, alignment validation

### Error Handling

- **Library Loading**: Detailed error messages for missing dependencies
- **Symbol Resolution**: Clear indication of available symbols
- **Parsing Errors**: Graceful failure with partial results when possible
- **Stack Protection**: Depth limits prevent stack overflow

### Security Considerations

- **No Code Execution**: Only data extraction from compiled libraries
- **Sandboxed Operation**: No file system access beyond specified paths
- **Memory Bounds**: All memory access is bounds-checked
- **Timeout Protection**: Automatic termination for runaway parsing

## Environment Variables

### `RUST_LOG`

Control logging output:

```bash
# Show all debug information
RUST_LOG=debug adze parse --dynamic lib.so input.txt

# Show only warnings and errors
RUST_LOG=warn adze build
```

### `ADZE_LOG_PERFORMANCE`

Enable performance monitoring for GLR parsing:

```bash
ADZE_LOG_PERFORMANCE=true adze parse grammar.rs input.txt
```

Output example:
```
🚀 Forest->Tree conversion: 247 nodes, depth 12, took 0.8ms
```

### `INSTA_UPDATE`

Control snapshot testing behavior:

```bash
# Always update snapshots
INSTA_UPDATE=always adze test

# Never update snapshots (CI mode)
INSTA_UPDATE=no adze test
```

## Exit Codes

- `0` - Success
- `1` - General error (parsing failure, invalid grammar, etc.)
- `2` - Missing feature (e.g., dynamic loading not compiled)
- `3` - Invalid arguments or usage
- `4` - File system error (permission denied, file not found)

## Configuration Files

### `rust-toolchain.toml`

Rust-sitter requires specific toolchain configuration:

```toml
[toolchain]
channel = "1.89"
edition = "2024"
components = ["rustfmt", "clippy"]
```

### `.gitignore` Additions

Generated parser files to ignore:

```gitignore
# Rust-sitter generated files
target/
parser.c
tree_sitter/
src/tree_sitter/
grammar.json
node-types.json
```

## Troubleshooting

### Common Issues

**"symbol not found"**
```bash
# Check available symbols
nm -D library.so | grep tree_sitter

# Try common symbol names
adze parse --dynamic lib.so input.txt --symbol language
adze parse --dynamic lib.so input.txt --symbol get_language
```

**"library not found"**
```bash
# Check library path
ls -la /path/to/library.so

# Check dependencies
ldd /path/to/library.so

# Install system dependencies
sudo apt install libtree-sitter-dev  # Ubuntu
brew install tree-sitter              # macOS
```

**"Dynamic loading not enabled"**
```bash
# Rebuild with dynamic feature
cargo build --features dynamic

# Or install with dynamic support
cargo install adze-cli --features dynamic
```

**"Parse tree too deep"**
- Indicates possible grammar recursion
- Check for left-recursive rules
- Use GLR mode for ambiguous grammars

**"Input too large"**
- Files over 100MB require streaming
- Consider splitting large inputs
- Use incremental parsing for large files

### Debug Mode

Enable verbose debugging:

```bash
adze --verbose parse --dynamic lib.so input.txt
```

Shows:
- Library loading steps
- Symbol resolution details  
- Parser creation process
- Memory usage information
- Performance timing

### Performance Tips

1. **Use Dynamic Loading** for quick experimentation
2. **Enable Performance Monitoring** to identify bottlenecks
3. **Use JSON Format** for machine processing
4. **Limit Text Length** in serialization for large trees
5. **Use GLR Runtime** for best performance and features

## Integration Examples

### Shell Scripting

```bash
#!/bin/bash
# Parse all Python files in a directory

for file in *.py; do
    result=$(adze parse --dynamic libtree-sitter-python.so "$file" --format json)
    status=$(echo "$result" | jq -r '.status')
    
    if [ "$status" = "ok" ]; then
        echo "✅ $file parsed successfully"
    else
        echo "❌ $file failed to parse"
    fi
done
```

### CI/CD Pipeline

```yaml
# GitHub Actions example
- name: Validate Grammar
  run: adze check src/grammar.rs

- name: Test Grammar  
  run: adze test --update=no

- name: Parse Test Files
  run: |
    for test_file in tests/fixtures/*.txt; do
      adze parse grammar.rs "$test_file" --format json
    done
```

### Editor Integration

```json
// VS Code task
{
    "label": "Parse Current File",
    "type": "shell", 
    "command": "adze",
    "args": [
        "parse", 
        "--dynamic",
        "libtree-sitter-${fileExtname}.so",
        "${file}",
        "--format", 
        "json"
    ]
}
```