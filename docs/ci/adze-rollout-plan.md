# Adze CI economics rollout plan

This document is the authoritative implementation map for the CI economics
rollout. Treat the rollout as **CI infrastructure product work**, not workflow
cleanup: every change must preserve proof quality while making the default PR
bill predictable.

Core rule:

> Every PR gets one cheap required proof. Everything else is advisory, routed,
> scheduled, manual, release-only, or label-triggered.

## Targets

| Target | LEM | Notes |
| --- | ---: | --- |
| Preferred ordinary PR | <=25 | Design center for routine code and docs changes. |
| Ordinary ceiling | <=35 | Above this, explain the risk surface and selected lanes. |
| Elevated PR | 36-75 | Warn and recommend labels / explicit acknowledgement. |
| High-cost PR | 76-125 | Warning plus explicit label recommendation. |
| Over ceiling | >125 | Fail unless `full-ci` or `ci-budget-override` is present. |

Linux is `1.0` LEM, Windows is `2.0`, and macOS is `10.0`. macOS and
Windows must not be ordinary PR defaults.

## Target default PR shape

### Ordinary Rust PR

| Lane | Blocking | Target LEM | Notes |
| --- | ---: | ---: | --- |
| PR Plan | no | ~1 | Changed surfaces, selected lanes, budget band. |
| Supported Rust Gate | yes | ~18-22 | Runs `just ci-supported`. |
| PR Gate Success | yes | ~1 | Stable required aggregate. |
| CI lane whitelist | advisory | ~1-2 | Prevents workflow sprawl. |
| ripr advisory | advisory | ~3-5 | Static oracle-gap signal; skip docs-only. |
| Test-policy smoke | advisory or blocking | ~1-3 | Disabled-test / hygiene smoke only. |

### Docs-only PR

| Lane | Blocking | Target LEM | Notes |
| --- | ---: | ---: | --- |
| PR Plan | no | ~1 | Classifies docs-only safely. |
| Docs Gate | yes | ~1-2 | Cheap formatting/doc hygiene proof. |
| PR Gate Success | yes | ~1 | Stable required aggregate. |
| Policy lint | advisory | ~1 | Optional workflow/policy hygiene. |

Everything else is non-default: routed, advisory, scheduled, manual,
release-only, or label-triggered.

## Status legend

- ✅ **already present** — rail exists in this repository.
- 🟨 **needs hardening** — rail exists but needs stricter metadata, enforcement,
  or branch-protection coordination.
- 🧹 **needs pruning** — rail exists but is still too broad or duplicative for
  ordinary PRs.
- ⏸ **deferred until actuals** — do not implement until enough `ci-actuals.json`
  receipts exist.

## Current control-plane inventory

| Rail | Status | Source of truth | Notes |
| --- | --- | --- | --- |
| LEM doctrine and budget bands | ✅ already present | `docs/ci/cost-and-verification-policy.md` | Preferred <=25 LEM, ordinary ceiling <=35 LEM. |
| Lane whitelist ledger | 🟨 needs hardening | `policy/ci-lane-whitelist.toml` | Advisory today; should become blocking only for workflow/policy changes. |
| Risk-pack vocabulary | ✅ already present | `policy/ci-risk-packs.toml` | Path/label vocabulary for expensive lanes. |
| PR Plan | 🟨 needs hardening | `.github/workflows/pr-plan.yml` | Static estimates exist; hard ceiling enforcement must remain override-label based. |
| PR Gate Success | 🟨 needs hardening | `.github/workflows/pr-gate.yml` | Aggregator exists; branch protection still requires legacy `CI / ci-supported`. |
| ci-actuals receipt | 🟨 needs hardening | `scripts/ci/emit-ci-actuals.py` | Receipt exists; normalize per-lane schema before learned estimates. |
| Coverage routing | ✅ already present | `.github/workflows/coverage.yml` | Non-default label/main/manual coverage lane. |
| Routing label vocabulary | 🟨 needs hardening | `docs/ci/labels.md`, `.github/settings.yml` | Settings must contain all labels used by risk packs and workflows. |

## Next implementation wave

The next wave replaces the older PR-numbered rollout stack with small,
single-intention PRs. Do not combine branch protection changes with workflow
pruning.

| Wave PR | Title | Status | Scope | Acceptance |
| ---: | --- | --- | --- | --- |
| 1 | `docs(ci): reconcile CI economics rollout status` | 🟨 needs hardening | Normalize this doc set and the lane ledger narrative. | `cargo xtask check-ci-lane-whitelist --mode advisory || true`; `cargo xtask policy-report || true`; `git diff --check`. |
| 2 | `ci(labels): add CI routing labels` | 🟨 needs hardening | Add all routing labels to `.github/settings.yml`. | `git diff --check`. |
| 3 | `ci: require PR Gate Success as the stable merge gate` | 🟨 needs hardening | Switch branch protection to `PR Gate / PR Gate Success` only after stability criteria pass. | `git diff --check`. |
| 4 | `ci: stop running legacy ci.yml on ordinary PRs` | 🧹 needs pruning | Remove ordinary `pull_request` execution from legacy `ci.yml`; keep push/schedule/manual. | `cargo xtask check-ci-lane-whitelist --mode advisory || true`; `git diff --check`. |
| 5 | `ci(test-policy): split PR smoke from full inventory enforcement` | 🧹 needs pruning | Create `test-policy-smoke` for every PR; move full inventory to main/nightly/manual/`full-ci`. | `cargo xtask test-policy-smoke`; `cargo xtask test-policy-full || true`; whitelist advisory; diff check. |
| 6 | `ci(pure-rust): make platform proof label and main only` | 🧹 needs pruning | PR execution only for `platform-matrix`, `pure-rust`, or `full-ci`; keep main/schedule/manual. | Whitelist advisory; diff check. |
| 7 | `ci(microcrate): route docs wasm and strict features off ordinary PRs` | 🧹 needs pruning | Keep changed crate groups; move docs/WASM/strict features to labels/main/manual. | Whitelist advisory; diff check. |
| 8 | `ci(perf): keep benchmark smoke default but label-gate comparisons` | 🧹 needs pruning | Keep compile smoke path-routed; move comparisons to `ci:perf`/`benchmarks`/`full-ci`/main/manual. | Whitelist advisory; diff check. |
| 9 | `ci(ts-bridge): consolidate smoke and move parity off default PR` | 🧹 needs pruning | Keep one path-routed smoke; move parity to main/schedule/manual/`ts-bridge`/`full-ci`. | Whitelist advisory; diff check. |
| 10 | `ci(api): route public API checks by API risk` | 🧹 needs pruning | Run SemVer/public API checks only on API-risk paths or labels. | `cargo semver-checks check-release -p adze || true`; `cargo public-api -p adze || true`; diff check. |
| 11 | `ci(policy): block undeclared workflow lanes` | 🟨 needs hardening | Make whitelist blocking only when workflows/policy/docs/scripts/xtask CI files change. | `cargo xtask check-ci-lane-whitelist --mode blocking`; diff check. |
| 12 | `ci(plan): enforce static LEM ceilings with override labels` | 🟨 needs hardening | Static budget warnings/failures; no learned estimates yet. | `cargo run -q -p xtask -- ci-plan --base origin/main --head HEAD --json-out target/ci/ci-plan.json`; fallback script; diff check. |
| 13 | `ci: split main smoke from nightly deep verification` | 🧹 needs pruning | Prevent push-to-main from becoming the cost sink; no macOS/windows on every main push. | Whitelist advisory; diff check. |
| 14 | `ci(actuals): normalize per-lane LEM receipts` | 🟨 needs hardening | Emit normalized per-lane actuals schema. | `python3 scripts/ci/emit-ci-actuals.py`; `python3 -m json.tool target/ci/ci-actuals.json`; diff check. |
| 15 | `ci(metrics): compute learned lane estimates from actuals` | ⏸ deferred until actuals | Advisory learned estimates after sufficient samples. | Generated estimates are advisory only. |
| 16 | `ci(metrics): update lane LEM baselines from measured actuals` | ⏸ deferred until actuals | Ratchet static `base_lem` from measured p90 with owner signoff. | Docs and ledger update together. |

## Default PR pruning decisions

| Lane family | Current concern | Target behavior |
| --- | --- | --- |
| Legacy `ci.yml` | Duplicates supported proof on ordinary PRs. | Push/schedule/manual only after PR Gate is required. |
| Test policy | Full inventory is useful but too expensive as an always-on PR lane. | Cheap PR smoke; full policy on main/nightly/manual/`full-ci`. |
| Pure Rust / OS matrix | Ubuntu/stable still duplicates supported proof; macOS/windows are high-multiplier. | Label/main/manual/scheduled only; no macOS/windows ordinary PR defaults. |
| Microcrate CI | Path routing exists but docs/WASM/strict feature work remains expensive. | Changed crate groups only by default; docs/WASM/strict features by label/main/manual. |
| Performance | Comparisons are expensive and duplicate benchmark workflows. | Compile smoke only by default and only on benchmark paths; comparisons by label/main/manual. |
| ts-bridge | Two smoke workflows and parity overlap. | One path-routed smoke; parity by label/main/schedule/manual. |
| API/SemVer | Valuable release signal, not useful for docs/fixture-only PRs. | API-risk paths or API/release labels; advisory until release prep. |

## Branch-protection migration

The required context is still `CI / ci-supported`. The target context is
`PR Gate / PR Gate Success`, but the migration must be a dedicated PR after the
criteria in `docs/ci/branch-protection.md` pass. Never require a raw matrix leaf
as a branch-protection context.

Rollback for the migration is to restore `CI / ci-supported` as the required
context in `.github/settings.yml`.

## Per-PR CI economics contract

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
| Docs/specs | Revert the PR. |
| Label registry | Revert `.github/settings.yml` label additions; workflows using absent labels remain safe but less discoverable. |
| Policy TOML | Revert the lane entry or exception; advisory lints stop using it. |
| PR Plan budget enforcement | Disable hard-ceiling mode or revert to warnings. |
| PR Gate branch protection | Restore `CI / ci-supported`. |
| Legacy `ci.yml` PR pruning | Restore the `pull_request` trigger. |
| Expensive lane routing | Restore previous triggers; deep lanes remain available throughout. |
| Learned estimates | Fall back to static `base_lem`. |
