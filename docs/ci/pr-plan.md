# PR Plan

`PR Plan` is the advisory job that posts a per-PR forecast: which CI lanes
are likely to run, what they will cost in LEM, and which budget band the PR
falls into.

## What it does

For each PR, the workflow:

1. computes the changed file list against `origin/main`,
2. classifies files into adze areas (docs, core_runtime, parser, microcrate,
   tablegen, grammar, governance, concurrency, wasm, performance, manifest,
   workflow),
3. matches risk packs from `policy/ci-risk-packs.toml`,
4. picks lanes from the whitelist plus any label opt-ins,
5. sums `base_lem` to estimate total cost,
6. writes `target/ci/ci-plan.json`,
7. appends a Markdown summary to the GitHub step summary.

It does not change which jobs run by itself. Routing PRs wire plan outputs,
path filters, and labels into job conditions. Budget enforcement should use
static `base_lem` first; learned estimates wait for actuals.

## Outputs

| Output | Use |
| --- | --- |
| `docs_only` | true when the PR only touches docs/README/changelog |
| `estimated_lem` | sum of `base_lem` across selected lanes |
| `band` | `ordinary` / `elevated` / `high` / `over-ceiling` |
| artifact `ci-plan` | the full `ci-plan.json` |

## Static vs learned

The Python script in `scripts/ci/pr-plan.py` uses static `base_lem` values
duplicated from `policy/ci-lane-whitelist.toml`. The canonical, testable
planner is `xtask ci plan` (PR 09). When that lands, the workflow switches
to `cargo run -p xtask -- ci plan` and the static script becomes a fallback.

## Reading the summary

The summary table for a typical PR looks like:

```
✅ Estimated LEM: 25 (ordinary)

Changed areas: core_runtime
Risk packs: core_runtime

| lane | LEM | blocking | reason |
|------|-----|---------|--------|
| ci-supported | 20 | yes | default frontdoor |
| ripr-advisory | 4 | no | risk pack: core_runtime |
| pr-plan | 1 | no | default frontdoor |
```

A docs-only PR drops the heavy lanes:

```
✅ Estimated LEM: 13 (ordinary)

Changed areas: docs
Risk packs: (none)
```

## Budget enforcement plan

| Band | Static LEM | Behavior |
| --- | ---: | --- |
| ordinary | 0–35 | pass |
| elevated | 36–75 | warning summary |
| high | 76–125 | warning plus explicit label recommendation |
| over ceiling | >125 | fail unless `full-ci` or `ci-budget-override` is present |

Do not use learned estimates for enforcement until the actuals criteria in
`docs/ci/learned-estimates.md` are met.

## Limitations

- Static estimates only, until learned actuals are advisory and then ratcheted.
- Budget enforcement is a later control-plane PR and should not be combined
  with branch-protection or workflow-pruning changes.
- The Python script does not consult `cargo metadata`, so transitive crate
  impacts are approximate. The xtask planner adds dependency-graph closure.
