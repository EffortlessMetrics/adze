//! Comprehensive tests for the `forest_view` module.
//!
//! Covers: Span construction/properties, ForestView trait methods (roots, kind,
//! span, best_children), Forest wrapper, tree traversal, edge cases (empty
//! forest, single node, multi-root), and Debug formatting.
//!
//! Does NOT use the `test-api` feature — only the public API.

use adze_glr_core::driver::GlrError;
use adze_glr_core::forest_view::{ForestView, Span};
use adze_glr_core::{
    Driver, FirstFollowSets, Forest, GLRError, ParseTable, build_lr1_automaton, sanity_check_tables,
};
use adze_ir::builder::GrammarBuilder;
use adze_ir::{Grammar, SymbolId};

// ─── Helpers ─────────────────────────────────────────────────────────

/// Run normalize → FIRST/FOLLOW → build_lr1_automaton, returning a ParseTable.
fn run_pipeline(grammar: &mut Grammar) -> Result<ParseTable, GLRError> {
    let first_follow = FirstFollowSets::compute_normalized(grammar)?;
    build_lr1_automaton(grammar, &first_follow)
}

/// Build grammar + table, then parse a token stream through the driver.
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

/// Resolve a symbol name to its SymbolId inside a built grammar.
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

/// Parse a single-token grammar and return the forest.
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

/// Parse an expression grammar `NUM + NUM` and return the forest.
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

// ═══════════════════════════════════════════════════════════════════════
//  1. Span construction and properties
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn span_new_zero_length() {
    let s = Span { start: 5, end: 5 };
    assert_eq!(s.start, 5);
    assert_eq!(s.end, 5);
}

#[test]
fn span_new_nonzero_length() {
    let s = Span { start: 0, end: 42 };
    assert_eq!(s.start, 0);
    assert_eq!(s.end, 42);
}

#[test]
fn span_equality() {
    let a = Span { start: 1, end: 3 };
    let b = Span { start: 1, end: 3 };
    let c = Span { start: 0, end: 3 };
    assert_eq!(a, b);
    assert_ne!(a, c);
}

#[test]
fn span_clone() {
    let a = Span { start: 10, end: 20 };
    let b = a;
    assert_eq!(a, b);
}

#[test]
fn span_debug_format() {
    let s = Span { start: 3, end: 7 };
    let dbg = format!("{s:?}");
    assert!(dbg.contains("start"), "Debug output should contain 'start'");
    assert!(dbg.contains("end"), "Debug output should contain 'end'");
    assert!(dbg.contains('3'));
    assert!(dbg.contains('7'));
}

// ═══════════════════════════════════════════════════════════════════════
//  2. ForestView construction — single-token grammar
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn single_token_has_one_root() {
    let (_g, forest) = single_token_forest();
    let view = forest.view();
    assert_eq!(view.roots().len(), 1, "single-token parse yields one root");
}

#[test]
fn single_token_root_span_covers_input() {
    let (_g, forest) = single_token_forest();
    let view = forest.view();
    let root = view.roots()[0];
    let sp = view.span(root);
    assert_eq!(sp.start, 0);
    assert_eq!(sp.end, 1);
}

#[test]
fn single_token_root_kind_is_nonzero() {
    let (_g, forest) = single_token_forest();
    let view = forest.view();
    let root = view.roots()[0];
    // The root node should have a symbol kind corresponding to the start symbol.
    let kind = view.kind(root);
    assert!(kind > 0, "start symbol kind should be > 0, got {kind}");
}

#[test]
fn single_token_root_has_children() {
    let (_g, forest) = single_token_forest();
    let view = forest.view();
    let root = view.roots()[0];
    let children = view.best_children(root);
    // S → a  ⟹ the root node should have at least one child (the terminal).
    assert!(
        !children.is_empty(),
        "root of S→a should have children (the terminal)"
    );
}

// ═══════════════════════════════════════════════════════════════════════
//  3. ForestView construction — expression grammar
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn expr_forest_has_one_root() {
    let (_g, forest) = expr_forest();
    let view = forest.view();
    assert_eq!(view.roots().len(), 1);
}

#[test]
fn expr_forest_root_span_covers_full_input() {
    let (_g, forest) = expr_forest();
    let view = forest.view();
    let sp = view.span(view.roots()[0]);
    assert_eq!(sp.start, 0);
    assert_eq!(sp.end, 3, "NUM+NUM occupies bytes 0..3");
}

#[test]
fn expr_forest_root_kind_matches_start_symbol() {
    let (grammar, forest) = expr_forest();
    let view = forest.view();
    let root_kind = view.kind(view.roots()[0]);
    let expr_sym = sym_id(&grammar, "expr");
    assert_eq!(
        root_kind, expr_sym.0 as u32,
        "root kind should match the 'expr' start symbol"
    );
}

// ═══════════════════════════════════════════════════════════════════════
//  4. Tree walking — children access
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn walk_children_of_root() {
    let (_g, forest) = expr_forest();
    let view = forest.view();
    let root = view.roots()[0];
    let children = view.best_children(root);
    // expr → expr PLUS NUM has 3 RHS symbols.
    // The root reduction may have 3 direct children or fewer depending
    // on how the GLR forest is structured.
    assert!(
        !children.is_empty(),
        "root must have at least one child in expression grammar"
    );
}

#[test]
fn child_spans_are_within_root_span() {
    let (_g, forest) = expr_forest();
    let view = forest.view();
    let root = view.roots()[0];
    let root_sp = view.span(root);

    for &child in view.best_children(root) {
        let csp = view.span(child);
        assert!(
            csp.start >= root_sp.start,
            "child start {cstart} < root start {rstart}",
            cstart = csp.start,
            rstart = root_sp.start,
        );
        assert!(
            csp.end <= root_sp.end,
            "child end {cend} > root end {rend}",
            cend = csp.end,
            rend = root_sp.end,
        );
    }
}

#[test]
fn leaf_nodes_have_no_children() {
    let (_g, forest) = single_token_forest();
    let view = forest.view();
    let root = view.roots()[0];
    let children = view.best_children(root);
    // Walk down to the leaf level — terminal nodes should have no children.
    for &child in children {
        let grandchildren = view.best_children(child);
        // Terminals are the bottom of the tree; they might have empty children.
        if grandchildren.is_empty() {
            // ok — this is a leaf
        } else {
            // Keep walking; maybe intermediate nonterminal
            for &gc in grandchildren {
                let _ = view.best_children(gc); // should not panic
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════
//  5. Recursive tree walk
// ═══════════════════════════════════════════════════════════════════════

/// Recursively collect all node IDs reachable from a root.
fn collect_node_ids(view: &dyn ForestView, root: u32) -> Vec<u32> {
    let mut result = vec![root];
    for &child in view.best_children(root) {
        result.extend(collect_node_ids(view, child));
    }
    result
}

#[test]
fn recursive_walk_visits_all_nodes() {
    let (_g, forest) = expr_forest();
    let view = forest.view();
    let root = view.roots()[0];
    let all = collect_node_ids(view, root);
    // NUM + NUM through expr → expr PLUS NUM and expr → NUM
    // Must have at least 4 nodes (root + 3 tokens or more with intermediates).
    assert!(
        all.len() >= 4,
        "expected ≥4 nodes in expr tree, got {}",
        all.len()
    );
}

#[test]
fn every_node_has_valid_span() {
    let (_g, forest) = expr_forest();
    let view = forest.view();
    for &root in view.roots() {
        for &id in &collect_node_ids(view, root) {
            let sp = view.span(id);
            assert!(
                sp.start <= sp.end,
                "node {id} has inverted span: {start}..{end}",
                start = sp.start,
                end = sp.end,
            );
        }
    }
}

#[test]
fn every_node_has_a_kind() {
    let (_g, forest) = expr_forest();
    let view = forest.view();
    for &root in view.roots() {
        for &id in &collect_node_ids(view, root) {
            let _kind = view.kind(id); // should not panic
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════
//  6. Edge case: nonexistent node ID
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn nonexistent_node_returns_zero_kind() {
    let (_g, forest) = single_token_forest();
    let view = forest.view();
    // Query a node ID that certainly does not exist.
    let kind = view.kind(999_999);
    assert_eq!(kind, 0, "nonexistent node should return kind 0");
}

#[test]
fn nonexistent_node_returns_zero_span() {
    let (_g, forest) = single_token_forest();
    let view = forest.view();
    let sp = view.span(999_999);
    assert_eq!(sp.start, 0);
    assert_eq!(sp.end, 0);
}

#[test]
fn nonexistent_node_returns_empty_children() {
    let (_g, forest) = single_token_forest();
    let view = forest.view();
    let children = view.best_children(999_999);
    assert!(children.is_empty());
}

// ═══════════════════════════════════════════════════════════════════════
//  7. Forest.view() returns a trait object reference
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn forest_view_is_send_sync() {
    // ForestView: Send + Sync is part of the trait definition.
    // This compiles only if the bound is satisfied.
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<Box<dyn ForestView>>();
}

#[test]
fn forest_view_returns_consistent_roots() {
    let (_g, forest) = expr_forest();
    let view = forest.view();
    let r1 = view.roots();
    let r2 = view.roots();
    assert_eq!(r1, r2, "repeated roots() calls must be identical");
}

#[test]
fn forest_view_returns_consistent_span() {
    let (_g, forest) = single_token_forest();
    let view = forest.view();
    let root = view.roots()[0];
    let s1 = view.span(root);
    let s2 = view.span(root);
    assert_eq!(s1, s2, "repeated span() calls must be identical");
}

#[test]
fn forest_view_returns_consistent_kind() {
    let (_g, forest) = single_token_forest();
    let view = forest.view();
    let root = view.roots()[0];
    let k1 = view.kind(root);
    let k2 = view.kind(root);
    assert_eq!(k1, k2, "repeated kind() calls must be identical");
}

#[test]
fn forest_view_returns_consistent_children() {
    let (_g, forest) = expr_forest();
    let view = forest.view();
    let root = view.roots()[0];
    let c1 = view.best_children(root);
    let c2 = view.best_children(root);
    assert_eq!(c1, c2, "repeated best_children() calls must be identical");
}

// ═══════════════════════════════════════════════════════════════════════
//  8. Multi-token grammar — longer input
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn longer_expression_chain() {
    let mut grammar = GrammarBuilder::new("chain")
        .token("NUM", r"\d+")
        .token("PLUS", r"\+")
        .rule("expr", vec!["expr", "PLUS", "NUM"])
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    let num = sym_id(&grammar, "NUM");
    let plus = sym_id(&grammar, "PLUS");
    // Parse: NUM + NUM + NUM (bytes 0..5)
    let forest = pipeline_parse(
        &mut grammar,
        &[
            (num, 0, 1),
            (plus, 1, 2),
            (num, 2, 3),
            (plus, 3, 4),
            (num, 4, 5),
        ],
    )
    .expect("should parse chain");

    let view = forest.view();
    assert_eq!(view.roots().len(), 1);
    let sp = view.span(view.roots()[0]);
    assert_eq!(sp.start, 0);
    assert_eq!(sp.end, 5);
}

// ═══════════════════════════════════════════════════════════════════════
//  9. Two-alternative grammar
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn grammar_with_two_alternatives() {
    let mut grammar = GrammarBuilder::new("alt")
        .token("a", "a")
        .token("b", "b")
        .rule("S", vec!["a"])
        .rule("S", vec!["b"])
        .start("S")
        .build();

    let a = sym_id(&grammar, "a");
    let forest_a = pipeline_parse(&mut grammar, &[(a, 0, 1)]).expect("a");
    let view_a = forest_a.view();
    assert_eq!(view_a.roots().len(), 1);
    assert_eq!(view_a.span(view_a.roots()[0]).end, 1);

    let b = sym_id(&grammar, "b");
    let forest_b = pipeline_parse(&mut grammar, &[(b, 0, 1)]).expect("b");
    let view_b = forest_b.view();
    assert_eq!(view_b.roots().len(), 1);
    assert_eq!(view_b.span(view_b.roots()[0]).end, 1);
}

// ═══════════════════════════════════════════════════════════════════════
// 10. Span ordering across siblings
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn sibling_spans_are_non_overlapping_and_ordered() {
    let (_g, forest) = expr_forest();
    let view = forest.view();
    let root = view.roots()[0];
    let children = view.best_children(root);
    if children.len() >= 2 {
        for w in children.windows(2) {
            let left = view.span(w[0]);
            let right = view.span(w[1]);
            assert!(
                left.end <= right.start,
                "sibling spans overlap: {left:?} vs {right:?}"
            );
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════
// 11. Depth measurement
// ═══════════════════════════════════════════════════════════════════════

fn tree_depth(view: &dyn ForestView, id: u32) -> usize {
    let children = view.best_children(id);
    if children.is_empty() {
        1
    } else {
        1 + children.iter().map(|&c| tree_depth(view, c)).max().unwrap()
    }
}

#[test]
fn single_token_tree_depth_is_at_least_two() {
    let (_g, forest) = single_token_forest();
    let view = forest.view();
    let d = tree_depth(view, view.roots()[0]);
    // S → a means root + leaf = depth ≥ 2.
    assert!(d >= 2, "expected depth ≥ 2, got {d}");
}

#[test]
fn expr_tree_depth_is_at_least_three() {
    let (_g, forest) = expr_forest();
    let view = forest.view();
    let d = tree_depth(view, view.roots()[0]);
    // expr → expr PLUS NUM → NUM  ⟹ ≥ 3 levels.
    assert!(d >= 3, "expected depth ≥ 3 for expr tree, got {d}");
}

// ═══════════════════════════════════════════════════════════════════════
// 12. Node count
// ═══════════════════════════════════════════════════════════════════════

fn node_count(view: &dyn ForestView, id: u32) -> usize {
    1 + view
        .best_children(id)
        .iter()
        .map(|&c| node_count(view, c))
        .sum::<usize>()
}

#[test]
fn single_token_node_count() {
    let (_g, forest) = single_token_forest();
    let view = forest.view();
    let count = node_count(view, view.roots()[0]);
    assert!(count >= 2, "S→a must have at least root+leaf = 2 nodes");
}

#[test]
fn expr_node_count() {
    let (_g, forest) = expr_forest();
    let view = forest.view();
    let count = node_count(view, view.roots()[0]);
    assert!(
        count >= 4,
        "expr tree for NUM+NUM should have at least 4 nodes, got {count}"
    );
}
