# Branch protection

## Today

The required check today is the existing branch-protection context
`CI / ci-supported` configured in `.github/settings.yml`. Branch protection
has not been changed by the control-plane rollout.

## Future

The rollout introduces a new aggregated check called **PR Gate Success**
(see `.github/workflows/pr-gate.yml`). It depends on:

- `PR Plan` (advisory)
- `Supported Rust Gate` (= `just ci-supported`)
- `Docs Gate` (fmt only, runs only on docs-only PRs)

`PR Gate Success` succeeds when exactly one of `Supported Rust Gate` /
`Docs Gate` succeeded and the other was skipped. PR Plan must not fail.

This is **additive only** until the dedicated promotion PR. The promotion PR
changes only branch-protection settings and docs; it must not also prune
legacy workflows. The target required context is:

```yaml
required_status_checks:
  strict: true
  contexts:
    - "PR Gate / PR Gate Success"
```

## Promotion criteria

Branch protection promotion to `PR Gate Success` is gated on:

| Criterion | Target |
| --- | --- |
| `PR Gate Success` job has run on every PR for | ≥ 14 calendar days |
| `PR Gate Success` flake rate | < 1% |
| Number of distinct PRs that exercised both `Supported Rust Gate` and `Docs Gate` paths | ≥ 5 each |
| `ci-actuals.json` artifacts uploaded | ≥ 30 PRs |
| Manual review of band/LEM accuracy | passes |

When all five gates clear, open the promotion PR. That PR itself only updates
`.github/settings.yml` (and any equivalent platform config) to require
`PR Gate / PR Gate Success` and stop requiring the legacy `CI / ci-supported`
context.

Do not remove or prune legacy workflow execution in the same PR. The follow-up
PR that removes ordinary `pull_request` execution from `ci.yml` is separate so
branch-protection rollback remains simple.

## Rollback

Removing `pr-gate.yml` reverts to the existing required check. No state
needs migrating because PR Gate Success is a new, separate workflow.

If the promotion PR has landed and the new required check is causing problems,
the rollback is to restore the previous required check name in
`.github/settings.yml`:

```yaml
required_status_checks:
  strict: true
  contexts:
    - "CI / ci-supported"
```

If the later legacy-PR-trigger pruning has also landed, restore that trigger in
`.github/workflows/ci.yml` as a second, explicit rollback step.
