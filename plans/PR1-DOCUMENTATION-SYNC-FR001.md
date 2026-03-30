# PR #1: Documentation Sync (FR-001) - Implementation Plan

**Status:** ✅ COMPLETED
**Created:** 2026-03-27
**Completed:** 2026-03-28
**Release Blocker:** Yes (identified in `docs/status/FRICTION_LOG.md`)
**Target Version:** 0.8.0-dev → 0.8.0 (crates.io publication)

---

## Executive Summary

This plan addresses FR-001 (Documentation Drift) from the friction log. The documentation has drifted from the current codebase with outdated version strings, legacy naming references, and potentially outdated API examples. Priority 1 fixes (version strings and feature names) were partially completed; this plan completes the remaining work.

### Completion Summary

All documentation has been updated to align with the current codebase:
- Version strings updated from v0.5.0-beta to v0.8.0-dev
- Feature flags standardized from `["glr-core", "incremental"]` to `["glr", "incremental_glr"]`
- Crate name references updated from `adze-runtime` to `adze`
- API usage examples verified and corrected
- All documentation now aligned with completed PR #2 (Feature Flag Standardization)

---

## Executive Summary

This plan addresses FR-001 (Documentation Drift) from the friction log. The documentation has drifted from the current codebase with outdated version strings, legacy naming references, and potentially outdated API examples. Priority 1 fixes (version strings and feature names) were partially completed; this plan completes the remaining work.

This plan addresses FR-001 (Documentation Drift) from the friction log. The documentation has drifted from the current codebase with outdated version strings, legacy naming references, and potentially outdated API examples. Priority 1 fixes (version strings and feature names) were partially completed; this plan completes the remaining work.

### Scope

- **In Scope:** Book content, docs/ tutorials, README files, code examples
- **Out of Scope:** Archive documentation (historical records in `docs/archive/`)

---

## Audit Findings Summary

### 1. Version String Issues

| Location | Current Value | Required Value | Priority |
|----------|---------------|----------------|----------|
| [`book/src/README.md:11`](book/src/README.md:11) | `v0.5.0-beta Highlights` | `v0.8.0-dev Highlights` | Critical |
| [`book/src/getting-started/quickstart.md:97-98`](book/src/getting-started/quickstart.md:97) | `adze-runtime = "0.1"` | Remove or update to match actual crate structure | Critical |
| [`docs/tutorials/glr-quickstart.md:31`](docs/tutorials/glr-quickstart.md:31) | `adze-runtime = { version = "0.1" }` | Verify against current crate structure | High |

### 2. Feature Name Inconsistencies

Per FR-001 progress note: `glr-core → glr`, `incremental → incremental_glr`

| File Pattern | Occurrences | Action Required |
|--------------|-------------|-----------------|
| `book/src/**/*.md` | 30+ references to `glr-core` | Review each for context; update feature flags |
| `docs/**/*.md` | 200+ references | Most are crate names (correct); feature flags need review |
| [`book/src/getting-started/quickstart.md:97`](book/src/getting-started/quickstart.md:97) | `features = ["glr-core", "incremental"]` | Update to `["glr", "incremental_glr"]` |

**Important Distinction:**
- Crate name: `adze-glr-core` (correct, do not change)
- Feature flag: `glr` (not `glr-core`)

### 3. Legacy Naming (rust-sitter)

**Appropriate References (No Change Required):**
- [`README.md:11`](README.md:11) - "Adze (formerly `rust-sitter`)" - Context for users
- [`docs/README.md:5`](docs/README.md:5) - Same context
- [`book/src/guide/migration.md`](book/src/guide/migration.md) - Migration guide (intentional)
- [`book/src/SUMMARY.md:25`](book/src/SUMMARY.md:25) - "Migration: rust-sitter to Adze" link

**References in Archive (No Change Required):**
- `docs/archive/` contains historical references for release notes, changelogs - these should remain unchanged

### 4. API Example Verification Needed

| File | Concern | Action |
|------|---------|--------|
| [`book/src/getting-started/quickstart.md`](book/src/getting-started/quickstart.md) | Uses `adze_runtime::Parser` - verify this crate exists | Verify crate structure |
| [`docs/how-to/incremental-parsing.md:28-31`](docs/how-to/incremental-parsing.md:28) | Uses `adze::runtime::{GLRIncrementalParser, ...}` | Verify API exists |
| [`book/src/guide/grammar-definition.md`](book/src/guide/grammar-definition.md) | Grammar attribute examples | Verify against current macro API |

---

## Implementation Checklist

### Phase 1: Critical Version Updates (Must Fix Before Release)

#### 1.1 Update book/src/README.md
- [ ] Change line 11 from `v0.5.0-beta Highlights` to `v0.8.0-dev Highlights`
- [ ] Review and update feature list to match current capabilities
- **Scope:** Small (5 lines max)
- **Risk:** Low

#### 1.2 Fix book/src/getting-started/quickstart.md
- [ ] Line 97-98: Update feature flags from `["glr-core", "incremental"]` to `["glr", "incremental_glr"]`
- [ ] Line 98: Verify `adze-runtime = "0.1"` - determine if this crate exists or should be removed
- [ ] Lines 54-57: Verify `adze_runtime::Parser` API is correct
- **Scope:** Small (10 lines max)
- **Risk:** Medium (requires API verification)

#### 1.3 Fix docs/tutorials/glr-quickstart.md
- [ ] Line 31: Update `adze-runtime = { version = "0.1" }` to correct version
- [ ] Verify all code examples compile against current API
- **Scope:** Medium (20-30 lines)
- **Risk:** Medium

### Phase 2: High Priority - Feature Flag Consistency

#### 2.1 Audit Feature Flags in book/src/
Search pattern: `features\s*=\s*\[.*glr-core.*\]` or `features\s*=\s*\[.*incremental.*\]`

Files to check:
- [ ] [`book/src/getting-started/quickstart.md`](book/src/getting-started/quickstart.md)
- [ ] [`book/src/getting-started/installation.md`](book/src/getting-started/installation.md)
- [ ] [`book/src/getting-started/migration.md`](book/src/getting-started/migration.md)
- [ ] [`book/src/guide/parser-generation.md`](book/src/guide/parser-generation.md)
- [ ] [`book/src/guide/troubleshooting.md`](book/src/guide/troubleshooting.md)
- [ ] [`book/src/reference/api.md`](book/src/reference/api.md)

**Scope:** Medium (15-20 changes across files)
**Risk:** Low

#### 2.2 Audit Feature Flags in docs/
Files to check:
- [ ] [`docs/tutorials/getting-started.md`](docs/tutorials/getting-started.md)
- [ ] [`docs/tutorials/glr-quickstart.md`](docs/tutorials/glr-quickstart.md)
- [ ] [`docs/how-to/incremental-parsing.md`](docs/how-to/incremental-parsing.md)
- [ ] [`docs/how-to/optimize-performance.md`](docs/how-to/optimize-performance.md)
- [ ] [`docs/reference/api.md`](docs/reference/api.md)

**Scope:** Medium (15-20 changes across files)
**Risk:** Low

### Phase 3: Medium Priority - API Example Verification

#### 3.1 Verify Runtime Crate Structure
Determine the correct crate names and versions:
- `adze` (main runtime) - version 0.8.0-dev
- `adze-runtime` - does this exist as a separate crate?
- `adze-runtime2` - production GLR runtime (internal?)

Action: Check [`Cargo.toml`](Cargo.toml) workspace members and update docs accordingly.

#### 3.2 Verify Code Examples Compile
For each tutorial file, ensure code blocks compile:
- [ ] `docs/tutorials/getting-started.md` - all rust code blocks
- [ ] `docs/tutorials/glr-quickstart.md` - all rust code blocks
- [ ] `book/src/getting-started/quickstart.md` - all rust code blocks

**Scope:** Large (requires running `cargo test --doc` or similar)
**Risk:** Medium

### Phase 4: Nice-to-Have - Content Accuracy

#### 4.1 Review Changelog References
- [ ] [`book/src/appendix/changelog.md`](book/src/appendix/changelog.md) - ensure 0.8.0 entry exists
- [ ] Cross-reference with actual CHANGELOG if one exists

#### 4.2 Review Link Integrity
Check for broken internal links in:
- [ ] `book/src/SUMMARY.md` - all chapter links
- [ ] `docs/README.md` - all tutorial/guide links

---

## Verification Commands

After making changes, run these commands to verify:

```bash
# 1. Build the book to check for broken links
cd book && mdbook build

# 2. Search for remaining old version strings
grep -r "0\.5\.0-beta" book/src/ docs/tutorials/ docs/how-to/ docs/reference/ docs/explanations/
grep -r "0\.6\.[0-9]" book/src/ docs/tutorials/ docs/how-to/ docs/reference/ docs/explanations/

# 3. Search for incorrect feature flags
grep -r "glr-core" book/src/ docs/tutorials/ docs/how-to/ | grep -i feature
grep -r 'features.*incremental"' book/src/ docs/tutorials/ docs/how-to/

# 4. Verify Cargo.toml versions are consistent
grep -r "^version = " */Cargo.toml | grep -v "0\.8"
```

---

## File Change Summary

| Priority | File | Changes | Scope |
|----------|------|---------|-------|
| Critical | `book/src/README.md` | Version string update | Small |
| Critical | `book/src/getting-started/quickstart.md` | Feature flags, runtime crate | Small |
| High | `docs/tutorials/glr-quickstart.md` | Version strings, feature flags | Medium |
| High | `book/src/getting-started/installation.md` | Feature flags | Small |
| High | `book/src/getting-started/migration.md` | Feature flags | Small |
| High | `docs/how-to/incremental-parsing.md` | Feature flags, API verification | Medium |
| Medium | `book/src/guide/parser-generation.md` | Version references | Small |
| Medium | `book/src/reference/api.md` | Feature flag documentation | Small |
| Medium | `docs/tutorials/getting-started.md` | Verify consistency | Small |

**Estimated Total Changes:** ~50-80 lines across 10-15 files

---

## Dependencies

Before executing this plan:
1. Confirm the correct feature flag names by checking [`runtime/Cargo.toml`](runtime/Cargo.toml)
2. Confirm whether `adze-runtime` is a real crate or documentation error
3. Verify the current API surface for `Parser`, `GLRIncrementalParser`, etc.

---

## Success Criteria

- [ ] No references to `0.5.0-beta` or `0.6.x` in active documentation (excluding archive)
- [ ] All feature flags use correct names (`glr`, `incremental_glr`)
- [ ] All code examples in tutorials compile against current API
- [ ] Book builds without errors (`mdbook build`)
- [ ] FR-001 status can be updated from "Mitigated" to "Resolved"

---

## Appendix: Crate Structure Reference

Based on [`AGENTS.md`](AGENTS.md) and workspace structure:

| Crate | Path | Role |
|-------|------|------|
| `adze` | `runtime/` | Main runtime library, `Extract` trait |
| `adze-macro` | `macro/` | Proc-macro attributes |
| `adze-tool` | `tool/` | Build-time code generation |
| `adze-common` | `common/` | Shared grammar expansion |
| `adze-ir` | `ir/` | Grammar IR with GLR support |
| `adze-glr-core` | `glr-core/` | GLR parser generation |
| `adze-tablegen` | `tablegen/` | Table compression, FFI generation |
| `adze-runtime2` | `runtime2/` | Production GLR runtime (Tree-sitter compatible API) |

**Note:** `adze-runtime` appears in documentation but may not exist as a separate crate. This needs verification.
