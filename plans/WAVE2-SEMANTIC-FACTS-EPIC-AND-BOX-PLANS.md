# Wave 2 Epic: Canonical Semantic Facts (No Provider Migration)

**Status:** Planned  
**Created:** 2026-04-29  
**Wave Goal:** Make semantic facts real and queryable while preserving existing user-visible behavior.

---

## Executive Summary

Wave 2 introduces canonical semantic facts as a new substrate across `perl-symbol`, exporter analysis, and `perl-workspace`, while intentionally deferring provider migration (completion, diagnostics, rename/safe-delete).

This wave is successful if we can produce and store canonical facts, derive deterministic definition/reference indexes from them, and compare old-vs-new query outcomes with deterministic receipts—without changing existing API behavior by default.

### Core constraint

> **Most reviewable and tested complete slice** per box.

This wave optimizes for correctness evidence and migration safety, not minimal diff size.

---

## Non-Goals (Explicitly Deferred)

Wave 2 must **not** include:

- completion migration
- undefined-symbol diagnostics migration
- rename/safe-delete migration
- full package graph
- Moose/Moo generated member support
- external `@INC` / CPAN index
- on-disk semantic persistence
- full type/value-shape inference

---

## Target Architecture After Wave 2

```text
perl-symbol / exporter / workspace
  → emit canonical facts

perl-workspace
  → stores fact shards
  → builds typed definition/reference indexes
  → keeps old public APIs working
  → can compare old vs new query answers
```

Interpretation:

- `perl-symbol` is a producer of declaration/reference facts.
- exporter analysis is a producer of export facts.
- `perl-workspace` becomes a derived index/query layer over facts, not the long-term owner of ad-hoc semantic truth.

---

## Delivery Shape: 8 Parallel Boxes

| Box | Scope | Provider behavior change? |
|---:|---|---|
| 1 | `SymbolDecl -> EntityFact` adapter | No |
| 2 | `SymbolRef -> OccurrenceFact` adapter | No |
| 3 | `ExportInfo -> ExportSet` adapter | No |
| 4 | `FileFactShard` write-through in workspace | No |
| 5 | `DefinitionCandidate` multimap behind compatibility APIs | No / shadow-only acceptable |
| 6 | Typed `ReferenceEdge` global index behind compatibility APIs | No / shadow-only acceptable |
| 7 | Shadow-compare receipts for query migration | No |
| 8 | Semantic scorecard v1 (counts + fixture coverage) | No |

---

## Merge Order (Required)

Do not merge in naive numeric order. Merge in this sequence:

1. **Boxes 1–3** (exact adapters)
2. **Box 8** (scorecard v1), if adapter outputs are consumed cleanly
3. **Box 4** (`FileFactShard` write-through)
4. **Boxes 5–6** (definition/reference indexes), one at a time
5. **Box 7** (shadow receipts)

If Box 4 is clean and 5–6 are both ready, merge Box 4 first, then cascade updates into 5–6.

---

## Box Plans

### Box 1 — `SymbolDecl -> EntityFact`

**Goal:** Implement the most reviewable and tested complete adapter from `perl-symbol::surface::SymbolDecl` to canonical semantic facts.

**Likely files:**
- `crates/perl-symbol/src/surface/decl.rs`
- `crates/perl-symbol/src/surface/mod.rs`
- `crates/perl-symbol/tests/...`
- `crates/perl-semantic-facts/src/...`

**In scope:**
- Add adapter function/module (e.g. `facts_from_symbol_decl(...)`).
- Emit `AnchorFact` for declaration spans.
- Emit `EntityFact` for declaration identity.
- Emit `EdgeFact::Defines` where supported.
- Cover currently projected declaration families (package/class/subroutine/method/variable/constant/label/format as applicable).
- Add deterministic/golden tests in existing fixture style.

**Done when:**
- Existing `SymbolDecl` behavior unchanged.
- Deterministic fact output under test.
- Unsupported declaration kinds are explicit.

**Verification:**
- `cargo test -p perl-symbol`
- `cargo test -p perl-semantic-facts`
- `cargo check --workspace --all-targets`

**Suggested PR title:**
- `feat(perl-symbol): adapt SymbolDecl into canonical semantic facts`

---

### Box 2 — `SymbolRef -> OccurrenceFact`

**Goal:** Implement the most reviewable and tested complete adapter from phase-1 `SymbolRef` to canonical occurrence/reference facts.

**Likely files:**
- `crates/perl-symbol/src/surface/ref.rs`
- `crates/perl-symbol/tests/...`
- `crates/perl-semantic-facts/src/...`

**In scope:**
- Add adapter (e.g. `facts_from_symbol_ref(...)`).
- Emit `AnchorFact` for reference spans.
- Emit `OccurrenceFact` for supported reference categories.
- Emit reference edges where supported.
- Preserve phase-1 coverage boundaries.
- Add deterministic/golden tests for currently supported categories.

**Done when:**
- Existing `SymbolRef` tests still pass.
- New adapter tests prove exact phase-1 output.
- Excluded phase-2 families are documented.

**Verification:**
- `cargo test -p perl-symbol`
- `cargo test -p perl-semantic-facts`
- `cargo check --workspace --all-targets`

**Suggested PR title:**
- `feat(perl-symbol): adapt SymbolRef into canonical occurrence facts`

---

### Box 3 — `ExportInfo -> ExportSet`

**Goal:** Implement the most reviewable and tested complete adapter from exporter analysis into canonical export facts.

**Likely files:**
- `crates/perl-semantic-analyzer/src/analysis/export_analyzer.rs`
- `crates/perl-semantic-analyzer/tests/...`
- `crates/perl-semantic-facts/src/...`

**In scope:**
- Add `ExportInfo -> ExportSet` conversion.
- Emit default exports.
- Emit optional exports.
- Emit export tag/group facts.
- Preserve provenance as import/export inference classification.
- Add fixtures for `@EXPORT`, `@EXPORT_OK`, `%EXPORT_TAGS`.

**Done when:**
- Existing exporter tests remain green.
- Deterministic mapping tests pass.
- Dynamic/unsupported patterns represented conservatively.

**Verification:**
- `cargo test -p perl-semantic-analyzer export`
- `cargo test -p perl-semantic-facts`
- `cargo check --workspace --all-targets`

**Suggested PR title:**
- `feat(exports): adapt ExportSymbolExtractor output into semantic ExportSet facts`

---

### Box 4 — `FileFactShard` write-through store

**Goal:** Add first write-through fact storage path in `perl-workspace`, preserving current query behavior.

**Likely files:**
- `crates/perl-workspace-index/src/workspace/...`
- `crates/perl-workspace-index/src/lib.rs`
- `crates/perl-workspace-index/tests/...`
- `crates/perl-semantic-facts/src/...`

**In scope:**
- Introduce `FileFactShard` with file identity, content hash, anchors/entities/occurrences/edges.
- Add shard storage lifecycle in workspace state.
- Populate from available adapters (allow partial population).
- Add lifecycle tests for add/reindex/remove/clear.

**Done when:**
- Deterministic shard lifecycle behavior.
- Reindex replaces stale file facts.
- Removing file removes shard.
- Existing behavior unchanged.

**Verification:**
- `cargo test -p perl-workspace facts`
- `cargo test -p perl-workspace`
- `cargo check --workspace --all-targets`

**Suggested PR title:**
- `feat(perl-workspace): add write-through FileFactShard storage`

---

### Box 5 — `DefinitionCandidate` multimap

**Goal:** Add deterministic multi-candidate definition indexing under compatibility APIs.

**Likely files:**
- `crates/perl-workspace-index/src/workspace/workspace_index.rs`
- `crates/perl-workspace-index/src/workspace/...`
- `crates/perl-workspace-index/tests/...`
- `crates/perl-semantic-facts/src/...`

**In scope:**
- Add `DefinitionCandidate` maps for qualified and bare keys.
- Deterministic ranking.
- Preserve `find_definition(...) -> Option<Location>` compatibility semantics.
- Add test/internal API to inspect candidate vectors.
- Cover ambiguity and cleanup scenarios.

**Done when:**
- Ambiguous bare names yield deterministic candidate ordering.
- Reindex/remove leaves no stale candidates.
- Existing definition tests remain green.

**Verification:**
- `cargo test -p perl-workspace definition`
- `cargo test -p perl-workspace`
- `cargo check --workspace --all-targets`

**Suggested PR title:**
- `feat(perl-workspace): add deterministic definition candidate multimap`

---

### Box 6 — Typed `ReferenceEdge` global index

**Goal:** Preserve reference kind/confidence in workspace global indexes while keeping existing outputs compatible.

**Likely files:**
- `crates/perl-workspace-index/src/workspace/workspace_index.rs`
- `crates/perl-workspace-index/src/workspace/...`
- `crates/perl-workspace-index/tests/...`
- `crates/perl-semantic-facts/src/...`

**In scope:**
- Add/use canonical typed reference edges.
- Store typed refs globally by name/entity keys.
- Preserve `find_references` compatibility output.
- Use typed refs for `count_usages` when safe, or document/test interim behavior.
- Add tests across supported ref classes.

**Done when:**
- Typed ref kind is retained internally.
- Existing APIs remain passing.
- Definition exclusion and cleanup behavior covered.

**Verification:**
- `cargo test -p perl-workspace reference`
- `cargo test -p perl-workspace`
- `cargo check --workspace --all-targets`

**Suggested PR title:**
- `feat(perl-workspace): preserve typed reference edges in the global index`

---

### Box 7 — Shadow-compare receipts

**Goal:** Add deterministic old-vs-new query comparison receipts for migration visibility.

**Likely files:**
- `xtask/src/...`
- `crates/perl-workspace-index/src/...`
- receipt schema/registry files
- `docs/project/...` (if needed)

**In scope:**
- Deterministic JSON receipts for `find_definition`, `find_references`, `count_usages`.
- Include query input, old summary, new summary, and verdict (`same`, `improved`, `regression`, `ambiguous`, `unavailable`).
- Add serialization tests.
- Add small harness/fixture command where cheap.

**Done when:**
- Receipts are deterministic.
- Missing fact-backed path becomes `unavailable` (no panic).
- Stable JSON shape under test.

**Verification:**
- `cargo test -p xtask semantic`
- `cargo test -p perl-workspace`
- `cargo check --workspace --all-targets`

**Suggested PR title:**
- `feat(semantic): add shadow-compare receipts for workspace query migration`

---

### Box 8 — Semantic scorecard v1

**Goal:** Emit v1 semantic scorecard with fact counts and fixture-family coverage.

**Likely files:**
- `xtask/src/tasks/...`
- semantic fixture directory
- `docs/project/status/...`
- `crates/perl-workspace-index/tests/...`

**In scope:**
- Deterministic scorecard rows for declaration/occurrence/export facts, definition candidates, reference edges.
- Explicit unavailable rows for future import/package graph/rename categories.
- Counts by confidence/provenance class where available.
- Fixture-family coverage reporting.

**Done when:**
- `cargo xtask semantic-scorecard` output is deterministic.
- Output is useful before full adapter completion.
- JSON shape/loading covered by tests.

**Verification:**
- `cargo xtask semantic-scorecard`
- `cargo test -p xtask semantic`
- `cargo check --workspace --all-targets`

**Suggested PR title:**
- `feat(semantic): emit v1 semantic facts scorecard`

---

## Review Routing Guidance

Default first-pass: **Haiku** for all boxes, including larger PRs.

Escalate one targeted **Sonnet** review only when flagged by first-pass results, especially for semantic correctness and dynamic-boundary risk.

Suggested routing:

- Haiku first pass: Boxes 1, 2, 3, 8
- Haiku + conditional Sonnet escalation: Boxes 4, 5, 6, 7

Process note: treat receipts/reconciler output as source of truth; labels are projections.

---

## Wave 2 Exit Criteria

Wave 2 is complete when we can truthfully assert:

- Symbol declarations can become canonical entities.
- Symbol references can become canonical occurrences.
- Export analysis can become canonical export sets.
- Workspace can store per-file fact shards.
- Workspace can represent multiple definition candidates.
- Workspace can preserve typed references globally.
- Old and new query answers can be compared.
- Semantic scorecard can report meaningful coverage.

---

## Forward Pointer: Wave 3

Wave 3 begins the first user-visible semantic migration, in this order:

1. `ImportSpec`
2. `visible_symbols_at`
3. completion on `visible_symbols_at`
4. undefined-symbol diagnostics on `visible_symbols_at`

