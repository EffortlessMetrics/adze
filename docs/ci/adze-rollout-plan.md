# Adze CI economics rollout plan

This document is the status ledger for the CI economics rollout. The detailed
agent contract and acceptance commands live in
`docs/ci/implementation-sequence.md`.

The rollout is CI infrastructure product work, not workflow cleanup. The
product invariant is:

> Every PR gets one cheap required proof. Everything else is advisory,
> routed, scheduled, manual, release-only, or label-triggered.

## Budget target

| Rule | Value |
| --- | ---: |
| Preferred ordinary PR | <=25 LEM |
| Ordinary PR ceiling | <=35 LEM |
| Elevated PR | 36–75 LEM, explicit risk surface |
| High-cost PR | 76–125 LEM, explicit label/ack |
| Over ceiling | >125 LEM, requires `full-ci` or `ci-budget-override` |

Linux is the unit runner (`1.0`), Windows is `2.0`, and macOS is `10.0`.
Windows and macOS must not be ordinary PR defaults.

## Status legend

- ✅ landed — present on `main` and represented in the current control plane
- 🟡 needs hardening — present, but still too broad, advisory-only, or waiting
  for a dedicated hardening PR
- 🔴 needs pruning — duplicate or too expensive for ordinary PR defaults
- ⏳ planned — not yet started
- ⏸ deferred — intentionally waiting on actuals or operator coordination

## Already present

| Rail | Status | Notes |
| --- | --- | --- |
| LEM doctrine and bands | ✅ | `docs/ci/cost-and-verification-policy.md` defines LEM, runner multipliers, and the static budget bands. |
| Lane registry | ✅ | `policy/ci-lane-whitelist.toml` records lane owner, cost, proof obligation, evidence, duplicate map, and review dates. |
| Risk-pack vocabulary | ✅ | `policy/ci-risk-packs.toml` maps paths and labels to risk-routed verification. |
| PR Plan | ✅ | `.github/workflows/pr-plan.yml` forecasts changed surfaces, selected lanes, and budget band. |
| PR Gate workflow | ✅ | `.github/workflows/pr-gate.yml` contains Supported Rust Gate, Docs Gate, and PR Gate Success. |
| CI actuals scaffold | ✅ | `ci-actuals.json` emission exists and is the input to future learned estimates. |
| Coverage routing | ✅ | Coverage is non-default and selected by label, main, or manual dispatch. |

## Needs hardening

| Rail | Status | Hardening needed |
| --- | --- | --- |
| Branch protection | 🟡 | Promote the required context from `CI / ci-supported` to `PR Gate / PR Gate Success` only after stability criteria in `docs/ci/branch-protection.md` are met. |
| Lane whitelist lint | 🟡 | Keep advisory generally, but make blocking for workflow/policy/CI-doc changes. |
| PR Plan budgets | 🟡 | Enforce static ceilings before learned estimates: warn for elevated/high and fail over ceiling unless `full-ci` or `ci-budget-override` is present. |
| Test policy | 🟡 | Split cheap PR smoke from full inventory/runtime enforcement. |
| CI labels | 🟡 | Keep repo settings synchronized with `docs/ci/labels.md` and `policy/ci-risk-packs.toml`. |
| CI actuals | 🟡 | Normalize receipts to per-lane wall minutes, multiplier, selected-by reasons, result, and total LEM. |

## Needs pruning

| Lane/workflow | Status | Pruning objective |
| --- | --- | --- |
| Legacy `ci.yml` PR trigger | 🔴 | After PR Gate is required, remove ordinary `pull_request` execution so the old umbrella lane does not duplicate the supported proof. |
| `pure-rust-ci.yml` | 🔴 | Make platform proof label/main/manual/scheduled only; no ordinary PR OS/toolchain matrix. |
| `microcrate-ci.yml` | 🔴 | Keep path-matched crate-group tests; route docs, WASM, formatting, and strict-feature checks away from ordinary PR default. |
| `performance.yml` / `benchmarks.yml` | 🔴 | Keep compile smoke path-routed; label-gate benchmark comparisons and full suites. |
| `smoke-ts-bridge.yml` / `ts-bridge-smoke.yml` / `ts-bridge-parity.yml` | 🔴 | Keep one path-routed smoke lane; move parity to main/schedule/manual/label. |
| API / SemVer checks | 🔴 | Route by public API paths or `api`/`release-check`/`breaking-change`/`full-ci` labels. |

## Deferred until actuals

| Item | Status | Promotion condition |
| --- | --- | --- |
| Learned LEM estimates | ⏸ | At least 30 days or enough per-lane samples with stable p50/p90/p95. |
| Lane ledger ratchet | ⏸ | Update `base_lem` only after measured p90 evidence and owner review. |

## Next implementation wave

The next wave should proceed in this order. See
`docs/ci/implementation-sequence.md` for the per-PR proof obligations and
rollback paths.

| Step | Title | Kind |
| ---: | --- | --- |
| 1 | `docs(ci): reconcile CI economics rollout status` | control plane |
| 2 | `ci(labels): add CI routing labels` | operator interface |
| 3 | `ci: require PR Gate Success as the stable merge gate` | branch protection |
| 4 | `ci: stop running legacy ci.yml on ordinary PRs` | duplicate removal |
| 5 | `ci(test-policy): split PR smoke from full inventory enforcement` | cost reduction |
| 6 | `ci(pure-rust): make platform proof label and main only` | cost reduction |
| 7 | `ci(microcrate): route docs wasm and strict features off ordinary PRs` | cost reduction |
| 8 | `ci(perf): keep benchmark smoke default but label-gate comparisons` | cost reduction |
| 9 | `ci(ts-bridge): consolidate smoke and move parity off default PR` | duplicate removal |
| 10 | `ci(api): route public API checks by API risk` | routing |
| 11 | `ci(policy): block undeclared workflow lanes` | guardrail |
| 12 | `ci(plan): enforce static LEM ceilings with override labels` | guardrail |
| 13 | `ci: split main smoke from nightly deep verification` | main cost control |
| 14 | `ci(actuals): normalize per-lane LEM receipts` | metrics |
| 15 | `ci(metrics): compute learned lane estimates from actuals` | deferred metrics |
| 16 | `ci(metrics): update lane LEM baselines from measured actuals` | deferred ratchet |

## Per-PR contract

Each CI economics PR body must include:

- estimated LEM impact,
- workflows touched,
- default PR effect,
- branch-protection impact,
- rollback path,
- proof obligation,
- cheaper signal considered,
- whether expensive runners were added, and
- confirmation that macOS/windows are not ordinary PR defaults.

## Rollback paths

| Layer | Rollback |
| --- | --- |
| docs | revert the docs-only PR |
| labels | revert `.github/settings.yml` label entries |
| policy TOML | revert the ledger change; advisory lint stops reporting the new rule |
| advisory workflow | disable/delete the workflow or restore previous `if:` guard |
| PR Gate Success promotion | switch required context back to `CI / ci-supported` |
| legacy PR trigger pruning | restore the `pull_request` trigger |
| risk-pack routing | restore old path filters and label conditions |
| budget enforcement | return PR Plan to advisory-only mode |
| actuals / learned estimates | fall back to static `base_lem` |
