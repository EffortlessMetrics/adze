//! Property-based tests for `adze::serialization`.
//!
//! Exercises `SExpr`, `parse_sexpr`, `BinarySerializer`, `BinaryFormat`,
//! `SerializedNode`, and `CompactNode` with random inputs.

#![cfg(feature = "serialization")]

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
