# CI labels

Labels are the operator interface for expensive verification. They give agents
and maintainers stable names for selecting non-default lanes without turning
those lanes into ordinary PR defaults.

## Cost labels

| Label | Meaning |
| --- | --- |
| `ci-budget-ack` | Acknowledge elevated/high LEM (35–125). |
| `ci-budget-override` | Allow over-ceiling static LEM (>125) when justified. |
| `full-ci` | Run all heavy lanes that are available for PR opt-in; implies budget override. |

## Risk-pack opt-in labels

| Label | Adds |
| --- | --- |
| `platform-matrix` | Full OS/toolchain matrix. Not an ordinary PR default. |
| `coverage` | Coverage instrumentation and upload. |
| `ci:golden` | Golden parse-tree tests for grammars. |
| `ci:perf` | Full benchmark comparison. |
| `ci:microcrate` | Full microcrate CI matrix. |
| `ci:concurrency` | Concurrency microcrate group. |
| `wasm` | WASM checks. |
| `fuzz` | Runtime fuzzing. |
| `benchmarks` | Full benchmark suite. |
| `property-tests` | Property-test runs on parser surfaces. |
| `mutation` | Mutation testing on parser/GLR core. |

## Release / API / security labels

| Label | Adds |
| --- | --- |
| `api` | Public API / SemVer checks on API-risk PRs. |
| `release-check` | Release-gate verification. |
| `security-audit` | Dependency / RUSTSEC audit. |
| `breaking-change` | Explicitly marks breaking API or behavior changes. |

## Skip labels

| Label | Effect |
| --- | --- |
| `skip-golden` | Skip golden tests when no explicit `ci:golden` opt-in is present. |
| `skip-perf` | Skip perf comparison when no explicit `ci:perf` opt-in is present. |

## Repo settings requirement

The labels above should exist in `.github/settings.yml` so the same vocabulary
is available to humans, PR Plan, and risk-pack routing. Adding a label does not
by itself add CI cost; workflows must explicitly route on it.

## Rules

- Labels may select deeper verification, but must not make macOS or Windows an
  ordinary PR default.
- Labels explain high cost and select lanes; they do not weaken
  `just ci-supported`.
- `ci-budget-override` is the only label that can permit an over-ceiling static
  PR Plan once budget enforcement lands, and it should be rare.
