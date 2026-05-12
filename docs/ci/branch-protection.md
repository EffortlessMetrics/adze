# Branch protection

Branch protection is part of the CI economics control plane. It must expose one
stable required proof instead of requiring raw matrix leaves or duplicate jobs.

## Current required context

The current repository setting still requires the legacy supported context:

```yaml
required_status_checks:
  strict: true
  contexts:
    - "CI / ci-supported"
```

That context remains the rollback anchor until the dedicated branch-protection
PR lands.

## Target required context

After `PR Gate / PR Gate Success` has demonstrated stability, branch protection
should require only:

```yaml
required_status_checks:
  strict: true
  contexts:
    - "PR Gate / PR Gate Success"
```

Do **not** require individual matrix leaves, platform jobs, benchmark jobs,
advisory lanes, or label-triggered deep lanes.

## PR Gate Success shape

`PR Gate Success` is the stable aggregate from `.github/workflows/pr-gate.yml`.
It depends on:

- `PR Plan` (control-plane planner; non-product proof),
- `Supported Rust Gate` (`just ci-supported`) for non-docs PRs,
- `Docs Gate` for docs-only PRs.

The aggregate succeeds when the applicable gate passes and the non-applicable
gate is skipped. This gives branch protection one stable status name while
allowing the internal implementation to evolve.

## Promotion criteria

Open the dedicated promotion PR only after:

| Criterion | Target |
| --- | --- |
| `PR Gate Success` has run on every PR for | >=14 calendar days |
| `PR Gate Success` flake rate | <1% |
| Supported Rust Gate path coverage | >=5 distinct PRs |
| Docs Gate path coverage | >=5 distinct PRs |
| `ci-actuals.json` artifacts uploaded | >=30 PRs, if artifact collection is already enabled |
| Manual review of docs-only and Rust PR behavior | passes |

## Dedicated promotion PR scope

The branch-protection PR may change:

1. `.github/settings.yml` required status checks, and
2. the user-facing docs that describe the required context.

It must **not** also prune workflows, reroute expensive lanes, or split test
policy. Those are separate rollout PRs.

## Rollback

If the aggregate required context causes merge blockage, restore the previous
required check in `.github/settings.yml`:

```yaml
required_status_checks:
  strict: true
  contexts:
    - "CI / ci-supported"
```

No data migration is required because `PR Gate Success` is an aggregate check,
not an owner of product artifacts.
