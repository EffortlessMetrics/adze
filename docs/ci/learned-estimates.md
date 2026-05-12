# Learned LEM estimates

Static `base_lem` values in `policy/ci-lane-whitelist.toml` are the source of
truth until enough CI actuals exist. Learned estimates are intentionally
advisory at first: they help owners recalibrate lane costs, but they do not
block PRs until the measured-data model is proven.

## Rollout phases

| PR | Title | Purpose | Blocking? |
| --- | --- | --- | --- |
| 14 | `ci(actuals): normalize per-lane LEM receipts` | Emit consistent per-lane receipts from PR Gate / routed workflows. | no |
| 15 | `ci(metrics): compute learned lane estimates from actuals` | Generate advisory p50/p90/p95 estimates from enough samples. | no |
| 16 | `ci(metrics): update lane LEM baselines from measured actuals` | Ratchet static lane ledger values after owner review. | ledger review only |

## Receipt schema target

`ci-actuals.json` should normalize each lane into the same shape:

```json
{
  "schema_version": 1,
  "pr": 123,
  "head_sha": "...",
  "lanes": [
    {
      "id": "ci-supported",
      "workflow": "PR Gate",
      "job": "Supported Rust Gate",
      "runner": "ubuntu_latest",
      "wall_minutes": 21.4,
      "runner_multiplier": 1.0,
      "lem": 21.4,
      "selected_by": ["default_pr"],
      "result": "success"
    }
  ],
  "total_lem": 27.2,
  "budget_band": "ordinary"
}
```

The receipt records what actually ran, why it was selected, and how its runner
multiplier converted wall minutes into LEM.

## Promotion criteria for learned estimates

PR 15 is opened only after there are enough samples to keep learned data from
being noisier than static estimates. A lane is eligible when:

- enough `ci-actuals.json` artifacts exist for the lane (target: at least 30
  days or an operator-approved representative sample);
- p50, p90, and p95 are stable enough to compare across recent windows;
- new lanes still fall back to static `base_lem`;
- macOS and Windows multipliers remain explicit rather than inferred away.

## Advisory model

For each lane, compute and publish:

```text
p50 LEM
p90 LEM
p95 LEM
sample count
last seen
outliers
```

A planner may display:

```text
estimate(lane) = max(static_floor(lane), p50_recent_actual(lane) * 1.15)
warning(lane)  = p90_recent_actual(lane)
hard(lane)     = p95_recent_actual(lane)
```

But static estimates remain authoritative until the ratchet PR updates the
ledger.

## Ratchet rules

PR 16 updates `policy/ci-lane-whitelist.toml` and docs together. It must:

- require enough samples for each updated lane;
- avoid lowering estimates below p90 without a written reason;
- keep macOS (`10x`) and Windows (`2x`) multipliers explicit;
- require owner signoff for expensive default lanes;
- update any exception expiry or duplicate-lane metadata affected by the new
  baseline;
- preserve the override semantics for `full-ci` and `ci-budget-override`.

## Why enforcement waits

Learned estimates are a product metric before they are a policy gate. Enforcing
from sparse data can underprice new lanes, overreact to temporary CI slowness,
or hide the deliberate safety margin in `base_lem`. Static estimates are the
correct enforcement input until receipts are representative.
