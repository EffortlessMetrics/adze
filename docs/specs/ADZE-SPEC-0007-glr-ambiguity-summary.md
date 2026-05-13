# ADZE-SPEC-0007: GLR ambiguity summary

Status: proposed
Owner: Adze maintainers
Created: 2026-05-13
Linked proposal: ../proposals/ADZE-PROP-0002-api-foundation.md
Linked ADRs: ADZE-ADR-0001 AdzeDocument one parse truth
Linked plan:
Linked issues:
Linked PRs:
Support-tier impact: experimental
Policy impact: none

## Problem

GLR parsing can produce multiple valid parse trees for ambiguous input. Users
need to know when ambiguity occurred, which alternatives existed, which tree
was selected, and why — without being forced to understand raw forest internals.
Ambiguity summaries must be native document facts, not separate GLR engine
state.

## Behavior

### Selected tree is always available

When a document exists, a selected tree is always available. Even if the GLR
engine forked and produced multiple alternatives, one tree is chosen as the
selected tree. The document's `root()` returns this selected tree.

### Ambiguity summaries are native document facts

Ambiguity summaries are stored in `AmbiguitySet` inside `AdzeDocument`, not in
a separate GLR engine data structure.

```rust
pub struct AmbiguitySite {
    pub node_id: NodeId,
    pub alternatives: Vec<AmbiguityAlternative>,
    pub selected_index: usize,
    pub selection_reason: SelectionReason,
}

pub enum SelectionReason {
    Precedence { higher_prec: u16 },
    Associativity { side: AssocSide },
    FirstInList,
    UserDefault,
}
```

### Full forest is opt-in

Raw GLR forest export is not part of the document's default surface. It is
opt-in for advanced tooling. The default document surface exposes summaries
only.

### Typed AST uses selected tree by default

`doc.ast::<T>()` extracts from the selected tree. It does not iterate over
alternatives or require the user to choose.

### Ambiguity is absent for unambiguous input

When parsing produces a single parse tree (no GLR forking), the ambiguity set
is empty. There is no cost to unambiguous parses.

### Ambiguity is visible through the native API

The document exposes ambiguity sites through `doc.ambiguities()`, which returns
an iterator over `AmbiguitySite`. Each site identifies where ambiguity occurred
in the tree and what alternatives existed.

### Selection reason is explicit

Every ambiguity site records why a particular alternative was selected. The
selection reason vocabulary is small: precedence, associativity, first-in-list,
user default. This is not a full decision log — it is a summary-level
explanation.

## Non-Goals

- Full GLR forest export or traversal API.
- Per-alternative typed AST extraction.
- Custom selection strategies.
- GLR engine internals exposure.
- Full ambiguity resolution for every grammar conflict.

## Required Evidence

- Ambiguous input produces a document with non-empty ambiguity set.
- Unambiguous input produces a document with empty ambiguity set.
- Selected tree matches the tree returned by `doc.root()`.
- `doc.ast::<T>()` extracts from the selected tree.
- Selection reason is populated for every ambiguity site.
- Ambiguity summaries survive JSON serialization.

## Acceptance Examples

### Ambiguous parse reports summary

```rust
let doc = grammar::parse_document("1 + 2 + 3")?; // if grammar is ambiguous
if !doc.ambiguities().is_empty() {
    let site = &doc.ambiguities()[0];
    assert!(!site.alternatives.is_empty());
    assert!(site.selected_index < site.alternatives.len());
}
```

### Selected tree matches root

```rust
let doc = grammar::parse_document(source)?;
let root_kind = doc.root().kind();
let selected = &doc.ambiguities()[0].alternatives[doc.ambiguities()[0].selected_index];
assert_eq!(root_kind, selected.root_kind);
```

### Unambiguous parse has no ambiguity

```rust
let doc = grammar::parse_document("1 + 2")?;
assert!(doc.ambiguities().is_empty());
```

## Test Mapping

- `cargo test -p adze --features "pure-rust,glr,runtime-e2e" --test test_e2e_ambiguous_grammar_glr`
- `cargo test -p adze --features "pure-rust,glr" --test parser_v4_comprehensive`
- `cargo test -p adze-glr-core conflict ambiguity -- --nocapture`

## Implementation Mapping

| Component | Owner |
| --- | --- |
| `AmbiguitySet`, `AmbiguitySite` | `runtime/src/document.rs` |
| `SelectionReason` | `runtime/src/document.rs` |
| GLR ambiguity detection | `runtime/src/parser_v4.rs` |
| Conflict cell inspection | `glr-core/` |

## CI Proof

```bash
cargo test -p adze --features "pure-rust,glr,runtime-e2e" --test test_e2e_ambiguous_grammar_glr -- --nocapture
cargo test -p adze-glr-core conflict ambiguity -- --nocapture
just ci-supported
```

## Metrics And Promotion Rule

Promotion from experimental to stabilizing requires:

- All known GLR conflict types produce populated ambiguity summaries.
- Selection reasons are correct for precedence and associativity conflicts.
- JSON serialization round-trips ambiguity summaries without data loss.
- No performance regression for unambiguous parses.
