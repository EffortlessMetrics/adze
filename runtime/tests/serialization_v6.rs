//! S-expression and tree serialization tests (v6).
//!
//! 64 tests across 8 categories exercising the `SExpr` data type:
//! atom creation, list creation, nesting, Display, equality, accessors, clone, edge cases.

#![cfg(feature = "serialization")]

use adze::serialization::SExpr;

// ---------------------------------------------------------------------------
// 1. sexpr_atom — atom creation and properties (8 tests)
// ---------------------------------------------------------------------------

#[test]
fn sexpr_atom_simple_word() {
    let a = SExpr::atom("hello");
    assert!(a.is_atom());
    assert!(!a.is_list());
}

#[test]
fn sexpr_atom_preserves_text() {
    let a = SExpr::atom("identifier");
    assert_eq!(a.as_atom(), Some("identifier"));
}

#[test]
fn sexpr_atom_with_spaces() {
    let a = SExpr::atom("hello world");
    assert_eq!(a.as_atom(), Some("hello world"));
}

#[test]
fn sexpr_atom_numeric_string() {
    let a = SExpr::atom("42");
    assert!(a.is_atom());
    assert_eq!(a.as_atom(), Some("42"));
}

#[test]
fn sexpr_atom_special_characters() {
    let a = SExpr::atom("+-*/");
    assert_eq!(a.as_atom(), Some("+-*/"));
}

#[test]
fn sexpr_atom_unicode() {
    let a = SExpr::atom("こんにちは");
    assert_eq!(a.as_atom(), Some("こんにちは"));
}

#[test]
fn sexpr_atom_empty_string() {
    let a = SExpr::atom("");
    assert!(a.is_atom());
    assert_eq!(a.as_atom(), Some(""));
}

#[test]
fn sexpr_atom_as_list_returns_none() {
    let a = SExpr::atom("x");
    assert_eq!(a.as_list(), None);
}

// ---------------------------------------------------------------------------
// 2. sexpr_list — list creation and properties (8 tests)
// ---------------------------------------------------------------------------

#[test]
fn sexpr_list_empty() {
    let l = SExpr::list(vec![]);
    assert!(l.is_list());
    assert!(!l.is_atom());
}

#[test]
fn sexpr_list_single_atom() {
    let l = SExpr::list(vec![SExpr::atom("a")]);
    let items = l.as_list().unwrap();
    assert_eq!(items.len(), 1);
}

#[test]
fn sexpr_list_multiple_atoms() {
    let l = SExpr::list(vec![SExpr::atom("a"), SExpr::atom("b"), SExpr::atom("c")]);
    let items = l.as_list().unwrap();
    assert_eq!(items.len(), 3);
}

#[test]
fn sexpr_list_preserves_order() {
    let l = SExpr::list(vec![SExpr::atom("x"), SExpr::atom("y"), SExpr::atom("z")]);
    let items = l.as_list().unwrap();
    assert_eq!(items[0].as_atom(), Some("x"));
    assert_eq!(items[1].as_atom(), Some("y"));
    assert_eq!(items[2].as_atom(), Some("z"));
}

#[test]
fn sexpr_list_as_atom_returns_none() {
    let l = SExpr::list(vec![]);
    assert_eq!(l.as_atom(), None);
}

#[test]
fn sexpr_list_is_not_atom() {
    let l = SExpr::list(vec![SExpr::atom("hello")]);
    assert!(!l.is_atom());
}

#[test]
fn sexpr_list_contains_lists() {
    let inner = SExpr::list(vec![SExpr::atom("inner")]);
    let outer = SExpr::list(vec![inner]);
    let items = outer.as_list().unwrap();
    assert!(items[0].is_list());
}

#[test]
fn sexpr_list_mixed_atoms_and_lists() {
    let l = SExpr::list(vec![
        SExpr::atom("name"),
        SExpr::list(vec![SExpr::atom("arg1"), SExpr::atom("arg2")]),
    ]);
    let items = l.as_list().unwrap();
    assert!(items[0].is_atom());
    assert!(items[1].is_list());
}

// ---------------------------------------------------------------------------
// 3. sexpr_nested — nested S-expression structures (8 tests)
// ---------------------------------------------------------------------------

#[test]
fn sexpr_nested_two_levels() {
    let inner = SExpr::list(vec![SExpr::atom("b")]);
    let outer = SExpr::list(vec![SExpr::atom("a"), inner]);
    let items = outer.as_list().unwrap();
    assert_eq!(items.len(), 2);
    assert!(items[1].is_list());
}

#[test]
fn sexpr_nested_three_levels() {
    let level3 = SExpr::atom("deep");
    let level2 = SExpr::list(vec![level3]);
    let level1 = SExpr::list(vec![level2]);
    let root = SExpr::list(vec![level1]);
    let l1 = root.as_list().unwrap();
    let l2 = l1[0].as_list().unwrap();
    let l3 = l2[0].as_list().unwrap();
    assert_eq!(l3[0].as_atom(), Some("deep"));
}

#[test]
fn sexpr_nested_sibling_lists() {
    let a = SExpr::list(vec![SExpr::atom("1")]);
    let b = SExpr::list(vec![SExpr::atom("2")]);
    let root = SExpr::list(vec![a, b]);
    let items = root.as_list().unwrap();
    assert_eq!(items.len(), 2);
    assert!(items[0].is_list());
    assert!(items[1].is_list());
}

#[test]
fn sexpr_nested_empty_inner_list() {
    let root = SExpr::list(vec![SExpr::list(vec![])]);
    let items = root.as_list().unwrap();
    let inner = items[0].as_list().unwrap();
    assert!(inner.is_empty());
}

#[test]
fn sexpr_nested_atom_at_leaf() {
    let tree = SExpr::list(vec![
        SExpr::list(vec![SExpr::atom("leaf1")]),
        SExpr::list(vec![SExpr::atom("leaf2")]),
    ]);
    let branches = tree.as_list().unwrap();
    assert_eq!(branches[0].as_list().unwrap()[0].as_atom(), Some("leaf1"));
    assert_eq!(branches[1].as_list().unwrap()[0].as_atom(), Some("leaf2"));
}

#[test]
fn sexpr_nested_heterogeneous_depth() {
    // (a (b (c)))  — different depths in siblings
    let c = SExpr::list(vec![SExpr::atom("c")]);
    let b = SExpr::list(vec![SExpr::atom("b"), c]);
    let root = SExpr::list(vec![SExpr::atom("a"), b]);
    assert!(root.as_list().unwrap()[0].is_atom());
    assert!(root.as_list().unwrap()[1].is_list());
}

#[test]
fn sexpr_nested_wide_tree() {
    let children: Vec<SExpr> = (0..10).map(|i| SExpr::atom(&i.to_string())).collect();
    let root = SExpr::list(children);
    assert_eq!(root.as_list().unwrap().len(), 10);
}

#[test]
fn sexpr_nested_list_of_empty_lists() {
    let root = SExpr::list(vec![
        SExpr::list(vec![]),
        SExpr::list(vec![]),
        SExpr::list(vec![]),
    ]);
    let items = root.as_list().unwrap();
    assert_eq!(items.len(), 3);
    for i in 0..items.len() {
        assert!(items[i].as_list().unwrap().is_empty());
    }
}

// ---------------------------------------------------------------------------
// 4. sexpr_display — Display/format output (8 tests)
// ---------------------------------------------------------------------------

#[test]
fn sexpr_display_atom() {
    let a = SExpr::atom("hello");
    assert_eq!(format!("{a}"), "hello");
}

#[test]
fn sexpr_display_empty_list() {
    let l = SExpr::list(vec![]);
    assert_eq!(format!("{l}"), "()");
}

#[test]
fn sexpr_display_single_item_list() {
    let l = SExpr::list(vec![SExpr::atom("x")]);
    assert_eq!(format!("{l}"), "(x)");
}

#[test]
fn sexpr_display_multi_item_list() {
    let l = SExpr::list(vec![SExpr::atom("a"), SExpr::atom("b"), SExpr::atom("c")]);
    assert_eq!(format!("{l}"), "(a b c)");
}

#[test]
fn sexpr_display_nested() {
    let inner = SExpr::list(vec![SExpr::atom("1"), SExpr::atom("2")]);
    let outer = SExpr::list(vec![SExpr::atom("fn"), inner]);
    assert_eq!(format!("{outer}"), "(fn (1 2))");
}

#[test]
fn sexpr_display_deeply_nested() {
    let d = SExpr::list(vec![SExpr::atom("d")]);
    let c = SExpr::list(vec![SExpr::atom("c"), d]);
    let b = SExpr::list(vec![SExpr::atom("b"), c]);
    let a = SExpr::list(vec![SExpr::atom("a"), b]);
    assert_eq!(format!("{a}"), "(a (b (c (d))))");
}

#[test]
fn sexpr_display_empty_atom() {
    let a = SExpr::atom("");
    assert_eq!(format!("{a}"), "");
}

#[test]
fn sexpr_display_to_string_matches_format() {
    let expr = SExpr::list(vec![SExpr::atom("+"), SExpr::atom("1"), SExpr::atom("2")]);
    assert_eq!(expr.to_string(), format!("{expr}"));
}

// ---------------------------------------------------------------------------
// 5. sexpr_equality — equality comparisons (8 tests)
// ---------------------------------------------------------------------------

#[test]
fn sexpr_equality_same_atoms() {
    assert_eq!(SExpr::atom("x"), SExpr::atom("x"));
}

#[test]
fn sexpr_equality_different_atoms() {
    assert_ne!(SExpr::atom("x"), SExpr::atom("y"));
}

#[test]
fn sexpr_equality_empty_lists() {
    assert_eq!(SExpr::list(vec![]), SExpr::list(vec![]));
}

#[test]
fn sexpr_equality_same_lists() {
    let a = SExpr::list(vec![SExpr::atom("1"), SExpr::atom("2")]);
    let b = SExpr::list(vec![SExpr::atom("1"), SExpr::atom("2")]);
    assert_eq!(a, b);
}

#[test]
fn sexpr_equality_different_list_contents() {
    let a = SExpr::list(vec![SExpr::atom("1")]);
    let b = SExpr::list(vec![SExpr::atom("2")]);
    assert_ne!(a, b);
}

#[test]
fn sexpr_equality_different_list_lengths() {
    let a = SExpr::list(vec![SExpr::atom("1")]);
    let b = SExpr::list(vec![SExpr::atom("1"), SExpr::atom("2")]);
    assert_ne!(a, b);
}

#[test]
fn sexpr_equality_atom_vs_list() {
    let a = SExpr::atom("x");
    let b = SExpr::list(vec![SExpr::atom("x")]);
    assert_ne!(a, b);
}

#[test]
fn sexpr_equality_nested_structures() {
    let make = || {
        SExpr::list(vec![
            SExpr::atom("root"),
            SExpr::list(vec![SExpr::atom("child")]),
        ])
    };
    assert_eq!(make(), make());
}

// ---------------------------------------------------------------------------
// 6. sexpr_access — accessor methods (8 tests)
// ---------------------------------------------------------------------------

#[test]
fn sexpr_access_atom_is_atom_true() {
    assert!(SExpr::atom("t").is_atom());
}

#[test]
fn sexpr_access_atom_is_list_false() {
    assert!(!SExpr::atom("t").is_list());
}

#[test]
fn sexpr_access_list_is_list_true() {
    assert!(SExpr::list(vec![]).is_list());
}

#[test]
fn sexpr_access_list_is_atom_false() {
    assert!(!SExpr::list(vec![]).is_atom());
}

#[test]
fn sexpr_access_as_atom_on_atom() {
    let a = SExpr::atom("val");
    assert_eq!(a.as_atom().unwrap(), "val");
}

#[test]
fn sexpr_access_as_atom_on_list() {
    let l = SExpr::list(vec![]);
    assert!(l.as_atom().is_none());
}

#[test]
fn sexpr_access_as_list_on_list() {
    let l = SExpr::list(vec![SExpr::atom("x")]);
    let items = l.as_list().unwrap();
    assert_eq!(items.len(), 1);
}

#[test]
fn sexpr_access_as_list_on_atom() {
    let a = SExpr::atom("v");
    assert!(a.as_list().is_none());
}

// ---------------------------------------------------------------------------
// 7. sexpr_clone — clone and copy operations (8 tests)
// ---------------------------------------------------------------------------

#[test]
fn sexpr_clone_atom() {
    let a = SExpr::atom("original");
    let b = a.clone();
    assert_eq!(a, b);
}

#[test]
fn sexpr_clone_list() {
    let l = SExpr::list(vec![SExpr::atom("a"), SExpr::atom("b")]);
    let l2 = l.clone();
    assert_eq!(l, l2);
}

#[test]
fn sexpr_clone_nested() {
    let expr = SExpr::list(vec![
        SExpr::atom("fn"),
        SExpr::list(vec![SExpr::atom("arg")]),
    ]);
    let cloned = expr.clone();
    assert_eq!(expr, cloned);
}

#[test]
fn sexpr_clone_independence_atom() {
    let a = SExpr::atom("original");
    let b = a.clone();
    // They are equal but are independent allocations
    assert_eq!(a.as_atom(), b.as_atom());
    assert_eq!(a, b);
}

#[test]
fn sexpr_clone_independence_list() {
    let l1 = SExpr::list(vec![SExpr::atom("x")]);
    let l2 = l1.clone();
    // Both have the same structure
    assert_eq!(l1.as_list().unwrap().len(), l2.as_list().unwrap().len());
}

#[test]
fn sexpr_clone_empty_list() {
    let l = SExpr::list(vec![]);
    let l2 = l.clone();
    assert_eq!(l, l2);
    assert!(l2.as_list().unwrap().is_empty());
}

#[test]
fn sexpr_clone_deep_tree() {
    let deep = SExpr::list(vec![SExpr::list(vec![SExpr::list(vec![SExpr::atom(
        "leaf",
    )])])]);
    let cloned = deep.clone();
    assert_eq!(format!("{deep}"), format!("{cloned}"));
}

#[test]
fn sexpr_clone_display_matches() {
    let expr = SExpr::list(vec![SExpr::atom("+"), SExpr::atom("a"), SExpr::atom("b")]);
    let cloned = expr.clone();
    assert_eq!(expr.to_string(), cloned.to_string());
}

// ---------------------------------------------------------------------------
// 8. sexpr_edge — edge cases (empty, deeply nested) (8 tests)
// ---------------------------------------------------------------------------

#[test]
fn sexpr_edge_empty_atom_is_atom() {
    let a = SExpr::atom("");
    assert!(a.is_atom());
    assert_eq!(a.as_atom(), Some(""));
}

#[test]
fn sexpr_edge_empty_list_len() {
    let l = SExpr::list(vec![]);
    assert!(l.as_list().unwrap().is_empty());
}

#[test]
fn sexpr_edge_deeply_nested_10_levels() {
    let mut expr = SExpr::atom("core");
    for _ in 0..10 {
        expr = SExpr::list(vec![expr]);
    }
    // Unwrap 10 levels
    let mut current = &expr;
    for _ in 0..10 {
        current = &current.as_list().unwrap()[0];
    }
    assert_eq!(current.as_atom(), Some("core"));
}

#[test]
fn sexpr_edge_deeply_nested_display() {
    let mut expr = SExpr::atom("x");
    for _ in 0..3 {
        expr = SExpr::list(vec![expr]);
    }
    assert_eq!(format!("{expr}"), "(((x)))");
}

#[test]
fn sexpr_edge_atom_with_parentheses_in_text() {
    // Atoms can contain arbitrary text including parens
    let a = SExpr::atom("(hello)");
    assert_eq!(a.as_atom(), Some("(hello)"));
}

#[test]
fn sexpr_edge_atom_with_newlines() {
    let a = SExpr::atom("line1\nline2");
    assert_eq!(a.as_atom(), Some("line1\nline2"));
}

#[test]
fn sexpr_edge_large_flat_list() {
    let items: Vec<SExpr> = (0..100).map(|i| SExpr::atom(&i.to_string())).collect();
    let l = SExpr::list(items);
    let list_items = l.as_list().unwrap();
    assert_eq!(list_items.len(), 100);
    assert_eq!(list_items[0].as_atom(), Some("0"));
    assert_eq!(list_items[99].as_atom(), Some("99"));
}

#[test]
fn sexpr_edge_list_with_only_empty_atoms() {
    let l = SExpr::list(vec![SExpr::atom(""), SExpr::atom(""), SExpr::atom("")]);
    let items = l.as_list().unwrap();
    assert_eq!(items.len(), 3);
    for i in 0..items.len() {
        assert_eq!(items[i].as_atom(), Some(""));
    }
    // Display: each empty atom contributes nothing visible between spaces
    assert_eq!(format!("{l}"), "(  )");
}
