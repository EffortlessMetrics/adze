# CI Lane Map

**Last updated:** 2026-05-08
**Purpose:** Classify every CI check so contributors can immediately tell
whether a red mark means "must fix before merge" or "inspect at your leisure."

## Lane semantics

| Signal | Meaning | Contributor action |
|--------|---------|--------------------|
| **Required** | Merge is blocked. Fix before requesting review. | Must fix. |
| **PR-only signal** | Runs on every PR, informational, not a merge gate. | Review if red; not a blocker. |
| **Push / scheduled** | Runs on `main` pushes or schedules. Not PR-blocking. | Inspect trend; fix in a follow-up. |
| **Advisory** | Uses nightly / unstable toolchains, `continue-on-error`, or non-blocking labels. May be red due to toolchain drift. | Inspect if curious. Not actionable for most PRs. |

Branch protection requires exactly one check: **`CI / ci-supported`** (via `pr-gate.yml`).

---

## Complete lane inventory

### Required (merge gate)

| Workflow | Job name | Trigger | Meaning |
|----------|----------|---------|---------|
| `pr-gate.yml` | `Supported Rust Gate` | PR + merge_group | Runs `just ci-supported`: fmt, clippy, tests on 7 core crates |
| `pr-gate.yml` | `PR Gate Success` | PR + merge_group | Aggregate: plan + supported/docs gate must pass |
| `pr-gate.yml` | `PR Plan` | PR | Computes docs_only, estimated LEM, budget band |

Required branch protection context: `CI / ci-supported` (maps to the `ci-supported` job in `ci.yml`, also exercised via `pr-gate.yml`).

### PR-only signal (non-blocking)

| Workflow | Job name | Trigger | Lane | Notes |
|----------|----------|---------|------|-------|
| `ci.yml` | `ci-supported` | PR + push | **Required** (via pr-gate) | The canonical green gate; runs on every event |
| `ci.yml` | `semver-checks` | PR only | PR-only | Detects breaking API changes |
| `ci.yml` | `api-stability` | PR only | PR-only | `cargo-public-api` diff; `continue-on-error` |
| `ci.yml` | `package-validation` | PR only | PR-only | Validates package manifests for release surface |
| `ci-policy.yml` | `CI Lane Whitelist` | PR + push | PR-only | Advisory xtask lane whitelist lint |
| `ripr.yml` | `ripr advisory` | PR | PR-only | Advisory report; non-blocking |
| `droid-review.yml` | `droid-review` | PR (non-draft) | PR-only | Factory Droid auto-review; `continue-on-error` |

### Push / scheduled (main health, not PR-blocking)

These jobs run on push to `main`, on schedules, or via `workflow_dispatch` with `run_full_ci`. They do **not** run on ordinary PRs unless explicitly opted in.

| Workflow | Job name | Trigger | Lane | Notes |
|----------|----------|---------|------|-------|
| `ci.yml` | `Lint` | Push only | Push | Full lint suite (bare no_mangle, debug blocks, fmt, clippy) |
| `ci.yml` | `Test` | Push only | Push | OS x features x toolchain matrix (3 OS, 4 features, 2 toolchains) |
| `ci.yml` | `Matrix Smoke Test` | Push only | Push | Workspace default + all-features test |
| `ci.yml` | `Test with Debug Assertions` | Push only | Push | Debug-assertion tests for glr-core, runtime, tablegen |
| `ci.yml` | `Test Release Mode` | Push only | Push | Release-mode tests with strict-invariants |
| `ci.yml` | `Benchmark Compilation` | Push only | Push | Bench compile check (no-run) |
| `ci.yml` | `Backend Build Matrix` | Push only | Push | pure-rust backend check + test |
| `ci.yml` | `Tree-sitter Compatibility API` | Push only | Push | ts-compat feature build + test |
| `ci.yml` | `Deterministic Codegen` | Push only | Push | Verifies build determinism |
| `ci.yml` | `Feature Matrix` | Push only | Push | Per-crate feature matrix checks |
| `ci.yml` | `Feature Matrix Extras` | Push only | Push | Feature powerset via cargo-hack |
| `ci.yml` | `MSRV (1.92.0)` | Push only | Push | Minimum Supported Rust Version check |
| `ci.yml` | `Security & Supply Chain` | Push only | Push | `cargo deny check` |
| `ci.yml` | `Documentation` | Push only | Push | `cargo doc --workspace` with `-D warnings` |
| `ci.yml` | `adze-python (Optimized Build)` | Push only | Push | Python grammar build + test |
| `ci.yml` | `Test Connectivity (Tripwires)` | Push only | Push | Enforces no disabled tests, non-zero discovery |
| `ci.yml` | `Code Coverage` | Push only | Push | `cargo llvm-cov` with threshold check |
| `ci.yml` | `Advisory / Unsafe Audit` | Push only | Advisory | `cargo geiger` report; `continue-on-error` |
| `ci.yml` | `Advisory / Cross Compilation (${{ matrix.target }})` | Push only | Advisory | 32-bit / ARM64 / WASM cross builds; `continue-on-error` |
| `ci.yml` | `Cross-platform` | Push only | Push | macOS + Windows cargo check + lib tests |
| `ci.yml` | `Advisory / WASM Build` | Push only | Advisory | WASM target check; `continue-on-error` |
| `ci.yml` | `Benches (unstable, opt-in)` | Dispatch only | Advisory | `unstable-benches` feature; only with `run_full_ci` |
| `pure-rust-ci.yml` | `Test Pure Rust Implementation` | Push + labeled PR | Push | OS x toolchain matrix for pure-rust path |
| `pure-rust-ci.yml` | `Test WASM Build` | Push + labeled PR | Advisory | WASM build + size check |
| `pure-rust-ci.yml` | `Golden Tests` | Push + labeled PR | Advisory | Tree-sitter parity; label-gated |
| `pure-rust-ci.yml` | `Integration Tests` | PR + push | PR-only | c2rust backend test |
| `pure-rust-ci.yml` | `Performance Regression Tests` | Push + labeled PR | Advisory | Benchmark run; label-gated |
| `pure-rust-ci.yml` | `Code Coverage` | Push + labeled PR | Advisory | Coverage report; label-gated |
| `core-tests.yml` | `core` | Scheduled (nightly) + dispatch | Scheduled | Full nightly canary: clippy, doc, all-features |
| `benchmarks.yml` | `Performance Benchmarks` | Push + labeled PR | Push | Benchmark comparison for PRs |
| `benchmarks.yml` | `Criterion HTML Report` | Push (main only) | Push | Criterion report generation |
| `coverage.yml` | `Codecov Coverage` | Push + labeled PR | Push | Dedicated coverage lane |
| `microcrate-ci.yml` | `Formatting` through `Strict Docs` | Push + path-routed PR | Push | Governance micro-crate tests |
| `golden-tests.yml` | `Golden Tests` | Push + path-routed PR | Push | Tree-sitter parity validation |
| `performance.yml` | `Performance Regression Check` | PR (path-routed) | PR-only | Benchmark comparison on perf-impact changes |
| `test-policy.yml` | `Enforce Test Policy` | PR + push | PR-only | Test naming, connectivity, coverage |
| `mdbook.yml` | `build` + `deploy` | Push + PR | Push | Documentation site build |
| `smoke-ts-bridge.yml` | `smoke` | Push + PR | PR-only | ts-bridge link verification |
| `ts-bridge-smoke.yml` | `smoke` | Push + PR | PR-only | ts-bridge smoke with libtree-sitter |
| `release.yml` | Various release jobs | Dispatch only | Dispatch | Manual release workflow |

### Advisory (nightly / unstable / non-blocking)

These jobs use nightly toolchains, unstable features, or are explicitly marked
`continue-on-error: true`. Red here means "inspect" not "block."

| Workflow | Job name | Trigger | Why advisory |
|----------|----------|---------|-------------|
| `ci.yml` | `Advisory / Miri` | Push only | Nightly miri; `continue-on-error` |
| `ci.yml` | `Advisory / Sanitizers` | Push only | Nightly + `-Zbuild-std`; `continue-on-error` |
| `ci.yml` | `Advisory / Minimal Versions` | Push only | Nightly + `-Z minimal-versions`; `continue-on-error` |
| `ci.yml` | `Advisory / Cross Compilation (${{ matrix.target }})` | Push only | Cross toolchain drift; `continue-on-error` |
| `ci.yml` | `Advisory / WASM Build` | Push only | Compile-check only; `continue-on-error` |
| `ci.yml` | `Advisory / Unsafe Audit` | Push only | `cargo-geiger` may lag toolchain; `continue-on-error` |
| `product-proof.yml` | `ci-product advisory canaries` | Scheduled (weekly) + dispatch | Intentionally advisory; `continue-on-error` |
| `criterion-smoke.yml` | `benchmark` | Scheduled (weekly) + dispatch | Non-blocking; `continue-on-error` |
| `ts-bridge-parity.yml` | `parity` | Scheduled (nightly) + dispatch | Non-blocking; `continue-on-error` |
| `clippy-quarantine-report.yml` | `quarantine-report` | Scheduled (weekly) + dispatch | Report only |
| `droid-security-scan.yml` | `droid-security-scan` | Scheduled (weekly) + dispatch | Advisory scan; `continue-on-error` |
| `fuzz.yml` | `fuzz` | Scheduled + labeled PR + dispatch | Fuzz targets; time-boxed |
| `droid-review.yml` | `droid-review` | PR (non-draft) | AI review; `continue-on-error` |
| `droid.yml` | `droid` | @droid mentions | AI assistant; `continue-on-error` |

---

## Advisory job name convention

Advisory jobs in `ci.yml` carry the `Advisory / ` prefix so the GitHub Checks
UI makes their non-blocking nature immediately visible. The following renames
have already been applied:

| New name (current) | Previous name |
|--------------------|---------------|
| `Advisory / Miri` | `Miri (UB Detection)` |
| `Advisory / Sanitizers` | `Sanitizers (ASAN/UBSAN)` |
| `Advisory / Minimal Versions` | `Minimal Versions` |
| `Advisory / Cross Compilation (${{ matrix.target }})` | `Cross Compilation (...)` |
| `Advisory / WASM Build` | `WASM Build Verification` |
| `Advisory / Unsafe Audit` | `Unsafe Code Audit` |

Jobs in other workflows already carry clear names or are inherently advisory
(scheduled/dispatch-only).

---

## Branch protection

Current required status check (via `.github/settings.yml`):

```yaml
required_status_checks:
  contexts:
    - "CI / ci-supported"
```

This is correct and intentionally single-gated. All other checks are optional
signal.

---

## How to read the GitHub Checks panel

1. **`CI / ci-supported` red?** — Stop. Fix before merge.
2. **`PR Gate / PR Gate Success` red?** — Same thing (aggregate gate).
3. **Any `Advisory / *` red?** — Inspect when convenient. May be nightly drift.
4. **Push-only jobs red on main?** — Create a follow-up issue. Not a PR blocker.
5. **PR-only signal red?** — Worth reviewing, but not a merge blocker.

---

## Relationship to other docs

- **`docs/status/KNOWN_RED.md`** — Tracks intentional exclusions from the supported lane.
- **`docs/status/SUPPORT_TIERS.md`** — Maps feature surfaces to proof commands and CI lanes.
- **`.github/CI_README.md`** — General CI infrastructure documentation.
- **This file** — Lane classification and contributor-facing reading guide.

---

## Maintenance

When adding a new CI job:
1. Add it to the appropriate table above.
2. If advisory, use `continue-on-error: true` and the `Advisory / ` name prefix.
3. If required, update `.github/settings.yml` branch protection contexts **and** this file.
4. If push-only, ensure it does not trigger on PRs (use `if:` guards).
