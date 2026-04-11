# Now / Next / Later

**Last updated:** 2026-04-11
**Status:** **Post-release hardening on `main`** — `adze` 0.8.0 is live on crates.io, the supported gate remains green, there is no open PR stack, and the remaining work is the residual advisory/broad CI tail on current `main`.

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
- [x] `main` is aligned with `origin/main` and is now the source of truth for the remaining hardening work.
- [x] GitHub currently shows no open PRs; any remaining hardening should restart from fresh branches off current `main`.
- [x] A restore audit on 2026-04-11 confirmed that the proof surfaces trimmed during publication are already present again on `main`.

---

## Now

### 🧪 Broad CI truthfulness and hardening
- [ ] Finish the workflow/toolchain tail on current `main`: sanitizers, minimal-versions, supply-chain checks, cross-compilation, and the long-running Miri lane.
- [ ] Clear the remaining `adze` broad-surface failures on current `main`: matrix smoke, coverage, strict-invariants release mode, feature-matrix `serialization` / `all-features` / `glr`, and the matching cross-platform test lanes.
- [ ] Clear the remaining GLR tail on current `main`: `adze-glr-core / all-features` and deterministic codegen.
- [ ] Keep `KNOWN_RED.md` aligned with the real advisory-lane state whenever a lane stops being intentionally red.

### 📦 Close remaining operational issues
- [ ] [Issue #269](https://github.com/EffortlessMetrics/adze/issues/269): Windows pure-rust benchmark-compilation tail is gated but still open; decide whether to trim further or close as acceptable.
- [ ] [Issue #268](https://github.com/EffortlessMetrics/adze/issues/268): Worktree cleanup script exists (`scripts/cleanup-worktrees.sh`); contributor documentation still needs finishing.
- [ ] Investigate the current rustdoc-only `Documentation` lane failure separately from reader-facing markdown/status drift.

---

## Next

### 📚 Documentation polish
- [ ] Continue tightening tutorial/reference accuracy around the actual 0.8.x release surface.
- [ ] Add contributor-facing guidance for temporary worktree lifecycle and closeout hygiene ([issue #268](https://github.com/EffortlessMetrics/adze/issues/268)).
- [ ] Keep roadmap/status docs aligned with the real repo state after each meaningful convergence wave, but only after the corresponding code/CI family lands on `main`.

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
