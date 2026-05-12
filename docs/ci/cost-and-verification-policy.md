# CI cost and verification policy

This document is the doctrine anchor for the adze CI economics rollout.
It is referenced by every workflow, every policy ledger entry, and every
follow-up PR in the rollout.

## Why this exists

We are not reducing CI because we want less verification.

Adze needs *more* verification than traditional PR workflows can economically
support: parser correctness, GLR behavior, generated tables, typed extraction,
grammar parity, product proof, WASM, feature compatibility, benchmarks, and
release/API stability.

The problem is not verification. The problem is verification economics.

At high agentic PR volume, broad defaults become the product's operating cost.
Adze targets a different model: Rust-native checks, cheap oracle-gap detection
with `ripr`, LEM visibility, and risk-routed deep lanes.

## The unit: LEM

`LEM = wall-clock job minutes × runner multiplier`

Linux is the unit (`1.0`). Windows costs `2x`, macOS costs `10x`, and some
external services (Docker build farms, AI review) carry their own multipliers.

| Band | LEM | Behavior |
| --- | --- | --- |
| ordinary | 0–35 | green; preferred default <25 |
| elevated | 36–75 | warning; explicit risk surface |
| high | 76–125 | high warning; explicit label/ack |
| over ceiling | >125 | fails unless `full-ci` or `ci-budget-override` |

The target is sub-`$0.50` ordinary PRs when possible. `$1` is a ceiling, not
the design center.

## What gets verified, where

Every PR gets one cheap required proof. Everything else is advisory, routed,
scheduled, manual, release-only, or label-triggered.

| Tier | Trigger | Examples |
| --- | --- | --- |
| frontdoor | every PR, blocking | PR Gate Success, Supported Rust Gate or Docs Gate |
| advisory | every PR, non-blocking | PR Plan, ripr, CI lane whitelist, lightweight policy smoke |
| risk-routed | risk pack or label matches | parser fuzz build, golden, microcrate group, API/SemVer |
| main smoke | push / merge queue | Linux supported proof plus product-stable smoke |
| deep | nightly, scheduled, label, manual | OS matrix, fuzz runtime, full coverage, full benchmarks |
| release | tag, manual | semver, MSRV, security audit, publish validation |

The ordinary default PR shape is expected to stay at or below 25-35 LEM:

| Lane | Blocking | Target LEM |
| --- | ---: | ---: |
| PR Plan | no | ~1 |
| Supported Rust Gate | yes | ~18-22 |
| PR Gate Success | yes | ~1 |
| CI lane whitelist | advisory | ~1-2 |
| ripr advisory | advisory | ~3-5 |
| lightweight test-policy smoke | advisory or blocking | ~1-3 |

## How we get there

The rollout is not "delete CI". It is, in order:

1. Stabilize the docs/control plane.
2. Make PR Gate authoritative.
3. Remove duplicate ordinary PR execution.
4. Route expensive lanes harder.
5. Enforce lane metadata.
6. Add labels and branch-protection rails.
7. Collect actuals.
8. Ratchet budgets from measured data.

The first implementation wave is deliberately documentation/specification heavy:
it reconciles the current state, records which rails are already present, and
separates hardening, pruning, and actuals-dependent work before workflow surgery
begins. See `docs/ci/adze-rollout-plan.md` for the per-PR queue.

## What we will not do

- Weaken the supported product proof lane (`just ci-supported`).
- Make `ripr` blocking.
- Enforce learned LEM budgets before actuals exist.
- Combine docs, branch protection, and large workflow pruning into a single PR.
- Add macOS or Windows as ordinary PR defaults.
- Make raw matrix leaves required branch-protection checks.
- Remove broad validation from `main`/nightly/label/manual/release paths.

## Related

- `docs/ci/lem-budgeting.md` – how LEM is computed and budgeted
- `docs/ci/verification-ladder.md` – tiers and what they prove
- `docs/ci/adze-rollout-plan.md` – per-PR rollout plan and status
- `docs/ci/labels.md` – label vocabulary used by routing
- `policy/ci-lane-whitelist.toml` – lane registry
- `policy/ci-risk-packs.toml` – risk pack routing map
