# CI Infrastructure for adze

This document describes the comprehensive CI setup for adze.

## Overview

The CI pipeline ensures code quality, API stability, and security through multiple automated checks:

### Jobs

1. **ci-supported** - Stable green gate (`just ci-supported`); runs on every push/PR
2. **Lint** - Enforces code formatting and clippy warnings
3. **Test** - Runs all tests using cargo-nextest for speed
4. **Feature Matrix** - Tests all feature combinations
5. **MSRV** - Ensures compatibility with Rust 1.92.0
6. **API Stability** - Detects breaking changes in public APIs
7. **Security** - Scans for vulnerabilities and license issues
8. **Documentation** - Builds docs with warnings as errors
9. **Coverage** - Generates coverage on non-PR CI runs and uploads report on main branch pushes
10. **Unsafe Audit** - Reports unsafe code usage

## Manual CI triggers

The `CI` workflow supports manual dispatch with two toggles:

- `run_full_ci` (workflow_dispatch only): Run the full non-PR lane in addition to PR-required lanes.
- `run_ci_supported_examples` (workflow_dispatch only): Enable experimental examples in `feature-matrix`.
  If `run_full_ci` is false, this is the only non-PR lane that runs on manual dispatch.
  Outside manual dispatch, experimental examples in `feature-matrix` only run when commit message includes `[test-examples]`.

## Required GitHub Settings

To make the CI effective, configure these branch protection rules:

### Required Status Checks

Required status checks are intentionally single-gated.
Set branch protection to require only: `CI / ci-supported`.
Everything else is optional signal (nightly/manual/canary).

### Recommended Settings
- Require branches to be up to date before merging
- Require conversation resolution

## Local Development

### Running CI Locally

```bash
# Install required tools
cargo install cargo-nextest cargo-hack cargo-deny cargo-llvm-cov

# Run the full CI suite locally
just ci-supported  # Stable green CI lane
cargo nextest run  # Fast parallel test runner
cargo deny check   # Security and license checks

# Test feature combinations
cargo hack test --feature-powerset --skip tree-sitter-standard
```

### Snapshot Testing

We use `insta` for snapshot testing of generated code:

```bash
# Review snapshot changes
cargo insta review

# Update snapshots in CI
INSTA_UPDATE=auto cargo test
```

### Fuzzing

Fuzz testing targets are in the `fuzz/` directory:

```bash
# Install cargo-fuzz
cargo install cargo-fuzz

# Run fuzzer (requires nightly)
cd fuzz
cargo +nightly fuzz run fuzz_lexer
cargo +nightly fuzz run fuzz_parser
cargo +nightly fuzz run fuzz_external_scanner
```

## Testing Strategies

### Contract Tests
- **Snapshot tests** - Ensure stable output format
- **Compile-fail tests** - Verify error messages
- **Property tests** - Check parser invariants

### Performance Regression
- Benchmarks run on main branch commits
- 10% regression threshold triggers alerts

### API Stability
- `cargo-public-api` - Detects API changes
- `cargo-semver-checks` - Validates semantic versioning

## Security

### Supply Chain Security
- `cargo-deny` checks for:
  - Security advisories
  - License compatibility
  - Banned dependencies
  - Duplicate dependencies

### Unsafe Code
- `cargo-geiger` reports unsafe usage
- Summary posted to PR comments

## Coverage

Code coverage is generated using `cargo-llvm-cov` and uploaded to codecov.io.

To generate coverage locally:
```bash
cargo llvm-cov --workspace --all-features --lcov --output-path lcov.info
```

## Maintenance

### Updating Dependencies
```bash
cargo update
cargo deny check
```

### Updating MSRV
1. Update `rust-version` in `Cargo.toml`
2. Update `.github/workflows/ci.yml` MSRV job
3. Test with: `cargo +1.XX.0 build --workspace`

## Troubleshooting

### Common Issues

**Snapshot test failures**
- Review with `cargo insta review`
- Accept changes if intentional

**Feature combination failures**
- Use `cargo hack` to test locally
- Consider adding feature gates

**API breaking changes**
- Run `cargo semver-checks` before PR
- Document breaking changes in CHANGELOG

**Fuzzing crashes**
- Minimize with `cargo fuzz tmin`
- Add regression test case
