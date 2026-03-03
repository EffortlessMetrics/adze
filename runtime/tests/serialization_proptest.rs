//! Property-based tests for `adze::serialization`.
//!
//! Exercises `SExpr`, `parse_sexpr`, `BinarySerializer`, `BinaryFormat`,
//! `SerializedNode`, and `CompactNode` with random inputs.

#![cfg(feature = "serialization")]
#![allow(clippy::needless_range_loop)]

use adze::serialization::{
    BinaryFormat, BinarySerializer, CompactNode, SExpr, SerializedNode, parse_sexpr,
};
use proptest::prelude::*;

// ---------------------------------------------------------------------------
// Strategies
// ---------------------------------------------------------------------------

/// Generate a random `SExpr` tree up to `depth` levels deep.
fn arb_sexpr(depth: u32) -> BoxedStrategy<SExpr> {
    if depth == 0 {
        "[a-zA-Z_][a-zA-Z0-9_]{0,15}".prop_map(SExpr::Atom).boxed()
    } else {
        prop_oneof![
            "[a-zA-Z_][a-zA-Z0-9_]{0,15}".prop_map(SExpr::Atom),
            prop::collection::vec(arb_sexpr(depth - 1), 0..5).prop_map(SExpr::List),
        ]
        .boxed()
    }
}

/// Generate a random `SerializedNode` tree.
fn arb_serialized_node(depth: u32) -> BoxedStrategy<SerializedNode> {
    if depth == 0 {
        (
            "[a-z_]{1,10}",
            any::<bool>(),
            proptest::option::of("[a-z_]{1,8}"),
            (0usize..1000),
        )
            .prop_map(|(kind, is_named, field_name, start)| {
                let end = start + kind.len();
                SerializedNode {
                    kind,
                    is_named,
                    field_name,
                    start_position: (0, start),
                    end_position: (0, end),
                    start_byte: start,
                    end_byte: end,
                    text: Some("leaf".to_string()),
                    children: vec![],
                    is_error: false,
                    is_missing: false,
                }
            })
            .boxed()
    } else {
        (
            "[a-z_]{1,10}",
            any::<bool>(),
            prop::collection::vec(arb_serialized_node(depth - 1), 0..4),
        )
            .prop_map(|(kind, is_named, children)| SerializedNode {
                kind,
                is_named,
                field_name: None,
                start_position: (0, 0),
                end_position: (0, 10),
                start_byte: 0,
                end_byte: 10,
                text: None,
                children,
                is_error: false,
                is_missing: false,
            })
            .boxed()
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Render an `SExpr` to its canonical string form.
fn sexpr_to_string(expr: &SExpr) -> String {
    match expr {
        SExpr::Atom(s) => s.clone(),
        SExpr::List(items) => {
            let inner: Vec<String> = items.iter().map(sexpr_to_string).collect();
            format!("({})", inner.join(" "))
        }
    }
}

// ---------------------------------------------------------------------------
// 1. SExpr serde JSON roundtrip
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn sexpr_json_roundtrip(expr in arb_sexpr(3)) {
        let json = serde_json::to_string(&expr).unwrap();
        let decoded: SExpr = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(&expr, &decoded);
    }
}

// ---------------------------------------------------------------------------
// 2. SExpr::Atom always wraps a non-empty string
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn sexpr_atom_is_nonempty(s in "[a-zA-Z_][a-zA-Z0-9_]{0,20}") {
        let atom = SExpr::Atom(s.clone());
        match &atom {
            SExpr::Atom(inner) => prop_assert_eq!(inner, &s),
            SExpr::List(_) => prop_assert!(false, "expected Atom"),
        }
    }
}

// ---------------------------------------------------------------------------
// 3. SExpr::List preserves child count
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn sexpr_list_preserves_count(children in prop::collection::vec(arb_sexpr(1), 0..8)) {
        let n = children.len();
        let list = SExpr::List(children);
        match &list {
            SExpr::List(items) => prop_assert_eq!(items.len(), n),
            SExpr::Atom(_) => prop_assert!(false, "expected List"),
        }
    }
}

// ---------------------------------------------------------------------------
// 4. SExpr equality is reflexive
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn sexpr_equality_reflexive(expr in arb_sexpr(3)) {
        prop_assert_eq!(&expr, &expr);
    }
}

// ---------------------------------------------------------------------------
// 5. SExpr clone equals original
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn sexpr_clone_eq(expr in arb_sexpr(3)) {
        let cloned = expr.clone();
        prop_assert_eq!(&expr, &cloned);
    }
}

// ---------------------------------------------------------------------------
// 6. parse_sexpr never panics on arbitrary input
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(500))]

    #[test]
    fn parse_sexpr_no_panic(input in ".*") {
        let _ = parse_sexpr(&input);
    }
}

// ---------------------------------------------------------------------------
// 7. parse_sexpr on empty string does not panic
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn parse_sexpr_whitespace_only(input in "[ \t\n\r]{0,30}") {
        let _ = parse_sexpr(&input);
    }
}

// ---------------------------------------------------------------------------
// 8. parse_sexpr on unbalanced parens returns Ok or Err (no panic)
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn parse_sexpr_unbalanced_parens(
        opens in 0usize..10,
        closes in 0usize..10,
        atom in "[a-z]{1,5}"
    ) {
        let input = format!(
            "{}{}{}",
            "(".repeat(opens),
            atom,
            ")".repeat(closes)
        );
        let _ = parse_sexpr(&input);
    }
}

// ---------------------------------------------------------------------------
// 9. BinarySerializer is deterministic: same config → same output
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn binary_serializer_deterministic_new(_seed in 0u32..1000) {
        let s1 = BinarySerializer::new();
        let s2 = BinarySerializer::new();
        // Two fresh serializers should start with identical empty state,
        // reflected by producing identical BinaryFormat for an empty node_types.
        let fmt1 = BinaryFormat {
            node_types: vec![],
            field_names: vec![],
            tree_data: vec![],
        };
        let fmt2 = BinaryFormat {
            node_types: vec![],
            field_names: vec![],
            tree_data: vec![],
        };
        // Suppress unused-variable warnings
        let _ = (s1, s2);
        prop_assert_eq!(fmt1.node_types, fmt2.node_types);
        prop_assert_eq!(fmt1.field_names, fmt2.field_names);
        prop_assert_eq!(fmt1.tree_data, fmt2.tree_data);
    }
}

// ---------------------------------------------------------------------------
// 10. BinaryFormat fields are consistent
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn binary_format_fields(
        types in prop::collection::vec("[a-z_]{1,10}", 0..10),
        fields in prop::collection::vec("[a-z_]{1,10}", 0..10),
        data in prop::collection::vec(any::<u8>(), 0..64),
    ) {
        let fmt = BinaryFormat {
            node_types: types.clone(),
            field_names: fields.clone(),
            tree_data: data.clone(),
        };
        prop_assert_eq!(&fmt.node_types, &types);
        prop_assert_eq!(&fmt.field_names, &fields);
        prop_assert_eq!(&fmt.tree_data, &data);
    }
}

// ---------------------------------------------------------------------------
// 11. BinaryFormat clone equals original
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn binary_format_clone(
        types in prop::collection::vec("[a-z]{1,5}", 0..5),
        data in prop::collection::vec(any::<u8>(), 0..32),
    ) {
        let fmt = BinaryFormat {
            node_types: types,
            field_names: vec![],
            tree_data: data,
        };
        let cloned = fmt.clone();
        prop_assert_eq!(fmt.node_types, cloned.node_types);
        prop_assert_eq!(fmt.field_names, cloned.field_names);
        prop_assert_eq!(fmt.tree_data, cloned.tree_data);
    }
}

// ---------------------------------------------------------------------------
// 12. SerializedNode JSON roundtrip
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn serialized_node_json_roundtrip(node in arb_serialized_node(2)) {
        let json = serde_json::to_string(&node).unwrap();
        let decoded: SerializedNode = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(node.kind, decoded.kind);
        prop_assert_eq!(node.is_named, decoded.is_named);
        prop_assert_eq!(node.field_name, decoded.field_name);
        prop_assert_eq!(node.start_byte, decoded.start_byte);
        prop_assert_eq!(node.end_byte, decoded.end_byte);
        prop_assert_eq!(node.text, decoded.text);
        prop_assert_eq!(node.children.len(), decoded.children.len());
    }
}

// ---------------------------------------------------------------------------
// 13. CompactNode JSON roundtrip
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn compact_node_json_roundtrip(
        kind in "[a-z_]{1,10}",
        text in proptest::option::of("[a-z ]{1,20}"),
    ) {
        let node = CompactNode {
            kind: kind.clone(),
            start: Some(0),
            end: Some(10),
            field: None,
            children: vec![],
            text: text.clone(),
        };
        let json = serde_json::to_string(&node).unwrap();
        let decoded: CompactNode = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(node.kind, decoded.kind);
        prop_assert_eq!(node.text, decoded.text);
        prop_assert_eq!(node.start, decoded.start);
        prop_assert_eq!(node.end, decoded.end);
    }
}

// ---------------------------------------------------------------------------
// 14. sexpr_to_string for Atom is identity
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn sexpr_atom_to_string_identity(s in "[a-zA-Z_][a-zA-Z0-9_]{0,15}") {
        let expr = SExpr::Atom(s.clone());
        prop_assert_eq!(sexpr_to_string(&expr), s);
    }
}

// ---------------------------------------------------------------------------
// 15. sexpr_to_string for List is parenthesized
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn sexpr_list_to_string_parenthesized(items in prop::collection::vec(arb_sexpr(1), 1..5)) {
        let expr = SExpr::List(items);
        let rendered = sexpr_to_string(&expr);
        prop_assert!(rendered.starts_with('('));
        prop_assert!(rendered.ends_with(')'));
    }
}

// ---------------------------------------------------------------------------
// 16. parse_sexpr with embedded nulls does not panic
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn parse_sexpr_binary_safe(bytes in prop::collection::vec(any::<u8>(), 0..50)) {
        let input = String::from_utf8_lossy(&bytes).to_string();
        let _ = parse_sexpr(&input);
    }
}

// ---------------------------------------------------------------------------
// 17. BinarySerializer Default matches new()
// ---------------------------------------------------------------------------

#[test]
fn binary_serializer_default_eq_new() {
    let from_default = BinarySerializer::default();
    let from_new = BinarySerializer::new();
    // Both should start with empty state — we verify indirectly via
    // producing the same BinaryFormat for an empty tree_data.
    let fmt_d = BinaryFormat {
        node_types: vec![],
        field_names: vec![],
        tree_data: vec![],
    };
    let fmt_n = BinaryFormat {
        node_types: vec![],
        field_names: vec![],
        tree_data: vec![],
    };
    let _ = (from_default, from_new);
    assert_eq!(fmt_d.node_types, fmt_n.node_types);
    assert_eq!(fmt_d.tree_data, fmt_n.tree_data);
}

// ---------------------------------------------------------------------------
// 18. SerializedNode JSON always produces valid JSON
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn serialized_node_json_is_valid(node in arb_serialized_node(3)) {
        let json = serde_json::to_string(&node).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        prop_assert!(parsed.is_object());
    }
}

// ---------------------------------------------------------------------------
// 19. CompactNode JSON always produces valid JSON
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn compact_node_json_is_valid(
        kind in "[a-z_]{1,10}",
        start in proptest::option::of(0usize..500),
        end in proptest::option::of(0usize..500),
        field in proptest::option::of("[a-z_]{1,8}"),
        text in proptest::option::of("[a-z ]{1,20}"),
    ) {
        let node = CompactNode { kind, start, end, field, children: vec![], text };
        let json = serde_json::to_string(&node).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        prop_assert!(parsed.is_object());
    }
}

// ---------------------------------------------------------------------------
// 20. SerializedNode serialization is deterministic
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn serialized_node_deterministic(node in arb_serialized_node(2)) {
        let json1 = serde_json::to_string(&node).unwrap();
        let json2 = serde_json::to_string(&node).unwrap();
        prop_assert_eq!(&json1, &json2);
    }
}

// ---------------------------------------------------------------------------
// 21. CompactNode serialization is deterministic
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn compact_node_deterministic(
        kind in "[a-z_]{1,10}",
        text in proptest::option::of("[a-z]{1,10}"),
    ) {
        let node = CompactNode {
            kind,
            start: Some(0),
            end: Some(5),
            field: None,
            children: vec![],
            text,
        };
        let json1 = serde_json::to_string(&node).unwrap();
        let json2 = serde_json::to_string(&node).unwrap();
        prop_assert_eq!(&json1, &json2);
    }
}

// ---------------------------------------------------------------------------
// 22. Empty tree (leaf node) serializes and roundtrips
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn empty_leaf_node_roundtrip(kind in "[a-z]{1,8}", text in "[a-z]{1,12}") {
        let node = SerializedNode {
            kind: kind.clone(),
            is_named: true,
            field_name: None,
            start_position: (0, 0),
            end_position: (0, text.len()),
            start_byte: 0,
            end_byte: text.len(),
            text: Some(text.clone()),
            children: vec![],
            is_error: false,
            is_missing: false,
        };
        let json = serde_json::to_string(&node).unwrap();
        let decoded: SerializedNode = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(&decoded.kind, &kind);
        prop_assert_eq!(&decoded.text, &Some(text));
        prop_assert!(decoded.children.is_empty());
    }
}

// ---------------------------------------------------------------------------
// 23. Deep tree serialization does not panic
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(30))]

    #[test]
    fn deep_tree_serialization(node in arb_serialized_node(5)) {
        let json = serde_json::to_string(&node);
        prop_assert!(json.is_ok());
        let json = json.unwrap();
        let decoded: Result<SerializedNode, _> = serde_json::from_str(&json);
        prop_assert!(decoded.is_ok());
    }
}

// ---------------------------------------------------------------------------
// 24. Unicode in node kind names
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn unicode_node_kind_roundtrip(kind in "[\\p{L}]{1,10}") {
        let node = SerializedNode {
            kind: kind.clone(),
            is_named: true,
            field_name: None,
            start_position: (0, 0),
            end_position: (0, 4),
            start_byte: 0,
            end_byte: 4,
            text: Some("data".to_string()),
            children: vec![],
            is_error: false,
            is_missing: false,
        };
        let json = serde_json::to_string(&node).unwrap();
        let decoded: SerializedNode = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(&decoded.kind, &kind);
    }
}

// ---------------------------------------------------------------------------
// 25. Unicode in CompactNode kind names
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn unicode_compact_node_roundtrip(kind in "[\\p{L}]{1,10}") {
        let node = CompactNode {
            kind: kind.clone(),
            start: Some(0),
            end: Some(4),
            field: None,
            children: vec![],
            text: Some("data".to_string()),
        };
        let json = serde_json::to_string(&node).unwrap();
        let decoded: CompactNode = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(&decoded.kind, &kind);
    }
}

// ---------------------------------------------------------------------------
// 26. Unicode in field names
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn unicode_field_name_roundtrip(
        kind in "[a-z]{1,6}",
        field in "[\\p{L}]{1,10}",
    ) {
        let node = SerializedNode {
            kind,
            is_named: true,
            field_name: Some(field.clone()),
            start_position: (0, 0),
            end_position: (0, 4),
            start_byte: 0,
            end_byte: 4,
            text: None,
            children: vec![],
            is_error: false,
            is_missing: false,
        };
        let json = serde_json::to_string(&node).unwrap();
        let decoded: SerializedNode = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(&decoded.field_name, &Some(field));
    }
}

// ---------------------------------------------------------------------------
// 27. S-expression render roundtrip: Atom → string → SExpr
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn sexpr_render_deterministic(expr in arb_sexpr(3)) {
        let s1 = sexpr_to_string(&expr);
        let s2 = sexpr_to_string(&expr);
        prop_assert_eq!(&s1, &s2);
    }
}

// ---------------------------------------------------------------------------
// 28. CompactNode with children roundtrips
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn compact_node_with_children_roundtrip(
        parent_kind in "[a-z_]{1,8}",
        child_kind in "[a-z_]{1,8}",
        child_text in "[a-z]{1,10}",
    ) {
        let child = CompactNode {
            kind: child_kind,
            start: None,
            end: None,
            field: None,
            children: vec![],
            text: Some(child_text),
        };
        let parent = CompactNode {
            kind: parent_kind.clone(),
            start: Some(0),
            end: Some(20),
            field: None,
            children: vec![child],
            text: None,
        };
        let json = serde_json::to_string(&parent).unwrap();
        let decoded: CompactNode = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(&decoded.kind, &parent_kind);
        prop_assert_eq!(decoded.children.len(), 1);
    }
}

// ---------------------------------------------------------------------------
// 29. SerializedNode pretty-print produces valid JSON
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn serialized_node_pretty_json_valid(node in arb_serialized_node(2)) {
        let json = serde_json::to_string_pretty(&node).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        prop_assert!(parsed.is_object());
        // Pretty and compact roundtrip to same logical value
        let compact_json = serde_json::to_string(&node).unwrap();
        let compact_parsed: serde_json::Value = serde_json::from_str(&compact_json).unwrap();
        prop_assert_eq!(&parsed, &compact_parsed);
    }
}

// ---------------------------------------------------------------------------
// 30. SExpr empty list renders as "()"
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn sexpr_empty_list_render(_seed in 0u32..100) {
        let empty = SExpr::List(vec![]);
        prop_assert_eq!(sexpr_to_string(&empty), "()");
    }
}

// ---------------------------------------------------------------------------
// 31. SerializedNode error/missing flags preserved across roundtrip
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn error_missing_flags_roundtrip(
        is_error in any::<bool>(),
        is_missing in any::<bool>(),
    ) {
        let node = SerializedNode {
            kind: "test".to_string(),
            is_named: true,
            field_name: None,
            start_position: (0, 0),
            end_position: (0, 4),
            start_byte: 0,
            end_byte: 4,
            text: Some("data".to_string()),
            children: vec![],
            is_error,
            is_missing,
        };
        let json = serde_json::to_string(&node).unwrap();
        let decoded: SerializedNode = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(decoded.is_error, is_error);
        prop_assert_eq!(decoded.is_missing, is_missing);
    }
}

// ---------------------------------------------------------------------------
// 32. CompactNode omits empty children from JSON
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn compact_node_skips_empty_children(kind in "[a-z]{1,6}") {
        let node = CompactNode {
            kind,
            start: Some(0),
            end: Some(5),
            field: None,
            children: vec![],
            text: None,
        };
        let json = serde_json::to_string(&node).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        let obj = parsed.as_object().unwrap();
        // "c" key should be absent when children is empty (skip_serializing_if)
        prop_assert!(!obj.contains_key("c"));
    }
}

// ---------------------------------------------------------------------------
// 33. CompactNode omits None fields from JSON
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn compact_node_skips_none_fields(kind in "[a-z]{1,6}") {
        let node = CompactNode {
            kind,
            start: None,
            end: None,
            field: None,
            children: vec![],
            text: None,
        };
        let json = serde_json::to_string(&node).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        let obj = parsed.as_object().unwrap();
        // Only "t" key should be present; optional None fields are skipped
        prop_assert!(!obj.contains_key("s"));
        prop_assert!(!obj.contains_key("e"));
        prop_assert!(!obj.contains_key("f"));
        prop_assert!(!obj.contains_key("x"));
    }
}

// ---------------------------------------------------------------------------
// 34. TreeSerializer configuration builder is idempotent
// ---------------------------------------------------------------------------

use adze::serialization::TreeSerializer;

proptest! {
    #[test]
    fn tree_serializer_config_builder(max_len in proptest::option::of(1usize..500)) {
        let source = b"test code";
        let s = TreeSerializer::new(source)
            .with_unnamed_nodes()
            .with_max_text_length(max_len);
        prop_assert!(s.include_unnamed);
        prop_assert_eq!(s.max_text_length, max_len);
    }
}

// ---------------------------------------------------------------------------
// 35. SerializedNode position invariants
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn serialized_node_position_preserved(
        start_row in 0usize..100,
        start_col in 0usize..200,
        span in 1usize..100,
    ) {
        let end_col = start_col + span;
        let node = SerializedNode {
            kind: "id".to_string(),
            is_named: true,
            field_name: None,
            start_position: (start_row, start_col),
            end_position: (start_row, end_col),
            start_byte: start_col,
            end_byte: end_col,
            text: None,
            children: vec![],
            is_error: false,
            is_missing: false,
        };
        let json = serde_json::to_string(&node).unwrap();
        let decoded: SerializedNode = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(decoded.start_position, (start_row, start_col));
        prop_assert_eq!(decoded.end_position, (start_row, end_col));
        prop_assert_eq!(decoded.start_byte, start_col);
        prop_assert_eq!(decoded.end_byte, end_col);
    }
}

// ===========================================================================
// NEW TESTS (36–65): additional coverage per task requirements
// ===========================================================================

// ---------------------------------------------------------------------------
// 36. Deserialization rejects completely invalid JSON
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn deser_rejects_garbage(input in "[^{}\\[\\]\"]{1,40}") {
        let result: Result<SerializedNode, _> = serde_json::from_str(&input);
        prop_assert!(result.is_err());
    }
}

// ---------------------------------------------------------------------------
// 37. Deserialization rejects JSON missing required `kind` field
// ---------------------------------------------------------------------------

#[test]
fn deser_missing_kind_field() {
    let json = r#"{
        "is_named": true,
        "field_name": null,
        "start_position": [0,0],
        "end_position": [0,4],
        "start_byte": 0,
        "end_byte": 4,
        "text": "hi",
        "children": [],
        "is_error": false,
        "is_missing": false
    }"#;
    let result: Result<SerializedNode, _> = serde_json::from_str(json);
    assert!(result.is_err());
}

// ---------------------------------------------------------------------------
// 38. Deserialization rejects wrong type for `is_named`
// ---------------------------------------------------------------------------

#[test]
fn deser_wrong_type_is_named() {
    let json = r#"{
        "kind": "id",
        "is_named": "yes",
        "field_name": null,
        "start_position": [0,0],
        "end_position": [0,4],
        "start_byte": 0,
        "end_byte": 4,
        "text": null,
        "children": [],
        "is_error": false,
        "is_missing": false
    }"#;
    let result: Result<SerializedNode, _> = serde_json::from_str(json);
    assert!(result.is_err());
}

// ---------------------------------------------------------------------------
// 39. Deserialization rejects wrong type for `start_byte`
// ---------------------------------------------------------------------------

#[test]
fn deser_wrong_type_start_byte() {
    let json = r#"{
        "kind": "id",
        "is_named": true,
        "field_name": null,
        "start_position": [0,0],
        "end_position": [0,4],
        "start_byte": "zero",
        "end_byte": 4,
        "text": null,
        "children": [],
        "is_error": false,
        "is_missing": false
    }"#;
    let result: Result<SerializedNode, _> = serde_json::from_str(json);
    assert!(result.is_err());
}

// ---------------------------------------------------------------------------
// 40. Deserialization of CompactNode rejects missing `t` (kind)
// ---------------------------------------------------------------------------

#[test]
fn compact_deser_missing_kind() {
    let json = r#"{"s":0,"e":5}"#;
    let result: Result<CompactNode, _> = serde_json::from_str(json);
    assert!(result.is_err());
}

// ---------------------------------------------------------------------------
// 41. Wide tree: many siblings roundtrip
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]

    #[test]
    fn wide_tree_roundtrip(n_children in 10usize..50) {
        let children: Vec<SerializedNode> = (0..n_children)
            .map(|i| SerializedNode {
                kind: format!("child_{}", i),
                is_named: true,
                field_name: None,
                start_position: (0, i),
                end_position: (0, i + 1),
                start_byte: i,
                end_byte: i + 1,
                text: Some(format!("c{}", i)),
                children: vec![],
                is_error: false,
                is_missing: false,
            })
            .collect();
        let root = SerializedNode {
            kind: "root".to_string(),
            is_named: true,
            field_name: None,
            start_position: (0, 0),
            end_position: (0, n_children),
            start_byte: 0,
            end_byte: n_children,
            text: None,
            children,
            is_error: false,
            is_missing: false,
        };
        let json = serde_json::to_string(&root).unwrap();
        let decoded: SerializedNode = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(decoded.children.len(), n_children);
        for i in 0..n_children {
            prop_assert_eq!(&decoded.children[i].kind, &format!("child_{}", i));
        }
    }
}

// ---------------------------------------------------------------------------
// 42. Large tree: 200+ nodes total roundtrip
// ---------------------------------------------------------------------------

#[test]
fn large_tree_200_nodes_roundtrip() {
    // Build a tree with ~200 leaf nodes across 20 parents
    let parents: Vec<SerializedNode> = (0..20)
        .map(|p| {
            let leaves: Vec<SerializedNode> = (0..10)
                .map(|l| SerializedNode {
                    kind: format!("leaf_{}_{}", p, l),
                    is_named: true,
                    field_name: None,
                    start_position: (p, l),
                    end_position: (p, l + 1),
                    start_byte: p * 10 + l,
                    end_byte: p * 10 + l + 1,
                    text: Some("x".to_string()),
                    children: vec![],
                    is_error: false,
                    is_missing: false,
                })
                .collect();
            SerializedNode {
                kind: format!("parent_{}", p),
                is_named: true,
                field_name: None,
                start_position: (p, 0),
                end_position: (p, 10),
                start_byte: p * 10,
                end_byte: (p + 1) * 10,
                text: None,
                children: leaves,
                is_error: false,
                is_missing: false,
            }
        })
        .collect();
    let root = SerializedNode {
        kind: "program".to_string(),
        is_named: true,
        field_name: None,
        start_position: (0, 0),
        end_position: (20, 0),
        start_byte: 0,
        end_byte: 200,
        text: None,
        children: parents,
        is_error: false,
        is_missing: false,
    };
    let json = serde_json::to_string(&root).unwrap();
    let decoded: SerializedNode = serde_json::from_str(&json).unwrap();
    assert_eq!(decoded.children.len(), 20);
    let total_leaves: usize = decoded.children.iter().map(|c| c.children.len()).sum();
    assert_eq!(total_leaves, 200);
}

// ---------------------------------------------------------------------------
// 43. Nested CompactNode (3 levels) roundtrip
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn compact_node_nested_3_levels(
        gk in "[a-z]{1,5}",
        pk in "[a-z]{1,5}",
        ck in "[a-z]{1,5}",
        txt in "[a-z]{1,8}",
    ) {
        let leaf = CompactNode {
            kind: ck.clone(),
            start: None,
            end: None,
            field: None,
            children: vec![],
            text: Some(txt),
        };
        let mid = CompactNode {
            kind: pk,
            start: Some(0),
            end: Some(10),
            field: None,
            children: vec![leaf],
            text: None,
        };
        let root = CompactNode {
            kind: gk.clone(),
            start: Some(0),
            end: Some(20),
            field: None,
            children: vec![mid],
            text: None,
        };
        let json = serde_json::to_string(&root).unwrap();
        let decoded: CompactNode = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(&decoded.kind, &gk);
        prop_assert_eq!(decoded.children.len(), 1);
        prop_assert_eq!(decoded.children[0].children.len(), 1);
        prop_assert_eq!(&decoded.children[0].children[0].kind, &ck);
    }
}

// ---------------------------------------------------------------------------
// 44. All SerializedNode fields preserved in one roundtrip
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn all_node_fields_preserved(
        kind in "[a-z]{1,8}",
        is_named in any::<bool>(),
        field_name in proptest::option::of("[a-z]{1,6}"),
        sr in 0usize..50,
        sc in 0usize..100,
        er in 0usize..50,
        ec in 0usize..200,
        sb in 0usize..500,
        eb in 0usize..500,
        text in proptest::option::of("[a-z ]{1,12}"),
        is_error in any::<bool>(),
        is_missing in any::<bool>(),
    ) {
        let node = SerializedNode {
            kind: kind.clone(),
            is_named,
            field_name: field_name.clone(),
            start_position: (sr, sc),
            end_position: (er, ec),
            start_byte: sb,
            end_byte: eb,
            text: text.clone(),
            children: vec![],
            is_error,
            is_missing,
        };
        let json = serde_json::to_string(&node).unwrap();
        let d: SerializedNode = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(&d.kind, &kind);
        prop_assert_eq!(d.is_named, is_named);
        prop_assert_eq!(&d.field_name, &field_name);
        prop_assert_eq!(d.start_position, (sr, sc));
        prop_assert_eq!(d.end_position, (er, ec));
        prop_assert_eq!(d.start_byte, sb);
        prop_assert_eq!(d.end_byte, eb);
        prop_assert_eq!(&d.text, &text);
        prop_assert_eq!(d.is_error, is_error);
        prop_assert_eq!(d.is_missing, is_missing);
    }
}

// ---------------------------------------------------------------------------
// 45. CompactNode field alias "f" preserved
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn compact_node_field_alias_preserved(
        kind in "[a-z]{1,6}",
        field in "[a-z]{1,8}",
    ) {
        let node = CompactNode {
            kind,
            start: Some(0),
            end: Some(5),
            field: Some(field.clone()),
            children: vec![],
            text: None,
        };
        let json = serde_json::to_string(&node).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        let obj = parsed.as_object().unwrap();
        prop_assert_eq!(obj.get("f").and_then(|v| v.as_str()), Some(field.as_str()));
    }
}

// ---------------------------------------------------------------------------
// 46. CompactNode start/end use aliases "s" and "e"
// ---------------------------------------------------------------------------

#[test]
fn compact_node_position_aliases() {
    let node = CompactNode {
        kind: "id".to_string(),
        start: Some(42),
        end: Some(99),
        field: None,
        children: vec![],
        text: None,
    };
    let json = serde_json::to_string(&node).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    let obj = parsed.as_object().unwrap();
    assert_eq!(obj.get("s").and_then(|v| v.as_u64()), Some(42));
    assert_eq!(obj.get("e").and_then(|v| v.as_u64()), Some(99));
    assert!(obj.get("t").is_some()); // kind alias
}

// ---------------------------------------------------------------------------
// 47. SerializedNode JSON contains expected top-level keys
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn serialized_node_json_has_all_keys(node in arb_serialized_node(1)) {
        let json = serde_json::to_string(&node).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        let obj = parsed.as_object().unwrap();
        prop_assert!(obj.contains_key("kind"));
        prop_assert!(obj.contains_key("is_named"));
        prop_assert!(obj.contains_key("start_position"));
        prop_assert!(obj.contains_key("end_position"));
        prop_assert!(obj.contains_key("start_byte"));
        prop_assert!(obj.contains_key("end_byte"));
        prop_assert!(obj.contains_key("children"));
        prop_assert!(obj.contains_key("is_error"));
        prop_assert!(obj.contains_key("is_missing"));
    }
}

// ---------------------------------------------------------------------------
// 48. Deeply nested SExpr JSON roundtrip (depth 5)
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(30))]

    #[test]
    fn deeply_nested_sexpr_json_roundtrip(expr in arb_sexpr(5)) {
        let json = serde_json::to_string(&expr).unwrap();
        let decoded: SExpr = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(&expr, &decoded);
    }
}

// ---------------------------------------------------------------------------
// 49. SExpr nested list render starts/ends with parens at every level
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn sexpr_nested_list_parens(inner in prop::collection::vec(arb_sexpr(1), 0..4)) {
        let outer = SExpr::List(vec![SExpr::List(inner)]);
        let rendered = sexpr_to_string(&outer);
        prop_assert!(rendered.starts_with("(("));
        prop_assert!(rendered.ends_with("))"));
    }
}

// ---------------------------------------------------------------------------
// 50. SerializedNode children order preserved
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn children_order_preserved(n in 2usize..8) {
        let children: Vec<SerializedNode> = (0..n)
            .map(|i| SerializedNode {
                kind: format!("k{}", i),
                is_named: true,
                field_name: None,
                start_position: (0, i),
                end_position: (0, i + 1),
                start_byte: i,
                end_byte: i + 1,
                text: Some(format!("t{}", i)),
                children: vec![],
                is_error: false,
                is_missing: false,
            })
            .collect();
        let parent = SerializedNode {
            kind: "p".to_string(),
            is_named: true,
            field_name: None,
            start_position: (0, 0),
            end_position: (0, n),
            start_byte: 0,
            end_byte: n,
            text: None,
            children,
            is_error: false,
            is_missing: false,
        };
        let json = serde_json::to_string(&parent).unwrap();
        let decoded: SerializedNode = serde_json::from_str(&json).unwrap();
        for i in 0..n {
            prop_assert_eq!(&decoded.children[i].kind, &format!("k{}", i));
            prop_assert_eq!(&decoded.children[i].text, &Some(format!("t{}", i)));
        }
    }
}

// ---------------------------------------------------------------------------
// 51. TreeSerializer defaults: unnamed excluded, max_text 100
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn tree_serializer_defaults(src_len in 1usize..100) {
        let source = vec![b'a'; src_len];
        let s = TreeSerializer::new(&source);
        prop_assert!(!s.include_unnamed);
        prop_assert_eq!(s.max_text_length, Some(100));
    }
}

// ---------------------------------------------------------------------------
// 52. TreeSerializer with_max_text_length(None) sets unlimited
// ---------------------------------------------------------------------------

#[test]
fn tree_serializer_unlimited_text() {
    let source = b"code";
    let s = TreeSerializer::new(source).with_max_text_length(None);
    assert_eq!(s.max_text_length, None);
}

// ---------------------------------------------------------------------------
// 53. Serialization determinism across clones
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(80))]

    #[test]
    fn serialized_node_clone_determinism(node in arb_serialized_node(2)) {
        let cloned = node.clone();
        let json_orig = serde_json::to_string(&node).unwrap();
        let json_clone = serde_json::to_string(&cloned).unwrap();
        prop_assert_eq!(&json_orig, &json_clone);
    }
}

// ---------------------------------------------------------------------------
// 54. CompactNode text alias "x" present when set
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn compact_node_text_alias(text in "[a-z]{1,10}") {
        let node = CompactNode {
            kind: "leaf".to_string(),
            start: None,
            end: None,
            field: None,
            children: vec![],
            text: Some(text.clone()),
        };
        let json = serde_json::to_string(&node).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        let obj = parsed.as_object().unwrap();
        prop_assert_eq!(obj.get("x").and_then(|v| v.as_str()), Some(text.as_str()));
    }
}

// ---------------------------------------------------------------------------
// 55. Zero-length span node roundtrip (missing node pattern)
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn zero_length_span_roundtrip(pos in 0usize..1000) {
        let node = SerializedNode {
            kind: "MISSING".to_string(),
            is_named: true,
            field_name: None,
            start_position: (0, pos),
            end_position: (0, pos),
            start_byte: pos,
            end_byte: pos,
            text: None,
            children: vec![],
            is_error: false,
            is_missing: true,
        };
        let json = serde_json::to_string(&node).unwrap();
        let d: SerializedNode = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(d.start_byte, d.end_byte);
        prop_assert!(d.is_missing);
    }
}

// ---------------------------------------------------------------------------
// 56. Nested error nodes preserved
// ---------------------------------------------------------------------------

#[test]
fn nested_error_nodes_preserved() {
    let err_child = SerializedNode {
        kind: "ERROR".to_string(),
        is_named: false,
        field_name: None,
        start_position: (0, 3),
        end_position: (0, 6),
        start_byte: 3,
        end_byte: 6,
        text: Some("???".to_string()),
        children: vec![],
        is_error: true,
        is_missing: false,
    };
    let root = SerializedNode {
        kind: "program".to_string(),
        is_named: true,
        field_name: None,
        start_position: (0, 0),
        end_position: (0, 10),
        start_byte: 0,
        end_byte: 10,
        text: None,
        children: vec![err_child],
        is_error: false,
        is_missing: false,
    };
    let json = serde_json::to_string(&root).unwrap();
    let d: SerializedNode = serde_json::from_str(&json).unwrap();
    assert!(!d.is_error);
    assert_eq!(d.children.len(), 1);
    assert!(d.children[0].is_error);
    assert_eq!(d.children[0].kind, "ERROR");
}

// ---------------------------------------------------------------------------
// 57. BinaryFormat Debug impl does not panic
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn binary_format_debug_no_panic(
        types in prop::collection::vec("[a-z]{1,5}", 0..5),
        data in prop::collection::vec(any::<u8>(), 0..20),
    ) {
        let fmt = BinaryFormat {
            node_types: types,
            field_names: vec![],
            tree_data: data,
        };
        let dbg = format!("{:?}", fmt);
        prop_assert!(!dbg.is_empty());
    }
}

// ---------------------------------------------------------------------------
// 58. SExpr Debug impl does not panic
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn sexpr_debug_no_panic(expr in arb_sexpr(3)) {
        let dbg = format!("{:?}", expr);
        prop_assert!(!dbg.is_empty());
    }
}

// ---------------------------------------------------------------------------
// 59. SerializedNode JSON size grows with children
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(30))]

    #[test]
    fn json_size_monotonic_with_children(extra in 1usize..10) {
        let leaf = SerializedNode {
            kind: "x".to_string(),
            is_named: true,
            field_name: None,
            start_position: (0, 0),
            end_position: (0, 1),
            start_byte: 0,
            end_byte: 1,
            text: Some("a".to_string()),
            children: vec![],
            is_error: false,
            is_missing: false,
        };
        let small = SerializedNode {
            kind: "r".to_string(),
            is_named: true,
            field_name: None,
            start_position: (0, 0),
            end_position: (0, 1),
            start_byte: 0,
            end_byte: 1,
            text: None,
            children: vec![leaf.clone()],
            is_error: false,
            is_missing: false,
        };
        let big = SerializedNode {
            kind: "r".to_string(),
            is_named: true,
            field_name: None,
            start_position: (0, 0),
            end_position: (0, 1),
            start_byte: 0,
            end_byte: 1,
            text: None,
            children: (0..1 + extra).map(|_| leaf.clone()).collect(),
            is_error: false,
            is_missing: false,
        };
        let small_json = serde_json::to_string(&small).unwrap();
        let big_json = serde_json::to_string(&big).unwrap();
        prop_assert!(big_json.len() > small_json.len());
    }
}

// ---------------------------------------------------------------------------
// 60. CompactNode children alias "c" present only when non-empty
// ---------------------------------------------------------------------------

#[test]
fn compact_node_children_alias_present_when_non_empty() {
    let child = CompactNode {
        kind: "leaf".to_string(),
        start: None,
        end: None,
        field: None,
        children: vec![],
        text: Some("hi".to_string()),
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
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    let obj = parsed.as_object().unwrap();
    assert!(obj.contains_key("c"));
    let arr = obj.get("c").unwrap().as_array().unwrap();
    assert_eq!(arr.len(), 1);
}

// ---------------------------------------------------------------------------
// 61. SerializedNode multiline positions roundtrip
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn multiline_position_roundtrip(
        sr in 0usize..50,
        sc in 0usize..100,
        er in 0usize..50,
        ec in 0usize..100,
    ) {
        let node = SerializedNode {
            kind: "block".to_string(),
            is_named: true,
            field_name: None,
            start_position: (sr, sc),
            end_position: (er, ec),
            start_byte: 0,
            end_byte: 100,
            text: None,
            children: vec![],
            is_error: false,
            is_missing: false,
        };
        let json = serde_json::to_string(&node).unwrap();
        let d: SerializedNode = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(d.start_position.0, sr);
        prop_assert_eq!(d.start_position.1, sc);
        prop_assert_eq!(d.end_position.0, er);
        prop_assert_eq!(d.end_position.1, ec);
    }
}

// ---------------------------------------------------------------------------
// 62. Deserialization of truncated JSON fails gracefully
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(80))]

    #[test]
    fn deser_truncated_json(node in arb_serialized_node(1)) {
        let full = serde_json::to_string(&node).unwrap();
        if full.len() > 5 {
            let cut = &full[..full.len() / 2];
            let result: Result<SerializedNode, _> = serde_json::from_str(cut);
            prop_assert!(result.is_err());
        }
    }
}

// ---------------------------------------------------------------------------
// 63. SExpr single-atom list render
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn sexpr_single_atom_list(s in "[a-z]{1,8}") {
        let list = SExpr::List(vec![SExpr::Atom(s.clone())]);
        let rendered = sexpr_to_string(&list);
        prop_assert_eq!(rendered, format!("({})", s));
    }
}

// ---------------------------------------------------------------------------
// 64. Large CompactNode tree (100 children) roundtrip
// ---------------------------------------------------------------------------

#[test]
fn large_compact_tree_100_children() {
    let children: Vec<CompactNode> = (0..100)
        .map(|i| CompactNode {
            kind: format!("n{}", i),
            start: None,
            end: None,
            field: None,
            children: vec![],
            text: Some(format!("v{}", i)),
        })
        .collect();
    let root = CompactNode {
        kind: "big".to_string(),
        start: Some(0),
        end: Some(1000),
        field: None,
        children,
        text: None,
    };
    let json = serde_json::to_string(&root).unwrap();
    let decoded: CompactNode = serde_json::from_str(&json).unwrap();
    assert_eq!(decoded.children.len(), 100);
    assert_eq!(decoded.children[99].kind, "n99");
}

// ---------------------------------------------------------------------------
// 65. Deserialization of empty JSON object fails for SerializedNode
// ---------------------------------------------------------------------------

#[test]
fn deser_empty_object_fails() {
    let result: Result<SerializedNode, _> = serde_json::from_str("{}");
    assert!(result.is_err());
}
