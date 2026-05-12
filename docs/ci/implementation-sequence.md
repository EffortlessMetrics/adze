# CI economics implementation sequence

This is the agent-facing implementation contract for the next CI economics
wave. Treat the work as CI infrastructure product work, not workflow cleanup.
The product invariant is:

> Every PR gets one cheap required proof. Everything else is advisory,
> routed, scheduled, manual, release-only, or label-triggered.

## Default PR shape

Ordinary Rust PRs should normally select this shape:

| Lane | Blocking | Target LEM | Notes |
| --- | ---: | ---: | --- |
| PR Plan | no | ~1 | Changed surfaces, selected lanes, budget band |
| Supported Rust Gate | yes | ~18–22 | `just ci-supported` |
| PR Gate Success | yes | ~1 | Stable required aggregate after promotion |
| CI lane whitelist | advisory | ~1–2 | Prevent workflow sprawl |
| ripr advisory | advisory | ~3–5 | Static oracle-gap signal; skip docs-only |
| test-policy smoke | advisory or blocking | ~1–3 | Disabled-test and inventory hygiene only |

Docs-only PRs should normally select this shape:

| Lane | Blocking | Target LEM |
| --- | ---: | ---: |
| PR Plan | no | ~1 |
| Docs Gate | yes | ~1–2 |
| PR Gate Success | yes | ~1 |
| policy lint | advisory | ~1 |

Everything else is non-default for ordinary PRs unless a changed path, label,
manual dispatch, schedule, `main`, release, or merge-queue rule selects it.

## Static budget contract

The rollout uses static `base_lem` estimates until enough actuals exist.
The current ordinary PR contract is:

| Budget rule | Value |
| --- | ---: |
| Preferred ordinary PR | <=25 LEM |
| Ordinary PR ceiling | <=35 LEM |
| Elevated warning | 36–75 LEM |
| High-cost warning / explicit label recommendation | 76–125 LEM |
| Hard ceiling without `full-ci` or `ci-budget-override` | >125 LEM |

Linux is the unit runner (`1.0`), Windows is `2.0`, and macOS is `10.0`.
Do not add Windows or macOS as default ordinary-PR lanes.

## Source-of-truth files

| File | Role |
| --- | --- |
| `docs/ci/cost-and-verification-policy.md` | Doctrine: LEM, budget bands, verification ladder |
| `docs/ci/implementation-sequence.md` | This executable rollout order and per-PR contract |
| `docs/ci/adze-rollout-plan.md` | Status table and implementation wave ledger |
| `docs/ci/branch-protection.md` | Required-check migration criteria and rollback |
| `docs/ci/labels.md` | Operator label vocabulary for expensive lanes |
| `.github/CI_LANES.md` | Contributor-facing lane map and required/advisory semantics |
| `policy/ci-lane-whitelist.toml` | Machine-readable lane registry and static LEM ledger |
| `policy/ci-risk-packs.toml` | Path/label routing vocabulary |
| `policy/ci-whitelist-exceptions.toml` | Temporary exceptions for expensive default lanes |

When a PR changes user-facing CI semantics, update the docs and the lane
ledger together. When a PR only documents a future phase, do not change
workflow behavior.

## Ordered implementation backlog

The sequence is intentionally SRP. Do not combine branch-protection changes
with workflow pruning, and do not combine docs-only reconciliation with large
routing changes.

| Step | Title | Purpose | Default PR effect | Branch protection impact | Acceptance |
| ---: | --- | --- | --- | --- | --- |
| 1 | `docs(ci): reconcile CI economics rollout status` | Normalize the control-plane docs and status tables. | None. | None. | `cargo xtask check-ci-lane-whitelist --mode advisory || true`; `cargo xtask policy-report || true`; `git diff --check` |
| 2 | `ci(labels): add CI routing labels` | Add stable operator labels in repo settings. | None unless workflows already consume labels. | None. | `git diff --check` |
| 3 | `ci: require PR Gate Success as the stable merge gate` | Make the aggregate check the single required context. | Required check name changes; proof stays `just ci-supported` or Docs Gate. | Required context becomes `PR Gate / PR Gate Success`. | `git diff --check` |
| 4 | `ci: stop running legacy ci.yml on ordinary PRs` | Stop duplicate ordinary-PR execution after PR Gate is authoritative. | Removes legacy duplicate PR cost. | None if step 3 already landed. | `cargo xtask check-ci-lane-whitelist --mode advisory || true`; `git diff --check` |
| 5 | `ci(test-policy): split PR smoke from full inventory enforcement` | Keep cheap hygiene on PRs and route full inventory to main/nightly/manual/labels. | Saves ~9–10 LEM. | Smoke can be advisory or blocking; full lane not required. | `cargo xtask test-policy-smoke`; `cargo xtask test-policy-full || true`; whitelist advisory; `git diff --check` |
| 6 | `ci(pure-rust): make platform proof label and main only` | Remove Ubuntu/stable pure-rust duplicate from ordinary PRs. | Saves ~18 LEM. | None. | whitelist advisory; `git diff --check` |
| 7 | `ci(microcrate): route docs wasm and strict features off ordinary PRs` | Keep matched crate-group tests; move docs/WASM/strict features to labels/main/manual. | Variable savings. | None. | whitelist advisory; `git diff --check` |
| 8 | `ci(perf): keep benchmark smoke default but label-gate comparisons` | Leave compile smoke path-routed; move comparisons to labels/main/manual. | Saves up to ~35 LEM on perf paths. | None. | whitelist advisory; `git diff --check` |
| 9 | `ci(ts-bridge): consolidate smoke and move parity off default PR` | Keep one path-routed smoke; move parity to main/schedule/manual/label. | Removes duplicate ts-bridge default cost. | None. | whitelist advisory; `git diff --check` |
| 10 | `ci(api): route public API checks by API risk` | Only run SemVer/API checks on public API surfaces or labels. | Removes docs/fixture/test-only API cost. | Advisory until release prep. | `cargo semver-checks check-release -p adze || true`; `cargo public-api -p adze || true`; `git diff --check` |
| 11 | `ci(policy): block undeclared workflow lanes` | Make lane-whitelist failures blocking only for workflow/policy/CI-doc changes. | Prevents future workflow sprawl. | None. | `cargo xtask check-ci-lane-whitelist --mode blocking`; `git diff --check` |
| 12 | `ci(plan): enforce static LEM ceilings with override labels` | Warn/fail by static budget band; do not use learned estimates yet. | Over-ceiling PRs fail unless explicitly overridden. | None. | `cargo run -q -p xtask -- ci-plan --base origin/main --head HEAD --json-out target/ci/ci-plan.json`; fallback script; `git diff --check` |
| 13 | `ci: split main smoke from nightly deep verification` | Prevent `main` pushes from becoming the new cost sink. | None. | None. | whitelist advisory; `git diff --check` |
| 14 | `ci(actuals): normalize per-lane LEM receipts` | Emit per-lane wall minutes, multipliers, LEM, selection reason, and result. | None. | None. | `python3 scripts/ci/emit-ci-actuals.py`; `python3 -m json.tool target/ci/ci-actuals.json`; `git diff --check` |
| 15 | `ci(metrics): compute learned lane estimates from actuals` | Produce advisory p50/p90/p95 estimates after enough samples. | None; advisory only. | None. | Learned-estimate generator output validates as JSON/Markdown. |
| 16 | `ci(metrics): update lane LEM baselines from measured actuals` | Ratchet static ledger after sufficient measured evidence. | Updates estimates, not routing. | None. | Docs and `policy/ci-lane-whitelist.toml` updated together. |

## Per-PR operating contract

Every CI economics PR body must include:

```markdown
## CI economics

- Estimated LEM impact:
- Workflows touched:
- Default PR effect:
- Branch protection impact:
- Rollback path:
- Proof obligation:
- Cheaper signal considered:
- Expensive runners added? yes/no
- macOS/windows default PR? must be no

## Verification

- [ ] `cargo xtask check-ci-lane-whitelist --mode advisory`
- [ ] `cargo xtask policy-report || true`
- [ ] `git diff --check`

## Claim boundary

This PR changes CI routing/cost only. It does not weaken `just ci-supported`,
remove deep verification from main/nightly/release/manual paths, or make
advisory lanes required.
```

## Agent rules

1. One CI intention per PR.
2. Never combine branch protection change with large workflow pruning.
3. Never introduce macOS or Windows as default PR lanes.
4. Never add a workflow job without a lane entry.
5. Never make a raw matrix leaf a required branch-protection check.
6. Keep `PR Gate Success` as the stable required summary after promotion.
7. Keep deep verification available through main/nightly/manual/label/release.
8. If a job is `duplicate_of` another lane, justify it or remove/reroute it.
9. If a lane is expensive and `default_pr=true`, it needs an exception and expiry.
10. After each merge, refresh `main` and rerun the lane whitelist check.
