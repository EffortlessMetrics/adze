# ADZE-PROP-0002: API foundation

Status: proposed
Owner: Adze maintainers
Created: 2026-05-13
Target milestone: 0.9.0
Linked specs: ADZE-SPEC-0003 canonical parse document; ADZE-SPEC-0004 typed CST and AST projections; ADZE-SPEC-0005 diagnostics and recovery; ADZE-SPEC-0006 Tree-sitter compatibility adapter; ADZE-SPEC-0007 GLR ambiguity summary; ADZE-SPEC-0008 JSON CLI WASM projections; ADZE-SPEC-0009 incremental document lifecycle; ADZE-SPEC-0010 language metadata and node-types
Linked ADRs: ADZE-ADR-0001 AdzeDocument one parse truth
Linked plan:
Linked issues:
Linked PRs:
Support-tier impact: ../status/SUPPORT_TIERS.md
Policy impact: ../policy/package-boundary.toml

## Problem

Adze 0.8 proved that Rust types can define grammar structure and generated
parsers can return typed ASTs. The runtime has accumulated multiple
parse-shaped products — typed AST via `parse()`, Tree-sitter `Tree` via
`ts_compat`, generic CST nodes, typed CST wrappers, structured diagnostics,
GLR ambiguity summaries, experimental JSON serialization — without a single
canonical parse-product boundary that ties them together.

This creates three risks:

1. **Drift.** Each surface can evolve independently. A field label that works
   in `ts_compat` could be missing from typed CST. A diagnostic that appears in
   error output could be absent from the native document.

2. **API sprawl.** Users cannot tell which entry point is the "real" one. The
   README says `parse()` is stable; the code also exposes `parse_document()`,
   `SyntaxNode`, `AdzeDocument`, `ts_compat::Tree`, and multiple parser
   constructors.

3. **Proof ambiguity.** Tests pass per surface, but a passing typed-AST test
   does not prove the native document has the same field metadata unless both
   share the same source of truth.

## Users And Surfaces

Four groups:

- **Rust users** who want `grammar::parse(source)` to produce typed values
  without understanding the internal document model.
- **Grammar authors and tool builders** who need structured diagnostics,
  typed CST, GLR ambiguity insight, or Tree-sitter compatibility for editor
  tooling.
- **CLI and WASM consumers** who need schema-versioned serialized output that
  represents the same facts as the Rust API.
- **Maintainers and agents** who need a bounded workspace where every public
  API claim maps to a spec, a proof command, and a support tier.

## Success Criteria

The API foundation is complete when:

1. `AdzeDocument` is the single canonical native parse product.
2. `parse()` remains the stable ergonomic typed-AST entry point, implemented as
   sugar over `parse_document()` + AST projection.
3. `parse_document()` returns the experimental native document boundary.
4. Typed CST wrappers project over `AdzeDocument`, not a second tree.
5. Typed AST extraction records provenance against document node IDs.
6. Structured diagnostics are document facts, not separate error vectors.
7. `ts_compat` is an adapter over native document data, not a competing parse
   truth.
8. GLR ambiguity summaries are native document facts; full forest is opt-in.
9. JSON/CLI/WASM outputs are schema-versioned projections of document facts.
10. Language metadata and node-types are projections, not separate registries.
11. Every public API surface has a spec, a proof command, and a support tier.

## Proposed Shape

One document, multiple projections:

```
AdzeDocument (canonical, monomorphic)
  |
  +-- parse::<T>()           stable typed AST (ergonomic sugar)
  +-- doc.ast::<T>()         experimental typed AST projection
  +-- doc.syntax()           experimental typed CST projection
  +-- doc.diagnostics()      stabilizing structured diagnostics
  +-- doc.as_tree_sitter()   advisory ts_compat adapter
  +-- doc.ambiguities()      experimental GLR ambiguity summary
  +-- doc.to_json_value()    experimental JSON projection
  +-- LanguageSchema          advisory metadata projection
```

Each projection reads from the same document. No projection owns copied tree
data or independent error state.

## Alternatives Considered

### Tree-sitter tree as the core

Make the `ts_compat::Tree` the central product.

Rejected: Tree-sitter compatibility is an adoption adapter, not Adze's full
native product. It should expose the selected tree, but should not own typed
ASTs, diagnostics, GLR ambiguity, or provenance.

### Generic AdzeDocument<TAst>

Store one typed AST inside the document.

Rejected: Ties a canonical document to one AST projection, complicates
serialization, and makes multiple semantic views harder.

### Separate parse products per output

Generate independent structures for CST, AST, ts_compat, diagnostics, and GLR.

Rejected: Invites drift. A passing S-expression test would not prove the native
document has the same field metadata.

### Delay until 1.0

Ship 0.9 without clarifying the document/projection architecture.

Rejected: The current ambiguity is already causing API sprawl. Waiting makes
the cleanup harder and the workspace larger.

## Specs To Create Or Update

| Spec | Purpose |
| --- | --- |
| ADZE-SPEC-0003 | Canonical parse document behavior |
| ADZE-SPEC-0004 | Typed CST and AST projection behavior |
| ADZE-SPEC-0005 | Diagnostics and recovery behavior |
| ADZE-SPEC-0006 | Tree-sitter compatibility adapter behavior |
| ADZE-SPEC-0007 | GLR ambiguity summary behavior |
| ADZE-SPEC-0008 | JSON / CLI / WASM projection behavior |
| ADZE-SPEC-0009 | Incremental document lifecycle (proposed) |
| ADZE-SPEC-0010 | Language metadata and node-types behavior |

## Architecture Decisions Needed

ADZE-ADR-0001 already records the one-parse-truth decision. No new ADRs are
needed unless implementation reveals a durable choice not covered by the
existing ADR.

## Implementation Campaign Shape

Phase 1: Land specs (this proposal + behavior specs).
Phase 2: Update `active.toml` with spec-linked work items.
Phase 3: Implement spec contracts as code changes, one projection at a time.
Phase 4: Promote surfaces in SUPPORT_TIERS.md as proof accumulates.

## Evidence Plan

Each spec defines its own required evidence and CI proof commands. The
aggregate evidence plan is:

- Every spec has at least one passing proof command.
- `just ci-supported` remains green throughout.
- SUPPORT_TIERS.md maps every surface to its spec and proof.
- PRODUCT_PROOF_MAP.md provides a release-readable summary.

## Risks

1. **Spec overspec.** Specs that are too detailed become implementation
   constraints rather than behavior contracts. Mitigation: specs define what
   must be true, not how it is implemented.
2. **Projection complexity.** Lazy projections over a monomorphic document may
   be harder to implement than parallel parse products. Mitigation: start with
   eager projections and make them lazy only when profiling demands it.
3. **Breaking changes.** Moving `parse()` to sugar over `parse_document()` may
   change error types or panics. Mitigation: keep `parse()` API identical;
   only the internal path changes.

## Non-Goals

- Full GLR forest API.
- Full Tree-sitter compatibility.
- Stable AdzeDocument ABI.
- Stable typed CST API.
- Query parity.
- Production benchmark claims.
- Incremental parsing stability.

## Exit Criteria

This proposal is complete when:

- All eight linked specs exist and are at least `proposed`.
- SUPPORT_TIERS.md references every spec.
- PRODUCT_PROOF_MAP.md maps every surface to a spec and proof command.
- The active goal manifest tracks spec-linked work items.
