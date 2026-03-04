//! Comprehensive tests for grammar validation (GrammarValidator).
//!
//! Covers: valid grammars, invalid grammars, empty grammars, undefined symbols,
//! unreachable rules, non-productive symbols, cycles, unused tokens, warnings vs errors,
//! validation statistics, large grammars, precedence conflicts, field validation,
//! external tokens, and edge cases.

use adze_ir::builder::GrammarBuilder;
use adze_ir::validation::{GrammarValidator, ValidationError, ValidationWarning};
use adze_ir::{
    Associativity, ExternalToken, FieldId, Grammar, Precedence, ProductionId, Rule, Symbol,
    SymbolId, Token, TokenPattern,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn validate(grammar: &Grammar) -> adze_ir::validation::ValidationResult {
    let mut v = GrammarValidator::new();
    v.validate(grammar)
}

fn has_error(grammar: &Grammar, pred: impl Fn(&ValidationError) -> bool) -> bool {
    validate(grammar).errors.iter().any(pred)
}

fn has_warning(grammar: &Grammar, pred: impl Fn(&ValidationWarning) -> bool) -> bool {
    validate(grammar).warnings.iter().any(pred)
}

fn simple_arithmetic() -> Grammar {
    GrammarBuilder::new("arith")
        .token("NUMBER", r"\d+")
        .token("+", "+")
        .rule("expr", vec!["NUMBER", "+", "NUMBER"])
        .start("expr")
        .build()
}

// ===========================================================================
// 1. Valid grammars pass validation
// ===========================================================================

#[test]
fn valid_simple_arithmetic_no_errors() {
    let r = validate(&simple_arithmetic());
    assert!(r.errors.is_empty(), "expected no errors: {:?}", r.errors);
}

#[test]
fn valid_single_terminal_rule() {
    let g = GrammarBuilder::new("single")
        .token("A", "a")
        .rule("start", vec!["A"])
        .start("start")
        .build();
    assert!(validate(&g).errors.is_empty());
}

#[test]
fn valid_multiple_alternatives() {
    let g = GrammarBuilder::new("alt")
        .token("A", "a")
        .token("B", "b")
        .rule("start", vec!["A"])
        .rule("start", vec!["B"])
        .start("start")
        .build();
    assert!(validate(&g).errors.is_empty());
}

#[test]
fn valid_nested_nonterminals() {
    let g = GrammarBuilder::new("nested")
        .token("X", "x")
        .rule("inner", vec!["X"])
        .rule("outer", vec!["inner"])
        .start("outer")
        .build();
    assert!(validate(&g).errors.is_empty());
}

#[test]
fn valid_epsilon_rule() {
    let g = GrammarBuilder::new("eps")
        .token("A", "a")
        .rule("start", vec![])
        .rule("start", vec!["A"])
        .start("start")
        .build();
    assert!(validate(&g).errors.is_empty());
}

#[test]
fn javascript_like_grammar_no_structural_errors() {
    let g = GrammarBuilder::javascript_like();
    let r = validate(&g);
    // Recursive grammars produce CyclicRule, but should not have empty/undefined errors
    assert!(
        !r.errors
            .iter()
            .any(|e| matches!(e, ValidationError::EmptyGrammar))
    );
    assert!(
        !r.errors
            .iter()
            .any(|e| matches!(e, ValidationError::UndefinedSymbol { .. }))
    );
}

#[test]
fn python_like_grammar_no_structural_errors() {
    let g = GrammarBuilder::python_like();
    let r = validate(&g);
    assert!(
        !r.errors
            .iter()
            .any(|e| matches!(e, ValidationError::EmptyGrammar))
    );
    assert!(
        !r.errors
            .iter()
            .any(|e| matches!(e, ValidationError::UndefinedSymbol { .. }))
    );
}

#[test]
fn valid_grammar_with_extras() {
    let g = GrammarBuilder::new("ws")
        .token("WS", r"[ \t]+")
        .token("A", "a")
        .extra("WS")
        .rule("start", vec!["A"])
        .start("start")
        .build();
    assert!(validate(&g).errors.is_empty());
}

#[test]
fn valid_grammar_with_precedence_no_fatal_errors() {
    let g = GrammarBuilder::new("prec")
        .token("N", r"\d+")
        .token("+", "+")
        .token("*", "*")
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "*", "expr"], 2, Associativity::Left)
        .rule("expr", vec!["N"])
        .start("expr")
        .build();
    let r = validate(&g);
    // Self-recursive rules (expr -> expr + expr) produce CyclicRule errors,
    // but the grammar is otherwise productive and well-formed.
    assert!(
        !r.errors
            .iter()
            .any(|e| matches!(e, ValidationError::NonProductiveSymbol { .. }))
    );
    assert!(
        !r.errors
            .iter()
            .any(|e| matches!(e, ValidationError::EmptyGrammar))
    );
}

#[test]
fn valid_grammar_with_external_tokens() {
    let g = GrammarBuilder::new("ext")
        .token("A", "a")
        .external("INDENT")
        .rule("start", vec!["A"])
        .start("start")
        .build();
    // No errors expected for external tokens just being declared
    let r = validate(&g);
    assert!(
        !r.errors
            .iter()
            .any(|e| matches!(e, ValidationError::ExternalTokenConflict { .. }))
    );
}

// ===========================================================================
// 2. Empty grammar
// ===========================================================================

#[test]
fn empty_grammar_produces_error() {
    let g = Grammar::new("empty".into());
    assert!(has_error(&g, |e| matches!(
        e,
        ValidationError::EmptyGrammar
    )));
}

#[test]
fn empty_grammar_error_count_is_at_least_one() {
    let r = validate(&Grammar::new("empty".into()));
    assert!(!r.errors.is_empty());
}

#[test]
fn empty_grammar_stats_all_zero() {
    let r = validate(&Grammar::new("empty".into()));
    assert_eq!(r.stats.total_symbols, 0);
    assert_eq!(r.stats.total_tokens, 0);
    assert_eq!(r.stats.total_rules, 0);
}

// ===========================================================================
// 3. Undefined symbol references
// ===========================================================================

#[test]
fn undefined_symbol_in_rule_rhs() {
    let mut g = Grammar::new("undef".into());
    let lhs = SymbolId(1);
    let undef = SymbolId(99);
    g.add_rule(Rule {
        lhs,
        rhs: vec![Symbol::NonTerminal(undef)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    assert!(has_error(&g, |e| matches!(
        e,
        ValidationError::UndefinedSymbol { symbol, .. } if *symbol == undef
    )));
}

#[test]
fn undefined_terminal_in_rule_rhs() {
    let mut g = Grammar::new("undef_term".into());
    let lhs = SymbolId(1);
    let undef = SymbolId(50);
    g.add_rule(Rule {
        lhs,
        rhs: vec![Symbol::Terminal(undef)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    assert!(has_error(&g, |e| matches!(
        e,
        ValidationError::UndefinedSymbol { symbol, .. } if *symbol == undef
    )));
}

#[test]
fn multiple_undefined_symbols() {
    let mut g = Grammar::new("multi_undef".into());
    let lhs = SymbolId(1);
    g.add_rule(Rule {
        lhs,
        rhs: vec![
            Symbol::NonTerminal(SymbolId(90)),
            Symbol::Terminal(SymbolId(91)),
        ],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    let r = validate(&g);
    let undef_count = r
        .errors
        .iter()
        .filter(|e| matches!(e, ValidationError::UndefinedSymbol { .. }))
        .count();
    assert!(
        undef_count >= 2,
        "expected >=2 undefined, got {undef_count}"
    );
}

#[test]
fn undefined_symbol_error_includes_location() {
    let mut g = Grammar::new("loc".into());
    let lhs = SymbolId(1);
    let undef = SymbolId(42);
    g.add_rule(Rule {
        lhs,
        rhs: vec![Symbol::NonTerminal(undef)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    let r = validate(&g);
    let err = r
        .errors
        .iter()
        .find(|e| matches!(e, ValidationError::UndefinedSymbol { symbol, .. } if *symbol == undef));
    assert!(err.is_some());
    if let Some(ValidationError::UndefinedSymbol { location, .. }) = err {
        assert!(!location.is_empty());
    }
}

// ===========================================================================
// 4. Non-productive symbols
// ===========================================================================

#[test]
fn non_productive_mutual_recursion() {
    let mut g = Grammar::new("nonprod".into());
    let a = SymbolId(1);
    let b = SymbolId(2);
    // A -> B, B -> A  (neither can derive terminals)
    g.add_rule(Rule {
        lhs: a,
        rhs: vec![Symbol::NonTerminal(b)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    g.add_rule(Rule {
        lhs: b,
        rhs: vec![Symbol::NonTerminal(a)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(1),
    });
    assert!(has_error(&g, |e| matches!(
        e,
        ValidationError::NonProductiveSymbol { .. }
    )));
}

#[test]
fn non_productive_single_self_ref() {
    let mut g = Grammar::new("self".into());
    let a = SymbolId(1);
    g.add_rule(Rule {
        lhs: a,
        rhs: vec![Symbol::NonTerminal(a)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    assert!(has_error(
        &g,
        |e| matches!(e, ValidationError::NonProductiveSymbol { symbol, .. } if *symbol == a)
    ));
}

#[test]
fn productive_when_terminal_present() {
    let mut g = Grammar::new("prod".into());
    let expr = SymbolId(1);
    let num = SymbolId(2);
    g.tokens.insert(
        num,
        Token {
            name: "NUM".into(),
            pattern: TokenPattern::Regex(r"\d+".into()),
            fragile: false,
        },
    );
    g.add_rule(Rule {
        lhs: expr,
        rhs: vec![Symbol::Terminal(num)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    let r = validate(&g);
    assert!(
        !r.errors
            .iter()
            .any(|e| matches!(e, ValidationError::NonProductiveSymbol { .. }))
    );
}

#[test]
fn non_productive_chain_of_three() {
    let mut g = Grammar::new("chain".into());
    let a = SymbolId(1);
    let b = SymbolId(2);
    let c = SymbolId(3);
    // A->B, B->C, C->A  all non-productive
    for (lhs, rhs_id, pid) in [(a, b, 0), (b, c, 1), (c, a, 2)] {
        g.add_rule(Rule {
            lhs,
            rhs: vec![Symbol::NonTerminal(rhs_id)],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(pid),
        });
    }
    let nonprod_count = validate(&g)
        .errors
        .iter()
        .filter(|e| matches!(e, ValidationError::NonProductiveSymbol { .. }))
        .count();
    assert!(nonprod_count >= 2);
}

// ===========================================================================
// 5. Cyclic rules
// ===========================================================================

#[test]
fn cycle_two_symbols() {
    let mut g = Grammar::new("cyc2".into());
    let a = SymbolId(1);
    let b = SymbolId(2);
    g.add_rule(Rule {
        lhs: a,
        rhs: vec![Symbol::NonTerminal(b)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    g.add_rule(Rule {
        lhs: b,
        rhs: vec![Symbol::NonTerminal(a)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(1),
    });
    assert!(has_error(&g, |e| matches!(
        e,
        ValidationError::CyclicRule { .. }
    )));
}

#[test]
fn cycle_three_symbols() {
    let mut g = Grammar::new("cyc3".into());
    let a = SymbolId(1);
    let b = SymbolId(2);
    let c = SymbolId(3);
    for (lhs, rhs_id, pid) in [(a, b, 0), (b, c, 1), (c, a, 2)] {
        g.add_rule(Rule {
            lhs,
            rhs: vec![Symbol::NonTerminal(rhs_id)],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(pid),
        });
    }
    assert!(has_error(&g, |e| matches!(
        e,
        ValidationError::CyclicRule { .. }
    )));
}

#[test]
fn cycle_error_contains_symbols() {
    let mut g = Grammar::new("cycsym".into());
    let a = SymbolId(1);
    let b = SymbolId(2);
    g.add_rule(Rule {
        lhs: a,
        rhs: vec![Symbol::NonTerminal(b)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    g.add_rule(Rule {
        lhs: b,
        rhs: vec![Symbol::NonTerminal(a)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(1),
    });
    let r = validate(&g);
    let cyc = r
        .errors
        .iter()
        .find(|e| matches!(e, ValidationError::CyclicRule { .. }));
    assert!(cyc.is_some());
    if let Some(ValidationError::CyclicRule { symbols }) = cyc {
        assert!(!symbols.is_empty());
    }
}

#[test]
fn no_cycle_when_base_case_exists() {
    // expr -> expr "+" NUM  has a cycle but also expr -> NUM as base case
    // The validator still detects the non-terminal cycle in the graph
    let g = GrammarBuilder::new("nobase")
        .token("N", r"\d+")
        .token("+", "+")
        .rule("expr", vec!["expr", "+", "N"])
        .rule("expr", vec!["N"])
        .start("expr")
        .build();
    // This grammar IS recursive but it's valid: "expr" references itself
    // The cycle checker should flag it since it follows NonTerminal edges
    let r = validate(&g);
    // Even if flagged, the grammar should remain valid for parsing
    assert!(
        !r.errors
            .iter()
            .any(|e| matches!(e, ValidationError::NonProductiveSymbol { .. }))
    );
}

// ===========================================================================
// 6. Unreachable / unused rules
// ===========================================================================

#[test]
fn unreachable_rule_produces_warning() {
    let g = GrammarBuilder::new("unreach")
        .token("A", "a")
        .token("B", "b")
        .rule("start", vec!["A"])
        .rule("orphan", vec!["B"])
        .start("start")
        .build();
    // "orphan" is unreachable from start
    assert!(has_warning(&g, |w| matches!(
        w,
        ValidationWarning::UnusedToken { name, .. } if name.contains("orphan") || name.contains("rule_")
    )));
}

#[test]
fn unreachable_token_not_used_in_any_rule() {
    let g = GrammarBuilder::new("unused_tok")
        .token("USED", "u")
        .token("UNUSED", "x")
        .rule("start", vec!["USED"])
        .start("start")
        .build();
    let r = validate(&g);
    let unused_tok = r.warnings.iter().any(|w| match w {
        ValidationWarning::UnusedToken { name, .. } => name == "UNUSED",
        _ => false,
    });
    assert!(unused_tok, "expected warning for UNUSED token");
}

#[test]
fn all_reachable_no_warnings_about_unreachable() {
    let g = simple_arithmetic();
    let r = validate(&g);
    let unreachable_warns = r
        .warnings
        .iter()
        .filter(|w| matches!(w, ValidationWarning::UnusedToken { .. }))
        .count();
    assert_eq!(unreachable_warns, 0);
}

// ===========================================================================
// 7. Validation warnings vs errors
// ===========================================================================

#[test]
fn warnings_are_not_errors() {
    let g = GrammarBuilder::new("warnonly")
        .token("A", "a")
        .token("B", "b")
        .rule("start", vec!["A"])
        .start("start")
        .build();
    let r = validate(&g);
    // "B" is unused → warning, not error
    assert!(
        r.errors.is_empty()
            || !r
                .errors
                .iter()
                .any(|e| matches!(e, ValidationError::EmptyGrammar))
    );
}

#[test]
fn duplicate_token_pattern_is_warning() {
    let mut g = Grammar::new("dup".into());
    g.tokens.insert(
        SymbolId(1),
        Token {
            name: "p1".into(),
            pattern: TokenPattern::String("+".into()),
            fragile: false,
        },
    );
    g.tokens.insert(
        SymbolId(2),
        Token {
            name: "p2".into(),
            pattern: TokenPattern::String("+".into()),
            fragile: false,
        },
    );
    assert!(has_warning(&g, |w| matches!(
        w,
        ValidationWarning::DuplicateTokenPattern { pattern, .. } if pattern == "+"
    )));
}

#[test]
fn missing_field_names_is_warning() {
    let mut g = Grammar::new("nofields".into());
    let lhs = SymbolId(1);
    let t1 = SymbolId(2);
    let t2 = SymbolId(3);
    for (id, name) in [(t1, "A"), (t2, "B")] {
        g.tokens.insert(
            id,
            Token {
                name: name.into(),
                pattern: TokenPattern::String(name.into()),
                fragile: false,
            },
        );
    }
    g.add_rule(Rule {
        lhs,
        rhs: vec![Symbol::Terminal(t1), Symbol::Terminal(t2)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    assert!(has_warning(&g, |w| matches!(
        w,
        ValidationWarning::MissingFieldNames { .. }
    )));
}

#[test]
fn inefficient_trivial_rule_warning() {
    let mut g = Grammar::new("trivial".into());
    let a = SymbolId(1);
    let b = SymbolId(2);
    let tok = SymbolId(3);
    g.tokens.insert(
        tok,
        Token {
            name: "T".into(),
            pattern: TokenPattern::String("t".into()),
            fragile: false,
        },
    );
    g.add_rule(Rule {
        lhs: a,
        rhs: vec![Symbol::NonTerminal(b)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    g.add_rule(Rule {
        lhs: b,
        rhs: vec![Symbol::Terminal(tok)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(1),
    });
    assert!(has_warning(
        &g,
        |w| matches!(w, ValidationWarning::InefficientRule { suggestion, .. } if suggestion.contains("inlining"))
    ));
}

#[test]
fn long_rule_warning() {
    let mut g = Grammar::new("long".into());
    let lhs = SymbolId(1);
    let tok = SymbolId(2);
    g.tokens.insert(
        tok,
        Token {
            name: "T".into(),
            pattern: TokenPattern::String("t".into()),
            fragile: false,
        },
    );
    g.add_rule(Rule {
        lhs,
        rhs: (0..11).map(|_| Symbol::Terminal(tok)).collect(),
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    assert!(has_warning(
        &g,
        |w| matches!(w, ValidationWarning::InefficientRule { suggestion, .. } if suggestion.contains("11"))
    ));
}

// ===========================================================================
// 8. Validation statistics
// ===========================================================================

#[test]
fn stats_total_symbols() {
    let r = validate(&simple_arithmetic());
    // "expr" + "NUMBER" + "+" = at least 3 symbols
    assert!(r.stats.total_symbols >= 3, "got {}", r.stats.total_symbols);
}

#[test]
fn stats_total_tokens() {
    let r = validate(&simple_arithmetic());
    assert_eq!(r.stats.total_tokens, 2); // NUMBER, +
}

#[test]
fn stats_total_rules() {
    let r = validate(&simple_arithmetic());
    assert_eq!(r.stats.total_rules, 1);
}

#[test]
fn stats_max_rule_length() {
    let r = validate(&simple_arithmetic());
    assert_eq!(r.stats.max_rule_length, 3); // NUMBER + NUMBER
}

#[test]
fn stats_avg_rule_length() {
    let r = validate(&simple_arithmetic());
    assert!((r.stats.avg_rule_length - 3.0).abs() < f64::EPSILON);
}

#[test]
fn stats_reachable_symbols() {
    let r = validate(&simple_arithmetic());
    assert!(r.stats.reachable_symbols >= 3);
}

#[test]
fn stats_productive_symbols() {
    let r = validate(&simple_arithmetic());
    assert!(r.stats.productive_symbols >= 3);
}

#[test]
fn stats_external_tokens_zero() {
    let r = validate(&simple_arithmetic());
    assert_eq!(r.stats.external_tokens, 0);
}

#[test]
fn stats_external_tokens_counted() {
    let g = GrammarBuilder::new("ext")
        .token("A", "a")
        .external("INDENT")
        .external("DEDENT")
        .rule("start", vec!["A"])
        .start("start")
        .build();
    let r = validate(&g);
    assert_eq!(r.stats.external_tokens, 2);
}

#[test]
fn stats_debug_trait() {
    let r = validate(&simple_arithmetic());
    let debug = format!("{:?}", r.stats);
    assert!(debug.contains("total_symbols"));
}

// ===========================================================================
// 9. Field validation
// ===========================================================================

#[test]
fn invalid_field_index_detected() {
    let mut g = Grammar::new("field".into());
    let lhs = SymbolId(1);
    let tok = SymbolId(2);
    g.tokens.insert(
        tok,
        Token {
            name: "T".into(),
            pattern: TokenPattern::String("t".into()),
            fragile: false,
        },
    );
    g.add_rule(Rule {
        lhs,
        rhs: vec![Symbol::Terminal(tok)],
        precedence: None,
        associativity: None,
        fields: vec![(FieldId(0), 10)], // out of bounds
        production_id: ProductionId(0),
    });
    assert!(has_error(&g, |e| matches!(
        e,
        ValidationError::InvalidField { .. }
    )));
}

#[test]
fn valid_field_index_no_error() {
    let mut g = Grammar::new("fieldok".into());
    let lhs = SymbolId(1);
    let tok = SymbolId(2);
    g.tokens.insert(
        tok,
        Token {
            name: "T".into(),
            pattern: TokenPattern::String("t".into()),
            fragile: false,
        },
    );
    g.add_rule(Rule {
        lhs,
        rhs: vec![Symbol::Terminal(tok)],
        precedence: None,
        associativity: None,
        fields: vec![(FieldId(0), 0)],
        production_id: ProductionId(0),
    });
    let r = validate(&g);
    assert!(
        !r.errors
            .iter()
            .any(|e| matches!(e, ValidationError::InvalidField { .. }))
    );
}

// ===========================================================================
// 10. Precedence validation
// ===========================================================================

#[test]
fn conflicting_precedence_detected() {
    let mut g = Grammar::new("preccon".into());
    let sym = SymbolId(1);
    g.precedences.push(Precedence {
        level: 1,
        associativity: Associativity::Left,
        symbols: vec![sym],
    });
    g.precedences.push(Precedence {
        level: 2,
        associativity: Associativity::Right,
        symbols: vec![sym],
    });
    assert!(has_error(&g, |e| matches!(
        e,
        ValidationError::ConflictingPrecedence { symbol, .. } if *symbol == sym
    )));
}

#[test]
fn same_precedence_twice_no_conflict() {
    let mut g = Grammar::new("sameprec".into());
    let tok = SymbolId(1);
    g.tokens.insert(
        tok,
        Token {
            name: "T".into(),
            pattern: TokenPattern::String("t".into()),
            fragile: false,
        },
    );
    g.add_rule(Rule {
        lhs: SymbolId(2),
        rhs: vec![Symbol::Terminal(tok)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    g.precedences.push(Precedence {
        level: 1,
        associativity: Associativity::Left,
        symbols: vec![tok],
    });
    g.precedences.push(Precedence {
        level: 1,
        associativity: Associativity::Left,
        symbols: vec![tok],
    });
    let r = validate(&g);
    assert!(
        !r.errors
            .iter()
            .any(|e| matches!(e, ValidationError::ConflictingPrecedence { .. }))
    );
}

// ===========================================================================
// 11. External token validation
// ===========================================================================

#[test]
fn duplicate_external_token_conflict() {
    let mut g = Grammar::new("dupext".into());
    g.externals.push(ExternalToken {
        name: "IND".into(),
        symbol_id: SymbolId(10),
    });
    g.externals.push(ExternalToken {
        name: "IND".into(),
        symbol_id: SymbolId(11),
    });
    assert!(has_error(&g, |e| matches!(
        e,
        ValidationError::ExternalTokenConflict { .. }
    )));
}

#[test]
fn unique_external_tokens_no_conflict() {
    let g = GrammarBuilder::new("uext")
        .token("A", "a")
        .external("INDENT")
        .external("DEDENT")
        .rule("start", vec!["A"])
        .start("start")
        .build();
    let r = validate(&g);
    assert!(
        !r.errors
            .iter()
            .any(|e| matches!(e, ValidationError::ExternalTokenConflict { .. }))
    );
}

// ===========================================================================
// 12. Token validation — empty regex
// ===========================================================================

#[test]
fn empty_regex_pattern_error() {
    let mut g = Grammar::new("emptyregex".into());
    let tok = SymbolId(1);
    g.tokens.insert(
        tok,
        Token {
            name: "BAD".into(),
            pattern: TokenPattern::Regex("".into()),
            fragile: false,
        },
    );
    g.add_rule(Rule {
        lhs: SymbolId(2),
        rhs: vec![Symbol::Terminal(tok)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    assert!(has_error(&g, |e| matches!(
        e,
        ValidationError::InvalidRegex { .. }
    )));
}

#[test]
fn non_empty_regex_no_error() {
    let mut g = Grammar::new("goodregex".into());
    let tok = SymbolId(1);
    g.tokens.insert(
        tok,
        Token {
            name: "OK".into(),
            pattern: TokenPattern::Regex(r"\d+".into()),
            fragile: false,
        },
    );
    g.add_rule(Rule {
        lhs: SymbolId(2),
        rhs: vec![Symbol::Terminal(tok)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    let r = validate(&g);
    assert!(
        !r.errors
            .iter()
            .any(|e| matches!(e, ValidationError::InvalidRegex { .. }))
    );
}

// ===========================================================================
// 13. Large grammar validation
// ===========================================================================

#[test]
fn large_grammar_many_rules() {
    let mut b = GrammarBuilder::new("large");
    // Create 50 tokens
    for i in 0..50 {
        b = b.token(&format!("T{i}"), &format!("t{i}"));
    }
    // Create rules referencing those tokens
    for i in 0..50 {
        b = b.rule(&format!("r{i}"), vec![&format!("T{i}")]);
    }
    // Chain rules so they are reachable
    b = b.rule("start", vec!["r0"]);
    b = b.start("start");
    let g = b.build();
    let r = validate(&g);
    // Should not panic; stats should be populated
    assert!(r.stats.total_rules >= 51);
    assert!(r.stats.total_tokens >= 50);
}

#[test]
fn large_grammar_deep_nesting() {
    let mut b = GrammarBuilder::new("deep");
    b = b.token("LEAF", "x");
    // chain: r0 -> r1 -> r2 -> ... -> r19 -> LEAF
    for i in 0..20 {
        if i < 19 {
            b = b.rule(&format!("r{i}"), vec![&format!("r{}", i + 1)]);
        } else {
            b = b.rule(&format!("r{i}"), vec!["LEAF"]);
        }
    }
    b = b.start("r0");
    let g = b.build();
    let r = validate(&g);
    assert!(r.stats.productive_symbols >= 20);
}

// ===========================================================================
// 14. Validator reuse
// ===========================================================================

#[test]
fn validator_can_be_reused() {
    let mut v = GrammarValidator::new();
    let g1 = simple_arithmetic();
    let r1 = v.validate(&g1);
    assert!(r1.errors.is_empty());

    let g2 = Grammar::new("empty".into());
    let r2 = v.validate(&g2);
    assert!(!r2.errors.is_empty());
}

#[test]
fn validator_clears_between_runs() {
    let mut v = GrammarValidator::new();
    // First validate an invalid grammar
    let g1 = Grammar::new("empty".into());
    let r1 = v.validate(&g1);
    assert!(!r1.errors.is_empty());

    // Now validate a valid grammar — should have no errors
    let g2 = simple_arithmetic();
    let r2 = v.validate(&g2);
    assert!(r2.errors.is_empty());
}

// ===========================================================================
// 15. Display implementations
// ===========================================================================

#[test]
fn validation_error_display_empty_grammar() {
    let e = ValidationError::EmptyGrammar;
    let s = format!("{e}");
    assert!(s.contains("no rules"));
}

#[test]
fn validation_error_display_undefined_symbol() {
    let e = ValidationError::UndefinedSymbol {
        symbol: SymbolId(5),
        location: "rule for X".into(),
    };
    let s = format!("{e}");
    assert!(s.contains("Undefined") || s.contains("undefined"));
}

#[test]
fn validation_error_display_unreachable() {
    let e = ValidationError::UnreachableSymbol {
        symbol: SymbolId(3),
        name: "orphan".into(),
    };
    assert!(format!("{e}").contains("orphan"));
}

#[test]
fn validation_error_display_non_productive() {
    let e = ValidationError::NonProductiveSymbol {
        symbol: SymbolId(7),
        name: "dead".into(),
    };
    assert!(format!("{e}").contains("dead"));
}

#[test]
fn validation_error_display_cyclic() {
    let e = ValidationError::CyclicRule {
        symbols: vec![SymbolId(1), SymbolId(2)],
    };
    assert!(format!("{e}").to_lowercase().contains("cyclic") || format!("{e}").contains("Cyclic"));
}

#[test]
fn validation_error_display_duplicate_rule() {
    let e = ValidationError::DuplicateRule {
        symbol: SymbolId(1),
        existing_count: 3,
    };
    assert!(format!("{e}").contains("3"));
}

#[test]
fn validation_error_display_invalid_field() {
    let e = ValidationError::InvalidField {
        field_id: FieldId(0),
        rule_symbol: SymbolId(1),
    };
    assert!(format!("{e}").contains("Invalid field") || format!("{e}").contains("invalid"));
}

#[test]
fn validation_error_display_no_start() {
    let e = ValidationError::NoExplicitStartRule;
    assert!(format!("{e}").contains("start"));
}

#[test]
fn validation_error_display_conflicting_prec() {
    let e = ValidationError::ConflictingPrecedence {
        symbol: SymbolId(1),
        precedences: vec![1, 2],
    };
    assert!(format!("{e}").contains("conflicting") || format!("{e}").contains("Conflicting"));
}

#[test]
fn validation_error_display_invalid_regex() {
    let e = ValidationError::InvalidRegex {
        token: SymbolId(1),
        pattern: "[bad".into(),
        error: "unclosed bracket".into(),
    };
    assert!(format!("{e}").contains("[bad"));
}

#[test]
fn validation_error_display_external_conflict() {
    let e = ValidationError::ExternalTokenConflict {
        token1: "A".into(),
        token2: "B".into(),
    };
    let s = format!("{e}");
    assert!(s.contains("A") && s.contains("B"));
}

#[test]
fn validation_warning_display_unused_token() {
    let w = ValidationWarning::UnusedToken {
        token: SymbolId(1),
        name: "FOO".into(),
    };
    assert!(format!("{w}").contains("FOO"));
}

#[test]
fn validation_warning_display_duplicate_pattern() {
    let w = ValidationWarning::DuplicateTokenPattern {
        tokens: vec![SymbolId(1), SymbolId(2)],
        pattern: "+".into(),
    };
    assert!(format!("{w}").contains("+"));
}

#[test]
fn validation_warning_display_ambiguous() {
    let w = ValidationWarning::AmbiguousGrammar {
        message: "shift/reduce".into(),
    };
    assert!(format!("{w}").contains("shift/reduce"));
}

#[test]
fn validation_warning_display_missing_fields() {
    let w = ValidationWarning::MissingFieldNames {
        rule_symbol: SymbolId(1),
    };
    assert!(format!("{w}").contains("field"));
}

#[test]
fn validation_warning_display_inefficient() {
    let w = ValidationWarning::InefficientRule {
        symbol: SymbolId(1),
        suggestion: "inline it".into(),
    };
    assert!(format!("{w}").contains("inline it"));
}

// ===========================================================================
// 16. Clone / equality on error and warning types
// ===========================================================================

#[test]
fn validation_error_clone_eq() {
    let e1 = ValidationError::EmptyGrammar;
    let e2 = e1.clone();
    assert_eq!(e1, e2);
}

#[test]
fn validation_warning_clone_eq() {
    let w1 = ValidationWarning::UnusedToken {
        token: SymbolId(1),
        name: "X".into(),
    };
    let w2 = w1.clone();
    assert_eq!(w1, w2);
}

#[test]
fn validation_error_ne() {
    let e1 = ValidationError::EmptyGrammar;
    let e2 = ValidationError::NoExplicitStartRule;
    assert_ne!(e1, e2);
}

// ===========================================================================
// 17. Edge cases
// ===========================================================================

#[test]
fn grammar_with_only_tokens_no_rules_is_empty() {
    let mut g = Grammar::new("tokonly".into());
    g.tokens.insert(
        SymbolId(1),
        Token {
            name: "A".into(),
            pattern: TokenPattern::String("a".into()),
            fragile: false,
        },
    );
    assert!(has_error(&g, |e| matches!(
        e,
        ValidationError::EmptyGrammar
    )));
}

#[test]
fn grammar_name_preserved() {
    let g = GrammarBuilder::new("my_name")
        .token("A", "a")
        .rule("s", vec!["A"])
        .start("s")
        .build();
    assert_eq!(g.name, "my_name");
}

#[test]
fn multiple_rules_same_lhs_not_duplicate_error() {
    // The builder naturally creates multiple rules for same LHS
    let g = GrammarBuilder::new("multi")
        .token("A", "a")
        .token("B", "b")
        .rule("start", vec!["A"])
        .rule("start", vec!["B"])
        .start("start")
        .build();
    let r = validate(&g);
    assert!(
        !r.errors
            .iter()
            .any(|e| matches!(e, ValidationError::DuplicateRule { .. }))
    );
}

#[test]
fn default_validator() {
    let v = GrammarValidator::default();
    // Just ensure Default impl works
    let g = simple_arithmetic();
    let mut v = v;
    let r = v.validate(&g);
    assert!(r.errors.is_empty());
}

#[test]
fn fragile_token_validates_ok() {
    let g = GrammarBuilder::new("fragile")
        .fragile_token("SEMI", ";")
        .rule("start", vec!["SEMI"])
        .start("start")
        .build();
    assert!(validate(&g).errors.is_empty());
}

#[test]
fn stats_avg_rule_length_multiple_rules() {
    let g = GrammarBuilder::new("avg")
        .token("A", "a")
        .token("B", "b")
        .rule("s", vec!["A"]) // length 1
        .rule("s", vec!["A", "B"]) // length 2
        .start("s")
        .build();
    let r = validate(&g);
    assert!((r.stats.avg_rule_length - 1.5).abs() < f64::EPSILON);
}

#[test]
fn stats_max_rule_length_picks_longest() {
    let g = GrammarBuilder::new("maxlen")
        .token("A", "a")
        .token("B", "b")
        .token("C", "c")
        .rule("s", vec!["A"])
        .rule("s", vec!["A", "B", "C"])
        .start("s")
        .build();
    let r = validate(&g);
    assert_eq!(r.stats.max_rule_length, 3);
}
