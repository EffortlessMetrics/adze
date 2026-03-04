#![cfg(feature = "pure-rust")]
#![allow(clippy::needless_range_loop)]

//! Comprehensive tests for the forest-to-tree converter.
//!
//! Exercises forest-to-tree conversion across disambiguation strategies,
//! error paths, tree structure preservation, depth/node-count accuracy,
//! performance characteristics, and edge cases.

use adze_runtime::Node;
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

/// Count total nodes in a converted tree by DFS traversal.
fn count_nodes(node: Node<'_>) -> usize {
    let mut total = 1;
    for i in 0..node.child_count() {
        total += count_nodes(node.child(i).unwrap());
    }
    total
}

/// Compute max depth of a converted tree (root = depth 0).
fn max_depth(node: Node<'_>) -> usize {
    let mut deepest = 0;
    for i in 0..node.child_count() {
        deepest = deepest.max(1 + max_depth(node.child(i).unwrap()));
    }
    deepest
}

// ===========================================================================
// Empty forest conversion
// ===========================================================================

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

// ===========================================================================
// Single-node forest
// ===========================================================================

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

#[test]
fn single_leaf_node_count_is_one() {
    let forest = build_forest(vec![leaf(0, 1, 0..1)], vec![0]);
    let converter = ForestConverter::new(DisambiguationStrategy::First);

    let tree = converter.to_tree(&forest, b"x").unwrap();
    assert_eq!(count_nodes(tree.root_node()), 1);
}

#[test]
fn single_leaf_depth_is_zero() {
    let forest = build_forest(vec![leaf(0, 1, 0..1)], vec![0]);
    let converter = ForestConverter::new(DisambiguationStrategy::First);

    let tree = converter.to_tree(&forest, b"x").unwrap();
    assert_eq!(max_depth(tree.root_node()), 0);
}

#[test]
fn source_bytes_stored_in_tree() {
    let forest = build_forest(vec![leaf(0, 1, 0..4)], vec![0]);
    let converter = ForestConverter::new(DisambiguationStrategy::First);

    let tree = converter.to_tree(&forest, b"test").unwrap();
    assert_eq!(tree.source_bytes(), Some(b"test".as_slice()));
}

// ===========================================================================
// Multi-node forest
// ===========================================================================

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

#[test]
fn three_level_deep_tree() {
    let forest = build_forest(
        vec![
            leaf(0, 1, 0..1),
            internal(1, 2, vec![0], 0..1),
            internal(2, 3, vec![1], 0..1),
        ],
        vec![2],
    );
    let converter = ForestConverter::new(DisambiguationStrategy::First);

    let tree = converter.to_tree(&forest, b"x").unwrap();
    let root = tree.root_node();

    assert_eq!(root.kind_id(), 3);
    let child = root.child(0).unwrap();
    assert_eq!(child.kind_id(), 2);
    let grandchild = child.child(0).unwrap();
    assert_eq!(grandchild.kind_id(), 1);
    assert_eq!(grandchild.child_count(), 0);
}

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

#[test]
fn dag_shaped_forest_shared_child() {
    let forest = build_forest(
        vec![
            leaf(0, 1, 0..1),
            internal(1, 10, vec![0], 0..1),
            internal(2, 20, vec![0], 0..1),
            internal(3, 30, vec![1, 2], 0..1),
        ],
        vec![3],
    );
    let converter = ForestConverter::new(DisambiguationStrategy::First);

    let tree = converter.to_tree(&forest, b"x").unwrap();
    let root = tree.root_node();

    assert_eq!(root.kind_id(), 30);
    assert_eq!(root.child(0).unwrap().child(0).unwrap().kind_id(), 1);
    assert_eq!(root.child(1).unwrap().child(0).unwrap().kind_id(), 1);
}

// ===========================================================================
// Performance logging behavior
// ===========================================================================

#[test]
fn conversion_completes_within_reasonable_time() {
    // Build a moderately large forest (500 leaf nodes + 1 root).
    let n = 500;
    let mut entries: Vec<(usize, ForestNode)> = (0..n).map(|i| leaf(i, 1, i..(i + 1))).collect();
    entries.push(internal(n, 2, (0..n).collect(), 0..n));
    let forest = build_forest(entries, vec![n]);
    let converter = ForestConverter::new(DisambiguationStrategy::First);

    let start = std::time::Instant::now();
    let tree = converter.to_tree(&forest, &vec![b'x'; n]).unwrap();
    let elapsed = start.elapsed();

    assert_eq!(count_nodes(tree.root_node()), n + 1);
    // Conversion of 501 nodes should take well under 1 second.
    assert!(
        elapsed.as_secs() < 1,
        "Conversion took too long: {elapsed:?}"
    );
}

// ===========================================================================
// Tree depth calculations
// ===========================================================================

#[test]
fn depth_of_linear_chain() {
    // chain: root(3) -> mid(2) -> mid(1) -> leaf(0)  — depth 3
    let forest = build_forest(
        vec![
            leaf(0, 1, 0..1),
            internal(1, 2, vec![0], 0..1),
            internal(2, 3, vec![1], 0..1),
            internal(3, 4, vec![2], 0..1),
        ],
        vec![3],
    );
    let converter = ForestConverter::new(DisambiguationStrategy::First);
    let tree = converter.to_tree(&forest, b"x").unwrap();

    assert_eq!(max_depth(tree.root_node()), 3);
}

#[test]
fn depth_of_balanced_binary_tree() {
    //         6(root)
    //        / \
    //     4(L)  5(R)
    //    / \   / \
    //   0  1  2   3
    let forest = build_forest(
        vec![
            leaf(0, 1, 0..1),
            leaf(1, 1, 1..2),
            leaf(2, 1, 2..3),
            leaf(3, 1, 3..4),
            internal(4, 2, vec![0, 1], 0..2),
            internal(5, 2, vec![2, 3], 2..4),
            internal(6, 3, vec![4, 5], 0..4),
        ],
        vec![6],
    );
    let converter = ForestConverter::new(DisambiguationStrategy::First);
    let tree = converter.to_tree(&forest, b"abcd").unwrap();

    assert_eq!(max_depth(tree.root_node()), 2);
}

#[test]
fn depth_of_unbalanced_tree() {
    //    4(root)
    //   / \
    //  3   0(leaf)
    //  |
    //  2
    //  |
    //  1(leaf)
    let forest = build_forest(
        vec![
            leaf(0, 1, 3..4),
            leaf(1, 1, 0..1),
            internal(2, 2, vec![1], 0..1),
            internal(3, 3, vec![2], 0..1),
            internal(4, 4, vec![3, 0], 0..4),
        ],
        vec![4],
    );
    let converter = ForestConverter::new(DisambiguationStrategy::First);
    let tree = converter.to_tree(&forest, b"abcd").unwrap();

    assert_eq!(max_depth(tree.root_node()), 3);
}

// ===========================================================================
// Node count accuracy
// ===========================================================================

#[test]
fn node_count_parent_with_two_children() {
    let forest = build_forest(
        vec![
            leaf(0, 1, 0..1),
            leaf(1, 2, 1..2),
            internal(2, 3, vec![0, 1], 0..2),
        ],
        vec![2],
    );
    let converter = ForestConverter::new(DisambiguationStrategy::First);
    let tree = converter.to_tree(&forest, b"ab").unwrap();

    assert_eq!(count_nodes(tree.root_node()), 3);
}

#[test]
fn node_count_of_balanced_binary_tree() {
    // 7 nodes total (see depth_of_balanced_binary_tree layout)
    let forest = build_forest(
        vec![
            leaf(0, 1, 0..1),
            leaf(1, 1, 1..2),
            leaf(2, 1, 2..3),
            leaf(3, 1, 3..4),
            internal(4, 2, vec![0, 1], 0..2),
            internal(5, 2, vec![2, 3], 2..4),
            internal(6, 3, vec![4, 5], 0..4),
        ],
        vec![6],
    );
    let converter = ForestConverter::new(DisambiguationStrategy::First);
    let tree = converter.to_tree(&forest, b"abcd").unwrap();

    assert_eq!(count_nodes(tree.root_node()), 7);
}

#[test]
fn node_count_dag_counts_shared_nodes_per_parent() {
    // DAG with shared child: root -> A -> shared, root -> B -> shared
    // In the tree, shared is duplicated, so total = 5 (root + A + B + shared*2)
    let forest = build_forest(
        vec![
            leaf(0, 1, 0..1),
            internal(1, 10, vec![0], 0..1),
            internal(2, 20, vec![0], 0..1),
            internal(3, 30, vec![1, 2], 0..1),
        ],
        vec![3],
    );
    let converter = ForestConverter::new(DisambiguationStrategy::First);
    let tree = converter.to_tree(&forest, b"x").unwrap();

    assert_eq!(count_nodes(tree.root_node()), 5);
}

// ===========================================================================
// Forest with errors
// ===========================================================================

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

#[test]
fn reject_ambiguity_with_multiple_roots() {
    let forest = build_forest(vec![leaf(0, 1, 0..1), leaf(1, 2, 0..1)], vec![0, 1]);
    let converter = ForestConverter::new(DisambiguationStrategy::RejectAmbiguity);

    let err = converter.to_tree(&forest, b"x").unwrap_err();
    assert!(matches!(err, ConversionError::AmbiguousForest { count: 2 }));
}

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

#[test]
fn conversion_error_display_messages() {
    let cases: Vec<(ConversionError, &str)> = vec![
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

    for (err, expected_substr) in cases {
        let msg = err.to_string();
        assert!(
            msg.contains(expected_substr),
            "Error message {msg:?} should contain {expected_substr:?}"
        );
    }
}

#[test]
fn conversion_error_converts_to_parse_error() {
    use adze_runtime::ParseError;

    let conversion_err = ConversionError::NoRoots;
    let parse_err: ParseError = conversion_err.into();
    let msg = format!("{parse_err}");
    assert!(msg.contains("no root") || msg.contains("No") || msg.contains("root"));
}

// ===========================================================================
// Deep forest conversion
// ===========================================================================

#[test]
fn deep_linear_chain_converts_correctly() {
    let depth = 100;
    let mut entries: Vec<(usize, ForestNode)> = vec![leaf(0, 1, 0..1)];
    for i in 1..=depth {
        entries.push(internal(i, (i + 1) as u16, vec![i - 1], 0..1));
    }
    let forest = build_forest(entries, vec![depth]);
    let converter = ForestConverter::new(DisambiguationStrategy::First);

    let tree = converter.to_tree(&forest, b"x").unwrap();
    assert_eq!(count_nodes(tree.root_node()), depth + 1);
    assert_eq!(max_depth(tree.root_node()), depth);
}

#[test]
fn deep_forest_root_and_leaf_symbols_are_correct() {
    let depth = 50;
    let mut entries: Vec<(usize, ForestNode)> = vec![leaf(0, 42, 0..1)];
    for i in 1..=depth {
        entries.push(internal(i, 100, vec![i - 1], 0..1));
    }
    let forest = build_forest(entries, vec![depth]);
    let converter = ForestConverter::new(DisambiguationStrategy::First);

    let tree = converter.to_tree(&forest, b"x").unwrap();
    // Root symbol
    assert_eq!(tree.root_node().kind_id(), 100);
    // Walk to deepest leaf
    let mut node = tree.root_node();
    while node.child_count() > 0 {
        node = node.child(0).unwrap();
    }
    assert_eq!(node.kind_id(), 42);
}

// ===========================================================================
// Disambiguation strategies
// ===========================================================================

#[test]
fn first_strategy_picks_first_root() {
    let forest = build_forest(vec![leaf(0, 77, 0..1), leaf(1, 88, 0..1)], vec![0, 1]);
    let converter = ForestConverter::new(DisambiguationStrategy::First);

    let tree = converter.to_tree(&forest, b"x").unwrap();
    assert_eq!(tree.root_node().kind_id(), 77);
}

#[test]
fn prefer_shift_falls_back_to_first() {
    let forest = build_forest(vec![leaf(0, 10, 0..1), leaf(1, 20, 0..1)], vec![0, 1]);
    let converter = ForestConverter::new(DisambiguationStrategy::PreferShift);

    let tree = converter.to_tree(&forest, b"x").unwrap();
    assert_eq!(tree.root_node().kind_id(), 10);
}

#[test]
fn prefer_reduce_falls_back_to_first() {
    let forest = build_forest(vec![leaf(0, 30, 0..1), leaf(1, 40, 0..1)], vec![0, 1]);
    let converter = ForestConverter::new(DisambiguationStrategy::PreferReduce);

    let tree = converter.to_tree(&forest, b"x").unwrap();
    assert_eq!(tree.root_node().kind_id(), 30);
}

#[test]
fn disambiguation_strategy_clone_and_eq() {
    let all = [
        DisambiguationStrategy::PreferShift,
        DisambiguationStrategy::PreferReduce,
        DisambiguationStrategy::Precedence,
        DisambiguationStrategy::First,
        DisambiguationStrategy::RejectAmbiguity,
    ];
    for i in 0..all.len() {
        for j in 0..all.len() {
            if i == j {
                assert_eq!(all[i], all[j]);
            } else {
                assert_ne!(all[i], all[j]);
            }
        }
    }
}

// ===========================================================================
// Ambiguity detection
// ===========================================================================

#[test]
fn detect_ambiguity_single_root_returns_none() {
    let forest = build_forest(vec![leaf(0, 1, 0..1)], vec![0]);
    let converter = ForestConverter::new(DisambiguationStrategy::First);
    assert_eq!(converter.detect_ambiguity(&forest), None);
}

#[test]
fn detect_ambiguity_multiple_roots_returns_count() {
    let forest = build_forest(
        vec![leaf(0, 1, 0..1), leaf(1, 2, 0..1), leaf(2, 3, 0..1)],
        vec![0, 1, 2],
    );
    let converter = ForestConverter::new(DisambiguationStrategy::First);
    assert_eq!(converter.detect_ambiguity(&forest), Some(3));
}

// ===========================================================================
// Byte range & edge cases
// ===========================================================================

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

#[test]
fn zero_length_range_leaf() {
    let forest = build_forest(vec![leaf(0, 50, 3..3)], vec![0]);
    let converter = ForestConverter::new(DisambiguationStrategy::First);

    let tree = converter.to_tree(&forest, b"abcde").unwrap();
    assert_eq!(tree.root_node().byte_range(), 3..3);
    assert_eq!(tree.root_node().start_byte(), tree.root_node().end_byte());
}

#[test]
fn large_symbol_id() {
    let forest = build_forest(vec![leaf(0, u16::MAX, 0..1)], vec![0]);
    let converter = ForestConverter::new(DisambiguationStrategy::First);

    let tree = converter.to_tree(&forest, b"x").unwrap();
    assert_eq!(tree.root_node().kind_id(), u16::MAX);
}

#[test]
fn empty_input_still_produces_tree() {
    let forest = build_forest(vec![leaf(0, 1, 0..0)], vec![0]);
    let converter = ForestConverter::new(DisambiguationStrategy::First);

    let tree = converter.to_tree(&forest, b"").unwrap();
    assert_eq!(tree.source_bytes(), Some(b"".as_slice()));
    assert_eq!(tree.root_node().byte_range(), 0..0);
}
