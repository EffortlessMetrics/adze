# CI Economics Rollout Status

Last review: 2026-05-12.

This file is the current status snapshot for operators and agents. The ordered
execution plan lives in `docs/ci/adze-rollout-plan.md`; this document answers
"what is already present, what still needs hardening, what needs pruning, and
what is deferred?" before an implementation PR starts.

## Status legend

- ✅ landed — present in the current branch and intended as active policy
- 🟡 needs hardening — present but advisory-only, too broad, or still carrying an exception
- 🧹 needs pruning — duplicate or expensive default execution to reroute/remove
- ⏳ planned — not yet implemented
- ⏸ deferred — waiting on actuals or coordination

## Already present

| # | Rail | Status | Notes |
| --- | --- | --- | --- |
| A01 | Cost doctrine | ✅ | `docs/ci/cost-and-verification-policy.md` defines LEM, multipliers, bands, and non-goals. |
| A02 | Lane whitelist | ✅ | `policy/ci-lane-whitelist.toml` registers workflow lanes with owner, LEM, evidence, and duplicate metadata. |
| A03 | Risk packs | ✅ | `policy/ci-risk-packs.toml` maps paths and labels to lane vocabulary. |
| A04 | PR Plan | ✅ | `.github/workflows/pr-plan.yml` emits docs-only, estimated LEM, and budget-band outputs. |
| A05 | PR Gate Success | ✅ | `.github/workflows/pr-gate.yml` exposes Supported Rust Gate, Docs Gate, and aggregate success shape. |
| A06 | ci-actuals scaffold | ✅ | `scripts/ci/emit-ci-actuals.py` gives the future learned-estimates pipeline a receipt path. |
| A07 | coverage routing | ✅ | `coverage.yml` is label/main/manual routed, not an ordinary default lane. |
| A08 | routing label spec | ✅ | `docs/ci/labels.md` defines the operator vocabulary for expensive lanes. |

## Needs hardening

| # | Rail | Status | Next implementation PR |
| --- | --- | --- | --- |
| H01 | Branch protection | 🟡 | Promote `PR Gate / PR Gate Success` after stability criteria; keep rollback to `CI / ci-supported`. |
| H02 | Lane whitelist enforcement | 🟡 | Make blocking only when workflows, CI policy, CI docs, scripts, or xtask change. |
| H03 | PR Plan budget behavior | 🟡 | Warn/fail from static estimates: ordinary passes; elevated/high warn; over-ceiling requires override labels. |
| H04 | Test policy | 🟡 | Split `test-policy-smoke` for ordinary PRs from `test-policy-full` for main/nightly/manual/`full-ci`. |
| H05 | ci-actuals schema | 🟡 | Record per-lane id, workflow, job, runner, wall minutes, multiplier, LEM, selected-by, result, and total LEM. |

## Needs pruning or stronger routing

| # | Lane / workflow | Status | Intended default-PR effect |
| --- | --- | --- | --- |
| P01 | legacy `ci.yml` PR trigger | 🧹 | Remove ordinary `pull_request` execution once PR Gate is required. |
| P02 | `pure-rust-ci.yml` | 🧹 | No ordinary default platform proof; run by `platform-matrix`, `pure-rust`, `full-ci`, main, schedule, or manual. |
| P03 | `microcrate-ci.yml` | 🧹 | Keep path-matched crate-group tests; move docs/WASM/strict features to labels/main/manual. |
| P04 | `performance.yml` and `benchmarks.yml` | 🧹 | Keep compile smoke path-routed; label-gate baseline comparisons and full suite. |
| P05 | ts-bridge workflows | 🧹 | Keep one path-routed smoke; move parity to main/schedule/manual/label. |
| P06 | API/SemVer checks | 🧹 | Route by public API risk paths or `api` / `release-check` / `breaking-change` / `full-ci` labels. |
| P07 | main-push deep verification | 🧹 | Keep main smoke cheap; move OS matrix, fuzz, full coverage, and benchmarks to nightly/manual/release. |

## Deferred until actuals

| # | Item | Status | Start condition |
| --- | --- | --- | --- |
| D01 | Learned estimates | ⏸ | >=30 days or enough per-lane actual samples with stable p50/p90/p95. |
| D02 | LEM ledger ratchet | ⏸ | Enough samples to update `base_lem`; owner signoff for expensive/default lanes. |

## Effective default PR lane cost (current target)

| Lane | Target default PR cost | Blocking |
| --- | ---: | ---: |
| PR Plan | ~1 LEM | no |
| Supported Rust Gate | ~18-22 LEM | yes |
| PR Gate Success | ~1 LEM | yes |
| CI Lane Whitelist | ~1-2 LEM | advisory |
| ripr advisory | ~3-5 LEM | advisory |
| test-policy smoke | ~1-3 LEM | advisory or yes |
| **Preferred ordinary PR target** | **<=25 LEM** | — |
| **Ordinary ceiling** | **<=35 LEM** | — |

Until the pruning/hardening PRs land, some existing lanes may still run more
broadly than this target. Those exceptions must remain visible in
`policy/ci-lane-whitelist.toml` or `policy/ci-whitelist-exceptions.toml` and
should be retired by the numbered rollout PRs in `docs/ci/adze-rollout-plan.md`.
