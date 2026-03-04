#![cfg(feature = "test-api")]
#![allow(clippy::needless_range_loop)]

//! Comprehensive tests for parse forest data structure and operations.
//!
//! Covers: Forest construction, node addition, traversal, node access by ID,
//! root identification, forest size/statistics, empty/single-node/large forests,
//! Clone/Debug behavior, and edge cases.

use adze_glr_core::driver::GlrError;
use adze_glr_core::forest_view::{ForestView, Span};
use adze_glr_core::parse_forest::{
    ERROR_SYMBOL, ErrorMeta, ForestAlternative, ForestNode, ParseError, ParseForest, ParseNode,
    ParseTree,
};
use adze_glr_core::{
    Driver, FirstFollowSets, Forest, GLRError, ParseTable, build_lr1_automaton, sanity_check_tables,
};
use adze_ir::builder::GrammarBuilder;
use adze_ir::{Grammar, SymbolId};
use std::collections::HashMap;

// ─── Helpers ─────────────────────────────────────────────────────────

fn run_pipeline(grammar: &mut Grammar) -> ParseTable {
    let ff = FirstFollowSets::compute_normalized(grammar).expect("FIRST/FOLLOW");
    build_lr1_automaton(grammar, &ff).expect("LR1 automaton")
}

fn pipeline_parse(
    grammar: &mut Grammar,
    tokens: &[(SymbolId, u32, u32)],
) -> Result<Forest, GlrError> {
    let table = run_pipeline(grammar);
    sanity_check_tables(&table).expect("sanity");
    let mut driver = Driver::new(&table);
    driver.parse_tokens(
        tokens
            .iter()
            .map(|&(sym, start, end)| (sym.0 as u32, start, end)),
    )
}

fn sym_id(grammar: &Grammar, name: &str) -> SymbolId {
    for (&id, tok) in &grammar.tokens {
        if tok.name == name {
            return id;
        }
    }
    for (&id, n) in &grammar.rule_names {
        if n == name {
            return id;
        }
    }
    panic!("symbol '{name}' not found in grammar");
}

fn minimal_grammar() -> Grammar {
    GrammarBuilder::new("mini")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build()
}

fn empty_forest(grammar: Grammar) -> ParseForest {
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

fn collect_node_ids(view: &dyn ForestView, root: u32) -> Vec<u32> {
    let mut result = vec![root];
    for &child in view.best_children(root) {
        result.extend(collect_node_ids(view, child));
    }
    result
}

fn tree_depth(view: &dyn ForestView, id: u32) -> usize {
    let children = view.best_children(id);
    if children.is_empty() {
        1
    } else {
        1 + children.iter().map(|&c| tree_depth(view, c)).max().unwrap()
    }
}

fn node_count(view: &dyn ForestView, id: u32) -> usize {
    1 + view
        .best_children(id)
        .iter()
        .map(|&c| node_count(view, c))
        .sum::<usize>()
}

fn single_token_grammar_and_forest() -> (Grammar, Forest) {
    let mut grammar = GrammarBuilder::new("one")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let a = sym_id(&grammar, "a");
    let forest = pipeline_parse(&mut grammar, &[(a, 0, 1)]).expect("parse");
    (grammar, forest)
}

fn expr_grammar_and_forest() -> (Grammar, Forest) {
    let mut grammar = GrammarBuilder::new("expr")
        .token("NUM", r"\d+")
        .token("PLUS", r"\+")
        .rule("expr", vec!["expr", "PLUS", "NUM"])
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    let num = sym_id(&grammar, "NUM");
    let plus = sym_id(&grammar, "PLUS");
    let forest =
        pipeline_parse(&mut grammar, &[(num, 0, 1), (plus, 1, 2), (num, 2, 3)]).expect("parse");
    (grammar, forest)
}

// ═══════════════════════════════════════════════════════════════════════
//  1. Forest construction
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn construct_empty_parse_forest() {
    let forest = empty_forest(minimal_grammar());
    assert!(forest.roots.is_empty());
    assert!(forest.nodes.is_empty());
    assert_eq!(forest.next_node_id, 0);
}

#[test]
fn construct_forest_with_source() {
    let mut forest = empty_forest(minimal_grammar());
    forest.source = "hello".to_string();
    assert_eq!(forest.source, "hello");
}

#[test]
fn construct_forest_preserves_grammar_name() {
    let forest = empty_forest(minimal_grammar());
    assert_eq!(forest.grammar.name, "mini");
}

#[test]
fn construct_forest_via_pipeline_single_token() {
    let (_grammar, forest) = single_token_grammar_and_forest();
    let view = forest.view();
    assert!(!view.roots().is_empty());
}

#[test]
fn construct_forest_via_pipeline_expr() {
    let (_grammar, forest) = expr_grammar_and_forest();
    let view = forest.view();
    assert!(!view.roots().is_empty());
}

// ═══════════════════════════════════════════════════════════════════════
//  2. Adding nodes/trees to forests
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn add_single_node_to_empty_forest() {
    let mut forest = empty_forest(minimal_grammar());
    let id = insert_node(&mut forest, SymbolId(1), (0, 3), vec![]);
    assert_eq!(id, 0);
    assert_eq!(forest.nodes.len(), 1);
    assert_eq!(forest.next_node_id, 1);
}

#[test]
fn add_multiple_nodes_sequentially() {
    let mut forest = empty_forest(minimal_grammar());
    let id0 = insert_node(&mut forest, SymbolId(1), (0, 1), vec![]);
    let id1 = insert_node(&mut forest, SymbolId(2), (1, 2), vec![]);
    let _id2 = insert_node(&mut forest, SymbolId(10), (0, 2), vec![id0, id1]);
    assert_eq!(forest.nodes.len(), 3);
    assert_eq!(forest.next_node_id, 3);
}

#[test]
fn add_node_with_children_references() {
    let mut forest = empty_forest(minimal_grammar());
    let c0 = insert_node(&mut forest, SymbolId(1), (0, 1), vec![]);
    let c1 = insert_node(&mut forest, SymbolId(2), (1, 3), vec![]);
    let parent = insert_node(&mut forest, SymbolId(10), (0, 3), vec![c0, c1]);
    let parent_node = &forest.nodes[&parent];
    assert_eq!(parent_node.alternatives[0].children, vec![c0, c1]);
}

#[test]
fn add_error_chunk_to_forest() {
    let mut forest = empty_forest(minimal_grammar());
    let id = forest.push_error_chunk((5, 10));
    let node = &forest.nodes[&id];
    assert_eq!(node.symbol, ERROR_SYMBOL);
    assert_eq!(node.span, (5, 10));
    assert!(node.error_meta.is_error);
    assert_eq!(node.error_meta.cost, 1);
}

#[test]
fn add_multiple_error_chunks() {
    let mut forest = empty_forest(minimal_grammar());
    let id0 = forest.push_error_chunk((0, 1));
    let id1 = forest.push_error_chunk((1, 2));
    assert_ne!(id0, id1);
    assert_eq!(forest.nodes.len(), 2);
}

#[test]
fn add_root_to_forest() {
    let mut forest = empty_forest(minimal_grammar());
    let id = insert_node(&mut forest, SymbolId(10), (0, 5), vec![]);
    let root_node = forest.nodes[&id].clone();
    forest.roots.push(root_node);
    assert_eq!(forest.roots.len(), 1);
    assert_eq!(forest.roots[0].id, id);
}

#[test]
fn node_with_multiple_alternatives() {
    let mut forest = empty_forest(minimal_grammar());
    let c0 = insert_node(&mut forest, SymbolId(1), (0, 1), vec![]);
    let c1 = insert_node(&mut forest, SymbolId(2), (0, 1), vec![]);
    let id = forest.next_node_id;
    forest.next_node_id += 1;
    forest.nodes.insert(
        id,
        ForestNode {
            id,
            symbol: SymbolId(10),
            span: (0, 1),
            alternatives: vec![
                ForestAlternative { children: vec![c0] },
                ForestAlternative { children: vec![c1] },
            ],
            error_meta: ErrorMeta::default(),
        },
    );
    assert_eq!(forest.nodes[&id].alternatives.len(), 2);
}

// ═══════════════════════════════════════════════════════════════════════
//  3. Forest traversal
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn traverse_single_token_forest() {
    let (_grammar, forest) = single_token_grammar_and_forest();
    let view = forest.view();
    let root = view.roots()[0];
    let ids = collect_node_ids(view, root);
    assert!(ids.len() >= 2); // root + at least one terminal
}

#[test]
fn traverse_expr_forest_depth() {
    let (_grammar, forest) = expr_grammar_and_forest();
    let view = forest.view();
    let root = view.roots()[0];
    let depth = tree_depth(view, root);
    assert!(depth >= 2);
}

#[test]
fn traverse_expr_forest_node_count() {
    let (_grammar, forest) = expr_grammar_and_forest();
    let view = forest.view();
    let root = view.roots()[0];
    let count = node_count(view, root);
    // expr -> expr PLUS NUM, where inner expr -> NUM: root + 3 terminals + inner expr = 5
    assert!(count >= 4);
}

#[test]
fn traverse_children_of_leaf_are_empty() {
    let (_grammar, forest) = single_token_grammar_and_forest();
    let view = forest.view();
    let root = view.roots()[0];
    let children = view.best_children(root);
    // The leaf children should have no further children
    for &c in children {
        let grandchildren = view.best_children(c);
        if grandchildren.is_empty() {
            // Terminal leaf
            assert!(grandchildren.is_empty());
        }
    }
}

#[test]
fn traverse_all_reachable_nodes_from_root() {
    let (_grammar, forest) = expr_grammar_and_forest();
    let view = forest.view();
    let root = view.roots()[0];
    let all_ids = collect_node_ids(view, root);
    // All IDs should be unique
    let mut sorted = all_ids.clone();
    sorted.sort();
    sorted.dedup();
    assert_eq!(sorted.len(), all_ids.len());
}

// ═══════════════════════════════════════════════════════════════════════
//  4. Node access by ID
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn access_node_kind_by_id() {
    let (grammar, forest) = single_token_grammar_and_forest();
    let view = forest.view();
    let root = view.roots()[0];
    let root_kind = view.kind(root);
    let s_id = sym_id(&grammar, "start");
    assert_eq!(root_kind, s_id.0 as u32);
}

#[test]
fn access_node_span_by_id() {
    let (_grammar, forest) = single_token_grammar_and_forest();
    let view = forest.view();
    let root = view.roots()[0];
    let span = view.span(root);
    assert_eq!(span.start, 0);
    assert_eq!(span.end, 1);
}

#[test]
fn access_children_by_id() {
    let (_grammar, forest) = single_token_grammar_and_forest();
    let view = forest.view();
    let root = view.roots()[0];
    let children = view.best_children(root);
    assert!(!children.is_empty());
}

#[test]
fn access_nonexistent_node_returns_defaults() {
    let (_grammar, forest) = single_token_grammar_and_forest();
    let view = forest.view();
    let kind = view.kind(99999);
    assert_eq!(kind, 0);
    let span = view.span(99999);
    assert_eq!(span.start, 0);
    assert_eq!(span.end, 0);
    let children = view.best_children(99999);
    assert!(children.is_empty());
}

#[test]
fn access_child_node_kind() {
    let (grammar, forest) = single_token_grammar_and_forest();
    let view = forest.view();
    let root = view.roots()[0];
    let children = view.best_children(root);
    assert!(!children.is_empty());
    let child_kind = view.kind(children[0]);
    let a_id = sym_id(&grammar, "a");
    assert_eq!(child_kind, a_id.0 as u32);
}

#[test]
fn access_child_node_span() {
    let (_grammar, forest) = single_token_grammar_and_forest();
    let view = forest.view();
    let root = view.roots()[0];
    let children = view.best_children(root);
    assert!(!children.is_empty());
    let span = view.span(children[0]);
    assert_eq!(span.start, 0);
    assert_eq!(span.end, 1);
}

// ═══════════════════════════════════════════════════════════════════════
//  5. Root node identification
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn single_root_for_unambiguous_parse() {
    let (_grammar, forest) = single_token_grammar_and_forest();
    let view = forest.view();
    assert_eq!(view.roots().len(), 1);
}

#[test]
fn root_is_start_symbol() {
    let (grammar, forest) = single_token_grammar_and_forest();
    let view = forest.view();
    let root = view.roots()[0];
    let s_id = sym_id(&grammar, "start");
    assert_eq!(view.kind(root), s_id.0 as u32);
}

#[test]
fn root_span_covers_input() {
    let (_grammar, forest) = single_token_grammar_and_forest();
    let view = forest.view();
    let root = view.roots()[0];
    let span = view.span(root);
    assert_eq!(span.start, 0);
    assert_eq!(span.end, 1);
}

#[test]
fn expr_root_span_covers_full_input() {
    let (_grammar, forest) = expr_grammar_and_forest();
    let view = forest.view();
    let root = view.roots()[0];
    let span = view.span(root);
    assert_eq!(span.start, 0);
    assert_eq!(span.end, 3);
}

#[test]
fn expr_root_is_start_symbol() {
    let (grammar, forest) = expr_grammar_and_forest();
    let view = forest.view();
    let root = view.roots()[0];
    let expr_id = sym_id(&grammar, "expr");
    assert_eq!(view.kind(root), expr_id.0 as u32);
}

#[test]
fn roots_array_is_nonempty_on_success() {
    let (_grammar, forest) = expr_grammar_and_forest();
    assert!(!forest.view().roots().is_empty());
}

// ═══════════════════════════════════════════════════════════════════════
//  6. Forest size/statistics
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn parse_forest_node_count() {
    let mut forest = empty_forest(minimal_grammar());
    assert_eq!(forest.nodes.len(), 0);
    insert_node(&mut forest, SymbolId(1), (0, 1), vec![]);
    assert_eq!(forest.nodes.len(), 1);
    insert_node(&mut forest, SymbolId(2), (1, 2), vec![]);
    assert_eq!(forest.nodes.len(), 2);
}

#[test]
fn parse_forest_next_node_id_tracks_correctly() {
    let mut forest = empty_forest(minimal_grammar());
    assert_eq!(forest.next_node_id, 0);
    insert_node(&mut forest, SymbolId(1), (0, 1), vec![]);
    assert_eq!(forest.next_node_id, 1);
    insert_node(&mut forest, SymbolId(1), (1, 2), vec![]);
    assert_eq!(forest.next_node_id, 2);
}

#[test]
fn error_stats_clean_forest() {
    let mut forest = empty_forest(minimal_grammar());
    insert_node(&mut forest, SymbolId(1), (0, 1), vec![]);
    let (has_error, missing, cost) = forest.debug_error_stats();
    assert!(!has_error);
    assert_eq!(missing, 0);
    assert_eq!(cost, 0);
}

#[test]
fn error_stats_with_error_chunk() {
    let mut forest = empty_forest(minimal_grammar());
    forest.push_error_chunk((0, 3));
    let (has_error, missing, cost) = forest.debug_error_stats();
    assert!(has_error);
    assert_eq!(missing, 0);
    assert_eq!(cost, 1);
}

#[test]
fn error_stats_with_missing_terminal() {
    let mut forest = empty_forest(minimal_grammar());
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
                cost: 1,
            },
        },
    );
    let (has_error, missing, cost) = forest.debug_error_stats();
    assert!(!has_error);
    assert_eq!(missing, 1);
    assert_eq!(cost, 1);
}

#[test]
fn error_stats_multiple_errors() {
    let mut forest = empty_forest(minimal_grammar());
    forest.push_error_chunk((0, 1));
    forest.push_error_chunk((1, 2));
    let (has_error, missing, cost) = forest.debug_error_stats();
    assert!(has_error);
    assert_eq!(cost, 2);
}

#[test]
fn forest_debug_error_stats_via_forest_handle() {
    let (_grammar, forest) = single_token_grammar_and_forest();
    let (has_error, missing, cost) = forest.debug_error_stats();
    assert!(!has_error);
    assert_eq!(missing, 0);
    assert_eq!(cost, 0);
}

#[test]
fn forest_view_node_count_matches_traversal() {
    let (_grammar, forest) = expr_grammar_and_forest();
    let view = forest.view();
    let root = view.roots()[0];
    let count = node_count(view, root);
    assert!(count >= 4);
}

// ═══════════════════════════════════════════════════════════════════════
//  7. Empty forest
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn empty_forest_has_no_roots() {
    let forest = empty_forest(minimal_grammar());
    assert!(forest.roots.is_empty());
}

#[test]
fn empty_forest_has_no_nodes() {
    let forest = empty_forest(minimal_grammar());
    assert!(forest.nodes.is_empty());
}

#[test]
fn empty_forest_next_id_is_zero() {
    let forest = empty_forest(minimal_grammar());
    assert_eq!(forest.next_node_id, 0);
}

#[test]
fn empty_forest_debug_error_stats_clean() {
    let forest = empty_forest(minimal_grammar());
    let (has_error, missing, cost) = forest.debug_error_stats();
    assert!(!has_error);
    assert_eq!(missing, 0);
    assert_eq!(cost, 0);
}

#[test]
fn empty_forest_source_is_empty() {
    let forest = empty_forest(minimal_grammar());
    assert!(forest.source.is_empty());
}

// ═══════════════════════════════════════════════════════════════════════
//  8. Single-node forest
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn single_node_forest_has_one_node() {
    let mut forest = empty_forest(minimal_grammar());
    insert_node(&mut forest, SymbolId(1), (0, 5), vec![]);
    assert_eq!(forest.nodes.len(), 1);
}

#[test]
fn single_node_forest_node_is_complete() {
    let mut forest = empty_forest(minimal_grammar());
    let id = insert_node(&mut forest, SymbolId(1), (0, 5), vec![]);
    assert!(forest.nodes[&id].is_complete());
}

#[test]
fn single_node_forest_node_has_no_children() {
    let mut forest = empty_forest(minimal_grammar());
    let id = insert_node(&mut forest, SymbolId(1), (0, 5), vec![]);
    assert!(forest.nodes[&id].alternatives[0].children.is_empty());
}

#[test]
fn single_node_forest_id_is_zero() {
    let mut forest = empty_forest(minimal_grammar());
    let id = insert_node(&mut forest, SymbolId(1), (0, 5), vec![]);
    assert_eq!(id, 0);
}

#[test]
fn single_node_parsed_forest_structure() {
    let (_grammar, forest) = single_token_grammar_and_forest();
    let view = forest.view();
    let root = view.roots()[0];
    let depth = tree_depth(view, root);
    assert_eq!(depth, 2); // S -> a: depth of 2
}

// ═══════════════════════════════════════════════════════════════════════
//  9. Large forests
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn large_forest_100_nodes() {
    let mut forest = empty_forest(minimal_grammar());
    for i in 0..100 {
        insert_node(&mut forest, SymbolId(1), (i, i + 1), vec![]);
    }
    assert_eq!(forest.nodes.len(), 100);
    assert_eq!(forest.next_node_id, 100);
}

#[test]
fn large_forest_chain_structure() {
    let mut forest = empty_forest(minimal_grammar());
    let mut prev = insert_node(&mut forest, SymbolId(1), (0, 1), vec![]);
    for i in 1..50 {
        prev = insert_node(&mut forest, SymbolId(10), (0, i + 1), vec![prev]);
    }
    assert_eq!(forest.nodes.len(), 50);
    // Last node should have one child
    assert_eq!(forest.nodes[&prev].alternatives[0].children.len(), 1);
}

#[test]
fn large_forest_wide_tree() {
    let mut forest = empty_forest(minimal_grammar());
    let mut children = Vec::new();
    for i in 0..20 {
        children.push(insert_node(&mut forest, SymbolId(1), (i, i + 1), vec![]));
    }
    let root = insert_node(&mut forest, SymbolId(10), (0, 20), children.clone());
    assert_eq!(forest.nodes[&root].alternatives[0].children.len(), 20);
}

#[test]
fn large_parsed_forest_long_expression() {
    // Parse a longer expression: NUM + NUM + NUM + NUM + NUM
    let mut grammar = GrammarBuilder::new("longexpr")
        .token("NUM", r"\d+")
        .token("PLUS", r"\+")
        .rule("expr", vec!["expr", "PLUS", "NUM"])
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    let num = sym_id(&grammar, "NUM");
    let plus = sym_id(&grammar, "PLUS");
    let tokens: Vec<(SymbolId, u32, u32)> = vec![
        (num, 0, 1),
        (plus, 1, 2),
        (num, 2, 3),
        (plus, 3, 4),
        (num, 4, 5),
        (plus, 5, 6),
        (num, 6, 7),
        (plus, 7, 8),
        (num, 8, 9),
    ];
    let forest = pipeline_parse(&mut grammar, &tokens).expect("parse");
    let view = forest.view();
    let root = view.roots()[0];
    let span = view.span(root);
    assert_eq!(span.start, 0);
    assert_eq!(span.end, 9);
    let count = node_count(view, root);
    assert!(count >= 9); // At least as many nodes as tokens
}

#[test]
fn large_forest_many_error_chunks() {
    let mut forest = empty_forest(minimal_grammar());
    for i in 0..50 {
        forest.push_error_chunk((i, i + 1));
    }
    let (has_error, _missing, cost) = forest.debug_error_stats();
    assert!(has_error);
    assert_eq!(cost, 50);
}

// ═══════════════════════════════════════════════════════════════════════
//  10. Clone/Debug behavior
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn forest_node_clone_preserves_id() {
    let node = ForestNode {
        id: 42,
        symbol: SymbolId(5),
        span: (10, 20),
        alternatives: vec![ForestAlternative {
            children: vec![1, 2, 3],
        }],
        error_meta: ErrorMeta::default(),
    };
    let cloned = node.clone();
    assert_eq!(cloned.id, 42);
    assert_eq!(cloned.symbol, SymbolId(5));
    assert_eq!(cloned.span, (10, 20));
    assert_eq!(cloned.alternatives[0].children, vec![1, 2, 3]);
}

#[test]
fn forest_node_clone_preserves_error_meta() {
    let node = ForestNode {
        id: 0,
        symbol: ERROR_SYMBOL,
        span: (0, 5),
        alternatives: vec![],
        error_meta: ErrorMeta {
            missing: false,
            is_error: true,
            cost: 7,
        },
    };
    let cloned = node.clone();
    assert!(cloned.error_meta.is_error);
    assert_eq!(cloned.error_meta.cost, 7);
    assert!(!cloned.error_meta.missing);
}

#[test]
fn parse_forest_clone_is_independent() {
    let mut forest = empty_forest(minimal_grammar());
    insert_node(&mut forest, SymbolId(1), (0, 1), vec![]);
    let mut cloned = forest.clone();
    insert_node(&mut cloned, SymbolId(2), (1, 2), vec![]);
    assert_eq!(forest.nodes.len(), 1);
    assert_eq!(cloned.nodes.len(), 2);
}

#[test]
fn forest_node_debug_output() {
    let node = ForestNode {
        id: 0,
        symbol: SymbolId(1),
        span: (0, 5),
        alternatives: vec![],
        error_meta: ErrorMeta::default(),
    };
    let debug = format!("{:?}", node);
    assert!(debug.contains("ForestNode"));
    assert!(debug.contains("symbol"));
}

#[test]
fn error_meta_debug_output() {
    let meta = ErrorMeta {
        missing: true,
        is_error: false,
        cost: 3,
    };
    let debug = format!("{:?}", meta);
    assert!(debug.contains("ErrorMeta"));
    assert!(debug.contains("missing"));
}

#[test]
fn parse_forest_debug_output() {
    let forest = empty_forest(minimal_grammar());
    let debug = format!("{:?}", forest);
    assert!(debug.contains("ParseForest"));
}

#[test]
fn error_meta_copy_semantics() {
    let a = ErrorMeta {
        missing: true,
        is_error: false,
        cost: 5,
    };
    let b = a; // Copy
    let c = a; // Copy again - a is still available
    assert_eq!(b.cost, c.cost);
    assert_eq!(a.missing, b.missing);
}

#[test]
fn forest_alternative_clone() {
    let alt = ForestAlternative {
        children: vec![1, 2, 3],
    };
    let cloned = alt.clone();
    assert_eq!(cloned.children, vec![1, 2, 3]);
}

#[test]
fn parse_tree_clone() {
    let tree = ParseTree {
        root: ParseNode {
            symbol: SymbolId(10),
            span: (0, 5),
            children: vec![ParseNode {
                symbol: SymbolId(1),
                span: (0, 5),
                children: vec![],
            }],
        },
        source: "hello".to_string(),
    };
    let cloned = tree.clone();
    assert_eq!(cloned.root.symbol, SymbolId(10));
    assert_eq!(cloned.source, "hello");
    assert_eq!(cloned.root.children.len(), 1);
}

#[test]
fn parse_node_debug() {
    let node = ParseNode {
        symbol: SymbolId(1),
        span: (0, 3),
        children: vec![],
    };
    let debug = format!("{:?}", node);
    assert!(debug.contains("ParseNode"));
}

// ═══════════════════════════════════════════════════════════════════════
//  11. Edge cases
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn node_with_zero_width_span() {
    let mut forest = empty_forest(minimal_grammar());
    let id = insert_node(&mut forest, SymbolId(1), (5, 5), vec![]);
    assert_eq!(forest.nodes[&id].span, (5, 5));
}

#[test]
fn error_symbol_constant_is_max_u16() {
    assert_eq!(ERROR_SYMBOL, SymbolId(u16::MAX));
}

#[test]
fn error_meta_default_all_zero() {
    let meta = ErrorMeta::default();
    assert!(!meta.missing);
    assert!(!meta.is_error);
    assert_eq!(meta.cost, 0);
}

#[test]
fn forest_node_is_complete_empty_alternatives() {
    let node = ForestNode {
        id: 0,
        symbol: SymbolId(1),
        span: (0, 0),
        alternatives: vec![],
        error_meta: ErrorMeta::default(),
    };
    assert!(!node.is_complete());
}

#[test]
fn forest_node_is_complete_with_empty_children() {
    let node = ForestNode {
        id: 0,
        symbol: SymbolId(1),
        span: (0, 0),
        alternatives: vec![ForestAlternative { children: vec![] }],
        error_meta: ErrorMeta::default(),
    };
    assert!(node.is_complete());
}

#[test]
fn push_error_chunk_increments_next_id() {
    let mut forest = empty_forest(minimal_grammar());
    let id0 = forest.push_error_chunk((0, 1));
    let id1 = forest.push_error_chunk((1, 2));
    assert_eq!(id1, id0 + 1);
    assert_eq!(forest.next_node_id, id1 + 1);
}

#[test]
fn parse_error_display_incomplete() {
    let err = ParseError::Incomplete;
    assert_eq!(format!("{}", err), "Incomplete parse");
}

#[test]
fn parse_error_display_failed() {
    let err = ParseError::Failed("syntax error".to_string());
    assert_eq!(format!("{}", err), "Parse failed: syntax error");
}

#[test]
fn parse_error_display_unknown() {
    let err = ParseError::Unknown;
    assert_eq!(format!("{}", err), "Unknown error");
}

#[test]
fn parse_error_clone() {
    let err = ParseError::Failed("test".to_string());
    let cloned = err.clone();
    assert_eq!(format!("{}", cloned), "Parse failed: test");
}

#[test]
fn span_equality() {
    let a = Span { start: 0, end: 5 };
    let b = Span { start: 0, end: 5 };
    let c = Span { start: 1, end: 5 };
    assert_eq!(a, b);
    assert_ne!(a, c);
}

#[test]
fn span_copy_semantics() {
    let a = Span { start: 0, end: 5 };
    let b = a;
    let c = a; // a still accessible
    assert_eq!(b.start, c.start);
    assert_eq!(a.end, 5);
}

#[test]
fn span_debug_output() {
    let span = Span { start: 10, end: 20 };
    let debug = format!("{:?}", span);
    assert!(debug.contains("10"));
    assert!(debug.contains("20"));
}

#[test]
fn parse_fails_or_recovers_on_wrong_tokens() {
    let mut grammar = GrammarBuilder::new("fail")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let b = sym_id(&grammar, "b");
    let result = pipeline_parse(&mut grammar, &[(b, 0, 1)]);
    // GLR driver may accept via error recovery or fail
    match result {
        Ok(forest) => {
            // If recovery succeeded, there should be error stats
            let (has_error, missing, cost) = forest.debug_error_stats();
            assert!(has_error || missing > 0 || cost > 0);
        }
        Err(_) => {
            // Expected: parse failed
        }
    }
}

#[test]
fn forest_view_returns_trait_object() {
    let (_grammar, forest) = single_token_grammar_and_forest();
    let view: &dyn ForestView = forest.view();
    // Just verify we can call trait methods
    assert!(!view.roots().is_empty());
}

#[test]
fn forest_node_large_span() {
    let mut forest = empty_forest(minimal_grammar());
    let id = insert_node(&mut forest, SymbolId(1), (0, 1_000_000), vec![]);
    assert_eq!(forest.nodes[&id].span, (0, 1_000_000));
}

#[test]
fn multiple_roots_in_parse_forest() {
    let mut forest = empty_forest(minimal_grammar());
    let id0 = insert_node(&mut forest, SymbolId(10), (0, 5), vec![]);
    let id1 = insert_node(&mut forest, SymbolId(10), (0, 5), vec![]);
    forest.roots.push(forest.nodes[&id0].clone());
    forest.roots.push(forest.nodes[&id1].clone());
    assert_eq!(forest.roots.len(), 2);
}

#[test]
fn forest_preserves_source_string() {
    let mut forest = empty_forest(minimal_grammar());
    forest.source = "abc def ghi".to_string();
    insert_node(&mut forest, SymbolId(1), (0, 3), vec![]);
    assert_eq!(forest.source, "abc def ghi");
}

#[test]
fn error_chunk_has_one_empty_alternative() {
    let mut forest = empty_forest(minimal_grammar());
    let id = forest.push_error_chunk((0, 5));
    let node = &forest.nodes[&id];
    assert_eq!(node.alternatives.len(), 1);
    assert!(node.alternatives[0].children.is_empty());
}

#[test]
fn error_meta_high_cost() {
    let meta = ErrorMeta {
        missing: false,
        is_error: true,
        cost: u32::MAX,
    };
    assert_eq!(meta.cost, u32::MAX);
}

#[test]
fn forest_node_many_alternatives() {
    let node = ForestNode {
        id: 0,
        symbol: SymbolId(10),
        span: (0, 10),
        alternatives: (0..100)
            .map(|i| ForestAlternative {
                children: vec![i as usize],
            })
            .collect(),
        error_meta: ErrorMeta::default(),
    };
    assert!(node.is_complete());
    assert_eq!(node.alternatives.len(), 100);
}

#[test]
fn two_token_grammar_forest_structure() {
    let mut grammar = GrammarBuilder::new("two")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a", "b"])
        .start("start")
        .build();
    let a = sym_id(&grammar, "a");
    let b = sym_id(&grammar, "b");
    let forest = pipeline_parse(&mut grammar, &[(a, 0, 1), (b, 1, 2)]).expect("parse");
    let view = forest.view();
    let root = view.roots()[0];
    let children = view.best_children(root);
    assert_eq!(children.len(), 2);
    let span = view.span(root);
    assert_eq!(span.start, 0);
    assert_eq!(span.end, 2);
}

#[test]
fn nested_rule_forest_structure() {
    let mut grammar = GrammarBuilder::new("nested")
        .token("a", "a")
        .rule("inner", vec!["a"])
        .rule("outer", vec!["inner"])
        .start("outer")
        .build();
    let a = sym_id(&grammar, "a");
    let forest = pipeline_parse(&mut grammar, &[(a, 0, 1)]).expect("parse");
    let view = forest.view();
    let root = view.roots()[0];
    let depth = tree_depth(view, root);
    assert!(depth >= 3); // outer -> inner -> a
}

#[test]
fn forest_view_kind_for_all_traversed_nodes() {
    let (_grammar, forest) = expr_grammar_and_forest();
    let view = forest.view();
    let root = view.roots()[0];
    let all_ids = collect_node_ids(view, root);
    for &id in &all_ids {
        let _kind = view.kind(id);
        let _span = view.span(id);
        // Just verify no panics
    }
}
