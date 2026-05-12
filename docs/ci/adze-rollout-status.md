# CI Economics Rollout Status

Last review: 2026-05-12.

This file is a status snapshot, not a live source of truth. Before using it for
execution, refresh with `gh pr list` and the current workflow state.

## Status legend

- ✅ landed — present on `main` and working
- 🟡 in progress — open PR or active follow-up
- ⏳ planned — not yet started
- ⏸ deferred — waiting on actuals or coordination

## Foundation layer

| # | Item | Status | Notes |
| --- | --- | --- | --- |
| F01 | CI lane whitelist (`policy/ci-lane-whitelist.toml`) | ✅ | All workflows mapped with owner, LEM, triggers |
| F02 | CI risk packs (`policy/ci-risk-packs.toml`) | ✅ | 10 risk packs: core_runtime, macro_tool, glr_core, tablegen, grammar_golden, microcrate_governance, concurrency, wasm, performance, manifest_release |
| F03 | PR Plan workflow (`pr-plan.yml`) | ✅ | Calls `xtask ci-plan`, emits `ci-plan.json` with outputs `docs_only`, `estimated_lem`, `band` |
| F04 | PR Gate Success workflow (`pr-gate.yml`) | ✅ | Supported Gate + Docs Gate + `PR Gate Success` aggregator |
| F05 | ci-actuals telemetry scaffold | ✅ | `scripts/ci/emit-ci-actuals.py` emits plan vs actual; uploaded as artifact |
| F06 | ripr advisory (`ripr.yml`) | ✅ | Advisory install attempts run on an isolated Rust 1.93 toolchain and fall back to a stub report on install failure |

## Control plane

| # | Item | Status | Notes |
| --- | --- | --- | --- |
| C01 | CI policy workflow (`ci-policy.yml`) | ✅ | Runs `check-ci-lane-whitelist --mode advisory` on every PR |
| C02 | Synchronize-only cancellation | ✅ | PR #563 merged — prevents label events from killing running jobs |
| C03 | Lane whitelist cost alignment | ✅ | PR #564 merged — corrects stale LEM for already-gated lanes |
| C04 | Real ripr provisioning | ✅ | PR #565 merged — isolated Rust 1.93 install with graceful stub fallback |
| C05 | Benchmark deduplication | ✅ | PR #566 merged — `performance-check` gated to `ci:perf` label |

## Routing

| # | Item | Status | Notes |
| --- | --- | --- | --- |
| R01 | Fuzz gating (label/push only) | ✅ | `fuzz.yml` — runtime fuzz requires `fuzz`/`full-ci` label or push/schedule |
| R02 | Pure-Rust PR matrix reduction | ✅ | `pure-rust-ci.yml` — ubuntu/stable default; full matrix on `platform-matrix`/`full-ci`/main |
| R03 | Golden tests grammar routing | ✅ | `golden-tests.yml` — paths + `ci:golden`/`full-ci` label gates |
| R04 | Microcrate CI risk-pack routing | ✅ | `microcrate-ci.yml` — per-group path detection (concurrency/governance/bdd/parser/core/runtime) |
| R05 | Benchmark PR ownership cleanup | ✅ | PR #566 merged — removes duplicate baseline+comparison from default PR path |

## Policy ledgers

| # | Item | Status | Notes |
| --- | --- | --- | --- |
| P01 | Clippy lint policy (`policy/clippy-lints.toml`) | ✅ | Schema, active, staged, and planned lints documented |
| P02 | No-panic allowlist (`policy/no-panic-allowlist.toml`) | ✅ | Schema in place; intentionally empty until `cargo xtask no-panic-propose --baseline` run |
| P03 | Non-Rust file policy (`policy/non-rust-allowlist.toml`) | ✅ | All non-Rust surfaces registered |
| P04 | ripr suppressions (`policy/ripr-suppressions.toml`) | ✅ | Schema in place; empty baseline |
| P05 | Workspace rust lints (`[workspace.lints.rust]`) | ✅ | `unsafe_op_in_unsafe_fn`, `unused_must_use`, `missing_docs`, `unused_extern_crates` |
| P06 | Strict Clippy Stage A (`[workspace.lints.clippy]`) | 🟡 | PR #567: `policy/clippy-stage-a` — `allow_attributes_without_reason` + stage-A rust lints |

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

This snapshot is intentionally conservative. The authoritative queue for
reducing the default shape is `docs/ci/adze-rollout-plan.md`; this section shows
why the next wave focuses on pruning duplicate/expensive ordinary PR lanes.

With all routing already in place, the effective default PR cost estimate:

| Lane | Effective default PR cost |
| --- | --- |
| PR Plan | ~1 LEM |
| Supported Rust Gate | ~20 LEM |
| PR Gate Success | ~1 LEM |
| CI Lane Whitelist | ~2 LEM |
| ripr advisory | ~4 LEM (isolated install attempted; stub report on toolchain/binary failure) |
| Test Policy | ~12 LEM |
| Pure Rust (ubuntu/stable only) | ~18 LEM |
| Microcrate CI (routed by risk pack) | ~5–20 LEM depending on changed surface |
| Fuzz build smoke (parser/glr paths only) | ~3 LEM |
| Criterion smoke | ~6 LEM |
| ts-bridge lanes | ~8 LEM |
| Clippy quarantine report | ~4 LEM |
| **Estimated total (typical runtime PR)** | **~65–80 LEM** |

Target is ≤25 LEM preferred and ≤35 LEM ceiling for ordinary PRs. The gap comes
from duplicate supported proof, full test-policy behavior, `pure-rust-ci`,
variable `microcrate-ci`, performance comparison, and ts-bridge/API checks still
being too broad for ordinary PR defaults. Active exceptions in
`policy/ci-whitelist-exceptions.toml` track these while tightening continues.

Next wave, in order:

1. add missing routing labels to repo settings;
2. promote `PR Gate / PR Gate Success` as the stable required context;
3. remove ordinary PR execution from legacy `ci.yml`;
4. split test-policy smoke/full;
5. route pure-rust, microcrate, performance, ts-bridge, and API checks harder;
6. enforce lane metadata only when workflow/policy files change;
7. collect actuals before learned estimates or budget ratchets.

See `docs/ci/lem-budgeting.md` for budget bands and override labels.
