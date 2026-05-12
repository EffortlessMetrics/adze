## Summary

<!-- What does this PR do and why? -->

## Test Plan

- [ ] Ran `cargo fmt --all --check`
- [ ] Ran `cargo clippy --all -- -D warnings`
- [ ] Ran `cargo test` (or relevant subset)
- [ ] Snapshot tests updated with `cargo insta review` (if applicable)

## CI economics

<!--
LEM = wall-clock job minutes × runner multiplier. See docs/ci/lem-budgeting.md
and docs/ci/adze-rollout-plan.md. Fill this out for CI/routing/policy PRs and
for any PR that is elevated (>35 LEM) or opts into heavy labels.
-->

- Estimated LEM impact: <!-- e.g. ~25 ordinary / saves ~18 default PR LEM -->
- Workflows touched: <!-- e.g. none / pr-plan.yml / pr-gate.yml -->
- Default PR effect: <!-- e.g. none / adds advisory step / removes duplicate PR lane -->
- Branch protection impact: <!-- usually: none -->
- Rollback path: <!-- exact revert/settings/workflow trigger rollback -->
- Proof obligation: <!-- command or signal proving the change -->
- Cheaper signal considered: <!-- e.g. smoke instead of full matrix -->
- Expensive runners added? <!-- yes/no -->
- macOS/windows default PR? <!-- must be no -->

## CI economics verification (CI-routing PRs)

- [ ] Ran `cargo xtask check-ci-lane-whitelist --mode advisory`
- [ ] Ran `cargo xtask policy-report || true`
- [ ] Ran `git diff --check`
- [ ] Claim boundary: this PR does not weaken `just ci-supported`, remove deep verification from main/nightly/release/manual paths, or make advisory lanes required.

## Notes for Reviewers

<!-- Breaking changes, design decisions, areas to focus review on, etc. -->
