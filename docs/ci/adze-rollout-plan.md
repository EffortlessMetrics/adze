# Adze CI economics rollout plan

This document is the authoritative implementation map for the CI economics
rollout. Treat the rollout as CI infrastructure product work: each PR should
change one intention, preserve the supported proof, and make the default PR
shape cheaper without deleting deep verification.

## Doctrine

Every PR gets one cheap required proof. Everything else is advisory, routed,
scheduled, manual, release-only, or label-triggered.

Ordinary PR budget targets:

```text
preferred ordinary PR: <=25 LEM
ordinary ceiling:      <=35 LEM
high-cost PRs:         require labels or override
```

`LEM = wall-clock job minutes × runner multiplier`; Linux is `1.0`, Windows is
`2x`, and macOS is `10x`. The design center is sub-`$0.50` ordinary PRs; `$1`
is a ceiling, not the target.

## Target default PR shape

### Ordinary Rust PRs

| Lane | Blocking | Target LEM | Notes |
| --- | ---: | ---: | --- |
| PR Plan | no | ~1 | Changed surfaces, selected lanes, budget band |
| Supported Rust Gate | yes | ~18-22 | `just ci-supported` |
| PR Gate Success | yes | ~1 | Stable required aggregate |
| CI lane whitelist | advisory | ~1-2 | Prevent workflow sprawl |
| ripr advisory | advisory | ~3-5 | Static oracle-gap signal; skip docs-only |
| Test-policy smoke | advisory or blocking | ~1-3 | Disabled-test / hygiene smoke only |

### Docs-only PRs

| Lane | Blocking | Target LEM |
| --- | ---: | ---: |
| PR Plan | no | ~1 |
| Docs Gate | yes | ~1-2 |
| PR Gate Success | yes | ~1 |
| Policy lint | advisory | ~1 |

Everything else must be routed by paths, labels, `main`, schedules, manual
dispatch, merge queues, or release events.

## Rollout sequence

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

## Status legend

- ✅ already present — implemented in the repo and available to build on
- 🟨 needs hardening — exists but needs stricter policy or operator rails
- 🧹 needs pruning — exists but still spends ordinary PR LEM unnecessarily
- ⏸ deferred until actuals — intentionally waits for telemetry or stability data
- ⏳ planned — not yet started

## Current truth table

### Already present

| Rail | Evidence | Notes |
| --- | --- | --- |
| CI cost doctrine | `docs/ci/cost-and-verification-policy.md` | Defines LEM, budget bands, verification ladder, and non-goals. |
| Lane registry | `policy/ci-lane-whitelist.toml` | Tracks owner, cost, evidence, triggers, duplicate lanes, and review windows. |
| Risk-pack vocabulary | `policy/ci-risk-packs.toml` | Provides path/label routing names for expensive verification. |
| PR Plan workflow | `.github/workflows/pr-plan.yml` | Emits changed surfaces, estimated LEM, docs-only classification, and budget band. |
| PR Gate workflow | `.github/workflows/pr-gate.yml` | Contains PR Plan, Supported Rust Gate, Docs Gate, PR Gate Success, and actuals emission. |
| Coverage lane routing | `.github/workflows/coverage.yml` | Non-default; label/main/manual coverage instrumentation. |
| Static actuals scaffold | `scripts/ci/emit-ci-actuals.py` | Produces a receipt format that can become per-lane telemetry. |
| Branch-protection source | `.github/settings.yml` | Currently still requires the legacy `CI / ci-supported` context. |

### Needs hardening

| Rail | Required follow-up | Why |
| --- | --- | --- |
| CI routing labels | Add the full routing vocabulary to `.github/settings.yml`. | Labels are the operator interface for expensive verification. |
| PR Gate Success | Promote to the sole required context after stability evidence. | Required checks should name a stable aggregate, not a raw matrix leaf. |
| Lane whitelist | Make blocking only for workflow/policy changes. | Prevents workflow sprawl without blocking unrelated code PRs. |
| PR Plan budget bands | Warn/fail from static estimates with `full-ci` / `ci-budget-override` escape. | Prevents unreviewed over-ceiling PR spend before learned estimates exist. |
| PR body contract | Require CI economics metadata on CI-routing PRs. | Keeps LEM impact, rollback, branch protection, and proof obligations explicit. |

### Needs pruning

| Lane / workflow | Required follow-up | Expected default-PR effect |
| --- | --- | --- |
| Legacy `ci.yml` PR trigger | Remove ordinary `pull_request` execution after PR Gate is required. | Avoids duplicate supported proof on ordinary PRs. |
| `test-policy` | Split cheap PR smoke from full inventory enforcement. | Saves ~9-10 LEM per ordinary PR. |
| `pure-rust-ci` | Make platform proof label/main/manual/scheduled only. | Saves ~18 LEM per ordinary PR. |
| `microcrate-ci` | Keep changed crate-group tests; move docs/WASM/strict features off default PR. | Variable savings on docs/API/compat PRs. |
| `performance.yml` | Keep benchmark compile smoke; label-gate comparisons. | Avoids up to ~35 LEM unless performance proof is requested. |
| ts-bridge workflows | Consolidate smoke and move parity off default PR. | Removes duplicate smoke/parity spend on ordinary PRs. |
| API/SemVer checks | Route by public API risk paths or labels. | Avoids API checks on docs/fixture/test-only PRs. |
| Main-push CI | Split main smoke from nightly deep verification. | Prevents `main` from becoming the new cost sink. |

### Deferred until actuals

| Item | Gate before implementation |
| --- | --- |
| Learned lane estimates | Enough `ci-actuals.json` receipts to compute stable p50/p90/p95. |
| Lane budget ratchet | Owner review plus enough measured samples; do not lower below p90 without reason. |

## Implementation PR queue

These are intentionally small single-responsibility PRs. Do not combine docs,
branch protection, and workflow pruning in one PR.

| # | Title | Status | Acceptance |
| --- | --- | --- | --- |
| 1 | `docs(ci): reconcile CI economics rollout status` | this PR | `cargo xtask check-ci-lane-whitelist --mode advisory || true`; `cargo xtask policy-report || true`; `git diff --check` |
| 2 | `ci(labels): add CI routing labels` | ⏳ | `git diff --check` |
| 3 | `ci: require PR Gate Success as the stable merge gate` | ⏳ | `git diff --check`; only after PR Gate stability evidence |
| 4 | `ci: stop running legacy ci.yml on ordinary PRs` | ⏳ | `cargo xtask check-ci-lane-whitelist --mode advisory || true`; `git diff --check` |
| 5 | `ci(test-policy): split PR smoke from full inventory enforcement` | ⏳ | `cargo xtask test-policy-smoke`; `cargo xtask test-policy-full || true`; whitelist lint; `git diff --check` |
| 6 | `ci(pure-rust): make platform proof label and main only` | ⏳ | whitelist lint; `git diff --check` |
| 7 | `ci(microcrate): route docs wasm and strict features off ordinary PRs` | ⏳ | whitelist lint; `git diff --check` |
| 8 | `ci(perf): keep benchmark smoke default but label-gate comparisons` | ⏳ | whitelist lint; `git diff --check` |
| 9 | `ci(ts-bridge): consolidate smoke and move parity off default PR` | ⏳ | whitelist lint; `git diff --check` |
| 10 | `ci(api): route public API checks by API risk` | ⏳ | semver/public-api commands may run advisory; `git diff --check` |
| 11 | `ci(policy): block undeclared workflow lanes` | ⏳ | `cargo xtask check-ci-lane-whitelist --mode blocking`; `git diff --check` |
| 12 | `ci(plan): enforce static LEM ceilings with override labels` | ⏳ | xtask and Python PR-plan commands; `git diff --check` |
| 13 | `ci: split main smoke from nightly deep verification` | ⏳ | whitelist lint; `git diff --check` |
| 14 | `ci(actuals): normalize per-lane LEM receipts` | ⏳ | `python3 scripts/ci/emit-ci-actuals.py`; `python3 -m json.tool target/ci/ci-actuals.json`; `git diff --check` |
| 15 | `ci(metrics): compute learned lane estimates from actuals` | ⏸ deferred until actuals | Advisory-only learned estimates; no blocking from learned data. |
| 16 | `ci(metrics): update lane LEM baselines from measured actuals` | ⏸ deferred until actuals | Enough samples, owner signoff, docs and ledger updated together. |

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
8. If a job is `duplicate_of` another lane, justify it or remove/reroute it.
9. If a lane is expensive and `default_pr = true`, it needs an exception and expiry.
10. After each merge, refresh `main` and rerun the lane whitelist check.

## Rollback paths

| Layer | Rollback |
| --- | --- |
| docs | Revert the docs PR. |
| settings labels | Remove the labels from `.github/settings.yml`. |
| branch protection | Restore `CI / ci-supported` as the required context. |
| legacy `ci.yml` trigger pruning | Restore the `pull_request` trigger. |
| lane whitelist | Revert policy TOML changes; advisory lints stop using the new metadata. |
| PR Plan enforcement | Disable enforcement and return to warnings-only. |
| expensive lane routing | Remove label/path conditions and return to previous trigger shape. |
| actuals / learned estimates | Fall back to static `base_lem` in the lane whitelist. |
