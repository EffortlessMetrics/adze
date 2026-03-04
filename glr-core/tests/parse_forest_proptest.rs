#![allow(clippy::needless_range_loop)]
//! Property-based tests for ParseForest in adze-glr-core.
//!
//! Run with: `cargo test -p adze-glr-core --test parse_forest_proptest`

use adze_glr_core::parse_forest::{
    ERROR_SYMBOL, ErrorMeta, ForestAlternative, ForestNode, ParseError,
};
use adze_glr_core::{ParseForest, ParseNode, ParseTree, SymbolId};
use adze_ir::builder::GrammarBuilder;
use proptest::prelude::*;
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn simple_grammar() -> adze_ir::Grammar {
    GrammarBuilder::new("test")
        .token("A", "a")
        .rule("expr", vec!["A"])
        .start("expr")
        .build()
}

fn empty_forest(grammar: adze_ir::Grammar) -> ParseForest {
    ParseForest {
        roots: Vec::new(),
        nodes: HashMap::new(),
        grammar,
        source: String::new(),
        next_node_id: 0,
    }
}

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

fn insert_node_with_meta(
    forest: &mut ParseForest,
    symbol: SymbolId,
    span: (usize, usize),
    children: Vec<usize>,
    meta: ErrorMeta,
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
            error_meta: meta,
        },
    );
    id
}

fn insert_ambiguous_node(
    forest: &mut ParseForest,
    symbol: SymbolId,
    span: (usize, usize),
    alternatives: Vec<Vec<usize>>,
) -> usize {
    let id = forest.next_node_id;
    forest.next_node_id += 1;
    forest.nodes.insert(
        id,
        ForestNode {
            id,
            symbol,
            span,
            alternatives: alternatives
                .into_iter()
                .map(|children| ForestAlternative { children })
                .collect(),
            error_meta: ErrorMeta::default(),
        },
    );
    id
}

// ---------------------------------------------------------------------------
// Strategies
// ---------------------------------------------------------------------------

fn arb_symbol_id() -> impl Strategy<Value = SymbolId> {
    (1u16..1000).prop_map(SymbolId)
}

fn arb_span(max_end: usize) -> impl Strategy<Value = (usize, usize)> {
    (0..max_end)
        .prop_flat_map(move |start| (Just(start), start..=max_end).prop_map(|(s, e)| (s, e)))
}

fn arb_error_meta() -> impl Strategy<Value = ErrorMeta> {
    (any::<bool>(), any::<bool>(), 0u32..100).prop_map(|(missing, is_error, cost)| ErrorMeta {
        missing,
        is_error,
        cost,
    })
}

// ===========================================================================
// 1. Forest node creation
// ===========================================================================

proptest! {
    #[test]
    fn node_id_equals_insertion_order(count in 1usize..50) {
        let mut forest = empty_forest(simple_grammar());
        for i in 0..count {
            let id = insert_node(&mut forest, SymbolId(1), (0, 1), vec![]);
            prop_assert_eq!(id, i);
        }
    }

    #[test]
    fn node_symbol_is_preserved(sym in 1u16..1000) {
        let mut forest = empty_forest(simple_grammar());
        let id = insert_node(&mut forest, SymbolId(sym), (0, 1), vec![]);
        prop_assert_eq!(forest.nodes[&id].symbol, SymbolId(sym));
    }

    #[test]
    fn node_span_is_preserved(start in 0usize..500, len in 0usize..500) {
        let end = start + len;
        let mut forest = empty_forest(simple_grammar());
        let id = insert_node(&mut forest, SymbolId(1), (start, end), vec![]);
        prop_assert_eq!(forest.nodes[&id].span, (start, end));
    }
}

// ===========================================================================
// 2. Forest root node
// ===========================================================================

proptest! {
    #[test]
    fn root_count_matches_pushes(count in 0usize..20) {
        let mut forest = empty_forest(simple_grammar());
        for _ in 0..count {
            let id = insert_node(&mut forest, SymbolId(1), (0, 1), vec![]);
            forest.roots.push(forest.nodes[&id].clone());
        }
        prop_assert_eq!(forest.roots.len(), count);
    }

    #[test]
    fn root_symbol_matches_inserted_node(sym in 1u16..500) {
        let mut forest = empty_forest(simple_grammar());
        let id = insert_node(&mut forest, SymbolId(sym), (0, 1), vec![]);
        forest.roots.push(forest.nodes[&id].clone());
        prop_assert_eq!(forest.roots[0].symbol, SymbolId(sym));
    }

    #[test]
    fn root_span_matches_inserted_node(start in 0usize..100, len in 1usize..100) {
        let end = start + len;
        let mut forest = empty_forest(simple_grammar());
        let id = insert_node(&mut forest, SymbolId(1), (start, end), vec![]);
        forest.roots.push(forest.nodes[&id].clone());
        prop_assert_eq!(forest.roots[0].span, (start, end));
    }
}

// ===========================================================================
// 3. Forest children
// ===========================================================================

proptest! {
    #[test]
    fn children_ids_are_preserved(child_count in 0usize..10) {
        let mut forest = empty_forest(simple_grammar());
        let children: Vec<usize> = (0..child_count)
            .map(|i| insert_node(&mut forest, SymbolId(1), (i, i + 1), vec![]))
            .collect();
        let parent = insert_node(&mut forest, SymbolId(2), (0, child_count), children.clone());
        prop_assert_eq!(&forest.nodes[&parent].alternatives[0].children, &children);
    }

    #[test]
    fn child_nodes_are_reachable(child_count in 1usize..8) {
        let mut forest = empty_forest(simple_grammar());
        let children: Vec<usize> = (0..child_count)
            .map(|i| insert_node(&mut forest, SymbolId(10), (i, i + 1), vec![]))
            .collect();
        let parent = insert_node(&mut forest, SymbolId(20), (0, child_count), children.clone());
        for &cid in &forest.nodes[&parent].alternatives[0].children {
            prop_assert!(forest.nodes.contains_key(&cid));
            prop_assert_eq!(forest.nodes[&cid].symbol, SymbolId(10));
        }
    }

    #[test]
    fn leaf_node_has_no_children(sym in 1u16..100) {
        let mut forest = empty_forest(simple_grammar());
        let id = insert_node(&mut forest, SymbolId(sym), (0, 1), vec![]);
        prop_assert!(forest.nodes[&id].alternatives[0].children.is_empty());
    }
}

// ===========================================================================
// 4. Forest ambiguity nodes
// ===========================================================================

proptest! {
    #[test]
    fn ambiguous_node_preserves_alternative_count(alt_count in 1usize..6) {
        let mut forest = empty_forest(simple_grammar());
        let alternatives: Vec<Vec<usize>> = (0..alt_count)
            .map(|_| {
                let child = insert_node(&mut forest, SymbolId(1), (0, 1), vec![]);
                vec![child]
            })
            .collect();
        let id = insert_ambiguous_node(&mut forest, SymbolId(5), (0, 1), alternatives.clone());
        prop_assert_eq!(forest.nodes[&id].alternatives.len(), alt_count);
    }

    #[test]
    fn ambiguous_node_children_per_alternative(alt_count in 1usize..5, children_per in 1usize..4) {
        let mut forest = empty_forest(simple_grammar());
        let mut alternatives = Vec::new();
        for _ in 0..alt_count {
            let children: Vec<usize> = (0..children_per)
                .map(|_| insert_node(&mut forest, SymbolId(1), (0, 1), vec![]))
                .collect();
            alternatives.push(children);
        }
        let id = insert_ambiguous_node(&mut forest, SymbolId(5), (0, 1), alternatives.clone());
        for i in 0..alt_count {
            prop_assert_eq!(forest.nodes[&id].alternatives[i].children.len(), children_per);
        }
    }

    #[test]
    fn single_alternative_is_not_ambiguous(sym in 1u16..100) {
        let mut forest = empty_forest(simple_grammar());
        let child = insert_node(&mut forest, SymbolId(1), (0, 1), vec![]);
        let id = insert_ambiguous_node(&mut forest, SymbolId(sym), (0, 1), vec![vec![child]]);
        prop_assert_eq!(forest.nodes[&id].alternatives.len(), 1);
    }
}

// ===========================================================================
// 5. Forest size
// ===========================================================================

proptest! {
    #[test]
    fn forest_node_count_equals_insertions(count in 0usize..50) {
        let mut forest = empty_forest(simple_grammar());
        for _ in 0..count {
            insert_node(&mut forest, SymbolId(1), (0, 1), vec![]);
        }
        prop_assert_eq!(forest.nodes.len(), count);
    }

    #[test]
    fn next_node_id_equals_total_insertions(regular in 0usize..20, errors in 0usize..20) {
        let mut forest = empty_forest(simple_grammar());
        for _ in 0..regular {
            insert_node(&mut forest, SymbolId(1), (0, 1), vec![]);
        }
        for _ in 0..errors {
            forest.push_error_chunk((0, 1));
        }
        prop_assert_eq!(forest.next_node_id, regular + errors);
    }

    #[test]
    fn mixed_insertions_total_size(regular in 0usize..15, errors in 0usize..15) {
        let mut forest = empty_forest(simple_grammar());
        for _ in 0..regular {
            insert_node(&mut forest, SymbolId(1), (0, 1), vec![]);
        }
        for _ in 0..errors {
            forest.push_error_chunk((0, 1));
        }
        prop_assert_eq!(forest.nodes.len(), regular + errors);
    }
}

// ===========================================================================
// 6. Forest depth
// ===========================================================================

fn compute_depth(forest: &ParseForest, node_id: usize) -> usize {
    let node = match forest.nodes.get(&node_id) {
        Some(n) => n,
        None => return 0,
    };
    let max_child_depth = node
        .alternatives
        .iter()
        .flat_map(|alt| &alt.children)
        .map(|&cid| compute_depth(forest, cid))
        .max()
        .unwrap_or(0);
    1 + max_child_depth
}

proptest! {
    #[test]
    fn linear_chain_depth_equals_length(depth in 1usize..20) {
        let mut forest = empty_forest(simple_grammar());
        let mut current = insert_node(&mut forest, SymbolId(1), (0, 1), vec![]);
        for _ in 1..depth {
            current = insert_node(&mut forest, SymbolId(1), (0, 1), vec![current]);
        }
        prop_assert_eq!(compute_depth(&forest, current), depth);
    }

    #[test]
    fn leaf_node_has_depth_one(sym in 1u16..100) {
        let mut forest = empty_forest(simple_grammar());
        let id = insert_node(&mut forest, SymbolId(sym), (0, 1), vec![]);
        prop_assert_eq!(compute_depth(&forest, id), 1);
    }

    #[test]
    fn wide_tree_has_depth_two(width in 1usize..15) {
        let mut forest = empty_forest(simple_grammar());
        let children: Vec<usize> = (0..width)
            .map(|i| insert_node(&mut forest, SymbolId(1), (i, i + 1), vec![]))
            .collect();
        let root = insert_node(&mut forest, SymbolId(2), (0, width), children);
        prop_assert_eq!(compute_depth(&forest, root), 2);
    }
}

// ===========================================================================
// 7. Empty forest
// ===========================================================================

proptest! {
    #[test]
    fn empty_forest_has_no_nodes(name_len in 1usize..20) {
        let name: String = (0..name_len).map(|_| 'x').collect();
        let grammar = GrammarBuilder::new(&name)
            .token("A", "a")
            .rule("expr", vec!["A"])
            .start("expr")
            .build();
        let forest = empty_forest(grammar);
        prop_assert!(forest.nodes.is_empty());
        prop_assert!(forest.roots.is_empty());
        prop_assert_eq!(forest.next_node_id, 0);
    }

    #[test]
    fn empty_forest_source_is_empty(_seed in 0u32..100) {
        let forest = empty_forest(simple_grammar());
        prop_assert!(forest.source.is_empty());
    }
}

// ===========================================================================
// 8. Forest with single derivation
// ===========================================================================

proptest! {
    #[test]
    fn single_derivation_leaf_only(sym in 1u16..500) {
        let mut forest = empty_forest(simple_grammar());
        let id = insert_node(&mut forest, SymbolId(sym), (0, 1), vec![]);
        forest.roots.push(forest.nodes[&id].clone());
        prop_assert_eq!(forest.roots.len(), 1);
        prop_assert_eq!(forest.nodes.len(), 1);
        prop_assert!(forest.nodes[&id].alternatives[0].children.is_empty());
    }

    #[test]
    fn single_derivation_parent_and_children(child_count in 1usize..8) {
        let mut forest = empty_forest(simple_grammar());
        let children: Vec<usize> = (0..child_count)
            .map(|i| insert_node(&mut forest, SymbolId(1), (i, i + 1), vec![]))
            .collect();
        let root = insert_node(&mut forest, SymbolId(100), (0, child_count), children.clone());
        forest.roots.push(forest.nodes[&root].clone());

        prop_assert_eq!(forest.roots.len(), 1);
        prop_assert_eq!(forest.roots[0].alternatives.len(), 1);
        prop_assert_eq!(forest.roots[0].alternatives[0].children.len(), child_count);
        prop_assert_eq!(forest.nodes.len(), child_count + 1);
    }

    #[test]
    fn single_derivation_is_complete(sym in 1u16..500) {
        let mut forest = empty_forest(simple_grammar());
        let id = insert_node(&mut forest, SymbolId(sym), (0, 1), vec![]);
        prop_assert!(forest.nodes[&id].is_complete());
    }
}

// ===========================================================================
// 9. Forest node metadata (ErrorMeta)
// ===========================================================================

proptest! {
    #[test]
    fn error_meta_default_is_clean(_seed in 0u32..100) {
        let meta = ErrorMeta::default();
        prop_assert!(!meta.missing);
        prop_assert!(!meta.is_error);
        prop_assert_eq!(meta.cost, 0);
    }

    #[test]
    fn error_meta_preserves_fields(missing in any::<bool>(), is_error in any::<bool>(), cost in 0u32..1000) {
        let meta = ErrorMeta { missing, is_error, cost };
        prop_assert_eq!(meta.missing, missing);
        prop_assert_eq!(meta.is_error, is_error);
        prop_assert_eq!(meta.cost, cost);
    }

    #[test]
    fn node_with_custom_meta_preserves_it(cost in 0u32..500) {
        let mut forest = empty_forest(simple_grammar());
        let meta = ErrorMeta { missing: true, is_error: false, cost };
        let id = insert_node_with_meta(&mut forest, SymbolId(1), (0, 1), vec![], meta);
        prop_assert!(forest.nodes[&id].error_meta.missing);
        prop_assert!(!forest.nodes[&id].error_meta.is_error);
        prop_assert_eq!(forest.nodes[&id].error_meta.cost, cost);
    }

    #[test]
    fn error_chunk_always_has_is_error_true(start in 0usize..100, len in 1usize..100) {
        let end = start + len;
        let mut forest = empty_forest(simple_grammar());
        let id = forest.push_error_chunk((start, end));
        prop_assert!(forest.nodes[&id].error_meta.is_error);
        prop_assert!(!forest.nodes[&id].error_meta.missing);
        prop_assert_eq!(forest.nodes[&id].error_meta.cost, 1);
    }

    #[test]
    fn error_chunk_symbol_is_error_symbol(start in 0usize..100, len in 1usize..100) {
        let end = start + len;
        let mut forest = empty_forest(simple_grammar());
        let id = forest.push_error_chunk((start, end));
        prop_assert_eq!(forest.nodes[&id].symbol, ERROR_SYMBOL);
    }

    #[test]
    fn error_chunk_span_is_preserved(start in 0usize..500, len in 0usize..500) {
        let end = start + len;
        let mut forest = empty_forest(simple_grammar());
        let id = forest.push_error_chunk((start, end));
        prop_assert_eq!(forest.nodes[&id].span, (start, end));
    }

    #[test]
    fn regular_node_default_meta_is_clean(sym in 1u16..500) {
        let mut forest = empty_forest(simple_grammar());
        let id = insert_node(&mut forest, SymbolId(sym), (0, 1), vec![]);
        let meta = &forest.nodes[&id].error_meta;
        prop_assert!(!meta.missing);
        prop_assert!(!meta.is_error);
        prop_assert_eq!(meta.cost, 0);
    }
}

// ===========================================================================
// 10. Additional structural properties
// ===========================================================================

proptest! {
    #[test]
    fn all_node_ids_are_unique(count in 2usize..30) {
        let mut forest = empty_forest(simple_grammar());
        let mut ids = Vec::new();
        for _ in 0..count {
            ids.push(insert_node(&mut forest, SymbolId(1), (0, 1), vec![]));
        }
        ids.sort();
        ids.dedup();
        prop_assert_eq!(ids.len(), count);
    }

    #[test]
    fn forest_clone_is_independent(count in 1usize..10) {
        let mut forest = empty_forest(simple_grammar());
        for _ in 0..count {
            insert_node(&mut forest, SymbolId(1), (0, 1), vec![]);
        }
        let cloned = forest.clone();
        insert_node(&mut forest, SymbolId(2), (0, 1), vec![]);
        prop_assert_eq!(cloned.nodes.len(), count);
        prop_assert_eq!(forest.nodes.len(), count + 1);
    }

    #[test]
    fn node_is_complete_with_alternatives(alt_count in 1usize..5) {
        let mut forest = empty_forest(simple_grammar());
        let alternatives: Vec<Vec<usize>> = (0..alt_count)
            .map(|_| vec![])
            .collect();
        let id = insert_ambiguous_node(&mut forest, SymbolId(1), (0, 1), alternatives);
        prop_assert!(forest.nodes[&id].is_complete());
    }

    #[test]
    fn parse_tree_preserves_source(src in "[a-z]{1,50}") {
        let tree = ParseTree {
            root: ParseNode {
                symbol: SymbolId(1),
                span: (0, src.len()),
                children: vec![],
            },
            source: src.clone(),
        };
        prop_assert_eq!(&tree.source, &src);
        prop_assert_eq!(tree.root.span, (0, src.len()));
    }

    #[test]
    fn parse_node_children_preserved(child_count in 0usize..10) {
        let children: Vec<ParseNode> = (0..child_count)
            .map(|i| ParseNode {
                symbol: SymbolId(i as u16 + 1),
                span: (i, i + 1),
                children: vec![],
            })
            .collect();
        let node = ParseNode {
            symbol: SymbolId(100),
            span: (0, child_count),
            children: children.clone(),
        };
        prop_assert_eq!(node.children.len(), child_count);
        for i in 0..child_count {
            prop_assert_eq!(node.children[i].symbol, SymbolId(i as u16 + 1));
        }
    }
}

// ===========================================================================
// 11. ParseError variants
// ===========================================================================

proptest! {
    #[test]
    fn parse_error_failed_preserves_message(msg in "[a-zA-Z0-9 ]{1,80}") {
        let err = ParseError::Failed(msg.clone());
        let displayed = format!("{err}");
        prop_assert!(displayed.contains(&msg));
    }
}
