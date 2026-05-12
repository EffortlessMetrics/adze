# Branch protection

Branch protection is part of the CI economics control plane. It must expose one
stable required context instead of requiring raw workflow leaves or matrix jobs.

## Current required context

The required check today is:

```text
CI / ci-supported
```

That context comes from the legacy `ci.yml` supported lane. The CI economics
rollout has not changed branch protection yet.

## Target required context

The target required check is:

```text
PR Gate / PR Gate Success
```

`PR Gate Success` is the stable aggregate in `.github/workflows/pr-gate.yml`.
It depends on:

- `PR Plan` (advisory control-plane planning),
- `Supported Rust Gate` (`just ci-supported`) for ordinary Rust PRs, and
- `Docs Gate` for docs-only PRs.

`PR Gate Success` succeeds when PR Plan did not fail and exactly the expected
cheap required proof passed: Supported Rust Gate for code PRs, or Docs Gate for
docs-only PRs.

## Migration rule

Move branch protection in a dedicated PR only. Do not combine this change with
workflow pruning, lane removal, or label-routing changes.

The settings change is:

```yaml
required_status_checks:
  strict: true
  contexts:
    - "PR Gate / PR Gate Success"
```

Never require raw matrix leaves (for example a single OS/toolchain matrix job),
nightly lanes, macOS/windows jobs, benchmark jobs, or advisory jobs as branch
protection contexts.

## Promotion criteria

Branch protection promotion to `PR Gate Success` is gated on:

| Criterion | Target |
| --- | --- |
| `PR Gate Success` job has run on every PR for | >=14 calendar days |
| `PR Gate Success` flake rate | <1% |
| Distinct PRs that exercised Supported Rust Gate | >=5 |
| Distinct PRs that exercised Docs Gate | >=5 |
| `ci-actuals.json` artifacts uploaded | >=30 PRs |
| Manual review of band/LEM accuracy | passes |

When all gates clear, open the dedicated branch-protection PR. That PR should
only update `.github/settings.yml` and the docs/policy references to the
required context.

## Relationship to legacy `ci.yml`

After branch protection requires `PR Gate / PR Gate Success`, a separate PR may
remove ordinary `pull_request` execution from legacy `ci.yml`. That is a
separate pruning step because it changes where duplicate PR cost is charged.

`ci.yml` deep jobs should remain available through push to `main`, schedule, and
manual dispatch unless a later dedicated PR reroutes them.

## Rollback

If the new required check causes problems, restore the previous required context
in `.github/settings.yml`:

```text
CI / ci-supported
```

No data migration is required. `PR Gate Success` is additive until the migration
PR lands, and the old supported proof remains `just ci-supported` either way.
