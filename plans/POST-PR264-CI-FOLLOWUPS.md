# Post-PR264 CI And Closeout Follow-Ups

**Status:** Active
**Created:** 2026-04-04
**Context:** PR [#264](https://github.com/EffortlessMetrics/adze/pull/264) merged on 2026-04-03 as `2a88deb6e6095682051729290987a78a0565d613`. The supported merge gate is green on `main`; the remaining work is follow-up hardening, not backlog convergence.

---

## Executive Summary

This plan turns the final last-mile friction from PR #264 into three bounded tracks:

1. backend-selection contract cleanup across feature-unified test surfaces
2. Windows pure-rust benchmark-tail reduction in CI
3. predictable temporary worktree cleanup after merge

The intent is to keep the supported lane green while making the broader workflow surface easier to trust and cheaper to operate.

---

## Tracking Issues

- [Issue #267](https://github.com/EffortlessMetrics/adze/issues/267) — Stabilize backend-selection contract across feature profiles and conflict tests
- [Issue #269](https://github.com/EffortlessMetrics/adze/issues/269) — Reduce and instrument the long Windows pure-rust benchmark-compile tail in CI
- [Issue #268](https://github.com/EffortlessMetrics/adze/issues/268) — Document and harden temporary worktree cleanup to avoid standalone `.git` drift

---

## Execution Order

### 1. Backend-selection contract first

This is the highest-value code/test follow-up because it already caused repeated current-head CI churn during PR #264.

Representative surfaces:
- `crates/parser-backend-core/tests/bdd_parser_backend_core.rs`
- `crates/parser-feature-contract/tests/bdd_parser.rs`
- `crates/runtime-governance/tests/integration_chain.rs`
- baseline contract anchor: `crates/feature-policy-contract/src/lib.rs`

Target outcome:
- one documented contract for conflict-backend behavior
- one shared assertion path instead of repeated panic-string handling
- representative matrix coverage proving the same contract from multiple crate families

### 2. Windows pure-rust tail second

This is an operational CI problem rather than a correctness bug, but it remains merge-friction because it can dominate the final wait on otherwise-green PRs.

Representative surface:
- `.github/workflows/pure-rust-ci.yml`
- job: `Test Pure Rust Implementation`
- final step: `Run benchmarks (check compilation)` running `cargo bench --no-run`

Target outcome:
- step-level timing/observability
- a clear answer on whether the Windows benchmark-compile step belongs on the required PR path
- either a faster path or a consciously reclassified advisory path

### 3. Worktree cleanup hardening third

This does not block CI correctness, but it does affect the safety and repeatability of multi-branch local iteration.

Representative failure:

```text
fatal: validation failed, cannot remove working tree: '/tmp/adze-local-improvements/.git' is not a .git file, error code 2
```

Target outcome:
- one documented convention for temporary worktrees versus standalone temp clones
- one cleanup recipe that validates a path before removing it
- optional helper automation for listing/pruning stale worktree registrations

---

## Guardrails

- Do not reopen PR #264 or treat these tracks as reasons to restart merge triage.
- Keep the supported merge gate explicit: `just ci-supported` remains the required baseline.
- Treat broader CI cleanup as hardening work unless and until a lane is intentionally promoted into the supported contract.
- Update `docs/status/KNOWN_RED.md`, `docs/status/FRICTION_LOG.md`, and `docs/status/NOW_NEXT_LATER.md` in the same change set whenever supported/advisory boundaries move.

---

## Success Criteria

- `main` stays green on the supported lane while these follow-ups land.
- Backend-selection expectations stop drifting across representative feature-unified test crates.
- Windows pure-rust CI no longer spends an opaque, merge-blocking tail on low-signal benchmark compilation.
- Temporary worktree cleanup becomes reproducible and documented enough to avoid manual guesswork.
