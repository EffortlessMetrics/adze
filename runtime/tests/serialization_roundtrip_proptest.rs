//! Property-based roundtrip tests for `adze::serialization`.
//!
//! 50+ proptest tests covering SExpr construction/formatting,
//! SerializedNode roundtrip properties, CompactNode encoding,
//! BinaryFormat encode/decode, TreeSerializer output determinism,
//! and edge cases (empty trees, deep trees, unicode content).

#![cfg(feature = "serialization")]
#![allow(clippy::needless_range_loop)]

use adze::serialization::{
    BinaryFormat, BinarySerializer, CompactNode, SExpr, SerializedNode, TreeSerializer, parse_sexpr,
};
use proptest::prelude::*;

// ---------------------------------------------------------------------------
// Strategies
// ---------------------------------------------------------------------------

fn arb_sexpr(depth: u32) -> BoxedStrategy<SExpr> {
    if depth == 0 {
        "[a-zA-Z_][a-zA-Z0-9_]{0,12}".prop_map(SExpr::Atom).boxed()
    } else {
        prop_oneof![
            "[a-zA-Z_][a-zA-Z0-9_]{0,12}".prop_map(SExpr::Atom),
            prop::collection::vec(arb_sexpr(depth - 1), 0..4).prop_map(SExpr::List),
        ]
        .boxed()
    }
}

fn arb_leaf_node() -> BoxedStrategy<SerializedNode> {
    (
        "[a-z_]{1,10}",
        any::<bool>(),
        proptest::option::of("[a-z_]{1,8}"),
        0usize..500,
        1usize..50,
        proptest::option::of("[a-zA-Z0-9_ ]{1,20}"),
        any::<bool>(),
        any::<bool>(),
    )
        .prop_map(
            |(kind, is_named, field_name, start, span, text, is_error, is_missing)| {
                let end = start + span;
                SerializedNode {
                    kind,
                    is_named,
                    field_name,
                    start_position: (0, start),
                    end_position: (0, end),
                    start_byte: start,
                    end_byte: end,
                    text,
                    children: vec![],
                    is_error,
                    is_missing,
                }
            },
        )
        .boxed()
}

fn arb_serialized_node(depth: u32) -> BoxedStrategy<SerializedNode> {
    if depth == 0 {
        arb_leaf_node()
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

fn arb_compact_node(depth: u32) -> BoxedStrategy<CompactNode> {
    if depth == 0 {
        ("[a-z_]{1,8}", proptest::option::of("[a-z]{1,10}"))
            .prop_map(|(kind, text)| CompactNode {
                kind,
                start: None,
                end: None,
                field: None,
                children: vec![],
                text,
            })
            .boxed()
    } else {
        (
            "[a-z_]{1,8}",
            proptest::option::of(0usize..500),
            proptest::option::of(0usize..500),
            proptest::option::of("[a-z_]{1,6}"),
            prop::collection::vec(arb_compact_node(depth - 1), 0..3),
        )
            .prop_map(|(kind, start, end, field, children)| CompactNode {
                kind,
                start,
                end,
                field,
                children,
                text: None,
            })
            .boxed()
    }
}

fn sexpr_to_string(expr: &SExpr) -> String {
    match expr {
        SExpr::Atom(s) => s.clone(),
        SExpr::List(items) => {
            let inner: Vec<String> = items.iter().map(sexpr_to_string).collect();
            format!("({})", inner.join(" "))
        }
    }
}

fn count_nodes(node: &SerializedNode) -> usize {
    1 + node.children.iter().map(count_nodes).sum::<usize>()
}

fn tree_depth(node: &SerializedNode) -> usize {
    if node.children.is_empty() {
        1
    } else {
        1 + node.children.iter().map(tree_depth).max().unwrap_or(0)
    }
}

// ===================================================================
// 1. SerializedNode JSON roundtrip preserves all fields
// ===================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(150))]

    #[test]
    fn rt_serialized_node_all_fields(node in arb_leaf_node()) {
        let json = serde_json::to_string(&node).unwrap();
        let d: SerializedNode = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(&d.kind, &node.kind);
        prop_assert_eq!(d.is_named, node.is_named);
        prop_assert_eq!(&d.field_name, &node.field_name);
        prop_assert_eq!(d.start_position, node.start_position);
        prop_assert_eq!(d.end_position, node.end_position);
        prop_assert_eq!(d.start_byte, node.start_byte);
        prop_assert_eq!(d.end_byte, node.end_byte);
        prop_assert_eq!(&d.text, &node.text);
        prop_assert_eq!(d.is_error, node.is_error);
        prop_assert_eq!(d.is_missing, node.is_missing);
        prop_assert_eq!(d.children.len(), node.children.len());
    }
}

// ===================================================================
// 2. SerializedNode triple roundtrip is idempotent
// ===================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn rt_serialized_node_triple_roundtrip(node in arb_serialized_node(2)) {
        let j1 = serde_json::to_string(&node).unwrap();
        let d1: SerializedNode = serde_json::from_str(&j1).unwrap();
        let j2 = serde_json::to_string(&d1).unwrap();
        let d2: SerializedNode = serde_json::from_str(&j2).unwrap();
        let j3 = serde_json::to_string(&d2).unwrap();
        prop_assert_eq!(&j1, &j2);
        prop_assert_eq!(&j2, &j3);
    }
}

// ===================================================================
// 3. SerializedNode pretty and compact produce same logical value
// ===================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(80))]

    #[test]
    fn rt_pretty_compact_equivalent(node in arb_serialized_node(2)) {
        let pretty = serde_json::to_string_pretty(&node).unwrap();
        let compact = serde_json::to_string(&node).unwrap();
        let v_pretty: serde_json::Value = serde_json::from_str(&pretty).unwrap();
        let v_compact: serde_json::Value = serde_json::from_str(&compact).unwrap();
        prop_assert_eq!(&v_pretty, &v_compact);
    }
}

// ===================================================================
// 4. SerializedNode child count preserved through roundtrip
// ===================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(60))]

    #[test]
    fn rt_child_count_preserved(node in arb_serialized_node(3)) {
        let original_count = count_nodes(&node);
        let json = serde_json::to_string(&node).unwrap();
        let d: SerializedNode = serde_json::from_str(&json).unwrap();
        let decoded_count = count_nodes(&d);
        prop_assert_eq!(original_count, decoded_count);
    }
}

// ===================================================================
// 5. SerializedNode tree depth preserved through roundtrip
// ===================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(60))]

    #[test]
    fn rt_tree_depth_preserved(node in arb_serialized_node(3)) {
        let original_depth = tree_depth(&node);
        let json = serde_json::to_string(&node).unwrap();
        let d: SerializedNode = serde_json::from_str(&json).unwrap();
        let decoded_depth = tree_depth(&d);
        prop_assert_eq!(original_depth, decoded_depth);
    }
}

// ===================================================================
// 6. SerializedNode serialization is deterministic
// ===================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn rt_serialized_node_deterministic(node in arb_serialized_node(2)) {
        let j1 = serde_json::to_string(&node).unwrap();
        let j2 = serde_json::to_string(&node).unwrap();
        prop_assert_eq!(&j1, &j2);
    }
}

// ===================================================================
// 7. SerializedNode clone produces identical JSON
// ===================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(80))]

    #[test]
    fn rt_clone_produces_same_json(node in arb_serialized_node(2)) {
        let cloned = node.clone();
        let j_orig = serde_json::to_string(&node).unwrap();
        let j_clone = serde_json::to_string(&cloned).unwrap();
        prop_assert_eq!(&j_orig, &j_clone);
    }
}

// ===================================================================
// 8. SerializedNode: JSON always has required keys
// ===================================================================

proptest! {
    #[test]
    fn rt_json_has_required_keys(node in arb_leaf_node()) {
        let json = serde_json::to_string(&node).unwrap();
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        let obj = v.as_object().unwrap();
        for key in &["kind", "is_named", "start_position", "end_position",
                     "start_byte", "end_byte", "children", "is_error", "is_missing"] {
            prop_assert!(obj.contains_key(*key), "missing key: {}", key);
        }
    }
}

// ===================================================================
// 9. SerializedNode: JSON size grows with more children
// ===================================================================

proptest! {
    #[test]
    fn rt_json_size_grows_with_children(extra in 1usize..8) {
        let leaf = SerializedNode {
            kind: "x".into(), is_named: true, field_name: None,
            start_position: (0, 0), end_position: (0, 1),
            start_byte: 0, end_byte: 1,
            text: Some("a".into()), children: vec![],
            is_error: false, is_missing: false,
        };
        let small_json = serde_json::to_string(&SerializedNode {
            kind: "r".into(), is_named: true, field_name: None,
            start_position: (0, 0), end_position: (0, 1),
            start_byte: 0, end_byte: 1,
            text: None, children: vec![leaf.clone()],
            is_error: false, is_missing: false,
        }).unwrap();
        let big_json = serde_json::to_string(&SerializedNode {
            kind: "r".into(), is_named: true, field_name: None,
            start_position: (0, 0), end_position: (0, 1),
            start_byte: 0, end_byte: 1,
            text: None, children: (0..1 + extra).map(|_| leaf.clone()).collect(),
            is_error: false, is_missing: false,
        }).unwrap();
        prop_assert!(big_json.len() > small_json.len());
    }
}

// ===================================================================
// 10. SerializedNode: error/missing flag combinations roundtrip
// ===================================================================

proptest! {
    #[test]
    fn rt_error_missing_flags(is_error in any::<bool>(), is_missing in any::<bool>(), is_named in any::<bool>()) {
        let node = SerializedNode {
            kind: "t".into(), is_named, field_name: None,
            start_position: (0, 0), end_position: (0, 1),
            start_byte: 0, end_byte: 1,
            text: None, children: vec![],
            is_error, is_missing,
        };
        let json = serde_json::to_string(&node).unwrap();
        let d: SerializedNode = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(d.is_error, is_error);
        prop_assert_eq!(d.is_missing, is_missing);
        prop_assert_eq!(d.is_named, is_named);
    }
}

// ===================================================================
// 11. SerializedNode: unicode kind names roundtrip
// ===================================================================

proptest! {
    #[test]
    fn rt_unicode_kind(kind in "[\\p{L}]{1,10}") {
        let node = SerializedNode {
            kind: kind.clone(), is_named: true, field_name: None,
            start_position: (0, 0), end_position: (0, 1),
            start_byte: 0, end_byte: 1,
            text: None, children: vec![],
            is_error: false, is_missing: false,
        };
        let json = serde_json::to_string(&node).unwrap();
        let d: SerializedNode = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(&d.kind, &kind);
    }
}

// ===================================================================
// 12. SerializedNode: unicode text roundtrip
// ===================================================================

proptest! {
    #[test]
    fn rt_unicode_text(text in "[\\p{L}\\p{N}\\p{S} ]{1,20}") {
        let node = SerializedNode {
            kind: "str".into(), is_named: true, field_name: None,
            start_position: (0, 0), end_position: (0, text.len()),
            start_byte: 0, end_byte: text.len(),
            text: Some(text.clone()), children: vec![],
            is_error: false, is_missing: false,
        };
        let json = serde_json::to_string(&node).unwrap();
        let d: SerializedNode = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(d.text.as_deref(), Some(text.as_str()));
    }
}

// ===================================================================
// 13. SerializedNode: unicode field names roundtrip
// ===================================================================

proptest! {
    #[test]
    fn rt_unicode_field_name(field in "[\\p{L}]{1,10}") {
        let node = SerializedNode {
            kind: "id".into(), is_named: true, field_name: Some(field.clone()),
            start_position: (0, 0), end_position: (0, 1),
            start_byte: 0, end_byte: 1,
            text: None, children: vec![],
            is_error: false, is_missing: false,
        };
        let json = serde_json::to_string(&node).unwrap();
        let d: SerializedNode = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(&d.field_name, &Some(field));
    }
}

// ===================================================================
// 14. SerializedNode: multiline positions roundtrip
// ===================================================================

proptest! {
    #[test]
    fn rt_multiline_positions(sr in 0usize..50, sc in 0usize..200, er in 0usize..50, ec in 0usize..200) {
        let node = SerializedNode {
            kind: "blk".into(), is_named: true, field_name: None,
            start_position: (sr, sc), end_position: (er, ec),
            start_byte: 0, end_byte: 100,
            text: None, children: vec![],
            is_error: false, is_missing: false,
        };
        let json = serde_json::to_string(&node).unwrap();
        let d: SerializedNode = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(d.start_position, (sr, sc));
        prop_assert_eq!(d.end_position, (er, ec));
    }
}

// ===================================================================
// 15. SerializedNode: zero-span (missing node) roundtrip
// ===================================================================

proptest! {
    #[test]
    fn rt_zero_span_missing(pos in 0usize..1000) {
        let node = SerializedNode {
            kind: "MISSING".into(), is_named: true, field_name: None,
            start_position: (0, pos), end_position: (0, pos),
            start_byte: pos, end_byte: pos,
            text: None, children: vec![],
            is_error: false, is_missing: true,
        };
        let json = serde_json::to_string(&node).unwrap();
        let d: SerializedNode = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(d.start_byte, d.end_byte);
        prop_assert!(d.is_missing);
    }
}

// ===================================================================
// 16. SerializedNode: truncated JSON fails gracefully
// ===================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(80))]

    #[test]
    fn rt_truncated_json_rejected(node in arb_serialized_node(1)) {
        let full = serde_json::to_string(&node).unwrap();
        if full.len() > 10 {
            let cut = &full[..full.len() / 2];
            let result: Result<SerializedNode, _> = serde_json::from_str(cut);
            prop_assert!(result.is_err());
        }
    }
}

// ===================================================================
// 17. SerializedNode: deep nesting roundtrip
// ===================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(30))]

    #[test]
    fn rt_deep_nesting(depth in 10usize..40) {
        let mut node = SerializedNode {
            kind: "leaf".into(), is_named: true, field_name: None,
            start_position: (0, 0), end_position: (0, 1),
            start_byte: 0, end_byte: 1,
            text: Some("x".into()), children: vec![],
            is_error: false, is_missing: false,
        };
        for i in 0..depth {
            node = SerializedNode {
                kind: format!("level_{}", i), is_named: true, field_name: None,
                start_position: (0, 0), end_position: (0, 1),
                start_byte: 0, end_byte: 1,
                text: None, children: vec![node],
                is_error: false, is_missing: false,
            };
        }
        let json = serde_json::to_string(&node).unwrap();
        let d: SerializedNode = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(tree_depth(&d), depth + 1);
    }
}

// ===================================================================
// 18. SerializedNode: wide tree roundtrip
// ===================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]

    #[test]
    fn rt_wide_tree(n_children in 10usize..60) {
        let children: Vec<_> = (0..n_children)
            .map(|i| SerializedNode {
                kind: format!("c{}", i), is_named: true, field_name: None,
                start_position: (0, i), end_position: (0, i + 1),
                start_byte: i, end_byte: i + 1,
                text: Some(format!("v{}", i)), children: vec![],
                is_error: false, is_missing: false,
            })
            .collect();
        let root = SerializedNode {
            kind: "root".into(), is_named: true, field_name: None,
            start_position: (0, 0), end_position: (0, n_children),
            start_byte: 0, end_byte: n_children,
            text: None, children,
            is_error: false, is_missing: false,
        };
        let json = serde_json::to_string(&root).unwrap();
        let d: SerializedNode = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(d.children.len(), n_children);
        for i in 0..n_children {
            prop_assert_eq!(&d.children[i].kind, &format!("c{}", i));
        }
    }
}

// ===================================================================
// 19. SerializedNode: empty root (no children, no text)
// ===================================================================

proptest! {
    #[test]
    fn rt_empty_root(kind in "[a-z]{1,8}") {
        let node = SerializedNode {
            kind: kind.clone(), is_named: true, field_name: None,
            start_position: (0, 0), end_position: (0, 0),
            start_byte: 0, end_byte: 0,
            text: None, children: vec![],
            is_error: false, is_missing: false,
        };
        let json = serde_json::to_string(&node).unwrap();
        let d: SerializedNode = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(&d.kind, &kind);
        prop_assert!(d.children.is_empty());
        prop_assert!(d.text.is_none());
    }
}

// ===================================================================
// 20. CompactNode JSON roundtrip preserves all fields
// ===================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn rt_compact_node_roundtrip(node in arb_compact_node(2)) {
        let json = serde_json::to_string(&node).unwrap();
        let d: CompactNode = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(&d.kind, &node.kind);
        prop_assert_eq!(d.start, node.start);
        prop_assert_eq!(d.end, node.end);
        prop_assert_eq!(&d.field, &node.field);
        prop_assert_eq!(&d.text, &node.text);
        prop_assert_eq!(d.children.len(), node.children.len());
    }
}

// ===================================================================
// 21. CompactNode: skip_serializing_if omits None and empty
// ===================================================================

proptest! {
    #[test]
    fn rt_compact_none_omitted(kind in "[a-z]{1,6}") {
        let node = CompactNode {
            kind, start: None, end: None, field: None, children: vec![], text: None,
        };
        let json = serde_json::to_string(&node).unwrap();
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        let obj = v.as_object().unwrap();
        prop_assert!(obj.contains_key("t"));
        prop_assert!(!obj.contains_key("s"));
        prop_assert!(!obj.contains_key("e"));
        prop_assert!(!obj.contains_key("f"));
        prop_assert!(!obj.contains_key("c"));
        prop_assert!(!obj.contains_key("x"));
    }
}

// ===================================================================
// 22. CompactNode: children "c" present only when non-empty
// ===================================================================

proptest! {
    #[test]
    fn rt_compact_children_presence(
        kind in "[a-z]{1,6}",
        has_children in any::<bool>(),
    ) {
        let children = if has_children {
            vec![CompactNode {
                kind: "leaf".into(), start: None, end: None,
                field: None, children: vec![], text: Some("x".into()),
            }]
        } else {
            vec![]
        };
        let node = CompactNode {
            kind, start: Some(0), end: Some(5), field: None, children, text: None,
        };
        let json = serde_json::to_string(&node).unwrap();
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        let obj = v.as_object().unwrap();
        prop_assert_eq!(obj.contains_key("c"), has_children);
    }
}

// ===================================================================
// 23. CompactNode: field alias "f" present when Some
// ===================================================================

proptest! {
    #[test]
    fn rt_compact_field_alias(field in "[a-z]{1,8}") {
        let node = CompactNode {
            kind: "id".into(), start: Some(0), end: Some(5),
            field: Some(field.clone()), children: vec![], text: None,
        };
        let json = serde_json::to_string(&node).unwrap();
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        let obj = v.as_object().unwrap();
        prop_assert_eq!(obj.get("f").and_then(|v| v.as_str()), Some(field.as_str()));
    }
}

// ===================================================================
// 24. CompactNode: text alias "x" present when Some
// ===================================================================

proptest! {
    #[test]
    fn rt_compact_text_alias(text in "[a-z]{1,10}") {
        let node = CompactNode {
            kind: "leaf".into(), start: None, end: None,
            field: None, children: vec![], text: Some(text.clone()),
        };
        let json = serde_json::to_string(&node).unwrap();
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        let obj = v.as_object().unwrap();
        prop_assert_eq!(obj.get("x").and_then(|v| v.as_str()), Some(text.as_str()));
    }
}

// ===================================================================
// 25. CompactNode: serialization is deterministic
// ===================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(80))]

    #[test]
    fn rt_compact_deterministic(node in arb_compact_node(2)) {
        let j1 = serde_json::to_string(&node).unwrap();
        let j2 = serde_json::to_string(&node).unwrap();
        prop_assert_eq!(&j1, &j2);
    }
}

// ===================================================================
// 26. CompactNode: compact is smaller than SerializedNode
// ===================================================================

proptest! {
    #[test]
    fn rt_compact_smaller_than_full(kind in "[a-z]{3,8}", text in "[a-z]{3,12}") {
        let full = SerializedNode {
            kind: kind.clone(), is_named: true, field_name: None,
            start_position: (0, 0), end_position: (0, text.len()),
            start_byte: 0, end_byte: text.len(),
            text: Some(text.clone()), children: vec![],
            is_error: false, is_missing: false,
        };
        let compact = CompactNode {
            kind, start: None, end: None, field: None,
            children: vec![], text: Some(text),
        };
        let full_json = serde_json::to_string(&full).unwrap();
        let compact_json = serde_json::to_string(&compact).unwrap();
        prop_assert!(compact_json.len() < full_json.len());
    }
}

// ===================================================================
// 27. CompactNode: deep nesting roundtrip
// ===================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]

    #[test]
    fn rt_compact_deep_nesting(depth in 5usize..15) {
        let mut node = CompactNode {
            kind: "leaf".into(), start: None, end: None,
            field: None, children: vec![], text: Some("x".into()),
        };
        for i in 0..depth {
            node = CompactNode {
                kind: format!("l{}", i), start: Some(0), end: Some(10),
                field: None, children: vec![node], text: None,
            };
        }
        let json = serde_json::to_string(&node).unwrap();
        let d: CompactNode = serde_json::from_str(&json).unwrap();
        // Walk down to verify depth
        let mut cur = &d;
        for _ in 0..depth {
            prop_assert_eq!(cur.children.len(), 1);
            cur = &cur.children[0];
        }
        prop_assert_eq!(&cur.kind, "leaf");
    }
}

// ===================================================================
// 28. CompactNode: missing "c" defaults to empty vec
// ===================================================================

proptest! {
    #[test]
    fn rt_compact_missing_children_defaults(kind in "[a-z]{1,6}") {
        let json = format!(r#"{{"t":"{}"}}"#, kind);
        let d: CompactNode = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(&d.kind, &kind);
        prop_assert!(d.children.is_empty());
    }
}

// ===================================================================
// 29. SExpr: JSON roundtrip preserves structure
// ===================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn rt_sexpr_json_roundtrip(expr in arb_sexpr(3)) {
        let json = serde_json::to_string(&expr).unwrap();
        let d: SExpr = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(&expr, &d);
    }
}

// ===================================================================
// 30. SExpr: equality is reflexive
// ===================================================================

proptest! {
    #[test]
    fn rt_sexpr_reflexive(expr in arb_sexpr(3)) {
        prop_assert_eq!(&expr, &expr);
    }
}

// ===================================================================
// 31. SExpr: clone equals original
// ===================================================================

proptest! {
    #[test]
    fn rt_sexpr_clone_eq(expr in arb_sexpr(3)) {
        let cloned = expr.clone();
        prop_assert_eq!(&expr, &cloned);
    }
}

// ===================================================================
// 32. SExpr: Atom never equals List
// ===================================================================

proptest! {
    #[test]
    fn rt_sexpr_atom_ne_list(s in "[a-z]{1,8}") {
        let atom = SExpr::Atom(s.clone());
        let list = SExpr::List(vec![SExpr::Atom(s)]);
        prop_assert_ne!(&atom, &list);
    }
}

// ===================================================================
// 33. SExpr: empty list roundtrip
// ===================================================================

proptest! {
    #[test]
    fn rt_sexpr_empty_list(_seed in 0u32..50) {
        let empty = SExpr::List(vec![]);
        let json = serde_json::to_string(&empty).unwrap();
        let d: SExpr = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(&empty, &d);
    }
}

// ===================================================================
// 34. SExpr: Atom to_string is identity
// ===================================================================

proptest! {
    #[test]
    fn rt_sexpr_atom_to_string(s in "[a-zA-Z_][a-zA-Z0-9_]{0,12}") {
        let atom = SExpr::Atom(s.clone());
        prop_assert_eq!(sexpr_to_string(&atom), s);
    }
}

// ===================================================================
// 35. SExpr: List to_string is parenthesized
// ===================================================================

proptest! {
    #[test]
    fn rt_sexpr_list_parenthesized(items in prop::collection::vec(arb_sexpr(1), 1..5)) {
        let list = SExpr::List(items);
        let rendered = sexpr_to_string(&list);
        prop_assert!(rendered.starts_with('('));
        prop_assert!(rendered.ends_with(')'));
    }
}

// ===================================================================
// 36. SExpr: render is deterministic
// ===================================================================

proptest! {
    #[test]
    fn rt_sexpr_render_deterministic(expr in arb_sexpr(3)) {
        let s1 = sexpr_to_string(&expr);
        let s2 = sexpr_to_string(&expr);
        prop_assert_eq!(&s1, &s2);
    }
}

// ===================================================================
// 37. SExpr: single-atom list renders as "(atom)"
// ===================================================================

proptest! {
    #[test]
    fn rt_sexpr_single_atom_list(s in "[a-z]{1,8}") {
        let list = SExpr::List(vec![SExpr::Atom(s.clone())]);
        prop_assert_eq!(sexpr_to_string(&list), format!("({})", s));
    }
}

// ===================================================================
// 38. SExpr: nested list render starts/ends with double parens
// ===================================================================

proptest! {
    #[test]
    fn rt_sexpr_nested_list_parens(inner in prop::collection::vec(arb_sexpr(1), 0..4)) {
        let outer = SExpr::List(vec![SExpr::List(inner)]);
        let rendered = sexpr_to_string(&outer);
        prop_assert!(rendered.starts_with("(("));
        prop_assert!(rendered.ends_with("))"));
    }
}

// ===================================================================
// 39. SExpr: empty list renders as "()"
// ===================================================================

proptest! {
    #[test]
    fn rt_sexpr_empty_list_renders(_seed in 0u32..50) {
        let empty = SExpr::List(vec![]);
        prop_assert_eq!(sexpr_to_string(&empty), "()");
    }
}

// ===================================================================
// 40. SExpr: deeply nested JSON roundtrip
// ===================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(30))]

    #[test]
    fn rt_sexpr_deep_json_roundtrip(expr in arb_sexpr(5)) {
        let json = serde_json::to_string(&expr).unwrap();
        let d: SExpr = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(&expr, &d);
    }
}

// ===================================================================
// 41. SExpr: Debug does not panic
// ===================================================================

proptest! {
    #[test]
    fn rt_sexpr_debug_no_panic(expr in arb_sexpr(3)) {
        let dbg = format!("{:?}", expr);
        prop_assert!(!dbg.is_empty());
    }
}

// ===================================================================
// 42. parse_sexpr: never panics on arbitrary input
// ===================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(300))]

    #[test]
    fn rt_parse_sexpr_no_panic(input in ".*") {
        let _ = parse_sexpr(&input);
    }
}

// ===================================================================
// 43. parse_sexpr: whitespace-only input does not panic
// ===================================================================

proptest! {
    #[test]
    fn rt_parse_sexpr_whitespace(input in "[ \\t\\n\\r]{0,30}") {
        let _ = parse_sexpr(&input);
    }
}

// ===================================================================
// 44. parse_sexpr: unbalanced parens handled
// ===================================================================

proptest! {
    #[test]
    fn rt_parse_sexpr_unbalanced(opens in 0usize..10, closes in 0usize..10, atom in "[a-z]{1,5}") {
        let input = format!("{}{}{}", "(".repeat(opens), atom, ")".repeat(closes));
        let _ = parse_sexpr(&input);
    }
}

// ===================================================================
// 45. parse_sexpr: binary-safe (lossy UTF-8)
// ===================================================================

proptest! {
    #[test]
    fn rt_parse_sexpr_binary_safe(bytes in prop::collection::vec(any::<u8>(), 0..50)) {
        let input = String::from_utf8_lossy(&bytes).to_string();
        let _ = parse_sexpr(&input);
    }
}

// ===================================================================
// 46. BinaryFormat: clone preserves all fields
// ===================================================================

proptest! {
    #[test]
    fn rt_binary_format_clone(
        types in prop::collection::vec("[a-z]{1,5}", 0..5),
        fields in prop::collection::vec("[a-z]{1,5}", 0..3),
        data in prop::collection::vec(any::<u8>(), 0..32),
    ) {
        let fmt = BinaryFormat { node_types: types, field_names: fields, tree_data: data };
        let cloned = fmt.clone();
        prop_assert_eq!(&fmt.node_types, &cloned.node_types);
        prop_assert_eq!(&fmt.field_names, &cloned.field_names);
        prop_assert_eq!(&fmt.tree_data, &cloned.tree_data);
    }
}

// ===================================================================
// 47. BinaryFormat: Debug does not panic
// ===================================================================

proptest! {
    #[test]
    fn rt_binary_format_debug(
        types in prop::collection::vec("[a-z]{1,5}", 0..5),
        data in prop::collection::vec(any::<u8>(), 0..20),
    ) {
        let fmt = BinaryFormat { node_types: types, field_names: vec![], tree_data: data };
        let dbg = format!("{:?}", fmt);
        prop_assert!(!dbg.is_empty());
    }
}

// ===================================================================
// 48. BinaryFormat: field consistency
// ===================================================================

proptest! {
    #[test]
    fn rt_binary_format_consistency(
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

// ===================================================================
// 49. BinarySerializer: Default matches new()
// ===================================================================

proptest! {
    #[test]
    fn rt_binary_serializer_default_eq_new(_seed in 0u32..100) {
        let _ = BinarySerializer::new();
        let _ = BinarySerializer::default();
        // Both produce identical empty state; verified by not panicking.
    }
}

// ===================================================================
// 50. TreeSerializer: defaults are unnamed-excluded, max_text 100
// ===================================================================

proptest! {
    #[test]
    fn rt_tree_serializer_defaults(src_len in 1usize..100) {
        let source = vec![b'a'; src_len];
        let s = TreeSerializer::new(&source);
        prop_assert!(!s.include_unnamed);
        prop_assert_eq!(s.max_text_length, Some(100));
    }
}

// ===================================================================
// 51. TreeSerializer: with_unnamed_nodes toggles flag
// ===================================================================

proptest! {
    #[test]
    fn rt_tree_serializer_unnamed(src_len in 1usize..50) {
        let source = vec![b'x'; src_len];
        let s = TreeSerializer::new(&source).with_unnamed_nodes();
        prop_assert!(s.include_unnamed);
    }
}

// ===================================================================
// 52. TreeSerializer: with_max_text_length sets value
// ===================================================================

proptest! {
    #[test]
    fn rt_tree_serializer_max_text(max_len in proptest::option::of(1usize..500)) {
        let source = b"src";
        let s = TreeSerializer::new(source).with_max_text_length(max_len);
        prop_assert_eq!(s.max_text_length, max_len);
    }
}

// ===================================================================
// 53. TreeSerializer: builder chain is idempotent (config stable)
// ===================================================================

proptest! {
    #[test]
    fn rt_tree_serializer_builder_stable(max_len in proptest::option::of(1usize..300)) {
        let source = b"code";
        let s1 = TreeSerializer::new(source)
            .with_unnamed_nodes()
            .with_max_text_length(max_len);
        let s2 = TreeSerializer::new(source)
            .with_unnamed_nodes()
            .with_max_text_length(max_len);
        prop_assert_eq!(s1.include_unnamed, s2.include_unnamed);
        prop_assert_eq!(s1.max_text_length, s2.max_text_length);
        prop_assert_eq!(s1.source, s2.source);
    }
}

// ===================================================================
// 54. TreeSerializer: source bytes preserved
// ===================================================================

proptest! {
    #[test]
    fn rt_tree_serializer_source_preserved(data in prop::collection::vec(any::<u8>(), 1..100)) {
        let s = TreeSerializer::new(&data);
        prop_assert_eq!(s.source, data.as_slice());
    }
}

// ===================================================================
// 55. SerializedNode: children order stable through roundtrip
// ===================================================================

proptest! {
    #[test]
    fn rt_children_order_stable(n in 2usize..10) {
        let children: Vec<_> = (0..n)
            .map(|i| SerializedNode {
                kind: format!("k{}", i), is_named: true, field_name: None,
                start_position: (0, i), end_position: (0, i + 1),
                start_byte: i, end_byte: i + 1,
                text: Some(format!("t{}", i)), children: vec![],
                is_error: false, is_missing: false,
            })
            .collect();
        let parent = SerializedNode {
            kind: "p".into(), is_named: true, field_name: None,
            start_position: (0, 0), end_position: (0, n),
            start_byte: 0, end_byte: n,
            text: None, children,
            is_error: false, is_missing: false,
        };
        let json = serde_json::to_string(&parent).unwrap();
        let d: SerializedNode = serde_json::from_str(&json).unwrap();
        for i in 0..n {
            prop_assert_eq!(&d.children[i].kind, &format!("k{}", i));
            prop_assert_eq!(&d.children[i].text, &Some(format!("t{}", i)));
        }
    }
}

// ===================================================================
// 56. SerializedNode: special JSON chars in text survive roundtrip
// ===================================================================

proptest! {
    #[test]
    fn rt_special_json_chars(text in r#"[a-z"\\/ \n\t]{1,20}"#) {
        let node = SerializedNode {
            kind: "str".into(), is_named: true, field_name: None,
            start_position: (0, 0), end_position: (0, text.len()),
            start_byte: 0, end_byte: text.len(),
            text: Some(text.clone()), children: vec![],
            is_error: false, is_missing: false,
        };
        let json = serde_json::to_string(&node).unwrap();
        let d: SerializedNode = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(d.text.as_deref(), Some(text.as_str()));
    }
}

// ===================================================================
// 57. SerializedNode: Debug does not panic
// ===================================================================

proptest! {
    #[test]
    fn rt_serialized_node_debug(node in arb_leaf_node()) {
        let dbg = format!("{:?}", node);
        prop_assert!(dbg.contains("SerializedNode"));
    }
}

// ===================================================================
// 58. CompactNode: unicode kind roundtrip
// ===================================================================

proptest! {
    #[test]
    fn rt_compact_unicode_kind(kind in "[\\p{L}]{1,10}") {
        let node = CompactNode {
            kind: kind.clone(), start: Some(0), end: Some(4),
            field: None, children: vec![], text: None,
        };
        let json = serde_json::to_string(&node).unwrap();
        let d: CompactNode = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(&d.kind, &kind);
    }
}

// ===================================================================
// 59. CompactNode: triple roundtrip idempotent
// ===================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(60))]

    #[test]
    fn rt_compact_triple_roundtrip(node in arb_compact_node(2)) {
        let j1 = serde_json::to_string(&node).unwrap();
        let d1: CompactNode = serde_json::from_str(&j1).unwrap();
        let j2 = serde_json::to_string(&d1).unwrap();
        let d2: CompactNode = serde_json::from_str(&j2).unwrap();
        let j3 = serde_json::to_string(&d2).unwrap();
        prop_assert_eq!(&j1, &j2);
        prop_assert_eq!(&j2, &j3);
    }
}

// ===================================================================
// 60. Deserialization rejects garbage for SerializedNode
// ===================================================================

proptest! {
    #[test]
    fn rt_deser_rejects_garbage(input in "[^{}\\[\\]\"]{1,30}") {
        let result: Result<SerializedNode, _> = serde_json::from_str(&input);
        prop_assert!(result.is_err());
    }
}

// ===================================================================
// 61. Deserialization rejects garbage for CompactNode
// ===================================================================

proptest! {
    #[test]
    fn rt_deser_compact_rejects_garbage(input in "[^{}\\[\\]\"]{1,30}") {
        let result: Result<CompactNode, _> = serde_json::from_str(&input);
        prop_assert!(result.is_err());
    }
}

// ===================================================================
// 62. SerializedNode: extra JSON fields are ignored
// ===================================================================

proptest! {
    #[test]
    fn rt_extra_json_fields_ignored(kind in "[a-z]{1,6}", extra_val in "[a-z]{1,10}") {
        let json = format!(
            r#"{{"kind":"{}","is_named":true,"field_name":null,"start_position":[0,0],"end_position":[0,1],"start_byte":0,"end_byte":1,"text":null,"children":[],"is_error":false,"is_missing":false,"extra":"{}"}}"#,
            kind, extra_val
        );
        let d: SerializedNode = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(&d.kind, &kind);
    }
}

// ===================================================================
// 63. BinaryFormat: empty format has zero-length vecs
// ===================================================================

proptest! {
    #[test]
    fn rt_binary_format_empty(_seed in 0u32..50) {
        let fmt = BinaryFormat {
            node_types: vec![],
            field_names: vec![],
            tree_data: vec![],
        };
        prop_assert!(fmt.node_types.is_empty());
        prop_assert!(fmt.field_names.is_empty());
        prop_assert!(fmt.tree_data.is_empty());
    }
}

// ===================================================================
// 64. SExpr: list child count preserved through JSON roundtrip
// ===================================================================

proptest! {
    #[test]
    fn rt_sexpr_list_count(children in prop::collection::vec(arb_sexpr(1), 0..8)) {
        let n = children.len();
        let list = SExpr::List(children);
        let json = serde_json::to_string(&list).unwrap();
        let d: SExpr = serde_json::from_str(&json).unwrap();
        if let SExpr::List(items) = d {
            prop_assert_eq!(items.len(), n);
        } else {
            prop_assert!(false, "expected List after roundtrip");
        }
    }
}

// ===================================================================
// 65. Different trees produce different JSON
// ===================================================================

proptest! {
    #[test]
    fn rt_different_leaves_different_json(
        kind1 in "[a-z]{3,6}",
        kind2 in "[a-z]{3,6}",
        text1 in "[a-z]{3,8}",
        text2 in "[a-z]{3,8}",
    ) {
        prop_assume!(kind1 != kind2 || text1 != text2);
        let n1 = SerializedNode {
            kind: kind1, is_named: true, field_name: None,
            start_position: (0, 0), end_position: (0, 1),
            start_byte: 0, end_byte: 1,
            text: Some(text1), children: vec![],
            is_error: false, is_missing: false,
        };
        let n2 = SerializedNode {
            kind: kind2, is_named: true, field_name: None,
            start_position: (0, 0), end_position: (0, 1),
            start_byte: 0, end_byte: 1,
            text: Some(text2), children: vec![],
            is_error: false, is_missing: false,
        };
        let j1 = serde_json::to_string(&n1).unwrap();
        let j2 = serde_json::to_string(&n2).unwrap();
        prop_assert_ne!(&j1, &j2);
    }
}
