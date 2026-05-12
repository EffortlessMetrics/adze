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

| Tier | Trigger | Examples |
| --- | --- | --- |
| frontdoor | every PR, blocking | `just ci-supported`, PR Gate Success after promotion |
| advisory | every PR, non-blocking | PR Plan, ripr, CI lane whitelist |
| risk-routed | risk pack or label matches | test-policy full, parser fuzz build, golden, microcrate group, API/SemVer |
| deep | `main`, nightly, label | OS matrix, fuzz runtime, full benchmarks |
| release | tag, manual | semver, MSRV, security audit |

## Implementation order

The rollout is not "delete CI". It proceeds in this control-plane order:

1. Stabilize docs/control plane.
2. Make PR Gate authoritative.
3. Remove duplicate ordinary-PR execution.
4. Route expensive lanes harder.
5. Enforce lane metadata.
6. Add labels and branch-protection rails.
7. Collect actuals.
8. Ratchet budgets from measured data.

The next implementation wave is specified in
`docs/ci/implementation-sequence.md`. `docs/ci/adze-rollout-plan.md` is the
current status ledger.

## What we will not do

- Weaken the supported product proof lane (`just ci-supported`).
- Make `ripr` blocking.
- Enforce learned LEM budgets before actuals exist.
- Combine docs, branch protection, and workflow pruning into a single PR.
- Remove broad validation from `main`/nightly/manual/label/release paths.
- Add macOS or Windows as ordinary default PR lanes.

## Related

- `docs/ci/lem-budgeting.md` – how LEM is computed and budgeted
- `docs/ci/verification-ladder.md` – tiers and what they prove
- `docs/ci/adze-rollout-plan.md` – rollout status ledger
- `docs/ci/implementation-sequence.md` – ordered implementation contract
- `docs/ci/labels.md` – label vocabulary used by routing
- `policy/ci-lane-whitelist.toml` – lane registry
- `policy/ci-risk-packs.toml` – risk pack routing map
