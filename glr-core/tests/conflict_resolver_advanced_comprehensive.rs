//! Comprehensive tests for advanced conflict resolution in `adze-glr-core`.
//!
//! Covers: `ConflictAnalyzer`, `ConflictStats`, `PrecedenceResolver`,
//! `PrecedenceDecision`, and their interactions with various grammar shapes.

use adze_glr_core::advanced_conflict::{
    ConflictAnalyzer, ConflictStats, PrecedenceDecision, PrecedenceResolver,
};
use adze_glr_core::{LexMode, ParseTable, StateId};
use adze_ir::builder::GrammarBuilder;
use adze_ir::{
    Associativity, Grammar, Precedence, PrecedenceKind, ProductionId, Rule, Symbol, SymbolId,
    Token, TokenPattern,
};
use std::collections::BTreeMap;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Build a minimal ParseTable from a Grammar (enough for `analyze_table`).
fn minimal_table(grammar: Grammar) -> ParseTable {
    ParseTable {
        action_table: vec![vec![vec![adze_glr_core::Action::Shift(StateId(1))]]],
        goto_table: vec![],
        symbol_metadata: vec![],
        state_count: 1,
        symbol_count: 1,
        symbol_to_index: BTreeMap::new(),
        index_to_symbol: vec![],
        external_scanner_states: vec![],
        rules: vec![],
        nonterminal_to_index: BTreeMap::new(),
        goto_indexing: adze_glr_core::GotoIndexing::NonterminalMap,
        eof_symbol: SymbolId(0),
        start_symbol: SymbolId(1),
        grammar,
        initial_state: StateId(0),
        token_count: 1,
        external_token_count: 0,
        lex_modes: vec![LexMode {
            lex_state: 0,
            external_lex_state: 0,
        }],
        extras: vec![],
        dynamic_prec_by_rule: vec![],
        rule_assoc_by_rule: vec![],
        alias_sequences: vec![],
        field_names: vec![],
        field_map: BTreeMap::new(),
    }
}

/// Resolve a symbol name to its `SymbolId` via the grammar's rule_names or tokens.
fn sym(grammar: &Grammar, name: &str) -> SymbolId {
    for (&id, n) in &grammar.rule_names {
        if n == name {
            return id;
        }
    }
    for (&id, tok) in &grammar.tokens {
        if tok.name == name {
            return id;
        }
    }
    panic!("symbol `{name}` not found in grammar");
}

/// Build a single-operator grammar: `s → s OP s | NUM`
fn single_op_grammar(
    op_name: &str,
    prec: i16,
    assoc: Associativity,
) -> (Grammar, SymbolId, SymbolId) {
    let grammar = GrammarBuilder::new("single_op")
        .token("NUM", r"\d+")
        .token(op_name, op_name)
        .precedence(prec, assoc, vec![op_name])
        .rule_with_precedence("s", vec!["s", op_name, "s"], prec, assoc)
        .rule("s", vec!["NUM"])
        .start("s")
        .build();
    let op = sym(&grammar, op_name);
    let s = sym(&grammar, "s");
    (grammar, op, s)
}

// =========================================================================
// 1. ConflictAnalyzer::new()
// =========================================================================

#[test]
fn analyzer_new_returns_zero_stats() {
    let analyzer = ConflictAnalyzer::new();
    let s = analyzer.get_stats();
    assert_eq!(s.shift_reduce_conflicts, 0);
    assert_eq!(s.reduce_reduce_conflicts, 0);
    assert_eq!(s.precedence_resolved, 0);
    assert_eq!(s.associativity_resolved, 0);
    assert_eq!(s.explicit_glr, 0);
    assert_eq!(s.default_resolved, 0);
}

#[test]
fn analyzer_default_trait_same_as_new() {
    let a = ConflictAnalyzer::default();
    let b = ConflictAnalyzer::new();
    let sa = a.get_stats();
    let sb = b.get_stats();
    assert_eq!(sa.shift_reduce_conflicts, sb.shift_reduce_conflicts);
    assert_eq!(sa.reduce_reduce_conflicts, sb.reduce_reduce_conflicts);
    assert_eq!(sa.precedence_resolved, sb.precedence_resolved);
    assert_eq!(sa.associativity_resolved, sb.associativity_resolved);
    assert_eq!(sa.explicit_glr, sb.explicit_glr);
    assert_eq!(sa.default_resolved, sb.default_resolved);
}

#[test]
fn analyzer_new_get_stats_is_not_null() {
    let analyzer = ConflictAnalyzer::new();
    let _stats = analyzer.get_stats(); // must not panic
}

// =========================================================================
// 2. ConflictAnalyzer with simple grammars (no conflicts)
// =========================================================================

#[test]
fn analyze_single_token_grammar_no_conflicts() {
    let grammar = GrammarBuilder::new("single")
        .token("A", "a")
        .rule("start", vec!["A"])
        .start("start")
        .build();
    let table = minimal_table(grammar);
    let mut analyzer = ConflictAnalyzer::new();
    let stats = analyzer.analyze_table(&table);
    assert_eq!(stats.shift_reduce_conflicts, 0);
    assert_eq!(stats.reduce_reduce_conflicts, 0);
}

#[test]
fn analyze_two_token_sequence_grammar_no_conflicts() {
    let grammar = GrammarBuilder::new("seq")
        .token("A", "a")
        .token("B", "b")
        .rule("start", vec!["A", "B"])
        .start("start")
        .build();
    let table = minimal_table(grammar);
    let mut analyzer = ConflictAnalyzer::new();
    let stats = analyzer.analyze_table(&table);
    assert_eq!(stats.shift_reduce_conflicts, 0);
}

#[test]
fn analyze_three_alternative_rules_no_conflicts() {
    let grammar = GrammarBuilder::new("alts")
        .token("A", "a")
        .token("B", "b")
        .token("C", "c")
        .rule("start", vec!["A"])
        .rule("start", vec!["B"])
        .rule("start", vec!["C"])
        .start("start")
        .build();
    let table = minimal_table(grammar);
    let mut analyzer = ConflictAnalyzer::new();
    let stats = analyzer.analyze_table(&table);
    assert_eq!(stats.shift_reduce_conflicts, 0);
}

#[test]
fn analyze_nested_nonterminals_no_conflicts() {
    let grammar = GrammarBuilder::new("nested")
        .token("X", "x")
        .rule("inner", vec!["X"])
        .rule("start", vec!["inner"])
        .start("start")
        .build();
    let table = minimal_table(grammar);
    let mut analyzer = ConflictAnalyzer::new();
    let stats = analyzer.analyze_table(&table);
    assert_eq!(stats.shift_reduce_conflicts, 0);
    assert_eq!(stats.reduce_reduce_conflicts, 0);
}

#[test]
fn analyze_default_parse_table_no_conflicts() {
    let table = ParseTable::default();
    let mut analyzer = ConflictAnalyzer::new();
    let stats = analyzer.analyze_table(&table);
    assert_eq!(stats.shift_reduce_conflicts, 0);
    assert_eq!(stats.reduce_reduce_conflicts, 0);
    assert_eq!(stats.precedence_resolved, 0);
}

// =========================================================================
// 3. ConflictAnalyzer with precedence grammars
// =========================================================================

#[test]
fn analyze_arithmetic_grammar_with_precedence() {
    let grammar = GrammarBuilder::new("arith")
        .token("NUM", r"\d+")
        .token("+", "+")
        .token("*", "*")
        .precedence(1, Associativity::Left, vec!["+"])
        .precedence(2, Associativity::Left, vec!["*"])
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "*", "expr"], 2, Associativity::Left)
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    let table = minimal_table(grammar);
    let mut analyzer = ConflictAnalyzer::new();
    let stats = analyzer.analyze_table(&table);
    // Current simplified implementation returns zeros
    assert_eq!(stats.shift_reduce_conflicts, 0);
}

#[test]
fn analyze_right_assoc_grammar() {
    let grammar = GrammarBuilder::new("right")
        .token("NUM", r"\d+")
        .token("^", "^")
        .precedence(3, Associativity::Right, vec!["^"])
        .rule_with_precedence("expr", vec!["expr", "^", "expr"], 3, Associativity::Right)
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    let table = minimal_table(grammar);
    let mut analyzer = ConflictAnalyzer::new();
    let stats = analyzer.analyze_table(&table);
    assert_eq!(stats.shift_reduce_conflicts, 0);
}

#[test]
fn analyze_mixed_associativity_grammar() {
    let grammar = GrammarBuilder::new("mixed")
        .token("NUM", r"\d+")
        .token("+", "+")
        .token("^", "^")
        .precedence(1, Associativity::Left, vec!["+"])
        .precedence(2, Associativity::Right, vec!["^"])
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "^", "expr"], 2, Associativity::Right)
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    let table = minimal_table(grammar);
    let mut analyzer = ConflictAnalyzer::new();
    let stats = analyzer.analyze_table(&table);
    assert_eq!(stats.reduce_reduce_conflicts, 0);
}

// =========================================================================
// 4. ConflictStats field access
// =========================================================================

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
fn stats_all_fields_nonzero() {
    let stats = ConflictStats {
        shift_reduce_conflicts: 10,
        reduce_reduce_conflicts: 20,
        precedence_resolved: 30,
        associativity_resolved: 40,
        explicit_glr: 50,
        default_resolved: 60,
    };
    assert_eq!(stats.shift_reduce_conflicts, 10);
    assert_eq!(stats.reduce_reduce_conflicts, 20);
    assert_eq!(stats.precedence_resolved, 30);
    assert_eq!(stats.associativity_resolved, 40);
    assert_eq!(stats.explicit_glr, 50);
    assert_eq!(stats.default_resolved, 60);
}

#[test]
fn stats_clone_preserves_values() {
    let stats = ConflictStats {
        shift_reduce_conflicts: 7,
        reduce_reduce_conflicts: 11,
        precedence_resolved: 13,
        associativity_resolved: 17,
        explicit_glr: 19,
        default_resolved: 23,
    };
    let cloned = stats.clone();
    assert_eq!(cloned.shift_reduce_conflicts, 7);
    assert_eq!(cloned.reduce_reduce_conflicts, 11);
    assert_eq!(cloned.precedence_resolved, 13);
    assert_eq!(cloned.associativity_resolved, 17);
    assert_eq!(cloned.explicit_glr, 19);
    assert_eq!(cloned.default_resolved, 23);
}

#[test]
fn stats_clone_is_independent() {
    let stats = ConflictStats {
        shift_reduce_conflicts: 100,
        ..Default::default()
    };
    let mut cloned = stats.clone();
    cloned.shift_reduce_conflicts = 999;
    assert_eq!(stats.shift_reduce_conflicts, 100);
    assert_eq!(cloned.shift_reduce_conflicts, 999);
}

#[test]
fn stats_partial_initialization() {
    let stats = ConflictStats {
        explicit_glr: 42,
        ..Default::default()
    };
    assert_eq!(stats.shift_reduce_conflicts, 0);
    assert_eq!(stats.explicit_glr, 42);
}

#[test]
fn stats_large_values() {
    let stats = ConflictStats {
        shift_reduce_conflicts: usize::MAX,
        reduce_reduce_conflicts: usize::MAX,
        precedence_resolved: usize::MAX,
        associativity_resolved: usize::MAX,
        explicit_glr: usize::MAX,
        default_resolved: usize::MAX,
    };
    assert_eq!(stats.shift_reduce_conflicts, usize::MAX);
    assert_eq!(stats.default_resolved, usize::MAX);
}

// =========================================================================
// 5. ConflictStats Debug format
// =========================================================================

#[test]
fn stats_debug_contains_struct_name() {
    let stats = ConflictStats::default();
    let dbg = format!("{stats:?}");
    assert!(dbg.contains("ConflictStats"));
}

#[test]
fn stats_debug_contains_all_field_names() {
    let stats = ConflictStats {
        shift_reduce_conflicts: 1,
        reduce_reduce_conflicts: 2,
        precedence_resolved: 3,
        associativity_resolved: 4,
        explicit_glr: 5,
        default_resolved: 6,
    };
    let dbg = format!("{stats:?}");
    assert!(dbg.contains("shift_reduce_conflicts"));
    assert!(dbg.contains("reduce_reduce_conflicts"));
    assert!(dbg.contains("precedence_resolved"));
    assert!(dbg.contains("associativity_resolved"));
    assert!(dbg.contains("explicit_glr"));
    assert!(dbg.contains("default_resolved"));
}

#[test]
fn stats_debug_contains_values() {
    let stats = ConflictStats {
        shift_reduce_conflicts: 99,
        ..Default::default()
    };
    let dbg = format!("{stats:?}");
    assert!(dbg.contains("99"));
}

#[test]
fn stats_debug_zero_values_shown() {
    let stats = ConflictStats::default();
    let dbg = format!("{stats:?}");
    assert!(dbg.contains("0"));
}

// =========================================================================
// 6. PrecedenceResolver::new()
// =========================================================================

#[test]
fn resolver_new_from_empty_grammar() {
    let grammar = Grammar::new("empty".to_string());
    let resolver = PrecedenceResolver::new(&grammar);
    assert_eq!(
        resolver.can_resolve_shift_reduce(SymbolId(0), SymbolId(0)),
        None,
    );
}

#[test]
fn resolver_new_from_grammar_with_tokens_only() {
    let grammar = GrammarBuilder::new("tok_only")
        .token("X", "x")
        .precedence(1, Associativity::Left, vec!["X"])
        .build();
    let resolver = PrecedenceResolver::new(&grammar);
    let x = sym(&grammar, "X");
    // No rules with precedence → reduce side returns None
    assert_eq!(resolver.can_resolve_shift_reduce(x, SymbolId(999)), None);
}

#[test]
fn resolver_new_from_grammar_with_rules_only() {
    let grammar = GrammarBuilder::new("rules_only")
        .token("A", "a")
        .rule("s", vec!["A"])
        .start("s")
        .build();
    let resolver = PrecedenceResolver::new(&grammar);
    let a = sym(&grammar, "A");
    let s = sym(&grammar, "s");
    assert_eq!(resolver.can_resolve_shift_reduce(a, s), None);
}

#[test]
fn resolver_new_extracts_token_precedences() {
    let grammar = GrammarBuilder::new("tok_prec")
        .token("NUM", r"\d+")
        .token("+", "+")
        .precedence(1, Associativity::Left, vec!["+"])
        .rule_with_precedence("s", vec!["s", "+", "s"], 1, Associativity::Left)
        .rule("s", vec!["NUM"])
        .start("s")
        .build();
    let resolver = PrecedenceResolver::new(&grammar);
    let plus = sym(&grammar, "+");
    let s = sym(&grammar, "s");
    // Should resolve since both token and rule have precedence info
    assert!(resolver.can_resolve_shift_reduce(plus, s).is_some());
}

// =========================================================================
// 7. PrecedenceResolver with various grammars
// =========================================================================

#[test]
fn resolver_left_assoc_same_prec_prefers_reduce() {
    let (grammar, op, s) = single_op_grammar("+", 1, Associativity::Left);
    let resolver = PrecedenceResolver::new(&grammar);
    assert_eq!(
        resolver.can_resolve_shift_reduce(op, s),
        Some(PrecedenceDecision::PreferReduce),
    );
}

#[test]
fn resolver_right_assoc_same_prec_prefers_shift() {
    let (grammar, op, s) = single_op_grammar("^", 2, Associativity::Right);
    let resolver = PrecedenceResolver::new(&grammar);
    assert_eq!(
        resolver.can_resolve_shift_reduce(op, s),
        Some(PrecedenceDecision::PreferShift),
    );
}

#[test]
fn resolver_none_assoc_same_prec_returns_error() {
    let (grammar, op, s) = single_op_grammar("~", 3, Associativity::None);
    let resolver = PrecedenceResolver::new(&grammar);
    assert_eq!(
        resolver.can_resolve_shift_reduce(op, s),
        Some(PrecedenceDecision::Error),
    );
}

#[test]
fn resolver_shift_higher_prec_prefers_shift() {
    let grammar = GrammarBuilder::new("hi_shift")
        .token("NUM", r"\d+")
        .token("LO", "+")
        .token("HI", "*")
        .precedence(1, Associativity::Left, vec!["LO"])
        .precedence(5, Associativity::Left, vec!["HI"])
        .rule_with_precedence("s", vec!["s", "LO", "s"], 1, Associativity::Left)
        .rule("s", vec!["NUM"])
        .start("s")
        .build();
    let resolver = PrecedenceResolver::new(&grammar);
    let hi = sym(&grammar, "HI");
    let s = sym(&grammar, "s");
    assert_eq!(
        resolver.can_resolve_shift_reduce(hi, s),
        Some(PrecedenceDecision::PreferShift),
    );
}

#[test]
fn resolver_shift_lower_prec_prefers_reduce() {
    let grammar = GrammarBuilder::new("lo_shift")
        .token("NUM", r"\d+")
        .token("HI", "*")
        .token("LO", "+")
        .precedence(10, Associativity::Left, vec!["HI"])
        .precedence(1, Associativity::Left, vec!["LO"])
        .rule_with_precedence("s", vec!["s", "HI", "s"], 10, Associativity::Left)
        .rule("s", vec!["NUM"])
        .start("s")
        .build();
    let resolver = PrecedenceResolver::new(&grammar);
    let lo = sym(&grammar, "LO");
    let s = sym(&grammar, "s");
    assert_eq!(
        resolver.can_resolve_shift_reduce(lo, s),
        Some(PrecedenceDecision::PreferReduce),
    );
}

#[test]
fn resolver_negative_prec_levels() {
    let (grammar, op, s) = single_op_grammar("-", -5, Associativity::Left);
    let resolver = PrecedenceResolver::new(&grammar);
    assert_eq!(
        resolver.can_resolve_shift_reduce(op, s),
        Some(PrecedenceDecision::PreferReduce),
    );
}

#[test]
fn resolver_zero_prec_level_right_assoc() {
    let (grammar, op, s) = single_op_grammar("|", 0, Associativity::Right);
    let resolver = PrecedenceResolver::new(&grammar);
    assert_eq!(
        resolver.can_resolve_shift_reduce(op, s),
        Some(PrecedenceDecision::PreferShift),
    );
}

#[test]
fn resolver_unknown_shift_returns_none() {
    let (grammar, _op, s) = single_op_grammar("+", 1, Associativity::Left);
    let resolver = PrecedenceResolver::new(&grammar);
    assert_eq!(resolver.can_resolve_shift_reduce(SymbolId(999), s), None);
}

#[test]
fn resolver_unknown_reduce_returns_none() {
    let (grammar, op, _s) = single_op_grammar("+", 1, Associativity::Left);
    let resolver = PrecedenceResolver::new(&grammar);
    assert_eq!(resolver.can_resolve_shift_reduce(op, SymbolId(999)), None);
}

#[test]
fn resolver_both_unknown_returns_none() {
    let grammar = Grammar::new("e".to_string());
    let resolver = PrecedenceResolver::new(&grammar);
    assert_eq!(
        resolver.can_resolve_shift_reduce(SymbolId(100), SymbolId(200)),
        None,
    );
}

#[test]
fn resolver_multi_level_three_operators() {
    let grammar = GrammarBuilder::new("multi3")
        .token("NUM", r"\d+")
        .token("+", "+")
        .token("*", "*")
        .token("^", "^")
        .precedence(1, Associativity::Left, vec!["+"])
        .precedence(2, Associativity::Left, vec!["*"])
        .precedence(3, Associativity::Right, vec!["^"])
        .rule_with_precedence("e", vec!["e", "+", "e"], 1, Associativity::Left)
        .rule_with_precedence("e", vec!["e", "*", "e"], 2, Associativity::Left)
        .rule_with_precedence("e", vec!["e", "^", "e"], 3, Associativity::Right)
        .rule("e", vec!["NUM"])
        .start("e")
        .build();
    let resolver = PrecedenceResolver::new(&grammar);
    let plus = sym(&grammar, "+");
    let star = sym(&grammar, "*");
    let caret = sym(&grammar, "^");
    let e = sym(&grammar, "e");

    // + vs e (prec 1 rule): same prec left → reduce
    assert_eq!(
        resolver.can_resolve_shift_reduce(plus, e),
        Some(PrecedenceDecision::PreferReduce),
    );
    // * vs e: higher shift → shift (if e has lowest rule prec)
    // Note: symbol_precedences stores the *last* annotated rule for `e`
    // so the result depends on iteration order. We just check it's Some.
    assert!(resolver.can_resolve_shift_reduce(star, e).is_some());
    assert!(resolver.can_resolve_shift_reduce(caret, e).is_some());
}

#[test]
fn resolver_javascript_like_grammar_plus_no_rule_prec() {
    let grammar = GrammarBuilder::javascript_like();
    let resolver = PrecedenceResolver::new(&grammar);
    let plus = sym(&grammar, "+");
    let expr = sym(&grammar, "expression");
    // javascript_like grammar doesn't set rule-level precedence → None
    assert_eq!(resolver.can_resolve_shift_reduce(plus, expr), None);
}

#[test]
fn resolver_python_like_grammar() {
    let grammar = GrammarBuilder::python_like();
    let resolver = PrecedenceResolver::new(&grammar);
    // With no precedence annotations, everything returns None
    assert_eq!(
        resolver.can_resolve_shift_reduce(SymbolId(1), SymbolId(2)),
        None,
    );
}

#[test]
fn resolver_rule_without_assoc_not_stored() {
    let mut grammar = Grammar::new("no_assoc".to_string());
    let tok = SymbolId(1);
    let nt = SymbolId(10);
    grammar.tokens.insert(
        tok,
        Token {
            name: "t".into(),
            pattern: TokenPattern::String("t".into()),
            fragile: false,
        },
    );
    grammar.precedences.push(Precedence {
        level: 1,
        associativity: Associativity::Left,
        symbols: vec![tok],
    });
    grammar.rules.insert(
        nt,
        vec![Rule {
            lhs: nt,
            rhs: vec![Symbol::Terminal(tok)],
            precedence: Some(PrecedenceKind::Static(1)),
            associativity: None, // missing → not stored
            fields: vec![],
            production_id: ProductionId(0),
        }],
    );
    let resolver = PrecedenceResolver::new(&grammar);
    assert_eq!(resolver.can_resolve_shift_reduce(tok, nt), None);
}

#[test]
fn resolver_rule_without_precedence_not_stored() {
    let mut grammar = Grammar::new("no_prec".to_string());
    let tok = SymbolId(1);
    let nt = SymbolId(10);
    grammar.tokens.insert(
        tok,
        Token {
            name: "t".into(),
            pattern: TokenPattern::String("t".into()),
            fragile: false,
        },
    );
    grammar.precedences.push(Precedence {
        level: 1,
        associativity: Associativity::Left,
        symbols: vec![tok],
    });
    grammar.rules.insert(
        nt,
        vec![Rule {
            lhs: nt,
            rhs: vec![Symbol::Terminal(tok)],
            precedence: None, // missing → not stored
            associativity: Some(Associativity::Left),
            fields: vec![],
            production_id: ProductionId(0),
        }],
    );
    let resolver = PrecedenceResolver::new(&grammar);
    assert_eq!(resolver.can_resolve_shift_reduce(tok, nt), None);
}

#[test]
fn resolver_multiple_tokens_same_level() {
    let mut grammar = Grammar::new("multi_tok".to_string());
    let tok_a = SymbolId(1);
    let tok_b = SymbolId(2);
    let nt = SymbolId(10);
    grammar.tokens.insert(
        tok_a,
        Token {
            name: "a".into(),
            pattern: TokenPattern::String("a".into()),
            fragile: false,
        },
    );
    grammar.tokens.insert(
        tok_b,
        Token {
            name: "b".into(),
            pattern: TokenPattern::String("b".into()),
            fragile: false,
        },
    );
    grammar.precedences.push(Precedence {
        level: 3,
        associativity: Associativity::Left,
        symbols: vec![tok_a, tok_b],
    });
    grammar.rules.insert(
        nt,
        vec![Rule {
            lhs: nt,
            rhs: vec![Symbol::Terminal(tok_a)],
            precedence: Some(PrecedenceKind::Static(3)),
            associativity: Some(Associativity::Left),
            fields: vec![],
            production_id: ProductionId(0),
        }],
    );
    let resolver = PrecedenceResolver::new(&grammar);
    assert_eq!(
        resolver.can_resolve_shift_reduce(tok_a, nt),
        Some(PrecedenceDecision::PreferReduce),
    );
    assert_eq!(
        resolver.can_resolve_shift_reduce(tok_b, nt),
        Some(PrecedenceDecision::PreferReduce),
    );
}

// =========================================================================
// 8. Multiple analyzers on same table
// =========================================================================

#[test]
fn two_analyzers_same_table_same_results() {
    let grammar = GrammarBuilder::new("shared")
        .token("A", "a")
        .rule("s", vec!["A"])
        .start("s")
        .build();
    let table = minimal_table(grammar);
    let mut a1 = ConflictAnalyzer::new();
    let mut a2 = ConflictAnalyzer::new();
    let s1 = a1.analyze_table(&table);
    let s2 = a2.analyze_table(&table);
    assert_eq!(s1.shift_reduce_conflicts, s2.shift_reduce_conflicts);
    assert_eq!(s1.reduce_reduce_conflicts, s2.reduce_reduce_conflicts);
    assert_eq!(s1.precedence_resolved, s2.precedence_resolved);
    assert_eq!(s1.associativity_resolved, s2.associativity_resolved);
    assert_eq!(s1.explicit_glr, s2.explicit_glr);
    assert_eq!(s1.default_resolved, s2.default_resolved);
}

#[test]
fn analyzer_reuse_resets_on_second_call() {
    let g1 = GrammarBuilder::new("g1")
        .token("A", "a")
        .rule("s", vec!["A"])
        .start("s")
        .build();
    let g2 = GrammarBuilder::new("g2")
        .token("B", "b")
        .rule("s", vec!["B"])
        .start("s")
        .build();
    let t1 = minimal_table(g1);
    let t2 = minimal_table(g2);
    let mut analyzer = ConflictAnalyzer::new();
    let _s1 = analyzer.analyze_table(&t1);
    let s2 = analyzer.analyze_table(&t2);
    // analyze_table resets stats before analysis
    assert_eq!(s2.shift_reduce_conflicts, 0);
}

#[test]
fn analyzer_get_stats_reflects_last_analysis() {
    let grammar = GrammarBuilder::new("t")
        .token("X", "x")
        .rule("s", vec!["X"])
        .start("s")
        .build();
    let table = minimal_table(grammar);
    let mut analyzer = ConflictAnalyzer::new();
    let returned = analyzer.analyze_table(&table);
    let stored = analyzer.get_stats().clone();
    assert_eq!(
        returned.shift_reduce_conflicts,
        stored.shift_reduce_conflicts,
    );
    assert_eq!(
        returned.reduce_reduce_conflicts,
        stored.reduce_reduce_conflicts,
    );
}

#[test]
fn three_analyzers_independent() {
    let grammar = GrammarBuilder::new("t")
        .token("X", "x")
        .rule("s", vec!["X"])
        .start("s")
        .build();
    let table = minimal_table(grammar);
    let mut a1 = ConflictAnalyzer::new();
    let mut a2 = ConflictAnalyzer::new();
    let mut a3 = ConflictAnalyzer::new();
    let s1 = a1.analyze_table(&table);
    let s2 = a2.analyze_table(&table);
    let s3 = a3.analyze_table(&table);
    assert_eq!(s1.shift_reduce_conflicts, s2.shift_reduce_conflicts);
    assert_eq!(s2.shift_reduce_conflicts, s3.shift_reduce_conflicts);
}

// =========================================================================
// 9. Conflict statistics consistency
// =========================================================================

#[test]
fn stats_returned_and_stored_are_consistent() {
    let grammar = GrammarBuilder::new("c")
        .token("A", "a")
        .token("B", "b")
        .rule("s", vec!["A", "B"])
        .start("s")
        .build();
    let table = minimal_table(grammar);
    let mut analyzer = ConflictAnalyzer::new();
    let returned = analyzer.analyze_table(&table);
    let stored = analyzer.get_stats();
    assert_eq!(
        returned.shift_reduce_conflicts,
        stored.shift_reduce_conflicts
    );
    assert_eq!(
        returned.reduce_reduce_conflicts,
        stored.reduce_reduce_conflicts
    );
    assert_eq!(returned.precedence_resolved, stored.precedence_resolved);
    assert_eq!(
        returned.associativity_resolved,
        stored.associativity_resolved
    );
    assert_eq!(returned.explicit_glr, stored.explicit_glr);
    assert_eq!(returned.default_resolved, stored.default_resolved);
}

#[test]
fn stats_sum_is_self_consistent() {
    // For any ConflictStats the resolved counts should not exceed total conflicts
    let stats = ConflictStats {
        shift_reduce_conflicts: 5,
        reduce_reduce_conflicts: 3,
        precedence_resolved: 2,
        associativity_resolved: 1,
        explicit_glr: 3,
        default_resolved: 2,
    };
    let total_conflicts = stats.shift_reduce_conflicts + stats.reduce_reduce_conflicts;
    let total_resolved = stats.precedence_resolved
        + stats.associativity_resolved
        + stats.explicit_glr
        + stats.default_resolved;
    // Sanity: we defined these values so they add up
    assert_eq!(total_conflicts, 8);
    assert_eq!(total_resolved, 8);
}

#[test]
fn analyze_then_reanalyze_gives_fresh_results() {
    let grammar = GrammarBuilder::new("fresh")
        .token("A", "a")
        .rule("s", vec!["A"])
        .start("s")
        .build();
    let table = minimal_table(grammar);
    let mut analyzer = ConflictAnalyzer::new();
    let first = analyzer.analyze_table(&table);
    let second = analyzer.analyze_table(&table);
    assert_eq!(first.shift_reduce_conflicts, second.shift_reduce_conflicts);
    assert_eq!(
        first.reduce_reduce_conflicts,
        second.reduce_reduce_conflicts
    );
}

// =========================================================================
// 10. Edge cases
// =========================================================================

#[test]
fn edge_empty_grammar_analyze() {
    let grammar = Grammar::new("empty".to_string());
    let table = minimal_table(grammar);
    let mut analyzer = ConflictAnalyzer::new();
    let stats = analyzer.analyze_table(&table);
    assert_eq!(stats.shift_reduce_conflicts, 0);
}

#[test]
fn edge_single_epsilon_rule() {
    let grammar = GrammarBuilder::new("eps")
        .token("A", "a")
        .rule("s", vec![]) // epsilon rule
        .start("s")
        .build();
    let table = minimal_table(grammar);
    let mut analyzer = ConflictAnalyzer::new();
    let stats = analyzer.analyze_table(&table);
    assert_eq!(stats.shift_reduce_conflicts, 0);
}

#[test]
fn edge_many_tokens_no_rules() {
    let mut builder = GrammarBuilder::new("many_tok");
    for i in 0..20 {
        builder = builder.token(&format!("T{i}"), &format!("t{i}"));
    }
    let grammar = builder.build();
    let resolver = PrecedenceResolver::new(&grammar);
    assert_eq!(
        resolver.can_resolve_shift_reduce(SymbolId(1), SymbolId(2)),
        None,
    );
}

#[test]
fn edge_many_precedence_levels() {
    // Use raw Grammar API for many precedence levels
    let mut grammar = Grammar::new("many_prec".to_string());
    let num = SymbolId(1);
    grammar.tokens.insert(
        num,
        Token {
            name: "NUM".into(),
            pattern: TokenPattern::Regex(r"\d+".into()),
            fragile: false,
        },
    );
    for i in 0u16..10 {
        let tok = SymbolId(100 + i);
        grammar.tokens.insert(
            tok,
            Token {
                name: format!("op{i}"),
                pattern: TokenPattern::String(format!("op{i}")),
                fragile: false,
            },
        );
        grammar.precedences.push(Precedence {
            level: i as i16,
            associativity: Associativity::Left,
            symbols: vec![tok],
        });
    }
    let nt = SymbolId(200);
    grammar.rule_names.insert(nt, "expr".into());
    grammar.rules.insert(
        nt,
        vec![Rule {
            lhs: nt,
            rhs: vec![Symbol::Terminal(num)],
            precedence: Some(PrecedenceKind::Static(5)),
            associativity: Some(Associativity::Left),
            fields: vec![],
            production_id: ProductionId(0),
        }],
    );
    let resolver = PrecedenceResolver::new(&grammar);
    // op9 (prec 9) vs expr (prec 5) → shift higher
    assert_eq!(
        resolver.can_resolve_shift_reduce(SymbolId(109), nt),
        Some(PrecedenceDecision::PreferShift),
    );
    // op0 (prec 0) vs expr (prec 5) → reduce higher
    assert_eq!(
        resolver.can_resolve_shift_reduce(SymbolId(100), nt),
        Some(PrecedenceDecision::PreferReduce),
    );
    // op5 (prec 5) vs expr (prec 5) → same prec, left assoc → reduce
    assert_eq!(
        resolver.can_resolve_shift_reduce(SymbolId(105), nt),
        Some(PrecedenceDecision::PreferReduce),
    );
}

#[test]
fn edge_large_grammar_many_rules() {
    let mut grammar = Grammar::new("large".to_string());
    let num = SymbolId(1);
    grammar.tokens.insert(
        num,
        Token {
            name: "NUM".into(),
            pattern: TokenPattern::Regex(r"\d+".into()),
            fragile: false,
        },
    );
    // Create 50 nonterminals each with a simple rule
    for i in 0u16..50 {
        let nt = SymbolId(100 + i);
        grammar.rule_names.insert(nt, format!("nt{i}"));
        grammar.rules.insert(
            nt,
            vec![Rule {
                lhs: nt,
                rhs: vec![Symbol::Terminal(num)],
                precedence: None,
                associativity: None,
                fields: vec![],
                production_id: ProductionId(i),
            }],
        );
    }
    let table = minimal_table(grammar);
    let mut analyzer = ConflictAnalyzer::new();
    let stats = analyzer.analyze_table(&table);
    assert_eq!(stats.shift_reduce_conflicts, 0);
}

#[test]
fn edge_resolver_high_symbol_ids() {
    let mut grammar = Grammar::new("high_ids".to_string());
    let tok = SymbolId(60000);
    let nt = SymbolId(65000);
    grammar.tokens.insert(
        tok,
        Token {
            name: "t".into(),
            pattern: TokenPattern::String("t".into()),
            fragile: false,
        },
    );
    grammar.precedences.push(Precedence {
        level: 7,
        associativity: Associativity::Right,
        symbols: vec![tok],
    });
    grammar.rule_names.insert(nt, "N".into());
    grammar.rules.insert(
        nt,
        vec![Rule {
            lhs: nt,
            rhs: vec![Symbol::Terminal(tok)],
            precedence: Some(PrecedenceKind::Static(7)),
            associativity: Some(Associativity::Right),
            fields: vec![],
            production_id: ProductionId(0),
        }],
    );
    let resolver = PrecedenceResolver::new(&grammar);
    assert_eq!(
        resolver.can_resolve_shift_reduce(tok, nt),
        Some(PrecedenceDecision::PreferShift),
    );
}

// =========================================================================
// PrecedenceDecision trait tests
// =========================================================================

#[test]
fn decision_eq_all_variants() {
    assert_eq!(
        PrecedenceDecision::PreferShift,
        PrecedenceDecision::PreferShift
    );
    assert_eq!(
        PrecedenceDecision::PreferReduce,
        PrecedenceDecision::PreferReduce
    );
    assert_eq!(PrecedenceDecision::Error, PrecedenceDecision::Error);
}

#[test]
fn decision_ne_all_pairs() {
    assert_ne!(
        PrecedenceDecision::PreferShift,
        PrecedenceDecision::PreferReduce
    );
    assert_ne!(PrecedenceDecision::PreferShift, PrecedenceDecision::Error);
    assert_ne!(PrecedenceDecision::PreferReduce, PrecedenceDecision::Error);
}

#[test]
fn decision_clone_all() {
    for d in [
        PrecedenceDecision::PreferShift,
        PrecedenceDecision::PreferReduce,
        PrecedenceDecision::Error,
    ] {
        assert_eq!(d.clone(), d);
    }
}

#[test]
fn decision_debug_prefer_shift() {
    let dbg = format!("{:?}", PrecedenceDecision::PreferShift);
    assert!(dbg.contains("PreferShift"));
}

#[test]
fn decision_debug_prefer_reduce() {
    let dbg = format!("{:?}", PrecedenceDecision::PreferReduce);
    assert!(dbg.contains("PreferReduce"));
}

#[test]
fn decision_debug_error() {
    let dbg = format!("{:?}", PrecedenceDecision::Error);
    assert!(dbg.contains("Error"));
}

// =========================================================================
// Additional PrecedenceResolver scenarios
// =========================================================================

#[test]
fn resolver_two_separate_resolvers_same_grammar() {
    let (grammar, op, s) = single_op_grammar("+", 2, Associativity::Left);
    let r1 = PrecedenceResolver::new(&grammar);
    let r2 = PrecedenceResolver::new(&grammar);
    assert_eq!(
        r1.can_resolve_shift_reduce(op, s),
        r2.can_resolve_shift_reduce(op, s),
    );
}

#[test]
fn resolver_prec_difference_of_one() {
    let mut grammar = Grammar::new("diff1".to_string());
    let tok = SymbolId(1);
    let nt = SymbolId(10);
    grammar.tokens.insert(
        tok,
        Token {
            name: "t".into(),
            pattern: TokenPattern::String("t".into()),
            fragile: false,
        },
    );
    grammar.precedences.push(Precedence {
        level: 6,
        associativity: Associativity::Left,
        symbols: vec![tok],
    });
    grammar.rules.insert(
        nt,
        vec![Rule {
            lhs: nt,
            rhs: vec![Symbol::Terminal(tok)],
            precedence: Some(PrecedenceKind::Static(5)),
            associativity: Some(Associativity::Left),
            fields: vec![],
            production_id: ProductionId(0),
        }],
    );
    let resolver = PrecedenceResolver::new(&grammar);
    // shift prec 6 > reduce prec 5 → prefer shift
    assert_eq!(
        resolver.can_resolve_shift_reduce(tok, nt),
        Some(PrecedenceDecision::PreferShift),
    );
}

#[test]
fn resolver_extreme_negative_prec() {
    let mut grammar = Grammar::new("extreme_neg".to_string());
    let tok = SymbolId(1);
    let nt = SymbolId(10);
    grammar.tokens.insert(
        tok,
        Token {
            name: "t".into(),
            pattern: TokenPattern::String("t".into()),
            fragile: false,
        },
    );
    grammar.precedences.push(Precedence {
        level: i16::MIN,
        associativity: Associativity::Left,
        symbols: vec![tok],
    });
    grammar.rules.insert(
        nt,
        vec![Rule {
            lhs: nt,
            rhs: vec![Symbol::Terminal(tok)],
            precedence: Some(PrecedenceKind::Static(i16::MIN)),
            associativity: Some(Associativity::Left),
            fields: vec![],
            production_id: ProductionId(0),
        }],
    );
    let resolver = PrecedenceResolver::new(&grammar);
    // same prec, left assoc → reduce
    assert_eq!(
        resolver.can_resolve_shift_reduce(tok, nt),
        Some(PrecedenceDecision::PreferReduce),
    );
}

#[test]
fn resolver_extreme_positive_prec() {
    let mut grammar = Grammar::new("extreme_pos".to_string());
    let tok = SymbolId(1);
    let nt = SymbolId(10);
    grammar.tokens.insert(
        tok,
        Token {
            name: "t".into(),
            pattern: TokenPattern::String("t".into()),
            fragile: false,
        },
    );
    grammar.precedences.push(Precedence {
        level: i16::MAX,
        associativity: Associativity::Right,
        symbols: vec![tok],
    });
    grammar.rules.insert(
        nt,
        vec![Rule {
            lhs: nt,
            rhs: vec![Symbol::Terminal(tok)],
            precedence: Some(PrecedenceKind::Static(i16::MAX)),
            associativity: Some(Associativity::Right),
            fields: vec![],
            production_id: ProductionId(0),
        }],
    );
    let resolver = PrecedenceResolver::new(&grammar);
    // same prec, right assoc → shift
    assert_eq!(
        resolver.can_resolve_shift_reduce(tok, nt),
        Some(PrecedenceDecision::PreferShift),
    );
}

#[test]
fn resolver_mixed_none_and_left_assoc() {
    let mut grammar = Grammar::new("mixed_assoc".to_string());
    let tok_a = SymbolId(1);
    let tok_b = SymbolId(2);
    let nt_a = SymbolId(10);
    let nt_b = SymbolId(11);
    for (tok_id, name) in [(tok_a, "a"), (tok_b, "b")] {
        grammar.tokens.insert(
            tok_id,
            Token {
                name: name.into(),
                pattern: TokenPattern::String(name.into()),
                fragile: false,
            },
        );
    }
    grammar.precedences.push(Precedence {
        level: 1,
        associativity: Associativity::Left,
        symbols: vec![tok_a],
    });
    grammar.precedences.push(Precedence {
        level: 1,
        associativity: Associativity::None,
        symbols: vec![tok_b],
    });
    grammar.rules.insert(
        nt_a,
        vec![Rule {
            lhs: nt_a,
            rhs: vec![Symbol::Terminal(tok_a)],
            precedence: Some(PrecedenceKind::Static(1)),
            associativity: Some(Associativity::Left),
            fields: vec![],
            production_id: ProductionId(0),
        }],
    );
    grammar.rules.insert(
        nt_b,
        vec![Rule {
            lhs: nt_b,
            rhs: vec![Symbol::Terminal(tok_b)],
            precedence: Some(PrecedenceKind::Static(1)),
            associativity: Some(Associativity::None),
            fields: vec![],
            production_id: ProductionId(1),
        }],
    );
    let resolver = PrecedenceResolver::new(&grammar);
    // tok_a (prec 1) vs nt_a (prec 1 left) → reduce
    assert_eq!(
        resolver.can_resolve_shift_reduce(tok_a, nt_a),
        Some(PrecedenceDecision::PreferReduce),
    );
    // tok_b (prec 1) vs nt_b (prec 1 none) → error
    assert_eq!(
        resolver.can_resolve_shift_reduce(tok_b, nt_b),
        Some(PrecedenceDecision::Error),
    );
}

// =========================================================================
// Analyzer with various grammar builders
// =========================================================================

#[test]
fn analyzer_javascript_like_grammar() {
    let grammar = GrammarBuilder::javascript_like();
    let table = minimal_table(grammar);
    let mut analyzer = ConflictAnalyzer::new();
    let stats = analyzer.analyze_table(&table);
    assert_eq!(stats.shift_reduce_conflicts, 0);
}

#[test]
fn analyzer_python_like_grammar() {
    let grammar = GrammarBuilder::python_like();
    let table = minimal_table(grammar);
    let mut analyzer = ConflictAnalyzer::new();
    let stats = analyzer.analyze_table(&table);
    assert_eq!(stats.reduce_reduce_conflicts, 0);
}

#[test]
fn analyzer_with_extras_grammar() {
    let grammar = GrammarBuilder::new("extras")
        .token("A", "a")
        .token("WS", r"\s+")
        .extra("WS")
        .rule("s", vec!["A"])
        .start("s")
        .build();
    let table = minimal_table(grammar);
    let mut analyzer = ConflictAnalyzer::new();
    let stats = analyzer.analyze_table(&table);
    assert_eq!(stats.shift_reduce_conflicts, 0);
}

#[test]
fn analyzer_sequential_calls_all_zero() {
    let grammars: Vec<Grammar> = (0..5)
        .map(|i| {
            GrammarBuilder::new(&format!("g{i}"))
                .token("A", "a")
                .rule("s", vec!["A"])
                .start("s")
                .build()
        })
        .collect();
    let mut analyzer = ConflictAnalyzer::new();
    for g in grammars {
        let table = minimal_table(g);
        let stats = analyzer.analyze_table(&table);
        assert_eq!(stats.shift_reduce_conflicts, 0);
        assert_eq!(stats.reduce_reduce_conflicts, 0);
    }
}

// =========================================================================
// Stats structural tests
// =========================================================================

#[test]
fn stats_default_clone_equals_default() {
    let a = ConflictStats::default();
    let b = a.clone();
    assert_eq!(a.shift_reduce_conflicts, b.shift_reduce_conflicts);
    assert_eq!(a.reduce_reduce_conflicts, b.reduce_reduce_conflicts);
    assert_eq!(a.precedence_resolved, b.precedence_resolved);
    assert_eq!(a.associativity_resolved, b.associativity_resolved);
    assert_eq!(a.explicit_glr, b.explicit_glr);
    assert_eq!(a.default_resolved, b.default_resolved);
}

#[test]
fn stats_debug_is_nonempty() {
    let stats = ConflictStats::default();
    let dbg = format!("{stats:?}");
    assert!(!dbg.is_empty());
}

#[test]
fn stats_field_mutation_does_not_affect_original() {
    let original = ConflictStats {
        shift_reduce_conflicts: 5,
        reduce_reduce_conflicts: 10,
        ..Default::default()
    };
    let mut cloned = original.clone();
    cloned.shift_reduce_conflicts = 0;
    cloned.reduce_reduce_conflicts = 0;
    assert_eq!(cloned.shift_reduce_conflicts, 0);
    assert_eq!(cloned.reduce_reduce_conflicts, 0);
    assert_eq!(original.shift_reduce_conflicts, 5);
    assert_eq!(original.reduce_reduce_conflicts, 10);
}
