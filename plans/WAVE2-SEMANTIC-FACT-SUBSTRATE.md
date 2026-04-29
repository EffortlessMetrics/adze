# Wave 2 Epic Plan — Semantic Fact Substrate (No Provider Cutover)

**Last updated:** 2026-04-29  
**Status:** Planned / execution-ready  
**Scope type:** Internal semantic substrate hardening (non-user-visible)

## Epic intent

Wave 2 makes semantic facts **real and flowing** through the parser/export/workspace stack while deliberately keeping provider behavior stable.

### Target state after Wave 2

```text
perl-symbol / exporter / workspace
  → emit canonical facts

perl-workspace
  → stores fact shards
  → builds typed definition/reference indexes
  → keeps old public APIs working
  → can compare old vs new query answers
```

### Explicit non-goals (must stay out of Wave 2)

- completion migration
- undefined-symbol diagnostic migration
- rename/safe-delete migration
- full package graph
- Moose/Moo generated member support
- external `@INC` / CPAN index
- on-disk semantic persistence
- full type/value-shape inference

## Why this wave exists

The migration problem is not “rewrite providers first.” The problem is overlapping semantic truth stored in incompatible shapes across `perl-symbol`, `perl-workspace`, and `perl-semantic-analyzer`.

Wave 2 resolves that by establishing a canonical fact substrate (anchors, entities, occurrences, typed edges, provenance, confidence) and using `perl-workspace` as a derived index/query layer.

## Execution principle

Every task should be delivered as the **most reviewable and tested complete slice**.

- Prefer evidence density (tests, deterministic fixtures, stable output shape).
- Avoid oversized “clever” diffs that hide semantics.
- Preserve old public behavior unless explicitly running shadow comparisons.

## Box breakdown (8 parallel boxes)

| Box | Deliverable | Provider behavior change? |
| --- | --- | --- |
| 1 | `SymbolDecl -> EntityFact` adapter | No |
| 2 | `SymbolRef -> OccurrenceFact` adapter | No |
| 3 | `ExportInfo -> ExportSet` adapter | No |
| 4 | `FileFactShard` write-through store in `perl-workspace` | No |
| 5 | `DefinitionCandidate` multimap behind compatibility APIs | No / shadow-only optional |
| 6 | Typed `ReferenceEdge` global index behind compatibility APIs | No / shadow-only optional |
| 7 | Shadow-compare receipts for definition/reference query families | No |
| 8 | Semantic scorecard v1 (fact counts + fixture coverage) | No |

## Merge strategy

Do **not** merge all 8 in simple numeric order.

1. Boxes 1–3 (exact adapters)
2. Box 8 (if adapter outputs are consumed cleanly)
3. Box 4 (`FileFactShard` write-through)
4. Boxes 5–6 (index upgrades), one PR at a time
5. Box 7 (shadow-compare receipts)

If Box 4 is clean and both 5–6 are ready, merge Box 4 first, rebase 5/6 onto it, then land each index PR separately.

## Box charters

### Box 1 — `SymbolDecl -> EntityFact`

**Goal:** exact declaration adapter from `perl-symbol::surface::SymbolDecl` to canonical facts.

**In scope:**
- `facts_from_symbol_decl(...)` (or equivalent)
- declaration anchor → `AnchorFact`
- declaration identity → `EntityFact`
- declaration relation → `EdgeFact::Defines` (when supported)
- coverage for package/class/sub/method/variable/constant/label/format currently projected
- deterministic fixture/golden tests

**Done when:**
- no regression in existing `SymbolDecl` behavior
- deterministic fact adapter outputs
- explicit handling for unsupported declaration kinds (no silent drops)

**Verification:**
- `cargo test -p perl-symbol`
- `cargo test -p perl-semantic-facts`
- `cargo check --workspace --all-targets`

### Box 2 — `SymbolRef -> OccurrenceFact`

**Goal:** exact phase-1 reference adapter from `SymbolRef` into occurrence/reference facts.

**In scope:**
- `facts_from_symbol_ref(...)` (or equivalent)
- reference span → `AnchorFact`
- current reference projection → `OccurrenceFact`
- typed reference edge where model supports it
- golden coverage for currently supported variable/bare-call/qualified-call families

**Done when:**
- existing `SymbolRef` tests remain green
- deterministic outputs for current phase-1 references
- excluded phase-2 families are documented

**Verification:**
- `cargo test -p perl-symbol`
- `cargo test -p perl-semantic-facts`
- `cargo check --workspace --all-targets`

### Box 3 — `ExportInfo -> ExportSet`

**Goal:** adapt existing exporter analysis to canonical export facts.

**In scope:**
- `ExportInfo -> ExportSet` conversion
- default exports, optional exports, tag/group exports
- provenance retained as import/export inference class
- deterministic fixtures for `@EXPORT`, `@EXPORT_OK`, `%EXPORT_TAGS`

**Done when:**
- existing exporter tests still pass
- deterministic mapping for arrays/tags
- unsupported/dynamic exporter patterns represented conservatively

**Verification:**
- `cargo test -p perl-semantic-analyzer export`
- `cargo test -p perl-semantic-facts`
- `cargo check --workspace --all-targets`

### Box 4 — `FileFactShard` write-through store

**Goal:** store per-file fact shards in workspace index without changing public query outputs.

**In scope:**
- `FileFactShard` containing file id/uri, content hash, anchors/entities/occurrences/edges
- workspace state storage for shards
- write-through population from whichever adapters are present
- clean partial population if some adapters are absent
- lifecycle tests: add/reindex/remove/clear

**Done when:**
- deterministic store/remove/reindex behavior
- stale facts replaced on reindex
- no public API behavior change

**Verification:**
- `cargo test -p perl-workspace facts`
- `cargo test -p perl-workspace`
- `cargo check --workspace --all-targets`

### Box 5 — `DefinitionCandidate` multimap

**Goal:** deterministic multi-candidate definition index behind compatibility wrappers.

**In scope:**
- qualified key → `Vec<candidate>`
- bare key → `Vec<candidate>`
- deterministic ranking
- compatibility wrapper preserving `find_definition(...) -> Option<Location>`
- internal/test-only API to inspect all candidates
- duplicate-name/reindex/remove cleanup tests

**Done when:**
- ambiguous bare names produce stable candidates
- no stale candidates after file lifecycle changes
- existing definition tests remain green

**Verification:**
- `cargo test -p perl-workspace definition`
- `cargo test -p perl-workspace`
- `cargo check --workspace --all-targets`

### Box 6 — typed `ReferenceEdge` global index

**Goal:** preserve typed reference metadata globally while retaining compatibility API outputs.

**In scope:**
- typed reference storage (`ReferenceEdge` or canonical equivalent)
- indexing by symbol/entity/name keys
- compatibility wrapper for existing `find_references`
- `count_usages` no worse than current behavior (prefer typed path where safe)
- definition-vs-usage/call/import/export tests where current extraction supports them

**Done when:**
- kind/confidence preserved internally
- compatibility APIs unchanged
- remove/reindex cleanup verified

**Verification:**
- `cargo test -p perl-workspace reference`
- `cargo test -p perl-workspace`
- `cargo check --workspace --all-targets`

### Box 7 — shadow-compare receipts

**Goal:** deterministic receipts comparing old vs new workspace query answers.

**In scope:**
- JSON receipts for `find_definition`, `find_references`, `count_usages`
- receipt fields: query, input, old summary, new summary, verdict
- verdict taxonomy: `same | improved | regression | ambiguous | unavailable`
- deterministic serialization tests
- optional fixture-backed harness if cheap

**Done when:**
- deterministic receipt shape
- new-path gaps reported as `unavailable`, not panic
- stable JSON tests passing

**Verification:**
- `cargo test -p xtask semantic`
- `cargo test -p perl-workspace`
- `cargo check --workspace --all-targets`

### Box 8 — semantic scorecard v1

**Goal:** produce deterministic scorecard from Wave 1 fixture manifest plus available adapters.

**In scope:**
- rows for declaration facts, occurrence facts, export facts, definition candidates, reference edges
- explicit unavailable rows for future import/package-graph/rename families
- count breakdowns by confidence/provenance class when available
- fixture-family coverage counts

**Done when:**
- `cargo xtask semantic-scorecard` (or equivalent) emits deterministic output
- scorecard remains useful when some adapters are not landed
- JSON/output shape tests pass

**Verification:**
- `cargo xtask semantic-scorecard`
- `cargo test -p xtask semantic`
- `cargo check --workspace --all-targets`

## Review routing guidance

Default first pass: **Haiku** for all boxes. Escalate to Sonnet once per box only when semantic correctness remains unresolved.

- Haiku first pass: boxes 1, 2, 3, 8
- Haiku + optional Sonnet escalation: boxes 4, 5, 6, 7

Use receipts/reconciler outputs as source-of-truth for migration confidence; do not rely on labels as authority.

## Definition of Wave 2 done

Wave 2 is complete when the following are true on `main`:

- Symbol declarations emit canonical entities.
- Symbol references emit canonical occurrences.
- Export analysis emits canonical export sets.
- Workspace stores per-file fact shards.
- Workspace can represent multiple definition candidates.
- Workspace preserves typed references globally.
- Old/new query outputs can be shadow-compared deterministically.
- Semantic scorecard reports meaningful coverage.

This prepares Wave 3 user-visible migration (`ImportSpec`, `visible_symbols_at`, completion/diagnostics cutover behind flags).
