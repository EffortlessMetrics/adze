# Learned LEM estimates (PR 18)

Static `base_lem` values in `policy/ci-lane-whitelist.toml` are the
starting estimate. Once `ci-actuals.json` artifacts have accumulated,
the planner can use observed durations instead.

## Promotion criteria

PR 18 is opened when:

- `target/ci/ci-actuals.json` artifacts have been uploaded by ≥ 30 PRs,
- the runner-multiplier × wall-clock time for each lane has a stable
  p50 and p90 within ±15% across two consecutive weeks,
- the band thresholds (`35` / `75` / `125`) still match the operator's
  cost target after observing actuals (see `docs/ci/lem-budgeting.md`).

## Model

```
estimate(lane) = max(static_floor(lane), p50_recent_actual(lane) × 1.15)
warning(lane)  = p90_recent_actual(lane)
hard(lane)     = p95_recent_actual(lane)
```

`static_floor` is the `base_lem` from the whitelist. The `× 1.15` factor
keeps the estimate slightly pessimistic, so a normal PR's actuals stay
under the estimate.

## Implementation outline

PR 18 will:

1. Add a `learned_estimates` section to the planner that loads the
   most recent `ci-actuals.json` artifacts (from the previous N runs on
   `main` or any matching workflow).
2. Replace the static `base_lem` lookup in `xtask ci plan` with
   `max(static_floor, p50_recent × 1.15)`.
3. Continue to fall back to `base_lem` whenever fewer than 5 actuals
   exist for a lane.
4. Add `--enforce-hard-ceiling` to the workflow only if the band
   thresholds have been validated against actuals.

## Why this is deferred

Until enough actuals exist, learned estimates are noisier than static
ones. Worse, they make PR planning depend on historical data that may
not exist for new lanes (a new lane has zero actuals and would either
fall back to `base_lem` or be silently underestimated). Static
estimates are correct in the meantime.

## Privacy and stability

- The learned estimates only consume aggregated p50/p90/p95 numbers.
- They are stored in the planner's working memory, not committed to
  the repo.
- Static `base_lem` remains the audit trail; learned estimates inform,
  they do not replace.
