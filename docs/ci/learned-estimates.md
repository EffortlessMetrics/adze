# Learned LEM estimates

Static `base_lem` values in `policy/ci-lane-whitelist.toml` are the current
source of truth. Learned estimates are intentionally deferred until actual CI
receipts exist in enough volume to make percentiles meaningful.

## Deferral rule

Do not use learned estimates for gating until at least one of these is true:

- >=30 days of `ci-actuals.json` artifacts are available, or
- each lane being learned has enough samples to produce stable p50, p90, and
  p95 values across two consecutive review windows.

Until then, PR Plan and the lane whitelist must use static estimates.

## Receipt schema target

The actuals pipeline should normalize receipts to this shape:

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

## Advisory model

Once receipts exist, compute and publish advisory lane summaries with:

| Field | Meaning |
| --- | --- |
| `p50_lem` | median observed LEM for the lane |
| `p90_lem` | conservative planning estimate candidate |
| `p95_lem` | hard-warning candidate |
| `sample_count` | number of usable receipts |
| `last_seen` | newest receipt timestamp |
| `outliers` | receipts excluded or flagged for review |

An implementation may use this formula for advisory estimates:

```text
estimate(lane) = max(static_floor(lane), p50_recent_actual(lane) * 1.15)
warning(lane)  = p90_recent_actual(lane)
hard(lane)     = p95_recent_actual(lane)
```

`static_floor` is the lane's `base_lem`. Learned estimates must not silently
lower an expensive default lane below p90 without an owner-approved reason.

## Ratchet rules

When enough samples exist, update `policy/ci-lane-whitelist.toml` only if:

1. the lane has enough samples for stable p90/p95,
2. macOS and Windows multipliers remain explicit,
3. expensive default lanes have owner signoff,
4. docs and ledger changes land together, and
5. rollback is to restore the previous static `base_lem`.

Learned estimates inform the ledger; they do not replace the ledger audit trail.
