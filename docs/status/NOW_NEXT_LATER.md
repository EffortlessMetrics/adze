# Now / Next / Later

**Last updated:** 2026-04-06
**Status:** **Post-PR264 closeout** — `main` is clean, the supported gate is green, PR-level follow-up closure is complete, and hardening continues on open issues `#268` and `#269`.

Adze status and rolling execution plan. For recurring pain points, see [`docs/status/FRICTION_LOG.md`](./FRICTION_LOG.md). For API stability guarantees per crate, see [`docs/status/API_STABILITY.md`](./API_STABILITY.md). For the focused follow-up execution plan after PR #264, see [`plans/POST-PR264-CI-FOLLOWUPS.md`](../../plans/POST-PR264-CI-FOLLOWUPS.md).

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
- [x] `gh pr list --state open` currently has no active follow-up PRs on GitHub (`0 open PRs`).
- [x] `/home/steven/code/rust-sitter` is clean on `main` and aligned with `origin/main`.
- [x] The remaining work is follow-up hardening, not PR-backlog triage.

---

## Now

### 🛠️ Convert the last-mile CI pain into tracked follow-up work
- [x] [Issue #267](https://github.com/EffortlessMetrics/adze/issues/267): stabilize backend-selection expectations across feature profiles and conflict tests.
- [ ] [Issue #269](https://github.com/EffortlessMetrics/adze/issues/269): reduce and instrument the long Windows pure-rust benchmark-compilation tail.
- [ ] [Issue #268](https://github.com/EffortlessMetrics/adze/issues/268): document and harden temporary worktree cleanup so local closeout stays predictable.

### 📦 Keep the supported contract explicit
- [ ] Treat broader CI/workflow cleanup as follow-up hardening, not as a reason to reopen the PR backlog.
- [ ] Keep `KNOWN_RED.md` current whenever an advisory lane is promoted into or removed from the supported contract.
- [ ] Preserve a clear distinction between "supported merge gate" and "useful advisory signal" in workflow/docs updates.

---

## Next

### 🚢 Publication and release preparation
- [ ] Reconfirm the publish surface and release checklist against the current `main` branch rather than the older RC-era status docs.
- [ ] Separate crates.io publication work from advisory CI hardening so release decisions stay legible.
- [ ] Trim or retire stale planning language that still reads like pre-merge backlog work.

### 📚 Documentation polish
- [ ] Continue tightening tutorial/reference accuracy around the actual post-merge API surface.
- [ ] Add contributor-facing guidance for temporary worktree lifecycle and closeout hygiene.
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
