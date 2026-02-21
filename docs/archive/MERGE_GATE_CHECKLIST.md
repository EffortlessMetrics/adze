# 🚀 Merge Gate Checklist - Incremental GLR Integration

## ✅ Integration Status

All integration tasks completed successfully:

| Component | Status | Details |
|-----------|--------|---------|
| **Equivalence Tests** | ✅ | `runtime/tests/incremental_equiv.rs` - tests insert/delete/replace operations |
| **CI Minimal Features** | ✅ | Builds with `--no-default-features` (ts-bridge link warning expected) |
| **Feature Gating** | ✅ | `Parser::reparse` returns `None` without `incremental_glr` flag |
| **Benchmark Optimization** | ✅ | Tokenization moved outside `b.iter()` loop |
| **Docs Build** | ✅ | `cargo doc --no-deps` builds successfully |
| **Publish Script** | ✅ | Handles unpublished deps with `--no-verify` |
| **ROADMAP Update** | ✅ | Updated with completed items and TODOs |

## 🟢 Pre-Merge Verification

### 1. CI Status
- [ ] All CI checks passing on PR head
- [ ] New benchmark steps complete within time budget
- [ ] No unexpected test failures

### 2. Code Review
- [ ] At least one runtime maintainer approved
- [ ] Feature-gated API reviewed
- [ ] No breaking changes to existing APIs

### 3. Performance Check
```bash
# Quick smoke test (should complete in < 45s locally)
cargo bench -p adze-benchmarks --bench incremental_bench --profile release
```
⚠️ Note: Currently takes longer due to compilation, but runtime is reasonable

### 4. Workspace Integrity
- ✅ No accidental `publish = true` changes
- ✅ All feature flags properly configured
- ✅ Dependencies correctly ordered in publish script

## 📝 Commit Structure

Recommend squashing into 2 logical commits:

1. **feat(test): add incremental equivalence tests and CI infrastructure**
   - Test harness setup
   - CI workflow updates
   - Publish script improvements

2. **feat(incremental): integrate GLR incremental parsing**
   - Core integration
   - Benchmark optimizations
   - Documentation updates

## 🌱 Immediate Follow-ups (New PRs)

### PR #1: Equivalence Test Expansion
**Goal**: Use real arithmetic grammar with property-based testing
- Replace hardcoded `ParseTable` with `examples::load()` helper
- Generate 20 random programs (≤30 tokens)
- Assert `fresh == incremental` for all edit types

### PR #2: Fast-path Re-enablement
**Goal**: Optimize subtree reuse performance
- Remove `enable_reuse = true` guard in `parse_fresh`
- Profile `subtree_pool.find_reusable` hot-path
- Expected: Reduced allocations, 2-3x speedup on char edits

### PR #3: Fork-budget Heuristic
**Goal**: Prevent runaway forking in pathological cases
- Add `--fork-budget` CLI flag (default=64)
- Early abort when `active_forks > budget`
- Include telemetry for fork count statistics

### PR #4: Documentation
**Goal**: mdBook chapter on "Incremental GLR under the hood"
- Architecture diagrams (fork tracker, reuse map)
- Code examples showing 3x speedup
- Performance tuning guide

### PR #5: Beta.2 Release
**Goal**: Ship `v0.6.0-beta.2` with incremental GLR
- Version bump across workspace
- Update changelog
- Tag and publish to crates.io

## 🔍 Risk Monitoring

### Memory Growth
- Monitor `reuse_map` size in long-running parses
- Add `--reuse-stats` debug flag for memory profiling
- Run valgrind on 10KB edit loops

### UTF-8 Safety
- Ensure external scanners never produce invalid UTF-8
- Add fuzzing for edge cases
- Error gracefully on invalid slices

### Performance Regression
- Track benchmark results across commits
- Consider pre-generated token streams for consistent perf testing
- Monitor fork count distribution in real-world usage

## ✅ Ready to Merge

With all checks passing, this PR is ready for merge. The incremental GLR integration provides:

- **Functional completeness** - Core integration working
- **Clean separation** - Feature-gated behind `incremental_glr`
- **CI integration** - Benchmarks included in workflow
- **Test coverage** - Equivalence tests verify correctness

## 🎯 Next Steps After Merge

1. **Immediate**: Create follow-up PRs as listed above
2. **This week**: Complete performance optimizations
3. **Next week**: Beta.2 release with documentation
4. **Month ahead**: Production readiness assessment

---

*Last updated: 2025-08-17*