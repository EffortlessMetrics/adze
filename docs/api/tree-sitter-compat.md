# Tree-sitter Compatibility API

**Status:** Advisory
**Spec:** ADZE-SPEC-0006
**Proof:** `cargo test -p adze --features "pure-rust,ts-compat" --test ts_compat_node_metadata -- --nocapture`

The Tree-sitter compatibility adapter (`ts_compat`) projects native document
data into a Tree-sitter-compatible API shape for ecosystem interop.

## Construction

```rust
let doc = grammar::parse_document("1 + 2")?;
let ts_tree = ts_compat::Tree::from_document(&doc);
```

## Node identity

Nodes expose two identity slots:

```rust
let node = ts_tree.root_node();
println!("visible kind: {}", node.kind());           // alias-adjusted
println!("grammar name: {}", node.grammar_name());   // raw rule name
```

## Child traversal

```rust
let root = ts_tree.root_node();
for i in 0..root.child_count() {
    let child = root.child(i).unwrap();
    println!("child {}: {}", i, child.kind());
}

// Named children exclude anonymous aliases
for i in 0..root.named_child_count() {
    let child = root.named_child(i).unwrap();
    println!("named child {}: {}", i, child.kind());
}
```

## S-expression output

```rust
let sexp = ts_tree.root_node().to_sexp();
println!("{}", sexp);
```

## Known gaps

- Full Tree-sitter API parity.
- Query predicate compatibility.
- node_types.json full parity.
- Error-tree parity beyond EOF.
- Imported grammar corpus compatibility.

This is an adoption adapter, not a full compatibility layer.
