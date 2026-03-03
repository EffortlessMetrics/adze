#![allow(clippy::needless_range_loop)]

//! Comprehensive edge-case tests for grammar validation in adze-ir.

use adze_ir::validation::{GrammarValidator, ValidationError, ValidationWarning};
use adze_ir::*;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn make_rule(lhs: u16, rhs: Vec<Symbol>, prod: u16) -> Rule {
    Rule {
        lhs: SymbolId(lhs),
        rhs,
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(prod),
    }
}

fn make_token(id: u16, name: &str, pattern: &str) -> (SymbolId, Token) {
    (
        SymbolId(id),
        Token {
            name: name.to_string(),
            pattern: TokenPattern::String(pattern.to_string()),
            fragile: false,
        },
    )
}

fn has_error(result: &ValidationResult, pred: impl Fn(&ValidationError) -> bool) -> bool {
    result.errors.iter().any(pred)
}

fn has_warning(result: &ValidationResult, pred: impl Fn(&ValidationWarning) -> bool) -> bool {
    result.warnings.iter().any(pred)
}

fn validate(g: &Grammar) -> ValidationResult {
    let mut v = GrammarValidator::new();
    v.validate(g)
}

// ---------------------------------------------------------------------------
// 1. Grammar with no rules
// ---------------------------------------------------------------------------

#[test]
fn no_rules_empty_grammar_error() {
    let g = Grammar::new("empty".into());
    let r = validate(&g);
    assert!(has_error(&r, |e| matches!(
        e,
        ValidationError::EmptyGrammar
    )));
}

#[test]
fn default_grammar_reports_empty() {
    let g = Grammar::default();
    let r = validate(&g);
    assert!(has_error(&r, |e| matches!(
        e,
        ValidationError::EmptyGrammar
    )));
}

#[test]
fn grammar_with_only_tokens_is_empty() {
    let mut g = Grammar::new("tok_only".into());
    let (tid, tok) = make_token(1, "a", "a");
    g.tokens.insert(tid, tok);
    let r = validate(&g);
    assert!(has_error(&r, |e| matches!(
        e,
        ValidationError::EmptyGrammar
    )));
}

// ---------------------------------------------------------------------------
// 2. Grammar with no start symbol (only externals, no rules)
// ---------------------------------------------------------------------------

#[test]
fn grammar_with_only_externals_is_empty() {
    let mut g = Grammar::new("ext_only".into());
    g.externals.push(ExternalToken {
        name: "indent".into(),
        symbol_id: SymbolId(50),
    });
    let r = validate(&g);
    assert!(has_error(&r, |e| matches!(
        e,
        ValidationError::EmptyGrammar
    )));
}

// ---------------------------------------------------------------------------
// 3. Circular rule references
// ---------------------------------------------------------------------------

#[test]
fn direct_self_recursion_no_base_case() {
    let mut g = Grammar::new("self_rec".into());
    g.add_rule(make_rule(0, vec![Symbol::NonTerminal(SymbolId(0))], 0));
    let r = validate(&g);
    assert!(
        has_error(&r, |e| matches!(e, ValidationError::CyclicRule { .. })),
        "expected CyclicRule, got: {:?}",
        r.errors
    );
}

#[test]
fn mutual_recursion_two_symbols() {
    let mut g = Grammar::new("mutual2".into());
    g.add_rule(make_rule(0, vec![Symbol::NonTerminal(SymbolId(1))], 0));
    g.add_rule(make_rule(1, vec![Symbol::NonTerminal(SymbolId(0))], 1));
    let r = validate(&g);
    assert!(has_error(&r, |e| matches!(
        e,
        ValidationError::CyclicRule { .. }
    )));
}

#[test]
fn mutual_recursion_three_symbols() {
    let mut g = Grammar::new("mutual3".into());
    // A -> B, B -> C, C -> A
    g.add_rule(make_rule(0, vec![Symbol::NonTerminal(SymbolId(1))], 0));
    g.add_rule(make_rule(1, vec![Symbol::NonTerminal(SymbolId(2))], 1));
    g.add_rule(make_rule(2, vec![Symbol::NonTerminal(SymbolId(0))], 2));
    let r = validate(&g);
    assert!(has_error(&r, |e| matches!(
        e,
        ValidationError::CyclicRule { .. }
    )));
}

#[test]
fn recursion_with_terminal_base_case_still_flags_cycle() {
    let mut g = Grammar::new("rec_base".into());
    let (tid, tok) = make_token(10, "x", "x");
    g.tokens.insert(tid, tok);
    // A -> A x  and  A -> x
    g.add_rule(make_rule(
        0,
        vec![
            Symbol::NonTerminal(SymbolId(0)),
            Symbol::Terminal(SymbolId(10)),
        ],
        0,
    ));
    g.add_rule(make_rule(0, vec![Symbol::Terminal(SymbolId(10))], 1));
    let r = validate(&g);
    assert!(has_error(&r, |e| matches!(
        e,
        ValidationError::CyclicRule { .. }
    )));
}

// ---------------------------------------------------------------------------
// 4. Rules referencing undefined symbols
// ---------------------------------------------------------------------------

#[test]
fn undefined_terminal_reference() {
    let mut g = Grammar::new("undef_t".into());
    g.add_rule(make_rule(0, vec![Symbol::Terminal(SymbolId(99))], 0));
    let r = validate(&g);
    assert!(has_error(&r, |e| matches!(
        e,
        ValidationError::UndefinedSymbol { symbol, .. } if *symbol == SymbolId(99)
    )));
}

#[test]
fn undefined_nonterminal_reference() {
    let mut g = Grammar::new("undef_nt".into());
    let (tid, tok) = make_token(1, "a", "a");
    g.tokens.insert(tid, tok);
    g.add_rule(make_rule(
        0,
        vec![
            Symbol::Terminal(SymbolId(1)),
            Symbol::NonTerminal(SymbolId(200)),
        ],
        0,
    ));
    let r = validate(&g);
    assert!(has_error(&r, |e| matches!(
        e,
        ValidationError::UndefinedSymbol { symbol, .. } if *symbol == SymbolId(200)
    )));
}

#[test]
fn undefined_external_reference() {
    let mut g = Grammar::new("undef_ext".into());
    g.add_rule(make_rule(0, vec![Symbol::External(SymbolId(42))], 0));
    let r = validate(&g);
    assert!(has_error(&r, |e| matches!(
        e,
        ValidationError::UndefinedSymbol { symbol, .. } if *symbol == SymbolId(42)
    )));
}

#[test]
fn multiple_undefined_symbols_all_reported() {
    let mut g = Grammar::new("multi_undef".into());
    g.add_rule(make_rule(
        0,
        vec![
            Symbol::Terminal(SymbolId(50)),
            Symbol::Terminal(SymbolId(60)),
            Symbol::NonTerminal(SymbolId(70)),
        ],
        0,
    ));
    let r = validate(&g);
    let undef_ids: Vec<SymbolId> = r
        .errors
        .iter()
        .filter_map(|e| match e {
            ValidationError::UndefinedSymbol { symbol, .. } => Some(*symbol),
            _ => None,
        })
        .collect();
    assert!(undef_ids.contains(&SymbolId(50)));
    assert!(undef_ids.contains(&SymbolId(60)));
    assert!(undef_ids.contains(&SymbolId(70)));
}

// ---------------------------------------------------------------------------
// 5. Duplicate symbol definitions
// ---------------------------------------------------------------------------

#[test]
fn duplicate_rules_same_lhs_same_rhs() {
    let mut g = Grammar::new("dup_same".into());
    let (tid, tok) = make_token(1, "x", "x");
    g.tokens.insert(tid, tok);
    g.add_rule(make_rule(0, vec![Symbol::Terminal(SymbolId(1))], 0));
    g.add_rule(make_rule(0, vec![Symbol::Terminal(SymbolId(1))], 1));
    let r = validate(&g);
    assert_eq!(r.stats.total_rules, 2);
    // Should not panic; duplicates are tolerated.
}

#[test]
fn many_duplicate_rules_for_same_lhs() {
    let mut g = Grammar::new("many_dup".into());
    let (tid, tok) = make_token(1, "x", "x");
    g.tokens.insert(tid, tok);
    for i in 0..20 {
        g.add_rule(make_rule(0, vec![Symbol::Terminal(SymbolId(1))], i));
    }
    let r = validate(&g);
    assert_eq!(r.stats.total_rules, 20);
}

// ---------------------------------------------------------------------------
// 6. Empty rule alternatives
// ---------------------------------------------------------------------------

#[test]
fn rule_with_empty_rhs() {
    let mut g = Grammar::new("empty_rhs".into());
    g.add_rule(make_rule(0, vec![], 0));
    let r = validate(&g);
    // An empty RHS is effectively epsilon; should not panic.
    assert_eq!(r.stats.total_rules, 1);
}

#[test]
fn choice_with_zero_alternatives() {
    let mut g = Grammar::new("empty_choice".into());
    g.add_rule(make_rule(0, vec![Symbol::Choice(vec![])], 0));
    let r = validate(&g);
    // Empty choice should not panic the validator.
    assert_eq!(r.stats.total_rules, 1);
}

#[test]
fn sequence_with_zero_elements() {
    let mut g = Grammar::new("empty_seq".into());
    g.add_rule(make_rule(0, vec![Symbol::Sequence(vec![])], 0));
    let r = validate(&g);
    assert_eq!(r.stats.total_rules, 1);
}

// ---------------------------------------------------------------------------
// 7. Rules with only epsilon
// ---------------------------------------------------------------------------

#[test]
fn epsilon_only_rule_does_not_panic() {
    let mut g = Grammar::new("eps".into());
    g.add_rule(make_rule(0, vec![Symbol::Epsilon], 0));
    let _r = validate(&g);
}

#[test]
fn multiple_epsilon_rules() {
    let mut g = Grammar::new("multi_eps".into());
    g.add_rule(make_rule(0, vec![Symbol::Epsilon], 0));
    g.add_rule(make_rule(0, vec![Symbol::Epsilon], 1));
    let r = validate(&g);
    assert_eq!(r.stats.total_rules, 2);
}

#[test]
fn epsilon_mixed_with_terminal() {
    let mut g = Grammar::new("eps_mix".into());
    let (tid, tok) = make_token(1, "x", "x");
    g.tokens.insert(tid, tok);
    g.add_rule(make_rule(
        0,
        vec![Symbol::Epsilon, Symbol::Terminal(SymbolId(1))],
        0,
    ));
    let r = validate(&g);
    // Mixing epsilon with terminals in the same alternative is unusual but should not crash.
    assert_eq!(r.stats.total_rules, 1);
}

// ---------------------------------------------------------------------------
// 8. Extremely deep nesting
// ---------------------------------------------------------------------------

#[test]
fn deeply_nested_optional() {
    let mut g = Grammar::new("deep_opt".into());
    let (tid, tok) = make_token(1, "x", "x");
    g.tokens.insert(tid, tok);
    let mut sym: Symbol = Symbol::Terminal(SymbolId(1));
    for _ in 0..50 {
        sym = Symbol::Optional(Box::new(sym));
    }
    g.add_rule(make_rule(0, vec![sym], 0));
    let _r = validate(&g);
    // Must complete without stack overflow.
}

#[test]
fn deeply_nested_repeat() {
    let mut g = Grammar::new("deep_rep".into());
    let (tid, tok) = make_token(1, "x", "x");
    g.tokens.insert(tid, tok);
    let mut sym: Symbol = Symbol::Terminal(SymbolId(1));
    for _ in 0..50 {
        sym = Symbol::Repeat(Box::new(sym));
    }
    g.add_rule(make_rule(0, vec![sym], 0));
    let _r = validate(&g);
}

#[test]
fn deeply_nested_choice_single_branch() {
    let mut g = Grammar::new("deep_choice".into());
    let (tid, tok) = make_token(1, "x", "x");
    g.tokens.insert(tid, tok);
    let mut sym: Symbol = Symbol::Terminal(SymbolId(1));
    for _ in 0..50 {
        sym = Symbol::Choice(vec![sym]);
    }
    g.add_rule(make_rule(0, vec![sym], 0));
    let _r = validate(&g);
}

#[test]
fn deeply_nested_sequence_single_element() {
    let mut g = Grammar::new("deep_seq".into());
    let (tid, tok) = make_token(1, "x", "x");
    g.tokens.insert(tid, tok);
    let mut sym: Symbol = Symbol::Terminal(SymbolId(1));
    for _ in 0..50 {
        sym = Symbol::Sequence(vec![sym]);
    }
    g.add_rule(make_rule(0, vec![sym], 0));
    let _r = validate(&g);
}

// ---------------------------------------------------------------------------
// 9. Many productions for a single nonterminal
// ---------------------------------------------------------------------------

#[test]
fn hundred_productions_for_one_nonterminal() {
    let mut g = Grammar::new("many_prods".into());
    for i in 0u16..100 {
        let (tid, tok) = make_token(100 + i, &format!("t{}", i), &format!("t{}", i));
        g.tokens.insert(tid, tok);
        g.add_rule(make_rule(0, vec![Symbol::Terminal(SymbolId(100 + i))], i));
    }
    let r = validate(&g);
    assert_eq!(r.stats.total_rules, 100);
    assert_eq!(r.stats.total_tokens, 100);
    // All errors should be limited to potential warnings, not crashes.
    assert!(
        !has_error(&r, |e| matches!(e, ValidationError::EmptyGrammar)),
        "grammar with 100 rules should not be empty"
    );
}

#[test]
fn large_rhs_triggers_inefficiency_warning() {
    let mut g = Grammar::new("long_rhs".into());
    let mut rhs = Vec::new();
    for i in 0u16..15 {
        let (tid, tok) = make_token(100 + i, &format!("t{}", i), &format!("t{}", i));
        g.tokens.insert(tid, tok);
        rhs.push(Symbol::Terminal(SymbolId(100 + i)));
    }
    g.add_rule(make_rule(0, rhs, 0));
    let r = validate(&g);
    assert!(
        has_warning(&r, |w| matches!(
            w,
            ValidationWarning::InefficientRule { .. }
        )),
        "expected InefficientRule warning for long rule, got: {:?}",
        r.warnings
    );
}

// ---------------------------------------------------------------------------
// 10. Valid grammars that pass validation
// ---------------------------------------------------------------------------

#[test]
fn minimal_valid_grammar() {
    let mut g = Grammar::new("minimal".into());
    let (tid, tok) = make_token(1, "x", "x");
    g.tokens.insert(tid, tok);
    g.add_rule(make_rule(0, vec![Symbol::Terminal(SymbolId(1))], 0));
    let r = validate(&g);
    assert!(r.errors.is_empty(), "errors: {:?}", r.errors);
}

#[test]
fn valid_grammar_two_nonterminals() {
    let mut g = Grammar::new("two_nt".into());
    let (tid, tok) = make_token(10, "a", "a");
    g.tokens.insert(tid, tok);
    // S -> A, A -> a
    g.add_rule(make_rule(0, vec![Symbol::NonTerminal(SymbolId(1))], 0));
    g.add_rule(make_rule(1, vec![Symbol::Terminal(SymbolId(10))], 1));
    let r = validate(&g);
    let real_errors: Vec<_> = r
        .errors
        .iter()
        .filter(|e| !matches!(e, ValidationError::CyclicRule { .. }))
        .collect();
    assert!(
        real_errors.is_empty(),
        "unexpected errors: {:?}",
        real_errors
    );
}

#[test]
fn valid_grammar_with_optional() {
    let mut g = Grammar::new("opt_valid".into());
    let (tid, tok) = make_token(1, "x", "x");
    g.tokens.insert(tid, tok);
    g.add_rule(make_rule(
        0,
        vec![Symbol::Optional(Box::new(Symbol::Terminal(SymbolId(1))))],
        0,
    ));
    let r = validate(&g);
    assert!(r.errors.is_empty(), "errors: {:?}", r.errors);
}

#[test]
fn valid_grammar_with_repeat() {
    let mut g = Grammar::new("rep_valid".into());
    let (tid, tok) = make_token(1, "x", "x");
    g.tokens.insert(tid, tok);
    g.add_rule(make_rule(
        0,
        vec![Symbol::Repeat(Box::new(Symbol::Terminal(SymbolId(1))))],
        0,
    ));
    let r = validate(&g);
    assert!(r.errors.is_empty(), "errors: {:?}", r.errors);
}

#[test]
fn valid_grammar_with_choice_of_terminals() {
    let mut g = Grammar::new("choice_valid".into());
    let (t1, tok1) = make_token(1, "a", "a");
    let (t2, tok2) = make_token(2, "b", "b");
    g.tokens.insert(t1, tok1);
    g.tokens.insert(t2, tok2);
    g.add_rule(make_rule(
        0,
        vec![Symbol::Choice(vec![
            Symbol::Terminal(SymbolId(1)),
            Symbol::Terminal(SymbolId(2)),
        ])],
        0,
    ));
    let r = validate(&g);
    assert!(r.errors.is_empty(), "errors: {:?}", r.errors);
}

#[test]
fn valid_grammar_with_sequence() {
    let mut g = Grammar::new("seq_valid".into());
    let (t1, tok1) = make_token(1, "a", "a");
    let (t2, tok2) = make_token(2, "b", "b");
    g.tokens.insert(t1, tok1);
    g.tokens.insert(t2, tok2);
    g.add_rule(make_rule(
        0,
        vec![Symbol::Sequence(vec![
            Symbol::Terminal(SymbolId(1)),
            Symbol::Terminal(SymbolId(2)),
        ])],
        0,
    ));
    let r = validate(&g);
    assert!(r.errors.is_empty(), "errors: {:?}", r.errors);
}

// ---------------------------------------------------------------------------
// Grammar.validate() (the Result-returning method on Grammar itself)
// ---------------------------------------------------------------------------

#[test]
fn grammar_validate_ok_for_valid_grammar() {
    let mut g = Grammar::new("gv".into());
    let (tid, tok) = make_token(1, "x", "x");
    g.tokens.insert(tid, tok);
    g.add_rule(make_rule(0, vec![Symbol::Terminal(SymbolId(1))], 0));
    assert!(g.validate().is_ok());
}

#[test]
fn grammar_validate_err_for_unresolved_symbol() {
    let mut g = Grammar::new("gv_unresolved".into());
    g.add_rule(make_rule(0, vec![Symbol::Terminal(SymbolId(99))], 0));
    assert!(g.validate().is_err());
}

#[test]
fn grammar_validate_err_for_unresolved_external() {
    let mut g = Grammar::new("gv_ext".into());
    g.add_rule(make_rule(0, vec![Symbol::External(SymbolId(42))], 0));
    assert!(g.validate().is_err());
}

// ---------------------------------------------------------------------------
// Field ordering (Grammar.validate)
// ---------------------------------------------------------------------------

#[test]
fn grammar_validate_err_for_bad_field_ordering() {
    let mut g = Grammar::new("field_order".into());
    let (tid, tok) = make_token(1, "x", "x");
    g.tokens.insert(tid, tok);
    g.add_rule(make_rule(0, vec![Symbol::Terminal(SymbolId(1))], 0));
    // Insert fields in reverse lexicographic order
    g.fields.insert(FieldId(0), "z_last".into());
    g.fields.insert(FieldId(1), "a_first".into());
    let result = g.validate();
    assert!(
        result.is_err(),
        "expected InvalidFieldOrdering error for non-lexicographic fields"
    );
}

#[test]
fn grammar_validate_ok_for_correct_field_ordering() {
    let mut g = Grammar::new("field_ok".into());
    let (tid, tok) = make_token(1, "x", "x");
    g.tokens.insert(tid, tok);
    g.add_rule(make_rule(0, vec![Symbol::Terminal(SymbolId(1))], 0));
    g.fields.insert(FieldId(0), "alpha".into());
    g.fields.insert(FieldId(1), "beta".into());
    assert!(g.validate().is_ok());
}

// ---------------------------------------------------------------------------
// Nested complex symbols with undefined inner
// ---------------------------------------------------------------------------

#[test]
fn repeat_one_with_undefined_inner() {
    let mut g = Grammar::new("rep1_undef".into());
    g.add_rule(make_rule(
        0,
        vec![Symbol::RepeatOne(Box::new(Symbol::Terminal(SymbolId(77))))],
        0,
    ));
    let r = validate(&g);
    assert!(has_error(&r, |e| matches!(
        e,
        ValidationError::UndefinedSymbol { symbol, .. } if *symbol == SymbolId(77)
    )));
}

#[test]
fn nested_optional_in_choice_with_undefined() {
    let mut g = Grammar::new("nested_undef".into());
    let (tid, tok) = make_token(1, "a", "a");
    g.tokens.insert(tid, tok);
    // Choice[ a, Optional(undefined) ]
    g.add_rule(make_rule(
        0,
        vec![Symbol::Choice(vec![
            Symbol::Terminal(SymbolId(1)),
            Symbol::Optional(Box::new(Symbol::NonTerminal(SymbolId(88)))),
        ])],
        0,
    ));
    let r = validate(&g);
    assert!(has_error(&r, |e| matches!(
        e,
        ValidationError::UndefinedSymbol { symbol, .. } if *symbol == SymbolId(88)
    )));
}

// ---------------------------------------------------------------------------
// Non-productive symbol detection
// ---------------------------------------------------------------------------

#[test]
fn nonterminal_only_referencing_undefined_is_non_productive() {
    let mut g = Grammar::new("nonprod".into());
    // A -> B, but B is not defined at all
    g.add_rule(make_rule(0, vec![Symbol::NonTerminal(SymbolId(1))], 0));
    let r = validate(&g);
    assert!(
        has_error(&r, |e| matches!(
            e,
            ValidationError::NonProductiveSymbol { .. }
        )) || has_error(&r, |e| matches!(e, ValidationError::UndefinedSymbol { .. })),
        "expected NonProductiveSymbol or UndefinedSymbol, got: {:?}",
        r.errors
    );
}

// ---------------------------------------------------------------------------
// Invalid field index
// ---------------------------------------------------------------------------

#[test]
fn field_index_beyond_rhs_length() {
    let mut g = Grammar::new("field_oob".into());
    let (tid, tok) = make_token(1, "x", "x");
    g.tokens.insert(tid, tok);
    g.add_rule(Rule {
        lhs: SymbolId(0),
        rhs: vec![Symbol::Terminal(SymbolId(1))],
        precedence: None,
        associativity: None,
        fields: vec![(FieldId(0), 999)],
        production_id: ProductionId(0),
    });
    g.fields.insert(FieldId(0), "f".into());
    let r = validate(&g);
    assert!(has_error(&r, |e| matches!(
        e,
        ValidationError::InvalidField { .. }
    )));
}

// ---------------------------------------------------------------------------
// Precedence conflicts
// ---------------------------------------------------------------------------

#[test]
fn same_symbol_at_two_precedence_levels() {
    let mut g = Grammar::new("prec_dup".into());
    let (tid, tok) = make_token(1, "op", "+");
    g.tokens.insert(tid, tok);
    g.add_rule(make_rule(0, vec![Symbol::Terminal(SymbolId(1))], 0));
    g.precedences.push(Precedence {
        level: 1,
        associativity: Associativity::Left,
        symbols: vec![SymbolId(1)],
    });
    g.precedences.push(Precedence {
        level: 5,
        associativity: Associativity::Right,
        symbols: vec![SymbolId(1)],
    });
    let r = validate(&g);
    assert!(has_error(&r, |e| matches!(
        e,
        ValidationError::ConflictingPrecedence { symbol, .. } if *symbol == SymbolId(1)
    )));
}

// ---------------------------------------------------------------------------
// External token conflicts
// ---------------------------------------------------------------------------

#[test]
fn duplicate_external_names_detected() {
    let mut g = Grammar::new("ext_dup".into());
    let (tid, tok) = make_token(1, "x", "x");
    g.tokens.insert(tid, tok);
    g.add_rule(make_rule(0, vec![Symbol::Terminal(SymbolId(1))], 0));
    g.externals.push(ExternalToken {
        name: "indent".into(),
        symbol_id: SymbolId(50),
    });
    g.externals.push(ExternalToken {
        name: "indent".into(),
        symbol_id: SymbolId(51),
    });
    let r = validate(&g);
    assert!(has_error(&r, |e| matches!(
        e,
        ValidationError::ExternalTokenConflict { .. }
    )));
}

// ---------------------------------------------------------------------------
// Invalid regex
// ---------------------------------------------------------------------------

#[test]
fn empty_regex_is_flagged() {
    let mut g = Grammar::new("bad_regex".into());
    g.tokens.insert(
        SymbolId(1),
        Token {
            name: "bad".into(),
            pattern: TokenPattern::Regex("".into()),
            fragile: false,
        },
    );
    g.add_rule(make_rule(0, vec![Symbol::Terminal(SymbolId(1))], 0));
    let r = validate(&g);
    assert!(has_error(&r, |e| matches!(
        e,
        ValidationError::InvalidRegex { .. }
    )));
}

// ---------------------------------------------------------------------------
// Duplicate token patterns (warning)
// ---------------------------------------------------------------------------

#[test]
fn duplicate_string_patterns_warn() {
    let mut g = Grammar::new("dup_pat".into());
    let (t1, tok1) = make_token(1, "plus", "+");
    let (t2, tok2) = make_token(2, "add", "+");
    g.tokens.insert(t1, tok1);
    g.tokens.insert(t2, tok2);
    g.add_rule(make_rule(
        0,
        vec![Symbol::Terminal(SymbolId(1)), Symbol::Terminal(SymbolId(2))],
        0,
    ));
    let r = validate(&g);
    assert!(has_warning(&r, |w| matches!(
        w,
        ValidationWarning::DuplicateTokenPattern { .. }
    )));
}

// ---------------------------------------------------------------------------
// Validation statistics
// ---------------------------------------------------------------------------

#[test]
fn stats_reflect_multiple_rules() {
    let mut g = Grammar::new("stats".into());
    let (tid, tok) = make_token(1, "x", "x");
    g.tokens.insert(tid, tok);
    g.add_rule(make_rule(0, vec![Symbol::Terminal(SymbolId(1))], 0));
    g.add_rule(make_rule(
        0,
        vec![Symbol::Terminal(SymbolId(1)), Symbol::Terminal(SymbolId(1))],
        1,
    ));
    let r = validate(&g);
    assert_eq!(r.stats.total_tokens, 1);
    assert_eq!(r.stats.total_rules, 2);
    assert_eq!(r.stats.max_rule_length, 2);
}

#[test]
fn stats_zero_for_empty_grammar() {
    let g = Grammar::new("empty_stats".into());
    let r = validate(&g);
    assert_eq!(r.stats.total_symbols, 0);
    assert_eq!(r.stats.total_rules, 0);
    assert_eq!(r.stats.total_tokens, 0);
}

// ---------------------------------------------------------------------------
// Validator reuse across grammars
// ---------------------------------------------------------------------------

#[test]
fn validator_reuse_does_not_leak_errors() {
    let mut v = GrammarValidator::new();

    // First: empty grammar produces error
    let g1 = Grammar::new("g1".into());
    let r1 = v.validate(&g1);
    assert!(!r1.errors.is_empty());

    // Second: valid grammar should have no errors
    let mut g2 = Grammar::new("g2".into());
    let (tid, tok) = make_token(1, "x", "x");
    g2.tokens.insert(tid, tok);
    g2.add_rule(make_rule(0, vec![Symbol::Terminal(SymbolId(1))], 0));
    let r2 = v.validate(&g2);
    assert!(r2.errors.is_empty(), "leaked errors: {:?}", r2.errors);
}
