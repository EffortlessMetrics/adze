# Semantic Wave 2 Epic Plan

## Epic intent

Wave 2 turns the Wave 1 semantic rails into real canonical fact production while keeping all existing provider behavior stable.

**Primary objective:** create the fact substrate and workspace indexing bridge so semantic truth has one canonical shape.

**Explicitly not in Wave 2:** completion migration, undefined-symbol diagnostics migration, rename/safe-delete migration, full package graph, Moose/Moo member generation support, external `@INC`/CPAN indexing, persisted semantic DB, deep type/value-shape inference.

## Target state after Wave 2

- `perl-symbol` emits canonical declaration and reference facts.
- exporter analysis emits canonical export facts.
- `perl-workspace` stores per-file fact shards.
- `perl-workspace` builds typed definition/reference indexes behind compatibility APIs.
- existing public query behavior stays intact.
- old-vs-new query answers can be compared via deterministic receipts.
- scorecard v1 reports fact coverage and fixture coverage.

## Review principle

Every slice should be the **most reviewable and tested complete slice**.

This means each PR should:
- keep behavior stable by default,
- include deterministic tests and fixtures,
- make unsupported coverage explicit,
- carry enough evidence to prove correctness (not just a minimal diff).

## Box breakdown (8 parallel tracks)

| Box | Scope | Output | Provider behavior change |
|---:|---|---|---|
| 1 | `SymbolDecl -> EntityFact` | declaration adapter (`AnchorFact`, `EntityFact`, `Defines`) | No |
| 2 | `SymbolRef -> OccurrenceFact` | reference adapter (`AnchorFact`, `OccurrenceFact`, typed ref edge) | No |
| 3 | `ExportInfo -> ExportSet` | export adapter (default/optional/tag exports + provenance) | No |
| 4 | `FileFactShard` write-through | workspace fact shard storage lifecycle | No |
| 5 | `DefinitionCandidate` multimap | deterministic candidate index behind compatibility wrapper | No (shadow/internal) |
| 6 | typed global `ReferenceEdge` index | kind/confidence-preserving refs behind compatibility output | No (shadow/internal) |
| 7 | shadow-compare receipts | deterministic old-vs-new query receipt JSONs | No |
| 8 | semantic scorecard v1 | fact counts + fixture-family coverage + unavailable rows | No |

## Merge sequencing

1. **Boxes 1–3** first (exact adapters).
2. **Box 8** next if it cleanly consumes adapter outputs.
3. **Box 4** write-through storage.
4. **Boxes 5–6** indexing upgrades (one PR at a time).
5. **Box 7** shadow-compare receipts.

If Box 4 lands before 5/6, rebase/index updates should cascade from Box 4.

## Definition of done per track

Each box must satisfy:

1. behavior compatibility maintained for existing APIs;
2. deterministic output for fact/index artifacts;
3. unsupported/dynamic boundaries represented explicitly (not dropped silently);
4. add/reindex/remove cleanup validated where stateful;
5. no provider cutover hidden in scope creep.

## Verification matrix

### Box 1
- `cargo test -p perl-symbol`
- `cargo test -p perl-semantic-facts`
- `cargo check --workspace --all-targets`

### Box 2
- `cargo test -p perl-symbol`
- `cargo test -p perl-semantic-facts`
- `cargo check --workspace --all-targets`

### Box 3
- `cargo test -p perl-semantic-analyzer export`
- `cargo test -p perl-semantic-facts`
- `cargo check --workspace --all-targets`

### Box 4
- `cargo test -p perl-workspace facts`
- `cargo test -p perl-workspace`
- `cargo check --workspace --all-targets`

### Box 5
- `cargo test -p perl-workspace definition`
- `cargo test -p perl-workspace`
- `cargo check --workspace --all-targets`

### Box 6
- `cargo test -p perl-workspace reference`
- `cargo test -p perl-workspace`
- `cargo check --workspace --all-targets`

### Box 7
- `cargo test -p xtask semantic`
- `cargo test -p perl-workspace`
- `cargo check --workspace --all-targets`

### Box 8
- `cargo xtask semantic-scorecard`
- `cargo test -p xtask semantic`
- `cargo check --workspace --all-targets`

## Review routing guidance

Default first pass: Haiku for all boxes.

Escalate to Sonnet only when flagged for unresolved semantic-correctness risk or high-risk dynamic-boundary behavior (typically Boxes 4–7).

## Success statement for Wave 2

Wave 2 is complete when we can claim:

- declarations become canonical entities,
- references become canonical occurrences,
- exporter analysis becomes canonical export sets,
- workspace stores and maintains file fact shards,
- workspace supports deterministic multiple definition candidates,
- workspace preserves typed references globally,
- old vs new query answers are receipt-comparable,
- scorecard v1 reports semantic fact coverage.

## Wave 3 handoff (next user-visible wave)

Wave 3 begins once this substrate is in place:

- `ImportSpec`
- `visible_symbols_at`
- completion backed by `visible_symbols_at`
- undefined diagnostics backed by `visible_symbols_at`
