# Now / Next / Later

**Last updated:** 2026-04-09
**Status:** **Post-release hardening** — `adze` 0.8.0 is live on crates.io, the supported gate remains green, and the active work is broad CI truthfulness, runtime surface repair, and doc/status sync.

Adze status and rolling execution plan. For recurring pain points, see [`docs/status/FRICTION_LOG.md`](./FRICTION_LOG.md). For API stability guarantees per crate, see [`docs/status/API_STABILITY.md`](./API_STABILITY.md). For the (substantially complete) post-PR264 follow-up plan, see [`plans/POST-PR264-CI-FOLLOWUPS.md`](../../plans/POST-PR264-CI-FOLLOWUPS.md).

---

## Done

### ✅ Baseline landed on `main`
- [x] The supported contract remains `just ci-supported` locally and `CI / ci-supported` in GitHub.
- [x] Supported crates compile, format, lint, test, and document cleanly on `main`.
- [x] Feature-matrix coverage no longer carries the prior expected failure in the supported lane.
- [x] PR [#264](https://github.com/EffortlessMetrics/adze/pull/264) merged on 2026-04-03 as commit `2a88deb6e6095682051729290987a78a0565d613`.
- [x] The temporary convergence worktrees/branches used for the PR stack were cleaned up.
- [x] A safety archive of the pre-cleanup dirty checkout was preserved outside `/tmp`.
- [x] Issue #268 worktree cleanup documentation and validation is now documented and backed by a helper script.

### ✅ Immediate close-out state
- [x] PR `#280` (workflow hardening) merged on 2026-04-06.
- [x] PR `#281` (roadmap/execution-state refresh) merged.
- [x] `/home/steven/code/rust-sitter` is clean on `main` and aligned with `origin/main`.
- [x] The remaining work is publication preparation and 0.9.0 planning, not backlog triage.

---

## Now

### 🧪 Broad CI truthfulness and hardening
- [ ] Merge the remaining post-release hardening PRs and keep `KNOWN_RED.md` aligned with the real advisory-lane state.
- [ ] Finish the runtime all-features and GLR/bench stability follow-up lanes now that the crates.io release is complete.
- [ ] Restore any valuable proof surfaces that were trimmed only to unblock publication into `publish = false` internal harnesses where needed.

### 📦 Close remaining operational issues
- [ ] [Issue #269](https://github.com/EffortlessMetrics/adze/issues/269): Windows pure-rust benchmark-compilation tail is gated but still open; decide whether to trim further or close as acceptable.
- [ ] [Issue #268](https://github.com/EffortlessMetrics/adze/issues/268): Worktree cleanup script exists (`scripts/cleanup-worktrees.sh`); contributor documentation still needs finishing.
- [ ] Keep `KNOWN_RED.md` current whenever an advisory lane is promoted into or removed from the supported contract.

---

## Next

### 📚 Documentation polish
- [ ] Continue tightening tutorial/reference accuracy around the actual 0.8.x release surface.
- [ ] Add contributor-facing guidance for temporary worktree lifecycle and closeout hygiene ([issue #268](https://github.com/EffortlessMetrics/adze/issues/268)).
- [ ] Keep roadmap/status docs aligned with the real repo state after each meaningful convergence wave.

---

## Later

### ⚡ Performance optimization
- Arena allocator for parse forest nodes.
- Incremental parsing improvements beyond conservative fallback.
- Benchmark suite with clearer regression detection and less CI noise.

### 🌳 Incremental parsing
- Move from conservative fallback toward active forest-splicing for editor-scale workflows.
- Revisit the currently deferred incremental path once the surrounding runtime contracts are steadier.

### 🔍 Query and tooling expansion
- Implement remaining Tree-sitter query predicates and cookbook coverage.
- Continue CLI/tooling polish now that the basic command surface exists.
- Stabilize the LSP generator and related developer tooling for broader use.
