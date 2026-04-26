# Golden-Master Tests for adze

This directory contains golden-master tests that ensure adze parsers produce identical output to the official Tree-sitter parsers.

## Structure

```
golden-tests/
├── python/
│   ├── fixtures/        # Python source files to parse
│   └── expected/        # Expected S-expressions and hashes
├── javascript/
│   ├── fixtures/        # JavaScript source files to parse
│   └── expected/        # Expected S-expressions and hashes
├── generate_references.sh  # Script to generate reference outputs
└── src/lib.rs           # Test harness
```

## Running Tests

### Prerequisites

1. Install Tree-sitter CLI:
   ```bash
   npm install -g tree-sitter-cli
   ```

2. Install language grammars:
   ```bash
   # Python
   git clone https://github.com/tree-sitter/tree-sitter-python
   cd tree-sitter-python
   tree-sitter generate
   cd ..

   # JavaScript
   git clone https://github.com/tree-sitter/tree-sitter-javascript
   cd tree-sitter-javascript
   tree-sitter generate
   cd ..
   ```

### Generate Reference Files

Run this once to generate the golden-master reference files:

```bash
cd golden-tests
./generate_references.sh
```

This will:
- Parse each fixture file using the official Tree-sitter parser
- Generate S-expression output
- Calculate SHA256 hashes for efficient comparison
- Save both in the `expected/` directories

### Run Golden Tests

```bash
# Run all golden tests (when grammars are integrated)
cargo test --features all-grammars

# Run only Python tests
cargo test --features python-grammar

# Run only JavaScript tests  
cargo test --features javascript-grammar
```

### Stable canary (smallest product-facing golden)

The `javascript/fixtures/canary_expression.js` fixture contains a single
semicolon (`;`) and is validated by
`javascript_canary_expression_golden` in strict mode. This test is the current
minimal canary that must parse and match a checked-in S-expression + SHA-256.

```bash
cargo test -p adze-golden-tests javascript_canary_expression_golden --features javascript-grammar -- --nocapture
```

### Update References

If the expected output changes (e.g., due to grammar updates), regenerate references:

```bash
UPDATE_GOLDEN=1 cargo test --features all-grammars
```

Or use the script:

```bash
./generate_references.sh
```

## Adding New Test Cases

1. Add source file to `{language}/fixtures/`
2. Run `./generate_references.sh` to generate expected output
3. Add test case to `src/lib.rs`:

```rust
#[test]
#[cfg(feature = "python-grammar")]
fn python_my_new_test() -> Result<()> {
    run_golden_test(GoldenTest {
        language: "python",
        fixture_name: "my_new_test.py",
    })
}
```

## How It Works

1. **Reference Generation**: Uses official Tree-sitter to parse fixtures and save S-expressions
2. **Hash Comparison**: Computes SHA256 of S-expressions for fast comparison
3. **Detailed Diff**: On mismatch, saves actual output for debugging
4. **CI Integration**: Tests run in CI to catch regressions

## Benefits

- **Byte-for-byte compatibility**: Ensures adze matches Tree-sitter exactly
- **Real-world code**: Tests against actual Python/JS code, not just synthetic examples
- **Fast comparison**: SHA256 hashes avoid storing large S-expression files in git
- **Easy debugging**: On failure, both expected and actual outputs are available
- **Regression prevention**: CI catches any deviation from expected behavior
