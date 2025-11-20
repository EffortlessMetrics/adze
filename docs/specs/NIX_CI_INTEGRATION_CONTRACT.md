# Nix CI Integration Contract

**Version**: 1.1.0
**Date**: 2025-11-20 (Updated: 2025-11-20)
**Status**: ⏳ **IN PROGRESS** (AC-1 ✅ COMPLETE, AC-2-5 pending)
**Predecessor**: [ADR-0008: Nix Development Environment](../adr/ADR-0008-NIX-DEVELOPMENT-ENVIRONMENT.md)
**Strategic Context**: [STRATEGIC_IMPLEMENTATION_PLAN.md Phase I](../plans/STRATEGIC_IMPLEMENTATION_PLAN.md)

---

## Executive Summary

**Goal**: Complete Phase I of the Strategic Implementation Plan by integrating Nix into GitHub Actions CI workflows, achieving true Infrastructure-as-Code with `dev environment = CI environment`.

**Current State** (2025-11-20):
- ✅ `flake.nix` created and working locally **COMPLETE**
- ✅ `justfile` with CI commands **COMPLETE**
- ✅ ADR-0008 approved **COMPLETE**
- ✅ **AC-1 COMPLETE**: Core CI jobs migrated to Nix (lint, test, docs, matrix-smoke)
- ⏳ AC-2-5 pending (local reproduction, performance, documentation, backwards compatibility)

**Target State**:
- ✅ All CI workflows use `nix develop --command just ci-*`
- ✅ Local development matches CI exactly
- ✅ Single Source of Truth for environment

**Completion Criteria**: This contract is **COMPLETE** when all acceptance criteria below are met and all CI jobs pass using Nix.

---

## I. Acceptance Criteria

### AC-1: CI Workflows Use Nix

**Status**: ✅ **COMPLETE** (2025-11-20)

**Success Criteria**:
1. ✅ `.github/workflows/ci.yml` uses `cachix/install-nix-action@v27` (5 core jobs migrated)
2. ✅ Core CI jobs run commands via `nix develop --command` (lint, test, docs, matrix-smoke)
3. ✅ Core jobs have no direct use of `dtolnay/rust-toolchain` (specialized jobs still use it as needed)
4. ✅ Environment variables come from `flake.nix` (RUST_TEST_THREADS, RAYON_NUM_THREADS, etc.)
5. ✅ CI tested on Ubuntu + macOS (Windows uses non-Nix fallback, Nix not supported natively)

**BDD Scenario**:
```gherkin
Scenario: CI uses Nix environment
  Given a pull request with code changes
  When GitHub Actions CI runs
  Then Nix is installed via cachix/install-nix-action
  And all commands run inside "nix develop"
  And the Rust toolchain respects rust-toolchain.toml
  And environment variables match flake.nix
  And all tests pass
```

**Test Plan**:
1. Update `.github/workflows/ci.yml` to use Nix
2. Open test PR
3. Verify CI passes on all platforms
4. Verify `just ci-all` command works
5. Compare test results to previous CI runs (should be identical)

---

### AC-2: Local Reproduction Capability

**Status**: PENDING

**Success Criteria**:
1. `nix develop --command just ci-all` runs exact CI suite locally
2. Test pass/fail results match CI exactly
3. Performance benchmarks are reproducible
4. Failure modes are debuggable locally
5. No "works in CI but not locally" scenarios

**BDD Scenario**:
```gherkin
Scenario: Reproduce CI locally
  Given a CI failure on a pull request
  When I run "nix develop --command just ci-all"
  Then I see the exact same failure locally
  And I can debug with identical environment
  And fixing it locally guarantees CI will pass
```

**Test Plan**:
1. Introduce deliberate test failure
2. Run `nix develop --command just ci-test` locally
3. Verify failure matches CI exactly
4. Fix the failure locally
5. Push fix and verify CI passes

---

### AC-3: Performance Baseline Consistency

**Status**: PENDING

**Success Criteria**:
1. Benchmark results consistent across runs (±2% variance)
2. No performance regressions from Nix overhead
3. Performance CI gates still function correctly
4. Flamegraphs and profiling work in Nix shell
5. `just ci-perf` produces stable results

**BDD Scenario**:
```gherkin
Scenario: Consistent performance benchmarks
  Given the Nix CI environment
  When I run "just ci-perf" five times
  Then results vary by less than 2%
  And performance gates trigger on real regressions
  And no false positives from environment variance
```

**Test Plan**:
1. Run `just ci-perf` 5 times in Nix shell
2. Calculate variance in results
3. Verify variance < 2%
4. Introduce intentional performance regression (10%)
5. Verify performance CI catches it

---

### AC-4: Documentation and Onboarding

**Status**: PENDING

**Success Criteria**:
1. CLAUDE.md updated with Nix quickstart
2. CI workflow documentation explains Nix integration
3. Troubleshooting guide for Nix issues
4. Video walkthrough or clear GIF showing setup
5. Contributor guide updated

**BDD Scenario**:
```gherkin
Scenario: New contributor onboarding
  Given a fresh clone of rust-sitter
  When a new contributor reads CLAUDE.md
  Then they can run "nix develop" successfully
  And they can run "just ci-all" successfully
  And they understand why we use Nix
  And they can troubleshoot common issues
```

**Deliverables**:
- [ ] Update `CLAUDE.md` with Nix quickstart section
- [ ] Add `docs/guides/NIX_TROUBLESHOOTING.md`
- [ ] Update `CONTRIBUTING.md` with Nix workflow
- [ ] Add Nix section to FAQ.md

---

### AC-5: Backwards Compatibility

**Status**: PENDING

**Success Criteria**:
1. Traditional setup still documented (legacy)
2. Developers can choose Nix or manual setup
3. CI exclusively uses Nix (enforced)
4. Migration path documented for existing contributors
5. No breakage of existing workflows

**BDD Scenario**:
```gherkin
Scenario: Gradual migration path
  Given a developer using traditional setup
  When they read the migration guide
  Then they can continue with manual setup (legacy)
  Or they can adopt Nix with clear steps
  And existing workflows still function
  And CI results are identical either way
```

**Deliverables**:
- [ ] Mark traditional setup as "legacy" in docs
- [ ] Create migration guide: `docs/guides/MIGRATING_TO_NIX.md`
- [ ] Update CLAUDE.md with both options clearly separated

---

## II. Implementation Plan

### Phase 1: CI Workflow Migration (Week 1, Days 1-3)

**Goal**: Update all GitHub Actions workflows to use Nix

**Tasks**:

1. **Update main CI workflow** (4 hours)
   - File: `.github/workflows/ci.yml`
   - Add `cachix/install-nix-action@v27` step
   - Replace cargo commands with `nix develop --command just ci-*`
   - Remove environment variables (use flake.nix)
   - Test on all platforms

2. **Update performance CI** (2 hours)
   - File: `.github/workflows/performance.yml`
   - Use `nix develop .#perf --command just ci-perf`
   - Verify benchmarks run correctly
   - Test regression detection

3. **Update test connectivity CI** (1 hour)
   - File: `.github/workflows/test-connectivity.yml`
   - Ensure test counting works with Nix
   - Verify feature combinations work

4. **Update other workflows** (2 hours)
   - Security audits, fuzzing, etc.
   - Ensure all use Nix consistently

**Acceptance Test**:
```bash
# Open test PR with CI workflow changes
git checkout -b nix-ci-integration
# ... make changes ...
git push -u origin nix-ci-integration
# Verify all CI jobs pass
# Verify "nix develop --command just ci-all" matches CI results
```

---

### Phase 2: Testing and Validation (Week 1, Days 4-5)

**Goal**: Comprehensive testing across all platforms and scenarios

**Tasks**:

1. **Platform testing** (3 hours)
   - Ubuntu: Verify Nix CI works
   - macOS: Test with Nix on macOS runner
   - Windows: Verify WSL-based Nix approach (or skip if not feasible)

2. **Feature matrix testing** (2 hours)
   - Test all feature combinations
   - Verify `--no-default-features`, `external_scanners`, `incremental_glr`, etc.
   - Ensure Nix doesn't break feature gating

3. **Performance validation** (2 hours)
   - Run benchmarks 5 times
   - Calculate variance
   - Compare to baseline from non-Nix runs
   - Document any differences

4. **Failure mode testing** (2 hours)
   - Introduce deliberate failures
   - Verify errors are clear
   - Test local reproduction
   - Document debugging workflow

**Acceptance Test**:
```bash
# Run complete test suite locally
nix develop --command just ci-all

# Run on all feature combinations
for feature in "" "external_scanners" "incremental_glr"; do
  nix develop --command cargo test --workspace $feature
done

# Verify performance consistency
for i in {1..5}; do
  nix develop --command just ci-perf
done
```

---

### Phase 3: Documentation Update (Week 1, Day 5)

**Goal**: Update all documentation to reflect Nix integration

**Tasks**:

1. **Update CLAUDE.md** (1 hour)
   - Add "Nix Development Shell" section at top
   - Move traditional setup to "Alternative: Traditional Setup"
   - Add troubleshooting tips
   - Link to detailed guides

2. **Create troubleshooting guide** (2 hours)
   - File: `docs/guides/NIX_TROUBLESHOOTING.md`
   - Common issues and solutions
   - Platform-specific notes
   - FAQ format

3. **Create migration guide** (1 hour)
   - File: `docs/guides/MIGRATING_TO_NIX.md`
   - Step-by-step migration
   - For developers with existing setups
   - Show before/after workflows

4. **Update ADR-0008** (30 mins)
   - Mark Week 1 tasks as COMPLETE
   - Update implementation status
   - Add lessons learned section

**Acceptance Test**:
```bash
# Verify all links work
for doc in CLAUDE.md docs/guides/NIX*.md docs/adr/ADR-0008*.md; do
  if [ -f "$doc" ]; then
    echo "Checking $doc..."
    # Manual review of content
  fi
done
```

---

## III. Success Metrics

### Quantitative Metrics

| Metric | Baseline | Target | Measurement |
|--------|----------|--------|-------------|
| CI pass rate | 95% | ≥95% | GitHub Actions |
| Local/CI result match | 80% | 100% | Test runs |
| Benchmark variance | 5% | <2% | 5-run average |
| Setup time (new contributor) | 30 min | 5 min | Manual timing |
| CI job duration | Current | +5% max | GitHub Actions |

### Qualitative Metrics

- [ ] Team feedback: Nix improves workflow (survey)
- [ ] Zero "works locally but not in CI" issues (2 weeks)
- [ ] New contributors report smooth onboarding
- [ ] Documentation clarity >4.5/5 (feedback)

---

## IV. Testing Strategy

### Unit Tests
- No new unit tests required (infrastructure change)
- All existing tests must pass

### Integration Tests
- [ ] CI workflow runs on test PR
- [ ] All platform combinations pass
- [ ] Feature matrix tests pass
- [ ] Performance benchmarks stable

### System Tests
- [ ] End-to-end workflow: clone → `nix develop` → `just ci-all` → success
- [ ] Reproduction: CI failure → local debug → fix → CI pass
- [ ] Multi-platform: Works on Ubuntu, macOS, Windows (WSL)

### BDD Scenarios
All scenarios from AC-1 through AC-5 must pass

---

## V. Risk Assessment

### High Risks

| Risk | Impact | Likelihood | Mitigation |
|------|--------|------------|------------|
| Windows CI incompatibility | HIGH | MEDIUM | Document WSL requirement or fallback to traditional |
| Nix install failures in CI | HIGH | LOW | Use stable cachix action, test thoroughly |
| Performance overhead | MEDIUM | LOW | Benchmark before/after, optimize if needed |

### Medium Risks

| Risk | Impact | Likelihood | Mitigation |
|------|--------|------------|------------|
| Team resistance | MEDIUM | MEDIUM | Show clear benefits, make optional initially |
| Documentation gaps | MEDIUM | MEDIUM | Comprehensive review, feedback loop |
| CI cache issues | MEDIUM | MEDIUM | Configure cachix, monitor cache hit rate |

### Mitigation Strategies

1. **Rollback Plan**: Keep traditional CI as backup branch for 1 week
2. **Incremental Adoption**: Merge but make Nix optional initially
3. **Monitoring**: Track CI success rate daily for first week
4. **Support**: Dedicated Nix troubleshooting in team chat

---

## VI. Completion Checklist

### Code Changes
- [ ] `.github/workflows/ci.yml` updated to use Nix
- [ ] `.github/workflows/performance.yml` updated
- [ ] `.github/workflows/test-connectivity.yml` updated
- [ ] Any other workflows updated

### Documentation
- [ ] `CLAUDE.md` updated with Nix quickstart
- [ ] `docs/guides/NIX_TROUBLESHOOTING.md` created
- [ ] `docs/guides/MIGRATING_TO_NIX.md` created
- [ ] `docs/adr/ADR-0008-NIX-DEVELOPMENT-ENVIRONMENT.md` updated
- [ ] `CONTRIBUTING.md` updated
- [ ] `FAQ.md` updated with Nix section

### Testing
- [ ] All CI jobs pass on test PR
- [ ] Ubuntu platform tested
- [ ] macOS platform tested
- [ ] Windows/WSL approach documented
- [ ] Feature matrix tested
- [ ] Performance benchmarks validated
- [ ] Local/CI reproduction verified

### Validation
- [ ] Team review completed
- [ ] At least 2 team members using Nix locally
- [ ] New contributor onboarding tested
- [ ] All success metrics met

---

## VII. References

**Related Documents**:
- [ADR-0008: Nix Development Environment](../adr/ADR-0008-NIX-DEVELOPMENT-ENVIRONMENT.md)
- [STRATEGIC_IMPLEMENTATION_PLAN.md](../plans/STRATEGIC_IMPLEMENTATION_PLAN.md)
- [flake.nix](/flake.nix)
- [justfile](/justfile)

**External Resources**:
- [Nix Flakes Documentation](https://nixos.wiki/wiki/Flakes)
- [cachix/install-nix-action](https://github.com/cachix/install-nix-action)
- [GitHub Actions with Nix](https://nixos.wiki/wiki/GitHub_Actions)

---

## VIII. Approval and Status

**Status**: ACTIVE
**Approved By**: Core Team
**Approval Date**: 2025-11-20
**Target Completion**: 2025-11-27 (1 week)
**Next Review**: Daily during implementation

---

**Contract Version**: 1.0.0
**Last Updated**: 2025-11-20
**Maintained By**: rust-sitter core team

---

END OF CONTRACT
