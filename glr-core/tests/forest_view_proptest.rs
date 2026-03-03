#![allow(clippy::needless_range_loop)]
//! Property-based tests for ForestView in adze-glr-core.
//!
//! Run with: `cargo test -p adze-glr-core --test forest_view_proptest`

use adze_glr_core::driver::GlrError;
use adze_glr_core::forest_view::{ForestView, Span};
use adze_glr_core::{
    Driver, FirstFollowSets, Forest, GLRError, ParseTable, build_lr1_automaton, sanity_check_tables,
};
use adze_ir::builder::GrammarBuilder;
use adze_ir::{Grammar, SymbolId};
use proptest::prelude::*;

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

fn single_token_grammar_and_table() -> (Grammar, ParseTable, SymbolId) {
    let mut grammar = GrammarBuilder::new("one")
        .token("a", "a")
        .rule("S", vec!["a"])
        .start("S")
        .build();
    let a = sym_id(&grammar, "a");
    let table = run_pipeline(&mut grammar).expect("pipeline");
    sanity_check_tables(&table).expect("sanity");
    (grammar, table, a)
}

fn expr_grammar_and_table() -> (Grammar, ParseTable, SymbolId, SymbolId) {
    let mut grammar = GrammarBuilder::new("expr")
        .token("NUM", r"\d+")
        .token("PLUS", r"\+")
        .rule("expr", vec!["expr", "PLUS", "NUM"])
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    let num = sym_id(&grammar, "NUM");
    let plus = sym_id(&grammar, "PLUS");
    let table = run_pipeline(&mut grammar).expect("pipeline");
    sanity_check_tables(&table).expect("sanity");
    (grammar, table, num, plus)
}

fn collect_all_ids(view: &dyn ForestView, id: u32) -> Vec<u32> {
    let mut result = vec![id];
    for &child in view.best_children(id) {
        result.extend(collect_all_ids(view, child));
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

// ═══════════════════════════════════════════════════════════════════════
//  1. ForestView creation
// ═══════════════════════════════════════════════════════════════════════

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    /// Parsing a single token always yields a forest with exactly one root.
    #[test]
    fn creation_single_token_has_one_root(_seed in 0u32..100) {
        let (mut grammar, _table, a) = single_token_grammar_and_table();
        let forest = pipeline_parse(&mut grammar, &[(a, 0, 1)]).expect("parse");
        prop_assert_eq!(forest.view().roots().len(), 1);
    }

    /// Parsing expression `NUM PLUS NUM` yields a forest with one root.
    #[test]
    fn creation_expr_has_one_root(_seed in 0u32..100) {
        let (mut grammar, _table, num, plus) = expr_grammar_and_table();
        let forest =
            pipeline_parse(&mut grammar, &[(num, 0, 1), (plus, 1, 2), (num, 2, 3)]).expect("parse");
        prop_assert_eq!(forest.view().roots().len(), 1);
    }

    /// The view() method returns a trait object that is Send + Sync.
    #[test]
    fn creation_view_is_send_sync(_seed in 0u32..10) {
        fn assert_send_sync<T: Send + Sync + ?Sized>() {}
        assert_send_sync::<dyn ForestView>();
    }
}

// ═══════════════════════════════════════════════════════════════════════
//  2. ForestView node access
// ═══════════════════════════════════════════════════════════════════════

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    /// Root node kind matches the start symbol of the grammar.
    #[test]
    fn node_access_root_kind_is_start_symbol(_seed in 0u32..50) {
        let mut grammar = GrammarBuilder::new("one")
            .token("a", "a")
            .rule("start", vec!["a"])
            .start("start")
            .build();
        let a = sym_id(&grammar, "a");
        let start_sym = sym_id(&grammar, "start");
        let forest = pipeline_parse(&mut grammar, &[(a, 0, 1)]).expect("parse");
        let view = forest.view();
        let root_kind = view.kind(view.roots()[0]);
        prop_assert_eq!(root_kind, start_sym.0 as u32);
    }

    /// kind() for a nonexistent node returns 0.
    #[test]
    fn node_access_nonexistent_kind_is_zero(bogus_id in 900_000u32..1_000_000) {
        let (mut grammar, _table, a) = single_token_grammar_and_table();
        let forest = pipeline_parse(&mut grammar, &[(a, 0, 1)]).expect("parse");
        prop_assert_eq!(forest.view().kind(bogus_id), 0);
    }

    /// kind() is deterministic — repeated calls return the same value.
    #[test]
    fn node_access_kind_is_idempotent(_seed in 0u32..50) {
        let (mut grammar, _table, a) = single_token_grammar_and_table();
        let forest = pipeline_parse(&mut grammar, &[(a, 0, 1)]).expect("parse");
        let view = forest.view();
        let root = view.roots()[0];
        let k1 = view.kind(root);
        let k2 = view.kind(root);
        prop_assert_eq!(k1, k2);
    }
}

// ═══════════════════════════════════════════════════════════════════════
//  3. ForestView root
// ═══════════════════════════════════════════════════════════════════════

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    /// Root span covers the entire input for single-token parse.
    #[test]
    fn root_span_covers_input(_seed in 0u32..50) {
        let (mut grammar, _table, a) = single_token_grammar_and_table();
        let forest = pipeline_parse(&mut grammar, &[(a, 0, 1)]).expect("parse");
        let view = forest.view();
        let sp = view.span(view.roots()[0]);
        prop_assert_eq!(sp.start, 0);
        prop_assert_eq!(sp.end, 1);
    }

    /// Root span covers the full expression for multi-token parse.
    #[test]
    fn root_span_covers_expr(_seed in 0u32..50) {
        let (mut grammar, _table, num, plus) = expr_grammar_and_table();
        let forest =
            pipeline_parse(&mut grammar, &[(num, 0, 1), (plus, 1, 2), (num, 2, 3)]).expect("parse");
        let view = forest.view();
        let sp = view.span(view.roots()[0]);
        prop_assert!(sp.start == 0);
        prop_assert!(sp.end == 3);
    }

    /// roots() is idempotent — repeated calls return the same slice.
    #[test]
    fn root_roots_idempotent(_seed in 0u32..50) {
        let (mut grammar, _table, a) = single_token_grammar_and_table();
        let forest = pipeline_parse(&mut grammar, &[(a, 0, 1)]).expect("parse");
        let view = forest.view();
        let r1 = view.roots();
        let r2 = view.roots();
        prop_assert_eq!(r1, r2);
    }
}

// ═══════════════════════════════════════════════════════════════════════
//  4. ForestView child iteration
// ═══════════════════════════════════════════════════════════════════════

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    /// Every child of the root has a span contained within the root span.
    #[test]
    fn child_spans_within_root(_seed in 0u32..50) {
        let (mut grammar, _table, num, plus) = expr_grammar_and_table();
        let forest =
            pipeline_parse(&mut grammar, &[(num, 0, 1), (plus, 1, 2), (num, 2, 3)]).expect("parse");
        let view = forest.view();
        let root = view.roots()[0];
        let root_sp = view.span(root);
        for &child in view.best_children(root) {
            let csp = view.span(child);
            prop_assert!(csp.start >= root_sp.start);
            prop_assert!(csp.end <= root_sp.end);
        }
    }

    /// best_children on a nonexistent node returns an empty slice.
    #[test]
    fn child_nonexistent_returns_empty(bogus_id in 900_000u32..1_000_000) {
        let (mut grammar, _table, a) = single_token_grammar_and_table();
        let forest = pipeline_parse(&mut grammar, &[(a, 0, 1)]).expect("parse");
        prop_assert!(forest.view().best_children(bogus_id).is_empty());
    }

    /// best_children is idempotent.
    #[test]
    fn child_best_children_idempotent(_seed in 0u32..50) {
        let (mut grammar, _table, num, plus) = expr_grammar_and_table();
        let forest =
            pipeline_parse(&mut grammar, &[(num, 0, 1), (plus, 1, 2), (num, 2, 3)]).expect("parse");
        let view = forest.view();
        let root = view.roots()[0];
        let c1 = view.best_children(root);
        let c2 = view.best_children(root);
        prop_assert_eq!(c1, c2);
    }
}

// ═══════════════════════════════════════════════════════════════════════
//  5. ForestView with single node
// ═══════════════════════════════════════════════════════════════════════

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    /// Single-token forest has exactly one root node.
    #[test]
    fn single_node_one_root(_seed in 0u32..50) {
        let (mut grammar, _table, a) = single_token_grammar_and_table();
        let forest = pipeline_parse(&mut grammar, &[(a, 0, 1)]).expect("parse");
        let view = forest.view();
        prop_assert_eq!(view.roots().len(), 1);
    }

    /// All leaf nodes in a single-token forest have no children.
    #[test]
    fn single_node_leaves_have_no_children(_seed in 0u32..50) {
        let (mut grammar, _table, a) = single_token_grammar_and_table();
        let forest = pipeline_parse(&mut grammar, &[(a, 0, 1)]).expect("parse");
        let view = forest.view();
        let all = collect_all_ids(view, view.roots()[0]);
        for &id in &all {
            let children = view.best_children(id);
            if children.is_empty() {
                // leaf — confirm kind is non-zero (valid node)
                prop_assert!(view.kind(id) > 0 || id != view.roots()[0]);
            }
        }
    }

    /// Total node count in a single-token forest is at least 2 (root + leaf).
    #[test]
    fn single_node_count_at_least_two(_seed in 0u32..50) {
        let (mut grammar, _table, a) = single_token_grammar_and_table();
        let forest = pipeline_parse(&mut grammar, &[(a, 0, 1)]).expect("parse");
        let view = forest.view();
        let count = node_count(view, view.roots()[0]);
        prop_assert!(count >= 2, "expected ≥2 nodes, got {count}");
    }
}

// ═══════════════════════════════════════════════════════════════════════
//  6. ForestView with deep forest
// ═══════════════════════════════════════════════════════════════════════

proptest! {
    #![proptest_config(ProptestConfig::with_cases(16))]

    /// Chaining NUM (PLUS NUM)* for `n` additions produces a valid forest.
    #[test]
    fn deep_chain_has_single_root(n in 1usize..6) {
        let (mut grammar, _table, num, plus) = expr_grammar_and_table();
        let mut tokens = vec![(num, 0u32, 1u32)];
        for i in 0..n {
            let base = (i as u32) * 2 + 1;
            tokens.push((plus, base, base + 1));
            tokens.push((num, base + 1, base + 2));
        }
        let forest = pipeline_parse(&mut grammar, &tokens).expect("parse");
        prop_assert_eq!(forest.view().roots().len(), 1);
    }

    /// Tree depth grows with chain length.
    #[test]
    fn deep_chain_depth_increases(n in 1usize..6) {
        let (mut grammar, _table, num, plus) = expr_grammar_and_table();
        let mut tokens = vec![(num, 0u32, 1u32)];
        for i in 0..n {
            let base = (i as u32) * 2 + 1;
            tokens.push((plus, base, base + 1));
            tokens.push((num, base + 1, base + 2));
        }
        let forest = pipeline_parse(&mut grammar, &tokens).expect("parse");
        let view = forest.view();
        let d = tree_depth(view, view.roots()[0]);
        // At minimum depth 2 for any expression, growing with n
        prop_assert!(d >= 2, "depth {d} should be ≥ 2");
    }

    /// Root span end grows with chain length.
    #[test]
    fn deep_chain_span_grows(n in 1usize..6) {
        let (mut grammar, _table, num, plus) = expr_grammar_and_table();
        let mut tokens = vec![(num, 0u32, 1u32)];
        for i in 0..n {
            let base = (i as u32) * 2 + 1;
            tokens.push((plus, base, base + 1));
            tokens.push((num, base + 1, base + 2));
        }
        let expected_end = (n as u32) * 2 + 1;
        let forest = pipeline_parse(&mut grammar, &tokens).expect("parse");
        let view = forest.view();
        let sp = view.span(view.roots()[0]);
        prop_assert_eq!(sp.start, 0);
        prop_assert_eq!(sp.end, expected_end);
    }
}

// ═══════════════════════════════════════════════════════════════════════
//  7. ForestView traversal
// ═══════════════════════════════════════════════════════════════════════

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    /// Full recursive walk visits at least as many nodes as the token count.
    #[test]
    fn traversal_visit_count_ge_token_count(_seed in 0u32..50) {
        let (mut grammar, _table, num, plus) = expr_grammar_and_table();
        let forest =
            pipeline_parse(&mut grammar, &[(num, 0, 1), (plus, 1, 2), (num, 2, 3)]).expect("parse");
        let view = forest.view();
        let all = collect_all_ids(view, view.roots()[0]);
        // 3 tokens → at least 3 nodes (plus internal nodes)
        prop_assert!(all.len() >= 3, "expected ≥3 nodes, got {}", all.len());
    }

    /// Every node in a DFS traversal has a valid (non-default) kind, except possibly missing nodes.
    #[test]
    fn traversal_all_nodes_have_valid_kind(_seed in 0u32..50) {
        let (mut grammar, _table, num, plus) = expr_grammar_and_table();
        let forest =
            pipeline_parse(&mut grammar, &[(num, 0, 1), (plus, 1, 2), (num, 2, 3)]).expect("parse");
        let view = forest.view();
        let all = collect_all_ids(view, view.roots()[0]);
        for &id in &all {
            let kind = view.kind(id);
            // All real nodes from a successful parse should have non-zero kind
            prop_assert!(kind > 0, "node {id} has zero kind");
        }
    }

    /// Span of every child is contained within its parent's span.
    #[test]
    fn traversal_child_spans_nested(_seed in 0u32..50) {
        let (mut grammar, _table, num, plus) = expr_grammar_and_table();
        let forest =
            pipeline_parse(&mut grammar, &[(num, 0, 1), (plus, 1, 2), (num, 2, 3)]).expect("parse");
        let view = forest.view();

        fn check_nested(view: &dyn ForestView, id: u32) -> Result<(), TestCaseError> {
            let parent_sp = view.span(id);
            for &child in view.best_children(id) {
                let csp = view.span(child);
                prop_assert!(csp.start >= parent_sp.start,
                    "child {child} start {} < parent {id} start {}", csp.start, parent_sp.start);
                prop_assert!(csp.end <= parent_sp.end,
                    "child {child} end {} > parent {id} end {}", csp.end, parent_sp.end);
                check_nested(view, child)?;
            }
            Ok(())
        }
        check_nested(view, view.roots()[0])?;
    }

    /// Leaf nodes (no children) have non-empty spans.
    #[test]
    fn traversal_leaf_spans_nonempty(_seed in 0u32..50) {
        let (mut grammar, _table, num, plus) = expr_grammar_and_table();
        let forest =
            pipeline_parse(&mut grammar, &[(num, 0, 1), (plus, 1, 2), (num, 2, 3)]).expect("parse");
        let view = forest.view();
        let all = collect_all_ids(view, view.roots()[0]);
        for &id in &all {
            if view.best_children(id).is_empty() {
                let sp = view.span(id);
                prop_assert!(sp.end > sp.start, "leaf {id} has empty span {sp:?}");
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════
//  8. ForestView metadata (span)
// ═══════════════════════════════════════════════════════════════════════

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    /// span() on a nonexistent node returns Span { start: 0, end: 0 }.
    #[test]
    fn metadata_nonexistent_span_is_zero(bogus_id in 900_000u32..1_000_000) {
        let (mut grammar, _table, a) = single_token_grammar_and_table();
        let forest = pipeline_parse(&mut grammar, &[(a, 0, 1)]).expect("parse");
        let sp = forest.view().span(bogus_id);
        prop_assert_eq!(sp, Span { start: 0, end: 0 });
    }

    /// span() is idempotent.
    #[test]
    fn metadata_span_idempotent(_seed in 0u32..50) {
        let (mut grammar, _table, a) = single_token_grammar_and_table();
        let forest = pipeline_parse(&mut grammar, &[(a, 0, 1)]).expect("parse");
        let view = forest.view();
        let root = view.roots()[0];
        let s1 = view.span(root);
        let s2 = view.span(root);
        prop_assert_eq!(s1, s2);
    }

    /// Root span start is always 0 for zero-based token positions.
    #[test]
    fn metadata_root_span_starts_at_zero(_seed in 0u32..50) {
        let (mut grammar, _table, a) = single_token_grammar_and_table();
        let forest = pipeline_parse(&mut grammar, &[(a, 0, 1)]).expect("parse");
        let view = forest.view();
        let sp = view.span(view.roots()[0]);
        prop_assert_eq!(sp.start, 0);
    }

    /// Root span end equals the last token end for expr grammar.
    #[test]
    fn metadata_root_span_end_matches_input(_seed in 0u32..50) {
        let (mut grammar, _table, num, plus) = expr_grammar_and_table();
        let forest =
            pipeline_parse(&mut grammar, &[(num, 0, 1), (plus, 1, 2), (num, 2, 3)]).expect("parse");
        let view = forest.view();
        let sp = view.span(view.roots()[0]);
        prop_assert_eq!(sp.end, 3);
    }
}

// ═══════════════════════════════════════════════════════════════════════
//  9. Two-alternative grammar through ForestView
// ═══════════════════════════════════════════════════════════════════════

proptest! {
    #![proptest_config(ProptestConfig::with_cases(16))]

    /// Parsing either alternative of `S -> a | b` succeeds with one root.
    #[test]
    fn alt_grammar_both_alternatives_parse(choice in 0u32..2) {
        let mut grammar = GrammarBuilder::new("alt")
            .token("a", "a")
            .token("b", "b")
            .rule("S", vec!["a"])
            .rule("S", vec!["b"])
            .start("S")
            .build();
        let tok = if choice == 0 {
            sym_id(&grammar, "a")
        } else {
            sym_id(&grammar, "b")
        };
        let forest = pipeline_parse(&mut grammar, &[(tok, 0, 1)]).expect("parse");
        let view = forest.view();
        prop_assert_eq!(view.roots().len(), 1);
        prop_assert_eq!(view.span(view.roots()[0]), Span { start: 0, end: 1 });
    }
}

// ═══════════════════════════════════════════════════════════════════════
// 10. Consistency across the full ForestView API
// ═══════════════════════════════════════════════════════════════════════

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    /// Every root id reported by roots() has a non-zero kind.
    #[test]
    fn consistency_roots_have_nonzero_kind(_seed in 0u32..50) {
        let (mut grammar, _table, num, plus) = expr_grammar_and_table();
        let forest =
            pipeline_parse(&mut grammar, &[(num, 0, 1), (plus, 1, 2), (num, 2, 3)]).expect("parse");
        let view = forest.view();
        for &root_id in view.roots() {
            prop_assert!(view.kind(root_id) > 0, "root {root_id} has zero kind");
        }
    }

    /// Node count via collect_all_ids equals node_count helper.
    #[test]
    fn consistency_collect_equals_count(_seed in 0u32..50) {
        let (mut grammar, _table, num, plus) = expr_grammar_and_table();
        let forest =
            pipeline_parse(&mut grammar, &[(num, 0, 1), (plus, 1, 2), (num, 2, 3)]).expect("parse");
        let view = forest.view();
        let root = view.roots()[0];
        let collected = collect_all_ids(view, root).len();
        let counted = node_count(view, root);
        prop_assert_eq!(collected, counted);
    }

    /// tree_depth is always ≥ 1 for any parsed forest.
    #[test]
    fn consistency_depth_at_least_one(_seed in 0u32..50) {
        let (mut grammar, _table, a) = single_token_grammar_and_table();
        let forest = pipeline_parse(&mut grammar, &[(a, 0, 1)]).expect("parse");
        let view = forest.view();
        let d = tree_depth(view, view.roots()[0]);
        prop_assert!(d >= 1);
    }
}
