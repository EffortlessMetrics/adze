# CI Infrastructure for rust-sitter

This document describes the comprehensive CI setup for rust-sitter.

## Overview

The CI pipeline ensures code quality, API stability, and security through multiple automated checks:

### Jobs

1. **Lint** - Enforces code formatting and clippy warnings
2. **Test** - Runs all tests using cargo-nextest for speed
3. **Feature Matrix** - Tests all feature combinations
4. **MSRV** - Ensures compatibility with Rust 1.75.0
5. **API Stability** - Detects breaking changes in public APIs
6. **Security** - Scans for vulnerabilities and license issues
7. **Documentation** - Builds docs with warnings as errors
8. **Coverage** - Tracks test coverage (main branch only)
9. **Unsafe Audit** - Reports unsafe code usage

## Required GitHub Settings

To make the CI effective, configure these branch protection rules:

### Required Status Checks
- `lint`
- `test`
- `msrv`
- `security`
- `docs`

### Recommended Settings
- Require branches to be up to date before merging
- Include administrators
- Require conversation resolution

## Local Development

### Running CI Locally

```bash
# Install required tools
cargo install cargo-nextest cargo-hack cargo-deny cargo-llvm-cov

# Run the full CI suite locally
cargo ci           # Clippy with warnings as errors
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