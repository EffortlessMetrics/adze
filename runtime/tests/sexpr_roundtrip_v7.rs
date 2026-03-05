//! Comprehensive tests for S-expression construction, accessors, Display, and equality.
#![cfg(feature = "serialization")]

use adze::serialization::SExpr;

// ============================================================
// 1. Atom construction (8 tests)
// ============================================================

#[test]
fn test_atom_convenience_simple() {
    let a = SExpr::atom("x");
    assert_eq!(a.as_atom(), Some("x"));
}

#[test]
fn test_atom_enum_variant() {
    let a = SExpr::Atom("y".into());
    assert_eq!(a.as_atom(), Some("y"));
}

#[test]
fn test_atom_empty_string() {
    let a = SExpr::atom("");
    assert_eq!(a.as_atom(), Some(""));
}

#[test]
fn test_atom_with_spaces() {
    let a = SExpr::atom("hello world");
    assert_eq!(a.as_atom(), Some("hello world"));
}

#[test]
fn test_atom_with_special_chars() {
    let a = SExpr::atom("foo-bar_baz");
    assert_eq!(a.as_atom(), Some("foo-bar_baz"));
}

#[test]
fn test_atom_unicode() {
    let a = SExpr::atom("λ");
    assert_eq!(a.as_atom(), Some("λ"));
}

#[test]
fn test_atom_numeric_string() {
    let a = SExpr::atom("42");
    assert_eq!(a.as_atom(), Some("42"));
}

#[test]
fn test_atom_long_string() {
    let long = "a".repeat(1000);
    let a = SExpr::atom(&long);
    assert_eq!(a.as_atom(), Some(long.as_str()));
}

// ============================================================
// 2. List construction (8 tests)
// ============================================================

#[test]
fn test_list_empty() {
    let l = SExpr::list(vec![]);
    assert_eq!(l.as_list(), Some([].as_slice()));
}

#[test]
fn test_list_single_atom() {
    let l = SExpr::list(vec![SExpr::atom("a")]);
    assert_eq!(l.as_list().unwrap().len(), 1);
}

#[test]
fn test_list_multiple_atoms() {
    let l = SExpr::list(vec![SExpr::atom("a"), SExpr::atom("b"), SExpr::atom("c")]);
    assert_eq!(l.as_list().unwrap().len(), 3);
}

#[test]
fn test_list_enum_variant() {
    let l = SExpr::List(vec![SExpr::Atom("x".into())]);
    assert_eq!(l.as_list().unwrap().len(), 1);
}

#[test]
fn test_list_nested() {
    let inner = SExpr::list(vec![SExpr::atom("inner")]);
    let outer = SExpr::list(vec![inner]);
    let items = outer.as_list().unwrap();
    assert_eq!(items.len(), 1);
    assert!(items[0].is_list());
}

#[test]
fn test_list_deeply_nested() {
    let mut expr = SExpr::atom("leaf");
    for _ in 0..10 {
        expr = SExpr::list(vec![expr]);
    }
    assert!(expr.is_list());
}

#[test]
fn test_list_mixed_atom_and_list() {
    let l = SExpr::list(vec![
        SExpr::atom("a"),
        SExpr::list(vec![SExpr::atom("b")]),
        SExpr::atom("c"),
    ]);
    let items = l.as_list().unwrap();
    assert!(items[0].is_atom());
    assert!(items[1].is_list());
    assert!(items[2].is_atom());
}

#[test]
fn test_list_two_nested_lists() {
    let l = SExpr::list(vec![
        SExpr::list(vec![SExpr::atom("a")]),
        SExpr::list(vec![SExpr::atom("b")]),
    ]);
    let items = l.as_list().unwrap();
    assert_eq!(items.len(), 2);
    assert!(items[0].is_list());
    assert!(items[1].is_list());
}

// ============================================================
// 3. Is predicates (8 tests)
// ============================================================

#[test]
fn test_is_atom_on_atom() {
    assert!(SExpr::atom("x").is_atom());
}

#[test]
fn test_is_atom_on_list() {
    assert!(!SExpr::list(vec![]).is_atom());
}

#[test]
fn test_is_list_on_list() {
    assert!(SExpr::list(vec![]).is_list());
}

#[test]
fn test_is_list_on_atom() {
    assert!(!SExpr::atom("x").is_list());
}

#[test]
fn test_is_atom_on_empty_string_atom() {
    assert!(SExpr::atom("").is_atom());
}

#[test]
fn test_is_list_on_nonempty_list() {
    assert!(SExpr::list(vec![SExpr::atom("a")]).is_list());
}

#[test]
fn test_is_atom_exclusive_of_is_list_for_atom() {
    let a = SExpr::atom("x");
    assert!(a.is_atom());
    assert!(!a.is_list());
}

#[test]
fn test_is_atom_exclusive_of_is_list_for_list() {
    let l = SExpr::list(vec![]);
    assert!(l.is_list());
    assert!(!l.is_atom());
}

// ============================================================
// 4. As accessors (8 tests)
// ============================================================

#[test]
fn test_as_atom_on_atom() {
    assert_eq!(SExpr::atom("hello").as_atom(), Some("hello"));
}

#[test]
fn test_as_atom_on_list_returns_none() {
    assert_eq!(SExpr::list(vec![]).as_atom(), None);
}

#[test]
fn test_as_list_on_list() {
    let l = SExpr::list(vec![SExpr::atom("a")]);
    let items = l.as_list().unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].as_atom(), Some("a"));
}

#[test]
fn test_as_list_on_atom_returns_none() {
    assert_eq!(SExpr::atom("x").as_list(), None);
}

#[test]
fn test_as_atom_empty_string() {
    assert_eq!(SExpr::atom("").as_atom(), Some(""));
}

#[test]
fn test_as_list_empty_list() {
    let l = SExpr::list(vec![]);
    assert!(l.as_list().unwrap().is_empty());
}

#[test]
fn test_as_list_nested_access() {
    let l = SExpr::list(vec![SExpr::list(vec![SExpr::atom("deep")])]);
    let inner = l.as_list().unwrap()[0].as_list().unwrap();
    assert_eq!(inner[0].as_atom(), Some("deep"));
}

#[test]
fn test_as_atom_preserves_content() {
    let content = "special chars: @#$%^&*()";
    let a = SExpr::atom(content);
    assert_eq!(a.as_atom(), Some(content));
}

// ============================================================
// 5. Display formatting (8 tests)
// ============================================================

#[test]
fn test_display_atom_simple() {
    assert_eq!(SExpr::atom("hello").to_string(), "hello");
}

#[test]
fn test_display_atom_numeric() {
    assert_eq!(SExpr::atom("42").to_string(), "42");
}

#[test]
fn test_display_empty_list() {
    assert_eq!(SExpr::list(vec![]).to_string(), "()");
}

#[test]
fn test_display_single_element_list() {
    let l = SExpr::list(vec![SExpr::atom("a")]);
    assert_eq!(l.to_string(), "(a)");
}

#[test]
fn test_display_multi_element_list() {
    let l = SExpr::list(vec![SExpr::atom("a"), SExpr::atom("b"), SExpr::atom("c")]);
    assert_eq!(l.to_string(), "(a b c)");
}

#[test]
fn test_display_nested_list() {
    let l = SExpr::list(vec![
        SExpr::atom("a"),
        SExpr::list(vec![SExpr::atom("b"), SExpr::atom("c")]),
    ]);
    assert_eq!(l.to_string(), "(a (b c))");
}

#[test]
fn test_display_deeply_nested() {
    let l = SExpr::list(vec![SExpr::list(vec![SExpr::list(vec![SExpr::atom("x")])])]);
    assert_eq!(l.to_string(), "(((x)))");
}

#[test]
fn test_display_mixed_nesting() {
    let l = SExpr::list(vec![
        SExpr::atom("define"),
        SExpr::list(vec![SExpr::atom("f"), SExpr::atom("x")]),
        SExpr::list(vec![SExpr::atom("+"), SExpr::atom("x"), SExpr::atom("1")]),
    ]);
    assert_eq!(l.to_string(), "(define (f x) (+ x 1))");
}

// ============================================================
// 6. Equality (8 tests)
// ============================================================

#[test]
fn test_eq_same_atoms() {
    assert_eq!(SExpr::atom("a"), SExpr::atom("a"));
}

#[test]
fn test_ne_different_atoms() {
    assert_ne!(SExpr::atom("a"), SExpr::atom("b"));
}

#[test]
fn test_eq_empty_lists() {
    assert_eq!(SExpr::list(vec![]), SExpr::list(vec![]));
}

#[test]
fn test_ne_atom_vs_list() {
    assert_ne!(SExpr::atom("a"), SExpr::list(vec![]));
}

#[test]
fn test_eq_same_single_element_lists() {
    assert_eq!(
        SExpr::list(vec![SExpr::atom("x")]),
        SExpr::list(vec![SExpr::atom("x")]),
    );
}

#[test]
fn test_ne_lists_different_length() {
    assert_ne!(
        SExpr::list(vec![SExpr::atom("a")]),
        SExpr::list(vec![SExpr::atom("a"), SExpr::atom("b")]),
    );
}

#[test]
fn test_eq_nested_lists() {
    let mk = || SExpr::list(vec![SExpr::list(vec![SExpr::atom("inner")])]);
    assert_eq!(mk(), mk());
}

#[test]
fn test_ne_nested_lists_different_content() {
    let a = SExpr::list(vec![SExpr::list(vec![SExpr::atom("x")])]);
    let b = SExpr::list(vec![SExpr::list(vec![SExpr::atom("y")])]);
    assert_ne!(a, b);
}

// ============================================================
// 7. Clone (6 tests)
// ============================================================

#[test]
fn test_clone_atom_equals_original() {
    let a = SExpr::atom("hello");
    let b = a.clone();
    assert_eq!(a, b);
}

#[test]
fn test_clone_list_equals_original() {
    let l = SExpr::list(vec![SExpr::atom("a"), SExpr::atom("b")]);
    let l2 = l.clone();
    assert_eq!(l, l2);
}

#[test]
fn test_clone_empty_list() {
    let l = SExpr::list(vec![]);
    let l2 = l.clone();
    assert_eq!(l, l2);
}

#[test]
fn test_clone_nested() {
    let l = SExpr::list(vec![SExpr::list(vec![SExpr::atom("deep")])]);
    let l2 = l.clone();
    assert_eq!(l, l2);
}

#[test]
fn test_clone_independence_atom() {
    let a = SExpr::atom("original");
    let _cloned = a.clone();
    // Rebind to a new value — original is unaffected
    let replaced = SExpr::atom("modified");
    assert_eq!(a.as_atom(), Some("original"));
    assert_eq!(replaced.as_atom(), Some("modified"));
}

#[test]
fn test_clone_independence_list() {
    let l = SExpr::list(vec![SExpr::atom("a")]);
    let _l2 = l.clone();
    // Original is unaffected by existence of clone
    assert_eq!(l.as_list().unwrap().len(), 1);
}

// ============================================================
// 8. Debug (6 tests)
// ============================================================

#[test]
fn test_debug_atom_contains_atom() {
    let a = SExpr::atom("hello");
    let dbg = format!("{a:?}");
    assert!(dbg.contains("Atom"));
    assert!(dbg.contains("hello"));
}

#[test]
fn test_debug_list_contains_list() {
    let l = SExpr::list(vec![]);
    let dbg = format!("{l:?}");
    assert!(dbg.contains("List"));
}

#[test]
fn test_debug_nested_contains_both() {
    let l = SExpr::list(vec![SExpr::atom("x")]);
    let dbg = format!("{l:?}");
    assert!(dbg.contains("List"));
    assert!(dbg.contains("Atom"));
    assert!(dbg.contains("x"));
}

#[test]
fn test_debug_atom_is_not_empty() {
    let dbg = format!("{:?}", SExpr::atom("z"));
    assert!(!dbg.is_empty());
}

#[test]
fn test_debug_list_is_not_empty() {
    let dbg = format!("{:?}", SExpr::list(vec![]));
    assert!(!dbg.is_empty());
}

#[test]
fn test_debug_differs_from_display() {
    let a = SExpr::atom("test");
    let debug_str = format!("{a:?}");
    let display_str = format!("{a}");
    // Debug includes variant name, Display does not
    assert_ne!(debug_str, display_str);
}
