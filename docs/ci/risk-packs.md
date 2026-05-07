# CI risk packs

Risk packs are the bridge between "what changed" and "what to verify". Each
pack names a surface (parser, tablegen, grammar/golden, governance,
concurrency, wasm, performance, manifest/release) and lists the lanes that
should run when that surface is touched.

## File

`policy/ci-risk-packs.toml`

## Pack fields

| Field | Required | Notes |
| --- | --- | --- |
| `description` | yes | one-line surface description |
| `paths` | yes | glob list of file paths that imply this pack |
| `keywords` | no | substring matches against changed file paths/names |
| `lanes` | yes | lane ids run by default when the pack matches |
| `deep_lanes` | yes | lane ids run when `full-ci` (or pack-specific) labels are present |
| `labels` | yes | labels that opt into this pack regardless of paths |

## How packs are selected

PR Plan selects a pack when any of the following is true:

1. a changed file matches `paths` for the pack,
2. a changed file's name contains any `keywords`,
3. a PR label matches one of `labels`.

A PR can match multiple packs. Selected lanes are the union (deduplicated)
of every matched pack's `lanes`. `deep_lanes` are added only when the
`full-ci` label is present, or when a pack-specific opt-in label
(`ci:perf`, `ci:golden`, etc.) appears.

## Why packs and not directories

The same directory can serve multiple intents — `crates/parser-*` is
parser surface, `crates/governance-*` is governance surface, but both
live under `crates/`. Risk packs let routing follow product semantics
rather than filesystem layout.

## Where packs are read

| Reader | Behavior |
| --- | --- |
| `scripts/ci/pr-plan.py` | mirrors a subset of packs as a static dict |
| `xtask ci plan` (PR 09) | reads `ci-risk-packs.toml` directly |
| Routing workflows (PRs 10–14) | gate jobs on pack hits, not on raw paths |

## Updating packs

When adding a new product surface (a new microcrate group, a new grammar
implementation, a new runtime layer), add a pack first, then route the
relevant lanes to it. The PR for the new surface should include the pack
update; the rollout plan in `docs/ci/adze-rollout-plan.md` describes the
lifecycle.
