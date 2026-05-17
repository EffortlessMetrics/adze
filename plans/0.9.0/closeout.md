# 0.9 Contract Convergence Closeout

Status: complete
Owner: release/product
Created: 2026-05-17
Linked proposal: ../../docs/proposals/ADZE-PROP-0001-0.9-contract-convergence.md
Linked specs:
- ../../docs/specs/ADZE-SPEC-0001-package-surface-boundary.md
- ../../docs/specs/ADZE-SPEC-0002-ci-economics.md
- ../../docs/specs/ADZE-SPEC-0011-product-proof-and-support-tiers.md
Linked plan: ./implementation-plan.md
Active manifest: ../../.adze/goals/active.toml
Support-tier map: ../../docs/status/SUPPORT_TIERS.md
Policy ledgers:
- ../../policy/package-boundary.toml
- ../../policy/ci-lane-whitelist.toml
- ../../policy/clippy-lints.toml
- ../../policy/non-rust-allowlist.toml

## Closeout Summary

The 0.9 contract-convergence campaign is complete as a repo-operating-system
milestone. The active goal manifest has no ready, active, or blocked work items,
and every tracked item has a completed PR receipt.

This closeout does not tag or publish a release. It records that the release
foundation is coherent: package boundaries are classified, temporary
microcrate seams are gone, CI economics are policy-ledgered, Rust/MSRV and
Clippy policy are aligned, stable README claims map to support-tier proof, and
the API foundation work is encoded as source-of-truth specs and proof-backed
implementation slices.

## What Shipped

- Source-of-truth scaffolding for proposals, specs, ADRs, plans, active goals,
  support tiers, and policy ledgers.
- The 0.9 contract-convergence proposal and implementation plan.
- Package-boundary policy and verifier coverage for every workspace package.
- Microcrate-to-SRP collapse closeout: no `owner-module-migration-target`
  entries remain in the release surface.
- Rust 1.95 MSRV/toolchain alignment.
- Clippy policy refresh after the package collapse and MSRV bump.
- CI-economics policy ledgers, lane whitelist checks, and route/calibration
  docs for bounded default proof.
- Product-proof support-tier contract and stable-product lane for README Stable
  claims.
- README capability-tier alignment against `SUPPORT_TIERS.md`, including
  intentionally excluded surfaces.
- API foundation specs and implementation receipts for the native
  `AdzeDocument` direction: document alpha, pure-parser bridge,
  generated `parse_document()`, typed AST projection, typed CST generation,
  document diagnostics, Tree-sitter compatibility projection, ambiguity
  summary, document JSON alpha, and language metadata/node-types.
- Non-Rust policy reporting for Rust migration candidates so follow-up cleanup
  can start from generated policy evidence instead of ad hoc lists.

## What Did Not Ship

- A release tag or crate publication.
- Promotion of runtime2, WASM/browser behavior, broad Tree-sitter compatibility,
  full query parity, full GLR forest export, or document JSON/WASM schemas to
  Stable.
- Branch-protection promotion of the advisory product-proof lanes.
- A stable incremental parsing guarantee.
- Complete burn-down of every non-Rust Rust-migration candidate reported by the
  file-policy checker.

## Evidence

The closeout evidence is intentionally layered:

```bash
python -c "import tomllib; tomllib.load(open('.adze/goals/active.toml','rb'))"
cargo run -q -p xtask -- check-active-goal
cargo run -q -p xtask -- check-doc-artifacts
cargo run -q -p xtask -- check-package-boundary --release-gate
cargo run -q -p xtask -- check-ci-lane-whitelist
just ci-product-stable
git diff --check
```

Release-candidate proof still needs the normal release checklist and supported
gate at release time:

```bash
just ci-supported
PACKAGE_BOUNDARY_RELEASE_GATE=1 ./scripts/validate-release-surface.sh
```

## Support-Tier Changes

`../../docs/status/SUPPORT_TIERS.md` remains the source of truth for product
claims and proof commands. The closeout state is:

- README rows marked Stable are limited to rows with repeatable proof commands.
- Stabilizing, Experimental, Advisory, Future, and Intentionally excluded rows
  remain explicitly tiered.
- Runtime2 and other developing surfaces are not promoted by this closeout.

## Policy Changes

- `../../policy/package-boundary.toml` is the release-surface ledger.
- `../../policy/ci-lane-whitelist.toml` is the CI lane source of truth.
- `../../policy/clippy-lints.toml` records active and planned lint policy.
- `../../policy/non-rust-allowlist.toml` remains the non-Rust exception ledger,
  now paired with generated Rust migration-candidate reporting.
- `../../policy/doc-artifacts.toml` registers this closeout as the milestone
  handoff record.

## Known Gaps

- Run the final release checklist before tagging or publishing.
- Decide whether and when `ci-product-stable` should become a required branch
  protection gate.
- Continue API foundation work without promoting developing projections faster
  than their support-tier proof.
- Use `cargo run -q -p xtask -- check-file-policy --mode advisory` to burn down
  Rust migration candidates as follow-up SRP/source-hygiene work.

## Follow-Up Issues

- GLR product proof remains bounded by the support-tier rows and exact canaries.
- Tablegen ABI completeness remains Stabilizing until broader generated-language
  roundtrip proof exists.
- Parse diagnostics remain Stabilizing until cross-path canaries justify
  promotion.
- Incremental document lifecycle remains post-0.9 unless a new accepted plan
  moves it into scope.

## Rollback

Reverting this closeout only removes the handoff record and status alignment.
It does not revert the package-boundary collapse, MSRV bump, lint policy,
support-tier proof, CI policy, or API foundation implementation receipts.
