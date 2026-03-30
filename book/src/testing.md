# Testing

This page gives a practical overview of running and writing tests in the Adze workspace. For the full testing strategy see the [Development Testing Guide](development/testing.md).

## Running the test suite

```bash
# All workspace tests (recommended: use capped concurrency)
cargo test

# Concurrency-capped variants (more stable on CI or constrained machines)
cargo t2                  # 2 test threads
cargo test-safe           # safe defaults
cargo test-ultra-safe     # single-threaded
./scripts/test-capped.sh  # auto-detect caps
```

### Per-crate tests

```bash
cargo test -p adze            # runtime
cargo test -p adze-macro       # proc-macro
cargo test -p adze-ir          # grammar IR
cargo test -p adze-glr-core    # GLR analysis (use --features test-api for internal helpers)
cargo test -p adze-tablegen    # table compression
cargo test -p adze-tool        # build tool
cargo test -p adze-runtime2    # GLR runtime
```

### Feature combinations

Some crates behave differently depending on feature flags:

```bash
cargo test -p adze --features glr
cargo test -p adze --features incremental_glr
cargo test -p adze --features all-features
```

## Golden tests

Golden tests verify Adze parsers produce byte-for-byte identical parse trees to the official Tree-sitter parsers.

```bash
cd golden-tests

# Generate reference S-expressions and SHA256 hashes (one-time)
./generate_references.sh

# Run all golden tests
cargo test --features all-grammars

# Run for a single language
cargo test --features python-grammar
cargo test --features javascript-grammar

# Update references after intentional parser changes
UPDATE_GOLDEN=1 cargo test --features python-grammar
```

See [Golden Tests Maintenance](guide/golden-tests-maintenance.md) for the full workflow.

## Snapshot tests (insta)

Example grammars use [insta](https://insta.rs) for snapshot testing:

```bash
cargo test -p example --features c-backend   # or --features pure-rust
cargo insta review                            # interactive diff review
```

When grammar output changes intentionally, review and accept the new snapshots.

## Writing a grammar test

The simplest pattern is to parse a string and assert against the typed AST:

```rust
#[cfg(test)]
mod tests {
    use super::grammar;

    #[test]
    fn addition() {
        let ast = grammar::parse("1 + 2").unwrap();
        assert_eq!(
            ast,
            grammar::Expression::Add(
                Box::new(grammar::Expression::Number(1)),
                (),
                Box::new(grammar::Expression::Number(2)),
            )
        );
    }

    #[test]
    fn precedence() {
        // Multiplication binds tighter than addition
        let ast = grammar::parse("1 + 2 * 3").unwrap();
        match ast {
            grammar::Expression::Add(_, _, rhs) => {
                assert!(matches!(*rhs, grammar::Expression::Mul(..)));
            }
            other => panic!("unexpected: {other:?}"),
        }
    }

    #[test]
    fn rejects_invalid_input() {
        assert!(grammar::parse("1 + + 2").is_err());
    }
}
```

## BDD framework

Adze includes a BDD (Behavior-Driven Development) framework in the `crates/bdd-*` family for tracking feature scenarios and governance:

```rust
use adze_bdd_contract::{BddScenario, BddScenarioStatus, BddPhase};

// Scenarios are defined declaratively
let scenario = BddScenario {
    name: "GLR conflict preservation",
    phase: BddPhase::Given,
    status: BddScenarioStatus::Passing,
    // ...
};
```

The governance crates (`crates/governance-*`) use BDD grids to track which features pass across different runtime configurations.

Run BDD-related tests:

```bash
cargo test -p bdd-grammar-analysis-core
cargo test -p bdd-governance-core
cargo test -p bdd-grid-core
```

## Test connectivity safeguards

Several layers prevent tests from being silently disconnected:

1. **CI job** — blocks `.rs.disabled` files and enforces non-zero test counts per crate.
2. **Pre-commit hook** — warns about disabled test files.
3. **Local check** — `./scripts/check-test-connectivity.sh` reports per-crate counts and orphans.

## Concurrency tips

| Variable | Default | Purpose |
|---|---|---|
| `RUST_TEST_THREADS` | 2 | Rust test parallelism |
| `RAYON_NUM_THREADS` | 4 | Rayon pool size |
| `TOKIO_WORKER_THREADS` | 2 | Tokio async workers |
| `CARGO_BUILD_JOBS` | 4 | Cargo compile jobs |

Lower these if tests fail with "Too many open files" or thread-creation errors. The `./scripts/preflight.sh` script auto-detects safe values.

## Further reading

- [Development Testing Guide](development/testing.md) — exhaustive testing strategy
- [Golden Tests Guide](development/golden-tests.md) — golden test internals
- [Performance Optimization](guide/performance.md) — benchmarking and profiling
