# Adze Testing Framework

Comprehensive testing framework for grammar development and validation.

## Overview

The Adze testing framework provides multiple testing strategies to ensure grammar correctness, performance, and compatibility:

- **Corpus Testing**: Test against real-world code examples
- **Property-Based Testing**: Automatically generate test cases
- **Fuzz Testing**: Find edge cases and crashes
- **Performance Benchmarking**: Measure and optimize speed
- **Differential Testing**: Compare against Tree-sitter
- **Snapshot Testing**: Track parser output changes

## Quick Start

```bash
# Run all tests for your grammar
adze test

# Run specific test suite
adze test --suite corpus

# Run with coverage
adze test --coverage

# Benchmark performance
adze bench
```

## Corpus Testing

### Directory Structure
```
tests/
├── corpus/
│   ├── basic.txt
│   ├── edge_cases.txt
│   └── real_world/
│       ├── example1.rs
│       └── example2.rs
└── fixtures/
    └── invalid/
```

### Test Format
```
=================
Basic Function
=================

fn main() {
    println!("Hello, world!");
}

---

(source_file
  (function_definition
    name: (identifier)
    parameters: (parameters)
    body: (block
      (expression_statement
        (macro_call
          name: (identifier)
          arguments: (arguments
            (string_literal)))))))

=================
With Comments
=================

// This is a comment
fn test() {} // inline

---

(source_file
  (comment)
  (function_definition
    name: (identifier)
    parameters: (parameters)
    body: (block))
  (comment))
```

### Running Corpus Tests
```rust
use adze::testing::{TestRunner, CorpusConfig};

#[test]
fn test_corpus() {
    let config = CorpusConfig::default()
        .with_dir("tests/corpus")
        .with_extensions(vec!["txt"])
        .with_update_mode(false);
    
    let mut runner = TestRunner::new(grammar());
    runner.run_corpus(config).unwrap();
}
```

## Property-Based Testing

### Automatic Test Generation
```rust
use adze::testing::{PropertyTest, Arbitrary};

#[test]
fn test_properties() {
    let mut tester = PropertyTest::new(grammar());
    
    // Test that all generated code can be parsed
    tester.check_parseable(1000);
    
    // Test that pretty-printing is stable
    tester.check_roundtrip(1000);
    
    // Test incremental parsing consistency
    tester.check_incremental(500);
}
```

### Custom Properties
```rust
impl Arbitrary for MyExpression {
    fn arbitrary(g: &mut Gen) -> Self {
        match g.gen_range(0..3) {
            0 => MyExpression::Number(g.gen()),
            1 => MyExpression::Identifier(
                (0..g.gen_range(1..10))
                    .map(|_| g.gen::<char>())
                    .collect()
            ),
            _ => MyExpression::Binary {
                left: Box::new(Self::arbitrary(g)),
                op: BinaryOp::arbitrary(g),
                right: Box::new(Self::arbitrary(g)),
            },
        }
    }
}
```

## Fuzz Testing

### Coverage-Guided Fuzzing
```rust
use adze::testing::{Fuzzer, FuzzConfig};

#[test]
fn fuzz_grammar() {
    let config = FuzzConfig::default()
        .with_max_len(1000)
        .with_timeout(Duration::from_secs(60))
        .with_corpus_dir("fuzz/corpus")
        .with_dict("fuzz/dict.txt")
        .with_coverage_guided(true);
    
    let mut fuzzer = Fuzzer::new(grammar(), config);
    fuzzer.fuzz();
}
```

### Continuous Fuzzing
```yaml
# .github/workflows/fuzz.yml
name: Fuzz Testing
on:
  schedule:
    - cron: '0 0 * * *'  # Daily
jobs:
  fuzz:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: adze/fuzz-action@v1
        with:
          duration: 3600  # 1 hour
          corpus: fuzz/corpus
```

## Performance Benchmarking

### Benchmark Suite
```rust
use adze::testing::{Benchmark, BenchConfig};

#[bench]
fn bench_parsing(b: &mut Bencher) {
    let config = BenchConfig::default()
        .with_files("benches/samples/*.rs")
        .with_warmup(10)
        .with_iterations(100);
    
    let bench = Benchmark::new(grammar(), config);
    b.iter(|| bench.run());
}
```

### Performance Tracking
```bash
# Run benchmarks and save results
adze bench --save results.json

# Compare with previous run
adze bench --baseline results.json

# Generate performance report
adze bench --report html
```

### Profiling
```bash
# CPU profiling
adze profile --flamegraph

# Memory profiling
adze profile --memory

# Cache analysis
adze profile --cache-stats
```

## Differential Testing

### Compare with Tree-sitter
```rust
use adze::testing::{DifferentialTest, TreeSitterGrammar};

#[test]
fn test_compatibility() {
    let ts_grammar = TreeSitterGrammar::load("tree-sitter-rust");
    let rs_grammar = grammar();
    
    let mut tester = DifferentialTest::new(ts_grammar, rs_grammar);
    
    // Test on corpus
    tester.test_corpus("tests/corpus/**/*.txt");
    
    // Test on real files
    tester.test_files("examples/**/*.rs");
    
    // Generate compatibility report
    let report = tester.compatibility_report();
    assert!(report.compatibility >= 0.99);
}
```

## Grammar Validation

### Lint Checks
```rust
use adze::testing::{GrammarLinter, LintLevel};

#[test]
fn lint_grammar() {
    let mut linter = GrammarLinter::new()
        .level(LintLevel::Strict)
        .enable_all();
    
    let results = linter.lint(grammar());
    
    for issue in results.issues() {
        match issue.severity {
            Severity::Error => panic!("{}", issue),
            Severity::Warning => eprintln!("Warning: {}", issue),
            Severity::Info => println!("Info: {}", issue),
        }
    }
}
```

### Common Lint Rules
- Unreachable rules
- Ambiguous patterns
- Missing error recovery
- Inefficient rule ordering
- Naming conventions
- Documentation coverage

## Snapshot Testing

### Using Insta
```rust
use insta::assert_snapshot;
use adze::testing::format_tree;

#[test]
fn test_snapshot() {
    let tree = grammar::parse("fn main() {}").unwrap();
    assert_snapshot!(format_tree(&tree));
}
```

### Updating Snapshots
```bash
# Review and update snapshots
cargo insta review

# Auto-accept all changes
cargo insta accept
```

## Test Organization

### Test Macros
```rust
use adze::test_grammar;

test_grammar! {
    grammar: my_grammar,
    corpus: "tests/corpus",
    
    pass: {
        "simple function" => "fn f() {}",
        "with params" => "fn f(x: i32) {}",
    },
    
    fail: {
        "missing brace" => "fn f() {",
        "invalid syntax" => "fn fn fn",
    },
    
    bench: {
        "large file" => include_str!("large.rs"),
    }
}
```

## CI Integration

### GitHub Actions
```yaml
name: Grammar Tests
on: [push, pull_request]
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: adze/test-action@v1
        with:
          coverage: true
          bench: true
          fuzz-duration: 300
```

### Pre-commit Hooks
```yaml
# .pre-commit-config.yaml
repos:
  # See https://github.com/EffortlessMetrics/adze for hook examples
  - repo: local
    hooks:
      - id: adze-test
      - id: adze-lint
      - id: adze-format
```

## Test Coverage

### Generate Coverage Report
```bash
# Run with coverage
adze test --coverage

# Generate HTML report
adze coverage --html

# Upload to Codecov
adze coverage --codecov
```

### Coverage Requirements
```toml
# adze.toml
[coverage]
minimum = 90
exclude = ["tests/**", "benches/**"]
```

## Debugging Tests

### Debug Mode
```bash
# Step through parsing
adze debug "fn main() {}"

# Show parser states
adze debug --states

# Trace token flow
adze debug --trace
```

### Test Utilities
```rust
use adze::testing::{assert_parse, assert_parse_error};

// Assert successful parse
assert_parse!(grammar, "fn main() {}");

// Assert parse error
assert_parse_error!(grammar, "fn fn fn");

// Assert specific tree structure
assert_parse!(grammar, "fn f() {}" => {
    root: {
        kind: "source_file",
        children: [{
            kind: "function_definition",
            field("name"): { text: "f" }
        }]
    }
});
```

## Best Practices

1. **Start with Corpus Tests**: Real examples catch most issues
2. **Add Property Tests**: Find edge cases automatically  
3. **Benchmark Early**: Track performance regressions
4. **Fuzz Regularly**: Discover crashes and hangs
5. **Monitor Coverage**: Ensure comprehensive testing
6. **Use Snapshots**: Track output stability
7. **Automate in CI**: Catch issues before merge

## Resources

- [Example Test Suites](../example/)
- [Developer Guide](./DEVELOPER_GUIDE.md)
