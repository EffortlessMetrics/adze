# Branch protection

## Current required context

Branch protection is managed through `.github/settings.yml`. The current
required status check is:

```text
CI / ci-supported
```

That context is still the active merge rail until the dedicated branch
protection PR lands. Do not combine that settings change with large workflow
pruning.

## Target required context

The target required status check is the stable aggregate:

```text
PR Gate / PR Gate Success
```

`PR Gate Success` is defined in `.github/workflows/pr-gate.yml` and aggregates:

- `PR Plan` for changed surfaces, docs-only classification, selected lanes, and
  static LEM band;
- `Supported Rust Gate` for ordinary Rust PRs (`just ci-supported`);
- `Docs Gate` for docs-only PRs;
- actuals receipt emission for later budget calibration.

A raw matrix leaf must never be made the required branch-protection context.
The required context should remain stable even when the underlying proof lanes
are refactored.

## Promotion PR

The promotion is a standalone PR titled:

```text
ci: require PR Gate Success as the stable merge gate
```

It may change only the branch-protection settings and directly related docs /
ledger metadata. It must not prune legacy workflows in the same PR.

Expected settings delta:

```yaml
required_status_checks:
  strict: true
  contexts:
    - "PR Gate / PR Gate Success"
```

## Promotion criteria

Open the promotion PR only after operators can verify that `PR Gate Success` is
healthy on recent PRs.

| Criterion | Target |
| --- | --- |
| `PR Gate Success` job has run on every PR for | several recent PRs / operator-approved stability window |
| `PR Gate Success` flake rate | low enough for the branch-protection owner to accept |
| Both supported Rust and docs-only paths have been exercised | yes |
| `ci-actuals.json` artifacts are emitted | yes |
| Manual review of docs-only classification and LEM band accuracy | passes |

The earlier long-window criteria remain useful for conservative rollout, but
the branch-protection owner may choose a shorter window if PR Gate has already
been stable in practice.

## Follow-up after promotion

After the required context has changed and remained green, open a separate PR:

```text
ci: stop running legacy ci.yml on ordinary PRs
```

That follow-up removes or guards the ordinary `pull_request` trigger from the
legacy `.github/workflows/ci.yml` umbrella. It must not delete deep jobs; it only
stops charging duplicate proof to ordinary PRs.

## Rollback

If the aggregate context causes merge problems, restore the previous required
context in `.github/settings.yml`:

```text
CI / ci-supported
```

If legacy `ci.yml` PR execution has already been removed, restore the
`pull_request` trigger in the separate workflow-pruning rollback. No data
migration is needed because PR Gate is an additive workflow.
