# Adze CI economics rollout plan

This document is the agent-readable control plane for the CI economics rollout.
Treat the rollout as **CI infrastructure product work**, not workflow cleanup.
The product principle is:

> Every PR gets one cheap required proof. Everything else is advisory, routed,
> scheduled, manual, release-only, or label-triggered.

## Budget doctrine

Adze budgets CI in Linux-equivalent minutes (LEM):

```text
LEM = wall-clock job minutes × runner multiplier
Linux = 1.0, Windows = 2.0, macOS = 10.0
```

Ordinary PRs should normally fit this shape:

| Budget | Target |
| --- | ---: |
| preferred ordinary PR | <=25 LEM |
| ordinary ceiling | <=35 LEM |
| elevated | 36-75 LEM, warning and explicit risk surface |
| high | 76-125 LEM, label/ack recommended |
| over ceiling | >125 LEM, requires `full-ci` or `ci-budget-override` |

The operating target is sub-$0.50 ordinary PRs. A $1 PR is a ceiling, not the
product design center.

## Target default PR shape

For ordinary Rust PRs:

| Lane | Blocking | Target LEM | Notes |
| --- | ---: | ---: | --- |
| PR Plan | no | ~1 | Changed surfaces, selected lanes, budget band |
| Supported Rust Gate | yes | ~18-22 | `just ci-supported` |
| PR Gate Success | yes | ~1 | Stable aggregate, future branch-protection context |
| CI lane whitelist | advisory | ~1-2 | Prevent workflow sprawl |
| ripr advisory | advisory | ~3-5 | Static oracle-gap signal; skip docs-only |
| test-policy smoke | advisory or blocking | ~1-3 | Disabled-test and hygiene smoke only |

For docs-only PRs:

| Lane | Blocking | Target LEM |
| --- | ---: | ---: |
| PR Plan | no | ~1 |
| Docs Gate | yes | ~1-2 |
| PR Gate Success | yes | ~1 |
| Policy lint | advisory | ~1 |

Everything else is outside the ordinary default and must be routed by path,
label, schedule, manual dispatch, main, merge queue, or release.

## Implementation order

```text
stabilize docs/control plane
-> make PR Gate authoritative
-> remove duplicate PR execution
-> route expensive lanes harder
-> enforce lane metadata
-> add labels / branch-protection rails
-> collect actuals
-> ratchet budgets from measured data
```

## Source-of-truth split

| Source | Role |
| --- | --- |
| `docs/ci/cost-and-verification-policy.md` | Doctrine, budget bands, and non-goals |
| `docs/ci/adze-rollout-plan.md` | Ordered rollout sequence and per-PR contract |
| `docs/ci/adze-rollout-status.md` | Current snapshot of present / hardening / pruning / deferred work |
| `docs/ci/branch-protection.md` | Required-check migration criteria and rollback |
| `docs/ci/labels.md` | Operator label vocabulary for expensive verification |
| `docs/ci/learned-estimates.md` | Future actuals-driven estimate model |
| `.github/CI_LANES.md` | Contributor-facing check semantics |
| `policy/ci-lane-whitelist.toml` | Lane registry: owner, LEM, evidence, duplicate map |
| `policy/ci-risk-packs.toml` | Path and label routing vocabulary |

## Queue discipline

Do not edit source-of-truth scaffolding, policy ledgers, or the CI economics
docs while a conflicting docs stack is still open unless the stack has landed,
been closed, or this branch has been explicitly rebased over it.

Before touching CI rollout files, run:

```bash
git diff --check
python -c "import pathlib, tomllib; p=pathlib.Path('.adze/goals/active.toml'); p.exists() and tomllib.load(p.open('rb'))"
```

## Status legend

- ✅ landed — present in the current branch and intended as active policy
- 🟡 needs hardening — present but too broad, advisory-only, or still carrying an exception
- 🧹 needs pruning — duplicate or expensive default execution to reroute/remove
- ⏳ planned — not yet implemented
- ⏸ deferred — blocked on actuals or coordination

## Current control-plane status

### Already present

| Rail | Status | Notes |
| --- | --- | --- |
| LEM doctrine and budget bands | ✅ | `<=25` preferred, `<=35` ordinary ceiling |
| CI lane whitelist | ✅ | Lane registry exists with owner, evidence, duplicate map |
| CI risk packs | ✅ | Path/label routing vocabulary exists |
| PR Plan | ✅ | Emits static estimate, docs-only signal, and budget band |
| PR Gate Success | ✅ | Aggregates supported/docs gate shape |
| ci-actuals scaffold | ✅ | Artifact scaffold exists for future learned estimates |
| coverage label gate | ✅ | Coverage is non-default and label/main/manual routed |

### Needs hardening

| Rail | Status | Required hardening |
| --- | --- | --- |
| PR Gate Success | 🟡 | Promote to the single required branch-protection context after stability window |
| CI lane whitelist | 🟡 | Make blocking only for workflow/policy/doc changes |
| PR Plan budgets | 🟡 | Warn/fail static over-budget PRs with override labels |
| test-policy | 🟡 | Split cheap PR smoke from full inventory enforcement |
| ci-actuals | 🟡 | Normalize per-lane receipts with runner multiplier and selected-by metadata |

### Needs pruning or stronger routing

| Rail | Status | Required pruning/routing |
| --- | --- | --- |
| legacy `ci.yml` PR execution | 🧹 | Stop ordinary PR trigger after PR Gate is authoritative |
| pure-rust platform proof | 🧹 | Make PR execution label/manual/main/schedule only |
| microcrate CI | 🧹 | Keep path-routed crate groups; move docs/WASM/strict features off default PR |
| performance comparison | 🧹 | Keep compile smoke path-routed; label-gate comparisons |
| ts-bridge | 🧹 | Consolidate smoke and move parity off ordinary default |
| API/SemVer checks | 🧹 | Route by API-risk paths and release/API labels |
| main-push deep verification | 🧹 | Keep main smoke cheap; move OS matrix/fuzz/coverage/benchmarks to nightly/manual/release |

### Deferred until actuals

| Rail | Status | Deferral condition |
| --- | --- | --- |
| learned estimates | ⏸ | Wait for >=30 days or enough `ci-actuals.json` samples |
| ledger ratchet | ⏸ | Update static `base_lem` from measured p90/p95 only after sufficient samples |

## Next implementation wave

These PRs are intentionally small and single-purpose. Do not combine branch
protection changes with workflow pruning.

| PR | Title | Status | Files / surfaces | Acceptance |
| ---: | --- | --- | --- | --- |
| 1 | `docs(ci): reconcile CI economics rollout status` | ✅ | CI docs and ledgers | `git diff --check`; advisory policy commands when available |
| 2 | `ci(labels): add CI routing labels` | ⏳ | `.github/settings.yml`, `docs/ci/labels.md` | `git diff --check` |
| 3 | `ci: require PR Gate Success as the stable merge gate` | ⏳ | `.github/settings.yml`, branch-protection docs, lane docs | `git diff --check` |
| 4 | `ci: stop running legacy ci.yml on ordinary PRs` | ⏳ | `.github/workflows/ci.yml`, lane docs/ledger | whitelist advisory; `git diff --check` |
| 5 | `ci(test-policy): split PR smoke from full inventory enforcement` | ⏳ | `test-policy.yml`, xtask/scripts, lane ledger | smoke/full policy commands; whitelist advisory |
| 6 | `ci(pure-rust): make platform proof label and main only` | ⏳ | `pure-rust-ci.yml`, lane ledger/docs | whitelist advisory; no macOS/Windows default PR |
| 7 | `ci(microcrate): route docs wasm and strict features off ordinary PRs` | ⏳ | `microcrate-ci.yml`, lane ledger/docs | whitelist advisory |
| 8 | `ci(perf): keep benchmark smoke default but label-gate comparisons` | ⏳ | `performance.yml`, `benchmarks.yml`, lane ledger/docs | whitelist advisory |
| 9 | `ci(ts-bridge): consolidate smoke and move parity off default PR` | ⏳ | ts-bridge workflows, lane ledger/docs | whitelist advisory |
| 10 | `ci(api): route public API checks by API risk` | ⏳ | API/SemVer jobs and docs | semver/public-api advisory; `git diff --check` |
| 11 | `ci(policy): block undeclared workflow lanes` | ⏳ | policy workflow, xtask, docs | whitelist blocking for workflow changes |
| 12 | `ci(plan): enforce static LEM ceilings with override labels` | ⏳ | PR Plan/xtask/scripts | static planner receipts; `git diff --check` |
| 13 | `ci: split main smoke from nightly deep verification` | ⏳ | main/schedule/manual workflow routing | whitelist advisory; no routine macOS main-push spend |
| 14 | `ci(actuals): normalize per-lane LEM receipts` | ⏳ | `scripts/ci/emit-ci-actuals.py`, docs | JSON schema validates |
| 15 | `ci(metrics): compute learned lane estimates from actuals` | ⏸ | metrics scripts/docs | advisory only after enough samples |
| 16 | `ci(metrics): update lane LEM baselines from measured actuals` | ⏸ | lane ledger/docs | owner signoff; estimates not below p90 without reason |

## Per-PR operating contract

Every CI economics PR body must include:

```markdown
## CI economics

- Estimated LEM impact:
- Workflows touched:
- Default PR effect:
- Branch protection impact:
- Rollback path:
- Proof obligation:
- Cheaper signal considered:
- Expensive runners added? yes/no
- macOS/windows default PR? must be no

## Verification

- [ ] `cargo xtask check-ci-lane-whitelist --mode advisory`
- [ ] `cargo xtask policy-report || true`
- [ ] `git diff --check`

## Claim boundary

This PR changes CI routing/cost only. It does not weaken `just ci-supported`,
remove deep verification from main/nightly/release/manual paths, or make
advisory lanes required.
```

## Agent rules

1. One CI intention per PR.
2. Never combine branch protection change with large workflow pruning.
3. Never introduce macOS or Windows as default PR lanes.
4. Never add a workflow job without a lane entry.
5. Never make a raw matrix leaf a required branch-protection check.
6. Keep PR Gate Success as the stable required summary.
7. Keep deep verification available through main/nightly/manual/label/release.
8. If a job is `duplicate_of` another lane, justify it or reroute/remove it.
9. If a lane is expensive and `default_pr = true`, it needs an exception and expiry.
10. After each merge, refresh `main` and rerun the lane whitelist check.

## Rollback paths

| Layer | Rollback |
| --- | --- |
| docs/specs | revert the PR |
| labels | remove the added labels from `.github/settings.yml` |
| policy TOML | revert the lane/risk-pack diff; advisory lints stop firing |
| advisory workflow | revert/delete workflow change; required gate remains unchanged |
| PR Gate Success promotion | switch required context back to `CI / ci-supported` |
| routing changes | restore prior triggers; deep lanes remain available through labels/manual/main |
| learned estimates | fall back to static `base_lem` values |
