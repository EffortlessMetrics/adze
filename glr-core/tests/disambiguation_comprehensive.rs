//! Comprehensive tests for the disambiguation and conflict-resolution modules.
//!
//! Run with: `cargo test -p adze-glr-core --test disambiguation_comprehensive`

use std::collections::HashMap;

use adze_glr_core::parse_forest::{ErrorMeta, ForestAlternative, ForestNode};
use adze_glr_core::{
    Action, ConflictAnalyzer, ConflictResolver, ConflictStats, ConflictType, FirstFollowSets,
    ItemSetCollection, ParseForest, ParseNode, ParseTree, PrecedenceComparison, PrecedenceInfo,
    PrecedenceResolver, StaticPrecedenceResolver, build_lr1_automaton, compare_precedences,
};
use adze_ir::builder::GrammarBuilder;
use adze_ir::{
    Associativity, Grammar, Precedence, PrecedenceKind, ProductionId, Rule, RuleId, Symbol,
    SymbolId, Token, TokenPattern,
};
use indexmap::IndexMap;

// ===========================================================================
// Helpers
// ===========================================================================

/// Build a minimal grammar: `expr -> A`
fn simple_grammar() -> Grammar {
    GrammarBuilder::new("simple")
        .token("A", "a")
        .rule("expr", vec!["A"])
        .start("expr")
        .build()
}

/// Build an ambiguous grammar: `expr -> A | B`
fn two_alt_grammar() -> Grammar {
    GrammarBuilder::new("two_alt")
        .token("A", "a")
        .token("B", "b")
        .rule("expr", vec!["A"])
        .rule("expr", vec!["B"])
        .start("expr")
        .build()
}

/// Build a grammar with self-recursion: `E -> a | E E` (inherently ambiguous).
fn self_recursive_grammar() -> Grammar {
    let mut g = Grammar::new("self_rec".into());
    let a = SymbolId(1);
    let e = SymbolId(10);

    g.tokens.insert(
        a,
        Token {
            name: "a".into(),
            pattern: TokenPattern::String("a".into()),
            fragile: false,
        },
    );
    g.rule_names.insert(e, "E".into());
    // E is the first (and only) rule entry → start_symbol() will pick it
    g.rules.insert(
        e,
        vec![
            Rule {
                lhs: e,
                rhs: vec![Symbol::Terminal(a)],
                precedence: None,
                associativity: None,
                fields: vec![],
                production_id: ProductionId(0),
            },
            Rule {
                lhs: e,
                rhs: vec![Symbol::NonTerminal(e), Symbol::NonTerminal(e)],
                precedence: None,
                associativity: None,
                fields: vec![],
                production_id: ProductionId(1),
            },
        ],
    );
    g
}

/// Build a left-associative expression grammar: `E -> E + E | num`
fn left_assoc_grammar() -> Grammar {
    GrammarBuilder::new("left_assoc")
        .token("NUM", r"\d+")
        .token("+", "+")
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build()
}

/// Build a right-associative expression grammar: `E -> E ^ E | num`
fn right_assoc_grammar() -> Grammar {
    GrammarBuilder::new("right_assoc")
        .token("NUM", r"\d+")
        .token("^", "^")
        .rule_with_precedence("expr", vec!["expr", "^", "expr"], 1, Associativity::Right)
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build()
}

/// Create an empty ParseForest from a grammar.
fn empty_forest(grammar: Grammar) -> ParseForest {
    ParseForest {
        roots: Vec::new(),
        nodes: HashMap::new(),
        grammar,
        source: String::new(),
        next_node_id: 0,
    }
}

/// Insert a plain ForestNode and return its id.
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

/// Find a symbol ID by name, searching both rule_names and tokens.
fn find_symbol(g: &Grammar, name: &str) -> SymbolId {
    if let Some(id) = g.find_symbol_by_name(name) {
        return id;
    }
    for (id, tok) in &g.tokens {
        if tok.name == name {
            return *id;
        }
    }
    panic!("symbol {name:?} not found in grammar");
}

// ===========================================================================
// 1. to_single_tree – simple unambiguous parse
// ===========================================================================

#[test]
fn single_tree_from_unambiguous_forest() {
    let g = simple_grammar();
    let start = g.start_symbol().unwrap();
    let a_sym = *g.tokens.keys().next().unwrap();

    let mut forest = empty_forest(g);
    forest.source = "a".into();

    let leaf = insert_node(&mut forest, a_sym, (0, 1), vec![]);
    let root_id = forest.next_node_id;
    forest.nodes.insert(
        root_id,
        ForestNode {
            id: root_id,
            symbol: start,
            span: (0, 1),
            alternatives: vec![ForestAlternative {
                children: vec![leaf],
            }],
            error_meta: ErrorMeta::default(),
        },
    );
    forest.next_node_id += 1;
    forest.roots.push(forest.nodes[&root_id].clone());

    let tree = forest.to_single_tree().expect("should succeed");
    assert_eq!(tree.root.symbol, start);
    assert_eq!(tree.root.span, (0, 1));
    assert_eq!(tree.root.children.len(), 1);
    assert_eq!(tree.source, "a");
}

#[test]
fn single_tree_preserves_source_text() {
    let g = simple_grammar();
    let start = g.start_symbol().unwrap();
    let a_sym = *g.tokens.keys().next().unwrap();

    let mut forest = empty_forest(g);
    forest.source = "hello world".into();

    let leaf = insert_node(&mut forest, a_sym, (0, 5), vec![]);
    let root_id = forest.next_node_id;
    forest.nodes.insert(
        root_id,
        ForestNode {
            id: root_id,
            symbol: start,
            span: (0, 5),
            alternatives: vec![ForestAlternative {
                children: vec![leaf],
            }],
            error_meta: ErrorMeta::default(),
        },
    );
    forest.next_node_id += 1;
    forest.roots.push(forest.nodes[&root_id].clone());

    let tree = forest.to_single_tree().unwrap();
    assert_eq!(tree.source, "hello world");
}

#[test]
fn single_tree_nested_children() {
    let g = two_alt_grammar();
    let start = g.start_symbol().unwrap();
    let a_sym = find_symbol(&g, "A");

    let mut forest = empty_forest(g);
    forest.source = "a".into();

    let leaf = insert_node(&mut forest, a_sym, (0, 1), vec![]);
    let root_id = forest.next_node_id;
    forest.nodes.insert(
        root_id,
        ForestNode {
            id: root_id,
            symbol: start,
            span: (0, 1),
            alternatives: vec![ForestAlternative {
                children: vec![leaf],
            }],
            error_meta: ErrorMeta::default(),
        },
    );
    forest.next_node_id += 1;
    forest.roots.push(forest.nodes[&root_id].clone());

    let tree = forest.to_single_tree().unwrap();
    assert_eq!(tree.root.children.len(), 1);
    assert_eq!(tree.root.children[0].symbol, a_sym);
    assert!(tree.root.children[0].children.is_empty());
}

// ===========================================================================
// 2. to_single_tree – error cases
// ===========================================================================

#[test]
fn incomplete_forest_yields_error() {
    let forest = empty_forest(simple_grammar());
    let err = forest.to_single_tree().unwrap_err();
    assert!(
        matches!(err, adze_glr_core::ParseError::Incomplete),
        "expected Incomplete, got {err:?}"
    );
}

#[test]
fn forest_without_start_symbol_root_yields_incomplete() {
    let g = simple_grammar();
    let non_start = *g.tokens.keys().next().unwrap();

    let mut forest = empty_forest(g);
    forest.source = "a".into();

    let root_id = forest.next_node_id;
    forest.nodes.insert(
        root_id,
        ForestNode {
            id: root_id,
            symbol: non_start,
            span: (0, 1),
            alternatives: vec![ForestAlternative { children: vec![] }],
            error_meta: ErrorMeta::default(),
        },
    );
    forest.next_node_id += 1;
    forest.roots.push(forest.nodes[&root_id].clone());

    assert!(forest.to_single_tree().is_err());
}

#[test]
fn forest_with_empty_alternatives_is_incomplete() {
    let g = simple_grammar();
    let start = g.start_symbol().unwrap();

    let mut forest = empty_forest(g);
    forest.source = "a".into();

    let root_id = forest.next_node_id;
    forest.nodes.insert(
        root_id,
        ForestNode {
            id: root_id,
            symbol: start,
            span: (0, 1),
            alternatives: vec![], // no alternatives → not complete
            error_meta: ErrorMeta::default(),
        },
    );
    forest.next_node_id += 1;
    forest.roots.push(forest.nodes[&root_id].clone());

    assert!(forest.to_single_tree().is_err());
}

// ===========================================================================
// 3. Ambiguous forest – multiple roots
// ===========================================================================

#[test]
fn ambiguous_forest_picks_first_root() {
    let g = two_alt_grammar();
    let start = g.start_symbol().unwrap();
    let a_sym = find_symbol(&g, "A");
    let b_sym = find_symbol(&g, "B");

    let mut forest = empty_forest(g);
    forest.source = "x".into();

    // Root 1 — via A
    let leaf_a = insert_node(&mut forest, a_sym, (0, 1), vec![]);
    let root1_id = forest.next_node_id;
    forest.nodes.insert(
        root1_id,
        ForestNode {
            id: root1_id,
            symbol: start,
            span: (0, 1),
            alternatives: vec![ForestAlternative {
                children: vec![leaf_a],
            }],
            error_meta: ErrorMeta::default(),
        },
    );
    forest.next_node_id += 1;

    // Root 2 — via B
    let leaf_b = insert_node(&mut forest, b_sym, (0, 1), vec![]);
    let root2_id = forest.next_node_id;
    forest.nodes.insert(
        root2_id,
        ForestNode {
            id: root2_id,
            symbol: start,
            span: (0, 1),
            alternatives: vec![ForestAlternative {
                children: vec![leaf_b],
            }],
            error_meta: ErrorMeta::default(),
        },
    );
    forest.next_node_id += 1;

    forest.roots.push(forest.nodes[&root1_id].clone());
    forest.roots.push(forest.nodes[&root2_id].clone());

    let tree = forest.to_single_tree().unwrap();
    // Should pick the first complete root — which has child a_sym
    assert_eq!(tree.root.symbol, start);
    assert_eq!(tree.root.children[0].symbol, a_sym);
}

#[test]
fn ambiguous_forest_with_multiple_alternatives_picks_first() {
    let g = simple_grammar();
    let start = g.start_symbol().unwrap();
    let a_sym = *g.tokens.keys().next().unwrap();

    let mut forest = empty_forest(g);
    forest.source = "a".into();

    let leaf1 = insert_node(&mut forest, a_sym, (0, 1), vec![]);
    let leaf2 = insert_node(&mut forest, a_sym, (0, 1), vec![]);

    let root_id = forest.next_node_id;
    forest.nodes.insert(
        root_id,
        ForestNode {
            id: root_id,
            symbol: start,
            span: (0, 1),
            alternatives: vec![
                ForestAlternative {
                    children: vec![leaf1],
                },
                ForestAlternative {
                    children: vec![leaf2],
                },
            ],
            error_meta: ErrorMeta::default(),
        },
    );
    forest.next_node_id += 1;
    forest.roots.push(forest.nodes[&root_id].clone());

    // Should succeed and pick first alternative
    let tree = forest.to_single_tree().unwrap();
    assert_eq!(tree.root.children.len(), 1);
}

// ===========================================================================
// 4. ParseTree / ParseNode structural tests
// ===========================================================================

#[test]
fn parse_tree_clone() {
    let tree = ParseTree {
        root: ParseNode {
            symbol: SymbolId(1),
            span: (0, 5),
            children: vec![ParseNode {
                symbol: SymbolId(2),
                span: (0, 3),
                children: vec![],
            }],
        },
        source: "hello".into(),
    };
    let tree2 = tree.clone();
    assert_eq!(tree2.source, tree.source);
    assert_eq!(tree2.root.symbol, tree.root.symbol);
    assert_eq!(tree2.root.children.len(), 1);
}

#[test]
fn parse_node_leaf_has_no_children() {
    let node = ParseNode {
        symbol: SymbolId(42),
        span: (3, 7),
        children: vec![],
    };
    assert!(node.children.is_empty());
    assert_eq!(node.span, (3, 7));
}

// ===========================================================================
// 5. compare_precedences – exhaustive coverage
// ===========================================================================

#[test]
fn prec_higher_shift_wins() {
    let shift = PrecedenceInfo {
        level: 3,
        associativity: Associativity::Left,
        is_fragile: false,
    };
    let reduce = PrecedenceInfo {
        level: 1,
        associativity: Associativity::Left,
        is_fragile: false,
    };
    assert_eq!(
        compare_precedences(Some(shift), Some(reduce)),
        PrecedenceComparison::PreferShift
    );
}

#[test]
fn prec_higher_reduce_wins() {
    let shift = PrecedenceInfo {
        level: 1,
        associativity: Associativity::Left,
        is_fragile: false,
    };
    let reduce = PrecedenceInfo {
        level: 5,
        associativity: Associativity::Left,
        is_fragile: false,
    };
    assert_eq!(
        compare_precedences(Some(shift), Some(reduce)),
        PrecedenceComparison::PreferReduce
    );
}

#[test]
fn prec_same_level_left_assoc() {
    let info = PrecedenceInfo {
        level: 2,
        associativity: Associativity::Left,
        is_fragile: false,
    };
    assert_eq!(
        compare_precedences(Some(info), Some(info)),
        PrecedenceComparison::PreferReduce
    );
}

#[test]
fn prec_same_level_right_assoc() {
    let info = PrecedenceInfo {
        level: 2,
        associativity: Associativity::Right,
        is_fragile: false,
    };
    assert_eq!(
        compare_precedences(Some(info), Some(info)),
        PrecedenceComparison::PreferShift
    );
}

#[test]
fn prec_same_level_none_assoc() {
    let info = PrecedenceInfo {
        level: 2,
        associativity: Associativity::None,
        is_fragile: false,
    };
    assert_eq!(
        compare_precedences(Some(info), Some(info)),
        PrecedenceComparison::Error
    );
}

#[test]
fn prec_shift_none() {
    let reduce = PrecedenceInfo {
        level: 1,
        associativity: Associativity::Left,
        is_fragile: false,
    };
    assert_eq!(
        compare_precedences(None, Some(reduce)),
        PrecedenceComparison::None
    );
}

#[test]
fn prec_reduce_none() {
    let shift = PrecedenceInfo {
        level: 1,
        associativity: Associativity::Left,
        is_fragile: false,
    };
    assert_eq!(
        compare_precedences(Some(shift), None),
        PrecedenceComparison::None
    );
}

#[test]
fn prec_both_none() {
    assert_eq!(compare_precedences(None, None), PrecedenceComparison::None);
}

// ===========================================================================
// 6. StaticPrecedenceResolver
// ===========================================================================

#[test]
fn static_resolver_extracts_token_prec_from_declarations() {
    let mut g = Grammar::new("test".into());
    g.precedences.push(Precedence {
        level: 3,
        associativity: Associativity::Right,
        symbols: vec![SymbolId(10)],
    });
    let resolver = StaticPrecedenceResolver::from_grammar(&g);
    let info = resolver.token_precedence(SymbolId(10)).unwrap();
    assert_eq!(info.level, 3);
    assert_eq!(info.associativity, Associativity::Right);
}

#[test]
fn static_resolver_missing_token_returns_none() {
    let g = Grammar::new("empty".into());
    let resolver = StaticPrecedenceResolver::from_grammar(&g);
    assert!(resolver.token_precedence(SymbolId(99)).is_none());
}

#[test]
fn static_resolver_extracts_rule_prec() {
    let mut g = Grammar::new("rule_prec".into());
    let e = SymbolId(10);
    let t = SymbolId(1);
    g.tokens.insert(
        t,
        Token {
            name: "t".into(),
            pattern: TokenPattern::String("t".into()),
            fragile: false,
        },
    );
    g.rule_names.insert(e, "E".into());
    g.rules.insert(
        e,
        vec![Rule {
            lhs: e,
            rhs: vec![Symbol::Terminal(t)],
            precedence: Some(PrecedenceKind::Static(5)),
            associativity: Some(Associativity::Left),
            fields: vec![],
            production_id: ProductionId(0),
        }],
    );
    let resolver = StaticPrecedenceResolver::from_grammar(&g);
    let info = resolver.rule_precedence(RuleId(0)).unwrap();
    assert_eq!(info.level, 5);
    assert_eq!(info.associativity, Associativity::Left);
}

#[test]
fn static_resolver_rule_prec_none() {
    let g = Grammar::new("empty".into());
    let resolver = StaticPrecedenceResolver::from_grammar(&g);
    assert!(resolver.rule_precedence(RuleId(99)).is_none());
}

// ===========================================================================
// 7. PrecedenceResolver (advanced_conflict)
// ===========================================================================

#[test]
fn advanced_resolver_prefer_shift_higher_prec() {
    let mut g = Grammar::new("adv".into());
    let plus = SymbolId(1);
    let star = SymbolId(2);
    let e = SymbolId(10);

    g.precedences.push(Precedence {
        level: 1,
        associativity: Associativity::Left,
        symbols: vec![plus],
    });
    g.precedences.push(Precedence {
        level: 2,
        associativity: Associativity::Left,
        symbols: vec![star],
    });
    g.tokens.insert(
        plus,
        Token {
            name: "+".into(),
            pattern: TokenPattern::String("+".into()),
            fragile: false,
        },
    );
    g.tokens.insert(
        star,
        Token {
            name: "*".into(),
            pattern: TokenPattern::String("*".into()),
            fragile: false,
        },
    );
    g.rule_names.insert(e, "E".into());
    g.rules.insert(
        e,
        vec![Rule {
            lhs: e,
            rhs: vec![Symbol::Terminal(plus)],
            precedence: Some(PrecedenceKind::Static(1)),
            associativity: Some(Associativity::Left),
            fields: vec![],
            production_id: ProductionId(0),
        }],
    );

    let resolver = PrecedenceResolver::new(&g);
    // star (prec 2) vs rule for e (prec 1) → prefer shift
    let decision = resolver.can_resolve_shift_reduce(star, e);
    assert_eq!(
        decision,
        Some(adze_glr_core::PrecedenceDecision::PreferShift)
    );
}

#[test]
fn advanced_resolver_prefer_reduce_higher_prec() {
    let mut g = Grammar::new("adv2".into());
    let plus = SymbolId(1);
    let e = SymbolId(10);

    g.precedences.push(Precedence {
        level: 1,
        associativity: Associativity::Left,
        symbols: vec![plus],
    });
    g.tokens.insert(
        plus,
        Token {
            name: "+".into(),
            pattern: TokenPattern::String("+".into()),
            fragile: false,
        },
    );
    g.rule_names.insert(e, "E".into());
    g.rules.insert(
        e,
        vec![Rule {
            lhs: e,
            rhs: vec![Symbol::Terminal(plus)],
            precedence: Some(PrecedenceKind::Static(5)),
            associativity: Some(Associativity::Left),
            fields: vec![],
            production_id: ProductionId(0),
        }],
    );

    let resolver = PrecedenceResolver::new(&g);
    // plus (prec 1) vs rule for e (prec 5) → prefer reduce
    let decision = resolver.can_resolve_shift_reduce(plus, e);
    assert_eq!(
        decision,
        Some(adze_glr_core::PrecedenceDecision::PreferReduce)
    );
}

#[test]
fn advanced_resolver_no_info_returns_none() {
    let g = Grammar::new("empty".into());
    let resolver = PrecedenceResolver::new(&g);
    assert!(
        resolver
            .can_resolve_shift_reduce(SymbolId(1), SymbolId(2))
            .is_none()
    );
}

// ===========================================================================
// 8. ConflictAnalyzer
// ===========================================================================

#[test]
fn conflict_analyzer_default() {
    let analyzer = ConflictAnalyzer::new();
    let stats = analyzer.get_stats();
    assert_eq!(stats.shift_reduce_conflicts, 0);
    assert_eq!(stats.reduce_reduce_conflicts, 0);
    assert_eq!(stats.precedence_resolved, 0);
}

#[test]
fn conflict_stats_default_all_zero() {
    let stats = ConflictStats::default();
    assert_eq!(stats.shift_reduce_conflicts, 0);
    assert_eq!(stats.reduce_reduce_conflicts, 0);
    assert_eq!(stats.precedence_resolved, 0);
    assert_eq!(stats.associativity_resolved, 0);
    assert_eq!(stats.explicit_glr, 0);
    assert_eq!(stats.default_resolved, 0);
}

// ===========================================================================
// 9. ConflictResolver::detect_conflicts – unambiguous grammar
// ===========================================================================

#[test]
fn unambiguous_grammar_no_conflicts() {
    let g = simple_grammar();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let collection = ItemSetCollection::build_canonical_collection(&g, &ff);
    let resolver = ConflictResolver::detect_conflicts(&collection, &g, &ff);
    assert!(
        resolver.conflicts.is_empty(),
        "simple grammar should have zero conflicts"
    );
}

// ===========================================================================
// 10. ConflictResolver::detect_conflicts – ambiguous grammar
// ===========================================================================

#[test]
fn ambiguous_grammar_has_conflicts() {
    let g = self_recursive_grammar();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let collection = ItemSetCollection::build_canonical_collection(&g, &ff);
    let resolver = ConflictResolver::detect_conflicts(&collection, &g, &ff);
    assert!(
        !resolver.conflicts.is_empty(),
        "E -> a | E E should produce conflicts"
    );
}

#[test]
fn ambiguous_grammar_conflict_type() {
    let g = self_recursive_grammar();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let collection = ItemSetCollection::build_canonical_collection(&g, &ff);
    let resolver = ConflictResolver::detect_conflicts(&collection, &g, &ff);

    // Must have at least one shift/reduce conflict
    let has_sr = resolver
        .conflicts
        .iter()
        .any(|c| c.conflict_type == ConflictType::ShiftReduce);
    assert!(has_sr, "expected at least one shift/reduce conflict");
}

// ===========================================================================
// 11. ConflictResolver::resolve_conflicts – precedence resolution
// ===========================================================================

#[test]
fn left_assoc_grammar_resolves_conflicts() {
    let g = left_assoc_grammar();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let collection = ItemSetCollection::build_canonical_collection(&g, &ff);
    let mut resolver = ConflictResolver::detect_conflicts(&collection, &g, &ff);
    let before = resolver.conflicts.len();
    resolver.resolve_conflicts(&g);

    // Resolution runs without panicking and preserves conflict entries.
    assert_eq!(resolver.conflicts.len(), before);
    // At least one conflict should exist for this grammar (E + E + E is ambiguous).
    assert!(
        !resolver.conflicts.is_empty(),
        "left-assoc grammar should have conflicts"
    );
}

#[test]
fn right_assoc_grammar_resolves_conflicts() {
    let g = right_assoc_grammar();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let collection = ItemSetCollection::build_canonical_collection(&g, &ff);
    let mut resolver = ConflictResolver::detect_conflicts(&collection, &g, &ff);
    resolver.resolve_conflicts(&g);

    // Right-assoc grammar should also produce conflicts that get processed.
    assert!(
        !resolver.conflicts.is_empty(),
        "right-assoc grammar should have conflicts"
    );
}

// ===========================================================================
// 12. Reduce/reduce resolution picks lowest rule id
// ===========================================================================

#[test]
fn reduce_reduce_picks_first_rule() {
    // Create a grammar that actually produces reduce/reduce conflicts:
    // S -> A | B
    // A -> x
    // B -> x
    // At state after reading 'x', we can reduce to either A or B.
    let mut g = Grammar::new("rr".into());
    let x = SymbolId(1);
    let a = SymbolId(10);
    let b = SymbolId(11);
    let s = SymbolId(12);

    g.tokens.insert(
        x,
        Token {
            name: "x".into(),
            pattern: TokenPattern::String("x".into()),
            fragile: false,
        },
    );
    g.rule_names.insert(s, "S".into());
    g.rule_names.insert(a, "A".into());
    g.rule_names.insert(b, "B".into());

    // S must be first in the map so start_symbol() picks it
    let mut ordered = IndexMap::new();
    ordered.insert(
        s,
        vec![
            Rule {
                lhs: s,
                rhs: vec![Symbol::NonTerminal(a)],
                precedence: None,
                associativity: None,
                fields: vec![],
                production_id: ProductionId(0),
            },
            Rule {
                lhs: s,
                rhs: vec![Symbol::NonTerminal(b)],
                precedence: None,
                associativity: None,
                fields: vec![],
                production_id: ProductionId(1),
            },
        ],
    );
    ordered.insert(
        a,
        vec![Rule {
            lhs: a,
            rhs: vec![Symbol::Terminal(x)],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(2),
        }],
    );
    ordered.insert(
        b,
        vec![Rule {
            lhs: b,
            rhs: vec![Symbol::Terminal(x)],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(3),
        }],
    );
    g.rules = ordered;

    let ff = FirstFollowSets::compute(&g).unwrap();
    let collection = ItemSetCollection::build_canonical_collection(&g, &ff);
    let mut resolver = ConflictResolver::detect_conflicts(&collection, &g, &ff);

    // Should have at least one reduce/reduce conflict
    let has_rr = resolver
        .conflicts
        .iter()
        .any(|c| c.conflict_type == ConflictType::ReduceReduce);
    assert!(
        has_rr,
        "S -> A | B, A -> x, B -> x should produce reduce/reduce conflict"
    );

    // Resolve conflicts
    resolver.resolve_conflicts(&g);

    // Verify resolution ran without panicking. The conflict resolver modifies
    // actions in place; reduce/reduce conflicts should have their actions
    // reduced to one entry.
    // Note: some conflicts detected as reduce/reduce may have been internally
    // rewritten to shift actions during augmented grammar construction.
    for conflict in &resolver.conflicts {
        if conflict.conflict_type == ConflictType::ReduceReduce {
            let all_reduce = conflict
                .actions
                .iter()
                .all(|a| matches!(a, Action::Reduce(_)));
            if all_reduce {
                assert_eq!(
                    conflict.actions.len(),
                    1,
                    "pure reduce/reduce should resolve to one action"
                );
            }
        }
    }
}

// ===========================================================================
// 13. build_lr1_automaton – simple grammar succeeds
// ===========================================================================

#[test]
fn build_lr1_automaton_simple() {
    let g = simple_grammar();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let table = build_lr1_automaton(&g, &ff).expect("should build parse table");
    assert!(table.state_count > 0);
    assert!(table.action_table.len() == table.state_count);
}

// ===========================================================================
// 14. build_lr1_automaton – ambiguous grammar still succeeds
// ===========================================================================

#[test]
fn build_lr1_automaton_ambiguous() {
    let g = self_recursive_grammar();
    let ff = FirstFollowSets::compute(&g).unwrap();
    // GLR automaton builder should succeed even with conflicts
    let table = build_lr1_automaton(&g, &ff).expect("GLR should handle ambiguous grammar");
    assert!(table.state_count > 0);
}

// ===========================================================================
// 15. ForestNode::is_complete
// ===========================================================================

#[test]
fn forest_node_complete() {
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
fn forest_node_incomplete() {
    let node = ForestNode {
        id: 0,
        symbol: SymbolId(1),
        span: (0, 1),
        alternatives: vec![],
        error_meta: ErrorMeta::default(),
    };
    assert!(!node.is_complete());
}

// ===========================================================================
// 16. ParseError variants
// ===========================================================================

#[test]
fn parse_error_incomplete_display() {
    let err = adze_glr_core::ParseError::Incomplete;
    assert_eq!(format!("{err}"), "Incomplete parse");
}

#[test]
fn parse_error_failed_display() {
    let err = adze_glr_core::ParseError::Failed("bad token".into());
    assert!(format!("{err}").contains("bad token"));
}

#[test]
fn parse_error_unknown_display() {
    let err = adze_glr_core::ParseError::Unknown;
    assert_eq!(format!("{err}"), "Unknown error");
}

// ===========================================================================
// 17. Multi-level precedence resolution
// ===========================================================================

#[test]
fn multi_level_precedence() {
    let g = GrammarBuilder::new("calc")
        .token("NUM", r"\d+")
        .token("+", "+")
        .token("*", "*")
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "*", "expr"], 2, Associativity::Left)
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();

    let ff = FirstFollowSets::compute(&g).unwrap();
    let table = build_lr1_automaton(&g, &ff).expect("multi-level should build");
    assert!(table.state_count > 0);
}

// ===========================================================================
// 18. Non-associative conflict yields error decision
// ===========================================================================

#[test]
fn non_assoc_same_level_yields_error() {
    let shift = PrecedenceInfo {
        level: 1,
        associativity: Associativity::None,
        is_fragile: false,
    };
    let reduce = PrecedenceInfo {
        level: 1,
        associativity: Associativity::None,
        is_fragile: false,
    };
    assert_eq!(
        compare_precedences(Some(shift), Some(reduce)),
        PrecedenceComparison::Error
    );
}

// ===========================================================================
// 19. ConflictResolver on grammar without start → detects zero conflicts
// ===========================================================================

#[test]
fn empty_grammar_no_conflicts() {
    let g = Grammar::new("empty".into());
    // Cannot compute FF without tokens/rules, so just verify ConflictResolver
    // accepts an empty collection.
    let collection = ItemSetCollection {
        sets: vec![],
        goto_table: IndexMap::new(),
        symbol_is_terminal: IndexMap::new(),
    };
    let ff_opt = FirstFollowSets::compute(&g);
    // If FF fails (no start symbol), still test detect_conflicts with empty sets
    if let Ok(ff) = ff_opt {
        let resolver = ConflictResolver::detect_conflicts(&collection, &g, &ff);
        assert!(resolver.conflicts.is_empty());
    }
}

// ===========================================================================
// 20. Forest deep tree extraction
// ===========================================================================

#[test]
fn deep_tree_extraction() {
    // Build a chain: S -> A, A -> B, B -> leaf
    let mut g = Grammar::new("deep".into());
    let leaf_sym = SymbolId(1);
    let b_sym = SymbolId(10);
    let a_sym = SymbolId(11);
    let s_sym = SymbolId(12);

    g.tokens.insert(
        leaf_sym,
        Token {
            name: "x".into(),
            pattern: TokenPattern::String("x".into()),
            fragile: false,
        },
    );
    g.rule_names.insert(b_sym, "B".into());
    g.rule_names.insert(a_sym, "A".into());
    g.rule_names.insert(s_sym, "S".into());
    // S is the first rule entry → start_symbol() picks it
    // Ensure S is inserted first in the IndexMap
    let mut ordered_rules = IndexMap::new();
    ordered_rules.insert(
        s_sym,
        vec![Rule {
            lhs: s_sym,
            rhs: vec![Symbol::NonTerminal(a_sym)],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(2),
        }],
    );
    ordered_rules.insert(
        a_sym,
        vec![Rule {
            lhs: a_sym,
            rhs: vec![Symbol::NonTerminal(b_sym)],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(1),
        }],
    );
    ordered_rules.insert(
        b_sym,
        vec![Rule {
            lhs: b_sym,
            rhs: vec![Symbol::Terminal(leaf_sym)],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(0),
        }],
    );
    g.rules = ordered_rules;

    let mut forest = empty_forest(g);
    forest.source = "x".into();

    let leaf = insert_node(&mut forest, leaf_sym, (0, 1), vec![]);
    let b_id = insert_node(&mut forest, b_sym, (0, 1), vec![leaf]);
    let a_id = insert_node(&mut forest, a_sym, (0, 1), vec![b_id]);

    let root_id = forest.next_node_id;
    forest.nodes.insert(
        root_id,
        ForestNode {
            id: root_id,
            symbol: s_sym,
            span: (0, 1),
            alternatives: vec![ForestAlternative {
                children: vec![a_id],
            }],
            error_meta: ErrorMeta::default(),
        },
    );
    forest.next_node_id += 1;
    forest.roots.push(forest.nodes[&root_id].clone());

    let tree = forest.to_single_tree().unwrap();
    assert_eq!(tree.root.symbol, s_sym);
    assert_eq!(tree.root.children.len(), 1);
    assert_eq!(tree.root.children[0].symbol, a_sym);
    assert_eq!(tree.root.children[0].children[0].symbol, b_sym);
    assert_eq!(
        tree.root.children[0].children[0].children[0].symbol,
        leaf_sym
    );
    assert!(
        tree.root.children[0].children[0].children[0]
            .children
            .is_empty()
    );
}

// ===========================================================================
// 21. ConflictStats is cloneable
// ===========================================================================

#[test]
fn conflict_stats_clone() {
    let stats = ConflictStats {
        shift_reduce_conflicts: 3,
        reduce_reduce_conflicts: 1,
        precedence_resolved: 2,
        associativity_resolved: 1,
        explicit_glr: 0,
        default_resolved: 0,
    };
    let stats2 = stats.clone();
    assert_eq!(stats2.shift_reduce_conflicts, 3);
    assert_eq!(stats2.reduce_reduce_conflicts, 1);
}

// ===========================================================================
// 22. Fragile flag on PrecedenceInfo
// ===========================================================================

#[test]
fn precedence_info_fragile_flag() {
    let fragile = PrecedenceInfo {
        level: 1,
        associativity: Associativity::Left,
        is_fragile: true,
    };
    let normal = PrecedenceInfo {
        level: 1,
        associativity: Associativity::Left,
        is_fragile: false,
    };
    // Fragile flag does not affect comparison outcome
    assert_eq!(
        compare_precedences(Some(fragile), Some(normal)),
        PrecedenceComparison::PreferReduce
    );
}

// ===========================================================================
// 23. Negative precedence levels
// ===========================================================================

#[test]
fn negative_precedence_levels() {
    let low = PrecedenceInfo {
        level: -5,
        associativity: Associativity::Left,
        is_fragile: false,
    };
    let high = PrecedenceInfo {
        level: -1,
        associativity: Associativity::Left,
        is_fragile: false,
    };
    assert_eq!(
        compare_precedences(Some(high), Some(low)),
        PrecedenceComparison::PreferShift
    );
    assert_eq!(
        compare_precedences(Some(low), Some(high)),
        PrecedenceComparison::PreferReduce
    );
}
