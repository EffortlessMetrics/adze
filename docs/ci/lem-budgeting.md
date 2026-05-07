# LEM budgeting

## Definition

```
LEM = wall-clock job minutes × runner multiplier
```

LEM is the unit of CI cost we report. It is intentionally coarse: a single,
linear number that any reviewer can compare against the budget bands.

## Runner multipliers

| Runner | Multiplier | Notes |
| --- | --- | --- |
| `ubuntu-latest` / `ubuntu-22.04` | 1.0 | the unit |
| `windows-latest` | 2.0 | doubles every minute |
| `macos-latest` | 10.0 | use sparingly, rarely on PR |
| docker build farm | 6.0 | only when truly required |
| external AI review | 4.0 | external service multiplier |

Multipliers live in `policy/ci-lane-whitelist.toml` under `[runner_multipliers]`.

## Budget bands

| Band | LEM | PR behavior |
| --- | --- | --- |
| ordinary | 0–35 | green; preferred default <25 |
| elevated | 36–75 | warning; reviewer should see why |
| high | 76–125 | warning, label-ack expected |
| over ceiling | >125 | fails unless `full-ci` or `ci-budget-override` |

Budget bands live in `policy/ci-lane-whitelist.toml` under `[budget]`.

## Estimation

A PR's estimated LEM is the sum of `base_lem` for every selected lane, scaled
by runner multiplier when the lane runs on a non-Linux runner. Lanes that the
PR Plan deselects (because no risk pack matched) contribute 0.

Estimates start from `base_lem` in the whitelist. Once `target/ci/ci-actuals.json`
artifacts have accumulated, learned estimates replace static numbers via:

```
estimate = max(static_floor, p50_recent_actual × 1.15)
warning  = p90_recent_actual
hard     = p95_recent_actual
```

Static fallback is always available; learned estimates never raise the hard
ceiling without an explicit policy update.

## Soft enforcement

`xtask ci plan` (and the PR Plan workflow) emits warnings into
`ci-plan.json` and the GitHub step summary based on the budget band:

| Band | Behavior |
| --- | --- |
| ordinary | no warning |
| elevated | warning unless `ci-budget-ack` is present |
| high | warning suggesting `ci-budget-ack` |
| over-ceiling | warning unless `ci-budget-override` or `full-ci` is present |

Hard enforcement is opt-in via `--enforce-hard-ceiling`. When that flag is
passed, the planner exits non-zero if the plan exceeds the hard ceiling
without `full-ci` or `ci-budget-override`. Today the workflow does **not**
pass `--enforce-hard-ceiling`; soft warnings only.

PR 17 of the rollout is the dedicated change that promotes hard enforcement
once actuals confirm the band thresholds.

## Ack and override labels

| Label | Effect |
| --- | --- |
| `ci-budget-ack` | acknowledge elevated/high LEM |
| `ci-budget-override` | allow >125 LEM |
| `full-ci` | run all heavy lanes; implies budget override |
| `ci:perf` / `ci:golden` / `ci:microcrate` / `ci:concurrency` | risk-pack opt-in |
| `platform-matrix` | run full OS/toolchain matrix |
| `fuzz` | run fuzz runtime on this PR |
| `coverage` | run coverage instrumentation |
| `wasm` | run wasm-check |
| `security-audit` | run security audit on this PR |
| `release-check` | run release-gate verification |

See `docs/ci/labels.md` for the full label vocabulary.

## What LEM is not

LEM is not dollars. It is a proxy for dollars that is stable across runner
price changes and across self-hosted vs cloud. To convert, multiply by the
operator's effective per-minute Linux cost.
