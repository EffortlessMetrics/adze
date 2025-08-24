# Clippy Quarantine Follow-up Issues

The following crates are currently in the clippy quarantine and need to be cleaned up:

## Core Crates (High Priority)
- [x] rust-sitter - Main runtime crate ✅ (Fixed 2025-01-24)
- [x] rust-sitter-tool - Build tool ✅ (Fixed 2025-01-24)
- [x] rust-sitter-tablegen - Table generation ✅ (Fixed 2025-01-24)
- [x] rust-sitter-glr-core - GLR parser core ✅ (Fixed 2025-01-24)
- [ ] rust-sitter-runtime - Runtime implementation

## Test/Example Crates (Medium Priority)
- [ ] rust-sitter-testing - Testing utilities
- [ ] glr-test-support - GLR test support
- [ ] test-mini - Minimal test crate
- [ ] test-vec-wrapper - Vector wrapper tests
- [ ] rust-sitter-python-simple - Simple Python grammar test
- [ ] rust-sitter-python-simpletest-vec-wrapper - Python vec wrapper test

## Language Implementations (Lower Priority)
- [ ] rust-sitter-go - Go grammar
- [ ] rust-sitter-javascript - JavaScript grammar
- [ ] rust-sitter-python - Python grammar

## Tools/Apps (Lower Priority)
- [ ] rust-sitter-benchmarks - Benchmarking suite
- [ ] rust-sitter-cli - Command-line interface
- [ ] rust-sitter-playground - Playground application

## How to Fix

For each crate:
1. Run `cargo clippy -p <crate-name> --all-targets --no-deps -- -D warnings`
2. Fix all warnings
3. Remove the crate from `.clippy-quarantine`
4. Run `./scripts/clippy-per-package.sh default` to verify
5. Commit with message: `chore: remove <crate-name> from clippy quarantine`
