# Clippy Quarantine Follow-up Issues

The following crates are currently in the clippy quarantine and need to be cleaned up:

## Core Crates (High Priority)
- [x] adze - Main runtime crate ✅ (Fixed 2025-01-24)
- [x] adze-tool - Build tool ✅ (Fixed 2025-01-24)
- [x] adze-tablegen - Table generation ✅ (Fixed 2025-01-24)
- [x] adze-glr-core - GLR parser core ✅ (Fixed 2025-01-24)
- [ ] adze-runtime - Runtime implementation

## Test/Example Crates (Medium Priority)
- [ ] adze-testing - Testing utilities
- [ ] glr-test-support - GLR test support
- [ ] test-mini - Minimal test crate
- [ ] test-vec-wrapper - Vector wrapper tests
- [ ] adze-python-simple - Simple Python grammar test
- [ ] adze-python-simpletest-vec-wrapper - Python vec wrapper test

## Language Implementations (Lower Priority)
- [ ] adze-go - Go grammar
- [ ] adze-javascript - JavaScript grammar
- [ ] adze-python - Python grammar

## Tools/Apps (Lower Priority)
- [ ] adze-benchmarks - Benchmarking suite
- [ ] adze-cli - Command-line interface
- [ ] adze-playground - Playground application

## How to Fix

For each crate:
1. Run `cargo clippy -p <crate-name> --all-targets --no-deps -- -D warnings`
2. Fix all warnings
3. Remove the crate from `.clippy-quarantine`
4. Run `./scripts/clippy-per-package.sh default` to verify
5. Commit with message: `chore: remove <crate-name> from clippy quarantine`
