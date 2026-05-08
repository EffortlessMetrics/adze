# CI Economics Rollout Status

Last review: 2026-05-08.

## Status legend

- ✅ landed — present on `main` and working
- 🟡 in progress — this PR stack
- ⏳ planned — not yet started
- ⏸ deferred — waiting on actuals or coordination

## Foundation layer

| # | Item | Status | Notes |
| --- | --- | --- | --- |
| F01 | CI lane whitelist (`policy/ci-lane-whitelist.toml`) | ✅ | All workflows mapped with owner, LEM, triggers |
| F02 | CI risk packs (`policy/ci-risk-packs.toml`) | ✅ | 10 risk packs: core_runtime, macro_tool, glr_core, tablegen, grammar_golden, microcrate_governance, concurrency, wasm, performance, manifest_release |
| F03 | PR Plan workflow (`pr-plan.yml`) | ✅ | Calls `xtask ci-plan`, emits `ci-plan.json`, docs_only/estimated_lem/band outputs |
| F04 | PR Gate Success workflow (`pr-gate.yml`) | ✅ | Supported Gate + Docs Gate + `PR Gate Success` aggregator |
| F05 | ci-actuals telemetry scaffold | ✅ | `scripts/ci/emit-ci-actuals.py` emits plan vs actual; uploaded as artifact |
| F06 | ripr advisory stub (`ripr.yml`) | ✅ | Graceful no-op stub; real binary provisioning in PR `ci/ripr-provision` |

## Control plane

| # | Item | Status | Notes |
| --- | --- | --- | --- |
| C01 | CI policy workflow (`ci-policy.yml`) | ✅ | Runs `check-ci-lane-whitelist --mode advisory` on every PR |
| C02 | Synchronize-only cancellation | 🟡 | PR: `ci/sync-cancellation` — prevents label events from killing running jobs |
| C03 | Lane whitelist cost alignment | 🟡 | PR: `ci/whitelist-align` — corrects stale LEM for already-gated lanes |
| C04 | Real ripr provisioning | 🟡 | PR: `ci/ripr-provision` — isolated Rust 1.93 install |
| C05 | Benchmark deduplication | 🟡 | PR: `ci/benchmark-dedup` — gate performance-check to ci:perf label |

## Routing

| # | Item | Status | Notes |
| --- | --- | --- | --- |
| R01 | Fuzz gating (label/push only) | ✅ | `fuzz.yml` — runtime fuzz requires `fuzz`/`full-ci` label or push/schedule |
| R02 | Pure-Rust PR matrix reduction | ✅ | `pure-rust-ci.yml` — ubuntu/stable default; full matrix on `platform-matrix`/`full-ci`/main |
| R03 | Golden tests grammar routing | ✅ | `golden-tests.yml` — paths + `ci:golden`/`full-ci` label gates |
| R04 | Microcrate CI risk-pack routing | ✅ | `microcrate-ci.yml` — per-group path detection (concurrency/governance/bdd/parser/core/runtime) |
| R05 | Benchmark PR ownership cleanup | 🟡 | PR: `ci/benchmark-dedup` — removes duplicate baseline+comparison from default PR path |

## Policy ledgers

| # | Item | Status | Notes |
| --- | --- | --- | --- |
| P01 | Clippy lint policy (`policy/clippy-lints.toml`) | ✅ | Schema, active, staged, and planned lints documented |
| P02 | No-panic allowlist (`policy/no-panic-allowlist.toml`) | ✅ | Schema in place; intentionally empty until `cargo xtask no-panic-propose --baseline` run |
| P03 | Non-Rust file policy (`policy/non-rust-allowlist.toml`) | ✅ | All non-Rust surfaces registered |
| P04 | ripr suppressions (`policy/ripr-suppressions.toml`) | ✅ | Schema in place; empty baseline |
| P05 | Workspace rust lints (`[workspace.lints.rust]`) | ✅ | `unsafe_op_in_unsafe_fn`, `unused_must_use`, `missing_docs`, `unused_extern_crates` |
| P06 | Strict Clippy Stage A (`[workspace.lints.clippy]`) | ⏳ | PR: `policy/clippy-stage-a` — add `allow_attributes_without_reason` and stage-A rust lints |

## MSRV and toolchain

| # | Item | Status | Notes |
| --- | --- | --- | --- |
| T01 | MSRV 1.93 | ⏳ | PR: `policy/msrv-1-93` — dedicated policy PR, prerequisite for ripr simplification |

## Branch protection

| # | Item | Status | Notes |
| --- | --- | --- | --- |
| B01 | Branch protection migration docs | ✅ | `docs/ci/branch-protection.md` — criteria for promoting required check |
| B02 | `PR Gate Success` stable run history | ⏸ | Needs ≥14 days and ≥5 distinct PRs per path before promotion |
| B03 | Migrate required check to `PR Gate Success` | ⏸ | After B02 — see `docs/ci/branch-protection.md` |

## Learned estimates

| # | Item | Status | Notes |
| --- | --- | --- | --- |
| L01 | ci-actuals collection | ✅ | Artifact upload on every PR Gate run |
| L02 | Learned LEM model | ⏸ | Needs ≥30 days of actuals; see `docs/ci/learned-estimates.md` |

## Effective default PR lane cost (current)

With all routing already in place, the effective default PR cost estimate:

| Lane | Effective default PR cost |
| --- | --- |
| PR Plan | ~1 LEM |
| Supported Rust Gate | ~20 LEM |
| PR Gate Success | ~1 LEM |
| CI Lane Whitelist | ~2 LEM |
| ripr advisory | ~4 LEM (stub today; install attempted after `ci/ripr-provision`) |
| Test Policy | ~12 LEM |
| Pure Rust (ubuntu/stable only) | ~18 LEM |
| Microcrate CI (routed by risk pack) | ~5–20 LEM depending on changed surface |
| Fuzz build smoke (parser/glr paths only) | ~3 LEM |
| Criterion smoke | ~6 LEM |
| ts-bridge lanes | ~8 LEM |
| Clippy quarantine report | ~4 LEM |
| **Estimated total (typical runtime PR)** | **~65–80 LEM** |

Target is ≤35 LEM for ordinary PRs. The gap comes from `pure-rust-ci` (18 LEM)
and `microcrate-ci` (variable) running broadly on every PR. Active exceptions in
`policy/ci-whitelist-exceptions.toml` track these while tightening continues.

See `docs/ci/lem-budgeting.md` for budget bands and override labels.
