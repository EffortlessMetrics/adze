# Learned LEM estimates

Static `base_lem` values in `policy/ci-lane-whitelist.toml` are the source of
truth until enough `ci-actuals.json` receipts exist. Learned estimates are a
future advisory calibration layer; they must not become the first enforcement
mechanism.

## Deferral rule

Do not use learned estimates for blocking decisions until both are true:

1. at least 30 days of receipts, or an equivalent sample size, exists for the
   lanes being calibrated; and
2. the per-lane p50/p90/p95 values are stable enough that updating static
   estimates will not hide real cost.

Static estimates and override labels (`full-ci`, `ci-budget-override`) remain
correct during the rollout.

## Receipt schema target

The actuals receipt should normalize each selected lane before learned estimates
consume it:

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

## Metrics to compute

For each lane, learned estimates should report:

| Metric | Use |
| --- | --- |
| p50 LEM | Typical observed cost. |
| p90 LEM | Budget baseline candidate; do not lower static `base_lem` below this without a written reason. |
| p95 LEM | Outlier / hard-warning signal. |
| sample count | Determines whether the estimate is mature enough to use. |
| last seen | Detects stale lanes. |
| outliers | Explains exceptional receipts that should not silently ratchet budgets. |

## Advisory model

When enough samples exist:

```text
estimate(lane) = max(static_floor(lane), p50_recent_actual(lane) * 1.15)
warning(lane)  = p90_recent_actual(lane)
hard(lane)     = p95_recent_actual(lane)
```

`static_floor` is the lane's `base_lem`. The 15% buffer keeps planning slightly
pessimistic so ordinary PR actuals usually land below the forecast.

## Ratchet rules

Static ledger updates are a separate PR after learned estimates have been
computed:

- require enough samples for the lane,
- do not lower estimates below p90 without a written reason,
- require owner signoff for expensive default lanes,
- keep Linux/windows/macOS multipliers explicit,
- update docs and `policy/ci-lane-whitelist.toml` together,
- keep learned estimates advisory until the updated static ledger has reviewed
  budgets.

## Why this is deferred

New or rarely selected lanes have sparse data. If learned estimates are enforced
too early, they either underestimate new lanes or make PR planning depend on
history that is not available. The current rollout therefore enforces static
bands first and uses actuals only after receipts are normalized and sampled.
