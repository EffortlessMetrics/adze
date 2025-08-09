# Post-Merge Hardening for rust-sitter v0.6.0

## Summary
This PR implements critical CI hardening and test improvements following the GLR vector resolution merge. These changes ensure the codebase maintains quality standards and prepares for the v0.6.0 release.

## Changes

### 1. CI Guardrails ✅
- Added `check-no-mangle.sh` script to enforce Rust 2024 compatibility
- Integrated script into CI lint job to prevent regression
- Added opt-in experimental examples testing with feature flag

### 2. Test Infrastructure ✅
- Updated incremental parsing tests to use `cfg_attr` for feature-based gating
- Tests now automatically run when `incremental_glr` feature is enabled
- Maintained backward compatibility for default builds

### 3. FFI Generation Fix ✅
- Fixed `test_pure_rust_parser.rs` example to use correct `LANGUAGE` constant
- Verified clean build compiles successfully with all targets

### 4. Project Documentation ✅
- Created `TRACKING_ISSUES.md` with prioritized task list
- Added issue templates for systematic tracking
- Documented known limitations and future work

## Testing
```bash
# Verify CI checks pass
cargo fmt --all -- --check
scripts/check-no-mangle.sh
cargo clippy --workspace --all-targets --all-features -- -D warnings

# Test with incremental feature
cargo test --workspace --features incremental_glr

# Verify examples build
cargo build -p rust-sitter-example --examples
```

## Release Readiness
With these changes, the codebase is ready for v0.6.0 release:
- ✅ No bare `#[no_mangle]` attributes
- ✅ Feature-gated tests properly configured
- ✅ CI enforces quality standards
- ✅ Known issues documented for post-release work

## Next Steps
1. Merge this PR to `main`
2. Run release workflow with v0.6.0
3. Open tracking issues from `TRACKING_ISSUES.md`
4. Begin work on v0.6.1 improvements (Parser::reparse implementation)