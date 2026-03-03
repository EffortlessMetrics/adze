//! Comprehensive tests for the ParseForest API.

use adze_glr_core::parse_forest::{
    ERROR_SYMBOL, ErrorMeta, ForestAlternative, ForestNode, ParseError,
};
use adze_glr_core::{ParseForest, ParseNode, ParseTree, SymbolId};
use adze_ir::builder::GrammarBuilder;
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Build a minimal grammar with a single rule `expr -> A`.
fn simple_grammar() -> adze_ir::Grammar {
    GrammarBuilder::new("test")
        .token("A", "a")
        .rule("expr", vec!["A"])
        .start("expr")
        .build()
}

/// Build a grammar with two alternative productions for `expr`.
fn ambiguous_grammar() -> adze_ir::Grammar {
    GrammarBuilder::new("ambiguous")
        .token("A", "a")
        .token("B", "b")
        .rule("expr", vec!["A"])
        .rule("expr", vec!["B"])
        .start("expr")
        .build()
}

/// Create an empty `ParseForest` from the given grammar.
fn empty_forest(grammar: adze_ir::Grammar) -> ParseForest {
    ParseForest {
        roots: Vec::new(),
        nodes: HashMap::new(),
        grammar,
        source: String::new(),
        next_node_id: 0,
    }
}

/// Insert a plain (non-error) `ForestNode` and return its id.
fn insert_node(
    forest: &mut ParseForest,
    symbol: SymbolId,
    span: (usize, usize),
    children: Vec<usize>,
) -> usize {
    let id = forest.next_node_id;
    forest.next_node_id += 1;
    forest.nodes.insert(
        id,
        ForestNode {
            id,
            symbol,
            span,
            alternatives: vec![ForestAlternative { children }],
            error_meta: ErrorMeta::default(),
        },
    );
    id
}

// ===========================================================================
// 1. Creating ParseForest from a grammar
// ===========================================================================

#[test]
fn create_empty_forest() {
    let forest = empty_forest(simple_grammar());
    assert!(forest.roots.is_empty());
    assert!(forest.nodes.is_empty());
    assert_eq!(forest.next_node_id, 0);
}

#[test]
fn create_forest_preserves_grammar_name() {
    let forest = empty_forest(simple_grammar());
    assert_eq!(forest.grammar.name, "test");
}

#[test]
fn create_forest_with_source() {
    let grammar = simple_grammar();
    let forest = ParseForest {
        roots: Vec::new(),
        nodes: HashMap::new(),
        grammar,
        source: "hello".to_string(),
        next_node_id: 0,
    };
    assert_eq!(forest.source, "hello");
}

// ===========================================================================
// 2. Adding nodes
// ===========================================================================

#[test]
fn add_single_node() {
    let mut forest = empty_forest(simple_grammar());
    let id = insert_node(&mut forest, SymbolId(1), (0, 1), vec![]);
    assert_eq!(id, 0);
    assert_eq!(forest.nodes.len(), 1);
    assert_eq!(forest.next_node_id, 1);
}

#[test]
fn add_multiple_nodes_increments_ids() {
    let mut forest = empty_forest(simple_grammar());
    let a = insert_node(&mut forest, SymbolId(1), (0, 1), vec![]);
    let b = insert_node(&mut forest, SymbolId(2), (1, 2), vec![]);
    let c = insert_node(&mut forest, SymbolId(3), (0, 2), vec![a, b]);
    assert_eq!(a, 0);
    assert_eq!(b, 1);
    assert_eq!(c, 2);
    assert_eq!(forest.nodes.len(), 3);
}

#[test]
fn push_error_chunk_creates_node() {
    let mut forest = empty_forest(simple_grammar());
    let id = forest.push_error_chunk((5, 10));
    assert_eq!(id, 0);
    let node = &forest.nodes[&id];
    assert_eq!(node.symbol, ERROR_SYMBOL);
    assert_eq!(node.span, (5, 10));
    assert!(node.error_meta.is_error);
    assert!(!node.error_meta.missing);
    assert_eq!(node.error_meta.cost, 1);
}

#[test]
fn push_error_chunk_increments_id() {
    let mut forest = empty_forest(simple_grammar());
    let a = forest.push_error_chunk((0, 1));
    let b = forest.push_error_chunk((1, 3));
    assert_eq!(a, 0);
    assert_eq!(b, 1);
    assert_eq!(forest.next_node_id, 2);
}

// ===========================================================================
// 3. Forest traversal
// ===========================================================================

#[test]
fn traverse_parent_to_children() {
    let mut forest = empty_forest(simple_grammar());
    let leaf = insert_node(&mut forest, SymbolId(1), (0, 1), vec![]);
    let root = insert_node(&mut forest, SymbolId(2), (0, 1), vec![leaf]);
    forest.roots.push(forest.nodes[&root].clone());

    let root_node = &forest.nodes[&root];
    let child_ids = &root_node.alternatives[0].children;
    assert_eq!(child_ids.len(), 1);
    assert_eq!(child_ids[0], leaf);

    let child = &forest.nodes[&child_ids[0]];
    assert_eq!(child.symbol, SymbolId(1));
}

#[test]
fn traverse_deep_tree() {
    let mut forest = empty_forest(simple_grammar());
    let n0 = insert_node(&mut forest, SymbolId(1), (0, 1), vec![]);
    let n1 = insert_node(&mut forest, SymbolId(2), (0, 1), vec![n0]);
    let n2 = insert_node(&mut forest, SymbolId(3), (0, 1), vec![n1]);
    let n3 = insert_node(&mut forest, SymbolId(4), (0, 1), vec![n2]);

    // Walk from n3 down to n0
    let mut current = n3;
    for expected_sym in [4u16, 3, 2, 1] {
        let node = &forest.nodes[&current];
        assert_eq!(node.symbol, SymbolId(expected_sym));
        if !node.alternatives[0].children.is_empty() {
            current = node.alternatives[0].children[0];
        }
    }
}

#[test]
fn traverse_wide_tree() {
    let mut forest = empty_forest(simple_grammar());
    let children: Vec<usize> = (0..5)
        .map(|i| insert_node(&mut forest, SymbolId(10), (i, i + 1), vec![]))
        .collect();
    let root = insert_node(&mut forest, SymbolId(20), (0, 5), children.clone());

    let root_node = &forest.nodes[&root];
    assert_eq!(root_node.alternatives[0].children.len(), 5);
    assert_eq!(root_node.alternatives[0].children, children);
}

// ===========================================================================
// 4. Node IDs
// ===========================================================================

#[test]
fn node_ids_are_unique() {
    let mut forest = empty_forest(simple_grammar());
    let mut ids = Vec::new();
    for _ in 0..10 {
        ids.push(insert_node(&mut forest, SymbolId(1), (0, 1), vec![]));
    }
    ids.sort();
    ids.dedup();
    assert_eq!(ids.len(), 10);
}

#[test]
fn node_id_matches_map_key() {
    let mut forest = empty_forest(simple_grammar());
    let id = insert_node(&mut forest, SymbolId(1), (0, 1), vec![]);
    let node = &forest.nodes[&id];
    assert_eq!(node.id, id);
}

#[test]
fn error_chunk_id_matches_map_key() {
    let mut forest = empty_forest(simple_grammar());
    let id = forest.push_error_chunk((0, 3));
    let node = &forest.nodes[&id];
    assert_eq!(node.id, id);
}

#[test]
fn mixed_node_and_error_ids_never_collide() {
    let mut forest = empty_forest(simple_grammar());
    let a = insert_node(&mut forest, SymbolId(1), (0, 1), vec![]);
    let b = forest.push_error_chunk((1, 2));
    let c = insert_node(&mut forest, SymbolId(2), (2, 3), vec![]);
    let d = forest.push_error_chunk((3, 4));

    let mut ids = vec![a, b, c, d];
    ids.sort();
    ids.dedup();
    assert_eq!(ids.len(), 4);
}

// ===========================================================================
// 5. Error nodes
// ===========================================================================

#[test]
fn error_symbol_is_max_u16() {
    assert_eq!(ERROR_SYMBOL, SymbolId(u16::MAX));
}

#[test]
fn error_meta_default_is_benign() {
    let meta = ErrorMeta::default();
    assert!(!meta.missing);
    assert!(!meta.is_error);
    assert_eq!(meta.cost, 0);
}

#[test]
fn error_chunk_has_empty_children() {
    let mut forest = empty_forest(simple_grammar());
    let id = forest.push_error_chunk((0, 5));
    let node = &forest.nodes[&id];
    assert_eq!(node.alternatives.len(), 1);
    assert!(node.alternatives[0].children.is_empty());
}

#[cfg(feature = "test-api")]
#[test]
fn debug_error_stats_no_errors() {
    let mut forest = empty_forest(simple_grammar());
    insert_node(&mut forest, SymbolId(1), (0, 1), vec![]);
    let (has_error, missing, cost) = forest.debug_error_stats();
    assert!(!has_error);
    assert_eq!(missing, 0);
    assert_eq!(cost, 0);
}

#[cfg(feature = "test-api")]
#[test]
fn debug_error_stats_with_error_chunk() {
    let mut forest = empty_forest(simple_grammar());
    forest.push_error_chunk((0, 3));
    let (has_error, missing, cost) = forest.debug_error_stats();
    assert!(has_error);
    assert_eq!(missing, 0);
    assert_eq!(cost, 1);
}

#[cfg(feature = "test-api")]
#[test]
fn debug_error_stats_with_missing_terminal() {
    let mut forest = empty_forest(simple_grammar());
    let id = forest.next_node_id;
    forest.next_node_id += 1;
    forest.nodes.insert(
        id,
        ForestNode {
            id,
            symbol: SymbolId(1),
            span: (0, 0),
            alternatives: vec![ForestAlternative { children: vec![] }],
            error_meta: ErrorMeta {
                missing: true,
                is_error: false,
                cost: 2,
            },
        },
    );
    let (has_error, missing, cost) = forest.debug_error_stats();
    assert!(!has_error);
    assert_eq!(missing, 1);
    assert_eq!(cost, 2);
}

#[cfg(feature = "test-api")]
#[test]
fn debug_error_stats_accumulates_costs() {
    let mut forest = empty_forest(simple_grammar());
    forest.push_error_chunk((0, 1));
    forest.push_error_chunk((2, 4));
    let (has_error, _missing, cost) = forest.debug_error_stats();
    assert!(has_error);
    assert_eq!(cost, 2);
}

// ===========================================================================
// 6. ForestNode variants (alternatives)
// ===========================================================================

#[test]
fn forest_node_is_complete_with_alternative() {
    let node = ForestNode {
        id: 0,
        symbol: SymbolId(1),
        span: (0, 1),
        alternatives: vec![ForestAlternative { children: vec![] }],
        error_meta: ErrorMeta::default(),
    };
    assert!(node.is_complete());
}

#[test]
fn forest_node_incomplete_when_no_alternatives() {
    let node = ForestNode {
        id: 0,
        symbol: SymbolId(1),
        span: (0, 1),
        alternatives: vec![],
        error_meta: ErrorMeta::default(),
    };
    assert!(!node.is_complete());
}

#[test]
fn forest_node_multiple_alternatives() {
    let node = ForestNode {
        id: 0,
        symbol: SymbolId(1),
        span: (0, 3),
        alternatives: vec![
            ForestAlternative {
                children: vec![1, 2],
            },
            ForestAlternative {
                children: vec![3, 4],
            },
            ForestAlternative { children: vec![5] },
        ],
        error_meta: ErrorMeta::default(),
    };
    assert!(node.is_complete());
    assert_eq!(node.alternatives.len(), 3);
}

#[test]
fn forest_alternative_with_no_children_is_epsilon() {
    let alt = ForestAlternative { children: vec![] };
    assert!(alt.children.is_empty());
}

// ===========================================================================
// 7. Forest size
// ===========================================================================

#[test]
fn empty_forest_has_zero_nodes() {
    let forest = empty_forest(simple_grammar());
    assert_eq!(forest.nodes.len(), 0);
}

#[test]
fn forest_size_after_insertions() {
    let mut forest = empty_forest(simple_grammar());
    for _ in 0..7 {
        insert_node(&mut forest, SymbolId(1), (0, 1), vec![]);
    }
    assert_eq!(forest.nodes.len(), 7);
}

#[test]
fn forest_size_includes_error_chunks() {
    let mut forest = empty_forest(simple_grammar());
    insert_node(&mut forest, SymbolId(1), (0, 1), vec![]);
    forest.push_error_chunk((1, 3));
    insert_node(&mut forest, SymbolId(2), (3, 4), vec![]);
    assert_eq!(forest.nodes.len(), 3);
}

// ===========================================================================
// 8. Multiple roots (ambiguity)
// ===========================================================================

#[test]
fn single_root() {
    let mut forest = empty_forest(simple_grammar());
    let id = insert_node(&mut forest, SymbolId(1), (0, 1), vec![]);
    forest.roots.push(forest.nodes[&id].clone());
    assert_eq!(forest.roots.len(), 1);
}

#[test]
fn multiple_roots_represent_ambiguity() {
    let mut forest = empty_forest(ambiguous_grammar());
    let r1 = insert_node(&mut forest, SymbolId(1), (0, 1), vec![]);
    let r2 = insert_node(&mut forest, SymbolId(2), (0, 1), vec![]);
    forest.roots.push(forest.nodes[&r1].clone());
    forest.roots.push(forest.nodes[&r2].clone());
    assert_eq!(forest.roots.len(), 2);
    assert_ne!(forest.roots[0].symbol, forest.roots[1].symbol);
}

#[test]
fn roots_with_different_spans() {
    let mut forest = empty_forest(simple_grammar());
    let r1 = insert_node(&mut forest, SymbolId(1), (0, 3), vec![]);
    let r2 = insert_node(&mut forest, SymbolId(1), (0, 5), vec![]);
    forest.roots.push(forest.nodes[&r1].clone());
    forest.roots.push(forest.nodes[&r2].clone());
    assert_eq!(forest.roots[0].span, (0, 3));
    assert_eq!(forest.roots[1].span, (0, 5));
}

// ===========================================================================
// 9. ParseTree and ParseNode
// ===========================================================================

#[test]
fn parse_tree_stores_source() {
    let tree = ParseTree {
        root: ParseNode {
            symbol: SymbolId(1),
            span: (0, 5),
            children: vec![],
        },
        source: "hello".to_string(),
    };
    assert_eq!(tree.source, "hello");
    assert_eq!(tree.root.span, (0, 5));
}

#[test]
fn parse_node_with_children() {
    let node = ParseNode {
        symbol: SymbolId(1),
        span: (0, 5),
        children: vec![
            ParseNode {
                symbol: SymbolId(2),
                span: (0, 2),
                children: vec![],
            },
            ParseNode {
                symbol: SymbolId(3),
                span: (2, 5),
                children: vec![],
            },
        ],
    };
    assert_eq!(node.children.len(), 2);
    assert_eq!(node.children[0].symbol, SymbolId(2));
    assert_eq!(node.children[1].symbol, SymbolId(3));
}

// ===========================================================================
// 10. ParseError variants
// ===========================================================================

#[test]
fn parse_error_incomplete_display() {
    let err = ParseError::Incomplete;
    assert_eq!(err.to_string(), "Incomplete parse");
}

#[test]
fn parse_error_failed_display() {
    let err = ParseError::Failed("unexpected token".to_string());
    assert!(err.to_string().contains("unexpected token"));
}

#[test]
fn parse_error_unknown_display() {
    let err = ParseError::Unknown;
    assert_eq!(err.to_string(), "Unknown error");
}

// ===========================================================================
// 11. Clone and Debug
// ===========================================================================

#[test]
fn forest_is_cloneable() {
    let mut forest = empty_forest(simple_grammar());
    insert_node(&mut forest, SymbolId(1), (0, 1), vec![]);
    let cloned = forest.clone();
    assert_eq!(cloned.nodes.len(), forest.nodes.len());
    assert_eq!(cloned.next_node_id, forest.next_node_id);
}

#[test]
fn forest_node_is_debuggable() {
    let node = ForestNode {
        id: 42,
        symbol: SymbolId(7),
        span: (0, 10),
        alternatives: vec![],
        error_meta: ErrorMeta::default(),
    };
    let debug_str = format!("{node:?}");
    assert!(debug_str.contains("42"));
    assert!(debug_str.contains("SymbolId"));
}

#[test]
fn parse_error_is_cloneable() {
    let err = ParseError::Failed("oops".to_string());
    let cloned = err.clone();
    assert_eq!(cloned.to_string(), "Parse failed: oops");
}
