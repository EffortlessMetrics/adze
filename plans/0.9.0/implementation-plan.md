# Adze 0.9.0 Contract Convergence Implementation Plan

Status: proposed
Owner: Adze maintainers
Created: 2026-05-12
Linked proposal: ../../docs/proposals/ADZE-PROP-0001-0.9-contract-convergence.md
Linked specs:
- ../../docs/specs/ADZE-SPEC-0001-package-surface-boundary.md
- ../../docs/specs/ADZE-SPEC-0002-ci-economics.md
Linked ADRs:
- ../../docs/adr/ADZE-ADR-0001-adze-document-one-parse-truth.md
Active goal: ../../.adze/goals/active.toml
Support-tier map: ../../docs/status/SUPPORT_TIERS.md

## Campaign Goal

Make 0.9.0 a contract-convergence release: reduce hidden workspace surface,
keep the supported core proof green, recalibrate CI economics, and promote only
the product claims that have support-tier evidence.

This plan sequences the work. It does not own the behavior contracts or policy
ledgers; those live in linked specs and `../../policy/*.toml`.

## Current Stack

The first docs stack establishes the source-of-truth model before product or
policy changes:

| PR | Work item | Artifact |
| --- | --- | --- |
| #681 | source-of-truth-scaffolding | README files for proposals, specs, ADRs, 0.9 plans, and active goals |
| #682 | contract-convergence-proposal | `ADZE-PROP-0001` |
| #683 | package-boundary-spec | `ADZE-SPEC-0001` |
| #684 | ci-economics-spec | `ADZE-SPEC-0002` |
| #685 | adze-document-adr | `ADZE-ADR-0001` |

Later PRs should rebase or retarget as earlier stack entries land.

## Work Item: source-of-truth-scaffolding

Status: active
Linked proposal:
Linked spec:
Linked ADR:
Blocks: contract-convergence-proposal; package-boundary-spec; ci-economics-spec; adze-document-adr; contract-convergence-plan
Blocked by: none

### Goal

Define where proposals, specs, ADRs, implementation plans, and active goal
manifests live.

### Production Delta

Add README scaffolding under:

- `../../docs/proposals/`
- `../../docs/specs/`
- `../../docs/adr/`
- `../../plans/0.9.0/`
- `../../.adze/goals/`

### Non-Goals

No behavior specs, policy ledgers, parser/runtime changes, or package moves.

### Acceptance

Each documentation layer states what it owns and what it must not duplicate.

### Proof Commands

```bash
git diff --check
```

### Rollback

Revert the scaffold PR.

## Work Item: contract-convergence-proposal

Status: active
Linked proposal: ../../docs/proposals/ADZE-PROP-0001-0.9-contract-convergence.md
Linked spec:
Linked ADR:
Blocks: package-boundary-spec; ci-economics-spec; contract-convergence-plan
Blocked by: source-of-truth-scaffolding

### Goal

Record why 0.9.0 is a contract-convergence milestone rather than a broad
feature-expansion milestone.

### Production Delta

Add `ADZE-PROP-0001`.

### Non-Goals

No package policy, CI implementation, parser/runtime changes, or support-tier
promotion.

### Acceptance

The proposal names the problem, users, success criteria, alternatives, evidence
plan, risks, non-goals, and exit criteria.

### Proof Commands

```bash
git diff --check
```

### Rollback

Revert the proposal PR.

## Work Item: package-boundary-spec

Status: active
Linked proposal: ../../docs/proposals/ADZE-PROP-0001-0.9-contract-convergence.md
Linked spec: ../../docs/specs/ADZE-SPEC-0001-package-surface-boundary.md
Linked ADR: ADZE-ADR-0002 no durable unpublished production crates
Blocks: package-boundary-audit; microcrate-collapse
Blocked by: contract-convergence-proposal

### Goal

Define the package categories and evidence required before collapsing or
reclassifying workspace packages.

### Production Delta

Add `ADZE-SPEC-0001`.

### Non-Goals

No package moves, verifier implementation, or policy TOML changes.

### Acceptance

The spec states that every workspace package must be a published crate,
dev-only crate, or owner-module migration target, and that there is no durable
unpublished production crate category.

### Proof Commands

```bash
git diff --check
```

### Rollback

Revert the spec PR.

## Work Item: ci-economics-spec

Status: active
Linked proposal: ../../docs/proposals/ADZE-PROP-0001-0.9-contract-convergence.md
Linked spec: ../../docs/specs/ADZE-SPEC-0002-ci-economics.md
Linked ADR:
Blocks: ci-economics-verifier; ci-lem-refresh
Blocked by: contract-convergence-proposal

### Goal

Define CI lane tiers, LEM bands, required lane metadata, evidence, and promotion
rules without copying the whitelist or risk-pack ledgers.

### Production Delta

Add `ADZE-SPEC-0002`.

### Non-Goals

No workflow, branch-protection, or policy TOML changes.

### Acceptance

The spec links to `../../policy/ci-lane-whitelist.toml`,
`../../policy/ci-risk-packs.toml`, and the existing CI policy docs as sources of
truth for concrete lane data.

### Proof Commands

```bash
git diff --check
```

### Rollback

Revert the spec PR.

## Work Item: adze-document-adr

Status: active
Linked proposal: ../../docs/proposals/ADZE-PROP-0001-0.9-contract-convergence.md
Linked spec: ADZE-SPEC-0003 canonical parse document
Linked ADR: ../../docs/adr/ADZE-ADR-0001-adze-document-one-parse-truth.md
Blocks: canonical-parse-document-spec; typed-cst-plan; ts-compat-over-document-plan
Blocked by: contract-convergence-proposal

### Goal

Record the durable architecture decision that `AdzeDocument` is one parse truth
with multiple projections.

### Production Delta

Add `ADZE-ADR-0001`.

### Non-Goals

No API stabilization, parser/runtime changes, or support-tier promotion.

### Acceptance

The ADR rejects Tree-sitter-as-core, generic `AdzeDocument<TAst>`, independent
parse products per output, and raw-forest-first native API design.

### Proof Commands

```bash
git diff --check
```

### Rollback

Revert the ADR PR.

## Work Item: contract-convergence-plan

Status: active
Linked proposal: ../../docs/proposals/ADZE-PROP-0001-0.9-contract-convergence.md
Linked spec:
Linked ADR:
Blocks: package-boundary-audit; ci-economics-verifier; microcrate-collapse
Blocked by: source-of-truth-scaffolding; contract-convergence-proposal

### Goal

Add this implementation plan and the active goal manifest so agents can follow
the campaign without scraping long prose.

### Production Delta

Add:

- `implementation-plan.md`
- `../../.adze/goals/active.toml`

### Non-Goals

No policy verifier, package move, runtime change, or support-tier promotion.

### Acceptance

The plan lists PR-sized work items with linked artifacts, non-goals, acceptance
criteria, proof commands, and rollback notes. The active goal manifest exposes
the same campaign state in machine-readable form.

### Proof Commands

```bash
git diff --check
python -c "import tomllib; tomllib.load(open('.adze/goals/active.toml', 'rb'))"
```

### Rollback

Revert the plan/manifest PR.

## Work Item: package-boundary-audit

Status: ready
Linked proposal: ../../docs/proposals/ADZE-PROP-0001-0.9-contract-convergence.md
Linked spec: ../../docs/specs/ADZE-SPEC-0001-package-surface-boundary.md
Linked ADR: ADZE-ADR-0002 no durable unpublished production crates
Blocks: microcrate-collapse; rust-1.95-msrv-bump; clippy-policy-refresh
Blocked by: package-boundary-spec; contract-convergence-plan

### Goal

Classify every workspace package and add verifier coverage for the package
surface boundary.

### Production Delta

Expected later changes:

- `../../policy/package-boundary.toml`
- package-boundary verifier command
- verifier tests
- support-tier or CI policy updates if the classification affects claims or
  routing

### Non-Goals

No package moves in the audit PR. Classification comes before collapse.

### Acceptance

Every workspace package is classified as a published crate, dev-only crate, or
owner-module migration target. No package is classified as durable unpublished
production code.

### Proof Commands

```bash
cargo metadata --format-version 1 --no-deps
cargo run -q -p xtask -- check-package-boundary
just ci-supported
```

### Rollback

Revert the verifier and ledger PR. Later package-collapse PRs must not proceed
without a replacement classification source of truth.

## Work Item: ci-economics-verifier

Status: ready
Linked proposal: ../../docs/proposals/ADZE-PROP-0001-0.9-contract-convergence.md
Linked spec: ../../docs/specs/ADZE-SPEC-0002-ci-economics.md
Linked ADR:
Blocks: ci-lem-refresh; microcrate-collapse-ci-routing
Blocked by: ci-economics-spec; contract-convergence-plan

### Goal

Ensure workflow jobs, lane tiers, risk packs, and LEM behavior are represented
in policy ledgers and checked by automation.

### Production Delta

Expected later changes:

- `../../policy/ci-lane-whitelist.toml`
- `../../policy/ci-risk-packs.toml`
- CI whitelist verifier updates
- PR Plan or equivalent LEM output updates

### Non-Goals

No branch-protection promotion and no learned-estimate enforcement before the
required actuals window exists.

### Acceptance

Every workflow job is whitelisted or exempted, ordinary PRs show visible LEM
estimates, and over-ceiling PRs require an explicit override.

### Proof Commands

```bash
cargo run -q -p xtask -- check-ci-lane-whitelist
cargo run -q -p xtask -- ci plan
just ci-supported
```

### Rollback

Revert verifier or policy changes; broad lanes return to their previous routing
until a replacement policy lands.

## Work Item: microcrate-collapse

Status: blocked
Linked proposal: ../../docs/proposals/ADZE-PROP-0001-0.9-contract-convergence.md
Linked spec: ../../docs/specs/ADZE-SPEC-0001-package-surface-boundary.md
Linked ADR: ADZE-ADR-0002 no durable unpublished production crates
Blocks: rust-1.95-msrv-bump; clippy-policy-refresh; ci-lem-refresh
Blocked by: package-boundary-audit

### Goal

Move or remove temporary owner-module migration targets in owner-sized batches.

### Production Delta

Later PRs may change workspace membership, module ownership, crate imports,
policy ledgers, CI routing, and docs that name package boundaries.

### Non-Goals

No opportunistic parser behavior changes or support-tier promotion.

### Acceptance

Each package-collapse PR removes a documented migration target or reclassifies
it with a new accepted contract. The supported gate remains green.

### Proof Commands

```bash
cargo metadata --format-version 1 --no-deps
cargo run -q -p xtask -- check-package-boundary
just check-msrv
just ci-supported
```

### Rollback

Revert the owner-sized package-collapse PR and restore the prior policy ledger
entry.

## Work Item: rust-1.95-msrv-bump

Status: blocked
Linked proposal: ../../docs/proposals/ADZE-PROP-0001-0.9-contract-convergence.md
Linked spec:
Linked ADR:
Blocks: clippy-policy-refresh
Blocked by: microcrate-collapse

### Goal

Bump workspace MSRV/toolchain policy after the package graph is smaller.

### Production Delta

Expected later changes include toolchain files, manifest `rust-version` fields,
CI setup, docs, and any xtask doctor or policy checks that enforce MSRV.

### Non-Goals

No lint promotion or package collapse in the same PR.

### Acceptance

All MSRV declarations agree and the supported gate is green.

### Proof Commands

```bash
just check-msrv
just ci-supported
```

### Rollback

Revert the MSRV bump PR.

## Work Item: clippy-policy-refresh

Status: blocked
Linked proposal: ../../docs/proposals/ADZE-PROP-0001-0.9-contract-convergence.md
Linked spec:
Linked ADR:
Blocks: product-proof-refresh
Blocked by: rust-1.95-msrv-bump

### Goal

Promote planned lint policy only after package collapse and MSRV bump reduce
workspace churn.

### Production Delta

Expected later changes:

- `../../policy/clippy-lints.toml`
- lint verifier updates
- targeted code fixes where required

### Non-Goals

No broad refactors hidden behind lint updates.

### Acceptance

Lint policy checks pass and any new `#[expect]` usage is justified by policy
rather than used to avoid straightforward cleanup.

### Proof Commands

```bash
cargo run -q -p xtask -- check-lint-policy
just clippy
just ci-supported
```

### Rollback

Revert the lint policy PR and any mechanical fixes tied only to that promotion.

## Work Item: product-proof-refresh

Status: blocked
Linked proposal: ../../docs/proposals/ADZE-PROP-0001-0.9-contract-convergence.md
Linked spec: ADZE-SPEC-0004 product proof and support tiers
Linked ADR:
Blocks: 0.9-closeout
Blocked by: clippy-policy-refresh; ci-economics-verifier

### Goal

Refresh stable README claims, support-tier mapping, and product-proof canaries
after package and CI policy changes land.

### Production Delta

Expected later changes:

- `../../docs/status/SUPPORT_TIERS.md`
- README or tutorial claim wording if needed
- exact product-proof canaries for promoted surfaces

### Non-Goals

No promotion of runtime2, WASM, Tree-sitter compatibility, CLI output, or broad
grammar support without receipts.

### Acceptance

Every stable README claim maps to a support-tier row and repeatable proof
command.

### Proof Commands

```bash
just ci-supported
cargo test -p adze --features pure-rust --test typed_ast_contract -- --nocapture
cargo test -p adze-cli readme_arithmetic_quickstart_builds_and_runs -- --exact --nocapture
```

### Rollback

Revert claim wording or support-tier promotion until the missing proof lands.
