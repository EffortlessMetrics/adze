# PR Description - Ready to Copy/Paste

## Title
test(runtime): port GLR integration tests to current API; harden gating; fix scanner wrapper

## Summary

* Port `runtime/tests/test_glr_parsing.rs` to `GLRParser` / Parser v4 APIs.
* Replace magic numbers with local `SYM_*` constants.
* Add deterministic assertions; gate optional bits (alternatives/expected symbols/GSS state) behind `incremental_glr` with inline comments.
* Fix `ExternalScanner` lifecycle: use `Arc<Mutex<dyn ExternalScanner + Send + Sync>>` (comment explains interior mutability for `&mut self` scanners shared across components).
* Clean up `ErrorRecoveryConfig` fields (use `HashSet`, add `scope_delimiters`, consistent names).
* Keep previously hidden tests **discoverable** with `#[ignore = "reason"]` (no `.rs.disabled` files).
* Pre-commit hook in `.githooks/` blocks `.rs.disabled`.

## Test Plan

```bash
# Just this file (noisy logs)
cargo test -p adze --test test_glr_parsing -- --nocapture

# Show tests the harness sees
cargo test -p adze --test test_glr_parsing -- --list --format terse

# Feature matrix for the crate
cargo test -p adze
cargo test -p adze --features external_scanners
cargo test -p adze --features incremental_glr
# (Optional) if query is stubbed/green:
# cargo test -p adze --all-features

# Tripwires
./scripts/check-test-connectivity.sh
rg -n '\.rs\.disabled' || true
```

## Notes for Reviewers

* Scanner wrapper uses `Arc<Mutex<..>>` so we can keep stateful `scan(&mut self, ...)` while sharing a scanner; comment added where it's defined.
* Tests that touch optional GLR APIs are guarded with `#[cfg(feature = "incremental_glr")]` and have a one-line rationale.
* `ParseTable` fixtures in tests fill new fields to stay forward-compatible.

## Changes Detail

### GLR Test Porting
- Updated `test_glr_parsing.rs` to use current APIs (`GLRParser`, `Parser` v4)
- Replaced magic symbol numbers with local constants (`SYM_EOF`, `SYM_NUMBER`, etc.)
- Added proper feature gates for incremental GLR APIs with explanatory comments
- Fixed `ErrorRecoveryConfig` initialization with missing `scope_delimiters` field

### External Scanner Lifecycle Fix
- Changed from `Arc<dyn ExternalScanner>` to `Arc<Mutex<dyn ExternalScanner + Send + Sync>>`
- Added comment explaining interior mutability requirement for stateful scanners
- Fixed test scanner implementation to match new trait signature

### Test Connectivity Improvements
- Re-enabled previously disabled test files (removed `.disabled` suffix)
- Tests now compile and run across all feature combinations
- Pre-commit hook prevents accidentally committing `.disabled` files

## Branch Information
- Branch: `tests/port-glr`
- Base: `main`
- Status: Already pushed to origin

## How to Create the PR
1. Go to: https://github.com/EffortlessSteven/adze/pull/new/tests/port-glr
2. Copy and paste the content above (from Title through Notes for Reviewers)
3. Click "Create pull request"