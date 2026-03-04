//! Comprehensive tests for conflict resolution and the ConflictAnalyzer.

use adze_glr_core::advanced_conflict::{
    ConflictAnalyzer, ConflictStats, PrecedenceDecision, PrecedenceResolver,
};
use adze_glr_core::{Action, FirstFollowSets, ParseTable, build_lr1_automaton};
use adze_ir::builder::GrammarBuilder;
use adze_ir::{
    Associativity, Grammar, Precedence, PrecedenceKind, ProductionId, Rule, Symbol, SymbolId,
};

// ── Helpers ──────────────────────────────────────────────────────────────────

fn build_table(
    name: &str,
    tokens: &[(&str, &str)],
    rules: &[(&str, Vec<&str>)],
    start: &str,
) -> ParseTable {
    let mut b = GrammarBuilder::new(name);
    for &(n, p) in tokens {
        b = b.token(n, p);
    }
    for (lhs, rhs) in rules {
        b = b.rule(lhs, rhs.clone());
    }
    b = b.start(start);
    let mut g = b.build();
    g.normalize();
    let ff = FirstFollowSets::compute(&g).unwrap();
    build_lr1_automaton(&g, &ff).unwrap()
}

fn build_table_with_prec(
    name: &str,
    tokens: &[(&str, &str)],
    plain_rules: &[(&str, Vec<&str>)],
    prec_rules: &[(&str, Vec<&str>, i16, Associativity)],
    start: &str,
) -> ParseTable {
    let mut b = GrammarBuilder::new(name);
    for &(n, p) in tokens {
        b = b.token(n, p);
    }
    for (lhs, rhs) in plain_rules {
        b = b.rule(lhs, rhs.clone());
    }
    for (lhs, rhs, prec, assoc) in prec_rules {
        b = b.rule_with_precedence(lhs, rhs.clone(), *prec, *assoc);
    }
    b = b.start(start);
    let mut g = b.build();
    g.normalize();
    let ff = FirstFollowSets::compute(&g).unwrap();
    build_lr1_automaton(&g, &ff).unwrap()
}

fn build_grammar_with_prec(
    name: &str,
    tokens: &[(&str, &str)],
    plain_rules: &[(&str, Vec<&str>)],
    prec_rules: &[(&str, Vec<&str>, i16, Associativity)],
    start: &str,
) -> Grammar {
    let mut b = GrammarBuilder::new(name);
    for &(n, p) in tokens {
        b = b.token(n, p);
    }
    for (lhs, rhs) in plain_rules {
        b = b.rule(lhs, rhs.clone());
    }
    for (lhs, rhs, prec, assoc) in prec_rules {
        b = b.rule_with_precedence(lhs, rhs.clone(), *prec, *assoc);
    }
    b = b.start(start);
    let mut g = b.build();
    g.normalize();
    g
}

// ════════════════════════════════════════════════════════════════════════════
// 1. ConflictAnalyzer construction
// ════════════════════════════════════════════════════════════════════════════

#[test]
fn analyzer_new_returns_instance() {
    let _a = ConflictAnalyzer::new();
}

#[test]
fn analyzer_default_returns_instance() {
    let _a = ConflictAnalyzer::default();
}

#[test]
fn analyzer_initial_stats_are_zero() {
    let a = ConflictAnalyzer::new();
    let s = a.get_stats();
    assert_eq!(s.shift_reduce_conflicts, 0);
    assert_eq!(s.reduce_reduce_conflicts, 0);
    assert_eq!(s.precedence_resolved, 0);
    assert_eq!(s.associativity_resolved, 0);
    assert_eq!(s.explicit_glr, 0);
    assert_eq!(s.default_resolved, 0);
}

// ════════════════════════════════════════════════════════════════════════════
// 2. Analyze simple grammar (no conflicts)
// ════════════════════════════════════════════════════════════════════════════

#[test]
fn analyze_single_token_grammar() {
    let pt = build_table("t", &[("a", "a")], &[("s", vec!["a"])], "s");
    let mut ca = ConflictAnalyzer::new();
    let stats = ca.analyze_table(&pt);
    assert_eq!(stats.shift_reduce_conflicts, 0);
    assert_eq!(stats.reduce_reduce_conflicts, 0);
}

#[test]
fn analyze_two_token_sequence() {
    let pt = build_table(
        "t",
        &[("a", "a"), ("b", "b")],
        &[("s", vec!["a", "b"])],
        "s",
    );
    let mut ca = ConflictAnalyzer::new();
    let stats = ca.analyze_table(&pt);
    assert_eq!(stats.shift_reduce_conflicts, 0);
}

#[test]
fn analyze_three_token_sequence() {
    let pt = build_table(
        "t",
        &[("a", "a"), ("b", "b"), ("c", "c")],
        &[("s", vec!["a", "b", "c"])],
        "s",
    );
    let mut ca = ConflictAnalyzer::new();
    let _stats = ca.analyze_table(&pt);
}

#[test]
fn analyze_chain_grammar_no_conflict() {
    let pt = build_table(
        "chain",
        &[("x", "x")],
        &[("a", vec!["x"]), ("b", vec!["a"]), ("s", vec!["b"])],
        "s",
    );
    let mut ca = ConflictAnalyzer::new();
    let stats = ca.analyze_table(&pt);
    assert_eq!(stats.shift_reduce_conflicts, 0);
}

#[test]
fn analyze_deep_chain_no_conflict() {
    let pt = build_table(
        "deep",
        &[("x", "x")],
        &[
            ("a", vec!["x"]),
            ("b", vec!["a"]),
            ("c", vec!["b"]),
            ("d", vec!["c"]),
            ("s", vec!["d"]),
        ],
        "s",
    );
    let mut ca = ConflictAnalyzer::new();
    let stats = ca.analyze_table(&pt);
    assert_eq!(stats.shift_reduce_conflicts, 0);
}

// ════════════════════════════════════════════════════════════════════════════
// 3. Analyze grammar with alternatives
// ════════════════════════════════════════════════════════════════════════════

#[test]
fn analyze_two_alternatives() {
    let pt = build_table(
        "alt",
        &[("a", "a"), ("b", "b")],
        &[("s", vec!["a"]), ("s", vec!["b"])],
        "s",
    );
    let mut ca = ConflictAnalyzer::new();
    let _stats = ca.analyze_table(&pt);
}

#[test]
fn analyze_three_alternatives() {
    let pt = build_table(
        "alt3",
        &[("a", "a"), ("b", "b"), ("c", "c")],
        &[("s", vec!["a"]), ("s", vec!["b"]), ("s", vec!["c"])],
        "s",
    );
    let mut ca = ConflictAnalyzer::new();
    let _stats = ca.analyze_table(&pt);
}

#[test]
fn analyze_five_alternatives() {
    let tokens: Vec<(String, String)> =
        (0..5).map(|i| (format!("t{i}"), format!("t{i}"))).collect();
    let tok_refs: Vec<(&str, &str)> = tokens
        .iter()
        .map(|(a, b)| (a.as_str(), b.as_str()))
        .collect();
    let rules: Vec<(&str, Vec<&str>)> = tokens
        .iter()
        .map(|(n, _)| ("s", vec![n.as_str()]))
        .collect();
    let pt = build_table("alt5", &tok_refs, &rules, "s");
    let mut ca = ConflictAnalyzer::new();
    let _stats = ca.analyze_table(&pt);
}

#[test]
fn analyze_alternatives_with_different_lengths() {
    let pt = build_table(
        "mixed",
        &[("a", "a"), ("b", "b"), ("c", "c")],
        &[("s", vec!["a"]), ("s", vec!["b", "c"])],
        "s",
    );
    let mut ca = ConflictAnalyzer::new();
    let _stats = ca.analyze_table(&pt);
}

#[test]
fn analyze_diamond_pattern() {
    let pt = build_table(
        "diamond",
        &[("x", "x"), ("y", "y")],
        &[
            ("a", vec!["x"]),
            ("b", vec!["y"]),
            ("s", vec!["a"]),
            ("s", vec!["b"]),
        ],
        "s",
    );
    let mut ca = ConflictAnalyzer::new();
    let _stats = ca.analyze_table(&pt);
}

// ════════════════════════════════════════════════════════════════════════════
// 4. ConflictStats fields
// ════════════════════════════════════════════════════════════════════════════

#[test]
fn stats_default_has_all_zero() {
    let s = ConflictStats::default();
    assert_eq!(s.shift_reduce_conflicts, 0);
    assert_eq!(s.reduce_reduce_conflicts, 0);
    assert_eq!(s.precedence_resolved, 0);
    assert_eq!(s.associativity_resolved, 0);
    assert_eq!(s.explicit_glr, 0);
    assert_eq!(s.default_resolved, 0);
}

#[test]
fn stats_clone_preserves_fields() {
    let mut s = ConflictStats::default();
    s.shift_reduce_conflicts = 3;
    s.reduce_reduce_conflicts = 2;
    s.precedence_resolved = 1;
    s.associativity_resolved = 4;
    s.explicit_glr = 5;
    s.default_resolved = 6;
    let c = s.clone();
    assert_eq!(c.shift_reduce_conflicts, 3);
    assert_eq!(c.reduce_reduce_conflicts, 2);
    assert_eq!(c.precedence_resolved, 1);
    assert_eq!(c.associativity_resolved, 4);
    assert_eq!(c.explicit_glr, 5);
    assert_eq!(c.default_resolved, 6);
}

#[test]
fn stats_debug_impl() {
    let s = ConflictStats::default();
    let dbg = format!("{s:?}");
    assert!(dbg.contains("ConflictStats"));
}

#[test]
fn stats_default_resolved_accessible() {
    let pt = build_table("t", &[("a", "a")], &[("s", vec!["a"])], "s");
    let mut ca = ConflictAnalyzer::new();
    let stats = ca.analyze_table(&pt);
    let _ = stats.default_resolved;
}

#[test]
fn stats_explicit_glr_accessible() {
    let pt = build_table("t", &[("a", "a")], &[("s", vec!["a"])], "s");
    let mut ca = ConflictAnalyzer::new();
    let stats = ca.analyze_table(&pt);
    let _ = stats.explicit_glr;
}

#[test]
fn stats_precedence_resolved_accessible() {
    let pt = build_table("t", &[("a", "a")], &[("s", vec!["a"])], "s");
    let mut ca = ConflictAnalyzer::new();
    let stats = ca.analyze_table(&pt);
    let _ = stats.precedence_resolved;
}

#[test]
fn stats_associativity_resolved_accessible() {
    let pt = build_table("t", &[("a", "a")], &[("s", vec!["a"])], "s");
    let mut ca = ConflictAnalyzer::new();
    let stats = ca.analyze_table(&pt);
    let _ = stats.associativity_resolved;
}

// ════════════════════════════════════════════════════════════════════════════
// 5. ConflictAnalyzer reuse across grammars
// ════════════════════════════════════════════════════════════════════════════

#[test]
fn reuse_analyzer_two_grammars() {
    let pt1 = build_table("g1", &[("a", "a")], &[("s", vec!["a"])], "s");
    let pt2 = build_table("g2", &[("b", "b")], &[("s", vec!["b"])], "s");
    let mut ca = ConflictAnalyzer::new();
    let _s1 = ca.analyze_table(&pt1);
    let _s2 = ca.analyze_table(&pt2);
}

#[test]
fn reuse_analyzer_three_grammars() {
    let pt1 = build_table("g1", &[("a", "a")], &[("s", vec!["a"])], "s");
    let pt2 = build_table("g2", &[("b", "b")], &[("s", vec!["b"])], "s");
    let pt3 = build_table("g3", &[("c", "c")], &[("s", vec!["c"])], "s");
    let mut ca = ConflictAnalyzer::new();
    let _ = ca.analyze_table(&pt1);
    let _ = ca.analyze_table(&pt2);
    let _ = ca.analyze_table(&pt3);
}

#[test]
fn reuse_analyzer_deterministic() {
    let pt = build_table("det", &[("a", "a")], &[("s", vec!["a"])], "s");
    let mut ca = ConflictAnalyzer::new();
    let s1 = ca.analyze_table(&pt);
    let s2 = ca.analyze_table(&pt);
    assert_eq!(s1.shift_reduce_conflicts, s2.shift_reduce_conflicts);
    assert_eq!(s1.reduce_reduce_conflicts, s2.reduce_reduce_conflicts);
    assert_eq!(s1.default_resolved, s2.default_resolved);
    assert_eq!(s1.precedence_resolved, s2.precedence_resolved);
    assert_eq!(s1.associativity_resolved, s2.associativity_resolved);
    assert_eq!(s1.explicit_glr, s2.explicit_glr);
}

#[test]
fn get_stats_reflects_last_analysis() {
    let pt1 = build_table("g1", &[("a", "a")], &[("s", vec!["a"])], "s");
    let pt2 = build_table(
        "g2",
        &[("a", "a"), ("b", "b")],
        &[("s", vec!["a"]), ("s", vec!["b"])],
        "s",
    );
    let mut ca = ConflictAnalyzer::new();
    let _ = ca.analyze_table(&pt1);
    let _ = ca.analyze_table(&pt2);
    let _ = ca.get_stats();
}

// ════════════════════════════════════════════════════════════════════════════
// 6. Analyze grammar with precedence
// ════════════════════════════════════════════════════════════════════════════

#[test]
fn analyze_with_left_precedence() {
    let pt = build_table_with_prec(
        "prec_left",
        &[("x", "x"), ("y", "y")],
        &[],
        &[
            ("s", vec!["x"], 1, Associativity::Left),
            ("s", vec!["y"], 2, Associativity::Left),
        ],
        "s",
    );
    let mut ca = ConflictAnalyzer::new();
    let _stats = ca.analyze_table(&pt);
}

#[test]
fn analyze_with_right_precedence() {
    let pt = build_table_with_prec(
        "prec_right",
        &[("x", "x"), ("y", "y")],
        &[],
        &[
            ("s", vec!["x"], 1, Associativity::Right),
            ("s", vec!["y"], 2, Associativity::Right),
        ],
        "s",
    );
    let mut ca = ConflictAnalyzer::new();
    let _stats = ca.analyze_table(&pt);
}

#[test]
fn analyze_with_none_associativity() {
    let pt = build_table_with_prec(
        "prec_none",
        &[("x", "x"), ("y", "y")],
        &[],
        &[
            ("s", vec!["x"], 1, Associativity::None),
            ("s", vec!["y"], 2, Associativity::None),
        ],
        "s",
    );
    let mut ca = ConflictAnalyzer::new();
    let _stats = ca.analyze_table(&pt);
}

#[test]
fn analyze_mixed_prec_and_plain_rules() {
    let pt = build_table_with_prec(
        "mixed_prec",
        &[("x", "x"), ("y", "y"), ("z", "z")],
        &[("s", vec!["z"])],
        &[
            ("s", vec!["x"], 1, Associativity::Left),
            ("s", vec!["y"], 2, Associativity::Left),
        ],
        "s",
    );
    let mut ca = ConflictAnalyzer::new();
    let _stats = ca.analyze_table(&pt);
}

// ════════════════════════════════════════════════════════════════════════════
// 7. Analyze grammar with associativity
// ════════════════════════════════════════════════════════════════════════════

#[test]
fn analyze_left_assoc_expression() {
    let pt = build_table_with_prec(
        "left_expr",
        &[("num", "[0-9]+"), ("plus", "\\+")],
        &[("expr", vec!["num"]), ("s", vec!["expr"])],
        &[("expr", vec!["expr", "plus", "expr"], 1, Associativity::Left)],
        "s",
    );
    let mut ca = ConflictAnalyzer::new();
    let _stats = ca.analyze_table(&pt);
}

#[test]
fn analyze_right_assoc_expression() {
    let pt = build_table_with_prec(
        "right_expr",
        &[("num", "[0-9]+"), ("eq", "=")],
        &[("expr", vec!["num"]), ("s", vec!["expr"])],
        &[("expr", vec!["expr", "eq", "expr"], 1, Associativity::Right)],
        "s",
    );
    let mut ca = ConflictAnalyzer::new();
    let _stats = ca.analyze_table(&pt);
}

#[test]
fn analyze_multi_level_prec_expression() {
    let pt = build_table_with_prec(
        "multi_prec",
        &[("num", "[0-9]+"), ("plus", "\\+"), ("star", "\\*")],
        &[("expr", vec!["num"]), ("s", vec!["expr"])],
        &[
            ("expr", vec!["expr", "plus", "expr"], 1, Associativity::Left),
            ("expr", vec!["expr", "star", "expr"], 2, Associativity::Left),
        ],
        "s",
    );
    let mut ca = ConflictAnalyzer::new();
    let _stats = ca.analyze_table(&pt);
}

#[test]
fn analyze_three_level_prec() {
    let pt = build_table_with_prec(
        "three_prec",
        &[
            ("num", "[0-9]+"),
            ("plus", "\\+"),
            ("star", "\\*"),
            ("caret", "\\^"),
        ],
        &[("expr", vec!["num"]), ("s", vec!["expr"])],
        &[
            ("expr", vec!["expr", "plus", "expr"], 1, Associativity::Left),
            ("expr", vec!["expr", "star", "expr"], 2, Associativity::Left),
            (
                "expr",
                vec!["expr", "caret", "expr"],
                3,
                Associativity::Right,
            ),
        ],
        "s",
    );
    let mut ca = ConflictAnalyzer::new();
    let _stats = ca.analyze_table(&pt);
}

// ════════════════════════════════════════════════════════════════════════════
// 8. Analyze larger grammars
// ════════════════════════════════════════════════════════════════════════════

#[test]
fn analyze_ten_token_alternatives() {
    let tokens: Vec<(String, String)> = (0..10)
        .map(|i| (format!("t{i}"), format!("t{i}")))
        .collect();
    let tok_refs: Vec<(&str, &str)> = tokens
        .iter()
        .map(|(a, b)| (a.as_str(), b.as_str()))
        .collect();
    let rules: Vec<(&str, Vec<&str>)> = tokens
        .iter()
        .map(|(n, _)| ("s", vec![n.as_str()]))
        .collect();
    let pt = build_table("big_alt", &tok_refs, &rules, "s");
    let mut ca = ConflictAnalyzer::new();
    let _stats = ca.analyze_table(&pt);
}

#[test]
fn analyze_long_sequence() {
    let tokens: Vec<(String, String)> =
        (0..8).map(|i| (format!("t{i}"), format!("t{i}"))).collect();
    let tok_refs: Vec<(&str, &str)> = tokens
        .iter()
        .map(|(a, b)| (a.as_str(), b.as_str()))
        .collect();
    let rhs: Vec<&str> = tokens.iter().map(|(n, _)| n.as_str()).collect();
    let pt = build_table("long_seq", &tok_refs, &[("s", rhs)], "s");
    let mut ca = ConflictAnalyzer::new();
    let _stats = ca.analyze_table(&pt);
}

#[test]
fn analyze_left_recursive_grammar() {
    let pt = build_table(
        "leftrec",
        &[("a", "a"), ("b", "b")],
        &[("s", vec!["s", "a"]), ("s", vec!["b"])],
        "s",
    );
    let mut ca = ConflictAnalyzer::new();
    let _stats = ca.analyze_table(&pt);
}

#[test]
fn analyze_right_recursive_grammar() {
    let pt = build_table(
        "rightrec",
        &[("a", "a"), ("b", "b")],
        &[("s", vec!["a", "s"]), ("s", vec!["b"])],
        "s",
    );
    let mut ca = ConflictAnalyzer::new();
    let _stats = ca.analyze_table(&pt);
}

#[test]
fn analyze_nested_nonterminals() {
    let pt = build_table(
        "nested",
        &[("x", "x"), ("y", "y"), ("z", "z")],
        &[
            ("a", vec!["x"]),
            ("b", vec!["a", "y"]),
            ("c", vec!["b", "z"]),
            ("s", vec!["c"]),
        ],
        "s",
    );
    let mut ca = ConflictAnalyzer::new();
    let _stats = ca.analyze_table(&pt);
}

#[test]
fn analyze_multiple_nonterminal_alternatives() {
    let pt = build_table(
        "multi_nt",
        &[("x", "x"), ("y", "y"), ("z", "z")],
        &[
            ("a", vec!["x"]),
            ("b", vec!["y"]),
            ("c", vec!["z"]),
            ("s", vec!["a"]),
            ("s", vec!["b"]),
            ("s", vec!["c"]),
        ],
        "s",
    );
    let mut ca = ConflictAnalyzer::new();
    let _stats = ca.analyze_table(&pt);
}

#[test]
fn analyze_expression_grammar_with_terms() {
    let pt = build_table(
        "expr_terms",
        &[("num", "[0-9]+"), ("plus", "\\+")],
        &[
            ("term", vec!["num"]),
            ("expr", vec!["term"]),
            ("expr", vec!["expr", "plus", "term"]),
            ("s", vec!["expr"]),
        ],
        "s",
    );
    let mut ca = ConflictAnalyzer::new();
    let _stats = ca.analyze_table(&pt);
}

#[test]
fn analyze_grammar_with_many_nonterminals() {
    let pt = build_table(
        "many_nt",
        &[("a", "a"), ("b", "b"), ("c", "c"), ("d", "d"), ("e", "e")],
        &[
            ("n1", vec!["a"]),
            ("n2", vec!["b"]),
            ("n3", vec!["c"]),
            ("n4", vec!["d"]),
            ("n5", vec!["e"]),
            ("s", vec!["n1", "n2", "n3", "n4", "n5"]),
        ],
        "s",
    );
    let mut ca = ConflictAnalyzer::new();
    let _stats = ca.analyze_table(&pt);
}

// ════════════════════════════════════════════════════════════════════════════
// 9. PrecedenceResolver
// ════════════════════════════════════════════════════════════════════════════

#[test]
fn precedence_resolver_new_empty_grammar() {
    let g = GrammarBuilder::new("empty")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let _pr = PrecedenceResolver::new(&g);
}

#[test]
fn precedence_resolver_no_match_returns_none() {
    let g = GrammarBuilder::new("none")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let pr = PrecedenceResolver::new(&g);
    // No precedences registered, so resolution returns None
    assert!(
        pr.can_resolve_shift_reduce(SymbolId(99), SymbolId(99))
            .is_none()
    );
}

#[test]
fn precedence_resolver_shift_higher() {
    let mut g = Grammar::new("test".to_string());
    g.precedences.push(Precedence {
        level: 2,
        associativity: Associativity::Left,
        symbols: vec![SymbolId(1)], // shift symbol
    });
    g.rules.insert(
        SymbolId(3),
        vec![Rule {
            lhs: SymbolId(3),
            rhs: vec![Symbol::Terminal(SymbolId(1))],
            precedence: Some(PrecedenceKind::Static(1)),
            associativity: Some(Associativity::Left),
            fields: Vec::new(),
            production_id: ProductionId(0),
        }],
    );
    let pr = PrecedenceResolver::new(&g);
    let decision = pr.can_resolve_shift_reduce(SymbolId(1), SymbolId(3));
    assert_eq!(decision, Some(PrecedenceDecision::PreferShift));
}

#[test]
fn precedence_resolver_reduce_higher() {
    let mut g = Grammar::new("test".to_string());
    g.precedences.push(Precedence {
        level: 1,
        associativity: Associativity::Left,
        symbols: vec![SymbolId(1)], // shift symbol
    });
    g.rules.insert(
        SymbolId(3),
        vec![Rule {
            lhs: SymbolId(3),
            rhs: vec![Symbol::Terminal(SymbolId(1))],
            precedence: Some(PrecedenceKind::Static(2)),
            associativity: Some(Associativity::Left),
            fields: Vec::new(),
            production_id: ProductionId(0),
        }],
    );
    let pr = PrecedenceResolver::new(&g);
    let decision = pr.can_resolve_shift_reduce(SymbolId(1), SymbolId(3));
    assert_eq!(decision, Some(PrecedenceDecision::PreferReduce));
}

#[test]
fn precedence_resolver_same_prec_left_assoc() {
    let mut g = Grammar::new("test".to_string());
    g.precedences.push(Precedence {
        level: 1,
        associativity: Associativity::Left,
        symbols: vec![SymbolId(1)],
    });
    g.rules.insert(
        SymbolId(3),
        vec![Rule {
            lhs: SymbolId(3),
            rhs: vec![Symbol::Terminal(SymbolId(1))],
            precedence: Some(PrecedenceKind::Static(1)),
            associativity: Some(Associativity::Left),
            fields: Vec::new(),
            production_id: ProductionId(0),
        }],
    );
    let pr = PrecedenceResolver::new(&g);
    let decision = pr.can_resolve_shift_reduce(SymbolId(1), SymbolId(3));
    assert_eq!(decision, Some(PrecedenceDecision::PreferReduce));
}

#[test]
fn precedence_resolver_same_prec_right_assoc() {
    let mut g = Grammar::new("test".to_string());
    g.precedences.push(Precedence {
        level: 1,
        associativity: Associativity::Right,
        symbols: vec![SymbolId(1)],
    });
    g.rules.insert(
        SymbolId(3),
        vec![Rule {
            lhs: SymbolId(3),
            rhs: vec![Symbol::Terminal(SymbolId(1))],
            precedence: Some(PrecedenceKind::Static(1)),
            associativity: Some(Associativity::Right),
            fields: Vec::new(),
            production_id: ProductionId(0),
        }],
    );
    let pr = PrecedenceResolver::new(&g);
    let decision = pr.can_resolve_shift_reduce(SymbolId(1), SymbolId(3));
    assert_eq!(decision, Some(PrecedenceDecision::PreferShift));
}

#[test]
fn precedence_resolver_same_prec_none_assoc() {
    let mut g = Grammar::new("test".to_string());
    g.precedences.push(Precedence {
        level: 1,
        associativity: Associativity::None,
        symbols: vec![SymbolId(1)],
    });
    g.rules.insert(
        SymbolId(3),
        vec![Rule {
            lhs: SymbolId(3),
            rhs: vec![Symbol::Terminal(SymbolId(1))],
            precedence: Some(PrecedenceKind::Static(1)),
            associativity: Some(Associativity::None),
            fields: Vec::new(),
            production_id: ProductionId(0),
        }],
    );
    let pr = PrecedenceResolver::new(&g);
    let decision = pr.can_resolve_shift_reduce(SymbolId(1), SymbolId(3));
    assert_eq!(decision, Some(PrecedenceDecision::Error));
}

#[test]
fn precedence_resolver_unknown_shift_symbol() {
    let mut g = Grammar::new("test".to_string());
    g.precedences.push(Precedence {
        level: 1,
        associativity: Associativity::Left,
        symbols: vec![SymbolId(1)],
    });
    g.rules.insert(
        SymbolId(3),
        vec![Rule {
            lhs: SymbolId(3),
            rhs: vec![Symbol::Terminal(SymbolId(1))],
            precedence: Some(PrecedenceKind::Static(1)),
            associativity: Some(Associativity::Left),
            fields: Vec::new(),
            production_id: ProductionId(0),
        }],
    );
    let pr = PrecedenceResolver::new(&g);
    // SymbolId(99) is not in token_precedences
    assert!(
        pr.can_resolve_shift_reduce(SymbolId(99), SymbolId(3))
            .is_none()
    );
}

#[test]
fn precedence_resolver_unknown_reduce_symbol() {
    let mut g = Grammar::new("test".to_string());
    g.precedences.push(Precedence {
        level: 1,
        associativity: Associativity::Left,
        symbols: vec![SymbolId(1)],
    });
    let pr = PrecedenceResolver::new(&g);
    // SymbolId(99) has no rules with precedence
    assert!(
        pr.can_resolve_shift_reduce(SymbolId(1), SymbolId(99))
            .is_none()
    );
}

#[test]
fn precedence_resolver_multiple_token_precs() {
    let mut g = Grammar::new("test".to_string());
    g.precedences.push(Precedence {
        level: 1,
        associativity: Associativity::Left,
        symbols: vec![SymbolId(1)],
    });
    g.precedences.push(Precedence {
        level: 2,
        associativity: Associativity::Right,
        symbols: vec![SymbolId(2)],
    });
    g.rules.insert(
        SymbolId(3),
        vec![Rule {
            lhs: SymbolId(3),
            rhs: vec![Symbol::Terminal(SymbolId(1))],
            precedence: Some(PrecedenceKind::Static(1)),
            associativity: Some(Associativity::Left),
            fields: Vec::new(),
            production_id: ProductionId(0),
        }],
    );
    let pr = PrecedenceResolver::new(&g);

    // Shift SymbolId(2) has prec 2 > reduce SymbolId(3) prec 1 → PreferShift
    let d1 = pr.can_resolve_shift_reduce(SymbolId(2), SymbolId(3));
    assert_eq!(d1, Some(PrecedenceDecision::PreferShift));

    // Shift SymbolId(1) same prec as reduce SymbolId(3), left assoc → PreferReduce
    let d2 = pr.can_resolve_shift_reduce(SymbolId(1), SymbolId(3));
    assert_eq!(d2, Some(PrecedenceDecision::PreferReduce));
}

#[test]
fn precedence_resolver_from_builder_grammar() {
    let g = build_grammar_with_prec(
        "builder_prec",
        &[("num", "[0-9]+"), ("plus", "\\+")],
        &[("expr", vec!["num"]), ("s", vec!["expr"])],
        &[("expr", vec!["expr", "plus", "expr"], 1, Associativity::Left)],
        "s",
    );
    let _pr = PrecedenceResolver::new(&g);
}

#[test]
fn precedence_resolver_from_builder_grammar_with_decl() {
    let g = GrammarBuilder::new("decl_prec")
        .token("num", "[0-9]+")
        .token("plus", "\\+")
        .token("star", "\\*")
        .precedence(1, Associativity::Left, vec!["plus"])
        .precedence(2, Associativity::Left, vec!["star"])
        .rule("expr", vec!["num"])
        .rule("expr", vec!["expr", "plus", "expr"])
        .rule("expr", vec!["expr", "star", "expr"])
        .start("expr")
        .build();
    let _pr = PrecedenceResolver::new(&g);
}

// ════════════════════════════════════════════════════════════════════════════
// 10. Fork actions in action table
// ════════════════════════════════════════════════════════════════════════════

#[test]
fn action_shift_variant() {
    let a = Action::Shift(adze_ir::StateId(5));
    assert!(matches!(a, Action::Shift(_)));
}

#[test]
fn action_reduce_variant() {
    let a = Action::Reduce(adze_ir::RuleId(3));
    assert!(matches!(a, Action::Reduce(_)));
}

#[test]
fn action_accept_variant() {
    let a = Action::Accept;
    assert!(matches!(a, Action::Accept));
}

#[test]
fn action_error_variant() {
    let a = Action::Error;
    assert!(matches!(a, Action::Error));
}

#[test]
fn action_fork_empty() {
    let a = Action::Fork(vec![]);
    assert!(matches!(a, Action::Fork(_)));
}

#[test]
fn action_fork_with_shift_reduce() {
    let a = Action::Fork(vec![
        Action::Shift(adze_ir::StateId(1)),
        Action::Reduce(adze_ir::RuleId(2)),
    ]);
    if let Action::Fork(ref inner) = a {
        assert_eq!(inner.len(), 2);
        assert!(matches!(inner[0], Action::Shift(_)));
        assert!(matches!(inner[1], Action::Reduce(_)));
    } else {
        panic!("expected Fork");
    }
}

#[test]
fn action_fork_with_multiple_reduces() {
    let a = Action::Fork(vec![
        Action::Reduce(adze_ir::RuleId(0)),
        Action::Reduce(adze_ir::RuleId(1)),
        Action::Reduce(adze_ir::RuleId(2)),
    ]);
    if let Action::Fork(ref inner) = a {
        assert_eq!(inner.len(), 3);
    } else {
        panic!("expected Fork");
    }
}

#[test]
fn action_fork_nested() {
    let a = Action::Fork(vec![
        Action::Fork(vec![Action::Shift(adze_ir::StateId(0))]),
        Action::Accept,
    ]);
    if let Action::Fork(ref inner) = a {
        assert_eq!(inner.len(), 2);
        assert!(matches!(inner[0], Action::Fork(_)));
    } else {
        panic!("expected Fork");
    }
}

#[test]
fn action_debug_impl() {
    let a = Action::Shift(adze_ir::StateId(0));
    let dbg = format!("{a:?}");
    assert!(!dbg.is_empty());
}

// ════════════════════════════════════════════════════════════════════════════
// Additional: PrecedenceDecision
// ════════════════════════════════════════════════════════════════════════════

#[test]
fn precedence_decision_prefer_shift_eq() {
    assert_eq!(
        PrecedenceDecision::PreferShift,
        PrecedenceDecision::PreferShift
    );
}

#[test]
fn precedence_decision_prefer_reduce_eq() {
    assert_eq!(
        PrecedenceDecision::PreferReduce,
        PrecedenceDecision::PreferReduce
    );
}

#[test]
fn precedence_decision_error_eq() {
    assert_eq!(PrecedenceDecision::Error, PrecedenceDecision::Error);
}

#[test]
fn precedence_decision_ne() {
    assert_ne!(
        PrecedenceDecision::PreferShift,
        PrecedenceDecision::PreferReduce
    );
    assert_ne!(PrecedenceDecision::PreferShift, PrecedenceDecision::Error);
    assert_ne!(PrecedenceDecision::PreferReduce, PrecedenceDecision::Error);
}

#[test]
fn precedence_decision_debug() {
    let d = PrecedenceDecision::PreferShift;
    let dbg = format!("{d:?}");
    assert!(dbg.contains("PreferShift"));
}

#[test]
fn precedence_decision_clone() {
    let d = PrecedenceDecision::PreferReduce;
    let c = d.clone();
    assert_eq!(d, c);
}

// ════════════════════════════════════════════════════════════════════════════
// Additional: Parse table queries after analysis
// ════════════════════════════════════════════════════════════════════════════

#[test]
fn analyzer_then_query_table_eof() {
    let pt = build_table("t", &[("a", "a")], &[("s", vec!["a"])], "s");
    let mut ca = ConflictAnalyzer::new();
    let _stats = ca.analyze_table(&pt);
    let _ = pt.eof();
}

#[test]
fn analyzer_then_query_table_start() {
    let pt = build_table("t", &[("a", "a")], &[("s", vec!["a"])], "s");
    let mut ca = ConflictAnalyzer::new();
    let _stats = ca.analyze_table(&pt);
    let _ = pt.start_symbol();
}

#[test]
fn analyzer_then_query_table_grammar() {
    let pt = build_table("t", &[("a", "a")], &[("s", vec!["a"])], "s");
    let mut ca = ConflictAnalyzer::new();
    let _stats = ca.analyze_table(&pt);
    let _ = pt.grammar();
}

#[test]
fn analyzer_with_expr_grammar_state_count() {
    let pt = build_table(
        "expr",
        &[("num", "[0-9]+"), ("plus", "\\+")],
        &[
            ("term", vec!["num"]),
            ("expr", vec!["term"]),
            ("expr", vec!["expr", "plus", "term"]),
            ("s", vec!["expr"]),
        ],
        "s",
    );
    let mut ca = ConflictAnalyzer::new();
    let _stats = ca.analyze_table(&pt);
    assert!(pt.state_count > 0);
}
