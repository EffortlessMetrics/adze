//! Property-based tests for runtime serialization types.
#![cfg(feature = "serialization")]

use adze::serialization::{SExpr, SerializedNode};
use proptest::prelude::*;

fn leaf_node(kind: String, text: String) -> SerializedNode {
    SerializedNode {
        kind,
        is_named: true,
        field_name: None,
        start_position: (0, 0),
        end_position: (0, text.len()),
        start_byte: 0,
        end_byte: text.len(),
        text: Some(text),
        children: vec![],
        is_error: false,
        is_missing: false,
    }
}

proptest! {
    #[test]
    fn node_kind_preserved(kind in "[a-z_]{1,20}") {
        let node = leaf_node(kind.clone(), "x".to_string());
        prop_assert_eq!(&node.kind, &kind);
    }

    #[test]
    fn node_text_preserved(text in "[ -~]{0,100}") {
        let node = leaf_node("t".to_string(), text.clone());
        prop_assert_eq!(node.text.as_deref(), Some(text.as_str()));
    }

    #[test]
    fn node_byte_range_valid(len in 0..1000usize) {
        let text = "x".repeat(len);
        let node = leaf_node("t".to_string(), text);
        prop_assert!(node.start_byte <= node.end_byte);
    }

    #[test]
    fn node_children_count(n in 0..20usize) {
        let mut node = leaf_node("parent".to_string(), String::new());
        for i in 0..n {
            node.children.push(leaf_node(format!("child_{}", i), "c".to_string()));
        }
        prop_assert_eq!(node.children.len(), n);
    }

    #[test]
    fn node_field_name_option(has_field in proptest::bool::ANY, name in "[a-z]{1,10}") {
        let mut node = leaf_node("t".to_string(), "x".to_string());
        if has_field {
            node.field_name = Some(name.clone());
            prop_assert_eq!(node.field_name.as_deref(), Some(name.as_str()));
        } else {
            prop_assert!(node.field_name.is_none());
        }
    }

    #[test]
    fn node_error_flag(is_error in proptest::bool::ANY) {
        let mut node = leaf_node("t".to_string(), "x".to_string());
        node.is_error = is_error;
        prop_assert_eq!(node.is_error, is_error);
    }

    #[test]
    fn node_missing_flag(is_missing in proptest::bool::ANY) {
        let mut node = leaf_node("t".to_string(), "x".to_string());
        node.is_missing = is_missing;
        prop_assert_eq!(node.is_missing, is_missing);
    }

    #[test]
    fn node_named_flag(is_named in proptest::bool::ANY) {
        let mut node = leaf_node("t".to_string(), "x".to_string());
        node.is_named = is_named;
        prop_assert_eq!(node.is_named, is_named);
    }

    #[test]
    fn node_position_consistency(
        sr in 0..1000usize, sc in 0..1000usize,
        er in 0..1000usize, ec in 0..1000usize
    ) {
        let node = SerializedNode {
            kind: "t".to_string(),
            is_named: true,
            field_name: None,
            start_position: (sr, sc),
            end_position: (er, ec),
            start_byte: 0,
            end_byte: 0,
            text: None,
            children: vec![],
            is_error: false,
            is_missing: false,
        };
        prop_assert_eq!(node.start_position, (sr, sc));
        prop_assert_eq!(node.end_position, (er, ec));
    }
}

// ── SExpr property tests ──

proptest! {
    #[test]
    fn sexpr_atom_preserved(value in "[a-zA-Z0-9_]{1,50}") {
        let expr = SExpr::Atom(value.clone());
        if let SExpr::Atom(ref v) = expr {
            prop_assert_eq!(v, &value);
        } else {
            prop_assert!(false, "Expected Atom");
        }
    }

    #[test]
    fn sexpr_list_length(n in 0..20usize) {
        let items: Vec<SExpr> = (0..n).map(|i| SExpr::Atom(format!("item_{}", i))).collect();
        let list = SExpr::List(items);
        if let SExpr::List(ref v) = list {
            prop_assert_eq!(v.len(), n);
        }
    }
}

// ── JSON roundtrip tests ──

proptest! {
    #[test]
    fn node_json_roundtrip(kind in "[a-z]{1,10}", text in "[a-z]{0,20}") {
        let node = leaf_node(kind, text);
        let json = serde_json::to_string(&node).unwrap();
        let back: SerializedNode = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(&node.kind, &back.kind);
        prop_assert_eq!(&node.text, &back.text);
        prop_assert_eq!(node.is_named, back.is_named);
    }

    #[test]
    fn node_with_children_json_roundtrip(n in 1..5usize) {
        let mut node = leaf_node("parent".to_string(), String::new());
        for i in 0..n {
            node.children.push(leaf_node(format!("c{}", i), format!("v{}", i)));
        }
        let json = serde_json::to_string(&node).unwrap();
        let back: SerializedNode = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(node.children.len(), back.children.len());
    }
}

// ── Regular tests ──

#[test]
fn node_clone() {
    let node = leaf_node("test".to_string(), "hello".to_string());
    let cloned = node.clone();
    assert_eq!(node.kind, cloned.kind);
    assert_eq!(node.text, cloned.text);
}

#[test]
fn node_debug() {
    let node = leaf_node("test".to_string(), "hello".to_string());
    let dbg = format!("{:?}", node);
    assert!(dbg.contains("test"));
}

#[test]
fn sexpr_atom_debug() {
    let expr = SExpr::Atom("hello".to_string());
    let dbg = format!("{:?}", expr);
    assert!(dbg.contains("hello"));
}

#[test]
fn sexpr_list_debug() {
    let expr = SExpr::List(vec![
        SExpr::Atom("a".to_string()),
        SExpr::Atom("b".to_string()),
    ]);
    let dbg = format!("{:?}", expr);
    assert!(!dbg.is_empty());
}

#[test]
fn node_nested_children_json() {
    let child = leaf_node("child".to_string(), "c".to_string());
    let mut parent = leaf_node("parent".to_string(), String::new());
    parent.children.push(child);
    let json = serde_json::to_string(&parent).unwrap();
    let back: SerializedNode = serde_json::from_str(&json).unwrap();
    assert_eq!(back.children.len(), 1);
    assert_eq!(back.children[0].kind, "child");
}
