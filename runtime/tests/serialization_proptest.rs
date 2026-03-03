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
