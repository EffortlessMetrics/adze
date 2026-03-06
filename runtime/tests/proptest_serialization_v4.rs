#![cfg(feature = "serialization")]

//! Property-based tests for `SExpr` construction and `Display` formatting.
//!
//! 45+ proptest properties across 9 categories covering atom/list construction,
//! Display output, clone equality, accessor consistency, and edge cases.

use adze::serialization::SExpr;
use proptest::prelude::*;

// ---------------------------------------------------------------------------
// Strategies
// ---------------------------------------------------------------------------

fn arb_atom_str() -> impl Strategy<Value = String> {
    "[a-zA-Z0-9_]{1,40}"
}

fn arb_sexpr(max_depth: u32) -> impl Strategy<Value = SExpr> {
    let leaf = arb_atom_str().prop_map(|s| SExpr::atom(&s));
    leaf.prop_recursive(max_depth, 64, 8, |inner| {
        prop::collection::vec(inner, 0..8).prop_map(SExpr::list)
    })
}

// ===========================================================================
// 1. Atom construction preserves content (5 properties)
// ===========================================================================

proptest! {
    #[test]
    fn atom_preserves_alphanumeric(s in "[a-zA-Z0-9]{1,50}") {
        let expr = SExpr::atom(&s);
        prop_assert_eq!(expr.as_atom().unwrap(), s.as_str());
    }

    #[test]
    fn atom_preserves_underscores(s in "[a-z_]{1,30}") {
        let expr = SExpr::atom(&s);
        prop_assert_eq!(expr.as_atom().unwrap(), s.as_str());
    }

    #[test]
    fn atom_preserves_mixed_case(s in "[A-Za-z]{2,20}") {
        let expr = SExpr::atom(&s);
        prop_assert_eq!(expr.as_atom().unwrap(), s.as_str());
    }

    #[test]
    fn atom_preserves_digits(s in "[0-9]{1,15}") {
        let expr = SExpr::atom(&s);
        prop_assert_eq!(expr.as_atom().unwrap(), s.as_str());
    }

    #[test]
    fn atom_preserves_length(s in "\\PC{1,60}") {
        let expr = SExpr::atom(&s);
        prop_assert_eq!(expr.as_atom().unwrap().len(), s.len());
    }
}

// ===========================================================================
// 2. List construction preserves children count (5 properties)
// ===========================================================================

proptest! {
    #[test]
    fn list_preserves_count_small(n in 0..10usize) {
        let items: Vec<SExpr> = (0..n).map(|i| SExpr::atom(&format!("x{i}"))).collect();
        let expr = SExpr::list(items);
        prop_assert_eq!(expr.as_list().unwrap().len(), n);
    }

    #[test]
    fn list_preserves_count_medium(n in 10..50usize) {
        let items: Vec<SExpr> = (0..n).map(|i| SExpr::atom(&format!("y{i}"))).collect();
        let expr = SExpr::list(items);
        prop_assert_eq!(expr.as_list().unwrap().len(), n);
    }

    #[test]
    fn list_preserves_children_content(elems in prop::collection::vec(arb_atom_str(), 1..8)) {
        let items: Vec<SExpr> = elems.iter().map(|e| SExpr::atom(e)).collect();
        let expr = SExpr::list(items);
        let children = expr.as_list().unwrap();
        for (child, expected) in children.iter().zip(elems.iter()) {
            prop_assert_eq!(child.as_atom().unwrap(), expected.as_str());
        }
    }

    #[test]
    fn list_of_lists_preserves_outer_count(n in 1..8usize) {
        let items: Vec<SExpr> = (0..n).map(|_| SExpr::list(vec![])).collect();
        let expr = SExpr::list(items);
        prop_assert_eq!(expr.as_list().unwrap().len(), n);
    }

    #[test]
    fn list_preserves_mixed_children(atoms in 0..5usize, lists in 0..5usize) {
        let mut items = Vec::new();
        for i in 0..atoms {
            items.push(SExpr::atom(&format!("a{i}")));
        }
        for _ in 0..lists {
            items.push(SExpr::list(vec![]));
        }
        let expr = SExpr::list(items);
        prop_assert_eq!(expr.as_list().unwrap().len(), atoms + lists);
    }
}

// ===========================================================================
// 3. Display output is non-empty (5 properties)
// ===========================================================================

proptest! {
    #[test]
    fn display_atom_non_empty(s in "[a-z]{1,20}") {
        let output = format!("{}", SExpr::atom(&s));
        prop_assert!(!output.is_empty());
    }

    #[test]
    fn display_list_non_empty(n in 0..6usize) {
        let items: Vec<SExpr> = (0..n).map(|i| SExpr::atom(&format!("e{i}"))).collect();
        let output = format!("{}", SExpr::list(items));
        prop_assert!(!output.is_empty());
    }

    #[test]
    fn display_nested_non_empty(depth in 1..5u32) {
        let mut expr = SExpr::atom("leaf");
        for _ in 0..depth {
            expr = SExpr::list(vec![expr]);
        }
        let output = format!("{expr}");
        prop_assert!(!output.is_empty());
    }

    #[test]
    fn display_arbitrary_non_empty(expr in arb_sexpr(3)) {
        let output = format!("{expr}");
        prop_assert!(!output.is_empty());
    }

    #[test]
    fn display_single_element_list_non_empty(s in "[a-z]{1,10}") {
        let output = format!("{}", SExpr::list(vec![SExpr::atom(&s)]));
        prop_assert!(!output.is_empty());
    }
}

// ===========================================================================
// 4. Atom Display output contains the atom value (5 properties)
// ===========================================================================

proptest! {
    #[test]
    fn display_atom_contains_value(s in "[a-zA-Z0-9]{1,30}") {
        let output = format!("{}", SExpr::atom(&s));
        prop_assert!(output.contains(&s));
    }

    #[test]
    fn display_atom_equals_value(s in "[a-zA-Z_]{1,20}") {
        let output = format!("{}", SExpr::atom(&s));
        prop_assert_eq!(output, s);
    }

    #[test]
    fn display_atom_numeric_prefix(n in 0..10000u32) {
        let s = format!("n{n}");
        let output = format!("{}", SExpr::atom(&s));
        prop_assert!(output.contains(&n.to_string()));
    }

    #[test]
    fn display_atom_starts_with_content(s in "[a-z]{1,15}") {
        let output = format!("{}", SExpr::atom(&s));
        prop_assert!(output.starts_with(&s));
    }

    #[test]
    fn display_atom_length_matches(s in "[a-zA-Z0-9_]{1,25}") {
        let output = format!("{}", SExpr::atom(&s));
        prop_assert_eq!(output.len(), s.len());
    }
}

// ===========================================================================
// 5. List Display output contains parentheses (5 properties)
// ===========================================================================

proptest! {
    #[test]
    fn display_list_starts_with_open_paren(n in 0..5usize) {
        let items: Vec<SExpr> = (0..n).map(|i| SExpr::atom(&format!("v{i}"))).collect();
        let output = format!("{}", SExpr::list(items));
        prop_assert!(output.starts_with('('));
    }

    #[test]
    fn display_list_ends_with_close_paren(n in 0..5usize) {
        let items: Vec<SExpr> = (0..n).map(|i| SExpr::atom(&format!("w{i}"))).collect();
        let output = format!("{}", SExpr::list(items));
        prop_assert!(output.ends_with(')'));
    }

    #[test]
    fn display_list_has_balanced_parens(expr in arb_sexpr(3)) {
        let output = format!("{expr}");
        if expr.is_list() {
            let opens = output.chars().filter(|&c| c == '(').count();
            let closes = output.chars().filter(|&c| c == ')').count();
            prop_assert_eq!(opens, closes);
        }
    }

    #[test]
    fn display_empty_list_is_parens_only(_dummy in 0..1i32) {
        let output = format!("{}", SExpr::list(vec![]));
        prop_assert_eq!(output, "()");
    }

    #[test]
    fn display_list_contains_child_atoms(elems in prop::collection::vec("[a-z]{1,8}", 1..5)) {
        let items: Vec<SExpr> = elems.iter().map(|e| SExpr::atom(e)).collect();
        let output = format!("{}", SExpr::list(items));
        for elem in &elems {
            prop_assert!(output.contains(elem.as_str()));
        }
    }
}

// ===========================================================================
// 6. Nested list Display is well-formed (5 properties)
// ===========================================================================

proptest! {
    #[test]
    fn nested_display_balanced_parens(expr in arb_sexpr(4)) {
        let output = format!("{expr}");
        let opens = output.chars().filter(|&c| c == '(').count();
        let closes = output.chars().filter(|&c| c == ')').count();
        prop_assert_eq!(opens, closes);
    }

    #[test]
    fn nested_single_wrap_adds_parens(s in "[a-z]{1,10}") {
        let inner = SExpr::atom(&s);
        let outer = SExpr::list(vec![inner]);
        let output = format!("{outer}");
        prop_assert_eq!(output, format!("({s})"));
    }

    #[test]
    fn nested_double_wrap_format(s in "[a-z]{1,8}") {
        let inner = SExpr::atom(&s);
        let mid = SExpr::list(vec![inner]);
        let outer = SExpr::list(vec![mid]);
        let output = format!("{outer}");
        prop_assert_eq!(output, format!("(({s}))"));
    }

    #[test]
    fn nested_depth_paren_count(depth in 1..6u32, s in "[a-z]{1,5}") {
        let mut expr = SExpr::atom(&s);
        for _ in 0..depth {
            expr = SExpr::list(vec![expr]);
        }
        let output = format!("{expr}");
        let opens = output.chars().filter(|&c| c == '(').count();
        prop_assert_eq!(opens, depth as usize);
    }

    #[test]
    fn nested_display_contains_leaf(depth in 1..5u32, s in "[a-z]{2,8}") {
        let mut expr = SExpr::atom(&s);
        for _ in 0..depth {
            expr = SExpr::list(vec![expr]);
        }
        let output = format!("{expr}");
        prop_assert!(output.contains(&s));
    }
}

// ===========================================================================
// 7. Clone equality (5 properties)
// ===========================================================================

proptest! {
    #[test]
    fn clone_atom_equals_original(s in arb_atom_str()) {
        let expr = SExpr::atom(&s);
        let cloned = expr.clone();
        prop_assert_eq!(expr, cloned);
    }

    #[test]
    fn clone_list_equals_original(n in 0..8usize) {
        let items: Vec<SExpr> = (0..n).map(|i| SExpr::atom(&format!("c{i}"))).collect();
        let expr = SExpr::list(items);
        let cloned = expr.clone();
        prop_assert_eq!(expr, cloned);
    }

    #[test]
    fn clone_arbitrary_equals_original(expr in arb_sexpr(3)) {
        let cloned = expr.clone();
        prop_assert_eq!(expr, cloned);
    }

    #[test]
    fn clone_display_matches(expr in arb_sexpr(3)) {
        let cloned = expr.clone();
        prop_assert_eq!(format!("{expr}"), format!("{cloned}"));
    }

    #[test]
    fn clone_nested_equals_original(depth in 1..5u32) {
        let mut expr = SExpr::atom("z");
        for _ in 0..depth {
            expr = SExpr::list(vec![expr]);
        }
        let cloned = expr.clone();
        prop_assert_eq!(expr, cloned);
    }
}

// ===========================================================================
// 8. Accessor consistency: is_atom XOR is_list (5 properties)
// ===========================================================================

proptest! {
    #[test]
    fn accessor_atom_xor(s in arb_atom_str()) {
        let expr = SExpr::atom(&s);
        prop_assert!(expr.is_atom() ^ expr.is_list());
    }

    #[test]
    fn accessor_list_xor(n in 0..6usize) {
        let items: Vec<SExpr> = (0..n).map(|i| SExpr::atom(&format!("q{i}"))).collect();
        let expr = SExpr::list(items);
        prop_assert!(expr.is_atom() ^ expr.is_list());
    }

    #[test]
    fn accessor_arbitrary_xor(expr in arb_sexpr(3)) {
        prop_assert!(expr.is_atom() ^ expr.is_list());
    }

    #[test]
    fn accessor_atom_returns_some_list_returns_none(s in "[a-z]{1,10}") {
        let expr = SExpr::atom(&s);
        prop_assert!(expr.as_atom().is_some());
        prop_assert!(expr.as_list().is_none());
    }

    #[test]
    fn accessor_list_returns_some_atom_returns_none(n in 0..5usize) {
        let items: Vec<SExpr> = (0..n).map(|i| SExpr::atom(&format!("r{i}"))).collect();
        let expr = SExpr::list(items);
        prop_assert!(expr.as_list().is_some());
        prop_assert!(expr.as_atom().is_none());
    }
}

// ===========================================================================
// 9. Edge cases (6 properties)
// ===========================================================================

proptest! {
    #[test]
    fn edge_empty_string_atom(_dummy in 0..1i32) {
        let expr = SExpr::atom("");
        prop_assert!(expr.is_atom());
        prop_assert_eq!(expr.as_atom().unwrap(), "");
    }

    #[test]
    fn edge_empty_list(_dummy in 0..1i32) {
        let expr = SExpr::list(vec![]);
        prop_assert!(expr.is_list());
        prop_assert!(expr.as_list().unwrap().is_empty());
    }

    #[test]
    fn edge_whitespace_atom(ws in "[ \\t\\n]{1,10}") {
        let expr = SExpr::atom(&ws);
        prop_assert_eq!(expr.as_atom().unwrap(), ws.as_str());
    }

    #[test]
    fn edge_special_chars_atom(s in "[()\\[\\]{}]{1,10}") {
        let expr = SExpr::atom(&s);
        prop_assert_eq!(expr.as_atom().unwrap(), s.as_str());
    }

    #[test]
    fn edge_display_deterministic(expr in arb_sexpr(3)) {
        let out1 = format!("{expr}");
        let out2 = format!("{expr}");
        prop_assert_eq!(out1, out2);
    }

    #[test]
    fn edge_single_char_atom(c in prop::char::range('a', 'z')) {
        let s = c.to_string();
        let expr = SExpr::atom(&s);
        prop_assert_eq!(format!("{expr}"), s);
    }
}
