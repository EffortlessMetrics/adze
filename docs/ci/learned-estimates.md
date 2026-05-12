# Learned LEM estimates

Static `base_lem` values in `policy/ci-lane-whitelist.toml` are the source of
truth until enough actuals exist. Learned estimates are intentionally deferred
and advisory-only at first; they must not make a new lane cheaper merely
because it has too few samples.

## Inputs

`ci-actuals.json` receipts should converge on this schema before learned
estimates are enabled:

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

Receipts should be emitted per lane, not only per workflow, so duplicate lanes
and routed lanes can be compared against the ledger independently.

## Promotion criteria

Learned estimates may be introduced only after:

- at least 30 days of `ci-actuals.json` artifacts, or enough samples to cover
  the default and routed lanes with stable percentiles,
- each learned lane has sample count, p50, p90, p95, last-seen timestamp, and
  outlier metadata,
- p50 and p90 are stable within ±15% across two consecutive weeks,
- the static budget thresholds (`25` / `35` / `75` / `125`) still match the
  operator cost target after observing actuals, and
- every expensive default lane has owner review before lowering an estimate.

## Advisory model

The first learned-estimate PR only generates advisory artifacts:

- `target/ci/learned-estimates.json`
- `docs/ci/learned-estimates.md` updates or generated tables

The advisory model should track:

| Field | Meaning |
| --- | --- |
| `p50_lem` | Median observed LEM for the lane |
| `p90_lem` | Conservative planning baseline candidate |
| `p95_lem` | Hard-warning candidate |
| `sample_count` | Number of usable receipts |
| `last_seen` | Most recent receipt timestamp |
| `outliers` | Runs excluded or separately reported with reason |

Do not block PRs from learned estimates in the first metrics PR.

## Ratchet rules

When enough samples exist, update `policy/ci-lane-whitelist.toml` and docs
together. The ratchet rules are:

1. Require enough samples for the lane and runner class.
2. Do not lower `base_lem` below observed p90 without an explicit written
   reason.
3. Keep Windows (`2.0`) and macOS (`10.0`) multipliers explicit.
4. Require owner signoff for expensive default lanes.
5. Preserve the static ledger as the auditable source of truth.

## Fallback behavior

If a lane has too few samples, is new, or changed its routing recently, use the
static `base_lem`. Learned data may explain why a future estimate should change;
it does not replace the ledger automatically.
