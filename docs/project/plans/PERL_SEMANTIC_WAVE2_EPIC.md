# Perl Semantic Substrate — Wave 2 Epic Plan

## Status

- **Wave:** 2
- **Date:** 2026-04-29
- **Scope type:** Internal semantic substrate migration (no provider cutover)
- **Primary intent:** Make canonical semantic facts real and queryable while preserving current public behavior.

## Why this wave exists

Wave 1 established the rails (vocabulary, fixtures, scorecard shape, query-contract docs, shadow-compare shape).
Wave 2 turns those rails into real data flow.

The core architectural objective is to stop carrying overlapping semantic truth across:

- `perl-symbol`
- `perl-workspace`
- `perl-semantic-analyzer`

…and instead converge on canonical semantic facts:

- anchors
- stable entities
- occurrences
- typed edges
- provenance
- confidence

`perl-workspace` should become a derived index/query layer over this substrate, not a second semantic source of truth.

## Non-goals for Wave 2

The following are explicitly **out of scope**:

- completion migration
- undefined-symbol diagnostics migration
- rename/safe-delete migration
- full package graph
- Moose/Moo generated-member support
- external `@INC` / CPAN index
- on-disk semantic persistence
- full type/value-shape inference

## Target state after Wave 2

```text
perl-symbol / exporter / workspace
  → emit canonical facts

perl-workspace
  → stores fact shards
  → builds typed definition/reference indexes
  → keeps old public APIs working
  → can compare old vs new query answers
```

## Delivery principle

Every implementation prompt and PR description should emphasize:

> **most reviewable and tested complete slice**

Bias toward evidence density and correctness proof, not smallest possible diff.

## Box matrix (8 parallel tracks)

| Box | Deliverable | Provider behavior change? |
|---:|---|---|
| 1 | `SymbolDecl -> EntityFact` adapter | No |
| 2 | `SymbolRef -> OccurrenceFact` adapter | No |
| 3 | `ExportInfo -> ExportSet` adapter | No |
| 4 | `FileFactShard` write-through store in `perl-workspace` | No |
| 5 | `DefinitionCandidate` multimap behind compatibility APIs | No / shadow-only acceptable |
| 6 | typed `ReferenceEdge` global index behind compatibility APIs | No / shadow-only acceptable |
| 7 | shadow-compare receipts for definition/reference queries | No |
| 8 | semantic scorecard v1: fact counts + fixture coverage | No |

## Merge order

Do not merge strictly in numeric order.

1. Boxes **1–3** (exact adapters)
2. Box **8** (scorecard v1), if it cleanly consumes adapter outputs
3. Box **4** (`FileFactShard` write-through)
4. Boxes **5–6** (candidate/ref indexes), one at a time
5. Box **7** (shadow compare receipts)

If Box 4 and both 5/6 are ready, merge **4 first**, then cascade updates to 5/6 and merge indexing PRs sequentially.

## Box-level implementation plans

### Box 1 — `SymbolDecl -> EntityFact`

**Goal**

Implement adapter from `perl-symbol::surface::SymbolDecl` into canonical facts.

**In scope**

- `facts_from_symbol_decl(...)` (or equivalent)
- emit `AnchorFact`
- emit `EntityFact`
- emit `EdgeFact::Defines` where supported
- cover currently projected kinds (package/class/subroutine/method/variable/constant/label/format)
- deterministic golden tests

**Done when**

- existing `SymbolDecl` behavior unchanged
- deterministic fact snapshots
- unsupported kinds explicit (never silent)

**Verification**

```bash
cargo test -p perl-symbol
cargo test -p perl-semantic-facts
cargo check --workspace --all-targets
```

**Suggested PR title**

`feat(perl-symbol): adapt SymbolDecl into canonical semantic facts`

---

### Box 2 — `SymbolRef -> OccurrenceFact`

**Goal**

Implement adapter from phase-1 `SymbolRef` into canonical occurrence/reference facts.

**In scope**

- `facts_from_symbol_ref(...)` (or equivalent)
- emit reference `AnchorFact`
- emit `OccurrenceFact`
- emit reference edge where model supports it
- preserve current phase-1 boundaries
- golden tests for variable/bare/qualified calls if already projected

**Done when**

- existing `SymbolRef` tests unchanged
- deterministic adapter output for current phase-1 references
- phase-2 exclusions documented

**Verification**

```bash
cargo test -p perl-symbol
cargo test -p perl-semantic-facts
cargo check --workspace --all-targets
```

**Suggested PR title**

`feat(perl-symbol): adapt SymbolRef into canonical occurrence facts`

---

### Box 3 — `ExportInfo -> ExportSet`

**Goal**

Adapt existing Exporter analysis output into canonical export facts.

**In scope**

- `ExportInfo -> ExportSet` conversion
- default exports
- optional exports
- tag/group facts
- provenance as `ImportExportInference` (or equivalent)
- fixtures for `@EXPORT`, `@EXPORT_OK`, `%EXPORT_TAGS`

**Done when**

- existing exporter tests remain green
- deterministic mapping for arrays/tags
- unsupported dynamic patterns modeled conservatively

**Verification**

```bash
cargo test -p perl-semantic-analyzer export
cargo test -p perl-semantic-facts
cargo check --workspace --all-targets
```

**Suggested PR title**

`feat(exports): adapt ExportSymbolExtractor output into semantic ExportSet facts`

---

### Box 4 — `FileFactShard` write-through store

**Goal**

Store canonical per-file fact shards in `perl-workspace` without changing public query behavior.

**In scope**

- `FileFactShard` with file identity/hash plus anchors/entities/occurrences/edges
- optional per-category hashes if cheap
- shard lifecycle in workspace state (add/reindex/remove/clear)
- partial population support when only subset of adapters are merged

**Done when**

- deterministic shard replacement on reindex
- deterministic shard cleanup on file remove
- legacy behavior unchanged

**Verification**

```bash
cargo test -p perl-workspace facts
cargo test -p perl-workspace
cargo check --workspace --all-targets
```

**Suggested PR title**

`feat(perl-workspace): add write-through FileFactShard storage`

---

### Box 5 — Definition candidate multimap

**Goal**

Add deterministic candidate multimap under existing definition API compatibility.

**In scope**

- qualified key -> `Vec<DefinitionCandidate>`
- bare key -> `Vec<DefinitionCandidate>`
- deterministic ranking
- compatibility wrapper keeps `find_definition(...) -> Option<Location>`
- internal/test API to inspect full candidate set
- tests for duplicate/bare/qualified and cleanup on reindex/remove

**Done when**

- ambiguous bare names yield deterministic candidate ordering
- no stale candidates after lifecycle operations
- existing definition tests continue to pass

**Verification**

```bash
cargo test -p perl-workspace definition
cargo test -p perl-workspace
cargo check --workspace --all-targets
```

**Suggested PR title**

`feat(perl-workspace): add deterministic definition candidate multimap`

---

### Box 6 — typed `ReferenceEdge` global index

**Goal**

Preserve reference kind/confidence globally while keeping old reference APIs.

**In scope**

- typed reference storage keyed by symbol/entity/name
- reuse canonical `ReferenceEdge` when available
- compatibility output for `find_references`
- enable `count_usages` to consume typed refs when safe
- tests for definition vs usage/call/import/export where extraction exists

**Done when**

- kind-preserving global references exist internally
- existing reference API behavior still passes
- no stale refs after remove/reindex

**Verification**

```bash
cargo test -p perl-workspace reference
cargo test -p perl-workspace
cargo check --workspace --all-targets
```

**Suggested PR title**

`feat(perl-workspace): preserve typed reference edges in the global index`

---

### Box 7 — semantic query shadow-compare receipts

**Goal**

Add deterministic receipt generation for old-vs-new query comparisons.

**In scope**

- deterministic JSON receipts for:
  - `find_definition`
  - `find_references`
  - `count_usages`
- include: query name, input, old summary, new summary, verdict
- verdict enum: `same | improved | regression | ambiguous | unavailable`
- deterministic serialization tests
- optional lightweight fixture-backed harness

**Done when**

- receipts are deterministic
- missing new path yields `unavailable` (not panic)
- JSON shape stability is test-covered

**Verification**

```bash
cargo test -p xtask semantic
cargo test -p perl-workspace
cargo check --workspace --all-targets
```

**Suggested PR title**

`feat(semantic): add shadow-compare receipts for workspace query migration`

---

### Box 8 — semantic scorecard v1

**Goal**

Emit useful fact-coverage scorecard output from Wave 1 fixture manifest + available adapters.

**In scope**

- deterministic rows for:
  - declaration facts
  - occurrence facts
  - export facts
  - definition candidates
  - reference edges
  - unavailable future rows (import/package graph/rename)
- counts by confidence/provenance bands if available
- fixture-family coverage totals
- integrate status update only if cheap and idiomatic in repo

**Done when**

- `cargo xtask semantic-scorecard` (or equivalent) is deterministic
- output remains useful before full adapter completion
- loading + JSON shape tests exist

**Verification**

```bash
cargo xtask semantic-scorecard
cargo test -p xtask semantic
cargo check --workspace --all-targets
```

**Suggested PR title**

`feat(semantic): emit v1 semantic facts scorecard`

## Review routing policy

Default first-pass review is Haiku, including large PRs.
Escalate to Sonnet for unresolved semantic-correctness risk and dynamic-boundary concerns.

- **Haiku first pass:** Boxes 1, 2, 3, 8
- **Haiku + Sonnet escalation if flagged:** Boxes 4, 5, 6, 7

Use receipts/reconciler as source of truth for migration status.
Do not use labels as authoritative state.

## Exit criteria for Wave 2

Wave 2 is complete when all statements below are true:

- Symbol declarations can emit canonical entities.
- Symbol references can emit canonical occurrences.
- Export analysis can emit canonical export sets.
- Workspace stores per-file fact shards.
- Workspace can represent multiple definition candidates.
- Workspace preserves typed references globally.
- Old/new query answers can be shadow-compared.
- Scorecard reports semantic fact coverage.

## Handoff to Wave 3

Wave 2 success enables first user-visible semantic cutovers in Wave 3:

- `ImportSpec`
- `visible_symbols_at`
- completion on `visible_symbols_at`
- undefined diagnostics on `visible_symbols_at`
