# Adze Testing Framework

Adze leverages the standard Rust testing ecosystem, augmented with specific tools for parser validation.

## Overview

- **Unit Tests**: Standard `#[test]` functions for grammar verification.
- **Snapshot Testing**: Use `insta` to track AST changes over time.
- **Corpus Tests**: Validate against large sets of real-world input files (common in Tree-sitter ecosystems).
- **Property Testing**: Optional integration with `proptest` or `quickcheck`.

## Quick Start

Since Adze generates Rust code, you test it like any other Rust project:

```bash
cargo test
```

The CLI provides a convenience wrapper that also manages snapshot updates:

```bash
# Run tests
adze test

# Run tests and update snapshots
adze test --update
```

## Snapshot Testing (Recommended)

We recommend using [insta](https://crates.io/crates/insta) to snapshot your parse trees. This makes it easy to see how grammar changes affect parsing output.

### Setup

Add `insta` to your `dev-dependencies`:

```toml
[dev-dependencies]
insta = "1.40"
```

### Writing a Test

```rust
use my_grammar::grammar;
use insta::assert_debug_snapshot;

#[test]
fn test_parser_snapshot() {
    let input = "1 + 2";
    let tree = grammar::parse(input).unwrap();
    assert_debug_snapshot!(tree);
}
```

### Reviewing Changes

When you change your grammar, tests will fail with a diff. Review and accept them:

```bash
cargo insta review
```

## Corpus Testing

For larger languages, it is common to have a `corpus/` directory with text files. You can write a custom test runner to iterate these files.

```rust
#[test]
fn test_corpus() {
    let corpus_dir = std::path::Path::new("tests/corpus");
    for entry in std::fs::read_dir(corpus_dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.extension().unwrap() == "txt" {
            let input = std::fs::read_to_string(&path).unwrap();
            let result = grammar::parse(&input);
            assert!(result.is_ok(), "Failed to parse {}", path.display());
        }
    }
}
```

## Integration with CI

Since standard `cargo test` works, no special CI setup is required.

```yaml
# .github/workflows/test.yml
steps:
  - uses: actions/checkout@v4
  - uses: dtolnay/rust-toolchain@stable
  - run: cargo test
```

## Debugging Parse Errors

If a test fails, you can enable debug logging to see the parser's internal state (shifts, reductions, etc.):

```bash
RUST_LOG=adze=debug cargo test test_name -- --nocapture
```

For GLR debugging (ambiguities):

```bash
ADZE_LOG_PERFORMANCE=true cargo test
```
