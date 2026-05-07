# Adze CI economics rollout plan

This document is the per-PR map for the rollout. It describes what each PR
should do, what should not be combined, and what the rollback path looks like.

The rollout target:

```
ordinary PR:        <35 LEM preferred, usually <$0.50
elevated PR:        35–75 LEM, explicit risk surface
high-cost PR:       75–125 LEM, explicit label/ack
over ceiling:       >125 LEM requires override
```

## Status legend

- ✅ landed — change is in `main`
- 🟡 in flight — open PR or partially merged on the rollout branch
- ⏳ planned — not yet started
- ⏸ deferred — needs actuals data or coordination

## PRs

| # | Title | Status | Notes |
| --- | --- | --- | --- |
| 01 | docs(ci): verification economics policy | 🟡 | This doc set |
| 02 | chore(ci): add CI lane whitelist map | 🟡 | `policy/ci-lane-whitelist.toml` |
| 03 | ci(policy): lint workflows against whitelist | 🟡 | `cargo xtask check-ci-lane-whitelist` |
| 04 | feat(ci): advisory PR Plan with LEM estimates | 🟡 | `.github/workflows/pr-plan.yml` |
| 05 | ci: add PR Gate Success summary | 🟡 | additive only |
| 06 | perf(ci): make PR caches restore-only | ⏳ | one-line per workflow |
| 07 | ci(ripr): advisory static exposure | 🟡 | non-blocking |
| 08 | ci(policy): risk-pack routing map | 🟡 | `policy/ci-risk-packs.toml` |
| 09 | feat(xtask): `xtask ci plan` | 🟡 | testable planner |
| 10 | perf(ci): gate fuzzing to nightly and labels | ⏸ | needs PR Plan actuals |
| 11 | perf(ci): unify benchmark PR ownership | ⏸ | needs PR Plan actuals |
| 12 | perf(ci): route golden tests by grammar risk | ⏸ | needs risk pack data |
| 13 | perf(ci): reduce pure-rust PR matrix | ⏸ | needs `full-ci` adoption |
| 14 | perf(ci): route microcrate CI by risk pack | ⏸ | needs PR Plan integration |
| 15 | ci: emit LEM actuals telemetry | 🟡 | additive only |
| 16 | ci(plan): warn on elevated LEM | 🟡 | soft warnings only |
| 17 | ci: make PR Gate Success the required check | ⏸ | needs branch protection coordination |
| 18 | ci(metrics): learned LEM estimates | ⏸ | needs ≥30 days of actuals |

## What this rollout branch contains

The rollout branch (`claude/adze-ci-economics-rollout-IxzNZ`) contains the
**foundation** — PRs 01–09 plus 15 and 16. Behavior-changing routing PRs
(10–14), branch-protection promotion (17), and learned estimates (18) are
intentionally deferred:

- **10–14** require PR Plan actuals to size the change confidently. They also
  carry the highest risk of dropping coverage on real defects, so they should
  ship as separate, reviewable PRs rather than bundled with policy scaffolding.
- **17** changes branch protection. That is a coordinated change with the
  release/CI owner.
- **18** needs ≥30 days of `ci-actuals.json` data to compute meaningful
  percentiles. Until then, static estimates are correct.

## Per-PR contract

Each PR body in this rollout must include:

- estimated LEM impact
- workflows touched
- default PR effect
- rollback path
- proof obligation
- cheaper signal considered
- branch protection impact (almost always: none, until PR 17)

## Rollback paths

| Layer | Rollback |
| --- | --- |
| docs | revert PR |
| policy TOML | revert PR; advisory lints stop firing |
| advisory workflow (PR Plan, ripr, whitelist lint) | delete the workflow file |
| PR Gate Success | delete the workflow file; old required checks remain |
| cache normalization | revert per-workflow `save-if` lines |
| risk-pack routing | revert path filters; lanes go back to default-on |
| branch protection promotion | switch the required check back |
