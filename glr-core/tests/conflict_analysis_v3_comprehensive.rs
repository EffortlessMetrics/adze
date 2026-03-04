//! Comprehensive tests for ConflictAnalyzer, ConflictStats, and PrecedenceResolver.

use adze_glr_core::advanced_conflict::{
    ConflictAnalyzer, ConflictStats, PrecedenceDecision, PrecedenceResolver,
};
use adze_glr_core::{FirstFollowSets, ParseTable, build_lr1_automaton};
use adze_ir::Associativity;
use adze_ir::builder::GrammarBuilder;

// ---------------------------------------------------------------------------
// Helper: build a ParseTable from tokens + rules
// ---------------------------------------------------------------------------

fn build_pt(
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
    let mut grammar = b.build();
    grammar.normalize();
    let ff = FirstFollowSets::compute(&grammar).unwrap();
    build_lr1_automaton(&grammar, &ff).unwrap()
}

fn build_pt_with_prec(
    name: &str,
    tokens: &[(&str, &str)],
    rules: &[(&str, Vec<&str>, i16, Associativity)],
    plain_rules: &[(&str, Vec<&str>)],
    start: &str,
) -> ParseTable {
    let mut b = GrammarBuilder::new(name);
    for &(n, p) in tokens {
        b = b.token(n, p);
    }
    for (lhs, rhs, prec, assoc) in rules {
        b = b.rule_with_precedence(lhs, rhs.clone(), *prec, *assoc);
    }
    for (lhs, rhs) in plain_rules {
        b = b.rule(lhs, rhs.clone());
    }
    b = b.start(start);
    let mut grammar = b.build();
    grammar.normalize();
    let ff = FirstFollowSets::compute(&grammar).unwrap();
    build_lr1_automaton(&grammar, &ff).unwrap()
}

fn build_grammar_for_resolver(
    name: &str,
    tokens: &[(&str, &str)],
    prec_rules: &[(&str, Vec<&str>, i16, Associativity)],
    plain_rules: &[(&str, Vec<&str>)],
    prec_decls: &[(i16, Associativity, Vec<&str>)],
    start: &str,
) -> adze_ir::Grammar {
    let mut b = GrammarBuilder::new(name);
    for &(n, p) in tokens {
        b = b.token(n, p);
    }
    for (lhs, rhs, prec, assoc) in prec_rules {
        b = b.rule_with_precedence(lhs, rhs.clone(), *prec, *assoc);
    }
    for (lhs, rhs) in plain_rules {
        b = b.rule(lhs, rhs.clone());
    }
    for (level, assoc, symbols) in prec_decls {
        b = b.precedence(*level, *assoc, symbols.clone());
    }
    b = b.start(start);
    let mut grammar = b.build();
    grammar.normalize();
    grammar
}

// ===========================================================================
// ConflictAnalyzer – construction
// ===========================================================================

#[test]
fn ca_new_creates_instance() {
    let _ca = ConflictAnalyzer::new();
}

#[test]
fn ca_default_creates_instance() {
    let _ca = ConflictAnalyzer::default();
}

#[test]
fn ca_new_and_default_equivalent_stats() {
    let ca1 = ConflictAnalyzer::new();
    let ca2 = ConflictAnalyzer::default();
    let s1 = ca1.get_stats();
    let s2 = ca2.get_stats();
    assert_eq!(s1.shift_reduce_conflicts, s2.shift_reduce_conflicts);
    assert_eq!(s1.reduce_reduce_conflicts, s2.reduce_reduce_conflicts);
}

// ===========================================================================
// ConflictStats – default fields
// ===========================================================================

#[test]
fn stats_default_all_zero() {
    let stats = ConflictStats::default();
    assert_eq!(stats.shift_reduce_conflicts, 0);
    assert_eq!(stats.reduce_reduce_conflicts, 0);
    assert_eq!(stats.precedence_resolved, 0);
    assert_eq!(stats.associativity_resolved, 0);
    assert_eq!(stats.explicit_glr, 0);
    assert_eq!(stats.default_resolved, 0);
}

#[test]
fn stats_clone_equals_original() {
    let stats = ConflictStats {
        shift_reduce_conflicts: 3,
        reduce_reduce_conflicts: 2,
        precedence_resolved: 1,
        associativity_resolved: 4,
        explicit_glr: 5,
        default_resolved: 6,
    };
    let cloned = stats.clone();
    assert_eq!(cloned.shift_reduce_conflicts, 3);
    assert_eq!(cloned.reduce_reduce_conflicts, 2);
    assert_eq!(cloned.precedence_resolved, 1);
    assert_eq!(cloned.associativity_resolved, 4);
    assert_eq!(cloned.explicit_glr, 5);
    assert_eq!(cloned.default_resolved, 6);
}

#[test]
fn stats_debug_format() {
    let stats = ConflictStats::default();
    let dbg = format!("{:?}", stats);
    assert!(dbg.contains("ConflictStats"));
}

// ===========================================================================
// analyze_table – trivial grammars
// ===========================================================================

#[test]
fn analyze_single_token_grammar() {
    let pt = build_pt("single", &[("a", "a")], &[("s", vec!["a"])], "s");
    let mut ca = ConflictAnalyzer::new();
    let stats = ca.analyze_table(&pt);
    // Simple grammar – no conflicts expected
    assert_eq!(stats.shift_reduce_conflicts, 0);
    assert_eq!(stats.reduce_reduce_conflicts, 0);
}

#[test]
fn analyze_two_token_sequence() {
    let pt = build_pt(
        "seq2",
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
    let pt = build_pt(
        "seq3",
        &[("a", "a"), ("b", "b"), ("c", "c")],
        &[("s", vec!["a", "b", "c"])],
        "s",
    );
    let mut ca = ConflictAnalyzer::new();
    let stats = ca.analyze_table(&pt);
    assert_eq!(stats.reduce_reduce_conflicts, 0);
}

#[test]
fn analyze_empty_alternative() {
    // grammar: s -> a | b
    let pt = build_pt(
        "alt2",
        &[("a", "a"), ("b", "b")],
        &[("s", vec!["a"]), ("s", vec!["b"])],
        "s",
    );
    let mut ca = ConflictAnalyzer::new();
    let _stats = ca.analyze_table(&pt);
}

// ===========================================================================
// analyze_table – chain grammars
// ===========================================================================

#[test]
fn analyze_two_level_chain() {
    let pt = build_pt(
        "chain2",
        &[("x", "x")],
        &[("inner", vec!["x"]), ("s", vec!["inner"])],
        "s",
    );
    let mut ca = ConflictAnalyzer::new();
    let _stats = ca.analyze_table(&pt);
}

#[test]
fn analyze_three_level_chain() {
    let pt = build_pt(
        "chain3",
        &[("x", "x")],
        &[("c", vec!["x"]), ("bx", vec!["c"]), ("s", vec!["bx"])],
        "s",
    );
    let mut ca = ConflictAnalyzer::new();
    let _stats = ca.analyze_table(&pt);
}

// ===========================================================================
// analyze_table – recursive grammars
// ===========================================================================

#[test]
fn analyze_left_recursive_grammar() {
    let pt = build_pt(
        "lrec",
        &[("a", "a"), ("b", "b")],
        &[("s", vec!["s", "a"]), ("s", vec!["b"])],
        "s",
    );
    let mut ca = ConflictAnalyzer::new();
    let _stats = ca.analyze_table(&pt);
}

#[test]
fn analyze_right_recursive_grammar() {
    let pt = build_pt(
        "rrec",
        &[("a", "a"), ("b", "b")],
        &[("s", vec!["a", "s"]), ("s", vec!["b"])],
        "s",
    );
    let mut ca = ConflictAnalyzer::new();
    let _stats = ca.analyze_table(&pt);
}

#[test]
fn analyze_mutual_recursion() {
    let pt = build_pt(
        "mutual",
        &[("x", "x"), ("y", "y")],
        &[
            ("p", vec!["x", "q"]),
            ("p", vec!["x"]),
            ("q", vec!["y", "p"]),
            ("q", vec!["y"]),
            ("s", vec!["p"]),
        ],
        "s",
    );
    let mut ca = ConflictAnalyzer::new();
    let _stats = ca.analyze_table(&pt);
}

// ===========================================================================
// analyze_table – expression grammars (potential shift/reduce conflicts)
// ===========================================================================

#[test]
fn analyze_simple_expr() {
    let pt = build_pt(
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
    let stats = ca.analyze_table(&pt);
    let _ = stats.default_resolved;
}

#[test]
fn analyze_add_mul_expr() {
    let pt = build_pt(
        "addmul",
        &[("num", "[0-9]+"), ("plus", "\\+"), ("star", "\\*")],
        &[
            ("factor", vec!["num"]),
            ("term", vec!["factor"]),
            ("term", vec!["term", "star", "factor"]),
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
fn analyze_parenthesized_expr() {
    let pt = build_pt(
        "paren",
        &[
            ("num", "[0-9]+"),
            ("lp", "\\("),
            ("rp", "\\)"),
            ("plus", "\\+"),
        ],
        &[
            ("atom", vec!["num"]),
            ("atom", vec!["lp", "expr", "rp"]),
            ("expr", vec!["atom"]),
            ("expr", vec!["expr", "plus", "atom"]),
            ("s", vec!["expr"]),
        ],
        "s",
    );
    let mut ca = ConflictAnalyzer::new();
    let _stats = ca.analyze_table(&pt);
}

// ===========================================================================
// analyze_table – ambiguous grammars (potential reduce/reduce)
// ===========================================================================

#[test]
fn analyze_ambiguous_two_rules_same_rhs() {
    // Both nt1 and nt2 derive "a", and s derives both – ambiguous at reduce
    let pt = build_pt(
        "ambig_rr",
        &[("a", "a")],
        &[
            ("nt1", vec!["a"]),
            ("nt2", vec!["a"]),
            ("s", vec!["nt1"]),
            ("s", vec!["nt2"]),
        ],
        "s",
    );
    let mut ca = ConflictAnalyzer::new();
    let _stats = ca.analyze_table(&pt);
}

#[test]
fn analyze_three_way_ambiguity() {
    let pt = build_pt(
        "ambig3",
        &[("a", "a")],
        &[
            ("p1", vec!["a"]),
            ("p2", vec!["a"]),
            ("p3", vec!["a"]),
            ("s", vec!["p1"]),
            ("s", vec!["p2"]),
            ("s", vec!["p3"]),
        ],
        "s",
    );
    let mut ca = ConflictAnalyzer::new();
    let _stats = ca.analyze_table(&pt);
}

// ===========================================================================
// analyze_table – with precedence rules
// ===========================================================================

#[test]
fn analyze_prec_left() {
    let pt = build_pt_with_prec(
        "pleft",
        &[("x", "x"), ("y", "y")],
        &[
            ("s", vec!["x"], 1, Associativity::Left),
            ("s", vec!["y"], 2, Associativity::Left),
        ],
        &[],
        "s",
    );
    let mut ca = ConflictAnalyzer::new();
    let _stats = ca.analyze_table(&pt);
}

#[test]
fn analyze_prec_right() {
    let pt = build_pt_with_prec(
        "pright",
        &[("x", "x"), ("y", "y")],
        &[
            ("s", vec!["x"], 1, Associativity::Right),
            ("s", vec!["y"], 2, Associativity::Right),
        ],
        &[],
        "s",
    );
    let mut ca = ConflictAnalyzer::new();
    let _stats = ca.analyze_table(&pt);
}

#[test]
fn analyze_prec_none() {
    let pt = build_pt_with_prec(
        "pnone",
        &[("x", "x")],
        &[("s", vec!["x"], 1, Associativity::None)],
        &[],
        "s",
    );
    let mut ca = ConflictAnalyzer::new();
    let _stats = ca.analyze_table(&pt);
}

#[test]
fn analyze_mixed_prec_and_plain() {
    let pt = build_pt_with_prec(
        "mixprec",
        &[("a", "a"), ("b", "b"), ("c", "c")],
        &[("s", vec!["a"], 1, Associativity::Left)],
        &[("s", vec!["b"]), ("s", vec!["c"])],
        "s",
    );
    let mut ca = ConflictAnalyzer::new();
    let _stats = ca.analyze_table(&pt);
}

// ===========================================================================
// analyze_table – diamond / merge patterns
// ===========================================================================

#[test]
fn analyze_diamond_grammar() {
    let pt = build_pt(
        "diamond",
        &[("x", "x"), ("y", "y")],
        &[
            ("left", vec!["x"]),
            ("right", vec!["y"]),
            ("s", vec!["left"]),
            ("s", vec!["right"]),
        ],
        "s",
    );
    let mut ca = ConflictAnalyzer::new();
    let _stats = ca.analyze_table(&pt);
}

#[test]
fn analyze_deep_diamond() {
    let pt = build_pt(
        "deepdiamond",
        &[("x", "x"), ("y", "y")],
        &[
            ("la", vec!["x"]),
            ("lb", vec!["la"]),
            ("ra", vec!["y"]),
            ("rb", vec!["ra"]),
            ("s", vec!["lb"]),
            ("s", vec!["rb"]),
        ],
        "s",
    );
    let mut ca = ConflictAnalyzer::new();
    let _stats = ca.analyze_table(&pt);
}

// ===========================================================================
// analyze_table – many alternatives
// ===========================================================================

#[test]
fn analyze_five_alternatives() {
    let pt = build_pt(
        "alt5",
        &[("a", "a"), ("b", "b"), ("c", "c"), ("d", "d"), ("e", "e")],
        &[
            ("s", vec!["a"]),
            ("s", vec!["b"]),
            ("s", vec!["c"]),
            ("s", vec!["d"]),
            ("s", vec!["e"]),
        ],
        "s",
    );
    let mut ca = ConflictAnalyzer::new();
    let _stats = ca.analyze_table(&pt);
}

#[test]
fn analyze_ten_alternatives() {
    let tok: Vec<(String, String)> = (0..10)
        .map(|i| (format!("t{i}"), format!("t{i}")))
        .collect();
    let tok_refs: Vec<(&str, &str)> = tok.iter().map(|(a, b)| (a.as_str(), b.as_str())).collect();
    let rules: Vec<(&str, Vec<&str>)> = tok.iter().map(|(n, _)| ("s", vec![n.as_str()])).collect();
    let pt = build_pt("alt10", &tok_refs, &rules, "s");
    let mut ca = ConflictAnalyzer::new();
    let _stats = ca.analyze_table(&pt);
}

// ===========================================================================
// Multiple analyses on same analyzer
// ===========================================================================

#[test]
fn reuse_analyzer_for_two_tables() {
    let pt1 = build_pt("r1", &[("a", "a")], &[("s", vec!["a"])], "s");
    let pt2 = build_pt("r2", &[("b", "b")], &[("s", vec!["b"])], "s");
    let mut ca = ConflictAnalyzer::new();
    let _s1 = ca.analyze_table(&pt1);
    let _s2 = ca.analyze_table(&pt2);
}

#[test]
fn reuse_analyzer_stats_reset() {
    let pt = build_pt("reset", &[("a", "a")], &[("s", vec!["a"])], "s");
    let mut ca = ConflictAnalyzer::new();
    let s1 = ca.analyze_table(&pt);
    let s2 = ca.analyze_table(&pt);
    assert_eq!(s1.shift_reduce_conflicts, s2.shift_reduce_conflicts);
    assert_eq!(s1.reduce_reduce_conflicts, s2.reduce_reduce_conflicts);
    assert_eq!(s1.default_resolved, s2.default_resolved);
}

// ===========================================================================
// get_stats mirrors analyze_table result
// ===========================================================================

#[test]
fn get_stats_after_analyze() {
    let pt = build_pt("gs", &[("a", "a")], &[("s", vec!["a"])], "s");
    let mut ca = ConflictAnalyzer::new();
    let returned = ca.analyze_table(&pt);
    let stored = ca.get_stats();
    assert_eq!(
        returned.shift_reduce_conflicts,
        stored.shift_reduce_conflicts
    );
    assert_eq!(
        returned.reduce_reduce_conflicts,
        stored.reduce_reduce_conflicts
    );
    assert_eq!(returned.default_resolved, stored.default_resolved);
}

#[test]
fn get_stats_before_analyze_all_zero() {
    let ca = ConflictAnalyzer::new();
    let stats = ca.get_stats();
    assert_eq!(stats.shift_reduce_conflicts, 0);
    assert_eq!(stats.reduce_reduce_conflicts, 0);
    assert_eq!(stats.precedence_resolved, 0);
    assert_eq!(stats.associativity_resolved, 0);
    assert_eq!(stats.explicit_glr, 0);
    assert_eq!(stats.default_resolved, 0);
}

// ===========================================================================
// Determinism of analysis
// ===========================================================================

#[test]
fn analysis_is_deterministic() {
    let pt = build_pt(
        "det",
        &[("a", "a"), ("b", "b")],
        &[("s", vec!["a"]), ("s", vec!["b"])],
        "s",
    );
    let mut ca1 = ConflictAnalyzer::new();
    let mut ca2 = ConflictAnalyzer::new();
    let s1 = ca1.analyze_table(&pt);
    let s2 = ca2.analyze_table(&pt);
    assert_eq!(s1.shift_reduce_conflicts, s2.shift_reduce_conflicts);
    assert_eq!(s1.reduce_reduce_conflicts, s2.reduce_reduce_conflicts);
    assert_eq!(s1.default_resolved, s2.default_resolved);
    assert_eq!(s1.precedence_resolved, s2.precedence_resolved);
    assert_eq!(s1.associativity_resolved, s2.associativity_resolved);
    assert_eq!(s1.explicit_glr, s2.explicit_glr);
}

// ===========================================================================
// PrecedenceResolver – construction
// ===========================================================================

#[test]
fn pr_new_from_empty_grammar() {
    let grammar = build_grammar_for_resolver(
        "empty_pr",
        &[("a", "a")],
        &[],
        &[("s", vec!["a"])],
        &[],
        "s",
    );
    let _resolver = PrecedenceResolver::new(&grammar);
}

#[test]
fn pr_new_with_single_prec_decl() {
    let grammar = build_grammar_for_resolver(
        "single_pd",
        &[("plus", "\\+"), ("a", "a")],
        &[],
        &[("s", vec!["a"])],
        &[(1, Associativity::Left, vec!["plus"])],
        "s",
    );
    let _resolver = PrecedenceResolver::new(&grammar);
}

#[test]
fn pr_new_with_multiple_prec_decls() {
    let grammar = build_grammar_for_resolver(
        "multi_pd",
        &[("plus", "\\+"), ("star", "\\*"), ("a", "a")],
        &[],
        &[("s", vec!["a"])],
        &[
            (1, Associativity::Left, vec!["plus"]),
            (2, Associativity::Left, vec!["star"]),
        ],
        "s",
    );
    let _resolver = PrecedenceResolver::new(&grammar);
}

// ===========================================================================
// PrecedenceResolver – can_resolve_shift_reduce
// ===========================================================================

#[test]
fn pr_higher_shift_prec_prefers_shift() {
    let grammar = build_grammar_for_resolver(
        "hs",
        &[("plus", "\\+"), ("star", "\\*"), ("num", "[0-9]+")],
        &[("expr", vec!["num"], 1, Associativity::Left)],
        &[("s", vec!["expr"])],
        &[
            (1, Associativity::Left, vec!["plus"]),
            (2, Associativity::Left, vec!["star"]),
        ],
        "s",
    );
    let resolver = PrecedenceResolver::new(&grammar);
    // star (prec 2) vs expr rule (prec 1) → shift
    let star_id = grammar.find_symbol_by_name("star").unwrap();
    let expr_id = grammar.find_symbol_by_name("expr").unwrap();
    let decision = resolver.can_resolve_shift_reduce(star_id, expr_id);
    assert_eq!(decision, Some(PrecedenceDecision::PreferShift));
}

#[test]
fn pr_higher_reduce_prec_prefers_reduce() {
    let grammar = build_grammar_for_resolver(
        "hr",
        &[("plus", "\\+"), ("star", "\\*"), ("num", "[0-9]+")],
        &[("expr", vec!["num"], 2, Associativity::Left)],
        &[("s", vec!["expr"])],
        &[(1, Associativity::Left, vec!["plus"])],
        "s",
    );
    let resolver = PrecedenceResolver::new(&grammar);
    let plus_id = grammar.find_symbol_by_name("plus").unwrap();
    let expr_id = grammar.find_symbol_by_name("expr").unwrap();
    let decision = resolver.can_resolve_shift_reduce(plus_id, expr_id);
    assert_eq!(decision, Some(PrecedenceDecision::PreferReduce));
}

#[test]
fn pr_same_prec_left_assoc_prefers_reduce() {
    let grammar = build_grammar_for_resolver(
        "sla",
        &[("plus", "\\+"), ("num", "[0-9]+")],
        &[("expr", vec!["num"], 1, Associativity::Left)],
        &[("s", vec!["expr"])],
        &[(1, Associativity::Left, vec!["plus"])],
        "s",
    );
    let resolver = PrecedenceResolver::new(&grammar);
    let plus_id = grammar.find_symbol_by_name("plus").unwrap();
    let expr_id = grammar.find_symbol_by_name("expr").unwrap();
    let decision = resolver.can_resolve_shift_reduce(plus_id, expr_id);
    assert_eq!(decision, Some(PrecedenceDecision::PreferReduce));
}

#[test]
fn pr_same_prec_right_assoc_prefers_shift() {
    let grammar = build_grammar_for_resolver(
        "sra",
        &[("eq", "="), ("num", "[0-9]+")],
        &[("expr", vec!["num"], 1, Associativity::Right)],
        &[("s", vec!["expr"])],
        &[(1, Associativity::Right, vec!["eq"])],
        "s",
    );
    let resolver = PrecedenceResolver::new(&grammar);
    let eq_id = grammar.find_symbol_by_name("eq").unwrap();
    let expr_id = grammar.find_symbol_by_name("expr").unwrap();
    let decision = resolver.can_resolve_shift_reduce(eq_id, expr_id);
    assert_eq!(decision, Some(PrecedenceDecision::PreferShift));
}

#[test]
fn pr_same_prec_none_assoc_returns_error() {
    let grammar = build_grammar_for_resolver(
        "sna",
        &[("cmp", "<"), ("num", "[0-9]+")],
        &[("expr", vec!["num"], 1, Associativity::None)],
        &[("s", vec!["expr"])],
        &[(1, Associativity::None, vec!["cmp"])],
        "s",
    );
    let resolver = PrecedenceResolver::new(&grammar);
    let cmp_id = grammar.find_symbol_by_name("cmp").unwrap();
    let expr_id = grammar.find_symbol_by_name("expr").unwrap();
    let decision = resolver.can_resolve_shift_reduce(cmp_id, expr_id);
    assert_eq!(decision, Some(PrecedenceDecision::Error));
}

#[test]
fn pr_unknown_shift_symbol_returns_none() {
    let grammar = build_grammar_for_resolver(
        "unk_s",
        &[("a", "a"), ("num", "[0-9]+")],
        &[("expr", vec!["num"], 1, Associativity::Left)],
        &[("s", vec!["expr"])],
        &[], // no prec decl for "a"
        "s",
    );
    let resolver = PrecedenceResolver::new(&grammar);
    let a_id = grammar.find_symbol_by_name("a").unwrap();
    let expr_id = grammar.find_symbol_by_name("expr").unwrap();
    let decision = resolver.can_resolve_shift_reduce(a_id, expr_id);
    assert_eq!(decision, None);
}

#[test]
fn pr_unknown_reduce_symbol_returns_none() {
    let grammar = build_grammar_for_resolver(
        "unk_r",
        &[("plus", "\\+"), ("num", "[0-9]+")],
        &[], // no rule_with_precedence
        &[("expr", vec!["num"]), ("s", vec!["expr"])],
        &[(1, Associativity::Left, vec!["plus"])],
        "s",
    );
    let resolver = PrecedenceResolver::new(&grammar);
    let plus_id = grammar.find_symbol_by_name("plus").unwrap();
    let expr_id = grammar.find_symbol_by_name("expr").unwrap();
    let decision = resolver.can_resolve_shift_reduce(plus_id, expr_id);
    assert_eq!(decision, None);
}

#[test]
fn pr_both_unknown_returns_none() {
    let grammar = build_grammar_for_resolver(
        "unk_both",
        &[("a", "a"), ("b", "b")],
        &[],
        &[("s", vec!["a", "b"])],
        &[],
        "s",
    );
    let resolver = PrecedenceResolver::new(&grammar);
    let a_id = grammar.find_symbol_by_name("a").unwrap();
    let b_id = grammar.find_symbol_by_name("b").unwrap();
    let decision = resolver.can_resolve_shift_reduce(a_id, b_id);
    assert_eq!(decision, None);
}

// ===========================================================================
// PrecedenceDecision enum
// ===========================================================================

#[test]
fn prec_decision_debug() {
    assert!(format!("{:?}", PrecedenceDecision::PreferShift).contains("PreferShift"));
    assert!(format!("{:?}", PrecedenceDecision::PreferReduce).contains("PreferReduce"));
    assert!(format!("{:?}", PrecedenceDecision::Error).contains("Error"));
}

#[test]
fn prec_decision_clone_eq() {
    let d = PrecedenceDecision::PreferShift;
    assert_eq!(d, d.clone());
    let d2 = PrecedenceDecision::PreferReduce;
    assert_eq!(d2, d2.clone());
    let d3 = PrecedenceDecision::Error;
    assert_eq!(d3, d3.clone());
}

#[test]
fn prec_decision_ne() {
    assert_ne!(
        PrecedenceDecision::PreferShift,
        PrecedenceDecision::PreferReduce
    );
    assert_ne!(PrecedenceDecision::PreferShift, PrecedenceDecision::Error);
    assert_ne!(PrecedenceDecision::PreferReduce, PrecedenceDecision::Error);
}

// ===========================================================================
// Larger / stress-style grammars
// ===========================================================================

#[test]
fn analyze_long_sequence() {
    let toks: Vec<(String, String)> = (0..8).map(|i| (format!("t{i}"), format!("t{i}"))).collect();
    let tok_refs: Vec<(&str, &str)> = toks.iter().map(|(a, b)| (a.as_str(), b.as_str())).collect();
    let rhs: Vec<&str> = toks.iter().map(|(n, _)| n.as_str()).collect();
    let pt = build_pt("longseq", &tok_refs, &[("s", rhs)], "s");
    let mut ca = ConflictAnalyzer::new();
    let _stats = ca.analyze_table(&pt);
}

#[test]
fn analyze_wide_grammar_many_nonterminals() {
    let pt = build_pt(
        "wide",
        &[("x", "x"), ("y", "y"), ("z", "z"), ("w", "w")],
        &[
            ("na", vec!["x"]),
            ("nb", vec!["y"]),
            ("nc", vec!["z"]),
            ("nd", vec!["w"]),
            ("s", vec!["na"]),
            ("s", vec!["nb"]),
            ("s", vec!["nc"]),
            ("s", vec!["nd"]),
        ],
        "s",
    );
    let mut ca = ConflictAnalyzer::new();
    let _stats = ca.analyze_table(&pt);
}

#[test]
fn analyze_nested_binary_ops() {
    let pt = build_pt(
        "binops",
        &[
            ("num", "[0-9]+"),
            ("plus", "\\+"),
            ("star", "\\*"),
            ("lp", "\\("),
            ("rp", "\\)"),
        ],
        &[
            ("atom", vec!["num"]),
            ("atom", vec!["lp", "expr", "rp"]),
            ("factor", vec!["atom"]),
            ("term", vec!["factor"]),
            ("term", vec!["term", "star", "factor"]),
            ("expr", vec!["term"]),
            ("expr", vec!["expr", "plus", "term"]),
            ("s", vec!["expr"]),
        ],
        "s",
    );
    let mut ca = ConflictAnalyzer::new();
    let _stats = ca.analyze_table(&pt);
}

// ===========================================================================
// Combined analyzer + resolver workflows
// ===========================================================================

#[test]
fn analyzer_then_resolver_on_same_grammar() {
    let grammar = build_grammar_for_resolver(
        "combo",
        &[("plus", "\\+"), ("num", "[0-9]+")],
        &[("expr", vec!["num"], 1, Associativity::Left)],
        &[("s", vec!["expr"])],
        &[(1, Associativity::Left, vec!["plus"])],
        "s",
    );
    let ff = FirstFollowSets::compute(&grammar).unwrap();
    let pt = build_lr1_automaton(&grammar, &ff).unwrap();

    let mut ca = ConflictAnalyzer::new();
    let _stats = ca.analyze_table(&pt);
    let _resolver = PrecedenceResolver::new(&grammar);
}

#[test]
fn resolver_multiple_levels() {
    let grammar = build_grammar_for_resolver(
        "levels",
        &[
            ("plus", "\\+"),
            ("star", "\\*"),
            ("hat", "\\^"),
            ("num", "[0-9]+"),
        ],
        &[("expr", vec!["num"], 2, Associativity::Left)],
        &[("s", vec!["expr"])],
        &[
            (1, Associativity::Left, vec!["plus"]),
            (2, Associativity::Left, vec!["star"]),
            (3, Associativity::Right, vec!["hat"]),
        ],
        "s",
    );
    let resolver = PrecedenceResolver::new(&grammar);

    let plus_id = grammar.find_symbol_by_name("plus").unwrap();
    let star_id = grammar.find_symbol_by_name("star").unwrap();
    let hat_id = grammar.find_symbol_by_name("hat").unwrap();
    let expr_id = grammar.find_symbol_by_name("expr").unwrap();

    // plus (1) vs expr (2) → reduce
    assert_eq!(
        resolver.can_resolve_shift_reduce(plus_id, expr_id),
        Some(PrecedenceDecision::PreferReduce)
    );
    // star (2) vs expr (2) → same level, left assoc → reduce
    assert_eq!(
        resolver.can_resolve_shift_reduce(star_id, expr_id),
        Some(PrecedenceDecision::PreferReduce)
    );
    // hat (3) vs expr (2) → shift
    assert_eq!(
        resolver.can_resolve_shift_reduce(hat_id, expr_id),
        Some(PrecedenceDecision::PreferShift)
    );
}

// ===========================================================================
// ConflictStats field access patterns
// ===========================================================================

#[test]
fn stats_fields_independently_settable() {
    let s = ConflictStats {
        shift_reduce_conflicts: 10,
        reduce_reduce_conflicts: 20,
        precedence_resolved: 30,
        associativity_resolved: 40,
        explicit_glr: 50,
        default_resolved: 60,
    };
    assert_eq!(s.shift_reduce_conflicts, 10);
    assert_eq!(s.reduce_reduce_conflicts, 20);
    assert_eq!(s.precedence_resolved, 30);
    assert_eq!(s.associativity_resolved, 40);
    assert_eq!(s.explicit_glr, 50);
    assert_eq!(s.default_resolved, 60);
}

#[test]
fn stats_partial_fields_default() {
    let s = ConflictStats {
        shift_reduce_conflicts: 5,
        ..ConflictStats::default()
    };
    assert_eq!(s.shift_reduce_conflicts, 5);
    assert_eq!(s.reduce_reduce_conflicts, 0);
    assert_eq!(s.default_resolved, 0);
}

// ===========================================================================
// Edge cases
// ===========================================================================

#[test]
fn analyze_single_char_tokens() {
    let pt = build_pt("sc", &[("a", "a")], &[("s", vec!["a"])], "s");
    let mut ca = ConflictAnalyzer::new();
    let stats = ca.analyze_table(&pt);
    assert_eq!(stats.explicit_glr, 0);
}

#[test]
fn analyze_grammar_with_many_prec_levels() {
    let mut b = GrammarBuilder::new("manyprec");
    b = b.token("num", "[0-9]+");
    for i in 0..5 {
        let tok_name = format!("op{i}");
        let tok_pat = format!("o{i}");
        b = b.token(&tok_name, &tok_pat);
        b = b.rule_with_precedence("s", vec!["num"], (i + 1) as i16, Associativity::Left);
    }
    b = b.start("s");
    let mut grammar = b.build();
    grammar.normalize();
    let ff = FirstFollowSets::compute(&grammar).unwrap();
    let pt = build_lr1_automaton(&grammar, &ff).unwrap();
    let mut ca = ConflictAnalyzer::new();
    let _stats = ca.analyze_table(&pt);
}

#[test]
fn analyze_self_recursive_with_terminal() {
    // s -> s a | a  (left-recursive list)
    let pt = build_pt(
        "selflist",
        &[("a", "a")],
        &[("s", vec!["s", "a"]), ("s", vec!["a"])],
        "s",
    );
    let mut ca = ConflictAnalyzer::new();
    let _stats = ca.analyze_table(&pt);
}

#[test]
fn analyze_two_token_alternatives_then_sequence() {
    // s -> a | b | a b
    let pt = build_pt(
        "altseq",
        &[("a", "a"), ("b", "b")],
        &[("s", vec!["a"]), ("s", vec!["b"]), ("s", vec!["a", "b"])],
        "s",
    );
    let mut ca = ConflictAnalyzer::new();
    let _stats = ca.analyze_table(&pt);
}

#[test]
fn pr_resolver_with_right_assoc_rule() {
    let grammar = build_grammar_for_resolver(
        "rassoc_rule",
        &[("eq", "="), ("id", "[a-z]+")],
        &[("assign", vec!["id"], 1, Associativity::Right)],
        &[("s", vec!["assign"])],
        &[(1, Associativity::Right, vec!["eq"])],
        "s",
    );
    let resolver = PrecedenceResolver::new(&grammar);
    let eq_id = grammar.find_symbol_by_name("eq").unwrap();
    let assign_id = grammar.find_symbol_by_name("assign").unwrap();
    assert_eq!(
        resolver.can_resolve_shift_reduce(eq_id, assign_id),
        Some(PrecedenceDecision::PreferShift)
    );
}

#[test]
fn pr_resolver_with_mixed_assoc_levels() {
    let grammar = build_grammar_for_resolver(
        "mixassoc",
        &[("plus", "\\+"), ("eq", "="), ("num", "[0-9]+")],
        &[("expr", vec!["num"], 1, Associativity::Left)],
        &[("s", vec!["expr"])],
        &[
            (1, Associativity::Left, vec!["plus"]),
            (2, Associativity::Right, vec!["eq"]),
        ],
        "s",
    );
    let resolver = PrecedenceResolver::new(&grammar);
    let plus_id = grammar.find_symbol_by_name("plus").unwrap();
    let eq_id = grammar.find_symbol_by_name("eq").unwrap();
    let expr_id = grammar.find_symbol_by_name("expr").unwrap();

    // plus (1) vs expr (1, Left) → reduce
    assert_eq!(
        resolver.can_resolve_shift_reduce(plus_id, expr_id),
        Some(PrecedenceDecision::PreferReduce)
    );
    // eq (2) vs expr (1) → shift (higher shift prec)
    assert_eq!(
        resolver.can_resolve_shift_reduce(eq_id, expr_id),
        Some(PrecedenceDecision::PreferShift)
    );
}
