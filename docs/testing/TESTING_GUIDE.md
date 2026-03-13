# Adze Testing Guide

This document provides comprehensive documentation for the Adze project's testing infrastructure, covering ~39,000+ tests across multiple testing methodologies.

## Table of Contents

1. [Testing Philosophy](#testing-philosophy)
2. [Test Categories](#test-categories)
3. [Running Tests](#running-tests)
4. [Writing Tests](#writing-tests)
5. [Test Infrastructure](#test-infrastructure)
6. [CI Integration](#ci-integration)
7. [Quick Reference](#quick-reference)

---

## Testing Philosophy

The Adze project employs a multi-layered testing strategy that emphasizes correctness, maintainability, and developer productivity.

### BDD-First Approach

Adze uses **Behavior-Driven Development (BDD)** for complex parser scenarios, particularly GLR parsing. BDD scenarios serve as executable specifications that document expected behavior.

**Benefits**:
- **Documentation as tests**: Scenarios serve dual purpose as specification and verification
- **Faster debugging**: Failures point to specific behavioral expectations
- **Better communication**: Gherkin syntax is accessible to non-developers
- **60% time savings**: Reported efficiency gain from structured approach

**Example BDD Scenario** (from [`BDD_GLR_CONFLICT_PRESERVATION.md`](../archive/plans/BDD_GLR_CONFLICT_PRESERVATION.md)):

```gherkin
Feature: GLR Conflict Detection and Preservation

  Scenario: Preserve Conflicts with Precedence Ordering (PreferShift)
    Given a shift/reduce conflict with precedence favoring shift
    When resolve_shift_reduce_conflict() is called
    Then both actions are preserved in order [shift, reduce]
    And the first action (shift) has higher runtime priority
```

See [ADR-007: BDD Framework for Parser Testing](../adr/007-bdd-framework-for-parser-testing.md) for the full rationale.

### Property-Based Testing with Proptest

Property-based testing generates random inputs to find edge cases that developers might miss. Adze uses `proptest` extensively for:

- Grammar builder invariants
- IR roundtrip serialization
- Symbol ID allocation
- Parse table generation

**Key Property**: "For all valid grammars, the builder produces a structurally valid Grammar"

```rust
proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn test_valid_grammar_single_token(
        name in grammar_name_strategy(),
        tname in token_name_strategy(),
        pat in token_pattern_strategy(),
    ) {
        let g = GrammarBuilder::new(&name)
            .token(&tname, &pat)
            .build();
        prop_assert!(!g.name.is_empty());
    }
}
```

### Snapshot Testing with Insta

Snapshot tests capture expected output and alert when changes occur. Adze uses `insta` for:

- Grammar normalization output
- Optimizer transformations
- JSON serialization formats
- Validation error messages

**Review snapshots with**:
```bash
cargo insta review
```

### Contract-Driven Testing

Test infrastructure enforces contracts between components:

- **Parse table invariants**: EOF column index must equal `token_count + external_token_count`
- **Symbol layout**: ERROR at column 0, terminals follow, then non-terminals
- **State validity**: Initial state must be in range of `state_count`

---

## Test Categories

### Unit Tests

Unit tests verify individual functions and methods in isolation.

**Location**: Inline in `src/` files under `#[cfg(test)]` modules

**Example** (from [`glr-test-support/src/lib.rs`](../../glr-test-support/src/lib.rs)):
```rust
/// Assert that a ParseTable respects all invariants
pub fn assert_parse_table_invariants(table: &ParseTable) {
    let eof_column = table.symbol_to_index.get(&table.eof_symbol);
    let expected_eof_column = table.token_count + table.external_token_count;
    assert_eq!(*eof_column.unwrap(), expected_eof_column);
}
```

### Integration Tests

Integration tests verify component interactions across crate boundaries.

**Location**: `tests/` directories within each crate

**Key integration test files**:
- [`ir/tests/e2e_pipeline_comprehensive.rs`](../../ir/tests/e2e_pipeline_comprehensive.rs) - End-to-end IR pipeline
- [`ir/tests/ir_to_glr_integration.rs`](../../ir/tests/ir_to_glr_integration.rs) - IR to GLR conversion
- [`testing/tests/cross_crate_pipeline.rs`](../../testing/tests/cross_crate_pipeline.rs) - Cross-crate workflows

### Property-Based Tests

Property tests use `proptest` to generate random inputs and verify invariants.

**Location**: Files with `proptest` in the name (e.g., `ir/tests/proptest_*.rs`)

**Key property test files**:
| File | Purpose |
|------|---------|
| [`ir/tests/proptest_builder_v3.rs`](../../ir/tests/proptest_builder_v3.rs) | GrammarBuilder properties |
| [`ir/tests/grammar_proptest.rs`](../../ir/tests/grammar_proptest.rs) | Grammar invariants |
| [`ir/tests/serde_roundtrip_comprehensive.rs`](../../ir/tests/serde_roundtrip_comprehensive.rs) | Serialization properties |
| [`ir/tests/optimizer_proptest.rs`](../../ir/tests/optimizer_proptest.rs) | Optimizer invariants |

**Common strategies** (from [`testing/src/strategies.rs`](../../testing/src/strategies.rs)):
```rust
/// Strategy that generates valid identifier-style names
pub fn ident_strategy() -> impl Strategy<Value = String> {
    prop::string::string_regex("[a-z][a-z0-9_]{0,7}")
        .unwrap()
        .prop_filter("non-empty", |s| !s.is_empty())
}

/// Strategy that generates a small but valid grammar
pub fn small_grammar_strategy() -> impl Strategy<Value = Grammar> { ... }
```

### Snapshot Tests

Snapshot tests capture expected output for comparison.

**Location**: `tests/snapshots/` directories with `.snap` files

**Key snapshot test files**:
- [`ir/tests/snapshot_tests.rs`](../../ir/tests/snapshot_tests.rs) - Normalization, optimization snapshots
- [`ir/tests/grammar_json_format.rs`](../../ir/tests/grammar_json_format.rs) - JSON serialization

**Snapshot review workflow**:
```bash
# After tests generate new snapshots
cargo insta review

# Accept all changes
cargo insta accept
```

### Fuzz Tests

Fuzz tests use `cargo fuzz` for continuous random input generation.

**Location**: [`fuzz/fuzz_targets/`](../../fuzz/fuzz_targets/)

**Key fuzz targets**:
| Target | Purpose |
|--------|---------|
| `fuzz_parser.rs` | Parser robustness |
| `fuzz_lexer.rs` | Lexer edge cases |
| `fuzz_bdd_grammar_analysis_core.rs` | Grammar analysis |
| `fuzz_concurrency_*.rs` | Concurrency safety |

**Running fuzz tests**:
```bash
cargo fuzz run fuzz_parser
```

### Golden Tests

Golden tests verify output matches reference files from Tree-sitter.

**Location**: [`golden-tests/`](../../golden-tests/)

**Structure**:
```
golden-tests/
├── python/
│   ├── fixtures/        # Python source files to parse
│   └── expected/        # Expected S-expressions and hashes
├── javascript/
│   ├── fixtures/        # JavaScript source files
│   └── expected/        # Expected outputs
└── src/lib.rs           # Test harness
```

**Running golden tests**:
```bash
# Run with all grammars
cargo test --features all-grammars

# Update golden references
UPDATE_GOLDEN=1 cargo test --features all-grammars
```

---

## Running Tests

### Essential Commands

```bash
# Core lib tests (recommended for daily development)
just test

# All tests with 2 threads
cargo t2

# Test specific crate
cargo test -p adze-ir

# Run specific test by name
cargo test test_normalize_simple

# Run ignored tests
cargo test -- --ignored
```

### PR Gate (Required Before Submitting)

```bash
just ci-supported
```

This runs:
1. `cargo fmt --all -- --check` - Format verification
2. `cargo clippy` on core crates - Linting with `-D warnings`
3. `cargo test` on core crates - All tests
4. Doc tests with serialization feature

### Snapshot Review

```bash
# Review pending snapshot changes
just snap
# Or equivalently:
cargo insta review
```

### Feature Matrix Testing

```bash
# Run feature matrix tests
just matrix
# Or:
./scripts/test-matrix.sh
```

### Mutation Testing

```bash
# Quick mutation check on adze-ir
just mutate

# Mutation test all supported crates
just mutate-all
```

### Test Thread Configuration

Tests are configured for stability across environments:

| Variable | Default | CI Value | Purpose |
|----------|---------|----------|---------|
| `RUST_TEST_THREADS` | 2 | 2 | Test concurrency |
| `RAYON_NUM_THREADS` | 4 | 4 | Rayon thread pool |
| `CARGO_BUILD_JOBS` | 4 | 2 | Parallel builds |

```bash
# Override thread count
RUST_TEST_THREADS=4 cargo test
```

---

## Writing Tests

### Test Naming Conventions

Follow the pattern: `test_<what>_<condition>_<expected>`

**Examples**:
```rust
#[test]
fn test_normalize_optional_completes() { }

#[test]
fn test_grammar_builder_empty_rhs_produces_epsilon() { }

#[test]
fn test_parse_table_invalid_state_returns_error() { }
```

### BDD Scenario Format

BDD scenarios follow Given-When-Then structure:

```gherkin
Feature: <feature name>

  Scenario: <scenario description>
    Given <initial context>
    When <action occurs>
    Then <expected outcome>
    And <additional assertions>
```

**Mapping to tests**:
Each scenario has acceptance criteria checkboxes that map to test implementations:

```markdown
**Acceptance Criteria**:
- [ ] Conflict detected in correct state
- [ ] Both shift and reduce actions identified
- [ ] Conflict type correctly classified as ShiftReduce
```

### Property Test Patterns

**Basic property test structure**:
```rust
use proptest::prelude::*;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn test_property_name(
        input1 in strategy1(),
        input2 in strategy2(),
    ) {
        // Execute system under test
        let result = function_under_test(input1, input2);

        // Assert property holds
        prop_assert!(result_satisfies_invariant(&result));
    }
}
```

**Using shared strategies**:
```rust
use testing::strategies::{ident_strategy, small_grammar_strategy};

proptest! {
    #[test]
    fn test_grammar_roundtrip(grammar in small_grammar_strategy()) {
        let json = serde_json::to_string(&grammar).unwrap();
        let parsed: Grammar = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(grammar, parsed);
    }
}
```

### Snapshot Test Patterns

**Basic snapshot test**:
```rust
#[test]
fn normalize_simple_grammar() {
    let mut grammar = Grammar::new("simple".into());
    // ... setup grammar ...

    grammar.normalize();
    insta::assert_snapshot!("normalize_simple", render_rules(&grammar));
}
```

**Snapshot with redactions** (for non-deterministic output):
```rust
insta::assert_json_snapshot!(&result, {
    ".timestamp" => "[timestamp]",
    ".id" => "[uuid]",
});
```

### Unit Test Patterns

**Using test assertions** (from [`testing/src/assertions.rs`](../../testing/src/assertions.rs)):
```rust
use testing::assertions::{assert_has_rule, assert_has_token, assert_start_symbol};

#[test]
fn test_grammar_structure() {
    let grammar = build_test_grammar();

    assert_has_rule(&grammar, "expr");
    assert_has_token(&grammar, "NUMBER");
    assert_start_symbol(&grammar, "program");
}
```

**Using fixtures** (from [`testing/src/fixtures.rs`](../../testing/src/fixtures.rs)):
```rust
use testing::fixtures::{load_fixture_or_panic, parse_corpus};

#[test]
fn test_with_fixture() {
    let source = load_fixture_or_panic("test.txt");
    // ... test with fixture content ...
}

#[test]
fn test_corpus_entry() {
    let entries = parse_corpus(&load_fixture_or_panic("corpus.txt"));
    assert_eq!(entries.len(), 3);
}
```

### Feature-Gated Test APIs

Tests requiring internal access should use feature gates:

```rust
// In library code
#[cfg(feature = "test-api")]
pub fn internal_state(&self) -> &InternalState { ... }

// In test code
#[cfg(feature = "test-api")]
#[test]
fn test_internal_state() { ... }
```

---

## Test Infrastructure

### `testing` Crate

**Location**: [`testing/`](../../testing/)

The main testing framework providing:

| Module | Purpose |
|--------|---------|
| [`assertions.rs`](../../testing/src/assertions.rs) | Custom assertion helpers for grammar/parse-table verification |
| [`fixtures.rs`](../../testing/src/fixtures.rs) | Test fixture loading and corpus file parsing |
| [`grammar_helpers.rs`](../../testing/src/grammar_helpers.rs) | Grammar construction helpers |
| [`snapshots.rs`](../../testing/src/snapshots.rs) | Snapshot comparison utilities |
| [`strategies.rs`](../../testing/src/strategies.rs) | Proptest strategies for grammar types |

**BetaTester framework** (from [`testing/src/lib.rs`](../../testing/src/lib.rs)):
```rust
pub struct BetaTester {
    results: Vec<GrammarTestResult>,
    config: TestConfig,
}

impl BetaTester {
    pub fn test_grammar(&mut self, grammar_name: &str) -> Result<GrammarTestResult>;
    pub fn compare_with_tree_sitter(&self, test_file: &Path, result: &TestResult);
}
```

### `glr-test-support` Crate

**Location**: [`glr-test-support/`](../../glr-test-support/)

Specialized support for GLR parser testing:

```rust
/// Build a minimal but fully-formed ParseTable suitable for unit tests
pub fn make_minimal_table(
    actions: Vec<Vec<Vec<Action>>>,
    gotos: Vec<Vec<StateId>>,
    rules: Vec<ParseRule>,
    start_symbol: SymbolId,
    eof_symbol: SymbolId,
    external_token_count: usize,
) -> ParseTable;

/// Assert that a ParseTable respects all invariants
pub fn assert_parse_table_invariants(table: &ParseTable);
```

**Key invariant**: EOF column index must equal `token_count + external_token_count`

### Test Fixtures

**Location**: [`corpus/`](../../corpus/) and crate-level `fixtures/` directories

**Fixture formats**:

1. **Plain text** - Entire file as test input
2. **Tree-sitter corpus format** - Structured test entries:
   ```
   === Title of test
   source code here
   ---

   (expected_tree)
   ```

**Loading fixtures**:
```rust
use testing::fixtures::{fixtures_dir, load_fixture, load_fixture_or_panic};

let fixture_path = fixtures_dir().join("test.txt");
let content = load_fixture("javascript/statements.txt");
```

### Benchmark Fixtures

**Location**: [`benchmarks/fixtures/`](../../benchmarks/fixtures/)

Fixtures for performance testing:
- `python/small.py`, `medium.py`, `large.py`
- `javascript/small.js`, `medium.js`, `large.js`

**Verification test**: [`benchmarks/tests/verify_fixture_parsing.rs`](../../benchmarks/tests/verify_fixture_parsing.rs)

---

## CI Integration

### PR Gate: `just ci-supported`

The single required check for branch protection:

```bash
just ci-supported
```

Runs on 7 core pipeline crates:
- `adze` (runtime)
- `adze-macro` (proc-macro)
- `adze-tool` (build-time codegen)
- `adze-common` (shared grammar expansion)
- `adze-ir` (grammar IR)
- `adze-glr-core` (GLR parser generation)
- `adze-tablegen` (table compression, FFI generation)

### CI Workflows

| Workflow | Purpose | Trigger |
|----------|---------|---------|
| [`ci.yml`](../../.github/workflows/ci.yml) | Main CI with `ci-supported` job | Push, PR |
| [`pure-rust-ci.yml`](../../.github/workflows/pure-rust-ci.yml) | Pure-Rust implementation | Push, PR |
| [`core-tests.yml`](../../.github/workflows/core-tests.yml) | Core crate testing | Push, PR |
| [`golden-tests.yml`](../../.github/workflows/golden-tests.yml) | Tree-sitter parity | Push, PR |
| [`fuzz.yml`](../../.github/workflows/fuzz.yml) | Fuzz testing | Schedule |
| [`benchmarks.yml`](../../.github/workflows/benchmarks.yml) | Performance benchmarks | Push |

### CI Environment Variables

Configured in [`ci.yml`](../../.github/workflows/ci.yml):
```yaml
env:
  RUST_BACKTRACE: 1
  RUST_TEST_THREADS: 2
  RAYON_NUM_THREADS: 4
  TOKIO_WORKER_THREADS: 2
  CARGO_BUILD_JOBS: 4
```

### Known CI Exclusions

See [`docs/status/KNOWN_RED.md`](../status/KNOWN_RED.md) for intentional exclusions from CI.

### Pre-commit Hooks

Install via:
```bash
.githooks/install.sh
```

Runs:
```bash
just pre           # Standard pre-commit
just pre-tests     # With test clippy enabled
just pre-docs      # With strict docs enabled
```

---

## Quick Reference

### Commands

| Command | Purpose |
|---------|---------|
| `just test` | Core lib tests |
| `just ci-supported` | PR gate (required) |
| `just snap` | Review snapshots |
| `just clippy` | Lint core crates |
| `just fmt` | Check formatting |
| `just mutate` | Mutation test adze-ir |
| `cargo test -p <crate>` | Test specific crate |
| `cargo test <name>` | Run specific test |

### Test File Patterns

| Pattern | Type |
|---------|------|
| `tests/*.rs` | Integration tests |
| `tests/proptest_*.rs` | Property tests |
| `tests/snapshot_*.rs` | Snapshot tests |
| `tests/*_comprehensive.rs` | Comprehensive test suites |
| `**/tests/**/*.snap` | Insta snapshots |
| `fuzz_targets/*.rs` | Fuzz targets |

### Key Files

| File | Purpose |
|------|---------|
| [`justfile`](../../justfile) | Development recipes |
| [`testing/src/lib.rs`](../../testing/src/lib.rs) | Test framework |
| [`testing/src/strategies.rs`](../../testing/src/strategies.rs) | Proptest strategies |
| [`glr-test-support/src/lib.rs`](../../glr-test-support/src/lib.rs) | GLR test helpers |
| [`docs/archive/plans/BDD_GLR_CONFLICT_PRESERVATION.md`](../archive/plans/BDD_GLR_CONFLICT_PRESERVATION.md) | BDD scenarios |

### Verification Checklist

Before submitting any changes:

```bash
# 1. Format
cargo fmt --all

# 2. Lint
just clippy

# 3. Test
just test

# 4. Full PR gate
just ci-supported

# 5. If snapshots changed
cargo insta review
```

If all pass, the PR is ready for review.
