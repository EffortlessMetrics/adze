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
9. **Coverage** - Tracks test coverage (main branch only)
10. **Unsafe Audit** - Reports unsafe code usage

## Required GitHub Settings

To make the CI effective, configure these branch protection rules:

### Required Status Checks

Required status checks are currently disabled pending stabilization of the full
test suite. The `ci-supported` workflow (`just ci-supported`) is the intended
stable gate for PRs. See `docs/status/KNOWN_RED.md` for details on what is
excluded and why.

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
