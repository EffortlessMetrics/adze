# Now / Next / Later

**Last updated:** 2026-05-07
**Status:** **Post-queue correctness proof** — `adze` 0.8.0 is live on crates.io, the supported gate remains bounded, and the live GitHub PR queue is empty as of the latest refresh. Remaining work is tracked as focused proof issues rather than a broad merge queue. The current push is tracked in [`CORRECTNESS_PUSH.md`](./CORRECTNESS_PUSH.md).

Adze status and rolling execution plan. For recurring pain points, see [`docs/status/FRICTION_LOG.md`](./FRICTION_LOG.md). For API stability guarantees per crate, see [`docs/status/API_STABILITY.md`](./API_STABILITY.md). For support-tier proof commands, see [`docs/status/SUPPORT_TIERS.md`](./SUPPORT_TIERS.md).

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

### ✅ Prior close-out state
- [x] PR `#280` (workflow hardening) merged on 2026-04-06.
- [x] PR `#281` (roadmap/execution-state refresh) merged.
- [x] `main` is aligned with `origin/main` and is now the source of truth for the remaining hardening work.
- [x] A restore audit on 2026-04-11 confirmed that the proof surfaces trimmed during publication are already present again on `main`.

---

## Now

### Correctness queue baseline
- [x] Refresh live PR state with `gh pr list --state open --limit 20 --json number,title,isDraft,headRefName,updatedAt,url`.
- [x] Confirm the live PR queue is empty after the tablegen/runtime correctness fixes landed.
- [x] Keep the historical queue closed; do not revive stale PR numbers from handoffs unless GitHub shows them open again.
- [ ] For any new correctness PR, keep the one-PR loop: rebase on current `main`, run focused proof, require hosted `CI / ci-supported`, and report red checks before merge.

### Product proof alignment
- [ ] Keep `just ci-supported` as the fast required gate.
- [x] Convert `scripts/ci-product.sh` from compile-only advisory smoke to bounded behavior canaries where behavior is currently truthful; benchmarks and WASM remain explicit compile/no-run canaries.
- [x] Track GLR product proof in [#460](https://github.com/EffortlessMetrics/adze/issues/460), tablegen ABI completeness in [#461](https://github.com/EffortlessMetrics/adze/issues/461), and parse diagnostics in [#463](https://github.com/EffortlessMetrics/adze/issues/463).
- [x] Close out CLI clean-room quickstart/truthfulness and README/support-tier reconciliation as landed proof work ([#464](https://github.com/EffortlessMetrics/adze/issues/464), [#465](https://github.com/EffortlessMetrics/adze/issues/465)).
- [ ] Keep README feature claims aligned with [`SUPPORT_TIERS.md`](./SUPPORT_TIERS.md): no Stable claim without a named proof command.

### Operational tail
- [ ] [Issue #269](https://github.com/EffortlessMetrics/adze/issues/269): Windows pure-rust benchmark-compilation tail is gated but still open; decide whether to trim further or close as acceptable.
- [ ] [Issue #268](https://github.com/EffortlessMetrics/adze/issues/268): Worktree cleanup script exists (`scripts/cleanup-worktrees.sh`); contributor documentation still needs finishing.
- [ ] Investigate the current rustdoc-only `Documentation` lane failure separately from reader-facing markdown/status drift.

---

## Next

### Behavior-proof product lane
- [x] Add an advisory stable product lane for README-stable claims.
- [ ] Promote `ci-product-stable` to required only after advisory behavior smokes pass consistently.
- [ ] Keep broad workspace, fuzzing, Miri, sanitizers, browser WASM, grammar corpus, runtime2, and benchmarks scheduled/manual unless explicitly promoted.

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
