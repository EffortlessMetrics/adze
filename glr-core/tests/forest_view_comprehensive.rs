#![allow(clippy::needless_range_loop)]

//! Comprehensive tests for ForestView, ForestNode, ErrorMeta, and parse forest APIs.
//!
//! Covers: ForestNode construction, ForestView creation and navigation,
//! error metadata tracking, forest node relationships, multiple parse paths,
//! empty forests, single-node forests, and forest serialization/display.

use adze_glr_core::driver::GlrError;
use adze_glr_core::forest_view::{ForestView, Span};
use adze_glr_core::parse_forest::{
    ErrorMeta, ForestAlternative, ForestNode, ParseError, ParseForest, ParseNode, ParseTree,
    ERROR_SYMBOL,
};
use adze_glr_core::{
    Driver, FirstFollowSets, Forest, GLRError, ParseTable, build_lr1_automaton, sanity_check_tables,
};
use adze_ir::builder::GrammarBuilder;
use adze_ir::{Grammar, SymbolId};
use std::collections::HashMap;

// ─── Helpers ─────────────────────────────────────────────────────────

fn run_pipeline(grammar: &mut Grammar) -> Result<ParseTable, GLRError> {
    let first_follow = FirstFollowSets::compute_normalized(grammar)?;
    build_lr1_automaton(grammar, &first_follow)
}

fn pipeline_parse(
    grammar: &mut Grammar,
    token_stream: &[(SymbolId, u32, u32)],
) -> Result<Forest, GlrError> {
    let table = run_pipeline(grammar).expect("pipeline should produce a table");
    sanity_check_tables(&table).expect("table sanity check");
    let mut driver = Driver::new(&table);
    driver.parse_tokens(
        token_stream
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

fn single_token_forest() -> (Grammar, Forest) {
    let mut grammar = GrammarBuilder::new("one")
        .token("a", "a")
        .rule("S", vec!["a"])
        .start("S")
        .build();
    let a = sym_id(&grammar, "a");
    let forest = pipeline_parse(&mut grammar, &[(a, 0, 1)]).expect("parse should succeed");
    (grammar, forest)
}

fn expr_forest() -> (Grammar, Forest) {
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

/// Build a minimal Grammar for constructing a ParseForest by hand.
fn minimal_grammar() -> Grammar {
    GrammarBuilder::new("mini")
        .token("a", "a")
        .rule("S", vec!["a"])
        .start("S")
        .build()
}

// ═══════════════════════════════════════════════════════════════════════
//  1. ForestNode construction
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn forest_node_default_error_meta() {
    let node = ForestNode {
        id: 0,
        symbol: SymbolId(1),
        span: (0, 5),
        alternatives: vec![],
        error_meta: ErrorMeta::default(),
    };
    assert_eq!(node.id, 0);
    assert_eq!(node.symbol, SymbolId(1));
    assert_eq!(node.span, (0, 5));
    assert!(!node.error_meta.missing);
    assert!(!node.error_meta.is_error);
    assert_eq!(node.error_meta.cost, 0);
}

#[test]
fn forest_node_is_complete_with_alternatives() {
    let complete = ForestNode {
        id: 1,
        symbol: SymbolId(2),
        span: (0, 3),
        alternatives: vec![ForestAlternative { children: vec![10, 11] }],
        error_meta: ErrorMeta::default(),
    };
    assert!(complete.is_complete());

    let incomplete = ForestNode {
        id: 2,
        symbol: SymbolId(2),
        span: (0, 3),
        alternatives: vec![],
        error_meta: ErrorMeta::default(),
    };
    assert!(!incomplete.is_complete());
}

#[test]
fn forest_node_multiple_alternatives() {
    let node = ForestNode {
        id: 5,
        symbol: SymbolId(3),
        span: (0, 10),
        alternatives: vec![
            ForestAlternative { children: vec![1, 2] },
            ForestAlternative { children: vec![3, 4, 5] },
        ],
        error_meta: ErrorMeta::default(),
    };
    assert!(node.is_complete());
    assert_eq!(node.alternatives.len(), 2);
    assert_eq!(node.alternatives[0].children, vec![1, 2]);
    assert_eq!(node.alternatives[1].children, vec![3, 4, 5]);
}

#[test]
fn forest_node_clone() {
    let node = ForestNode {
        id: 7,
        symbol: SymbolId(4),
        span: (2, 8),
        alternatives: vec![ForestAlternative { children: vec![100] }],
        error_meta: ErrorMeta { missing: true, is_error: false, cost: 3 },
    };
    let cloned = node.clone();
    assert_eq!(cloned.id, 7);
    assert_eq!(cloned.symbol, SymbolId(4));
    assert_eq!(cloned.span, (2, 8));
    assert!(cloned.error_meta.missing);
    assert_eq!(cloned.error_meta.cost, 3);
}

// ═══════════════════════════════════════════════════════════════════════
//  2. ErrorMeta tracking
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn error_meta_default_is_clean() {
    let meta = ErrorMeta::default();
    assert!(!meta.missing);
    assert!(!meta.is_error);
    assert_eq!(meta.cost, 0);
}

#[test]
fn error_meta_is_copy() {
    let a = ErrorMeta { missing: true, is_error: false, cost: 5 };
    let b = a; // Copy
    assert_eq!(a.missing, b.missing);
    assert_eq!(a.is_error, b.is_error);
    assert_eq!(a.cost, b.cost);
}

#[test]
fn error_meta_missing_terminal() {
    let meta = ErrorMeta { missing: true, is_error: false, cost: 1 };
    assert!(meta.missing);
    assert!(!meta.is_error);
    assert_eq!(meta.cost, 1);
}

#[test]
fn error_meta_error_chunk() {
    let meta = ErrorMeta { missing: false, is_error: true, cost: 2 };
    assert!(!meta.missing);
    assert!(meta.is_error);
    assert_eq!(meta.cost, 2);
}

#[test]
fn error_meta_debug_format() {
    let meta = ErrorMeta { missing: true, is_error: false, cost: 42 };
    let dbg = format!("{meta:?}");
    assert!(dbg.contains("missing"));
    assert!(dbg.contains("true"));
    assert!(dbg.contains("42"));
}

// ═══════════════════════════════════════════════════════════════════════
//  3. Empty and single-node ParseForests
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn empty_parse_forest_has_no_roots() {
    let forest = ParseForest {
        roots: vec![],
        nodes: HashMap::new(),
        grammar: minimal_grammar(),
        source: String::new(),
        next_node_id: 0,
    };
    assert!(forest.roots.is_empty());
    assert!(forest.nodes.is_empty());
}

#[test]
fn single_node_parse_forest() {
    let node = ForestNode {
        id: 0,
        symbol: SymbolId(1),
        span: (0, 1),
        alternatives: vec![ForestAlternative { children: vec![] }],
        error_meta: ErrorMeta::default(),
    };
    let mut nodes = HashMap::new();
    nodes.insert(0, node.clone());
    let forest = ParseForest {
        roots: vec![node],
        nodes,
        grammar: minimal_grammar(),
        source: "a".to_string(),
        next_node_id: 1,
    };
    assert_eq!(forest.roots.len(), 1);
    assert_eq!(forest.nodes.len(), 1);
    assert_eq!(forest.roots[0].span, (0, 1));
}

#[test]
fn push_error_chunk_increments_node_id() {
    let mut forest = ParseForest {
        roots: vec![],
        nodes: HashMap::new(),
        grammar: minimal_grammar(),
        source: "abc".to_string(),
        next_node_id: 0,
    };
    let id0 = forest.push_error_chunk((0, 2));
    let id1 = forest.push_error_chunk((2, 3));
    assert_eq!(id0, 0);
    assert_eq!(id1, 1);
    assert_eq!(forest.next_node_id, 2);
    assert_eq!(forest.nodes.len(), 2);
}

#[test]
fn push_error_chunk_sets_error_meta() {
    let mut forest = ParseForest {
        roots: vec![],
        nodes: HashMap::new(),
        grammar: minimal_grammar(),
        source: "xyz".to_string(),
        next_node_id: 0,
    };
    let id = forest.push_error_chunk((1, 3));
    let node = &forest.nodes[&id];
    assert_eq!(node.symbol, ERROR_SYMBOL);
    assert!(node.error_meta.is_error);
    assert!(!node.error_meta.missing);
    assert_eq!(node.error_meta.cost, 1);
    assert_eq!(node.span, (1, 3));
}

#[test]
fn error_symbol_is_u16_max() {
    assert_eq!(ERROR_SYMBOL, SymbolId(u16::MAX));
}

// ═══════════════════════════════════════════════════════════════════════
//  4. debug_error_stats
// ═══════════════════════════════════════════════════════════════════════

#[cfg(feature = "test-api")]
#[test]
fn debug_error_stats_clean_forest() {
    let node = ForestNode {
        id: 0,
        symbol: SymbolId(1),
        span: (0, 1),
        alternatives: vec![ForestAlternative { children: vec![] }],
        error_meta: ErrorMeta::default(),
    };
    let mut nodes = HashMap::new();
    nodes.insert(0, node.clone());
    let forest = ParseForest {
        roots: vec![node],
        nodes,
        grammar: minimal_grammar(),
        source: "a".to_string(),
        next_node_id: 1,
    };
    let (has_error, missing, cost) = forest.debug_error_stats();
    assert!(!has_error);
    assert_eq!(missing, 0);
    assert_eq!(cost, 0);
}

#[cfg(feature = "test-api")]
#[test]
fn debug_error_stats_with_error_chunk() {
    let mut forest = ParseForest {
        roots: vec![],
        nodes: HashMap::new(),
        grammar: minimal_grammar(),
        source: "abc".to_string(),
        next_node_id: 0,
    };
    forest.push_error_chunk((0, 3));
    let (has_error, missing, cost) = forest.debug_error_stats();
    assert!(has_error);
    assert_eq!(missing, 0);
    assert_eq!(cost, 1);
}

#[cfg(feature = "test-api")]
#[test]
fn debug_error_stats_with_missing_terminal() {
    let node = ForestNode {
        id: 0,
        symbol: SymbolId(1),
        span: (0, 0),
        alternatives: vec![ForestAlternative { children: vec![] }],
        error_meta: ErrorMeta { missing: true, is_error: false, cost: 1 },
    };
    let mut nodes = HashMap::new();
    nodes.insert(0, node.clone());
    let forest = ParseForest {
        roots: vec![node],
        nodes,
        grammar: minimal_grammar(),
        source: String::new(),
        next_node_id: 1,
    };
    let (has_error, missing, cost) = forest.debug_error_stats();
    assert!(!has_error);
    assert_eq!(missing, 1);
    assert_eq!(cost, 1);
}

// ═══════════════════════════════════════════════════════════════════════
//  5. Forest node relationships
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn parent_child_relationship_in_parse_forest() {
    let child = ForestNode {
        id: 1,
        symbol: SymbolId(2),
        span: (0, 1),
        alternatives: vec![ForestAlternative { children: vec![] }],
        error_meta: ErrorMeta::default(),
    };
    let parent = ForestNode {
        id: 0,
        symbol: SymbolId(3),
        span: (0, 1),
        alternatives: vec![ForestAlternative { children: vec![1] }],
        error_meta: ErrorMeta::default(),
    };
    let mut nodes = HashMap::new();
    nodes.insert(0, parent.clone());
    nodes.insert(1, child.clone());
    let forest = ParseForest {
        roots: vec![parent],
        nodes,
        grammar: minimal_grammar(),
        source: "a".to_string(),
        next_node_id: 2,
    };
    let root = &forest.roots[0];
    assert_eq!(root.alternatives[0].children, vec![1]);
    let child_node = &forest.nodes[&1];
    assert_eq!(child_node.symbol, SymbolId(2));
}

// ═══════════════════════════════════════════════════════════════════════
//  6. ForestView via Driver — single token
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn single_token_view_has_one_root() {
    let (_g, forest) = single_token_forest();
    let view = forest.view();
    assert_eq!(view.roots().len(), 1);
}

#[test]
fn single_token_root_span_covers_input() {
    let (_g, forest) = single_token_forest();
    let view = forest.view();
    let sp = view.span(view.roots()[0]);
    assert_eq!(sp.start, 0);
    assert_eq!(sp.end, 1);
}

// ═══════════════════════════════════════════════════════════════════════
//  7. ForestView navigation — expression grammar
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn expr_root_kind_matches_start_symbol() {
    let (grammar, forest) = expr_forest();
    let view = forest.view();
    let root_kind = view.kind(view.roots()[0]);
    let expr_sym = sym_id(&grammar, "expr");
    assert_eq!(root_kind, expr_sym.0 as u32);
}

#[test]
fn expr_child_spans_within_root() {
    let (_g, forest) = expr_forest();
    let view = forest.view();
    let root = view.roots()[0];
    let root_sp = view.span(root);
    for &child in view.best_children(root) {
        let csp = view.span(child);
        assert!(csp.start >= root_sp.start && csp.end <= root_sp.end);
    }
}

#[test]
fn expr_recursive_walk_covers_all_nodes() {
    let (_g, forest) = expr_forest();
    let view = forest.view();
    let all = collect_node_ids(view, view.roots()[0]);
    assert!(all.len() >= 4, "expected ≥4 nodes, got {}", all.len());
}

// ═══════════════════════════════════════════════════════════════════════
//  8. Multiple parse paths (two-alternative grammar)
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn two_alternative_grammar_parses_both() {
    let mut grammar = GrammarBuilder::new("alt")
        .token("a", "a")
        .token("b", "b")
        .rule("S", vec!["a"])
        .rule("S", vec!["b"])
        .start("S")
        .build();

    let a = sym_id(&grammar, "a");
    let forest_a = pipeline_parse(&mut grammar, &[(a, 0, 1)]).expect("a");
    assert_eq!(forest_a.view().roots().len(), 1);

    let b = sym_id(&grammar, "b");
    let forest_b = pipeline_parse(&mut grammar, &[(b, 0, 1)]).expect("b");
    assert_eq!(forest_b.view().roots().len(), 1);
}

#[test]
fn longer_chain_has_single_root() {
    let mut grammar = GrammarBuilder::new("chain")
        .token("NUM", r"\d+")
        .token("PLUS", r"\+")
        .rule("expr", vec!["expr", "PLUS", "NUM"])
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    let num = sym_id(&grammar, "NUM");
    let plus = sym_id(&grammar, "PLUS");
    let forest = pipeline_parse(
        &mut grammar,
        &[(num, 0, 1), (plus, 1, 2), (num, 2, 3), (plus, 3, 4), (num, 4, 5)],
    )
    .expect("chain parse");
    let view = forest.view();
    assert_eq!(view.roots().len(), 1);
    assert_eq!(view.span(view.roots()[0]).end, 5);
}

// ═══════════════════════════════════════════════════════════════════════
//  9. Nonexistent node edge cases
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn nonexistent_node_returns_zero_kind() {
    let (_g, forest) = single_token_forest();
    assert_eq!(forest.view().kind(999_999), 0);
}

#[test]
fn nonexistent_node_returns_zero_span() {
    let (_g, forest) = single_token_forest();
    let sp = forest.view().span(999_999);
    assert_eq!(sp, Span { start: 0, end: 0 });
}

#[test]
fn nonexistent_node_returns_empty_children() {
    let (_g, forest) = single_token_forest();
    assert!(forest.view().best_children(999_999).is_empty());
}

// ═══════════════════════════════════════════════════════════════════════
// 10. ForestView consistency
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn forest_view_is_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<Box<dyn ForestView>>();
}

#[test]
fn repeated_calls_are_idempotent() {
    let (_g, forest) = expr_forest();
    let view = forest.view();
    let root = view.roots()[0];
    assert_eq!(view.roots(), view.roots());
    assert_eq!(view.span(root), view.span(root));
    assert_eq!(view.kind(root), view.kind(root));
    assert_eq!(view.best_children(root), view.best_children(root));
}

// ═══════════════════════════════════════════════════════════════════════
// 11. Depth and node count
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn single_token_depth_at_least_two() {
    let (_g, forest) = single_token_forest();
    let d = tree_depth(forest.view(), forest.view().roots()[0]);
    assert!(d >= 2, "expected depth ≥ 2, got {d}");
}

#[test]
fn expr_depth_at_least_three() {
    let (_g, forest) = expr_forest();
    let d = tree_depth(forest.view(), forest.view().roots()[0]);
    assert!(d >= 3, "expected depth ≥ 3, got {d}");
}

#[test]
fn expr_node_count_at_least_four() {
    let (_g, forest) = expr_forest();
    let count = node_count(forest.view(), forest.view().roots()[0]);
    assert!(count >= 4, "expected ≥4 nodes, got {count}");
}

// ═══════════════════════════════════════════════════════════════════════
// 12. Forest serialization / display (Debug formatting)
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn forest_node_debug_contains_id_and_symbol() {
    let node = ForestNode {
        id: 42,
        symbol: SymbolId(7),
        span: (10, 20),
        alternatives: vec![],
        error_meta: ErrorMeta::default(),
    };
    let dbg = format!("{node:?}");
    assert!(dbg.contains("42"), "Debug should contain node id");
    assert!(dbg.contains("10"), "Debug should contain span start");
    assert!(dbg.contains("20"), "Debug should contain span end");
}

#[test]
fn parse_tree_debug_format() {
    let tree = ParseTree {
        root: ParseNode {
            symbol: SymbolId(1),
            span: (0, 5),
            children: vec![ParseNode {
                symbol: SymbolId(2),
                span: (0, 5),
                children: vec![],
            }],
        },
        source: "hello".to_string(),
    };
    let dbg = format!("{tree:?}");
    assert!(dbg.contains("hello"), "Debug should contain source text");
    assert!(dbg.contains("children"), "Debug should show children");
}

#[test]
fn parse_error_display() {
    let incomplete = ParseError::Incomplete;
    assert_eq!(format!("{incomplete}"), "Incomplete parse");

    let failed = ParseError::Failed("unexpected token".to_string());
    assert!(format!("{failed}").contains("unexpected token"));

    let unknown = ParseError::Unknown;
    assert_eq!(format!("{unknown}"), "Unknown error");
}

#[test]
fn span_debug_format() {
    let s = Span { start: 3, end: 7 };
    let dbg = format!("{s:?}");
    assert!(dbg.contains("start") && dbg.contains("end"));
    assert!(dbg.contains('3') && dbg.contains('7'));
}
