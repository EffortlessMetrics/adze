# API Freeze Implementation Summary

## ✅ Completed API Lockdown

The adze API is now frozen with comprehensive safeguards against accidental breaking changes.

### What's Been Implemented

#### 1. Baseline Tag Created
- **Tag**: `v0.8.0-dev.api-freeze-1`
- **Purpose**: Fixed reference point for semver checks
- **Usage**: All CI semver checks compare against this tag

#### 2. Compiler & MSRV Pinned
- **MSRV**: 1.89
- **Edition**: 2024
- **Lints**: Added `unused_extern_crates = "deny"`
- **Location**: `/Cargo.toml` workspace configuration

#### 3. Re-export Surface Stabilized
- **File**: `runtime/src/lib.rs`
- **Stable exports**:
  - `TSSymbol` and `SymbolId`
  - `glr_incremental::{Edit, GLRToken, IncrementalGLRParser}` (when `pure-rust` enabled)
- **Guarantee**: These re-exports won't move or be removed in minor versions

#### 4. Sealed Trait Pattern
- **Trait**: `Extract<Output>`
- **Implementation**: Sealed with private `sealed::Sealed` supertrait
- **Benefit**: Can add new methods without breaking changes
- **Note**: Blanket impl allows macro-generated code to work

#### 5. CI Enforcement
- **Semver checks**: Compare against baseline tag, not branch
- **Public API diff**: Non-blocking visibility into API changes
- **MSRV test**: Ensures 1.89 compatibility

#### 6. Developer Documentation
- **File**: `DEVELOPER_GUIDE.md`
- **Contents**: Command cheat sheet, API change process, common issues

### How to Work With the Frozen API

#### Making Intentional Breaking Changes

1. **Document the change** in CHANGELOG.md
2. **Bump version** (pre-1.0: minor bump = breaking)
3. **Update baseline tag** after release:
   ```bash
   git tag -f v0.8.0-dev.api-freeze-1 <new-commit>
   git push --tags --force
   ```

#### Adding New Features (Non-Breaking)

1. **Add new API** alongside existing
2. **Mark old API** as `#[deprecated]` if replacing
3. **Document migration** path clearly
4. **No version bump** needed (just patch)

#### Checking for Breaking Changes Locally

```bash
# Install tool if needed
cargo install cargo-semver-checks

# Check against baseline
cargo semver-checks check-release \
  -p adze \
  --baseline-rev v0.8.0-dev.api-freeze-1
```

### What This Prevents

- ❌ Accidental removal of public types/functions
- ❌ Unintentional changes to type signatures
- ❌ Moving re-exports to different paths
- ❌ Breaking macro-generated code
- ❌ Silent API drift over time

### What This Allows

- ✅ Adding new features additively
- ✅ Deprecating old APIs with migration paths
- ✅ Internal refactoring without API changes
- ✅ Intentional breaking changes with proper versioning

### Next Steps When Ready to Release

1. Review all changes since baseline
2. Decide on version bump (patch/minor/major)
3. Update CHANGELOG.md
4. Tag the release
5. Move baseline tag if major/minor bump
6. Publish to crates.io

### Current Status

The API is now locked at the `v0.8.0-dev.api-freeze-1` baseline. Any changes that break this contract will be caught by CI and require explicit acknowledgment through version bumping and baseline tag updates.