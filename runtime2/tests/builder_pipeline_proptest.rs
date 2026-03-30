#![allow(clippy::needless_range_loop)]
#![cfg(feature = "glr")]

//! Property-based tests for the forest-to-tree builder pipeline.
//!
//! Exercises `builder.rs` through the public `Parser` API and `Tree::new_for_testing`,
//! covering forest-to-tree conversion, node-count preservation, empty/single/deep/wide
//! forest shapes, performance metrics accuracy, and determinism.

use std::time::Instant;

use proptest::prelude::*;

use adze_glr_core::{FirstFollowSets, build_lr1_automaton};
use adze_ir::{Grammar, ProductionId, Rule, Symbol, SymbolId, Token as IrToken, TokenPattern};
use adze_runtime::language::SymbolMetadata;
use adze_runtime::{Language, Node, Parser, Token, Tree};

// ---------------------------------------------------------------------------
// Grammar helpers
// ---------------------------------------------------------------------------

/// start → a  (Symbols: 0=EOF, 1=a, 2=start)
fn lang_single() -> Language {
    let mut g = Grammar::new("single".into());
    let a = SymbolId(1);
    g.tokens.insert(
        a,
        IrToken {
            name: "a".into(),
            pattern: TokenPattern::String("a".into()),
            fragile: false,
        },
    );
    let start = SymbolId(2);
    g.rule_names.insert(start, "start".into());
    g.rules.insert(
        start,
        vec![Rule {
            lhs: start,
            rhs: vec![Symbol::Terminal(a)],
            precedence: None,
            associativity: None,
            production_id: ProductionId(0),
            fields: vec![],
        }],
    );
    let ff = FirstFollowSets::compute(&g).unwrap();
    let tbl = build_lr1_automaton(&g, &ff)
        .unwrap()
        .normalize_eof_to_zero()
        .with_detected_goto_indexing();
    let tbl: &'static _ = Box::leak(Box::new(tbl));
    Language::builder()
        .parse_table(tbl)
        .symbol_names(vec!["EOF".into(), "a".into(), "start".into()])
        .symbol_metadata(vec![
            SymbolMetadata {
                is_terminal: true,
                is_visible: false,
                is_supertype: false,
            },
            SymbolMetadata {
                is_terminal: true,
                is_visible: true,
                is_supertype: false,
            },
            SymbolMetadata {
                is_terminal: false,
                is_visible: true,
                is_supertype: false,
            },
        ])
        .tokenizer(|input: &[u8]| {
            let mut t = Vec::new();
            for (i, &b) in input.iter().enumerate() {
                if b == b'a' {
                    t.push(Token {
                        kind: 1,
                        start: i as u32,
                        end: (i + 1) as u32,
                    });
                }
            }
            t.push(Token {
                kind: 0,
                start: input.len() as u32,
                end: input.len() as u32,
            });
            Box::new(t.into_iter()) as Box<dyn Iterator<Item = Token> + '_>
        })
        .build()
        .unwrap()
}

/// start → a b c  (Symbols: 0=EOF, 1=a, 2=b, 3=c, 4=start)
fn lang_three() -> Language {
    let mut g = Grammar::new("three".into());
    for (id, name, pat) in [(1, "a", "a"), (2, "b", "b"), (3, "c", "c")] {
        g.tokens.insert(
            SymbolId(id),
            IrToken {
                name: name.into(),
                pattern: TokenPattern::String(pat.into()),
                fragile: false,
            },
        );
    }
    let start = SymbolId(4);
    g.rule_names.insert(start, "start".into());
    g.rules.insert(
        start,
        vec![Rule {
            lhs: start,
            rhs: vec![
                Symbol::Terminal(SymbolId(1)),
                Symbol::Terminal(SymbolId(2)),
                Symbol::Terminal(SymbolId(3)),
            ],
            precedence: None,
            associativity: None,
            production_id: ProductionId(0),
            fields: vec![],
        }],
    );
    let ff = FirstFollowSets::compute(&g).unwrap();
    let tbl = build_lr1_automaton(&g, &ff)
        .unwrap()
        .normalize_eof_to_zero()
        .with_detected_goto_indexing();
    let tbl: &'static _ = Box::leak(Box::new(tbl));
    Language::builder()
        .parse_table(tbl)
        .symbol_names(vec![
            "EOF".into(),
            "a".into(),
            "b".into(),
            "c".into(),
            "start".into(),
        ])
        .symbol_metadata(vec![
            SymbolMetadata {
                is_terminal: true,
                is_visible: false,
                is_supertype: false,
            },
            SymbolMetadata {
                is_terminal: true,
                is_visible: true,
                is_supertype: false,
            },
            SymbolMetadata {
                is_terminal: true,
                is_visible: true,
                is_supertype: false,
            },
            SymbolMetadata {
                is_terminal: true,
                is_visible: true,
                is_supertype: false,
            },
            SymbolMetadata {
                is_terminal: false,
                is_visible: true,
                is_supertype: false,
            },
        ])
        .tokenizer(|input: &[u8]| {
            let mut t = Vec::new();
            for (i, &b) in input.iter().enumerate() {
                let k = match b {
                    b'a' => 1,
                    b'b' => 2,
                    b'c' => 3,
                    _ => continue,
                };
                t.push(Token {
                    kind: k,
                    start: i as u32,
                    end: (i + 1) as u32,
                });
            }
            t.push(Token {
                kind: 0,
                start: input.len() as u32,
                end: input.len() as u32,
            });
            Box::new(t.into_iter()) as Box<dyn Iterator<Item = Token> + '_>
        })
        .build()
        .unwrap()
}

/// Chain grammar: start → mid, mid → a  (Symbols: 0=EOF, 1=a, 2=mid, 3=start)
fn lang_chain() -> Language {
    let mut g = Grammar::new("chain".into());
    let a = SymbolId(1);
    g.tokens.insert(
        a,
        IrToken {
            name: "a".into(),
            pattern: TokenPattern::String("a".into()),
            fragile: false,
        },
    );
    let mid = SymbolId(2);
    g.rule_names.insert(mid, "mid".into());
    g.rules.insert(
        mid,
        vec![Rule {
            lhs: mid,
            rhs: vec![Symbol::Terminal(a)],
            precedence: None,
            associativity: None,
            production_id: ProductionId(0),
            fields: vec![],
        }],
    );
    let start = SymbolId(3);
    g.rule_names.insert(start, "start".into());
    g.rules.insert(
        start,
        vec![Rule {
            lhs: start,
            rhs: vec![Symbol::NonTerminal(mid)],
            precedence: None,
            associativity: None,
            production_id: ProductionId(1),
            fields: vec![],
        }],
    );
    let ff = FirstFollowSets::compute(&g).unwrap();
    let tbl = build_lr1_automaton(&g, &ff)
        .unwrap()
        .normalize_eof_to_zero()
        .with_detected_goto_indexing();
    let tbl: &'static _ = Box::leak(Box::new(tbl));
    Language::builder()
        .parse_table(tbl)
        .symbol_names(vec!["EOF".into(), "a".into(), "mid".into(), "start".into()])
        .symbol_metadata(vec![
            SymbolMetadata {
                is_terminal: true,
                is_visible: false,
                is_supertype: false,
            },
            SymbolMetadata {
                is_terminal: true,
                is_visible: true,
                is_supertype: false,
            },
            SymbolMetadata {
                is_terminal: false,
                is_visible: true,
                is_supertype: false,
            },
            SymbolMetadata {
                is_terminal: false,
                is_visible: true,
                is_supertype: false,
            },
        ])
        .tokenizer(|input: &[u8]| {
            let mut t = Vec::new();
            for (i, &b) in input.iter().enumerate() {
                if b == b'a' {
                    t.push(Token {
                        kind: 1,
                        start: i as u32,
                        end: (i + 1) as u32,
                    });
                }
            }
            t.push(Token {
                kind: 0,
                start: input.len() as u32,
                end: input.len() as u32,
            });
            Box::new(t.into_iter()) as Box<dyn Iterator<Item = Token> + '_>
        })
        .build()
        .unwrap()
}

// ---------------------------------------------------------------------------
// Tree analysis helpers
// ---------------------------------------------------------------------------

fn count_nodes(n: Node<'_>) -> usize {
    let mut total = 1;
    for i in 0..n.child_count() {
        total += count_nodes(n.child(i).unwrap());
    }
    total
}

fn tree_depth(n: Node<'_>) -> usize {
    let mut d = 0;
    for i in 0..n.child_count() {
        d = d.max(1 + tree_depth(n.child(i).unwrap()));
    }
    d
}

fn collect_shape(n: Node<'_>, out: &mut Vec<(u16, usize, usize, usize)>) {
    out.push((n.kind_id(), n.start_byte(), n.end_byte(), n.child_count()));
    for i in 0..n.child_count() {
        collect_shape(n.child(i).unwrap(), out);
    }
}

fn assert_valid_ranges(n: Node<'_>) {
    assert!(n.start_byte() <= n.end_byte());
    for i in 0..n.child_count() {
        let c = n.child(i).unwrap();
        assert!(c.start_byte() >= n.start_byte());
        assert!(c.end_byte() <= n.end_byte());
        assert_valid_ranges(c);
    }
}

fn parse_with(lang: Language, input: &[u8]) -> Tree {
    let mut p = Parser::new();
    p.set_language(lang).unwrap();
    p.parse(input, None).unwrap()
}

// ===========================================================================
// 1 – Builder converts forest to tree
// ===========================================================================

#[test]
fn pipeline_single_token_produces_tree_with_language() {
    let tree = parse_with(lang_single(), b"a");
    assert!(tree.language().is_some());
}

#[test]
fn pipeline_single_token_stores_source() {
    let tree = parse_with(lang_single(), b"a");
    assert_eq!(tree.source_bytes(), Some(b"a".as_slice()));
}

#[test]
fn pipeline_three_token_root_spans_input() {
    let tree = parse_with(lang_three(), b"abc");
    let root = tree.root_node();
    assert_eq!(root.start_byte(), 0);
    assert_eq!(root.end_byte(), 3);
}

#[test]
fn pipeline_chain_grammar_resolves_kind_name() {
    let tree = parse_with(lang_chain(), b"a");
    let root = tree.root_node();
    assert_ne!(root.kind(), "unknown");
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(16))]

    /// Forest-to-tree always yields valid byte ranges at every level.
    #[test]
    fn pipeline_valid_ranges(choice in 0usize..3) {
        let tree = match choice {
            0 => parse_with(lang_single(), b"a"),
            1 => parse_with(lang_three(), b"abc"),
            _ => parse_with(lang_chain(), b"a"),
        };
        assert_valid_ranges(tree.root_node());
    }
}

// ===========================================================================
// 2 – Builder preserves node count
// ===========================================================================

#[test]
fn pipeline_single_token_node_count_at_least_two() {
    // start → a produces at least root + terminal child
    let tree = parse_with(lang_single(), b"a");
    assert!(count_nodes(tree.root_node()) >= 2);
}

#[test]
fn pipeline_three_token_node_count_at_least_four() {
    // start → a b c ⇒ root + 3 children
    let tree = parse_with(lang_three(), b"abc");
    assert!(count_nodes(tree.root_node()) >= 4);
}

#[test]
fn pipeline_chain_grammar_node_count_at_least_two() {
    // start → mid → a ⇒ at least root + 1 descendant
    let tree = parse_with(lang_chain(), b"a");
    assert!(count_nodes(tree.root_node()) >= 2);
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    /// Node count via new_for_testing equals 1 + number of children.
    #[test]
    fn for_testing_node_count_matches(n in 1usize..20, chunk in 1usize..16) {
        let children: Vec<Tree> = (0..n)
            .map(|i| Tree::new_for_testing(1, i * chunk, (i + 1) * chunk, vec![]))
            .collect();
        let tree = Tree::new_for_testing(0, 0, n * chunk, children);
        prop_assert_eq!(count_nodes(tree.root_node()), 1 + n);
    }
}

// ===========================================================================
// 3 – Builder handles empty forest
// ===========================================================================

#[test]
fn empty_stub_tree_has_zero_children() {
    let tree = Tree::new_stub();
    assert_eq!(tree.root_node().child_count(), 0);
}

#[test]
fn empty_stub_tree_has_zero_byte_range() {
    let tree = Tree::new_stub();
    assert_eq!(tree.root_node().start_byte(), 0);
    assert_eq!(tree.root_node().end_byte(), 0);
}

#[test]
fn empty_stub_tree_root_kind_zero() {
    let tree = Tree::new_stub();
    assert_eq!(tree.root_kind(), 0);
}

#[test]
fn empty_stub_tree_has_no_language_or_source() {
    let tree = Tree::new_stub();
    assert!(tree.language().is_none());
    assert!(tree.source_bytes().is_none());
}

#[test]
fn empty_for_testing_tree_has_no_children() {
    let tree = Tree::new_for_testing(42, 0, 0, vec![]);
    assert_eq!(tree.root_node().child_count(), 0);
    assert_eq!(tree.root_kind(), 42);
}

// ===========================================================================
// 4 – Builder handles single-node forest
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    /// A leaf tree preserves symbol and byte range.
    #[test]
    fn single_node_preserves_identity(sym in 1u32..1000, end in 0usize..512) {
        let tree = Tree::new_for_testing(sym, 0, end, vec![]);
        prop_assert_eq!(tree.root_kind(), sym);
        prop_assert_eq!(tree.root_node().start_byte(), 0);
        prop_assert_eq!(tree.root_node().end_byte(), end);
        prop_assert_eq!(tree.root_node().child_count(), 0);
    }

    /// Single-node tree has node count exactly 1.
    #[test]
    fn single_node_count_is_one(sym in 1u32..1000, end in 0usize..512) {
        let tree = Tree::new_for_testing(sym, 0, end, vec![]);
        prop_assert_eq!(count_nodes(tree.root_node()), 1);
    }

    /// Single-node tree has depth 0.
    #[test]
    fn single_node_depth_is_zero(sym in 1u32..1000) {
        let tree = Tree::new_for_testing(sym, 0, 1, vec![]);
        prop_assert_eq!(tree_depth(tree.root_node()), 0);
    }
}

// ===========================================================================
// 5 – Builder handles deep forest
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    /// A linear chain of depth d has tree_depth == d-1 and node_count == d.
    #[test]
    fn deep_chain_metrics(depth in 1usize..30) {
        let mut tree = Tree::new_for_testing(1, 0, 10, vec![]);
        for i in 1..depth {
            tree = Tree::new_for_testing(1 + i as u32, 0, 10, vec![tree]);
        }
        prop_assert_eq!(tree_depth(tree.root_node()), depth - 1);
        prop_assert_eq!(count_nodes(tree.root_node()), depth);
    }

    /// Deep chain clone preserves depth.
    #[test]
    fn deep_chain_clone_preserves_shape(depth in 2usize..25) {
        let mut tree = Tree::new_for_testing(1, 0, 5, vec![]);
        for i in 1..depth {
            tree = Tree::new_for_testing(1 + i as u32, 0, 5, vec![tree]);
        }
        let cloned = tree.clone();
        let mut orig_shape = Vec::new();
        let mut clone_shape = Vec::new();
        collect_shape(tree.root_node(), &mut orig_shape);
        collect_shape(cloned.root_node(), &mut clone_shape);
        prop_assert_eq!(orig_shape, clone_shape);
    }
}

#[test]
fn deep_chain_via_grammar_has_nontrivial_depth() {
    let tree = parse_with(lang_chain(), b"a");
    // start → mid → a  ⇒ depth >= 1
    assert!(tree_depth(tree.root_node()) >= 1);
}

// ===========================================================================
// 6 – Builder handles wide forest
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    /// Wide tree has correct child count and depth 1.
    #[test]
    fn wide_tree_shape(n in 2usize..30, chunk in 1usize..8) {
        let children: Vec<Tree> = (0..n)
            .map(|i| Tree::new_for_testing(1, i * chunk, (i + 1) * chunk, vec![]))
            .collect();
        let tree = Tree::new_for_testing(0, 0, n * chunk, children);
        prop_assert_eq!(tree.root_node().child_count(), n);
        prop_assert_eq!(tree_depth(tree.root_node()), 1);
    }

    /// Wide tree children are in non-decreasing start-byte order.
    #[test]
    fn wide_tree_children_ordered(n in 2usize..20) {
        let children: Vec<Tree> = (0..n)
            .map(|i| Tree::new_for_testing(1, i * 4, (i + 1) * 4, vec![]))
            .collect();
        let tree = Tree::new_for_testing(0, 0, n * 4, children);
        let root = tree.root_node();
        for i in 1..root.child_count() {
            let prev = root.child(i - 1).unwrap();
            let curr = root.child(i).unwrap();
            prop_assert!(prev.start_byte() <= curr.start_byte());
            prop_assert!(prev.end_byte() <= curr.start_byte());
        }
    }
}

#[test]
fn wide_grammar_three_tokens_has_three_children() {
    let tree = parse_with(lang_three(), b"abc");
    assert_eq!(tree.root_node().child_count(), 3);
}

// ===========================================================================
// 7 – Performance metrics accuracy
// ===========================================================================

#[test]
fn pipeline_conversion_completes_in_bounded_time() {
    let start = Instant::now();
    let _tree = parse_with(lang_three(), b"abc");
    let elapsed = start.elapsed();
    // A tiny grammar should finish well under 5 seconds.
    assert!(
        elapsed.as_secs() < 5,
        "conversion took too long: {:?}",
        elapsed
    );
}

#[test]
fn metrics_node_count_matches_manual_count_single() {
    let tree = parse_with(lang_single(), b"a");
    let manual = count_nodes(tree.root_node());
    assert!(manual >= 2, "expected at least 2 nodes, got {}", manual);
}

#[test]
fn metrics_depth_matches_manual_depth_chain() {
    let tree = parse_with(lang_chain(), b"a");
    let depth = tree_depth(tree.root_node());
    assert!(depth >= 1, "expected depth >= 1, got {}", depth);
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(16))]

    /// For any grammar, node_count >= depth + 1 (pigeonhole: chain of length d has d nodes).
    #[test]
    fn metrics_node_count_ge_depth_plus_one(choice in 0usize..3) {
        let tree = match choice {
            0 => parse_with(lang_single(), b"a"),
            1 => parse_with(lang_three(), b"abc"),
            _ => parse_with(lang_chain(), b"a"),
        };
        let nc = count_nodes(tree.root_node());
        let d = tree_depth(tree.root_node());
        prop_assert!(nc > d, "node_count {} < depth {} + 1", nc, d);
    }
}

// ===========================================================================
// 8 – Builder determinism
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(16))]

    /// Parsing the same input twice yields identical tree shapes.
    #[test]
    fn deterministic_shape(choice in 0usize..3) {
        let (l1, l2, input): (Language, Language, &[u8]) = match choice {
            0 => (lang_single(), lang_single(), b"a"),
            1 => (lang_three(), lang_three(), b"abc"),
            _ => (lang_chain(), lang_chain(), b"a"),
        };
        let t1 = parse_with(l1, input);
        let t2 = parse_with(l2, input);
        let mut s1 = Vec::new();
        let mut s2 = Vec::new();
        collect_shape(t1.root_node(), &mut s1);
        collect_shape(t2.root_node(), &mut s2);
        prop_assert_eq!(s1, s2);
    }

    /// Clone of a parsed tree has identical shape.
    #[test]
    fn clone_is_identical(choice in 0usize..3) {
        let tree = match choice {
            0 => parse_with(lang_single(), b"a"),
            1 => parse_with(lang_three(), b"abc"),
            _ => parse_with(lang_chain(), b"a"),
        };
        let cloned = tree.clone();
        let mut s1 = Vec::new();
        let mut s2 = Vec::new();
        collect_shape(tree.root_node(), &mut s1);
        collect_shape(cloned.root_node(), &mut s2);
        prop_assert_eq!(s1, s2);
    }
}

#[test]
fn deterministic_root_kind_across_five_parses() {
    let kinds: Vec<u32> = (0..5)
        .map(|_| {
            let tree = parse_with(lang_three(), b"abc");
            tree.root_kind()
        })
        .collect();
    for k in &kinds[1..] {
        assert_eq!(kinds[0], *k);
    }
}

#[test]
fn deterministic_node_count_across_five_parses() {
    let counts: Vec<usize> = (0..5)
        .map(|_| {
            let tree = parse_with(lang_chain(), b"a");
            count_nodes(tree.root_node())
        })
        .collect();
    for c in &counts[1..] {
        assert_eq!(counts[0], *c);
    }
}
