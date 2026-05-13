# GLR Ambiguity Summaries

**Status:** Experimental
**Spec:** ADZE-SPEC-0007
**Proof:** `cargo test -p adze --features "pure-rust,glr,runtime-e2e" --test test_e2e_ambiguous_grammar_glr -- --nocapture`

GLR parsing can produce multiple valid parse trees. Ambiguity summaries are
native document facts that explain where ambiguity occurred and which tree was
selected.

## Accessing ambiguity summaries

```rust
let doc = grammar::parse_document("1 - 2 - 3")?; // ambiguous for some grammars
for site in doc.ambiguities() {
    println!("ambiguous at node {:?}", site.node_id);
    println!("  {} alternatives", site.alternatives.len());
    println!("  selected: {}", site.selected_index);
    println!("  reason: {:?}", site.selection_reason);
}
```

## Selection reasons

| Reason | Meaning |
| --- | --- |
| `Precedence` | Higher-precedence alternative selected. |
| `Associativity` | Left/right-associative resolution. |
| `FirstInList` | First alternative in production order. |
| `UserDefault` | User-specified default selection. |

## Default behavior

- The selected tree is always available via `doc.root()`.
- `doc.ast::<T>()` extracts from the selected tree.
- Full GLR forest export is opt-in and not yet exposed.

## Not promised

- Full GLR forest traversal API.
- Per-alternative typed AST extraction.
- Custom selection strategies.
