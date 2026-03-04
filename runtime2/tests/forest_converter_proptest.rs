#![cfg(feature = "pure-rust")]
#![allow(clippy::needless_range_loop)]

//! Property-based tests for the forest-to-tree converter (`ForestConverter`).

use proptest::prelude::*;

use adze_glr_core::SymbolId;
use adze_runtime::forest_converter::{ConversionError, DisambiguationStrategy, ForestConverter};
use adze_runtime::glr_engine::{ForestNode, ForestNodeId, ParseForest};

// ---------------------------------------------------------------------------
// Strategies
// ---------------------------------------------------------------------------

fn arb_strategy() -> impl Strategy<Value = DisambiguationStrategy> {
    prop_oneof![
        Just(DisambiguationStrategy::First),
        Just(DisambiguationStrategy::PreferShift),
        Just(DisambiguationStrategy::PreferReduce),
        Just(DisambiguationStrategy::Precedence),
    ]
}

/// Produce a single-leaf ParseForest with arbitrary symbol and byte range.
fn arb_single_leaf_forest() -> impl Strategy<Value = (ParseForest, Vec<u8>)> {
    (1u16..500, 0usize..128).prop_map(|(sym, len)| {
        let forest = ParseForest {
            nodes: vec![ForestNode {
                symbol: SymbolId(sym),
                children: vec![],
                range: 0..len,
            }],
            roots: vec![ForestNodeId(0)],
        };
        let input = vec![b'a'; len];
        (forest, input)
    })
}

/// Produce a two-level forest: one root with 1..=max_children leaf children.
fn arb_parent_children_forest(
    max_children: usize,
) -> impl Strategy<Value = (ParseForest, Vec<u8>)> {
    let max_c = max_children.max(1);
    (1u16..500, 1usize..=max_c, 1usize..64).prop_map(move |(sym, n_children, chunk)| {
        let total_len = n_children * chunk;
        let mut nodes = Vec::with_capacity(n_children + 1);
        let mut child_ids = Vec::with_capacity(n_children);
        for i in 0..n_children {
            let start = i * chunk;
            let end = start + chunk;
            nodes.push(ForestNode {
                symbol: SymbolId((sym + 1 + i as u16) % 500),
                children: vec![],
                range: start..end,
            });
            child_ids.push(ForestNodeId(i));
        }
        nodes.push(ForestNode {
            symbol: SymbolId(sym),
            children: child_ids,
            range: 0..total_len,
        });
        let root_idx = n_children;
        let forest = ParseForest {
            nodes,
            roots: vec![ForestNodeId(root_idx)],
        };
        let input = vec![b'b'; total_len];
        (forest, input)
    })
}

/// Build a linear chain forest of given depth (root -> child -> ... -> leaf).
fn arb_linear_chain_forest(max_depth: usize) -> impl Strategy<Value = (ParseForest, Vec<u8>)> {
    let max_d = max_depth.max(1);
    (1u16..500, 1usize..=max_d, 1usize..32).prop_map(move |(sym, depth, len)| {
        let mut nodes = Vec::with_capacity(depth);
        // Build bottom-up: node 0 is the leaf, node depth-1 is the root
        for i in 0..depth {
            let children = if i == 0 {
                vec![]
            } else {
                vec![ForestNodeId(i - 1)]
            };
            nodes.push(ForestNode {
                symbol: SymbolId((sym + i as u16) % 500),
                children,
                range: 0..len,
            });
        }
        let root_idx = depth - 1;
        let forest = ParseForest {
            nodes,
            roots: vec![ForestNodeId(root_idx)],
        };
        let input: Vec<u8> = vec![b'x'; len];
        (forest, input)
    })
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Recursively count all nodes in a tree.
fn count_tree_nodes(node: adze_runtime::Node<'_>) -> usize {
    let mut total = 1;
    for i in 0..node.child_count() {
        total += count_tree_nodes(node.child(i).unwrap());
    }
    total
}

/// Compute maximum depth (root = 0).
fn tree_depth(node: adze_runtime::Node<'_>) -> usize {
    let mut deepest = 0;
    for i in 0..node.child_count() {
        deepest = deepest.max(1 + tree_depth(node.child(i).unwrap()));
    }
    deepest
}

/// Verify every node has start_byte <= end_byte.
fn assert_ranges_valid(node: adze_runtime::Node<'_>) {
    assert!(
        node.start_byte() <= node.end_byte(),
        "node kind_id={} has start {} > end {}",
        node.kind_id(),
        node.start_byte(),
        node.end_byte()
    );
    for i in 0..node.child_count() {
        assert_ranges_valid(node.child(i).unwrap());
    }
}

/// Verify parent range encompasses all children.
fn assert_parent_covers_children(node: adze_runtime::Node<'_>) {
    for i in 0..node.child_count() {
        let child = node.child(i).unwrap();
        assert!(
            child.start_byte() >= node.start_byte(),
            "child start {} < parent start {}",
            child.start_byte(),
            node.start_byte()
        );
        assert!(
            child.end_byte() <= node.end_byte(),
            "child end {} > parent end {}",
            child.end_byte(),
            node.end_byte()
        );
        assert_parent_covers_children(child);
    }
}

// ===========================================================================
// 1 – Converter produces valid tree (proptest)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    /// Any single-leaf forest converts to a tree with valid ranges.
    #[test]
    fn single_leaf_produces_valid_ranges((forest, input) in arb_single_leaf_forest()) {
        let converter = ForestConverter::new(DisambiguationStrategy::First);
        let tree = converter.to_tree(&forest, &input).unwrap();
        assert_ranges_valid(tree.root_node());
    }

    /// Parent-children forest produces a tree whose root covers all children.
    #[test]
    fn parent_children_root_covers_children(
        (forest, input) in arb_parent_children_forest(8)
    ) {
        let converter = ForestConverter::new(DisambiguationStrategy::First);
        let tree = converter.to_tree(&forest, &input).unwrap();
        assert_parent_covers_children(tree.root_node());
    }

    /// Converted tree always has start_byte <= end_byte at every node.
    #[test]
    fn all_nodes_have_valid_ranges(
        (forest, input) in arb_parent_children_forest(6)
    ) {
        let converter = ForestConverter::new(DisambiguationStrategy::First);
        let tree = converter.to_tree(&forest, &input).unwrap();
        assert_ranges_valid(tree.root_node());
    }

    /// Root byte_range matches the forest root's range.
    #[test]
    fn root_byte_range_matches_forest(
        (forest, input) in arb_single_leaf_forest()
    ) {
        let converter = ForestConverter::new(DisambiguationStrategy::First);
        let tree = converter.to_tree(&forest, &input).unwrap();
        let expected = forest.nodes[forest.roots[0].0].range.clone();
        prop_assert_eq!(tree.root_node().byte_range(), expected);
    }

    /// Source bytes are preserved in the tree.
    #[test]
    fn source_bytes_preserved(
        (forest, input) in arb_single_leaf_forest()
    ) {
        let converter = ForestConverter::new(DisambiguationStrategy::First);
        let tree = converter.to_tree(&forest, &input).unwrap();
        prop_assert_eq!(tree.source_bytes(), Some(input.as_slice()));
    }
}

// ===========================================================================
// 2 – Conversion preserves structure (proptest)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    /// Node count in the tree equals node count reachable from the forest root.
    #[test]
    fn node_count_matches_forest(
        (forest, input) in arb_parent_children_forest(8)
    ) {
        let converter = ForestConverter::new(DisambiguationStrategy::First);
        let tree = converter.to_tree(&forest, &input).unwrap();
        // Count reachable nodes in forest from root
        fn count_forest(forest: &ParseForest, id: ForestNodeId) -> usize {
            let n = &forest.nodes[id.0];
            1 + n.children.iter().map(|c| count_forest(forest, *c)).sum::<usize>()
        }
        let expected = count_forest(&forest, forest.roots[0]);
        prop_assert_eq!(count_tree_nodes(tree.root_node()), expected);
    }

    /// Child count of root matches forest root's children count.
    #[test]
    fn child_count_matches_forest(
        (forest, input) in arb_parent_children_forest(8)
    ) {
        let converter = ForestConverter::new(DisambiguationStrategy::First);
        let tree = converter.to_tree(&forest, &input).unwrap();
        let root_forest = &forest.nodes[forest.roots[0].0];
        prop_assert_eq!(tree.root_node().child_count(), root_forest.children.len());
    }

    /// Symbol ID of the root matches the forest root's symbol.
    #[test]
    fn root_symbol_preserved(
        (forest, input) in arb_single_leaf_forest()
    ) {
        let converter = ForestConverter::new(DisambiguationStrategy::First);
        let tree = converter.to_tree(&forest, &input).unwrap();
        let expected_sym = forest.nodes[forest.roots[0].0].symbol.0;
        prop_assert_eq!(tree.root_node().kind_id(), expected_sym);
    }

    /// Each child's symbol ID matches corresponding forest child's symbol.
    #[test]
    fn children_symbols_preserved(
        (forest, input) in arb_parent_children_forest(8)
    ) {
        let converter = ForestConverter::new(DisambiguationStrategy::First);
        let tree = converter.to_tree(&forest, &input).unwrap();
        let root_forest = &forest.nodes[forest.roots[0].0];
        let root_node = tree.root_node();
        for i in 0..root_forest.children.len() {
            let child_sym = forest.nodes[root_forest.children[i].0].symbol.0;
            let tree_child = root_node.child(i).unwrap();
            prop_assert_eq!(tree_child.kind_id(), child_sym);
        }
    }

    /// Linear chain depth is preserved.
    #[test]
    fn linear_chain_depth_preserved(
        (forest, input) in arb_linear_chain_forest(10)
    ) {
        let converter = ForestConverter::new(DisambiguationStrategy::First);
        let tree = converter.to_tree(&forest, &input).unwrap();
        // depth = number of nodes - 1 (root depth 0, then each link adds 1)
        let expected_depth = forest.nodes.len() - 1;
        prop_assert_eq!(tree_depth(tree.root_node()), expected_depth);
    }

    /// Children byte ranges are preserved from the forest.
    #[test]
    fn children_ranges_preserved(
        (forest, input) in arb_parent_children_forest(6)
    ) {
        let converter = ForestConverter::new(DisambiguationStrategy::First);
        let tree = converter.to_tree(&forest, &input).unwrap();
        let root_forest = &forest.nodes[forest.roots[0].0];
        let root_node = tree.root_node();
        for i in 0..root_forest.children.len() {
            let expected_range = forest.nodes[root_forest.children[i].0].range.clone();
            let tree_child = root_node.child(i).unwrap();
            prop_assert_eq!(tree_child.byte_range(), expected_range);
        }
    }
}

// ===========================================================================
// 3 – Empty forest handling
// ===========================================================================

#[test]
fn empty_forest_returns_no_roots_error() {
    let forest = ParseForest {
        nodes: vec![],
        roots: vec![],
    };
    let converter = ForestConverter::new(DisambiguationStrategy::First);
    let err = converter.to_tree(&forest, b"").unwrap_err();
    assert!(matches!(err, ConversionError::NoRoots));
}

#[test]
fn empty_forest_error_display() {
    let forest = ParseForest {
        nodes: vec![],
        roots: vec![],
    };
    let converter = ForestConverter::new(DisambiguationStrategy::First);
    let err = converter.to_tree(&forest, b"").unwrap_err();
    assert_eq!(err.to_string(), "Forest has no root nodes");
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    /// Empty forest always fails regardless of disambiguation strategy.
    #[test]
    fn empty_forest_fails_all_strategies(strategy in arb_strategy()) {
        let forest = ParseForest {
            nodes: vec![],
            roots: vec![],
        };
        let converter = ForestConverter::new(strategy);
        let err = converter.to_tree(&forest, b"").unwrap_err();
        prop_assert!(matches!(err, ConversionError::NoRoots));
    }

    /// Empty forest fails regardless of input content.
    #[test]
    fn empty_forest_fails_any_input(input in prop::collection::vec(any::<u8>(), 0..64)) {
        let forest = ParseForest {
            nodes: vec![],
            roots: vec![],
        };
        let converter = ForestConverter::new(DisambiguationStrategy::First);
        let err = converter.to_tree(&forest, &input).unwrap_err();
        prop_assert!(matches!(err, ConversionError::NoRoots));
    }
}

// ===========================================================================
// 4 – Single-node forest
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    /// Single-leaf tree has zero children.
    #[test]
    fn single_leaf_has_no_children(
        (forest, input) in arb_single_leaf_forest()
    ) {
        let converter = ForestConverter::new(DisambiguationStrategy::First);
        let tree = converter.to_tree(&forest, &input).unwrap();
        prop_assert_eq!(tree.root_node().child_count(), 0);
    }

    /// Single-leaf tree node count is 1.
    #[test]
    fn single_leaf_node_count_is_one(
        (forest, input) in arb_single_leaf_forest()
    ) {
        let converter = ForestConverter::new(DisambiguationStrategy::First);
        let tree = converter.to_tree(&forest, &input).unwrap();
        prop_assert_eq!(count_tree_nodes(tree.root_node()), 1);
    }

    /// Single-leaf tree depth is 0.
    #[test]
    fn single_leaf_depth_is_zero(
        (forest, input) in arb_single_leaf_forest()
    ) {
        let converter = ForestConverter::new(DisambiguationStrategy::First);
        let tree = converter.to_tree(&forest, &input).unwrap();
        prop_assert_eq!(tree_depth(tree.root_node()), 0);
    }

    /// Single-leaf succeeds with any strategy (no ambiguity).
    #[test]
    fn single_leaf_succeeds_all_strategies(
        strategy in arb_strategy(),
        sym in 1u16..500,
        len in 0usize..64,
    ) {
        let forest = ParseForest {
            nodes: vec![ForestNode {
                symbol: SymbolId(sym),
                children: vec![],
                range: 0..len,
            }],
            roots: vec![ForestNodeId(0)],
        };
        let input = vec![0u8; len];
        let converter = ForestConverter::new(strategy);
        let tree = converter.to_tree(&forest, &input).unwrap();
        prop_assert_eq!(tree.root_node().kind_id(), sym);
    }
}

// ===========================================================================
// 5 – Converter determinism (proptest)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    /// Converting the same single-leaf forest twice yields identical trees.
    #[test]
    fn single_leaf_deterministic(
        (forest, input) in arb_single_leaf_forest()
    ) {
        let converter = ForestConverter::new(DisambiguationStrategy::First);
        let t1 = converter.to_tree(&forest, &input).unwrap();
        let t2 = converter.to_tree(&forest, &input).unwrap();
        prop_assert_eq!(t1.root_node().kind_id(), t2.root_node().kind_id());
        prop_assert_eq!(t1.root_node().byte_range(), t2.root_node().byte_range());
        prop_assert_eq!(t1.root_node().child_count(), t2.root_node().child_count());
    }

    /// Converting the same parent-children forest twice yields identical trees.
    #[test]
    fn parent_children_deterministic(
        (forest, input) in arb_parent_children_forest(8)
    ) {
        let converter = ForestConverter::new(DisambiguationStrategy::First);
        let t1 = converter.to_tree(&forest, &input).unwrap();
        let t2 = converter.to_tree(&forest, &input).unwrap();
        prop_assert_eq!(count_tree_nodes(t1.root_node()), count_tree_nodes(t2.root_node()));
        prop_assert_eq!(t1.root_node().kind_id(), t2.root_node().kind_id());
        for i in 0..t1.root_node().child_count() {
            let c1 = t1.root_node().child(i).unwrap();
            let c2 = t2.root_node().child(i).unwrap();
            prop_assert_eq!(c1.kind_id(), c2.kind_id());
            prop_assert_eq!(c1.byte_range(), c2.byte_range());
        }
    }

    /// Converting a linear chain forest twice yields identical depth and symbols.
    #[test]
    fn linear_chain_deterministic(
        (forest, input) in arb_linear_chain_forest(8)
    ) {
        let converter = ForestConverter::new(DisambiguationStrategy::First);
        let t1 = converter.to_tree(&forest, &input).unwrap();
        let t2 = converter.to_tree(&forest, &input).unwrap();
        prop_assert_eq!(tree_depth(t1.root_node()), tree_depth(t2.root_node()));
        prop_assert_eq!(count_tree_nodes(t1.root_node()), count_tree_nodes(t2.root_node()));
    }

    /// Different strategies on single-root forest produce trees with same structure.
    #[test]
    fn different_strategies_same_single_root(
        (forest, input) in arb_parent_children_forest(4),
        s1 in arb_strategy(),
        s2 in arb_strategy(),
    ) {
        // Single root → no disambiguation needed → same result regardless of strategy
        let c1 = ForestConverter::new(s1);
        let c2 = ForestConverter::new(s2);
        let t1 = c1.to_tree(&forest, &input).unwrap();
        let t2 = c2.to_tree(&forest, &input).unwrap();
        prop_assert_eq!(t1.root_node().kind_id(), t2.root_node().kind_id());
        prop_assert_eq!(count_tree_nodes(t1.root_node()), count_tree_nodes(t2.root_node()));
    }
}

// ===========================================================================
// 6 – Ambiguity detection and multi-root handling
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    /// detect_ambiguity returns None for single-root forests.
    #[test]
    fn no_ambiguity_for_single_root(
        (forest, _input) in arb_single_leaf_forest()
    ) {
        let converter = ForestConverter::new(DisambiguationStrategy::First);
        prop_assert_eq!(converter.detect_ambiguity(&forest), None);
    }

    /// detect_ambiguity returns Some(n) for n-root forests.
    #[test]
    fn ambiguity_detected_for_multi_root(n_roots in 2usize..8) {
        let mut nodes = Vec::new();
        let mut roots = Vec::new();
        for i in 0..n_roots {
            nodes.push(ForestNode {
                symbol: SymbolId(i as u16 + 1),
                children: vec![],
                range: 0..1,
            });
            roots.push(ForestNodeId(i));
        }
        let forest = ParseForest { nodes, roots };
        let converter = ForestConverter::new(DisambiguationStrategy::First);
        prop_assert_eq!(converter.detect_ambiguity(&forest), Some(n_roots));
    }

    /// RejectAmbiguity strategy rejects multi-root forests.
    #[test]
    fn reject_ambiguity_multi_root(n_roots in 2usize..6) {
        let mut nodes = Vec::new();
        let mut roots = Vec::new();
        for i in 0..n_roots {
            nodes.push(ForestNode {
                symbol: SymbolId(i as u16 + 1),
                children: vec![],
                range: 0..1,
            });
            roots.push(ForestNodeId(i));
        }
        let forest = ParseForest { nodes, roots };
        let converter = ForestConverter::new(DisambiguationStrategy::RejectAmbiguity);
        let err = converter.to_tree(&forest, b"x").unwrap_err();
        match err {
            ConversionError::AmbiguousForest { count } => prop_assert_eq!(count, n_roots),
            other => prop_assert!(false, "expected AmbiguousForest, got {:?}", other),
        }
    }

    /// First strategy always selects the first root in multi-root forest.
    #[test]
    fn first_strategy_selects_first_root(n_roots in 2usize..6) {
        let mut nodes = Vec::new();
        let mut roots = Vec::new();
        for i in 0..n_roots {
            nodes.push(ForestNode {
                symbol: SymbolId(100 + i as u16),
                children: vec![],
                range: 0..1,
            });
            roots.push(ForestNodeId(i));
        }
        let forest = ParseForest { nodes, roots };
        let converter = ForestConverter::new(DisambiguationStrategy::First);
        let tree = converter.to_tree(&forest, b"x").unwrap();
        prop_assert_eq!(tree.root_node().kind_id(), 100);
    }
}

// ===========================================================================
// 7 – Invalid node references
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
fn invalid_child_node_id_returns_error() {
    let forest = ParseForest {
        nodes: vec![ForestNode {
            symbol: SymbolId(1),
            children: vec![ForestNodeId(99)],
            range: 0..1,
        }],
        roots: vec![ForestNodeId(0)],
    };
    let converter = ForestConverter::new(DisambiguationStrategy::First);
    let err = converter.to_tree(&forest, b"x").unwrap_err();
    assert!(matches!(
        err,
        ConversionError::InvalidNodeId { node_id: 99 }
    ));
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    /// Any out-of-bounds root ID fails with InvalidNodeId.
    #[test]
    fn out_of_bounds_root_fails(bad_id in 1usize..100) {
        let forest = ParseForest {
            nodes: vec![ForestNode {
                symbol: SymbolId(1),
                children: vec![],
                range: 0..1,
            }],
            roots: vec![ForestNodeId(bad_id)],
        };
        let converter = ForestConverter::new(DisambiguationStrategy::First);
        let err = converter.to_tree(&forest, b"x").unwrap_err();
        match err {
            ConversionError::InvalidNodeId { node_id } => prop_assert_eq!(node_id, bad_id),
            other => prop_assert!(false, "expected InvalidNodeId, got {:?}", other),
        }
    }
}

// ===========================================================================
// 8 – ConversionError display formatting
// ===========================================================================

#[test]
fn conversion_error_no_roots_display() {
    assert_eq!(
        ConversionError::NoRoots.to_string(),
        "Forest has no root nodes"
    );
}

#[test]
fn conversion_error_ambiguous_display() {
    let err = ConversionError::AmbiguousForest { count: 3 };
    assert_eq!(err.to_string(), "Ambiguous forest: 3 valid parses");
}

#[test]
fn conversion_error_invalid_forest_display() {
    let err = ConversionError::InvalidForest {
        reason: "broken".to_string(),
    };
    assert_eq!(err.to_string(), "Invalid forest structure: broken");
}

#[test]
fn conversion_error_invalid_node_display() {
    let err = ConversionError::InvalidNodeId { node_id: 7 };
    assert_eq!(err.to_string(), "Invalid node reference: 7");
}

#[test]
fn conversion_error_is_std_error() {
    let err: Box<dyn std::error::Error> = Box::new(ConversionError::NoRoots);
    assert_eq!(err.to_string(), "Forest has no root nodes");
}
