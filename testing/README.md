# Adze Beta Testing Framework

This crate provides automated testing tools for validating adze compatibility with Tree-sitter grammars.

## Features

- **Grammar Testing**: Test individual grammars against reference implementations
- **Corpus Testing**: Validate against the official Tree-sitter grammar corpus
- **Performance Benchmarking**: Compare parsing speed with C implementation
- **Compatibility Reports**: Generate detailed JSON and Markdown reports
- **External Scanner Support**: Test grammars with custom scanners
- **Crypto Fixtures**: Generate deterministic RSA PEM fixtures at runtime via `uselesskey`

## Deterministic Crypto Fixtures

Use [`adze_testing::crypto_fixtures`] to generate RSA fixtures in tests without
committing PEM files:

```rust
use adze_testing::crypto_fixtures::{CorruptPem, rsa_fixture};

let keypair = rsa_fixture(module_path!(), "jwt-issuer");
let private_pem = keypair.private_key_pkcs8_pem();
let bad_pem = keypair.private_key_pkcs8_pem_corrupt(CorruptPem::BadHeader);

assert!(private_pem.contains("BEGIN PRIVATE KEY"));
assert_ne!(private_pem, bad_pem);
```

## Installation

```bash
cargo install --path testing
```

## Usage

### Test a Single Grammar

```bash
adze-test test \
  --grammar javascript \
  --path grammars/javascript/grammar.js \
  --files grammars/javascript/test/corpus/*.txt \
  --tree-sitter /usr/local/bin/tree-sitter \
  --benchmark
```

### Run Test Suite

Create a configuration file (`test-suite.json`):

```json
{
  "grammars": [
    {
      "name": "javascript",
      "path": "grammars/javascript/grammar.js",
      "test_files": ["grammars/javascript/test/corpus/*.txt"],
      "tree_sitter_path": "/usr/local/bin/tree-sitter"
    }
  ]
}
```

Run the suite:

```bash
adze-test suite --config test-suite.json --output reports/
```

### Test Tree-sitter Corpus

```bash
adze-test corpus \
  --tree-sitter-path ~/tree-sitter \
  --grammars javascript python rust \
  --output corpus-reports/
```

## Report Format

### Compatibility Report

```json
{
  "version": "0.1.0",
  "date": "2024-01-23T12:00:00Z",
  "total_grammars": 5,
  "passed_grammars": 4,
  "overall_compatibility": 95.2,
  "average_speedup": 1.8,
  "grammar_results": [
    {
      "name": "javascript",
      "compatibility_score": 100.0,
      "total_tests": 150,
      "failed_tests": 0,
      "speedup": 2.1
    }
  ]
}
```

### Markdown Report

The tool generates human-readable Markdown reports with:
- Overall compatibility percentage
- Performance comparisons
- Per-grammar test results
- Detailed error messages

## Grammar Support Status

| Language | Compatibility | Performance | Notes |
|----------|--------------|-------------|-------|
| JavaScript | 100% | 2.1x | Full support |
| Python | 95% | 1.8x | Indentation scanner |
| Rust | 100% | 2.0x | Full support |
| Go | 100% | 1.9x | Full support |
| Ruby | 92% | 1.7x | Heredoc scanner |

## Development

### Adding New Tests

1. Add grammar to `test-suite.json`
2. Ensure test corpus files exist
3. Implement external scanner if needed
4. Run tests and review reports

### Debugging Failed Tests

Set environment variables:
- `ADZE_DEBUG=1` - Enable debug output
- `ADZE_TRACE=1` - Trace parser execution
- `ADZE_EMIT_ARTIFACTS=1` - Save intermediate files

## Future Enhancements

- [ ] Fuzzing support for grammar testing
- [ ] Visual diff tool for parse tree comparison
- [ ] Integration with CI/CD pipelines
- [ ] Web dashboard for test results
- [ ] Automated regression detection
