//! Comprehensive tests for `advanced_conflict` module.
//!
//! Covers `ConflictAnalyzer`, `ConflictStats`, `PrecedenceResolver`, and
//! `PrecedenceDecision` across a wide range of grammar shapes.

use adze_glr_core::advanced_conflict::{
    ConflictAnalyzer, ConflictStats, PrecedenceDecision, PrecedenceResolver,
};
use adze_glr_core::{Action, LexMode, ParseTable, StateId};
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
        action_table: vec![vec![vec![Action::Shift(StateId(1))]]],
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
    // Check rule_names first
    for (&id, n) in &grammar.rule_names {
        if n == name {
            return id;
        }
    }
    // Then tokens
    for (&id, tok) in &grammar.tokens {
        if tok.name == name {
            return id;
        }
    }
    panic!("symbol `{name}` not found in grammar");
}

// =========================================================================
// ConflictStats tests
// =========================================================================

#[test]
fn stats_default_is_all_zeros() {
    let stats = ConflictStats::default();
    assert_eq!(stats.shift_reduce_conflicts, 0);
    assert_eq!(stats.reduce_reduce_conflicts, 0);
    assert_eq!(stats.precedence_resolved, 0);
    assert_eq!(stats.associativity_resolved, 0);
    assert_eq!(stats.explicit_glr, 0);
    assert_eq!(stats.default_resolved, 0);
}

#[test]
fn stats_clone_preserves_values() {
    let stats = ConflictStats {
        shift_reduce_conflicts: 3,
        reduce_reduce_conflicts: 7,
        precedence_resolved: 2,
        ..Default::default()
    };
    let cloned = stats.clone();
    assert_eq!(cloned.shift_reduce_conflicts, 3);
    assert_eq!(cloned.reduce_reduce_conflicts, 7);
    assert_eq!(cloned.precedence_resolved, 2);
}

#[test]
fn stats_debug_format() {
    let stats = ConflictStats::default();
    let dbg = format!("{stats:?}");
    assert!(dbg.contains("ConflictStats"));
    assert!(dbg.contains("shift_reduce_conflicts"));
}

// =========================================================================
// ConflictAnalyzer tests
// =========================================================================

#[test]
fn analyzer_new_has_zero_stats() {
    let analyzer = ConflictAnalyzer::new();
    let s = analyzer.get_stats();
    assert_eq!(s.shift_reduce_conflicts, 0);
    assert_eq!(s.reduce_reduce_conflicts, 0);
}

#[test]
fn analyzer_default_equals_new() {
    let a = ConflictAnalyzer::default();
    let b = ConflictAnalyzer::new();
    assert_eq!(
        a.get_stats().shift_reduce_conflicts,
        b.get_stats().shift_reduce_conflicts
    );
    assert_eq!(
        a.get_stats().reduce_reduce_conflicts,
        b.get_stats().reduce_reduce_conflicts
    );
}

#[test]
fn analyze_trivial_table_no_conflicts() {
    let grammar = GrammarBuilder::new("trivial")
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
fn analyze_table_returns_same_as_get_stats() {
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
        stored.shift_reduce_conflicts
    );
    assert_eq!(
        returned.reduce_reduce_conflicts,
        stored.reduce_reduce_conflicts
    );
}

#[test]
fn analyze_table_resets_stats_on_second_call() {
    let grammar = GrammarBuilder::new("t")
        .token("X", "x")
        .rule("s", vec!["X"])
        .start("s")
        .build();
    let table = minimal_table(grammar);
    let mut analyzer = ConflictAnalyzer::new();
    let _ = analyzer.analyze_table(&table);
    // Calling again should reset, not accumulate.
    let stats = analyzer.analyze_table(&table);
    assert_eq!(stats.shift_reduce_conflicts, 0);
}

#[test]
fn analyze_default_table() {
    let table = ParseTable::default();
    let mut analyzer = ConflictAnalyzer::new();
    let stats = analyzer.analyze_table(&table);
    assert_eq!(stats.shift_reduce_conflicts, 0);
}

// =========================================================================
// PrecedenceDecision tests
// =========================================================================

#[test]
fn decision_eq() {
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
fn decision_ne() {
    assert_ne!(
        PrecedenceDecision::PreferShift,
        PrecedenceDecision::PreferReduce
    );
    assert_ne!(PrecedenceDecision::PreferReduce, PrecedenceDecision::Error);
    assert_ne!(PrecedenceDecision::PreferShift, PrecedenceDecision::Error);
}

#[test]
fn decision_clone() {
    let d = PrecedenceDecision::PreferShift;
    assert_eq!(d, d.clone());
}

#[test]
fn decision_debug_format() {
    let dbg = format!("{:?}", PrecedenceDecision::Error);
    assert!(dbg.contains("Error"));
}

// =========================================================================
// PrecedenceResolver — single operator grammars
// =========================================================================

/// Build a single-operator grammar: `s → s OP s | NUM`
/// with a single precedence level and associativity on the rule.
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

#[test]
fn resolver_shift_higher_precedence_prefers_shift() {
    // Token at prec 5 vs rule at prec 2 → shift wins.
    let grammar = GrammarBuilder::new("hi_shift")
        .token("A", "a")
        .token("B", "b")
        .precedence(5, Associativity::Left, vec!["B"])
        .rule_with_precedence("s", vec!["s", "A", "s"], 2, Associativity::Left)
        .rule("s", vec!["A"])
        .start("s")
        .build();
    let b = sym(&grammar, "B");
    let s = sym(&grammar, "s");
    let resolver = PrecedenceResolver::new(&grammar);
    assert_eq!(
        resolver.can_resolve_shift_reduce(b, s),
        Some(PrecedenceDecision::PreferShift)
    );
}

#[test]
fn resolver_shift_lower_precedence_prefers_reduce() {
    // Token at prec 1 vs rule at prec 3 → reduce wins.
    let grammar = GrammarBuilder::new("lo_shift")
        .token("A", "a")
        .token("B", "b")
        .precedence(1, Associativity::Left, vec!["A"])
        .rule_with_precedence("s", vec!["s", "A", "s"], 3, Associativity::Left)
        .rule("s", vec!["B"])
        .start("s")
        .build();
    let a = sym(&grammar, "A");
    let s = sym(&grammar, "s");
    let resolver = PrecedenceResolver::new(&grammar);
    assert_eq!(
        resolver.can_resolve_shift_reduce(a, s),
        Some(PrecedenceDecision::PreferReduce)
    );
}

#[test]
fn resolver_same_prec_left_assoc_prefers_reduce() {
    let (grammar, op, s) = single_op_grammar("+", 1, Associativity::Left);
    let resolver = PrecedenceResolver::new(&grammar);
    assert_eq!(
        resolver.can_resolve_shift_reduce(op, s),
        Some(PrecedenceDecision::PreferReduce)
    );
}

#[test]
fn resolver_same_prec_right_assoc_prefers_shift() {
    let (grammar, op, s) = single_op_grammar("^", 1, Associativity::Right);
    let resolver = PrecedenceResolver::new(&grammar);
    assert_eq!(
        resolver.can_resolve_shift_reduce(op, s),
        Some(PrecedenceDecision::PreferShift)
    );
}

#[test]
fn resolver_same_prec_none_assoc_returns_error() {
    let (grammar, op, s) = single_op_grammar("~", 1, Associativity::None);
    let resolver = PrecedenceResolver::new(&grammar);
    assert_eq!(
        resolver.can_resolve_shift_reduce(op, s),
        Some(PrecedenceDecision::Error)
    );
}

// =========================================================================
// PrecedenceResolver — missing information
// =========================================================================

#[test]
fn resolver_unknown_shift_symbol_returns_none() {
    let (grammar, _op, s) = single_op_grammar("+", 1, Associativity::Left);
    let resolver = PrecedenceResolver::new(&grammar);
    let unknown = SymbolId(999);
    assert_eq!(resolver.can_resolve_shift_reduce(unknown, s), None);
}

#[test]
fn resolver_unknown_reduce_symbol_returns_none() {
    let (grammar, op, _s) = single_op_grammar("+", 1, Associativity::Left);
    let resolver = PrecedenceResolver::new(&grammar);
    let unknown = SymbolId(999);
    assert_eq!(resolver.can_resolve_shift_reduce(op, unknown), None);
}

#[test]
fn resolver_both_unknown_returns_none() {
    let grammar = GrammarBuilder::new("empty")
        .token("A", "a")
        .rule("s", vec!["A"])
        .start("s")
        .build();
    let resolver = PrecedenceResolver::new(&grammar);
    assert_eq!(
        resolver.can_resolve_shift_reduce(SymbolId(100), SymbolId(200)),
        None
    );
}

// =========================================================================
// PrecedenceResolver — grammar with no precedence info
// =========================================================================

#[test]
fn resolver_no_precedence_always_returns_none() {
    let grammar = GrammarBuilder::new("noprec")
        .token("X", "x")
        .token("Y", "y")
        .rule("s", vec!["X", "Y"])
        .start("s")
        .build();
    let resolver = PrecedenceResolver::new(&grammar);
    let x = sym(&grammar, "X");
    let s = sym(&grammar, "s");
    assert_eq!(resolver.can_resolve_shift_reduce(x, s), None);
}

// =========================================================================
// PrecedenceResolver — complex grammars
// =========================================================================

#[test]
fn resolver_javascript_like_grammar() {
    // javascript_like() uses rule_with_precedence but no .precedence()
    // declarations, so token_precedences is empty → None for all lookups.
    let grammar = GrammarBuilder::javascript_like();
    let resolver = PrecedenceResolver::new(&grammar);
    let plus = sym(&grammar, "+");
    let expr = sym(&grammar, "expression");
    assert_eq!(resolver.can_resolve_shift_reduce(plus, expr), None);
}

#[test]
fn resolver_multi_level_precedence() {
    // Use separate nonterminals so each gets its own precedence.
    let grammar = GrammarBuilder::new("multi")
        .token("NUM", r"\d+")
        .token("+", "+")
        .token("*", "*")
        .token("^", "^")
        .precedence(1, Associativity::Left, vec!["+"])
        .precedence(2, Associativity::Left, vec!["*"])
        .precedence(3, Associativity::Right, vec!["^"])
        .rule_with_precedence("add", vec!["add", "+", "add"], 1, Associativity::Left)
        .rule("add", vec!["NUM"])
        .rule_with_precedence("mul", vec!["mul", "*", "mul"], 2, Associativity::Left)
        .rule("mul", vec!["NUM"])
        .rule_with_precedence("pow", vec!["pow", "^", "pow"], 3, Associativity::Right)
        .rule("pow", vec!["NUM"])
        .start("add")
        .build();

    let resolver = PrecedenceResolver::new(&grammar);
    let plus = sym(&grammar, "+");
    let star = sym(&grammar, "*");
    let caret = sym(&grammar, "^");
    let add = sym(&grammar, "add");
    let mul = sym(&grammar, "mul");
    let pow = sym(&grammar, "pow");

    // ^ (3) vs add (1) → shift
    assert_eq!(
        resolver.can_resolve_shift_reduce(caret, add),
        Some(PrecedenceDecision::PreferShift)
    );
    // * (2) vs add (1) → shift
    assert_eq!(
        resolver.can_resolve_shift_reduce(star, add),
        Some(PrecedenceDecision::PreferShift)
    );
    // + (1) vs add (1, left) → reduce
    assert_eq!(
        resolver.can_resolve_shift_reduce(plus, add),
        Some(PrecedenceDecision::PreferReduce)
    );
    // + (1) vs mul (2) → reduce (mul's prec higher)
    assert_eq!(
        resolver.can_resolve_shift_reduce(plus, mul),
        Some(PrecedenceDecision::PreferReduce)
    );
    // ^ (3) vs pow (3, right) → shift
    assert_eq!(
        resolver.can_resolve_shift_reduce(caret, pow),
        Some(PrecedenceDecision::PreferShift)
    );
}

// =========================================================================
// PrecedenceResolver — precedence from grammar.precedences only
// =========================================================================

#[test]
fn resolver_uses_precedences_from_grammar_declarations() {
    // Build grammar with raw Precedence + Rule (no GrammarBuilder shortcut)
    let mut grammar = Grammar::new("raw".to_string());
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
        level: 5,
        associativity: Associativity::Left,
        symbols: vec![tok_a],
    });
    grammar.precedences.push(Precedence {
        level: 10,
        associativity: Associativity::Right,
        symbols: vec![tok_b],
    });
    grammar.rules.insert(
        nt,
        vec![Rule {
            lhs: nt,
            rhs: vec![Symbol::Terminal(tok_a)],
            precedence: Some(PrecedenceKind::Static(5)),
            associativity: Some(Associativity::Left),
            fields: vec![],
            production_id: ProductionId(0),
        }],
    );

    let resolver = PrecedenceResolver::new(&grammar);
    // b (prec 10) shift vs nt (prec 5) reduce → shift wins
    assert_eq!(
        resolver.can_resolve_shift_reduce(tok_b, nt),
        Some(PrecedenceDecision::PreferShift)
    );
    // a (prec 5) shift vs nt (prec 5, left) reduce → left assoc → reduce wins
    assert_eq!(
        resolver.can_resolve_shift_reduce(tok_a, nt),
        Some(PrecedenceDecision::PreferReduce)
    );
}

// =========================================================================
// PrecedenceResolver — multiple rules on the same nonterminal
// =========================================================================

#[test]
fn resolver_picks_first_annotated_rule_for_symbol_prec() {
    // When multiple rules map to the same nonterminal, the resolver stores
    // the last one encountered (HashMap insert semantics). We verify that
    // resolution still returns *some* decision.
    let grammar = GrammarBuilder::new("multi_rule")
        .token("A", "a")
        .token("B", "b")
        .precedence(1, Associativity::Left, vec!["A"])
        .precedence(2, Associativity::Left, vec!["B"])
        .rule_with_precedence("s", vec!["s", "A", "s"], 1, Associativity::Left)
        .rule_with_precedence("s", vec!["s", "B", "s"], 2, Associativity::Left)
        .rule("s", vec!["A"])
        .start("s")
        .build();

    let resolver = PrecedenceResolver::new(&grammar);
    let a = sym(&grammar, "A");
    let s = sym(&grammar, "s");
    // Should return Some — exact decision depends on insertion order.
    assert!(resolver.can_resolve_shift_reduce(a, s).is_some());
}

// =========================================================================
// Edge cases
// =========================================================================

#[test]
fn resolver_empty_grammar_returns_none() {
    let grammar = Grammar::new("empty".to_string());
    let resolver = PrecedenceResolver::new(&grammar);
    assert_eq!(
        resolver.can_resolve_shift_reduce(SymbolId(0), SymbolId(0)),
        None
    );
}

#[test]
fn resolver_grammar_with_only_tokens_no_rules() {
    let grammar = GrammarBuilder::new("tokens_only")
        .token("X", "x")
        .precedence(1, Associativity::Left, vec!["X"])
        .build();
    let resolver = PrecedenceResolver::new(&grammar);
    let x = sym(&grammar, "X");
    // No rule → no symbol_precedences → None
    assert_eq!(resolver.can_resolve_shift_reduce(x, SymbolId(999)), None);
}

#[test]
fn resolver_rule_without_associativity_is_not_stored() {
    // Rules that lack associativity should not populate symbol_precedences.
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
            associativity: None, // no associativity
            fields: vec![],
            production_id: ProductionId(0),
        }],
    );

    let resolver = PrecedenceResolver::new(&grammar);
    // token_precedences has tok, but symbol_precedences does NOT have nt
    assert_eq!(resolver.can_resolve_shift_reduce(tok, nt), None);
}

#[test]
fn analyzer_with_javascript_like_grammar() {
    let grammar = GrammarBuilder::javascript_like();
    let table = minimal_table(grammar);
    let mut analyzer = ConflictAnalyzer::new();
    let stats = analyzer.analyze_table(&table);
    // The simplified analyze_table always returns zero — ensure no panic.
    assert_eq!(stats.shift_reduce_conflicts, 0);
}

#[test]
fn analyzer_with_python_like_grammar() {
    let grammar = GrammarBuilder::python_like();
    let table = minimal_table(grammar);
    let mut analyzer = ConflictAnalyzer::new();
    let stats = analyzer.analyze_table(&table);
    assert_eq!(stats.reduce_reduce_conflicts, 0);
}
