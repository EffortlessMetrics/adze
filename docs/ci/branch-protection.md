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

## Rollback

Removing `pr-gate.yml` reverts to the existing required check. No state
needs migrating because PR Gate Success is a new, separate workflow.
