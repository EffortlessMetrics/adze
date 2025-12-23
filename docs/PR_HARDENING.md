# Post-Merge Hardening for rust-sitter v0.6.0

## Summary

This PR implements critical CI hardening and test improvements following the GLR vector resolution merge. These changes ensure the codebase maintains quality standards and prepares for the v0.6.0 release.

## Changes

### 1. CI Guardrails ✅
- Added `check-no-mangle.sh` script to enforce Rust 2024 compatibility
- Integrated script into CI lint job to prevent regression
- Verified experimental examples are properly feature-gated

### 2. Incremental Test Gating ✅
- Updated all incremental parsing tests with `#[cfg_attr(not(feature = "incremental_glr"), ignore)]`
- Tests now automatically run when the feature is enabled
- Prevents false failures in default CI runs

### 3. FFI Generation Verification ✅
- Verified pure-Rust FFI generation works with clean builds
- Confirmed `RUST_SITTER_EMIT_ARTIFACTS=true` produces expected outputs
- No compilation errors in experimental examples

### 4. Tracking Issues Documentation ✅
- Created comprehensive `TRACKING_ISSUES.md` with prioritized work items
- Documented 8 tracking issues across high/medium/low priority
- Added issue template for consistent tracking

## Files Changed

- `runtime/tests/incremental_glr_comprehensive_test.rs` - Added feature gating
- `runtime/tests/test_incremental_simple.rs` - Added feature gating  
- `runtime/tests/incremental_reuse_test.rs` - Added feature gating
- `TRACKING_ISSUES.md` - New file documenting known issues and roadmap

## Testing

- [x] CI passes with all checks
- [x] Incremental tests are properly ignored without feature flag
- [x] Pure-Rust FFI generation completes successfully
- [x] No new warnings introduced

## Next Steps

After merging this PR:
1. Merge to `main`
2. Tag v0.6.0 release
3. Publish crates to crates.io
4. Create GitHub release with changelog

The branch is now ready for the v0.6.0 release with proper safeguards in place.