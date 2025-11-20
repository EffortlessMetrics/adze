# CI Nix Migration Plan

**Version**: 1.0.0
**Date**: 2025-11-20
**Status**: ACTIVE
**Parent Contract**: [NIX_CI_INTEGRATION_CONTRACT.md](./NIX_CI_INTEGRATION_CONTRACT.md)

---

## Executive Summary

**Goal**: Migrate core CI jobs in `.github/workflows/ci.yml` to use Nix, achieving "dev environment = CI environment" while maintaining all existing CI coverage.

**Current State**:
- ✅ `nix-ci.yml` exists with 6 jobs (showcase workflow)
- ⚠️ `ci.yml` has 25 jobs using traditional toolchain
- ❌ Duplicate CI logic violates Single Source of Truth

**Target State**:
- ✅ Core jobs in `ci.yml` use Nix (`nix develop --command`)
- ✅ Specialized jobs stay traditional (miri, sanitizers need nightly)
- ✅ `nix-ci.yml` either merged or deprecated

---

## I. Job Classification

### Core Jobs → Migrate to Nix ✅

**These benefit from Nix reproducibility and should match local development:**

1. **lint** - Formatting and clippy checks
   - Current: `cargo fmt`, `cargo clippy`, custom scripts
   - Target: `nix develop --command just ci-fmt`, `just ci-clippy`
   - Benefit: Same lints locally and in CI

2. **test** - Main test matrix (os × features × toolchain)
   - Current: `cargo nextest run --workspace`
   - Target: `nix develop --command cargo test --workspace`
   - Benefit: Reproducible test environments

3. **matrix-smoke** - Workspace smoke tests
   - Current: `cargo test --workspace --all-features`
   - Target: `nix develop --command cargo test --workspace --all-features`
   - Benefit: Guaranteed dependency consistency

4. **docs** - Documentation build
   - Current: `cargo doc --workspace --no-deps --all-features`
   - Target: `nix develop --command just ci-doc`
   - Benefit: Same rustdoc version locally/CI

5. **test-release** - Release mode tests
   - Current: `cargo test --release`
   - Target: `nix develop --command cargo test --release`
   - Benefit: Consistent release builds

6. **benchmarks** - Benchmark compilation
   - Current: `cargo bench --no-run`
   - Target: `nix develop .#perf --command cargo bench`
   - Benefit: Performance shell consistency

### Specialized Jobs → Keep Traditional ❌

**These need specific toolchains that Nix doesn't benefit:**

7. **miri** - UB detection (requires nightly + miri component)
8. **sanitizers** - ASAN/UBSAN (requires nightly + sanitizers)
9. **fuzz** - Fuzzing (requires nightly + cargo-fuzz)
10. **minimal-versions** - Min dependency check (requires nightly -Z flag)
11. **msrv** - MSRV check (requires specific Rust 1.89.0)
12. **semver-checks** - API breaking change detection
13. **cross-compile** - Cross-platform targets
14. **coverage** - Code coverage (cargo-llvm-cov)
15. **unsafe-audit** - Unsafe code audit (cargo-geiger)

### Utility Jobs → Keep As-Is ℹ️

16. **deterministic-codegen** - Reproducibility check
17. **feature-matrix** - Feature powerset (cargo-hack)
18. **test-connectivity** - Test discovery tripwires
19. **publish-dry-run** - Publishability check
20. **api-stability** - API diff tracking
21. **security** - Security audit (cargo-deny)
22. **backend-matrix** - Pure-rust vs c-backend
23. **test-debug-assertions** - Debug assertion tests
24. **ts-compat** - Tree-sitter compatibility
25. **benches-unstable** - Unstable benchmarks (opt-in)

---

## II. Implementation Plan

### Step 1: Update `lint` Job to Use Nix

**Current `lint` job** (lines 28-81 of ci.yml):
```yaml
lint:
  name: Lint
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v4
    - uses: dtolnay/rust-toolchain@stable  # ← Remove this
      with:
        components: clippy, rustfmt
    - uses: Swatinem/rust-cache@v2

    - name: Check formatting
      run: cargo fmt --all -- --check

    - name: Clippy (default features, per-package)
      run: scripts/clippy-per-package.sh default
```

**New `lint` job**:
```yaml
lint:
  name: Lint (Nix)
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v4

    - name: Install Nix
      uses: cachix/install-nix-action@v27
      with:
        nix_path: nixpkgs=channel:nixos-24.05
        extra_nix_config: |
          experimental-features = nix-command flakes

    - name: Check formatting
      run: nix develop --command just ci-fmt

    - name: Run clippy
      run: nix develop --command just ci-clippy

    # Keep existing custom script checks
    - name: Check for bare #[no_mangle]
      run: nix develop --command scripts/check-no-mangle.sh

    - name: Check commented debug blocks
      run: nix develop --command python3 tools/check_debug_blocks.py

    - name: Clippy - collect per-package outputs
      run: nix develop --command ./scripts/clippy-collect.sh || true

    - name: Upload Clippy reports
      uses: actions/upload-artifact@v4
      with:
        name: clippy-report
        path: clippy-report
```

**Changes**:
- ✅ Remove `dtolnay/rust-toolchain@stable`
- ✅ Add `cachix/install-nix-action@v27`
- ✅ Use `nix develop --command just ci-fmt` and `just ci-clippy`
- ✅ Wrap custom scripts in `nix develop --command`
- ❌ Remove environment variables (come from flake.nix)

---

### Step 2: Update `test` Matrix Job to Use Nix

**Current `test` job** (lines 83-156):
```yaml
test:
  name: Test (${{ matrix.os }} / ${{ matrix.features }})
  strategy:
    matrix:
      os: [ubuntu-latest, macos-latest, windows-latest]
      features: ["", "--features external_scanners", ...]
      toolchain: [stable, beta]
  runs-on: ${{ matrix.os }}
  steps:
    - uses: dtolnay/rust-toolchain@${{ matrix.toolchain }}  # ← Remove
    - run: cargo nextest run --workspace ${{ matrix.features }}
```

**New `test` job**:
```yaml
test:
  name: Test (Nix: ${{ matrix.os }} / ${{ matrix.features }})
  strategy:
    matrix:
      os: [ubuntu-latest, macos-latest]  # Remove windows (Nix not supported natively)
      features: ["", "--features external_scanners", "--features incremental_glr"]
      toolchain: [stable]  # Remove beta (controlled by flake.nix)
  runs-on: ${{ matrix.os }}
  steps:
    - uses: actions/checkout@v4

    - name: Install Nix
      uses: cachix/install-nix-action@v27
      with:
        nix_path: nixpkgs=channel:nixos-24.05
        extra_nix_config: |
          experimental-features = nix-command flakes

    - name: Run tests with nextest
      run: nix develop --command cargo nextest run --workspace ${{ matrix.features }}

    - name: Run doctests
      run: nix develop --command cargo test --doc --workspace ${{ matrix.features }}
```

**Changes**:
- ✅ Remove `dtolnay/rust-toolchain`
- ✅ Add Nix installation
- ✅ Wrap cargo commands in `nix develop --command`
- ⚠️ Drop Windows (Nix doesn't support Windows natively, only WSL)
- ⚠️ Drop beta toolchain (Nix flake controls Rust version)

---

### Step 3: Add Windows-Specific Non-Nix Test Job

Since Nix doesn't support Windows natively, keep a minimal Windows test job using traditional toolchain:

```yaml
test-windows:
  name: Test (Windows, non-Nix)
  runs-on: windows-latest
  steps:
    - uses: actions/checkout@v4
    - uses: dtolnay/rust-toolchain@stable
    - uses: Swatinem/rust-cache@v2

    - name: Run tests
      run: cargo test --workspace -- --test-threads=2

    - name: Note about Nix
      run: |
        echo "::notice::Windows CI uses traditional toolchain (Nix not supported)"
        echo "For local Windows development, use WSL + Nix"
```

---

### Step 4: Update `docs` Job

**Current**:
```yaml
docs:
  runs-on: ubuntu-latest
  steps:
    - uses: dtolnay/rust-toolchain@stable
    - run: cargo doc --workspace --no-deps --all-features
```

**New**:
```yaml
docs:
  name: Documentation (Nix)
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v4

    - name: Install Nix
      uses: cachix/install-nix-action@v27
      with:
        nix_path: nixpkgs=channel:nixos-24.05
        extra_nix_config: |
          experimental-features = nix-command flakes

    - name: Build documentation
      run: nix develop --command just ci-doc
```

---

### Step 5: Update Environment Variables

**Current** (lines 12-25):
```yaml
env:
  RUST_BACKTRACE: 1
  CARGO_TERM_COLOR: always
  COVERAGE_THRESHOLD: 80
  RUST_TEST_THREADS: 2
  RAYON_NUM_THREADS: 4
  # ... many more
```

**New** (remove all, they come from flake.nix):
```yaml
env:
  # Environment variables are set by the Nix devShell (see flake.nix)
  # This ensures local development matches CI exactly
  CARGO_TERM_COLOR: always  # Keep for visibility
  COVERAGE_THRESHOLD: 80    # Keep (not in flake.nix)
```

---

## III. Testing Strategy

### Local Testing Before Pushing

```bash
# Test lint job locally
nix develop --command just ci-fmt
nix develop --command just ci-clippy

# Test test job locally
nix develop --command cargo nextest run --workspace
nix develop --command cargo test --doc --workspace

# Test docs job locally
nix develop --command just ci-doc

# Test feature matrix
for feature in "" "--features external_scanners" "--features incremental_glr"; do
  nix develop --command cargo test --workspace $feature -- --test-threads=2
done
```

### CI Testing Strategy

1. **Create feature branch**: `nix-ci-migration-phase1`
2. **Update jobs incrementally**:
   - Commit 1: Update `lint` job only
   - Commit 2: Update `test` job only
   - Commit 3: Update `docs` job only
3. **Open PR and verify**:
   - Check that migrated jobs pass
   - Verify specialized jobs still work
   - Compare performance (should be similar or better)
4. **Merge when all tests pass**

---

## IV. Rollback Plan

If Nix CI fails:

1. **Immediate**: Revert the PR
2. **Debug**: Test locally with `nix develop`
3. **Fix**: Address issues in separate branch
4. **Retry**: Re-apply migration when fixed

**Rollback commands**:
```bash
git revert <commit-hash>
git push -f origin <branch>
```

---

## V. Success Criteria

This migration is **COMPLETE** when:

1. ✅ `lint` job uses Nix and passes
2. ✅ `test` matrix uses Nix and passes on Ubuntu + macOS
3. ✅ `docs` job uses Nix and passes
4. ✅ Windows test job exists (non-Nix fallback)
5. ✅ All specialized jobs still work
6. ✅ Local `nix develop --command just ci-all` matches CI results
7. ✅ PR with migration merged to main

---

## VI. Follow-Up Tasks (Phase 2)

After Phase 1 completes:

1. Evaluate `nix-ci.yml`:
   - Option A: Deprecate (functionality now in main ci.yml)
   - Option B: Keep as showcase/demo workflow
   - Option C: Use for nightly/experimental features

2. Migrate more jobs to Nix:
   - `matrix-smoke` → `nix develop --command cargo test --all-features`
   - `test-release` → `nix develop --command cargo test --release`
   - `benchmarks` → `nix develop .#perf --command cargo bench`

3. Update documentation:
   - Update ADR-0008 status to COMPLETE
   - Update NIX_CI_INTEGRATION_CONTRACT.md status
   - Update CLAUDE.md with CI migration notes

---

## VII. Risk Assessment

| Risk | Impact | Likelihood | Mitigation |
|------|--------|------------|------------|
| macOS Nix installation slow | MEDIUM | HIGH | Use cachix for binary cache |
| Windows lack of Nix support | MEDIUM | CERTAIN | Keep separate Windows job |
| Flake.nix changes break CI | HIGH | LOW | Test locally first, rollback plan |
| Performance overhead from Nix | LOW | LOW | Benchmark before/after |

---

## VIII. References

- [NIX_CI_INTEGRATION_CONTRACT.md](./NIX_CI_INTEGRATION_CONTRACT.md) - Parent contract
- [ADR-0008-NIX-DEVELOPMENT-ENVIRONMENT.md](../adr/ADR-0008-NIX-DEVELOPMENT-ENVIRONMENT.md) - Nix rationale
- [flake.nix](/flake.nix) - Dev environment definition
- [justfile](/justfile) - CI commands
- [.github/workflows/nix-ci.yml](../../.github/workflows/nix-ci.yml) - Existing Nix CI

---

**Plan Version**: 1.0.0
**Last Updated**: 2025-11-20
**Next Review**: After Phase 1 completion

---

END OF MIGRATION PLAN
