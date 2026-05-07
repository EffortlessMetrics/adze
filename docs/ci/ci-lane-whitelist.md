# CI lane whitelist

The CI lane whitelist is the single registry of every workflow job that runs
on PR or push for the adze repository. It exists so that:

- new workflows can never be added without a written reason and owner,
- expensive default-PR lanes always have a visible exception with an expiry,
- duplicates are flagged before they accrete cost,
- every lane has a known evidence artifact, so reviewers know what proves it.

## Files

| File | Purpose |
| --- | --- |
| `policy/ci-lane-whitelist.toml` | the lane registry |
| `policy/ci-whitelist-exceptions.toml` | exceptions for expensive defaults |
| `docs/ci/ci-lane-whitelist.md` | this doc |

## Lane fields

| Field | Required | Notes |
| --- | --- | --- |
| `id` | yes | stable lane id; referenced by risk packs and exceptions |
| `workflow` | yes | repo-relative path to the workflow file |
| `job` | yes | job name, or `multiple` for multi-job workflows |
| `kind` | yes | one of: rust, microcrate, platform, fuzz, performance, golden, ffi, docs, policy, control, static-exposure, product-proof, release, external |
| `tier` | yes | frontdoor, advisory, compatibility, deep, release |
| `default_pr` | yes | does the lane run on every PR by default |
| `blocking` | yes | does the lane block merge today |
| `runner` | yes | one of the keys under `[runner_multipliers]`, or `mixed` |
| `base_lem` | yes | static LEM estimate before learned actuals |
| `owner` | yes | team handle, e.g. `core/parser` |
| `intent` | yes | one-sentence description of what the lane proves |
| `failure_mode` | yes | what slips through when the lane is broken |
| `proof_obligation` | yes | the actual command/check that runs |
| `evidence` | yes | artifacts/files reviewers can inspect |
| `allowed_triggers` | yes | event triggers the lane is allowed on |
| `duplicate_of` | no | other lanes whose proof obligation overlaps |
| `expensive` | no | true if base_lem is high enough to require an exception on default PR |
| `default_pr_exception` | no | id in `ci-whitelist-exceptions.toml` |
| `review_after` | yes | when this lane should be reviewed |
| `expires` | yes | when this lane entry must be reconfirmed |

## Whitelist lint rules

Run via:

```
cargo xtask check-ci-lane-whitelist
```

The lint warns (advisory) when:

- a workflow file has jobs not listed in the whitelist,
- a lane is `default_pr=true` and `expensive=true` without an exception id,
- a lane is missing `intent`, `failure_mode`, `proof_obligation`, `owner`, or
  `review_after`/`expires`,
- a lane references a `duplicate_of` id that does not exist,
- a lane uses a runner that has no multiplier defined,
- a workflow file is referenced that does not exist.

## Exception lifecycle

1. A lane is added to `ci-lane-whitelist.toml` with `expensive = true` and
   `default_pr_exception = "..."`.
2. The matching exception is added to `ci-whitelist-exceptions.toml` with an
   owner, an issue pointer, a `review_after`, and an `expires`.
3. When the lane is later routed by risk pack or label, the exception is
   removed and the lane's `default_pr` is set to `false`.
