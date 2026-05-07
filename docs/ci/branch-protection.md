# Branch protection

## Today

The required check today is the existing `ci-supported` job in `ci.yml`
(see `docs/status/KNOWN_RED.md`). Branch protection has not been changed
by the rollout.

## Future

The rollout introduces a new aggregated check called **PR Gate Success**
(see `.github/workflows/pr-gate.yml`). It depends on:

- `PR Plan` (advisory)
- `Supported Rust Gate` (= `just ci-supported`)
- `Docs Gate` (fmt only, runs only on docs-only PRs)

`PR Gate Success` succeeds when exactly one of `Supported Rust Gate` /
`Docs Gate` succeeded and the other was skipped. PR Plan must not fail.

This is **additive only** in the rollout's foundation phase. Branch
protection is *not* updated yet. PR 17 in `docs/ci/adze-rollout-plan.md`
is the dedicated change that flips the required check to
`PR Gate Success` once the workflow has been stable for a sufficient
window of PRs.

## PR 17 — promotion criteria

Branch protection promotion to `PR Gate Success` is gated on:

| Criterion | Target |
| --- | --- |
| `PR Gate Success` job has run on every PR for | ≥ 14 calendar days |
| `PR Gate Success` flake rate | < 1% |
| Number of distinct PRs that exercised both `Supported Rust Gate` and `Docs Gate` paths | ≥ 5 each |
| `ci-actuals.json` artifacts uploaded | ≥ 30 PRs |
| Manual review of band/LEM accuracy | passes |

When all five gates clear, PR 17 is opened. PR 17 itself only:

1. updates `.github/settings.yml` (and any equivalent platform config) to
   require `PR Gate Success` and stop requiring the legacy `ci-supported`
   job, **and**
2. removes the redundant `ci-supported` job from `ci.yml` if it is no
   longer used by anything else (it is also reachable via the new
   `pr-gate.yml`).

## Rollback

Removing `pr-gate.yml` reverts to the existing required check. No state
needs migrating because PR Gate Success is a new, separate workflow.

If PR 17 has landed and the new required check is causing problems, the
rollback is to restore the previous required check name in
`.github/settings.yml` and re-add the legacy `ci-supported` job
configuration.
