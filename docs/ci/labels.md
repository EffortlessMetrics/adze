# CI labels

Labels are the operator interface for CI economics. They opt a PR into deeper
verification, acknowledge budget bands, or route a specific risk pack. Labels do
not weaken `just ci-supported`, and they must not make macOS/windows ordinary PR
defaults.

## Budget labels

| Label | Meaning |
| --- | --- |
| `full-ci` | Opt into all heavy verification lanes that are safe for the event. Implies budget override intent. |
| `ci-budget-override` | Allow a static PR Plan over the hard ceiling (>125 LEM). |
| `ci-budget-ack` | Acknowledge elevated/high cost (35-125 LEM) without forcing all lanes. |

## Risk-pack opt-ins

| Label | Adds |
| --- | --- |
| `platform-matrix` | Full OS/toolchain platform proof. |
| `pure-rust` | Pure-Rust implementation proof without implying unrelated heavy lanes. |
| `coverage` | Coverage instrumentation/reporting. |
| `ci:golden` | Grammar golden/parity lanes. |
| `ci:perf` | Performance comparison lanes. |
| `ci:microcrate` | Full microcrate CI matrix. |
| `ci:concurrency` | Concurrency microcrate group. |
| `wasm` | WASM checks. |
| `fuzz` | Runtime fuzzing lanes. |
| `benchmarks` | Full benchmark suite/reporting. |
| `api` | Public API / SemVer checks. |
| `release-check` | Release-prep verification. |
| `security-audit` | Dependency and RUSTSEC audit lanes. |
| `mutation` | Mutation testing on parser/GLR core. |
| `property-tests` | Property-test runs on parser surfaces. |
| `ts-bridge` | ts-bridge parity or non-smoke FFI validation. |

## Skip labels

| Label | Effect |
| --- | --- |
| `skip-golden` | Skip golden tests where the workflow supports it and no explicit golden opt-in is present. |
| `skip-perf` | Skip performance comparison where the workflow supports it and no explicit perf opt-in is present. |

## Required settings labels

`.github/settings.yml` should define every label that appears in this document,
`policy/ci-risk-packs.toml`, `policy/ci-lane-whitelist.toml`, or workflow label
conditions. Missing labels make expensive verification harder for operators and
agents to request consistently.

## Notes

- Labels select additional proof; they do not replace the cheap required gate.
- Labels may explain or allow high LEM plans, but only `full-ci` and
  `ci-budget-override` can allow over-ceiling static plans.
- Expensive labels should be paired with a PR body explanation of LEM impact,
  default PR effect, branch-protection impact, rollback path, and proof
  obligation.
