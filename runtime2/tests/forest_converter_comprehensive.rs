#![cfg(feature = "pure-rust")]

//! Comprehensive tests for the ForestConverter module.
//!
//! Exercises forest-to-tree conversion across disambiguation strategies,
//! error paths, tree structure preservation, and edge cases.

use adze_runtime::forest_converter::{ConversionError, DisambiguationStrategy, ForestConverter};
use adze_runtime::glr_engine::{ForestNode, ForestNodeId, ParseForest};

use adze_glr_core::SymbolId;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn leaf(id: usize, symbol: u16, range: std::ops::Range<usize>) -> (usize, ForestNode) {
    (
        id,
        ForestNode {
            symbol: SymbolId(symbol),
            children: vec![],
            range,
        },
    )
}

fn internal(
    id: usize,
    symbol: u16,
    children: Vec<usize>,
    range: std::ops::Range<usize>,
) -> (usize, ForestNode) {
    (
        id,
        ForestNode {
            symbol: SymbolId(symbol),
            children: children.into_iter().map(ForestNodeId).collect(),
            range,
        },
    )
}

/// Build a ParseForest from a list of `(id, ForestNode)` pairs and root ids.
/// Nodes are sorted by id so index == id.
fn build_forest(mut entries: Vec<(usize, ForestNode)>, roots: Vec<usize>) -> ParseForest {
    entries.sort_by_key(|(id, _)| *id);
    ParseForest {
        nodes: entries.into_iter().map(|(_, n)| n).collect(),
        roots: roots.into_iter().map(ForestNodeId).collect(),
    }
}

// ---------------------------------------------------------------------------
// 1. Single leaf node converts correctly
// ---------------------------------------------------------------------------

#[test]
fn single_leaf_preserves_symbol_and_range() {
    let forest = build_forest(vec![leaf(0, 5, 0..3)], vec![0]);
    let converter = ForestConverter::new(DisambiguationStrategy::First);

    let tree = converter.to_tree(&forest, b"abc").unwrap();
    let root = tree.root_node();

    assert_eq!(root.kind_id(), 5);
    assert_eq!(root.byte_range(), 0..3);
    assert_eq!(root.child_count(), 0);
}

// ---------------------------------------------------------------------------
// 2. Source bytes are stored in the tree
// ---------------------------------------------------------------------------

#[test]
fn source_bytes_stored_in_tree() {
    let forest = build_forest(vec![leaf(0, 1, 0..4)], vec![0]);
    let converter = ForestConverter::new(DisambiguationStrategy::First);

    let tree = converter.to_tree(&forest, b"test").unwrap();
    assert_eq!(tree.source_bytes(), Some(b"test".as_slice()));
}

// ---------------------------------------------------------------------------
// 3. Parent-child structure is preserved
// ---------------------------------------------------------------------------

#[test]
fn parent_with_two_children() {
    let forest = build_forest(
        vec![
            leaf(0, 10, 0..1),
            leaf(1, 11, 1..2),
            internal(2, 20, vec![0, 1], 0..2),
        ],
        vec![2],
    );
    let converter = ForestConverter::new(DisambiguationStrategy::First);

    let tree = converter.to_tree(&forest, b"ab").unwrap();
    let root = tree.root_node();

    assert_eq!(root.kind_id(), 20);
    assert_eq!(root.child_count(), 2);
    assert_eq!(root.child(0).unwrap().kind_id(), 10);
    assert_eq!(root.child(1).unwrap().kind_id(), 11);
}

// ---------------------------------------------------------------------------
// 4. Three-level deep tree
// ---------------------------------------------------------------------------

#[test]
fn three_level_deep_tree() {
    let forest = build_forest(
        vec![
            leaf(0, 1, 0..1),              // grandchild
            internal(1, 2, vec![0], 0..1), // child
            internal(2, 3, vec![1], 0..1), // root
        ],
        vec![2],
    );
    let converter = ForestConverter::new(DisambiguationStrategy::First);

    let tree = converter.to_tree(&forest, b"x").unwrap();
    let root = tree.root_node();

    assert_eq!(root.kind_id(), 3);
    assert_eq!(root.child_count(), 1);
    let child = root.child(0).unwrap();
    assert_eq!(child.kind_id(), 2);
    assert_eq!(child.child_count(), 1);
    let grandchild = child.child(0).unwrap();
    assert_eq!(grandchild.kind_id(), 1);
    assert_eq!(grandchild.child_count(), 0);
}

// ---------------------------------------------------------------------------
// 5. Empty forest returns NoRoots error
// ---------------------------------------------------------------------------

#[test]
fn empty_forest_returns_no_roots() {
    let forest = ParseForest {
        nodes: vec![],
        roots: vec![],
    };
    let converter = ForestConverter::new(DisambiguationStrategy::First);

    let err = converter.to_tree(&forest, b"").unwrap_err();
    assert!(matches!(err, ConversionError::NoRoots));
    assert_eq!(err.to_string(), "Forest has no root nodes");
}

// ---------------------------------------------------------------------------
// 6. Invalid root node ID
// ---------------------------------------------------------------------------

#[test]
fn invalid_root_node_id_returns_error() {
    let forest = ParseForest {
        nodes: vec![],
        roots: vec![ForestNodeId(42)],
    };
    let converter = ForestConverter::new(DisambiguationStrategy::First);

    let err = converter.to_tree(&forest, b"").unwrap_err();
    assert!(matches!(
        err,
        ConversionError::InvalidNodeId { node_id: 42 }
    ));
}

// ---------------------------------------------------------------------------
// 7. Invalid child reference returns error
// ---------------------------------------------------------------------------

#[test]
fn invalid_child_reference_returns_error() {
    let forest = build_forest(vec![internal(0, 1, vec![999], 0..1)], vec![0]);
    let converter = ForestConverter::new(DisambiguationStrategy::First);

    let err = converter.to_tree(&forest, b"x").unwrap_err();
    assert!(matches!(
        err,
        ConversionError::InvalidNodeId { node_id: 999 }
    ));
    assert!(err.to_string().contains("999"));
}

// ---------------------------------------------------------------------------
// 8. Multiple roots with RejectAmbiguity strategy
// ---------------------------------------------------------------------------

#[test]
fn reject_ambiguity_with_multiple_roots() {
    let forest = build_forest(vec![leaf(0, 1, 0..1), leaf(1, 2, 0..1)], vec![0, 1]);
    let converter = ForestConverter::new(DisambiguationStrategy::RejectAmbiguity);

    let err = converter.to_tree(&forest, b"x").unwrap_err();
    assert!(matches!(err, ConversionError::AmbiguousForest { count: 2 }));
    assert!(err.to_string().contains("2 valid parses"));
}

// ---------------------------------------------------------------------------
// 9. Multiple roots with First strategy picks first
// ---------------------------------------------------------------------------

#[test]
fn first_strategy_picks_first_root() {
    let forest = build_forest(vec![leaf(0, 77, 0..1), leaf(1, 88, 0..1)], vec![0, 1]);
    let converter = ForestConverter::new(DisambiguationStrategy::First);

    let tree = converter.to_tree(&forest, b"x").unwrap();
    assert_eq!(tree.root_node().kind_id(), 77);
}

// ---------------------------------------------------------------------------
// 10. Multiple roots with PreferShift falls back to first
// ---------------------------------------------------------------------------

#[test]
fn prefer_shift_with_multiple_roots_falls_back_to_first() {
    let forest = build_forest(vec![leaf(0, 10, 0..1), leaf(1, 20, 0..1)], vec![0, 1]);
    let converter = ForestConverter::new(DisambiguationStrategy::PreferShift);

    let tree = converter.to_tree(&forest, b"x").unwrap();
    assert_eq!(tree.root_node().kind_id(), 10);
}

// ---------------------------------------------------------------------------
// 11. Multiple roots with PreferReduce falls back to first
// ---------------------------------------------------------------------------

#[test]
fn prefer_reduce_with_multiple_roots_falls_back_to_first() {
    let forest = build_forest(vec![leaf(0, 30, 0..1), leaf(1, 40, 0..1)], vec![0, 1]);
    let converter = ForestConverter::new(DisambiguationStrategy::PreferReduce);

    let tree = converter.to_tree(&forest, b"x").unwrap();
    assert_eq!(tree.root_node().kind_id(), 30);
}

// ---------------------------------------------------------------------------
// 12. detect_ambiguity: unambiguous forest
// ---------------------------------------------------------------------------

#[test]
fn detect_ambiguity_single_root_returns_none() {
    let forest = build_forest(vec![leaf(0, 1, 0..1)], vec![0]);
    let converter = ForestConverter::new(DisambiguationStrategy::First);

    assert_eq!(converter.detect_ambiguity(&forest), None);
}

// ---------------------------------------------------------------------------
// 13. detect_ambiguity: multiple roots
// ---------------------------------------------------------------------------

#[test]
fn detect_ambiguity_multiple_roots_returns_count() {
    let forest = build_forest(
        vec![leaf(0, 1, 0..1), leaf(1, 2, 0..1), leaf(2, 3, 0..1)],
        vec![0, 1, 2],
    );
    let converter = ForestConverter::new(DisambiguationStrategy::First);

    assert_eq!(converter.detect_ambiguity(&forest), Some(3));
}

// ---------------------------------------------------------------------------
// 14. detect_ambiguity: empty forest
// ---------------------------------------------------------------------------

#[test]
fn detect_ambiguity_empty_forest_returns_none() {
    let forest = ParseForest {
        nodes: vec![],
        roots: vec![],
    };
    let converter = ForestConverter::new(DisambiguationStrategy::First);

    assert_eq!(converter.detect_ambiguity(&forest), None);
}

// ---------------------------------------------------------------------------
// 15. Wide tree: root with many children
// ---------------------------------------------------------------------------

#[test]
fn wide_tree_with_many_children() {
    let count = 10;
    let mut entries: Vec<(usize, ForestNode)> =
        (0..count).map(|i| leaf(i, i as u16, i..(i + 1))).collect();
    entries.push(internal(count, 100, (0..count).collect(), 0..count));
    let forest = build_forest(entries, vec![count]);
    let converter = ForestConverter::new(DisambiguationStrategy::First);

    let tree = converter.to_tree(&forest, &vec![b'a'; count]).unwrap();
    let root = tree.root_node();

    assert_eq!(root.kind_id(), 100);
    assert_eq!(root.child_count(), count);
    for i in 0..count {
        assert_eq!(root.child(i).unwrap().kind_id(), i as u16);
        assert_eq!(root.child(i).unwrap().byte_range(), i..(i + 1));
    }
}

// ---------------------------------------------------------------------------
// 16. Byte ranges preserved through conversion
// ---------------------------------------------------------------------------

#[test]
fn byte_ranges_preserved_for_internal_nodes() {
    let forest = build_forest(
        vec![
            leaf(0, 1, 5..10),
            leaf(1, 2, 10..20),
            internal(2, 3, vec![0, 1], 5..20),
        ],
        vec![2],
    );
    let converter = ForestConverter::new(DisambiguationStrategy::First);

    let tree = converter.to_tree(&forest, &[0u8; 20]).unwrap();
    let root = tree.root_node();

    assert_eq!(root.byte_range(), 5..20);
    assert_eq!(root.start_byte(), 5);
    assert_eq!(root.end_byte(), 20);
}

// ---------------------------------------------------------------------------
// 17. ConversionError Display implementations
// ---------------------------------------------------------------------------

#[test]
fn conversion_error_display_messages() {
    let errors = vec![
        (ConversionError::NoRoots, "Forest has no root nodes"),
        (
            ConversionError::AmbiguousForest { count: 5 },
            "5 valid parses",
        ),
        (
            ConversionError::InvalidForest {
                reason: "broken".to_string(),
            },
            "broken",
        ),
        (ConversionError::InvalidNodeId { node_id: 99 }, "99"),
        (
            ConversionError::CycleDetected { node_id: 7 },
            "Cycle detected",
        ),
    ];

    for (err, expected_substr) in errors {
        let msg = err.to_string();
        assert!(
            msg.contains(expected_substr),
            "Error message {msg:?} should contain {expected_substr:?}"
        );
    }
}

// ---------------------------------------------------------------------------
// 18. ConversionError converts to ParseError
// ---------------------------------------------------------------------------

#[test]
fn conversion_error_converts_to_parse_error() {
    use adze_runtime::ParseError;

    let conversion_err = ConversionError::NoRoots;
    let parse_err: ParseError = conversion_err.into();
    let msg = format!("{parse_err}");
    assert!(msg.contains("no root") || msg.contains("No") || msg.contains("root"));
}

// ---------------------------------------------------------------------------
// 19. DisambiguationStrategy equality and clone
// ---------------------------------------------------------------------------

#[test]
fn disambiguation_strategy_clone_and_eq() {
    let s1 = DisambiguationStrategy::PreferShift;
    let s2 = s1;
    assert_eq!(s1, s2);

    let s3 = DisambiguationStrategy::RejectAmbiguity;
    assert_ne!(s1, s3);

    // All variants are distinct
    let all = [
        DisambiguationStrategy::PreferShift,
        DisambiguationStrategy::PreferReduce,
        DisambiguationStrategy::Precedence,
        DisambiguationStrategy::First,
        DisambiguationStrategy::RejectAmbiguity,
    ];
    for (i, a) in all.iter().enumerate() {
        for (j, b) in all.iter().enumerate() {
            if i == j {
                assert_eq!(a, b);
            } else {
                assert_ne!(a, b);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// 20. ForestConverter Debug impl
// ---------------------------------------------------------------------------

#[test]
fn forest_converter_debug_not_empty() {
    let converter = ForestConverter::new(DisambiguationStrategy::Precedence);
    let debug = format!("{converter:?}");
    assert!(!debug.is_empty());
    assert!(debug.contains("ForestConverter"));
}

// ---------------------------------------------------------------------------
// 21. Zero-length range leaf node
// ---------------------------------------------------------------------------

#[test]
fn zero_length_range_leaf() {
    let forest = build_forest(vec![leaf(0, 50, 3..3)], vec![0]);
    let converter = ForestConverter::new(DisambiguationStrategy::First);

    let tree = converter.to_tree(&forest, b"abcde").unwrap();
    let root = tree.root_node();

    assert_eq!(root.byte_range(), 3..3);
    assert_eq!(root.start_byte(), root.end_byte());
}

// ---------------------------------------------------------------------------
// 22. DAG-shaped forest (shared children)
// ---------------------------------------------------------------------------

#[test]
fn dag_shaped_forest_shared_child() {
    // Two parent nodes share the same child node.
    // The converter should handle this without errors.
    let forest = build_forest(
        vec![
            leaf(0, 1, 0..1),                  // shared child
            internal(1, 10, vec![0], 0..1),    // parent A
            internal(2, 20, vec![0], 0..1),    // parent B (shares child 0)
            internal(3, 30, vec![1, 2], 0..1), // root
        ],
        vec![3],
    );
    let converter = ForestConverter::new(DisambiguationStrategy::First);

    let tree = converter.to_tree(&forest, b"x").unwrap();
    let root = tree.root_node();

    assert_eq!(root.kind_id(), 30);
    assert_eq!(root.child_count(), 2);
    // Both children should reference the shared grandchild
    assert_eq!(root.child(0).unwrap().child(0).unwrap().kind_id(), 1);
    assert_eq!(root.child(1).unwrap().child(0).unwrap().kind_id(), 1);
}

// ---------------------------------------------------------------------------
// 23. Large symbol IDs
// ---------------------------------------------------------------------------

#[test]
fn large_symbol_id() {
    let forest = build_forest(vec![leaf(0, u16::MAX, 0..1)], vec![0]);
    let converter = ForestConverter::new(DisambiguationStrategy::First);

    let tree = converter.to_tree(&forest, b"x").unwrap();
    assert_eq!(tree.root_node().kind_id(), u16::MAX);
}

// ---------------------------------------------------------------------------
// 24. Empty input bytes with non-zero range
// ---------------------------------------------------------------------------

#[test]
fn empty_input_still_produces_tree() {
    let forest = build_forest(vec![leaf(0, 1, 0..0)], vec![0]);
    let converter = ForestConverter::new(DisambiguationStrategy::First);

    let tree = converter.to_tree(&forest, b"").unwrap();
    assert_eq!(tree.source_bytes(), Some(b"".as_slice()));
    assert_eq!(tree.root_node().byte_range(), 0..0);
}

// ---------------------------------------------------------------------------
// 25. ConversionError is std::error::Error
// ---------------------------------------------------------------------------

#[test]
fn conversion_error_implements_std_error() {
    let err: Box<dyn std::error::Error> = Box::new(ConversionError::NoRoots);
    // Ensure Display + Debug + Error trait work
    let _ = format!("{err}");
    let _ = format!("{err:?}");
}

// ---------------------------------------------------------------------------
// 26. Reject ambiguity with three roots reports correct count
// ---------------------------------------------------------------------------

#[test]
fn reject_ambiguity_reports_exact_root_count() {
    let forest = build_forest(
        vec![
            leaf(0, 1, 0..1),
            leaf(1, 2, 0..1),
            leaf(2, 3, 0..1),
            leaf(3, 4, 0..1),
        ],
        vec![0, 1, 2, 3],
    );
    let converter = ForestConverter::new(DisambiguationStrategy::RejectAmbiguity);

    match converter.to_tree(&forest, b"x").unwrap_err() {
        ConversionError::AmbiguousForest { count } => assert_eq!(count, 4),
        other => panic!("Expected AmbiguousForest, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// 27. Children ordering is preserved
// ---------------------------------------------------------------------------

#[test]
fn children_ordering_matches_forest() {
    let forest = build_forest(
        vec![
            leaf(0, 100, 0..1),
            leaf(1, 200, 1..2),
            leaf(2, 300, 2..3),
            internal(3, 999, vec![0, 1, 2], 0..3),
        ],
        vec![3],
    );
    let converter = ForestConverter::new(DisambiguationStrategy::First);

    let tree = converter.to_tree(&forest, b"abc").unwrap();
    let root = tree.root_node();

    let ids: Vec<u16> = (0..root.child_count())
        .map(|i| root.child(i).unwrap().kind_id())
        .collect();
    assert_eq!(ids, vec![100, 200, 300]);
}
