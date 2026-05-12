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
| `benchmarks` | full benchmark suite / benchmark comparison |
| `ci:golden` | golden tests for grammars |
| `ci:microcrate` | full microcrate CI matrix |
| `ci:concurrency` | concurrency microcrate group |
| `platform-matrix` | full OS/toolchain matrix |
| `fuzz` | fuzz runtime |
| `coverage` | coverage instrumentation |
| `wasm` | wasm-check |
| `api` | public API and SemVer checks |

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

- `.github/settings.yml` must define every routing label before workflows depend
  on it. The current implementation queue adds missing labels in a dedicated
  labels PR.
- Labels are the operator interface for expensive verification. They should make
  a lane eligible; they should not silently make raw matrix leaves required.
- Labels never *raise* the hard ceiling. They explain a high LEM number; only
  `full-ci` or `ci-budget-override` may allow over-ceiling work once static
  budget enforcement lands.
