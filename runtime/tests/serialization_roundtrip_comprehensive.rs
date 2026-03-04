#![cfg(feature = "serialization")]
//! Comprehensive serialization roundtrip tests for parse tree types.
//!
//! Covers: SerializedNode construction/field access, nested trees, JSON roundtrip,
//! SExpr, CompactSerializer/CompactNode, TreeSerializer, SExpressionSerializer,
//! edge cases, large trees, deep nesting, error/missing nodes, field names.

use adze::serialization::*;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn leaf_node(kind: &str, text: &str, start: usize, end: usize) -> SerializedNode {
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

fn branch_node(
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

fn assert_node_eq(a: &SerializedNode, b: &SerializedNode) {
    assert_eq!(a.kind, b.kind);
    assert_eq!(a.is_named, b.is_named);
    assert_eq!(a.field_name, b.field_name);
    assert_eq!(a.start_position, b.start_position);
    assert_eq!(a.end_position, b.end_position);
    assert_eq!(a.start_byte, b.start_byte);
    assert_eq!(a.end_byte, b.end_byte);
    assert_eq!(a.text, b.text);
    assert_eq!(a.is_error, b.is_error);
    assert_eq!(a.is_missing, b.is_missing);
    assert_eq!(a.children.len(), b.children.len());
    for (ca, cb) in a.children.iter().zip(b.children.iter()) {
        assert_node_eq(ca, cb);
    }
}

// ===================================================================
// 1. SerializedNode – basic construction and field access
// ===================================================================

#[test]
fn test_leaf_node_construction() {
    let n = leaf_node("number", "42", 0, 2);
    assert_eq!(n.kind, "number");
    assert!(n.is_named);
    assert_eq!(n.field_name, None);
    assert_eq!(n.text, Some("42".to_string()));
    assert!(n.children.is_empty());
    assert!(!n.is_error);
    assert!(!n.is_missing);
    assert_eq!(n.start_byte, 0);
    assert_eq!(n.end_byte, 2);
}

#[test]
fn test_leaf_node_positions() {
    let n = leaf_node("id", "x", 5, 6);
    assert_eq!(n.start_position, (0, 5));
    assert_eq!(n.end_position, (0, 6));
    assert_eq!(n.start_byte, 5);
    assert_eq!(n.end_byte, 6);
}

#[test]
fn test_branch_node_construction() {
    let n = branch_node("expression", vec![leaf_node("id", "a", 0, 1)], 0, 1);
    assert_eq!(n.kind, "expression");
    assert!(n.text.is_none());
    assert_eq!(n.children.len(), 1);
}

#[test]
fn test_unnamed_node() {
    let n = SerializedNode {
        kind: "+".to_string(),
        is_named: false,
        field_name: None,
        start_position: (0, 2),
        end_position: (0, 3),
        start_byte: 2,
        end_byte: 3,
        text: Some("+".to_string()),
        children: vec![],
        is_error: false,
        is_missing: false,
    };
    assert!(!n.is_named);
    assert_eq!(n.kind, "+");
}

#[test]
fn test_field_name_some() {
    let mut n = leaf_node("identifier", "x", 0, 1);
    n.field_name = Some("name".to_string());
    assert_eq!(n.field_name, Some("name".to_string()));
}

#[test]
fn test_field_name_none() {
    let n = leaf_node("identifier", "x", 0, 1);
    assert_eq!(n.field_name, None);
}

// ===================================================================
// 2. Nested SerializedNode trees
// ===================================================================

#[test]
fn test_two_level_nesting() {
    let tree = branch_node(
        "program",
        vec![branch_node(
            "statement",
            vec![leaf_node("id", "x", 0, 1)],
            0,
            1,
        )],
        0,
        1,
    );
    assert_eq!(tree.children.len(), 1);
    assert_eq!(tree.children[0].children.len(), 1);
    assert_eq!(tree.children[0].children[0].kind, "id");
}

#[test]
fn test_three_level_nesting() {
    let tree = branch_node(
        "root",
        vec![branch_node(
            "level1",
            vec![branch_node(
                "level2",
                vec![leaf_node("leaf", "v", 0, 1)],
                0,
                1,
            )],
            0,
            1,
        )],
        0,
        1,
    );
    assert_eq!(tree.children[0].children[0].children[0].kind, "leaf");
}

#[test]
fn test_wide_tree() {
    let children: Vec<_> = (0..10)
        .map(|i| leaf_node("item", &format!("i{}", i), i, i + 1))
        .collect();
    let tree = branch_node("list", children, 0, 10);
    assert_eq!(tree.children.len(), 10);
    assert_eq!(tree.children[9].text, Some("i9".to_string()));
}

#[test]
fn test_mixed_named_unnamed_children() {
    let named = leaf_node("identifier", "x", 0, 1);
    let unnamed = SerializedNode {
        kind: ";".to_string(),
        is_named: false,
        field_name: None,
        start_position: (0, 1),
        end_position: (0, 2),
        start_byte: 1,
        end_byte: 2,
        text: Some(";".to_string()),
        children: vec![],
        is_error: false,
        is_missing: false,
    };
    let parent = branch_node("statement", vec![named, unnamed], 0, 2);
    assert!(parent.children[0].is_named);
    assert!(!parent.children[1].is_named);
}

// ===================================================================
// 3. JSON serialization / deserialization roundtrip
// ===================================================================

#[test]
fn test_leaf_json_roundtrip() {
    let original = leaf_node("number", "42", 0, 2);
    let json = serde_json::to_string(&original).unwrap();
    let decoded: SerializedNode = serde_json::from_str(&json).unwrap();
    assert_node_eq(&original, &decoded);
}

#[test]
fn test_branch_json_roundtrip() {
    let original = branch_node(
        "binary_expr",
        vec![
            leaf_node("number", "1", 0, 1),
            leaf_node("operator", "+", 2, 3),
            leaf_node("number", "2", 4, 5),
        ],
        0,
        5,
    );
    let json = serde_json::to_string(&original).unwrap();
    let decoded: SerializedNode = serde_json::from_str(&json).unwrap();
    assert_node_eq(&original, &decoded);
}

#[test]
fn test_pretty_json_roundtrip() {
    let original = branch_node("prog", vec![leaf_node("id", "x", 0, 1)], 0, 1);
    let json = serde_json::to_string_pretty(&original).unwrap();
    assert!(json.contains('\n'));
    let decoded: SerializedNode = serde_json::from_str(&json).unwrap();
    assert_node_eq(&original, &decoded);
}

#[test]
fn test_json_contains_all_fields() {
    let n = leaf_node("foo", "bar", 3, 6);
    let json = serde_json::to_string(&n).unwrap();
    assert!(json.contains("\"kind\""));
    assert!(json.contains("\"is_named\""));
    assert!(json.contains("\"field_name\""));
    assert!(json.contains("\"start_position\""));
    assert!(json.contains("\"end_position\""));
    assert!(json.contains("\"start_byte\""));
    assert!(json.contains("\"end_byte\""));
    assert!(json.contains("\"text\""));
    assert!(json.contains("\"children\""));
    assert!(json.contains("\"is_error\""));
    assert!(json.contains("\"is_missing\""));
}

#[test]
fn test_json_field_name_present() {
    let mut n = leaf_node("id", "x", 0, 1);
    n.field_name = Some("left".to_string());
    let json = serde_json::to_string(&n).unwrap();
    assert!(json.contains("\"left\""));
    let decoded: SerializedNode = serde_json::from_str(&json).unwrap();
    assert_eq!(decoded.field_name, Some("left".to_string()));
}

#[test]
fn test_json_field_name_null() {
    let n = leaf_node("id", "x", 0, 1);
    let json = serde_json::to_string(&n).unwrap();
    assert!(json.contains("\"field_name\":null"));
    let decoded: SerializedNode = serde_json::from_str(&json).unwrap();
    assert_eq!(decoded.field_name, None);
}

#[test]
fn test_json_text_null_for_branch() {
    let n = branch_node("root", vec![leaf_node("a", "a", 0, 1)], 0, 1);
    let json = serde_json::to_string(&n).unwrap();
    assert!(json.contains("\"text\":null"));
}

#[test]
fn test_nested_json_roundtrip() {
    let deep = branch_node(
        "a",
        vec![branch_node(
            "b",
            vec![branch_node("c", vec![leaf_node("d", "v", 0, 1)], 0, 1)],
            0,
            1,
        )],
        0,
        1,
    );
    let json = serde_json::to_string(&deep).unwrap();
    let decoded: SerializedNode = serde_json::from_str(&json).unwrap();
    assert_node_eq(&deep, &decoded);
}

#[test]
fn test_error_node_json_roundtrip() {
    let mut n = leaf_node("ERROR", "bad", 5, 8);
    n.is_error = true;
    n.is_named = false;
    let json = serde_json::to_string(&n).unwrap();
    let decoded: SerializedNode = serde_json::from_str(&json).unwrap();
    assert!(decoded.is_error);
    assert!(!decoded.is_named);
}

#[test]
fn test_missing_node_json_roundtrip() {
    let mut n = leaf_node("identifier", "", 10, 10);
    n.is_missing = true;
    n.text = None;
    let json = serde_json::to_string(&n).unwrap();
    let decoded: SerializedNode = serde_json::from_str(&json).unwrap();
    assert!(decoded.is_missing);
    assert_eq!(decoded.text, None);
}

#[test]
fn test_json_double_roundtrip() {
    let original = branch_node(
        "root",
        vec![
            leaf_node("a", "hello", 0, 5),
            leaf_node("b", "world", 6, 11),
        ],
        0,
        11,
    );
    let json1 = serde_json::to_string(&original).unwrap();
    let decoded1: SerializedNode = serde_json::from_str(&json1).unwrap();
    let json2 = serde_json::to_string(&decoded1).unwrap();
    assert_eq!(json1, json2);
}

// ===================================================================
// 4. SExpr generation and structure
// ===================================================================

#[test]
fn test_sexpr_atom() {
    let a = SExpr::Atom("hello".to_string());
    if let SExpr::Atom(ref s) = a {
        assert_eq!(s, "hello");
    } else {
        panic!("Expected Atom");
    }
}

#[test]
fn test_sexpr_list_empty() {
    let l = SExpr::List(vec![]);
    if let SExpr::List(ref items) = l {
        assert!(items.is_empty());
    } else {
        panic!("Expected List");
    }
}

#[test]
fn test_sexpr_list_with_atoms() {
    let l = SExpr::List(vec![
        SExpr::Atom("a".to_string()),
        SExpr::Atom("b".to_string()),
    ]);
    if let SExpr::List(ref items) = l {
        assert_eq!(items.len(), 2);
    } else {
        panic!("Expected List");
    }
}

#[test]
fn test_sexpr_nested_list() {
    let inner = SExpr::List(vec![SExpr::Atom("x".to_string())]);
    let outer = SExpr::List(vec![SExpr::Atom("fn".to_string()), inner]);
    if let SExpr::List(ref items) = outer {
        assert_eq!(items.len(), 2);
        assert!(matches!(&items[1], SExpr::List(_)));
    }
}

#[test]
fn test_sexpr_equality() {
    let a = SExpr::Atom("hello".to_string());
    let b = SExpr::Atom("hello".to_string());
    assert_eq!(a, b);
}

#[test]
fn test_sexpr_inequality() {
    let a = SExpr::Atom("hello".to_string());
    let b = SExpr::Atom("world".to_string());
    assert_ne!(a, b);
}

#[test]
fn test_sexpr_list_equality() {
    let a = SExpr::List(vec![SExpr::Atom("x".to_string())]);
    let b = SExpr::List(vec![SExpr::Atom("x".to_string())]);
    assert_eq!(a, b);
}

#[test]
fn test_sexpr_list_inequality_different_length() {
    let a = SExpr::List(vec![SExpr::Atom("x".to_string())]);
    let b = SExpr::List(vec![
        SExpr::Atom("x".to_string()),
        SExpr::Atom("y".to_string()),
    ]);
    assert_ne!(a, b);
}

#[test]
fn test_sexpr_atom_vs_list() {
    let a = SExpr::Atom("x".to_string());
    let b = SExpr::List(vec![SExpr::Atom("x".to_string())]);
    assert_ne!(a, b);
}

#[test]
fn test_sexpr_clone() {
    let original = SExpr::List(vec![SExpr::Atom("hi".to_string())]);
    let cloned = original.clone();
    assert_eq!(original, cloned);
}

#[test]
fn test_sexpr_debug_format() {
    let a = SExpr::Atom("hello".to_string());
    let dbg = format!("{:?}", a);
    assert!(dbg.contains("Atom"));
    assert!(dbg.contains("hello"));
}

#[test]
fn test_sexpr_json_roundtrip() {
    let original = SExpr::List(vec![
        SExpr::Atom("define".to_string()),
        SExpr::List(vec![
            SExpr::Atom("x".to_string()),
            SExpr::Atom("1".to_string()),
        ]),
    ]);
    let json = serde_json::to_string(&original).unwrap();
    let decoded: SExpr = serde_json::from_str(&json).unwrap();
    assert_eq!(original, decoded);
}

#[test]
fn test_parse_sexpr_returns_ok() {
    let result = parse_sexpr("(foo bar)");
    assert!(result.is_ok());
}

// ===================================================================
// 5. CompactNode / CompactSerializer output
// ===================================================================

#[test]
fn test_compact_node_leaf_serialization() {
    let node = CompactNode {
        kind: "id".to_string(),
        start: None,
        end: None,
        field: None,
        children: vec![],
        text: Some("test".to_string()),
    };
    let json = serde_json::to_string(&node).unwrap();
    assert!(json.contains("\"t\":\"id\""));
    assert!(json.contains("\"x\":\"test\""));
    // start/end omitted for leaf
    assert!(!json.contains("\"s\""));
    assert!(!json.contains("\"e\""));
}

#[test]
fn test_compact_node_branch_serialization() {
    let child = CompactNode {
        kind: "leaf".to_string(),
        start: None,
        end: None,
        field: None,
        children: vec![],
        text: Some("v".to_string()),
    };
    let parent = CompactNode {
        kind: "root".to_string(),
        start: Some(0),
        end: Some(5),
        field: None,
        children: vec![child],
        text: None,
    };
    let json = serde_json::to_string(&parent).unwrap();
    assert!(json.contains("\"s\":0"));
    assert!(json.contains("\"e\":5"));
    assert!(json.contains("\"c\":["));
}

#[test]
fn test_compact_node_field_serialization() {
    let node = CompactNode {
        kind: "identifier".to_string(),
        start: Some(0),
        end: Some(3),
        field: Some("name".to_string()),
        children: vec![],
        text: Some("foo".to_string()),
    };
    let json = serde_json::to_string(&node).unwrap();
    assert!(json.contains("\"f\":\"name\""));
}

#[test]
fn test_compact_node_no_field() {
    let node = CompactNode {
        kind: "number".to_string(),
        start: None,
        end: None,
        field: None,
        children: vec![],
        text: Some("42".to_string()),
    };
    let json = serde_json::to_string(&node).unwrap();
    assert!(!json.contains("\"f\""));
}

#[test]
fn test_compact_node_empty_children_omitted() {
    let node = CompactNode {
        kind: "leaf".to_string(),
        start: None,
        end: None,
        field: None,
        children: vec![],
        text: Some("v".to_string()),
    };
    let json = serde_json::to_string(&node).unwrap();
    assert!(!json.contains("\"c\""));
}

#[test]
fn test_compact_node_json_roundtrip() {
    let original = CompactNode {
        kind: "expr".to_string(),
        start: Some(0),
        end: Some(10),
        field: Some("body".to_string()),
        children: vec![CompactNode {
            kind: "num".to_string(),
            start: None,
            end: None,
            field: None,
            children: vec![],
            text: Some("7".to_string()),
        }],
        text: None,
    };
    let json = serde_json::to_string(&original).unwrap();
    let decoded: CompactNode = serde_json::from_str(&json).unwrap();
    assert_eq!(decoded.kind, "expr");
    assert_eq!(decoded.start, Some(0));
    assert_eq!(decoded.end, Some(10));
    assert_eq!(decoded.field, Some("body".to_string()));
    assert_eq!(decoded.children.len(), 1);
    assert_eq!(decoded.children[0].text, Some("7".to_string()));
}

#[test]
fn test_compact_node_no_text_no_children() {
    let node = CompactNode {
        kind: "empty".to_string(),
        start: Some(0),
        end: Some(0),
        field: None,
        children: vec![],
        text: None,
    };
    let json = serde_json::to_string(&node).unwrap();
    assert!(!json.contains("\"x\""));
    assert!(!json.contains("\"c\""));
}

// ===================================================================
// 6. TreeSerializer configuration
// ===================================================================

#[test]
fn test_tree_serializer_defaults() {
    let src = b"hello";
    let ser = TreeSerializer::new(src);
    assert!(!ser.include_unnamed);
    assert_eq!(ser.max_text_length, Some(100));
    assert_eq!(ser.source, b"hello");
}

#[test]
fn test_tree_serializer_with_unnamed() {
    let ser = TreeSerializer::new(b"x").with_unnamed_nodes();
    assert!(ser.include_unnamed);
}

#[test]
fn test_tree_serializer_with_max_text_length() {
    let ser = TreeSerializer::new(b"x").with_max_text_length(Some(50));
    assert_eq!(ser.max_text_length, Some(50));
}

#[test]
fn test_tree_serializer_no_max_text_length() {
    let ser = TreeSerializer::new(b"x").with_max_text_length(None);
    assert_eq!(ser.max_text_length, None);
}

#[test]
fn test_tree_serializer_chained_config() {
    let ser = TreeSerializer::new(b"code")
        .with_unnamed_nodes()
        .with_max_text_length(Some(200));
    assert!(ser.include_unnamed);
    assert_eq!(ser.max_text_length, Some(200));
}

// ===================================================================
// 7. SExpressionSerializer configuration
// ===================================================================

#[test]
fn test_sexpr_serializer_construction() {
    // Verify SExpressionSerializer can be constructed without panic
    let _ser = SExpressionSerializer::new(b"src");
}

#[test]
fn test_sexpr_serializer_with_positions_chaining() {
    // Verify with_positions() chaining works without panic
    let _ser = SExpressionSerializer::new(b"src").with_positions();
}

// ===================================================================
// 8. Edge cases
// ===================================================================

#[test]
fn test_empty_kind_string() {
    let n = leaf_node("", "text", 0, 4);
    let json = serde_json::to_string(&n).unwrap();
    let decoded: SerializedNode = serde_json::from_str(&json).unwrap();
    assert_eq!(decoded.kind, "");
}

#[test]
fn test_empty_text_string() {
    let mut n = leaf_node("empty", "", 0, 0);
    n.text = Some(String::new());
    let json = serde_json::to_string(&n).unwrap();
    let decoded: SerializedNode = serde_json::from_str(&json).unwrap();
    assert_eq!(decoded.text, Some(String::new()));
}

#[test]
fn test_no_text() {
    let mut n = leaf_node("missing", "", 5, 5);
    n.text = None;
    let json = serde_json::to_string(&n).unwrap();
    let decoded: SerializedNode = serde_json::from_str(&json).unwrap();
    assert_eq!(decoded.text, None);
}

#[test]
fn test_zero_length_span() {
    let n = leaf_node("phantom", "", 10, 10);
    assert_eq!(n.start_byte, n.end_byte);
    assert_eq!(n.start_position, n.end_position);
}

#[test]
fn test_unicode_text_roundtrip() {
    let n = leaf_node("string", "héllo wörld", 0, 13);
    let json = serde_json::to_string(&n).unwrap();
    let decoded: SerializedNode = serde_json::from_str(&json).unwrap();
    assert_eq!(decoded.text, Some("héllo wörld".to_string()));
}

#[test]
fn test_unicode_emoji_roundtrip() {
    let n = leaf_node("emoji", "🎉🚀", 0, 8);
    let json = serde_json::to_string(&n).unwrap();
    let decoded: SerializedNode = serde_json::from_str(&json).unwrap();
    assert_eq!(decoded.text, Some("🎉🚀".to_string()));
}

#[test]
fn test_unicode_cjk_roundtrip() {
    let n = leaf_node("cjk", "你好世界", 0, 12);
    let json = serde_json::to_string(&n).unwrap();
    let decoded: SerializedNode = serde_json::from_str(&json).unwrap();
    assert_eq!(decoded.text, Some("你好世界".to_string()));
}

#[test]
fn test_special_chars_in_text() {
    let n = leaf_node("string", "line1\nline2\ttab", 0, 14);
    let json = serde_json::to_string(&n).unwrap();
    let decoded: SerializedNode = serde_json::from_str(&json).unwrap();
    assert_eq!(decoded.text, Some("line1\nline2\ttab".to_string()));
}

#[test]
fn test_quotes_in_text() {
    let n = leaf_node("string", r#"he said "hi""#, 0, 12);
    let json = serde_json::to_string(&n).unwrap();
    let decoded: SerializedNode = serde_json::from_str(&json).unwrap();
    assert_eq!(decoded.text, Some(r#"he said "hi""#.to_string()));
}

#[test]
fn test_backslash_in_text() {
    let n = leaf_node("path", r"C:\Users\test", 0, 13);
    let json = serde_json::to_string(&n).unwrap();
    let decoded: SerializedNode = serde_json::from_str(&json).unwrap();
    assert_eq!(decoded.text, Some(r"C:\Users\test".to_string()));
}

#[test]
fn test_null_byte_in_kind() {
    let n = leaf_node("kind\0with_null", "v", 0, 1);
    let json = serde_json::to_string(&n).unwrap();
    let decoded: SerializedNode = serde_json::from_str(&json).unwrap();
    assert_eq!(decoded.kind, "kind\0with_null");
}

#[test]
fn test_empty_children_vec() {
    let n = branch_node("root", vec![], 0, 0);
    assert!(n.children.is_empty());
    let json = serde_json::to_string(&n).unwrap();
    let decoded: SerializedNode = serde_json::from_str(&json).unwrap();
    assert!(decoded.children.is_empty());
}

#[test]
fn test_large_byte_offsets() {
    let n = SerializedNode {
        kind: "token".to_string(),
        is_named: true,
        field_name: None,
        start_position: (999, 888),
        end_position: (1000, 0),
        start_byte: 1_000_000,
        end_byte: 1_000_010,
        text: Some("big offset".to_string()),
        children: vec![],
        is_error: false,
        is_missing: false,
    };
    let json = serde_json::to_string(&n).unwrap();
    let decoded: SerializedNode = serde_json::from_str(&json).unwrap();
    assert_eq!(decoded.start_byte, 1_000_000);
    assert_eq!(decoded.end_byte, 1_000_010);
    assert_eq!(decoded.start_position, (999, 888));
    assert_eq!(decoded.end_position, (1000, 0));
}

#[test]
fn test_multiline_position() {
    let n = SerializedNode {
        kind: "block".to_string(),
        is_named: true,
        field_name: None,
        start_position: (0, 0),
        end_position: (5, 1),
        start_byte: 0,
        end_byte: 30,
        text: None,
        children: vec![],
        is_error: false,
        is_missing: false,
    };
    let json = serde_json::to_string(&n).unwrap();
    let decoded: SerializedNode = serde_json::from_str(&json).unwrap();
    assert_eq!(decoded.start_position.0, 0);
    assert_eq!(decoded.end_position.0, 5);
}

// ===================================================================
// 9. Large trees
// ===================================================================

#[test]
fn test_wide_tree_100_children() {
    let children: Vec<_> = (0..100)
        .map(|i| leaf_node("item", &format!("v{}", i), i, i + 1))
        .collect();
    let tree = branch_node("list", children, 0, 100);
    let json = serde_json::to_string(&tree).unwrap();
    let decoded: SerializedNode = serde_json::from_str(&json).unwrap();
    assert_eq!(decoded.children.len(), 100);
    assert_eq!(decoded.children[99].text, Some("v99".to_string()));
}

#[test]
fn test_large_tree_many_branches() {
    let tree = branch_node(
        "program",
        (0..50)
            .map(|i| {
                branch_node(
                    "statement",
                    vec![leaf_node("id", &format!("s{}", i), i * 2, i * 2 + 1)],
                    i * 2,
                    i * 2 + 1,
                )
            })
            .collect(),
        0,
        100,
    );
    let json = serde_json::to_string(&tree).unwrap();
    let decoded: SerializedNode = serde_json::from_str(&json).unwrap();
    assert_eq!(decoded.children.len(), 50);
    for (i, child) in decoded.children.iter().enumerate() {
        assert_eq!(child.children[0].text, Some(format!("s{}", i)));
    }
}

#[test]
fn test_json_size_grows_with_tree() {
    let small = branch_node("r", vec![leaf_node("a", "x", 0, 1)], 0, 1);
    let big = branch_node(
        "r",
        (0..20).map(|i| leaf_node("a", "x", i, i + 1)).collect(),
        0,
        20,
    );
    let small_json = serde_json::to_string(&small).unwrap();
    let big_json = serde_json::to_string(&big).unwrap();
    assert!(big_json.len() > small_json.len());
}

// ===================================================================
// 10. Deep nesting
// ===================================================================

#[test]
fn test_deep_nesting_10_levels() {
    let mut current = leaf_node("leaf", "deep", 0, 4);
    for i in (0..10).rev() {
        current = branch_node(&format!("level{}", i), vec![current], 0, 4);
    }
    let json = serde_json::to_string(&current).unwrap();
    let decoded: SerializedNode = serde_json::from_str(&json).unwrap();

    let mut node = &decoded;
    for i in 0..10 {
        assert_eq!(node.kind, format!("level{}", i));
        assert_eq!(node.children.len(), 1);
        node = &node.children[0];
    }
    assert_eq!(node.kind, "leaf");
    assert_eq!(node.text, Some("deep".to_string()));
}

#[test]
fn test_deep_nesting_50_levels() {
    let mut current = leaf_node("bottom", "val", 0, 3);
    for _ in 0..50 {
        current = branch_node("wrapper", vec![current], 0, 3);
    }
    let json = serde_json::to_string(&current).unwrap();
    let decoded: SerializedNode = serde_json::from_str(&json).unwrap();

    let mut depth = 0;
    let mut node = &decoded;
    while !node.children.is_empty() {
        node = &node.children[0];
        depth += 1;
    }
    assert_eq!(depth, 50);
    assert_eq!(node.kind, "bottom");
}

// ===================================================================
// 11. Error nodes and missing nodes
// ===================================================================

#[test]
fn test_error_node_fields() {
    let n = SerializedNode {
        kind: "ERROR".to_string(),
        is_named: false,
        field_name: None,
        start_position: (1, 5),
        end_position: (1, 10),
        start_byte: 15,
        end_byte: 20,
        text: Some("invalid".to_string()),
        children: vec![],
        is_error: true,
        is_missing: false,
    };
    assert!(n.is_error);
    assert!(!n.is_missing);
    assert!(!n.is_named);
    assert_eq!(n.kind, "ERROR");
}

#[test]
fn test_error_node_with_children() {
    let err = SerializedNode {
        kind: "ERROR".to_string(),
        is_named: false,
        field_name: None,
        start_position: (0, 0),
        end_position: (0, 10),
        start_byte: 0,
        end_byte: 10,
        text: None,
        children: vec![leaf_node("id", "x", 0, 1), leaf_node("junk", "!!!", 2, 5)],
        is_error: true,
        is_missing: false,
    };
    let json = serde_json::to_string(&err).unwrap();
    let decoded: SerializedNode = serde_json::from_str(&json).unwrap();
    assert!(decoded.is_error);
    assert_eq!(decoded.children.len(), 2);
}

#[test]
fn test_missing_node_zero_width() {
    let n = SerializedNode {
        kind: "semicolon".to_string(),
        is_named: true,
        field_name: None,
        start_position: (2, 5),
        end_position: (2, 5),
        start_byte: 30,
        end_byte: 30,
        text: None,
        children: vec![],
        is_error: false,
        is_missing: true,
    };
    assert!(n.is_missing);
    assert_eq!(n.start_byte, n.end_byte);
}

#[test]
fn test_both_error_and_missing() {
    let n = SerializedNode {
        kind: "MISSING".to_string(),
        is_named: false,
        field_name: None,
        start_position: (0, 0),
        end_position: (0, 0),
        start_byte: 0,
        end_byte: 0,
        text: None,
        children: vec![],
        is_error: true,
        is_missing: true,
    };
    let json = serde_json::to_string(&n).unwrap();
    let decoded: SerializedNode = serde_json::from_str(&json).unwrap();
    assert!(decoded.is_error);
    assert!(decoded.is_missing);
}

#[test]
fn test_error_nodes_in_tree() {
    let tree = branch_node(
        "program",
        vec![
            leaf_node("id", "x", 0, 1),
            SerializedNode {
                kind: "ERROR".to_string(),
                is_named: false,
                field_name: None,
                start_position: (0, 2),
                end_position: (0, 5),
                start_byte: 2,
                end_byte: 5,
                text: Some("!!!".to_string()),
                children: vec![],
                is_error: true,
                is_missing: false,
            },
            leaf_node("id", "y", 6, 7),
        ],
        0,
        7,
    );
    let json = serde_json::to_string(&tree).unwrap();
    let decoded: SerializedNode = serde_json::from_str(&json).unwrap();
    assert!(!decoded.children[0].is_error);
    assert!(decoded.children[1].is_error);
    assert!(!decoded.children[2].is_error);
}

// ===================================================================
// 12. Field name presence / absence
// ===================================================================

#[test]
fn test_various_field_names() {
    let fields = ["left", "right", "name", "body", "condition", "alternative"];
    for field in &fields {
        let mut n = leaf_node("id", "v", 0, 1);
        n.field_name = Some(field.to_string());
        let json = serde_json::to_string(&n).unwrap();
        let decoded: SerializedNode = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.field_name, Some(field.to_string()));
    }
}

#[test]
fn test_field_name_on_branch() {
    let mut n = branch_node("block", vec![leaf_node("stmt", "x", 0, 1)], 0, 1);
    n.field_name = Some("body".to_string());
    let json = serde_json::to_string(&n).unwrap();
    let decoded: SerializedNode = serde_json::from_str(&json).unwrap();
    assert_eq!(decoded.field_name, Some("body".to_string()));
}

#[test]
fn test_mixed_field_names_in_children() {
    let mut left = leaf_node("id", "a", 0, 1);
    left.field_name = Some("left".to_string());
    let op = leaf_node("op", "+", 2, 3);
    let mut right = leaf_node("id", "b", 4, 5);
    right.field_name = Some("right".to_string());

    let tree = branch_node("binary_expr", vec![left, op, right], 0, 5);
    let json = serde_json::to_string(&tree).unwrap();
    let decoded: SerializedNode = serde_json::from_str(&json).unwrap();
    assert_eq!(decoded.children[0].field_name, Some("left".to_string()));
    assert_eq!(decoded.children[1].field_name, None);
    assert_eq!(decoded.children[2].field_name, Some("right".to_string()));
}

#[test]
fn test_empty_field_name_string() {
    let mut n = leaf_node("id", "x", 0, 1);
    n.field_name = Some(String::new());
    let json = serde_json::to_string(&n).unwrap();
    let decoded: SerializedNode = serde_json::from_str(&json).unwrap();
    assert_eq!(decoded.field_name, Some(String::new()));
}

// ===================================================================
// Additional: BinaryFormat, Clone, Debug
// ===================================================================

#[test]
fn test_serialized_node_clone() {
    let original = branch_node("root", vec![leaf_node("a", "hello", 0, 5)], 0, 5);
    let cloned = original.clone();
    assert_node_eq(&original, &cloned);
}

#[test]
fn test_serialized_node_debug() {
    let n = leaf_node("id", "x", 0, 1);
    let dbg = format!("{:?}", n);
    assert!(dbg.contains("id"));
    assert!(dbg.contains("SerializedNode"));
}

#[test]
fn test_compact_node_clone() {
    let original = CompactNode {
        kind: "num".to_string(),
        start: Some(0),
        end: Some(2),
        field: None,
        children: vec![],
        text: Some("42".to_string()),
    };
    let cloned = original.clone();
    assert_eq!(cloned.kind, "num");
    assert_eq!(cloned.text, Some("42".to_string()));
}

#[test]
fn test_compact_node_debug() {
    let n = CompactNode {
        kind: "test".to_string(),
        start: None,
        end: None,
        field: None,
        children: vec![],
        text: None,
    };
    let dbg = format!("{:?}", n);
    assert!(dbg.contains("CompactNode"));
}

#[test]
fn test_binary_format_construction() {
    let bf = BinaryFormat {
        node_types: vec!["program".to_string(), "id".to_string()],
        field_names: vec!["name".to_string()],
        tree_data: vec![0, 1, 2, 3],
    };
    assert_eq!(bf.node_types.len(), 2);
    assert_eq!(bf.field_names.len(), 1);
    assert_eq!(bf.tree_data.len(), 4);
}

#[test]
fn test_binary_format_clone() {
    let bf = BinaryFormat {
        node_types: vec!["a".to_string()],
        field_names: vec![],
        tree_data: vec![42],
    };
    let cloned = bf.clone();
    assert_eq!(cloned.node_types, bf.node_types);
    assert_eq!(cloned.tree_data, bf.tree_data);
}

#[test]
fn test_binary_serializer_new() {
    // Just verify it can be constructed without panic
    let _bs = BinarySerializer::new();
}

#[test]
fn test_binary_serializer_default() {
    let _bs: BinarySerializer = Default::default();
}

// ===================================================================
// Additional: JSON value inspection
// ===================================================================

#[test]
fn test_json_value_types() {
    let n = SerializedNode {
        kind: "test".to_string(),
        is_named: true,
        field_name: Some("f".to_string()),
        start_position: (1, 2),
        end_position: (3, 4),
        start_byte: 10,
        end_byte: 20,
        text: Some("val".to_string()),
        children: vec![],
        is_error: false,
        is_missing: true,
    };
    let v: serde_json::Value = serde_json::to_value(&n).unwrap();
    assert!(v["kind"].is_string());
    assert!(v["is_named"].is_boolean());
    assert!(v["field_name"].is_string());
    assert!(v["start_byte"].is_number());
    assert!(v["end_byte"].is_number());
    assert!(v["text"].is_string());
    assert!(v["children"].is_array());
    assert!(v["is_error"].is_boolean());
    assert!(v["is_missing"].is_boolean());
}

#[test]
fn test_json_position_is_array() {
    let n = leaf_node("x", "y", 0, 1);
    let v: serde_json::Value = serde_json::to_value(&n).unwrap();
    assert!(v["start_position"].is_array());
    assert!(v["end_position"].is_array());
    assert_eq!(v["start_position"][0], 0);
    assert_eq!(v["start_position"][1], 0);
}

#[test]
fn test_deterministic_serialization() {
    let n = branch_node(
        "root",
        vec![leaf_node("a", "x", 0, 1), leaf_node("b", "y", 2, 3)],
        0,
        3,
    );
    let json1 = serde_json::to_string(&n).unwrap();
    let json2 = serde_json::to_string(&n).unwrap();
    assert_eq!(json1, json2);
}

#[test]
fn test_from_json_string_literal() {
    let json = r#"{
        "kind": "number",
        "is_named": true,
        "field_name": null,
        "start_position": [0, 0],
        "end_position": [0, 3],
        "start_byte": 0,
        "end_byte": 3,
        "text": "123",
        "children": [],
        "is_error": false,
        "is_missing": false
    }"#;
    let n: SerializedNode = serde_json::from_str(json).unwrap();
    assert_eq!(n.kind, "number");
    assert!(n.is_named);
    assert_eq!(n.text, Some("123".to_string()));
    assert_eq!(n.start_position, (0, 0));
    assert_eq!(n.end_position, (0, 3));
}

#[test]
fn test_sexpr_deeply_nested() {
    let mut expr = SExpr::Atom("leaf".to_string());
    for _ in 0..20 {
        expr = SExpr::List(vec![expr]);
    }
    let json = serde_json::to_string(&expr).unwrap();
    let decoded: SExpr = serde_json::from_str(&json).unwrap();
    assert_eq!(expr, decoded);
}

#[test]
fn test_sexpr_wide_list() {
    let items: Vec<SExpr> = (0..100)
        .map(|i| SExpr::Atom(format!("item{}", i)))
        .collect();
    let list = SExpr::List(items);
    if let SExpr::List(ref v) = list {
        assert_eq!(v.len(), 100);
    }
    let json = serde_json::to_string(&list).unwrap();
    let decoded: SExpr = serde_json::from_str(&json).unwrap();
    assert_eq!(list, decoded);
}

#[test]
fn test_compact_node_nested_roundtrip() {
    let tree = CompactNode {
        kind: "program".to_string(),
        start: Some(0),
        end: Some(20),
        field: None,
        children: vec![CompactNode {
            kind: "func".to_string(),
            start: Some(0),
            end: Some(20),
            field: Some("body".to_string()),
            children: vec![CompactNode {
                kind: "id".to_string(),
                start: None,
                end: None,
                field: Some("name".to_string()),
                children: vec![],
                text: Some("main".to_string()),
            }],
            text: None,
        }],
        text: None,
    };
    let json = serde_json::to_string(&tree).unwrap();
    let decoded: CompactNode = serde_json::from_str(&json).unwrap();
    assert_eq!(decoded.kind, "program");
    assert_eq!(decoded.children[0].kind, "func");
    assert_eq!(
        decoded.children[0].children[0].text,
        Some("main".to_string())
    );
}

#[test]
fn test_special_json_chars_in_kind() {
    let n = leaf_node("node/type:special", "v", 0, 1);
    let json = serde_json::to_string(&n).unwrap();
    let decoded: SerializedNode = serde_json::from_str(&json).unwrap();
    assert_eq!(decoded.kind, "node/type:special");
}

#[test]
fn test_very_long_text() {
    let long_text: String = "a".repeat(10_000);
    let n = leaf_node("bigtext", &long_text, 0, 10_000);
    let json = serde_json::to_string(&n).unwrap();
    let decoded: SerializedNode = serde_json::from_str(&json).unwrap();
    assert_eq!(decoded.text.unwrap().len(), 10_000);
}
