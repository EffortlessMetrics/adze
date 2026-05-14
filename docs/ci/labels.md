# CI labels

The label vocabulary used by PR Plan to opt in to deeper verification.

## Cost labels

| Label | Meaning |
| --- | --- |
| `ci-budget-ack` | acknowledge elevated/high LEM (35–125) |
| `ci-budget-override` | allow >125 LEM |
| `full-ci` | run all heavy lanes; implies budget override |

## Risk-pack opt-in

| Label | Adds |
| --- | --- |
| `ci:perf` | full benchmark comparison |
| `ci:golden` | golden tests for grammars |
| `ci:microcrate` | full microcrate CI matrix |
| `ci:concurrency` | concurrency owner-module opt-in; standalone concurrency microcrates are collapsed |
| `platform-matrix` | full OS/toolchain matrix |
| `fuzz` | fuzz runtime |
| `coverage` | coverage instrumentation |
| `wasm` | wasm-check |

## Release / API

| Label | Adds |
| --- | --- |
| `release-check` | release-gate verification |
| `security-audit` | dependency / RUSTSEC audit |
| `mutation` | mutation testing on parser/glr core |
| `property-tests` | property-test runs on parser |

## Skip labels

| Label | Effect |
| --- | --- |
| `skip-golden` | skip golden tests (only honored when no `ci:golden`) |
| `skip-perf` | skip perf compare (only honored when no `ci:perf`) |

## Notes

- Labels are advisory until the routing PRs (10–14) land. Until then they
  appear in the PR Plan summary but do not change which lanes run.
- Labels never *raise* the hard ceiling. They explain a high LEM number; they
  do not silence safety checks.
