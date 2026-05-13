# 0.9 Microcrate To SRP Submodule Transition Plan

Status: active
Owner: release/package
Created: 2026-05-13
Linked proposal: ../../docs/proposals/ADZE-PROP-0001-0.9-contract-convergence.md
Linked spec: ../../docs/specs/ADZE-SPEC-0001-package-surface-boundary.md
Linked ADR: ../../docs/adr/ADZE-ADR-0002-no-durable-unpublished-production-crates.md
Active goal: ../../.adze/goals/active.toml
Policy ledger: ../../policy/package-boundary.toml

## Goal

Before the next release, transition migration-target microcrates into SRP
submodules under their owning public crate, dev-only crate, or xtask/tooling
module.

This is a release prerequisite, not a documentation preference. A package can
remain a standalone crate at release only when it is a published surface or a
durable dev-only/tooling surface with a current owner and proof rationale.

## Current State

As of the post-#697 workspace:

```text
published packages: 10
dev-only packages: 15
owner-module migration targets: 52
```

The package-boundary ledger is the source of truth for the exact package list.

## Target Shape

For each migration target, choose exactly one outcome:

1. **Delete** it when it has no live external consumer.
2. **Inline** it into the single crate that consumes it.
3. **Move** it into an SRP submodule under the owning crate or xtask module.
4. **Reclassify** it only with an accepted ADR explaining why it remains a
   standalone crate before release.

The expected end state before release:

```text
owner-module migration targets: 0
```

## Work Item: retire-zero-reverse-dependency-facades

Status: active
Linked PRs: #696, #697

### Goal

Remove facade crates that have no reverse dependencies and only re-export or
wrap another owner surface.

### Production Delta

Delete the crate directory, remove the workspace member, remove lockfile and
release-list entries, update policy ledgers, update microcrate CI routing, and
run package-boundary proof.

### Proof Commands

```bash
cargo metadata --format-version 1 --no-deps
cargo run -q -p xtask -- check-package-boundary
cargo run -q -p xtask -- check-ci-lane-whitelist
just ci-supported
```

## Work Item: parser-contracts-to-srp-submodules

Status: ready
Owner: parser-contracts

### Goal

Move parser contract/support crates into SRP submodules under their actual
consumer, or remove them when they are only test scaffolding.

### Candidate Packages

- `adze-grammar-analysis-core`
- `adze-parser-backend-core`
- `adze-parser-contract`
- `adze-parser-feature-profile-core`
- `adze-parser-governance-contract`
- `adze-parsetable-metadata`

### Proof Commands

```bash
cargo metadata --format-version 1 --no-deps
cargo run -q -p xtask -- check-package-boundary
cargo test -p adze-parser-backend-core -p adze-parser-contract -p adze-parser-governance-contract -p adze-parsetable-metadata -- --test-threads=2
just ci-supported
```

## Work Item: governance-to-xtask-srp-submodules

Status: ready
Owner: governance/policy

### Goal

Move governance matrix/status/runtime reporting microcrates into xtask or the
single policy owner that consumes them.

### Candidate Packages

- `adze-governance-contract`
- `adze-governance-matrix-contract`
- `adze-governance-matrix-core`
- `adze-governance-matrix-core-impl`
- `adze-governance-metadata`
- `adze-governance-runtime-core`
- `adze-governance-runtime-profile-core`
- `adze-governance-runtime-reporting`
- `adze-governance-status-core`

### Proof Commands

```bash
cargo metadata --format-version 1 --no-deps
cargo run -q -p xtask -- check-package-boundary
cargo test -p xtask --bin xtask policy -- --nocapture
just ci-supported
```

## Work Item: concurrency-to-srp-submodules

Status: ready
Owner: governance/concurrency

### Goal

Collapse concurrency policy/configuration helper crates into one owner module or
remove unused seams.

### Candidate Packages

- `adze-concurrency-bounded-map-core`
- `adze-concurrency-caps-contract-core`
- `adze-concurrency-caps-core`
- `adze-concurrency-env-contract-core`
- `adze-concurrency-init-core`
- `adze-concurrency-init-rayon-core`
- `adze-concurrency-map-core`
- `adze-concurrency-normalize-core`
- `adze-concurrency-parse-core`
- `adze-concurrency-plan-core`

### Proof Commands

```bash
cargo metadata --format-version 1 --no-deps
cargo run -q -p xtask -- check-package-boundary
cargo test -p adze-concurrency-caps-core -p adze-concurrency-env-contract-core -p adze-concurrency-init-core -- --test-threads=2
just ci-supported
```

## Work Item: bdd-to-srp-submodules

Status: ready
Owner: governance/bdd

### Goal

Collapse BDD/governance fixture and contract crates into one owner module or
test support module.

### Candidate Packages

- `adze-bdd-contract`
- `adze-bdd-governance-contract`
- `adze-bdd-governance-core`
- `adze-bdd-governance-fixtures`
- `adze-bdd-grammar-fixtures`
- `adze-bdd-grid-contract`
- `adze-bdd-grid-core`
- `adze-bdd-scenario-fixtures`

### Proof Commands

```bash
cargo metadata --format-version 1 --no-deps
cargo run -q -p xtask -- check-package-boundary
cargo test -p adze-bdd-governance-core -p adze-bdd-grid-core -p adze-bdd-grammar-fixtures -- --test-threads=2
just ci-supported
```

## Work Item: runtime-governance-to-srp-submodules

Status: ready
Owner: runtime-governance

### Goal

Move runtime governance support crates into the runtime, runtime2, or xtask
owner modules that consume them.

### Candidate Packages

- `adze-runtime-governance`
- `adze-runtime-governance-api`
- `adze-runtime-governance-matrix`
- `adze-runtime2-governance`

### Proof Commands

```bash
cargo metadata --format-version 1 --no-deps
cargo run -q -p xtask -- check-package-boundary
cargo test -p adze-runtime-governance -p adze-runtime-governance-api -- --test-threads=2
just ci-supported
```

## Work Item: source-location-and-formatting-srp-submodules

Status: ready
Owner: diagnostics/source-location and syntax-formatting

### Goal

Move source-location, stack-pool, syntax-formatting, and table-metadata helpers
into the owner crate where each API is actually used, unless a release-facing
reason keeps it standalone.

### Candidate Packages

- `adze-error-location-core`
- `adze-linecol-core`
- `adze-stack-pool-core`
- `adze-ts-format-core`

### Proof Commands

```bash
cargo metadata --format-version 1 --no-deps
cargo run -q -p xtask -- check-package-boundary
cargo test -p adze-linecol-core -p adze-error-location-core -p adze-stack-pool-core -p adze-ts-format-core -- --test-threads=2
just ci-supported
```

## Release Gate

Before the next release:

```bash
cargo run -q -p xtask -- check-package-boundary
```

must report no unclassified packages and the release checklist must confirm no
remaining `owner-module-migration-target` entries in `policy/package-boundary.toml`.

If any migration target remains, release is blocked unless a new accepted ADR
reclassifies that package as a durable standalone crate.

## Rollback

Each owner-sized transition PR should be independently revertible. Rollback must
restore workspace membership, lockfile entries, release lists, policy ledger
classification, CI routing, and docs references for the affected owner group.
