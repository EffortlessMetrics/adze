#![allow(clippy::needless_range_loop)]
//! Comprehensive property-based + unit tests for ParseForest, ForestView, Forest,
//! ForestNode, ErrorMeta, ParseTree, ParseNode, and ParseError.
//!
//! Run with:
//!   RUST_TEST_THREADS=2 cargo test -p adze-glr-core --test forest_comprehensive_proptest

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
use proptest::prelude::*;
use std::collections::{HashMap, HashSet};

// ─── Helpers ─────────────────────────────────────────────────────────

fn minimal_grammar() -> Grammar {
    GrammarBuilder::new("mini")
        .token("a", "a")
        .rule("S", vec!["a"])
        .start("S")
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

// ─── Proptest strategies ─────────────────────────────────────────────

fn arb_symbol_id() -> impl Strategy<Value = SymbolId> {
    (1u16..1000).prop_map(SymbolId)
}

fn arb_span(max_end: usize) -> impl Strategy<Value = (usize, usize)> {
    (0..max_end)
        .prop_flat_map(move |start| (Just(start), start..=max_end).prop_map(|(s, e)| (s, e)))
}

fn arb_error_meta_valid() -> impl Strategy<Value = ErrorMeta> {
    // Generate only valid combos: either missing or is_error, not both
    prop_oneof![
        (0u32..100).prop_map(|cost| ErrorMeta {
            missing: false,
            is_error: false,
            cost,
        }),
        (1u32..100).prop_map(|cost| ErrorMeta {
            missing: true,
            is_error: false,
            cost,
        }),
        (1u32..100).prop_map(|cost| ErrorMeta {
            missing: false,
            is_error: true,
            cost,
        }),
    ]
}

// ═══════════════════════════════════════════════════════════════════════
//  1. ParseForest construction invariants (property tests)
// ═══════════════════════════════════════════════════════════════════════

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn prop_node_ids_are_sequential(count in 1usize..30) {
        let mut forest = empty_forest(minimal_grammar());
        let mut ids = Vec::new();
        for _ in 0..count {
            ids.push(insert_node(&mut forest, SymbolId(1), (0, 1), vec![]));
        }
        for (i, &id) in ids.iter().enumerate() {
            prop_assert_eq!(id, i, "node id should equal insertion order");
        }
    }

    #[test]
    fn prop_next_node_id_tracks_all_insertions(regular in 0usize..15, errors in 0usize..15) {
        let mut forest = empty_forest(minimal_grammar());
        for _ in 0..regular {
            insert_node(&mut forest, SymbolId(1), (0, 1), vec![]);
        }
        for _ in 0..errors {
            forest.push_error_chunk((0, 1));
        }
        prop_assert_eq!(forest.next_node_id, regular + errors);
        prop_assert_eq!(forest.nodes.len(), regular + errors);
    }

    #[test]
    fn prop_symbol_roundtrips(sym in 1u16..1000) {
        let mut forest = empty_forest(minimal_grammar());
        let id = insert_node(&mut forest, SymbolId(sym), (0, 1), vec![]);
        prop_assert_eq!(forest.nodes[&id].symbol, SymbolId(sym));
    }

    #[test]
    fn prop_span_roundtrips(start in 0usize..500, len in 0usize..500) {
        let end = start + len;
        let mut forest = empty_forest(minimal_grammar());
        let id = insert_node(&mut forest, SymbolId(1), (start, end), vec![]);
        prop_assert_eq!(forest.nodes[&id].span, (start, end));
    }

    #[test]
    fn prop_children_roundtrip(child_count in 0usize..10) {
        let mut forest = empty_forest(minimal_grammar());
        let children: Vec<usize> = (0..child_count)
            .map(|i| insert_node(&mut forest, SymbolId(1), (i, i + 1), vec![]))
            .collect();
        let parent = insert_node(&mut forest, SymbolId(2), (0, child_count), children.clone());
        prop_assert_eq!(&forest.nodes[&parent].alternatives[0].children, &children);
    }
}

// ═══════════════════════════════════════════════════════════════════════
//  2. Error chunk property tests
// ═══════════════════════════════════════════════════════════════════════

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn prop_error_chunk_symbol_is_error(start in 0usize..200, len in 0usize..200) {
        let end = start + len;
        let mut forest = empty_forest(minimal_grammar());
        let id = forest.push_error_chunk((start, end));
        prop_assert_eq!(forest.nodes[&id].symbol, ERROR_SYMBOL);
    }

    #[test]
    fn prop_error_chunk_meta_invariants(start in 0usize..200, len in 0usize..200) {
        let end = start + len;
        let mut forest = empty_forest(minimal_grammar());
        let id = forest.push_error_chunk((start, end));
        let node = &forest.nodes[&id];
        prop_assert!(node.error_meta.is_error);
        prop_assert!(!node.error_meta.missing);
        prop_assert_eq!(node.error_meta.cost, 1);
    }

    #[test]
    fn prop_error_chunk_has_one_empty_alternative(start in 0usize..100, len in 0usize..100) {
        let end = start + len;
        let mut forest = empty_forest(minimal_grammar());
        let id = forest.push_error_chunk((start, end));
        let node = &forest.nodes[&id];
        prop_assert_eq!(node.alternatives.len(), 1);
        prop_assert!(node.alternatives[0].children.is_empty());
    }
}

// ═══════════════════════════════════════════════════════════════════════
//  3. Forest depth property tests
// ═══════════════════════════════════════════════════════════════════════

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn prop_linear_chain_depth(depth in 1usize..20) {
        let mut forest = empty_forest(minimal_grammar());
        let mut current = insert_node(&mut forest, SymbolId(1), (0, 1), vec![]);
        for _ in 1..depth {
            current = insert_node(&mut forest, SymbolId(1), (0, 1), vec![current]);
        }
        prop_assert_eq!(compute_depth(&forest, current), depth);
    }

    #[test]
    fn prop_wide_tree_depth_is_two(width in 1usize..15) {
        let mut forest = empty_forest(minimal_grammar());
        let children: Vec<usize> = (0..width)
            .map(|i| insert_node(&mut forest, SymbolId(1), (i, i + 1), vec![]))
            .collect();
        let root = insert_node(&mut forest, SymbolId(2), (0, width), children);
        prop_assert_eq!(compute_depth(&forest, root), 2);
    }

    #[test]
    fn prop_leaf_depth_is_one(sym in 1u16..100) {
        let mut forest = empty_forest(minimal_grammar());
        let id = insert_node(&mut forest, SymbolId(sym), (0, 1), vec![]);
        prop_assert_eq!(compute_depth(&forest, id), 1);
    }
}

// ═══════════════════════════════════════════════════════════════════════
//  4. Ambiguity / multiple alternatives property tests
// ═══════════════════════════════════════════════════════════════════════

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn prop_ambiguous_node_alternative_count(alt_count in 1usize..6) {
        let mut forest = empty_forest(minimal_grammar());
        let alternatives: Vec<Vec<usize>> = (0..alt_count)
            .map(|_| {
                let child = insert_node(&mut forest, SymbolId(1), (0, 1), vec![]);
                vec![child]
            })
            .collect();
        let id = insert_ambiguous_node(&mut forest, SymbolId(5), (0, 1), alternatives);
        prop_assert_eq!(forest.nodes[&id].alternatives.len(), alt_count);
        prop_assert!(forest.nodes[&id].is_complete());
    }

    #[test]
    fn prop_ambiguous_children_per_alt(alt_count in 1usize..4, children_per in 1usize..4) {
        let mut forest = empty_forest(minimal_grammar());
        let mut alternatives = Vec::new();
        for _ in 0..alt_count {
            let children: Vec<usize> = (0..children_per)
                .map(|_| insert_node(&mut forest, SymbolId(1), (0, 1), vec![]))
                .collect();
            alternatives.push(children);
        }
        let id = insert_ambiguous_node(&mut forest, SymbolId(5), (0, 1), alternatives);
        for alt in &forest.nodes[&id].alternatives {
            prop_assert_eq!(alt.children.len(), children_per);
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════
//  5. Clone independence property tests
// ═══════════════════════════════════════════════════════════════════════

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn prop_clone_independence(count in 1usize..10) {
        let mut forest = empty_forest(minimal_grammar());
        for _ in 0..count {
            insert_node(&mut forest, SymbolId(1), (0, 1), vec![]);
        }
        let mut cloned = forest.clone();
        insert_node(&mut cloned, SymbolId(2), (0, 1), vec![]);
        prop_assert_eq!(forest.nodes.len(), count);
        prop_assert_eq!(cloned.nodes.len(), count + 1);
    }

    #[test]
    fn prop_clone_roots_independence(count in 1usize..5) {
        let mut forest = empty_forest(minimal_grammar());
        for _ in 0..count {
            let id = insert_node(&mut forest, SymbolId(1), (0, 1), vec![]);
            forest.roots.push(forest.nodes[&id].clone());
        }
        let mut cloned = forest.clone();
        cloned.roots.clear();
        prop_assert_eq!(forest.roots.len(), count);
        prop_assert!(cloned.roots.is_empty());
    }
}

// ═══════════════════════════════════════════════════════════════════════
//  6. ErrorMeta property tests
// ═══════════════════════════════════════════════════════════════════════

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn prop_error_meta_copy_semantics(missing in any::<bool>(), is_error in any::<bool>(), cost in 0u32..1000) {
        let meta = ErrorMeta { missing, is_error, cost };
        let copied = meta;
        prop_assert_eq!(meta.missing, copied.missing);
        prop_assert_eq!(meta.is_error, copied.is_error);
        prop_assert_eq!(meta.cost, copied.cost);
    }

    #[test]
    fn prop_error_meta_debug_contains_fields(cost in 0u32..100) {
        let meta = ErrorMeta { missing: true, is_error: false, cost };
        let dbg = format!("{meta:?}");
        prop_assert!(dbg.contains("missing"));
        prop_assert!(dbg.contains("cost"));
    }
}

// ═══════════════════════════════════════════════════════════════════════
//  7. ParseTree / ParseNode property tests
// ═══════════════════════════════════════════════════════════════════════

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn prop_parse_tree_preserves_source(src in "[a-z]{1,30}") {
        let tree = ParseTree {
            root: ParseNode {
                symbol: SymbolId(1),
                span: (0, src.len()),
                children: vec![],
            },
            source: src.clone(),
        };
        prop_assert_eq!(&tree.source, &src);
    }

    #[test]
    fn prop_parse_node_children_preserved(count in 0usize..10) {
        let children: Vec<ParseNode> = (0..count)
            .map(|i| ParseNode {
                symbol: SymbolId(i as u16 + 1),
                span: (i, i + 1),
                children: vec![],
            })
            .collect();
        let node = ParseNode {
            symbol: SymbolId(100),
            span: (0, count),
            children,
        };
        prop_assert_eq!(node.children.len(), count);
    }

    #[test]
    fn prop_parse_error_failed_message(msg in "[a-zA-Z0-9 ]{1,40}") {
        let err = ParseError::Failed(msg.clone());
        let displayed = format!("{}", err);
        prop_assert!(displayed.contains(&msg));
    }

    #[test]
    fn prop_parse_node_deep_nesting(depth in 1usize..15) {
        let mut node = ParseNode {
            symbol: SymbolId(1),
            span: (0, 1),
            children: vec![],
        };
        for i in 2..=depth {
            node = ParseNode {
                symbol: SymbolId(i as u16),
                span: (0, 1),
                children: vec![node],
            };
        }
        // Walk to leaf
        let mut cur = &node;
        for _ in 0..(depth - 1) {
            prop_assert_eq!(cur.children.len(), 1);
            cur = &cur.children[0];
        }
        prop_assert!(cur.children.is_empty());
        prop_assert_eq!(cur.symbol, SymbolId(1));
    }
}

// ═══════════════════════════════════════════════════════════════════════
//  8. ForestView via Driver — property tests
// ═══════════════════════════════════════════════════════════════════════

proptest! {
    #![proptest_config(ProptestConfig::with_cases(16))]

    #[test]
    fn prop_expr_chain_has_one_root(extra_terms in 0usize..3) {
        let mut grammar = GrammarBuilder::new("expr")
            .token("NUM", r"\d+")
            .token("PLUS", r"\+")
            .rule("expr", vec!["expr", "PLUS", "NUM"])
            .rule("expr", vec!["NUM"])
            .start("expr")
            .build();
        let num = sym_id(&grammar, "NUM");
        let plus = sym_id(&grammar, "PLUS");
        let mut tokens = vec![(num, 0u32, 1u32)];
        for i in 0..extra_terms {
            let base = (i as u32 + 1) * 2 - 1;
            tokens.push((plus, base, base + 1));
            tokens.push((num, base + 1, base + 2));
        }
        let forest = pipeline_parse(&mut grammar, &tokens).expect("parse");
        prop_assert_eq!(forest.view().roots().len(), 1);
    }

    #[test]
    fn prop_forest_view_idempotent_roots(_seed in 0u32..10) {
        let (_g, forest) = single_token_forest();
        let view = forest.view();
        prop_assert_eq!(view.roots(), view.roots());
    }

    #[test]
    fn prop_forest_view_idempotent_span(_seed in 0u32..10) {
        let (_g, forest) = single_token_forest();
        let view = forest.view();
        let root = view.roots()[0];
        prop_assert_eq!(view.span(root), view.span(root));
    }

    #[test]
    fn prop_forest_view_idempotent_kind(_seed in 0u32..10) {
        let (_g, forest) = single_token_forest();
        let view = forest.view();
        let root = view.roots()[0];
        prop_assert_eq!(view.kind(root), view.kind(root));
    }

    #[test]
    fn prop_forest_view_idempotent_children(_seed in 0u32..10) {
        let (_g, forest) = single_token_forest();
        let view = forest.view();
        let root = view.roots()[0];
        prop_assert_eq!(view.best_children(root), view.best_children(root));
    }
}

// ═══════════════════════════════════════════════════════════════════════
//  9. Unit tests — ForestNode basics
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn node_no_alternatives_is_incomplete() {
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
fn node_single_empty_alt_is_complete() {
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
fn node_multiple_alts_all_accessible() {
    let node = ForestNode {
        id: 0,
        symbol: SymbolId(1),
        span: (0, 5),
        alternatives: vec![
            ForestAlternative {
                children: vec![1, 2],
            },
            ForestAlternative { children: vec![3] },
            ForestAlternative {
                children: vec![4, 5, 6],
            },
        ],
        error_meta: ErrorMeta::default(),
    };
    assert_eq!(node.alternatives.len(), 3);
    assert_eq!(node.alternatives[0].children.len(), 2);
    assert_eq!(node.alternatives[1].children.len(), 1);
    assert_eq!(node.alternatives[2].children.len(), 3);
}

// ═══════════════════════════════════════════════════════════════════════
// 10. Unit tests — empty forest edge cases
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn empty_forest_no_roots() {
    let forest = empty_forest(minimal_grammar());
    assert!(forest.roots.is_empty());
    assert!(forest.nodes.is_empty());
    assert_eq!(forest.next_node_id, 0);
}

#[test]
fn empty_forest_preserves_grammar_name() {
    let forest = empty_forest(minimal_grammar());
    assert_eq!(forest.grammar.name, "mini");
}

#[test]
fn empty_forest_source_is_empty() {
    let forest = empty_forest(minimal_grammar());
    assert!(forest.source.is_empty());
}

// ═══════════════════════════════════════════════════════════════════════
// 11. Unit tests — single node forest
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn single_node_forest_basic() {
    let mut forest = empty_forest(minimal_grammar());
    let id = insert_node(&mut forest, SymbolId(42), (0, 5), vec![]);
    assert_eq!(id, 0);
    assert_eq!(forest.nodes.len(), 1);
    assert_eq!(forest.nodes[&0].symbol, SymbolId(42));
    assert_eq!(forest.nodes[&0].span, (0, 5));
}

#[test]
fn single_node_as_root() {
    let mut forest = empty_forest(minimal_grammar());
    let id = insert_node(&mut forest, SymbolId(1), (0, 1), vec![]);
    forest.roots.push(forest.nodes[&id].clone());
    assert_eq!(forest.roots.len(), 1);
    assert_eq!(forest.roots[0].id, 0);
}

// ═══════════════════════════════════════════════════════════════════════
// 12. Unit tests — deep nesting in ParseForest
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn deep_chain_depth_correct() {
    let mut forest = empty_forest(minimal_grammar());
    let mut current = insert_node(&mut forest, SymbolId(1), (0, 1), vec![]);
    for _ in 1..10 {
        current = insert_node(&mut forest, SymbolId(1), (0, 1), vec![current]);
    }
    assert_eq!(compute_depth(&forest, current), 10);
}

#[test]
fn deep_chain_all_nodes_reachable() {
    let mut forest = empty_forest(minimal_grammar());
    let mut current = insert_node(&mut forest, SymbolId(1), (0, 1), vec![]);
    for _ in 1..5 {
        current = insert_node(&mut forest, SymbolId(1), (0, 1), vec![current]);
    }
    // Walk from root to leaf
    let mut visited = 0;
    let mut id = current;
    loop {
        visited += 1;
        let node = &forest.nodes[&id];
        if node.alternatives[0].children.is_empty() {
            break;
        }
        id = node.alternatives[0].children[0];
    }
    assert_eq!(visited, 5);
}

// ═══════════════════════════════════════════════════════════════════════
// 13. Unit tests — error chunk edge cases
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn error_symbol_value() {
    assert_eq!(ERROR_SYMBOL, SymbolId(u16::MAX));
}

#[test]
fn error_chunk_zero_width_span() {
    let mut forest = empty_forest(minimal_grammar());
    let id = forest.push_error_chunk((5, 5));
    assert_eq!(forest.nodes[&id].span, (5, 5));
    assert!(forest.nodes[&id].error_meta.is_error);
}

#[test]
fn multiple_error_chunks_sequential() {
    let mut forest = empty_forest(minimal_grammar());
    let a = forest.push_error_chunk((0, 3));
    let b = forest.push_error_chunk((3, 7));
    let c = forest.push_error_chunk((7, 10));
    assert_eq!(a, 0);
    assert_eq!(b, 1);
    assert_eq!(c, 2);
    assert_eq!(forest.nodes.len(), 3);
    for id in [a, b, c] {
        assert_eq!(forest.nodes[&id].symbol, ERROR_SYMBOL);
    }
}

#[test]
fn mixed_regular_and_error_nodes() {
    let mut forest = empty_forest(minimal_grammar());
    let n0 = insert_node(&mut forest, SymbolId(1), (0, 1), vec![]);
    let e0 = forest.push_error_chunk((1, 3));
    let n1 = insert_node(&mut forest, SymbolId(2), (3, 5), vec![]);
    let e1 = forest.push_error_chunk((5, 6));

    let mut all_ids: Vec<usize> = vec![n0, e0, n1, e1];
    all_ids.sort();
    all_ids.dedup();
    assert_eq!(all_ids.len(), 4);
    assert_ne!(forest.nodes[&n0].symbol, ERROR_SYMBOL);
    assert_eq!(forest.nodes[&e0].symbol, ERROR_SYMBOL);
}

// ═══════════════════════════════════════════════════════════════════════
// 14. Unit tests — ErrorMeta
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn error_meta_default_all_zero() {
    let meta = ErrorMeta::default();
    assert!(!meta.missing);
    assert!(!meta.is_error);
    assert_eq!(meta.cost, 0);
}

#[test]
fn error_meta_missing_flag() {
    let meta = ErrorMeta {
        missing: true,
        is_error: false,
        cost: 5,
    };
    assert!(meta.missing);
    assert!(!meta.is_error);
}

#[test]
fn error_meta_is_error_flag() {
    let meta = ErrorMeta {
        missing: false,
        is_error: true,
        cost: 3,
    };
    assert!(!meta.missing);
    assert!(meta.is_error);
}

#[test]
fn error_meta_copy_works() {
    let a = ErrorMeta {
        missing: true,
        is_error: false,
        cost: 99,
    };
    let b = a;
    assert_eq!(a.cost, b.cost);
    assert_eq!(a.missing, b.missing);
}

// ═══════════════════════════════════════════════════════════════════════
// 15. Unit tests — ParseError
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn parse_error_incomplete() {
    let err = ParseError::Incomplete;
    assert_eq!(format!("{err}"), "Incomplete parse");
}

#[test]
fn parse_error_failed() {
    let err = ParseError::Failed("bad token".to_string());
    assert!(format!("{err}").contains("bad token"));
}

#[test]
fn parse_error_unknown() {
    assert_eq!(format!("{}", ParseError::Unknown), "Unknown error");
}

#[test]
fn parse_error_debug() {
    let err = ParseError::Failed("xyz".to_string());
    let dbg = format!("{err:?}");
    assert!(dbg.contains("Failed"));
    assert!(dbg.contains("xyz"));
}

#[test]
fn parse_error_clone() {
    let err = ParseError::Failed("msg".to_string());
    let cloned = err.clone();
    assert_eq!(format!("{cloned}"), "Parse failed: msg");
}

// ═══════════════════════════════════════════════════════════════════════
// 16. Unit tests — ParseTree / ParseNode
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn parse_tree_stores_source() {
    let tree = ParseTree {
        root: ParseNode {
            symbol: SymbolId(1),
            span: (0, 3),
            children: vec![],
        },
        source: "abc".to_string(),
    };
    assert_eq!(tree.source, "abc");
}

#[test]
fn parse_node_leaf_has_no_children() {
    let node = ParseNode {
        symbol: SymbolId(1),
        span: (0, 1),
        children: vec![],
    };
    assert!(node.children.is_empty());
}

#[test]
fn parse_node_wide() {
    let node = ParseNode {
        symbol: SymbolId(1),
        span: (0, 10),
        children: (0..5)
            .map(|i| ParseNode {
                symbol: SymbolId(i + 2),
                span: (i as usize * 2, i as usize * 2 + 2),
                children: vec![],
            })
            .collect(),
    };
    assert_eq!(node.children.len(), 5);
}

#[test]
fn parse_tree_debug_contains_source() {
    let tree = ParseTree {
        root: ParseNode {
            symbol: SymbolId(1),
            span: (0, 5),
            children: vec![],
        },
        source: "hello".to_string(),
    };
    let dbg = format!("{tree:?}");
    assert!(dbg.contains("hello"));
}

// ═══════════════════════════════════════════════════════════════════════
// 17. Unit tests — ForestView through Driver
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn single_token_view_one_root() {
    let (_g, forest) = single_token_forest();
    assert_eq!(forest.view().roots().len(), 1);
}

#[test]
fn single_token_view_root_span() {
    let (_g, forest) = single_token_forest();
    let view = forest.view();
    let sp = view.span(view.roots()[0]);
    assert_eq!(sp.start, 0);
    assert_eq!(sp.end, 1);
}

#[test]
fn expr_root_kind_is_start_symbol() {
    let (grammar, forest) = expr_forest();
    let view = forest.view();
    let root_kind = view.kind(view.roots()[0]);
    let expr_sym = sym_id(&grammar, "expr");
    assert_eq!(root_kind, expr_sym.0 as u32);
}

#[test]
fn expr_children_within_root_span() {
    let (_g, forest) = expr_forest();
    let view = forest.view();
    let root = view.roots()[0];
    let root_sp = view.span(root);
    for &child in view.best_children(root) {
        let csp = view.span(child);
        assert!(csp.start >= root_sp.start);
        assert!(csp.end <= root_sp.end);
    }
}

#[test]
fn expr_recursive_walk_covers_nodes() {
    let (_g, forest) = expr_forest();
    let view = forest.view();
    let all = collect_node_ids(view, view.roots()[0]);
    assert!(all.len() >= 4);
}

// ═══════════════════════════════════════════════════════════════════════
// 18. Unit tests — ForestView nonexistent node edge cases
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn nonexistent_node_kind_zero() {
    let (_g, forest) = single_token_forest();
    assert_eq!(forest.view().kind(999_999), 0);
}

#[test]
fn nonexistent_node_span_zero() {
    let (_g, forest) = single_token_forest();
    let sp = forest.view().span(999_999);
    assert_eq!(sp, Span { start: 0, end: 0 });
}

#[test]
fn nonexistent_node_empty_children() {
    let (_g, forest) = single_token_forest();
    assert!(forest.view().best_children(999_999).is_empty());
}

// ═══════════════════════════════════════════════════════════════════════
// 19. Unit tests — ForestView depth and node count
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn single_token_tree_depth_at_least_two() {
    let (_g, forest) = single_token_forest();
    let d = tree_depth(forest.view(), forest.view().roots()[0]);
    assert!(d >= 2);
}

#[test]
fn expr_tree_depth_at_least_three() {
    let (_g, forest) = expr_forest();
    let d = tree_depth(forest.view(), forest.view().roots()[0]);
    assert!(d >= 3);
}

#[test]
fn expr_node_count_at_least_four() {
    let (_g, forest) = expr_forest();
    let count = node_count(forest.view(), forest.view().roots()[0]);
    assert!(count >= 4);
}

// ═══════════════════════════════════════════════════════════════════════
// 20. Unit tests — Span type
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn span_equality() {
    let a = Span { start: 1, end: 5 };
    let b = Span { start: 1, end: 5 };
    assert_eq!(a, b);
}

#[test]
fn span_inequality() {
    let a = Span { start: 0, end: 3 };
    let b = Span { start: 0, end: 4 };
    assert_ne!(a, b);
}

#[test]
fn span_debug() {
    let s = Span { start: 10, end: 20 };
    let dbg = format!("{s:?}");
    assert!(dbg.contains("10"));
    assert!(dbg.contains("20"));
}

#[test]
fn span_clone() {
    let a = Span { start: 3, end: 7 };
    let b = a;
    assert_eq!(a, b);
}

// ═══════════════════════════════════════════════════════════════════════
// 21. Unit tests — ForestView is Send + Sync
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn forest_view_is_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<Box<dyn ForestView>>();
}

// ═══════════════════════════════════════════════════════════════════════
// 22. Unit tests — ForestAlternative
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn forest_alternative_empty_is_epsilon() {
    let alt = ForestAlternative { children: vec![] };
    assert!(alt.children.is_empty());
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
fn forest_alternative_debug() {
    let alt = ForestAlternative { children: vec![42] };
    let dbg = format!("{alt:?}");
    assert!(dbg.contains("42"));
}

// ═══════════════════════════════════════════════════════════════════════
// 23. Unit tests — shared children across alternatives
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn shared_children_across_alternatives() {
    let mut forest = empty_forest(minimal_grammar());
    let shared = insert_node(&mut forest, SymbolId(1), (0, 1), vec![]);
    let left = insert_node(&mut forest, SymbolId(2), (1, 2), vec![]);
    let right = insert_node(&mut forest, SymbolId(3), (1, 2), vec![]);

    let id = forest.next_node_id;
    forest.next_node_id += 1;
    forest.nodes.insert(
        id,
        ForestNode {
            id,
            symbol: SymbolId(10),
            span: (0, 2),
            alternatives: vec![
                ForestAlternative {
                    children: vec![shared, left],
                },
                ForestAlternative {
                    children: vec![shared, right],
                },
            ],
            error_meta: ErrorMeta::default(),
        },
    );

    let parent = &forest.nodes[&id];
    assert_eq!(parent.alternatives[0].children[0], shared);
    assert_eq!(parent.alternatives[1].children[0], shared);
    assert_ne!(
        parent.alternatives[0].children[1],
        parent.alternatives[1].children[1]
    );
}

// ═══════════════════════════════════════════════════════════════════════
// 24. Unit tests — ForestNode debug format
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn forest_node_debug_contains_id() {
    let node = ForestNode {
        id: 77,
        symbol: SymbolId(5),
        span: (10, 20),
        alternatives: vec![],
        error_meta: ErrorMeta::default(),
    };
    let dbg = format!("{node:?}");
    assert!(dbg.contains("77"));
}

// ═══════════════════════════════════════════════════════════════════════
// 25. Property test — node IDs in forest are unique set
// ═══════════════════════════════════════════════════════════════════════

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    #[test]
    fn prop_all_node_ids_unique(count in 2usize..30) {
        let mut forest = empty_forest(minimal_grammar());
        let mut ids = Vec::new();
        for i in 0..count {
            if i % 3 == 0 {
                ids.push(forest.push_error_chunk((i, i + 1)));
            } else {
                ids.push(insert_node(&mut forest, SymbolId(1), (i, i + 1), vec![]));
            }
        }
        let set: HashSet<usize> = ids.iter().copied().collect();
        prop_assert_eq!(set.len(), count);
    }
}

// ═══════════════════════════════════════════════════════════════════════
// 26. Unit tests — two-alternative grammar parse
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn two_alt_grammar_parses_first() {
    let mut grammar = GrammarBuilder::new("alt")
        .token("a", "a")
        .token("b", "b")
        .rule("S", vec!["a"])
        .rule("S", vec!["b"])
        .start("S")
        .build();
    let a = sym_id(&grammar, "a");
    let forest = pipeline_parse(&mut grammar, &[(a, 0, 1)]).expect("parse a");
    assert_eq!(forest.view().roots().len(), 1);
}

#[test]
fn two_alt_grammar_parses_second() {
    let mut grammar = GrammarBuilder::new("alt")
        .token("a", "a")
        .token("b", "b")
        .rule("S", vec!["a"])
        .rule("S", vec!["b"])
        .start("S")
        .build();
    let b = sym_id(&grammar, "b");
    let forest = pipeline_parse(&mut grammar, &[(b, 0, 1)]).expect("parse b");
    assert_eq!(forest.view().roots().len(), 1);
}

// ═══════════════════════════════════════════════════════════════════════
// 27. Unit test — longer chain keeps single root
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn longer_chain_single_root() {
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
        &[
            (num, 0, 1),
            (plus, 1, 2),
            (num, 2, 3),
            (plus, 3, 4),
            (num, 4, 5),
        ],
    )
    .expect("chain parse");
    let view = forest.view();
    assert_eq!(view.roots().len(), 1);
    assert_eq!(view.span(view.roots()[0]).end, 5);
}

// ═══════════════════════════════════════════════════════════════════════
// 28. Property test — ForestView child spans within parent
// ═══════════════════════════════════════════════════════════════════════

proptest! {
    #![proptest_config(ProptestConfig::with_cases(16))]

    #[test]
    fn prop_child_spans_within_parent(_seed in 0u32..8) {
        let (_g, forest) = expr_forest();
        let view = forest.view();
        let root = view.roots()[0];
        fn check_spans(view: &dyn ForestView, id: u32) {
            let parent_sp = view.span(id);
            for &child in view.best_children(id) {
                let csp = view.span(child);
                assert!(csp.start >= parent_sp.start);
                assert!(csp.end <= parent_sp.end);
                check_spans(view, child);
            }
        }
        check_spans(view, root);
    }
}

// ═══════════════════════════════════════════════════════════════════════
// 29. Unit tests — Forest wrapper type
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn forest_view_returns_dyn_ref() {
    let (_g, forest) = single_token_forest();
    let _view: &dyn ForestView = forest.view();
}

#[test]
fn forest_from_expr_has_nodes() {
    let (_g, forest) = expr_forest();
    let view = forest.view();
    let all = collect_node_ids(view, view.roots()[0]);
    assert!(!all.is_empty());
}

// ═══════════════════════════════════════════════════════════════════════
// 30. Property test — node ID matches map key in ParseForest
// ═══════════════════════════════════════════════════════════════════════

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn prop_node_id_matches_map_key(count in 1usize..20) {
        let mut forest = empty_forest(minimal_grammar());
        for _ in 0..count {
            insert_node(&mut forest, SymbolId(1), (0, 1), vec![]);
        }
        for (&key, node) in &forest.nodes {
            prop_assert_eq!(key, node.id);
        }
    }

    #[test]
    fn prop_error_chunk_id_matches_key(count in 1usize..15) {
        let mut forest = empty_forest(minimal_grammar());
        for i in 0..count {
            forest.push_error_chunk((i, i + 1));
        }
        for (&key, node) in &forest.nodes {
            prop_assert_eq!(key, node.id);
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════
// 31. Unit tests — multiple roots in ParseForest
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn multiple_roots_different_symbols() {
    let mut forest = empty_forest(minimal_grammar());
    let r1 = insert_node(&mut forest, SymbolId(1), (0, 1), vec![]);
    let r2 = insert_node(&mut forest, SymbolId(2), (0, 1), vec![]);
    forest.roots.push(forest.nodes[&r1].clone());
    forest.roots.push(forest.nodes[&r2].clone());
    assert_eq!(forest.roots.len(), 2);
    assert_ne!(forest.roots[0].symbol, forest.roots[1].symbol);
}

#[test]
fn multiple_roots_different_spans() {
    let mut forest = empty_forest(minimal_grammar());
    let r1 = insert_node(&mut forest, SymbolId(1), (0, 3), vec![]);
    let r2 = insert_node(&mut forest, SymbolId(1), (0, 5), vec![]);
    forest.roots.push(forest.nodes[&r1].clone());
    forest.roots.push(forest.nodes[&r2].clone());
    assert_eq!(forest.roots[0].span, (0, 3));
    assert_eq!(forest.roots[1].span, (0, 5));
}

// ═══════════════════════════════════════════════════════════════════════
// 32. Property test — valid ErrorMeta never triggers invariant violation
// ═══════════════════════════════════════════════════════════════════════

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn prop_valid_meta_no_invariant_violation(meta in arb_error_meta_valid()) {
        // A node with valid meta should not have both is_error and missing true
        prop_assert!(!(meta.is_error && meta.missing));
    }

    #[test]
    fn prop_forest_source_preserved(src in "[a-z]{0,30}") {
        let grammar = minimal_grammar();
        let forest = ParseForest {
            roots: Vec::new(),
            nodes: HashMap::new(),
            grammar,
            source: src.clone(),
            next_node_id: 0,
        };
        prop_assert_eq!(&forest.source, &src);
    }
}

// ═══════════════════════════════════════════════════════════════════════
// 33. Unit test — zero-width span nodes
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn zero_width_span_regular_node() {
    let mut forest = empty_forest(minimal_grammar());
    let id = insert_node(&mut forest, SymbolId(1), (5, 5), vec![]);
    let node = &forest.nodes[&id];
    assert_eq!(node.span.0, node.span.1);
}

#[test]
fn zero_width_error_chunk() {
    let mut forest = empty_forest(minimal_grammar());
    let id = forest.push_error_chunk((3, 3));
    assert_eq!(forest.nodes[&id].span, (3, 3));
}

// ═══════════════════════════════════════════════════════════════════════
// 34. Unit test — forest clone preserves full structure
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn clone_preserves_tree_structure() {
    let mut forest = empty_forest(minimal_grammar());
    let leaf = insert_node(&mut forest, SymbolId(1), (0, 1), vec![]);
    let mid = insert_node(&mut forest, SymbolId(2), (0, 1), vec![leaf]);
    let root = insert_node(&mut forest, SymbolId(3), (0, 1), vec![mid]);
    forest.roots.push(forest.nodes[&root].clone());

    let cloned = forest.clone();
    assert_eq!(cloned.nodes.len(), 3);
    assert_eq!(cloned.roots.len(), 1);
    assert_eq!(cloned.roots[0].symbol, SymbolId(3));
    assert_eq!(cloned.nodes[&mid].alternatives[0].children, vec![leaf]);
}
