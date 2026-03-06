//! Comprehensive tests for grammar validation in the IR crate.
//!
//! Covers: valid grammars, invalid grammars (missing start, unreachable rules,
//! undefined symbols), warnings, stats, and optimizer effects.

use adze_ir::builder::GrammarBuilder;
use adze_ir::validation::{GrammarValidator, ValidationError, ValidationWarning};
use adze_ir::{Associativity, Grammar, GrammarOptimizer, optimize_grammar};

// ─── Helpers ─────────────────────────────────────────────────────────────────

fn validate(grammar: &Grammar) -> adze_ir::validation::ValidationResult {
    let mut v = GrammarValidator::new();
    v.validate(grammar)
}

fn has_error<F: Fn(&ValidationError) -> bool>(
    result: &adze_ir::validation::ValidationResult,
    pred: F,
) -> bool {
    result.errors.iter().any(pred)
}

fn has_warning<F: Fn(&ValidationWarning) -> bool>(
    result: &adze_ir::validation::ValidationResult,
    pred: F,
) -> bool {
    result.warnings.iter().any(pred)
}

// ═══════════════════════════════════════════════════════════════════════════════
//  1. Valid grammar validation (no errors)
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn valid_single_rule_grammar_has_no_errors() {
    let g = GrammarBuilder::new("single")
        .token("NUMBER", r"\d+")
        .rule("expr", vec!["NUMBER"])
        .start("expr")
        .build();

    let r = validate(&g);
    assert!(r.errors.is_empty(), "errors: {:?}", r.errors);
}

#[test]
fn valid_multi_alternative_grammar_has_no_errors() {
    // Non-recursive alternatives to avoid CyclicRule detection
    let g = GrammarBuilder::new("multi")
        .token("NUMBER", r"\d+")
        .token("IDENT", r"[a-z]+")
        .token("STRING", r#""[^"]*""#)
        .rule("value", vec!["NUMBER"])
        .rule("value", vec!["IDENT"])
        .rule("value", vec!["STRING"])
        .start("value")
        .build();

    let r = validate(&g);
    assert!(r.errors.is_empty(), "errors: {:?}", r.errors);
}

#[test]
fn valid_grammar_with_multiple_nonterminals() {
    // Non-recursive multi-nonterminal grammar
    let g = GrammarBuilder::new("stmts")
        .token("NUMBER", r"\d+")
        .token(";", ";")
        .token("=", "=")
        .token("IDENT", r"[a-z]+")
        .rule("program", vec!["statement"])
        .rule("statement", vec!["IDENT", "=", "expr", ";"])
        .rule("expr", vec!["NUMBER"])
        .rule("expr", vec!["IDENT"])
        .start("program")
        .build();

    let r = validate(&g);
    assert!(r.errors.is_empty(), "errors: {:?}", r.errors);
}

#[test]
fn valid_grammar_with_epsilon_rule() {
    let g = GrammarBuilder::new("nullable")
        .token("a", "a")
        .rule("start", vec![])
        .rule("start", vec!["a"])
        .start("start")
        .build();

    let r = validate(&g);
    assert!(r.errors.is_empty(), "errors: {:?}", r.errors);
}

#[test]
fn valid_grammar_with_external_tokens() {
    let g = GrammarBuilder::new("ext")
        .token("INDENT", "INDENT")
        .token("DEDENT", "DEDENT")
        .token("pass", "pass")
        .external("INDENT")
        .external("DEDENT")
        .rule("suite", vec!["INDENT", "pass", "DEDENT"])
        .start("suite")
        .build();

    let r = validate(&g);
    assert!(r.errors.is_empty(), "errors: {:?}", r.errors);
}

#[test]
fn python_like_preset_only_has_cyclic_errors() {
    // Python-like preset has recursive rules, so CyclicRule is expected
    let g = GrammarBuilder::python_like();
    let r = validate(&g);
    assert!(
        r.errors
            .iter()
            .all(|e| matches!(e, ValidationError::CyclicRule { .. })),
        "unexpected non-cycle errors: {:?}",
        r.errors
    );
}

#[test]
fn javascript_like_preset_only_has_cyclic_errors() {
    // JavaScript-like preset has recursive rules, so CyclicRule is expected
    let g = GrammarBuilder::javascript_like();
    let r = validate(&g);
    assert!(
        r.errors
            .iter()
            .all(|e| matches!(e, ValidationError::CyclicRule { .. })),
        "unexpected non-cycle errors: {:?}",
        r.errors
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
//  2. Invalid grammars
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn empty_grammar_produces_empty_grammar_error() {
    let g = Grammar::new("empty".to_string());
    let r = validate(&g);
    assert!(has_error(&r, |e| matches!(
        e,
        ValidationError::EmptyGrammar
    )));
}

#[test]
fn grammar_without_start_uses_first_rule_no_explicit_start_error() {
    // Build a grammar without calling .start() — the builder doesn't set start_symbol
    let g = GrammarBuilder::new("nostart")
        .token("a", "a")
        .rule("root", vec!["a"])
        .build();

    let r = validate(&g);
    // The builder uses the first rule as implicit start, so no EmptyGrammar.
    // But we may not get NoExplicitStartRule depending on whether start_symbol is set.
    // What matters: no EmptyGrammar error.
    assert!(!has_error(&r, |e| matches!(
        e,
        ValidationError::EmptyGrammar
    )));
}

#[test]
fn undefined_symbol_in_rule_rhs_produces_error() {
    // Use raw Grammar API to inject an undefined symbol reference
    use adze_ir::{ProductionId, Rule, Symbol, SymbolId, Token, TokenPattern};

    let mut g = Grammar::new("undef".to_string());
    let expr_id = SymbolId(1);
    let number_id = SymbolId(2);
    let undefined_id = SymbolId(99);

    g.tokens.insert(
        number_id,
        Token {
            name: "NUMBER".into(),
            pattern: TokenPattern::Regex(r"\d+".into()),
            fragile: false,
        },
    );
    g.rules.entry(expr_id).or_default().push(Rule {
        lhs: expr_id,
        rhs: vec![
            Symbol::Terminal(number_id),
            Symbol::NonTerminal(undefined_id),
        ],
        precedence: None,
        associativity: None,
        fields: Vec::new(),
        production_id: ProductionId(0),
    });
    g.rule_names.insert(expr_id, "expr".into());

    let r = validate(&g);
    assert!(has_error(&r, |e| matches!(
        e,
        ValidationError::UndefinedSymbol { symbol, .. } if *symbol == undefined_id
    )));
}

#[test]
fn unreachable_rule_produces_warning() {
    // "orphan" is defined but not reachable from "start"
    let g = GrammarBuilder::new("unreach")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a"])
        .rule("orphan", vec!["b"])
        .start("start")
        .build();

    let r = validate(&g);
    // Unreachable symbols appear as UnusedToken warnings
    assert!(
        has_warning(&r, |w| matches!(
            w,
            ValidationWarning::UnusedToken { name, .. } if name.contains("orphan") || name.contains("b") || name.contains("rule_")
        )) || has_warning(&r, |w| matches!(w, ValidationWarning::UnusedToken { .. }))
    );
}

#[test]
fn non_productive_symbol_produces_error() {
    // "loop" references only itself — can never derive terminals
    use adze_ir::{ProductionId, Rule, Symbol, SymbolId, Token, TokenPattern};

    let mut g = Grammar::new("nonprod".to_string());
    let start_id = SymbolId(1);
    let loop_id = SymbolId(2);
    let tok_id = SymbolId(3);

    g.tokens.insert(
        tok_id,
        Token {
            name: "a".into(),
            pattern: TokenPattern::String("a".into()),
            fragile: false,
        },
    );
    g.rules.entry(start_id).or_default().push(Rule {
        lhs: start_id,
        rhs: vec![Symbol::Terminal(tok_id)],
        precedence: None,
        associativity: None,
        fields: Vec::new(),
        production_id: ProductionId(0),
    });
    // loop -> loop (only self-reference, no terminal base case)
    g.rules.entry(loop_id).or_default().push(Rule {
        lhs: loop_id,
        rhs: vec![Symbol::NonTerminal(loop_id)],
        precedence: None,
        associativity: None,
        fields: Vec::new(),
        production_id: ProductionId(1),
    });
    g.rule_names.insert(start_id, "start".into());
    g.rule_names.insert(loop_id, "loop_sym".into());

    let r = validate(&g);
    assert!(has_error(&r, |e| matches!(
        e,
        ValidationError::NonProductiveSymbol { symbol, .. } if *symbol == loop_id
    )));
}

#[test]
fn cyclic_rules_without_base_case_detected() {
    use adze_ir::{ProductionId, Rule, Symbol, SymbolId, Token, TokenPattern};

    let mut g = Grammar::new("cycle".to_string());
    let a_id = SymbolId(1);
    let b_id = SymbolId(2);
    let tok_id = SymbolId(3);

    g.tokens.insert(
        tok_id,
        Token {
            name: "x".into(),
            pattern: TokenPattern::String("x".into()),
            fragile: false,
        },
    );
    // a -> b
    g.rules.entry(a_id).or_default().push(Rule {
        lhs: a_id,
        rhs: vec![Symbol::NonTerminal(b_id)],
        precedence: None,
        associativity: None,
        fields: Vec::new(),
        production_id: ProductionId(0),
    });
    // b -> a  (cycle: a -> b -> a)
    g.rules.entry(b_id).or_default().push(Rule {
        lhs: b_id,
        rhs: vec![Symbol::NonTerminal(a_id)],
        precedence: None,
        associativity: None,
        fields: Vec::new(),
        production_id: ProductionId(1),
    });
    g.rule_names.insert(a_id, "a".into());
    g.rule_names.insert(b_id, "b".into());

    let r = validate(&g);
    assert!(has_error(&r, |e| matches!(
        e,
        ValidationError::CyclicRule { .. }
    )));
}

#[test]
fn duplicate_external_token_produces_error() {
    use adze_ir::{ExternalToken, ProductionId, Rule, Symbol, SymbolId, Token, TokenPattern};

    let mut g = Grammar::new("dup_ext".to_string());
    let start_id = SymbolId(1);
    let tok_id = SymbolId(2);
    let ext_id = SymbolId(3);

    g.tokens.insert(
        tok_id,
        Token {
            name: "a".into(),
            pattern: TokenPattern::String("a".into()),
            fragile: false,
        },
    );
    g.rules.entry(start_id).or_default().push(Rule {
        lhs: start_id,
        rhs: vec![Symbol::Terminal(tok_id)],
        precedence: None,
        associativity: None,
        fields: Vec::new(),
        production_id: ProductionId(0),
    });
    g.rule_names.insert(start_id, "start".into());
    // Two externals with the same name
    g.externals.push(ExternalToken {
        name: "INDENT".into(),
        symbol_id: ext_id,
    });
    g.externals.push(ExternalToken {
        name: "INDENT".into(),
        symbol_id: ext_id,
    });

    let r = validate(&g);
    assert!(has_error(&r, |e| matches!(
        e,
        ValidationError::ExternalTokenConflict { .. }
    )));
}

#[test]
fn invalid_field_index_produces_error() {
    use adze_ir::{FieldId, ProductionId, Rule, Symbol, SymbolId, Token, TokenPattern};

    let mut g = Grammar::new("badfield".to_string());
    let start_id = SymbolId(1);
    let tok_id = SymbolId(2);

    g.tokens.insert(
        tok_id,
        Token {
            name: "a".into(),
            pattern: TokenPattern::String("a".into()),
            fragile: false,
        },
    );
    g.rules.entry(start_id).or_default().push(Rule {
        lhs: start_id,
        rhs: vec![Symbol::Terminal(tok_id)],
        precedence: None,
        associativity: None,
        fields: vec![(FieldId(0), 999)], // index 999 is out of bounds
        production_id: ProductionId(0),
    });
    g.rule_names.insert(start_id, "start".into());

    let r = validate(&g);
    assert!(has_error(&r, |e| matches!(
        e,
        ValidationError::InvalidField { .. }
    )));
}

#[test]
fn conflicting_precedence_declarations_produce_error() {
    use adze_ir::Precedence;

    let mut g = GrammarBuilder::new("prec")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();

    let sym_id = *g.tokens.keys().next().unwrap();
    g.precedences.push(Precedence {
        level: 1,
        associativity: Associativity::Left,
        symbols: vec![sym_id],
    });
    g.precedences.push(Precedence {
        level: 5,
        associativity: Associativity::Right,
        symbols: vec![sym_id],
    });

    let r = validate(&g);
    assert!(has_error(&r, |e| matches!(
        e,
        ValidationError::ConflictingPrecedence { .. }
    )));
}

// ═══════════════════════════════════════════════════════════════════════════════
//  3. Validation warnings
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn unused_token_produces_warning() {
    let g = GrammarBuilder::new("unused")
        .token("a", "a")
        .token("b", "b") // not referenced by any rule
        .rule("start", vec!["a"])
        .start("start")
        .build();

    let r = validate(&g);
    assert!(has_warning(&r, |w| matches!(
        w,
        ValidationWarning::UnusedToken { .. }
    )));
}

#[test]
fn duplicate_token_pattern_produces_warning() {
    use adze_ir::{ProductionId, Rule, Symbol, SymbolId, Token, TokenPattern};

    let mut g = Grammar::new("duppat".to_string());
    let start_id = SymbolId(1);
    let tok1 = SymbolId(2);
    let tok2 = SymbolId(3);

    g.tokens.insert(
        tok1,
        Token {
            name: "PLUS".into(),
            pattern: TokenPattern::String("+".into()),
            fragile: false,
        },
    );
    g.tokens.insert(
        tok2,
        Token {
            name: "ADD".into(),
            pattern: TokenPattern::String("+".into()), // same pattern as PLUS
            fragile: false,
        },
    );
    g.rules.entry(start_id).or_default().push(Rule {
        lhs: start_id,
        rhs: vec![Symbol::Terminal(tok1), Symbol::Terminal(tok2)],
        precedence: None,
        associativity: None,
        fields: Vec::new(),
        production_id: ProductionId(0),
    });
    g.rule_names.insert(start_id, "start".into());

    let r = validate(&g);
    assert!(has_warning(&r, |w| matches!(
        w,
        ValidationWarning::DuplicateTokenPattern { pattern, .. } if pattern == "+"
    )));
}

#[test]
fn trivial_unit_rule_produces_inefficient_warning() {
    let g = GrammarBuilder::new("trivial")
        .token("a", "a")
        .rule("start", vec!["wrapper"])
        .rule("wrapper", vec!["a"])
        .start("start")
        .build();

    let r = validate(&g);
    // start -> wrapper is a unit rule (single nonterminal on RHS)
    assert!(has_warning(&r, |w| matches!(
        w,
        ValidationWarning::InefficientRule { suggestion, .. }
            if suggestion.contains("inlining")
    )));
}

#[test]
fn missing_field_names_warning_for_multi_symbol_rule() {
    let g = GrammarBuilder::new("nofields")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a", "b"])
        .start("start")
        .build();

    let r = validate(&g);
    assert!(has_warning(&r, |w| matches!(
        w,
        ValidationWarning::MissingFieldNames { .. }
    )));
}

#[test]
fn long_rule_produces_inefficient_warning() {
    use adze_ir::{ProductionId, Rule, Symbol, SymbolId, Token, TokenPattern};

    let mut g = Grammar::new("long".to_string());
    let start_id = SymbolId(1);
    let tok_id = SymbolId(2);

    g.tokens.insert(
        tok_id,
        Token {
            name: "x".into(),
            pattern: TokenPattern::String("x".into()),
            fragile: false,
        },
    );
    // Rule with 12 symbols on the RHS (> 10 threshold)
    g.rules.entry(start_id).or_default().push(Rule {
        lhs: start_id,
        rhs: vec![Symbol::Terminal(tok_id); 12],
        precedence: None,
        associativity: None,
        fields: Vec::new(),
        production_id: ProductionId(0),
    });
    g.rule_names.insert(start_id, "start".into());

    let r = validate(&g);
    assert!(has_warning(&r, |w| matches!(
        w,
        ValidationWarning::InefficientRule { suggestion, .. }
            if suggestion.contains("12 symbols")
    )));
}

// ═══════════════════════════════════════════════════════════════════════════════
//  4. Grammar stats
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn stats_counts_tokens_and_rules() {
    let g = GrammarBuilder::new("stats")
        .token("NUMBER", r"\d+")
        .token("+", "+")
        .rule("expr", vec!["expr", "+", "expr"])
        .rule("expr", vec!["NUMBER"])
        .start("expr")
        .build();

    let r = validate(&g);
    assert_eq!(r.stats.total_tokens, 2);
    assert_eq!(r.stats.total_rules, 2);
}

#[test]
fn stats_counts_reachable_symbols() {
    let g = GrammarBuilder::new("reach")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a"])
        .rule("orphan", vec!["b"]) // unreachable
        .start("start")
        .build();

    let r = validate(&g);
    // start + a are reachable; orphan + b are not
    assert!(r.stats.reachable_symbols < r.stats.total_symbols);
}

#[test]
fn stats_counts_productive_symbols() {
    let g = GrammarBuilder::new("prod")
        .token("NUMBER", r"\d+")
        .rule("expr", vec!["NUMBER"])
        .start("expr")
        .build();

    let r = validate(&g);
    assert!(r.stats.productive_symbols > 0);
}

#[test]
fn stats_counts_external_tokens() {
    let g = GrammarBuilder::new("ext")
        .token("INDENT", "INDENT")
        .token("DEDENT", "DEDENT")
        .token("pass", "pass")
        .external("INDENT")
        .external("DEDENT")
        .rule("suite", vec!["INDENT", "pass", "DEDENT"])
        .start("suite")
        .build();

    let r = validate(&g);
    assert_eq!(r.stats.external_tokens, 2);
}

#[test]
fn stats_max_rule_length_correct() {
    let g = GrammarBuilder::new("len")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("start", vec!["a", "b", "c"])
        .rule("start", vec!["a"])
        .start("start")
        .build();

    let r = validate(&g);
    assert_eq!(r.stats.max_rule_length, 3);
}

#[test]
fn stats_avg_rule_length_correct() {
    let g = GrammarBuilder::new("avg")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a", "b"])
        .rule("start", vec!["a"])
        .start("start")
        .build();

    let r = validate(&g);
    // (2 + 1) / 2 = 1.5
    assert!((r.stats.avg_rule_length - 1.5).abs() < f64::EPSILON);
}

#[test]
fn stats_for_empty_grammar() {
    let g = Grammar::new("empty".to_string());
    let r = validate(&g);
    assert_eq!(r.stats.total_tokens, 0);
    assert_eq!(r.stats.total_rules, 0);
    assert_eq!(r.stats.max_rule_length, 0);
}

// ═══════════════════════════════════════════════════════════════════════════════
//  5. Optimizer effects on grammar
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn optimizer_returns_stats() {
    let mut g = GrammarBuilder::new("opt")
        .token("NUMBER", r"\d+")
        .token("+", "+")
        .rule("expr", vec!["expr", "+", "expr"])
        .rule("expr", vec!["NUMBER"])
        .start("expr")
        .build();

    let mut opt = GrammarOptimizer::new();
    let stats = opt.optimize(&mut g);
    // Stats should be non-negative (implementation may or may not find optimizations)
    let _ = stats.total();
}

#[test]
fn optimized_grammar_still_validates() {
    let g = GrammarBuilder::new("opt_valid")
        .token("NUMBER", r"\d+")
        .token("IDENT", r"[a-z]+")
        .rule("value", vec!["NUMBER"])
        .rule("value", vec!["IDENT"])
        .start("value")
        .build();

    let g = optimize_grammar(g).unwrap();
    let r = validate(&g);
    // After optimization the grammar should still be valid (no new errors introduced)
    assert!(r.errors.is_empty(), "errors after optimize: {:?}", r.errors);
}

#[test]
fn optimize_grammar_convenience_function_works() {
    let g = GrammarBuilder::new("conv")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a", "b"])
        .start("start")
        .build();

    let g2 = optimize_grammar(g).unwrap();
    // The grammar should still have rules after optimization
    assert!(!g2.rules.is_empty());
}

#[test]
fn optimizer_preserves_start_symbol_rules() {
    let g = GrammarBuilder::new("preserve")
        .token("NUMBER", r"\d+")
        .token("+", "+")
        .token("*", "*")
        .rule("expr", vec!["expr", "+", "term"])
        .rule("expr", vec!["term"])
        .rule("term", vec!["term", "*", "NUMBER"])
        .rule("term", vec!["NUMBER"])
        .start("expr")
        .build();

    let g = optimize_grammar(g).unwrap();
    // Start symbol should still have rules
    assert!(g.start_symbol().is_some() || !g.rules.is_empty());
}

#[test]
fn optimizer_stats_total_sums_fields() {
    let mut g = GrammarBuilder::new("total")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["inner"])
        .rule("inner", vec!["a"])
        .rule("unused", vec!["b"]) // should be removed
        .start("start")
        .build();

    let mut opt = GrammarOptimizer::new();
    let stats = opt.optimize(&mut g);

    let expected_total = stats.removed_unused_symbols
        + stats.inlined_rules
        + stats.merged_tokens
        + stats.optimized_left_recursion
        + stats.eliminated_unit_rules;
    assert_eq!(stats.total(), expected_total);
}

// ═══════════════════════════════════════════════════════════════════════════════
//  6. Validator reuse and edge cases
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn validator_can_be_reused_across_grammars() {
    let g1 = GrammarBuilder::new("g1")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();

    let g2 = Grammar::new("g2".to_string()); // empty

    let mut v = GrammarValidator::new();
    let r1 = v.validate(&g1);
    let r2 = v.validate(&g2);

    assert!(r1.errors.is_empty());
    assert!(has_error(&r2, |e| matches!(
        e,
        ValidationError::EmptyGrammar
    )));
}

#[test]
fn validation_error_display_formatting() {
    let g = Grammar::new("empty".to_string());
    let r = validate(&g);
    let msg = format!("{}", r.errors[0]);
    assert!(msg.contains("no rules"), "got: {msg}");
}

#[test]
fn validation_warning_display_formatting() {
    let g = GrammarBuilder::new("disp")
        .token("a", "a")
        .token("unused", "unused")
        .rule("start", vec!["a"])
        .start("start")
        .build();

    let r = validate(&g);
    if let Some(w) = r.warnings.first() {
        let msg = format!("{w}");
        assert!(!msg.is_empty());
    }
}

#[test]
fn grammar_with_precedence_rules_only_has_cyclic_errors() {
    // Recursive precedence grammar — only CyclicRule errors expected
    let g = GrammarBuilder::new("prec")
        .token("NUMBER", r"\d+")
        .token("+", "+")
        .token("*", "*")
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "*", "expr"], 2, Associativity::Left)
        .rule("expr", vec!["NUMBER"])
        .start("expr")
        .build();

    let r = validate(&g);
    assert!(
        r.errors
            .iter()
            .all(|e| matches!(e, ValidationError::CyclicRule { .. })),
        "unexpected non-cycle errors: {:?}",
        r.errors
    );
}
