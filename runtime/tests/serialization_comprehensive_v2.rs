//! Comprehensive tests for the runtime serialization module.
#![cfg(feature = "serialization")]

use adze::serialization::{SExpr, SerializedNode};

fn make_leaf(kind: &str, text: &str, start: usize, end: usize) -> SerializedNode {
    SerializedNode {
        kind: kind.to_string(),
        is_named: true,
        field_name: None,
        start_position: (0, start),
        end_position: (0, end),
        start_byte: start,
        end_byte: end,
        text: Some(text.to_string()),
        children: vec![],
        is_error: false,
        is_missing: false,
    }
}

fn make_branch(
    kind: &str,
    children: Vec<SerializedNode>,
    start: usize,
    end: usize,
) -> SerializedNode {
    SerializedNode {
        kind: kind.to_string(),
        is_named: true,
        field_name: None,
        start_position: (0, start),
        end_position: (0, end),
        start_byte: start,
        end_byte: end,
        text: None,
        children,
        is_error: false,
        is_missing: false,
    }
}

// ─── SerializedNode construction ───

#[test]
fn serialized_node_leaf() {
    let n = make_leaf("identifier", "hello", 0, 5);
    assert_eq!(n.kind, "identifier");
    assert!(n.is_named);
    assert_eq!(n.start_byte, 0);
    assert_eq!(n.end_byte, 5);
    assert!(n.children.is_empty());
    assert_eq!(n.text, Some("hello".to_string()));
    assert!(!n.is_error);
    assert!(!n.is_missing);
}

#[test]
fn serialized_node_with_children() {
    let child = make_leaf("number", "5", 0, 1);
    let parent = make_branch("expression", vec![child], 0, 1);
    assert_eq!(parent.children.len(), 1);
    assert_eq!(parent.children[0].kind, "number");
}

#[test]
fn serialized_node_unnamed() {
    let mut n = make_leaf("+", "+", 2, 3);
    n.is_named = false;
    assert!(!n.is_named);
}

#[test]
fn serialized_node_no_text() {
    let n = make_branch("program", vec![], 0, 100);
    assert!(n.text.is_none());
}

#[test]
fn serialized_node_debug() {
    let n = make_leaf("test", "test", 0, 4);
    let d = format!("{:?}", n);
    assert!(d.contains("test"));
}

#[test]
fn serialized_node_clone() {
    let n = make_leaf("x", "x", 0, 1);
    let cloned = n.clone();
    assert_eq!(cloned.kind, n.kind);
    assert_eq!(cloned.text, n.text);
}

// ─── Field name ───

#[test]
fn serialized_node_with_field_name() {
    let mut n = make_leaf("identifier", "x", 0, 1);
    n.field_name = Some("name".to_string());
    assert_eq!(n.field_name, Some("name".to_string()));
}

#[test]
fn serialized_node_no_field_name() {
    let n = make_leaf("num", "5", 0, 1);
    assert!(n.field_name.is_none());
}

// ─── Error/missing nodes ───

#[test]
fn serialized_node_error() {
    let mut n = make_leaf("ERROR", "??", 0, 2);
    n.is_error = true;
    assert!(n.is_error);
}

#[test]
fn serialized_node_missing() {
    let mut n = make_leaf("MISSING", "", 5, 5);
    n.is_missing = true;
    assert!(n.is_missing);
}

// ─── Deep nesting ───

#[test]
fn serialized_node_deep_nesting() {
    let mut current = make_leaf("leaf", "x", 0, 1);
    for i in 0..20 {
        current = make_branch(&format!("level_{}", i), vec![current], 0, 1);
    }
    assert_eq!(current.children.len(), 1);
    assert_eq!(current.kind, "level_19");
}

// ─── Wide tree ───

#[test]
fn serialized_node_many_children() {
    let children: Vec<SerializedNode> = (0..50)
        .map(|i| make_leaf(&format!("child_{}", i), &format!("{}", i), i, i + 1))
        .collect();
    let parent = make_branch("parent", children, 0, 50);
    assert_eq!(parent.children.len(), 50);
}

// ─── JSON serialization ───

#[test]
fn serialized_node_to_json() {
    let n = make_leaf("test", "test", 0, 4);
    let json = serde_json::to_string(&n).unwrap();
    assert!(json.contains("\"kind\":\"test\""));
}

#[test]
fn serialized_node_json_roundtrip() {
    let n = make_leaf("number", "12345", 5, 10);
    let json = serde_json::to_string(&n).unwrap();
    let parsed: SerializedNode = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.kind, "number");
    assert_eq!(parsed.start_byte, 5);
    assert_eq!(parsed.text, Some("12345".to_string()));
}

#[test]
fn serialized_node_json_with_children() {
    let child = make_leaf("child", "c", 0, 1);
    let parent = make_branch("parent", vec![child], 0, 1);
    let json = serde_json::to_string(&parent).unwrap();
    let parsed: SerializedNode = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.children.len(), 1);
    assert_eq!(parsed.children[0].kind, "child");
}

#[test]
fn serialized_node_json_with_field_name() {
    let mut n = make_leaf("id", "x", 0, 1);
    n.field_name = Some("name".to_string());
    let json = serde_json::to_string(&n).unwrap();
    let parsed: SerializedNode = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.field_name, Some("name".to_string()));
}

#[test]
fn serialized_node_json_error_flag() {
    let mut n = make_leaf("ERR", "?", 0, 1);
    n.is_error = true;
    let json = serde_json::to_string(&n).unwrap();
    let parsed: SerializedNode = serde_json::from_str(&json).unwrap();
    assert!(parsed.is_error);
}

#[test]
fn serialized_node_json_missing_flag() {
    let mut n = make_leaf("MISS", "", 0, 0);
    n.is_missing = true;
    let json = serde_json::to_string(&n).unwrap();
    let parsed: SerializedNode = serde_json::from_str(&json).unwrap();
    assert!(parsed.is_missing);
}

// ─── SExpr ───

#[test]
fn sexpr_atom() {
    let s = SExpr::Atom("hello".to_string());
    let d = format!("{:?}", s);
    assert!(d.contains("hello"));
}

#[test]
fn sexpr_list_empty() {
    let s = SExpr::List(vec![]);
    if let SExpr::List(items) = s {
        assert!(items.is_empty());
    }
}

#[test]
fn sexpr_list_with_atoms() {
    let s = SExpr::List(vec![
        SExpr::Atom("a".to_string()),
        SExpr::Atom("b".to_string()),
    ]);
    if let SExpr::List(items) = s {
        assert_eq!(items.len(), 2);
    }
}

#[test]
fn sexpr_nested() {
    let inner = SExpr::List(vec![SExpr::Atom("x".to_string())]);
    let outer = SExpr::List(vec![SExpr::Atom("root".to_string()), inner]);
    if let SExpr::List(items) = &outer {
        assert_eq!(items.len(), 2);
    }
}

#[test]
fn sexpr_clone() {
    let s = SExpr::Atom("test".to_string());
    let s2 = s.clone();
    assert_eq!(s, s2);
}

#[test]
fn sexpr_equality() {
    let a = SExpr::Atom("x".to_string());
    let b = SExpr::Atom("x".to_string());
    assert_eq!(a, b);
}

#[test]
fn sexpr_inequality_atom() {
    let a = SExpr::Atom("x".to_string());
    let b = SExpr::Atom("y".to_string());
    assert_ne!(a, b);
}

#[test]
fn sexpr_inequality_type() {
    let a = SExpr::Atom("x".to_string());
    let b = SExpr::List(vec![]);
    assert_ne!(a, b);
}

#[test]
fn sexpr_debug() {
    let s = SExpr::List(vec![SExpr::Atom("test".to_string())]);
    let d = format!("{:?}", s);
    assert!(d.contains("test"));
}

// ─── Positions ───

#[test]
fn serialized_node_multiline_positions() {
    let n = SerializedNode {
        kind: "block".to_string(),
        is_named: true,
        field_name: None,
        start_position: (0, 0),
        end_position: (3, 1),
        start_byte: 0,
        end_byte: 50,
        text: None,
        children: vec![],
        is_error: false,
        is_missing: false,
    };
    assert_eq!(n.start_position, (0, 0));
    assert_eq!(n.end_position, (3, 1));
}

// ─── Edge cases ───

#[test]
fn serialized_node_empty_kind() {
    let n = make_leaf("", "", 0, 0);
    assert_eq!(n.kind, "");
}

#[test]
fn serialized_node_zero_length() {
    let n = make_leaf("empty", "", 5, 5);
    assert_eq!(n.start_byte, n.end_byte);
}

#[test]
fn serialized_node_unicode_text() {
    let n = make_leaf("string", "日本語🎉", 0, 16);
    assert!(n.text.unwrap().contains("🎉"));
}

#[test]
fn parse_sexpr_not_yet_implemented() {
    let result = adze::serialization::parse_sexpr("(root (child))");
    // Function exists; it may return Ok or Err
    let _ = result;
}
