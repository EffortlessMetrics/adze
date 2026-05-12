# CI Economics Rollout Status

Last review: 2026-05-12.

This is the agent-readable status snapshot for the CI economics rollout. The
implementation sequence and per-PR contract live in
`docs/ci/adze-rollout-plan.md`.

## Status legend

- ✅ already present — rail exists in this repository.
- 🟨 needs hardening — rail exists but must be made authoritative or enforced.
- 🧹 needs pruning — rail exists but is too broad or duplicative for ordinary PRs.
- ⏸ deferred until actuals — wait for enough `ci-actuals.json` receipts.

## Already present

| Rail | Source | Notes |
| --- | --- | --- |
| CI cost doctrine | `docs/ci/cost-and-verification-policy.md` | LEM model, cost bands, verification ladder. |
| Lane whitelist | `policy/ci-lane-whitelist.toml` | Owner/cost/evidence/duplicate metadata for workflow jobs. |
| Risk packs | `policy/ci-risk-packs.toml` | Path and label vocabulary for routed lanes. |
| PR Plan | `.github/workflows/pr-plan.yml` | Computes changed surfaces, selected lanes, estimated LEM, and budget band. |
| PR Gate Success | `.github/workflows/pr-gate.yml` | Aggregates Supported Rust Gate or Docs Gate behind one stable check. |
| CI policy workflow | `.github/workflows/ci-policy.yml` | Runs lane whitelist lint in advisory mode. |
| ci-actuals scaffold | `scripts/ci/emit-ci-actuals.py` | Emits `ci-actuals.json`; schema still needs normalization. |
| Non-default coverage | `.github/workflows/coverage.yml` | Coverage is label/main/manual, not an ordinary default. |

## Needs hardening

| Work | Why | Target PR |
| --- | --- | --- |
| Reconcile docs/control plane | Agents need one current truth table and stable sequence. | PR 1 |
| Add routing labels to repo settings | Labels are the operator interface for expensive verification. | PR 2 |
| Promote PR Gate Success | Branch protection should require the aggregate, not a raw legacy job. | PR 3 |
| Make lane whitelist blocking for workflow changes | Prevent new undeclared lanes and expensive defaults from landing silently. | PR 11 |
| Enforce static budget ceilings | Over-ceiling plans should fail unless override labels are present. | PR 12 |
| Normalize ci-actuals receipts | Learned estimates need lane-level receipts. | PR 14 |

## Needs pruning

| Lane family | Current issue | Target PR |
| --- | --- | --- |
| Legacy `ci.yml` | Still has ordinary PR trigger and duplicates supported proof. | PR 4 |
| Test policy | Full inventory enforcement is too expensive for every PR. | PR 5 |
| Pure Rust / OS matrix | Ubuntu/stable PR default duplicates supported proof; macOS/windows must be non-default. | PR 6 |
| Microcrate CI | Path routing exists, but docs/WASM/strict feature checks need harder labels/main/manual routing. | PR 7 |
| Performance comparison | Expensive comparison should not be a default PR lane. | PR 8 |
| ts-bridge smoke/parity | Duplicate smokes and default parity should be consolidated/rerouted. | PR 9 |
| API/SemVer | Should run on API-risk paths or API/release labels, not docs/fixture-only PRs. | PR 10 |
| Main push deep verification | Moving everything to main would become the next cost sink. | PR 13 |

## Deferred until actuals

| Work | Criteria |
| --- | --- |
| Learned estimates | >=30 days or enough normalized per-lane receipts; advisory only. |
| Ratchet static lane costs | Enough samples, p90-based updates, owner signoff for expensive lanes. |

## Current default PR gap

The target ordinary PR shape is <=25 LEM preferred and <=35 LEM ceiling. Current
ordinary PR cost can still exceed that because several useful but duplicative
lanes remain default or broad-path PR lanes: full test policy (~12 LEM),
pure-rust ubuntu/stable (~18 LEM), microcrate CI (variable), ts-bridge lanes,
and performance/API signals on some surfaces.

The pruning PRs must preserve deep verification through routed, advisory,
scheduled, `main`, release, and manual paths while removing those costs from the
ordinary default.
