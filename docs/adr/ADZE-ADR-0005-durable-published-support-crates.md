# ADZE-ADR-0005: Durable Published Support Crates

Status: accepted
Date: 2026-05-14
Owner: release/package
Linked proposal: ../proposals/ADZE-PROP-0001-0.9-contract-convergence.md
Linked specs: ../specs/ADZE-SPEC-0001-package-surface-boundary.md
Linked plan: ../../plans/0.9.0/microcrate-collapse.md

## Decision

The following workspace packages are durable published support crates for the
0.9 release surface, not owner-module migration targets:

- `adze-bdd-governance-core`
- `adze-linecol-core`
- `adze-parsetable-metadata`

They remain separate because each owns a narrow cross-crate contract consumed by
more than one production package:

- `adze-bdd-governance-core` owns BDD governance grids, backend feature
  profiles, and runtime governance reporting used by GLR, tablegen, runtime,
  runtime2, and governance proof tests.
- `adze-linecol-core` owns byte-oriented line/column tracking used by both
  runtime implementations and fuzz/test surfaces.
- `adze-parsetable-metadata` owns serialized parse-table metadata shared by
  table generation and runtime loading.

These crates are public support surfaces. Their existence does not promote any
parser, GLR, Tree-sitter compatibility, WASM, CLI, or diagnostics claim by
itself. Product claims still require support-tier proof.

## Context

ADZE-ADR-0002 rejects durable unpublished production crates. The 0.9
microcrate-collapse work removed many facade and temporary production crates by
moving them into SRP owner modules. After that collapse, these three packages
remain as small cross-crate boundaries with direct production consumers.

Forcing these support crates into one consumer would either invert dependency
direction or make unrelated crates depend on heavier owners:

- moving `adze-linecol-core` into `adze` would leave `adze-runtime` without an
  independent source-location primitive;
- moving `adze-parsetable-metadata` into tablegen would make runtime loading
  depend on a code generation owner for serialized table contracts;
- moving `adze-bdd-governance-core` into runtime, runtime2, tablegen, or GLR
  would make the remaining consumers depend on the wrong owner for governance
  proof contracts.

The package-boundary ledger already supports this release state through the
`published` category. The release-blocking category is only
`owner-module-migration-target`.

## Consequences

- These three packages must keep release metadata, publish intent, and package
  boundary entries current.
- They remain eligible for `scripts/release-crates.txt`.
- They must not be used as a shortcut for stable product claims. Stable claims
  still map through `docs/status/SUPPORT_TIERS.md`.
- If any package stops being a cross-crate production contract, it should be
  collapsed into its owner module in a future package-boundary PR.
- The release gate may pass with these packages classified as `published`.

## Alternatives Considered

### Collapse `adze-linecol-core` into runtime

Rejected. Both runtime implementations need the primitive. Moving it into one
runtime would create the wrong dependency direction for the other.

### Collapse `adze-parsetable-metadata` into tablegen

Rejected. Runtime parse-table loading needs the same metadata contract without
depending on tablegen as the semantic owner of serialized runtime data.

### Collapse `adze-bdd-governance-core` into one runtime or tablegen owner

Rejected. The governance grid/profile contract is consumed across GLR,
tablegen, runtime, runtime2, and governance proof tests. No single consumer is
the honest owner.

### Keep them as migration targets

Rejected. Migration targets are explicitly pre-release transition states and
must not remain in the release-gate ledger.

## Follow-Up Specs And Plans

- `../specs/ADZE-SPEC-0001-package-surface-boundary.md`
- `../../plans/0.9.0/microcrate-collapse.md`
- `../../policy/package-boundary.toml`
