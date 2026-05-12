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
| ordinary | 0-35 | green; preferred default <=25 |
| elevated | 36-75 | warning summary; explicit risk surface |
| high | 76-125 | high warning plus explicit label recommendation |
| over ceiling | >125 | fail unless `full-ci` or `ci-budget-override` |

The target is sub-`$0.50` ordinary PRs when possible. `$1` is a ceiling, not
the design center.

## What gets verified, where

| Tier | Trigger | Examples |
| --- | --- | --- |
| frontdoor | every PR, blocking | `PR Gate Success`, `just ci-supported` or Docs Gate |
| advisory | every PR, non-blocking | PR Plan, ripr, lane whitelist |
| risk-routed | risk pack matches | parser fuzz build, golden, matching microcrate group |
| deep | `main`, nightly, label, manual | OS matrix, fuzz runtime, full benchmarks, full test-policy inventory |
| release | tag, manual | semver, MSRV, security audit |

The frontdoor target is deliberately narrow. Ordinary Rust PRs should normally
pay for PR Plan (~1 LEM), Supported Rust Gate (~18-22 LEM), PR Gate Success
(~1 LEM), cheap advisory policy signals, and at most a smoke-level test-policy
lane. Docs-only PRs should pay for PR Plan, Docs Gate, PR Gate Success, and
policy lint only. All other proof stays available, but it is selected by path,
label, schedule, `main`, release, or manual dispatch.

## How we get there

The rollout is not "delete CI". It proceeds as infrastructure product work:

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

The already-present rails are the policy docs, lane whitelist, risk packs,
PR Plan, PR Gate Success, and ci-actuals scaffold. The next work is to harden
those rails, prune duplicate ordinary-PR execution, and defer learned estimates
until enough receipts exist. See `docs/ci/adze-rollout-plan.md` for the
single-intention PR sequence.

## What we will not do

- Weaken the supported product proof lane (`just ci-supported`).
- Make `ripr` blocking.
- Enforce learned LEM budgets before actuals exist.
- Combine docs, policy, and routing changes into a single PR.
- Remove broad validation from `main`/nightly/label paths.
- Introduce macOS or Windows as ordinary PR defaults.
- Make a raw matrix leaf the required branch-protection context.

## Related

- `docs/ci/lem-budgeting.md` – how LEM is computed and budgeted
- `docs/ci/verification-ladder.md` – tiers and what they prove
- `docs/ci/adze-rollout-plan.md` – per-PR rollout plan and status
- `docs/ci/labels.md` – label vocabulary used by routing
- `policy/ci-lane-whitelist.toml` – lane registry
- `policy/ci-risk-packs.toml` – risk pack routing map
