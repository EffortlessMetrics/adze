# CI labels

Labels are the operator interface for expensive verification. They let a PR opt
into deep proof without making that proof an ordinary default for everyone.

## Budget labels

| Label | Meaning |
| --- | --- |
| `ci-budget-ack` | Acknowledge elevated/high LEM (36-125) without changing the hard ceiling. |
| `ci-budget-override` | Allow an over-ceiling PR (>125 LEM) when the cost is intentional. |
| `full-ci` | Run broad deep verification; implies the operator accepts high CI spend. |

## Routing labels

| Label | Adds |
| --- | --- |
| `platform-matrix` | Full OS/toolchain proof. Windows and macOS remain opt-in, scheduled, manual, or release-only. |
| `pure-rust` | Pure-Rust/platform proof without requesting every unrelated deep lane. |
| `coverage` | Coverage instrumentation and Codecov artifacts. |
| `ci:golden` | Grammar golden/parity validation. |
| `ci:perf` | Performance comparison lanes. |
| `benchmarks` | Full benchmark suite where supported. |
| `ci:microcrate` | Full microcrate/governance matrix. |
| `ci:concurrency` | Concurrency/rayon/bootstrap risk pack. |
| `wasm` | WASM checks and browser/playground surfaces. |
| `fuzz` | Runtime fuzzing lanes. |
| `ts-bridge` | ts-bridge parity/deep checks. |
| `api` | Public API and SemVer checks. |
| `release-check` | Release readiness lanes. |
| `security-audit` | Dependency/RUSTSEC/supply-chain audit lanes. |
| `mutation` | Mutation testing on parser/GLR surfaces. |
| `property-tests` | Property-test runs on parser/GLR surfaces. |

## Skip labels

| Label | Effect |
| --- | --- |
| `skip-golden` | Skip golden tests when no explicit `ci:golden`/`full-ci` opt-in is present. |
| `skip-perf` | Skip performance comparison when no explicit `ci:perf`/`benchmarks`/`full-ci` opt-in is present. |

## Rules

- Labels may select deeper verification; they must not make macOS or Windows an
  ordinary PR default.
- Labels explain and route cost; they do not weaken `just ci-supported`.
- `full-ci` and `ci-budget-override` are the only labels that can authorize an
  over-ceiling static plan.
- When a workflow starts honoring a label, update this file,
  `.github/settings.yml`, `.github/CI_LANES.md`, and the CI lane whitelist or
  risk-pack ledger in the same PR.
