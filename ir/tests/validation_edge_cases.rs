// Edge-case tests for IR grammar validation
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

// ---------------------------------------------------------------------------
// Empty grammar
// ---------------------------------------------------------------------------

#[test]
fn empty_grammar_produces_empty_grammar_error() {
    let grammar = Grammar::new("empty".into());
    let mut v = GrammarValidator::new();
    let r = v.validate(&grammar);

    assert!(has_error(&r, |e| matches!(
        e,
        ValidationError::EmptyGrammar
    )));
    assert_eq!(r.stats.total_symbols, 0);
    assert_eq!(r.stats.total_rules, 0);
}

#[test]
fn default_grammar_is_empty() {
    let grammar = Grammar::default();
    let mut v = GrammarValidator::new();
    let r = v.validate(&grammar);

    assert!(has_error(&r, |e| matches!(
        e,
        ValidationError::EmptyGrammar
    )));
}

// ---------------------------------------------------------------------------
// Grammar with only one terminal
// ---------------------------------------------------------------------------

#[test]
fn single_terminal_rule_is_valid() {
    let mut g = Grammar::new("single_term".into());
    let (tid, tok) = make_token(1, "x", "x");
    g.tokens.insert(tid, tok);
    g.add_rule(make_rule(0, vec![Symbol::Terminal(SymbolId(1))], 0));

    let mut v = GrammarValidator::new();
    let r = v.validate(&g);

    // No errors expected: one rule referencing one defined terminal.
    assert!(r.errors.is_empty(), "errors: {:?}", r.errors);
    assert_eq!(r.stats.total_rules, 1);
    assert_eq!(r.stats.total_tokens, 1);
}

// ---------------------------------------------------------------------------
// Unreachable non-terminals
// ---------------------------------------------------------------------------

#[test]
fn unreachable_nonterminal_produces_warning() {
    let mut g = Grammar::new("unreach".into());
    let (tid, tok) = make_token(10, "a", "a");
    g.tokens.insert(tid, tok);

    // Start rule: S -> a
    g.add_rule(make_rule(0, vec![Symbol::Terminal(SymbolId(10))], 0));

    // Unreachable rule: X -> a (never referenced from S)
    g.add_rule(make_rule(5, vec![Symbol::Terminal(SymbolId(10))], 1));

    let mut v = GrammarValidator::new();
    let r = v.validate(&g);

    // The unreachable symbol should appear as a warning (UnusedToken variant is used
    // for unreachable symbols in the current implementation).
    assert!(
        has_warning(&r, |w| matches!(w, ValidationWarning::UnusedToken { .. })),
        "expected an unused/unreachable warning, got warnings: {:?}",
        r.warnings
    );
}

// ---------------------------------------------------------------------------
// Duplicate rules (same LHS, same RHS)
// ---------------------------------------------------------------------------

#[test]
fn duplicate_rules_accepted_without_panic() {
    let mut g = Grammar::new("dup".into());
    let (tid, tok) = make_token(1, "x", "x");
    g.tokens.insert(tid, tok);

    // Two identical rules for the same LHS
    g.add_rule(make_rule(0, vec![Symbol::Terminal(SymbolId(1))], 0));
    g.add_rule(make_rule(0, vec![Symbol::Terminal(SymbolId(1))], 1));

    let mut v = GrammarValidator::new();
    let r = v.validate(&g);

    // Validation should complete; whether it flags duplicates is implementation-defined.
    assert_eq!(r.stats.total_rules, 2);
}

// ---------------------------------------------------------------------------
// Circular / recursive references
// ---------------------------------------------------------------------------

#[test]
fn direct_self_recursion_detected_as_cycle() {
    let mut g = Grammar::new("self_rec".into());
    let (tid, tok) = make_token(10, "x", "x");
    g.tokens.insert(tid, tok);

    // A -> A  (direct self-recursion with no base case terminal)
    g.add_rule(make_rule(0, vec![Symbol::NonTerminal(SymbolId(0))], 0));

    let mut v = GrammarValidator::new();
    let r = v.validate(&g);

    assert!(
        has_error(&r, |e| matches!(e, ValidationError::CyclicRule { .. })),
        "expected CyclicRule error, got: {:?}",
        r.errors
    );
}

#[test]
fn mutual_recursion_detected_as_cycle() {
    let mut g = Grammar::new("mutual".into());
    let (tid, tok) = make_token(10, "x", "x");
    g.tokens.insert(tid, tok);

    // A -> B, B -> A (mutual recursion, no base case)
    g.add_rule(make_rule(0, vec![Symbol::NonTerminal(SymbolId(1))], 0));
    g.add_rule(make_rule(1, vec![Symbol::NonTerminal(SymbolId(0))], 1));

    let mut v = GrammarValidator::new();
    let r = v.validate(&g);

    assert!(
        has_error(&r, |e| matches!(e, ValidationError::CyclicRule { .. })),
        "expected CyclicRule error, got: {:?}",
        r.errors
    );
}

#[test]
fn recursion_with_base_case_still_flags_cycle() {
    let mut g = Grammar::new("rec_base".into());
    let (tid, tok) = make_token(10, "x", "x");
    g.tokens.insert(tid, tok);

    // A -> A x  (left-recursive, but has terminal in RHS)
    g.add_rule(make_rule(
        0,
        vec![
            Symbol::NonTerminal(SymbolId(0)),
            Symbol::Terminal(SymbolId(10)),
        ],
        0,
    ));
    // A -> x    (base case)
    g.add_rule(make_rule(0, vec![Symbol::Terminal(SymbolId(10))], 1));

    let mut v = GrammarValidator::new();
    let r = v.validate(&g);

    // The cycle checker flags all cycles regardless of base cases.
    assert!(
        has_error(&r, |e| matches!(e, ValidationError::CyclicRule { .. })),
        "expected CyclicRule for recursive rule, got: {:?}",
        r.errors
    );
}

// ---------------------------------------------------------------------------
// Invalid / undefined symbol IDs
// ---------------------------------------------------------------------------

#[test]
fn undefined_terminal_in_rhs() {
    let mut g = Grammar::new("undef_term".into());

    // Rule references terminal 99 which is never defined
    g.add_rule(make_rule(0, vec![Symbol::Terminal(SymbolId(99))], 0));

    let mut v = GrammarValidator::new();
    let r = v.validate(&g);

    assert!(
        has_error(&r, |e| matches!(
            e,
            ValidationError::UndefinedSymbol { symbol, .. } if *symbol == SymbolId(99)
        )),
        "expected UndefinedSymbol for 99, got: {:?}",
        r.errors
    );
}

#[test]
fn undefined_nonterminal_in_rhs() {
    let mut g = Grammar::new("undef_nt".into());
    let (tid, tok) = make_token(1, "a", "a");
    g.tokens.insert(tid, tok);

    // S -> a UNKNOWN
    g.add_rule(make_rule(
        0,
        vec![
            Symbol::Terminal(SymbolId(1)),
            Symbol::NonTerminal(SymbolId(200)),
        ],
        0,
    ));

    let mut v = GrammarValidator::new();
    let r = v.validate(&g);

    assert!(
        has_error(&r, |e| matches!(
            e,
            ValidationError::UndefinedSymbol { symbol, .. } if *symbol == SymbolId(200)
        )),
        "expected UndefinedSymbol for 200, got: {:?}",
        r.errors
    );
}

#[test]
fn undefined_external_in_rhs() {
    let mut g = Grammar::new("undef_ext".into());

    g.add_rule(make_rule(0, vec![Symbol::External(SymbolId(42))], 0));

    let mut v = GrammarValidator::new();
    let r = v.validate(&g);

    assert!(
        has_error(&r, |e| matches!(
            e,
            ValidationError::UndefinedSymbol { symbol, .. } if *symbol == SymbolId(42)
        )),
        "expected UndefinedSymbol for external 42, got: {:?}",
        r.errors
    );
}

// ---------------------------------------------------------------------------
// Start symbol with no rules
// ---------------------------------------------------------------------------

#[test]
fn grammar_with_tokens_but_no_rules_is_empty() {
    let mut g = Grammar::new("tokens_only".into());
    let (tid, tok) = make_token(1, "tok", "tok");
    g.tokens.insert(tid, tok);

    let mut v = GrammarValidator::new();
    let r = v.validate(&g);

    // No rules means the grammar is empty.
    assert!(has_error(&r, |e| matches!(
        e,
        ValidationError::EmptyGrammar
    )));
}

// ---------------------------------------------------------------------------
// Non-productive symbols
// ---------------------------------------------------------------------------

#[test]
fn nonterminal_referencing_only_undefined_is_non_productive() {
    let mut g = Grammar::new("nonprod".into());

    // A -> B  but B has no rules and is not a token
    g.add_rule(make_rule(0, vec![Symbol::NonTerminal(SymbolId(1))], 0));

    let mut v = GrammarValidator::new();
    let r = v.validate(&g);

    // SymbolId(1) is undefined; SymbolId(0) referencing it is non-productive.
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
fn field_index_out_of_bounds_is_error() {
    let mut g = Grammar::new("bad_field".into());
    let (tid, tok) = make_token(1, "x", "x");
    g.tokens.insert(tid, tok);

    g.add_rule(Rule {
        lhs: SymbolId(0),
        rhs: vec![Symbol::Terminal(SymbolId(1))],
        precedence: None,
        associativity: None,
        fields: vec![(FieldId(0), 999)], // index 999 out of bounds
        production_id: ProductionId(0),
    });
    g.fields.insert(FieldId(0), "f".into());

    let mut v = GrammarValidator::new();
    let r = v.validate(&g);

    assert!(
        has_error(&r, |e| matches!(e, ValidationError::InvalidField { .. })),
        "expected InvalidField, got: {:?}",
        r.errors
    );
}

// ---------------------------------------------------------------------------
// Conflicting precedences
// ---------------------------------------------------------------------------

#[test]
fn conflicting_precedences_detected() {
    let mut g = Grammar::new("prec_conflict".into());
    let (tid, tok) = make_token(1, "op", "+");
    g.tokens.insert(tid, tok);

    g.add_rule(make_rule(0, vec![Symbol::Terminal(SymbolId(1))], 0));

    // Same symbol at two different precedence levels
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

    let mut v = GrammarValidator::new();
    let r = v.validate(&g);

    assert!(
        has_error(&r, |e| matches!(
            e,
            ValidationError::ConflictingPrecedence { symbol, .. } if *symbol == SymbolId(1)
        )),
        "expected ConflictingPrecedence, got: {:?}",
        r.errors
    );
}

// ---------------------------------------------------------------------------
// External token conflicts
// ---------------------------------------------------------------------------

#[test]
fn duplicate_external_token_names_detected() {
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

    let mut v = GrammarValidator::new();
    let r = v.validate(&g);

    assert!(
        has_error(&r, |e| matches!(
            e,
            ValidationError::ExternalTokenConflict { .. }
        )),
        "expected ExternalTokenConflict, got: {:?}",
        r.errors
    );
}

// ---------------------------------------------------------------------------
// Empty regex token
// ---------------------------------------------------------------------------

#[test]
fn empty_regex_pattern_is_invalid() {
    let mut g = Grammar::new("empty_regex".into());
    g.tokens.insert(
        SymbolId(1),
        Token {
            name: "bad".into(),
            pattern: TokenPattern::Regex("".into()),
            fragile: false,
        },
    );
    g.add_rule(make_rule(0, vec![Symbol::Terminal(SymbolId(1))], 0));

    let mut v = GrammarValidator::new();
    let r = v.validate(&g);

    assert!(
        has_error(&r, |e| matches!(e, ValidationError::InvalidRegex { .. })),
        "expected InvalidRegex for empty regex, got: {:?}",
        r.errors
    );
}

// ---------------------------------------------------------------------------
// Duplicate token patterns (warning)
// ---------------------------------------------------------------------------

#[test]
fn duplicate_token_patterns_produce_warning() {
    let mut g = Grammar::new("dup_pat".into());
    let (tid1, tok1) = make_token(1, "plus", "+");
    let (tid2, tok2) = make_token(2, "add", "+"); // same pattern
    g.tokens.insert(tid1, tok1);
    g.tokens.insert(tid2, tok2);

    g.add_rule(make_rule(
        0,
        vec![Symbol::Terminal(SymbolId(1)), Symbol::Terminal(SymbolId(2))],
        0,
    ));

    let mut v = GrammarValidator::new();
    let r = v.validate(&g);

    assert!(
        has_warning(&r, |w| matches!(
            w,
            ValidationWarning::DuplicateTokenPattern { .. }
        )),
        "expected DuplicateTokenPattern warning, got: {:?}",
        r.warnings
    );
}

// ---------------------------------------------------------------------------
// Complex symbols in RHS (Optional, Repeat, Choice, Sequence)
// ---------------------------------------------------------------------------

#[test]
fn optional_with_undefined_inner_detected() {
    let mut g = Grammar::new("opt_undef".into());

    g.add_rule(make_rule(
        0,
        vec![Symbol::Optional(Box::new(Symbol::Terminal(SymbolId(99))))],
        0,
    ));

    let mut v = GrammarValidator::new();
    let r = v.validate(&g);

    assert!(
        has_error(&r, |e| matches!(
            e,
            ValidationError::UndefinedSymbol { symbol, .. } if *symbol == SymbolId(99)
        )),
        "expected UndefinedSymbol inside Optional, got: {:?}",
        r.errors
    );
}

#[test]
fn repeat_with_undefined_inner_detected() {
    let mut g = Grammar::new("rep_undef".into());

    g.add_rule(make_rule(
        0,
        vec![Symbol::Repeat(Box::new(Symbol::NonTerminal(SymbolId(77))))],
        0,
    ));

    let mut v = GrammarValidator::new();
    let r = v.validate(&g);

    assert!(
        has_error(&r, |e| matches!(e, ValidationError::UndefinedSymbol { .. })),
        "expected UndefinedSymbol inside Repeat, got: {:?}",
        r.errors
    );
}

#[test]
fn choice_with_undefined_branch_detected() {
    let mut g = Grammar::new("choice_undef".into());
    let (tid, tok) = make_token(1, "a", "a");
    g.tokens.insert(tid, tok);

    g.add_rule(make_rule(
        0,
        vec![Symbol::Choice(vec![
            Symbol::Terminal(SymbolId(1)),
            Symbol::NonTerminal(SymbolId(88)),
        ])],
        0,
    ));

    let mut v = GrammarValidator::new();
    let r = v.validate(&g);

    assert!(
        has_error(&r, |e| matches!(
            e,
            ValidationError::UndefinedSymbol { symbol, .. } if *symbol == SymbolId(88)
        )),
        "expected UndefinedSymbol inside Choice, got: {:?}",
        r.errors
    );
}

#[test]
fn sequence_with_undefined_element_detected() {
    let mut g = Grammar::new("seq_undef".into());
    let (tid, tok) = make_token(1, "a", "a");
    g.tokens.insert(tid, tok);

    g.add_rule(make_rule(
        0,
        vec![Symbol::Sequence(vec![
            Symbol::Terminal(SymbolId(1)),
            Symbol::Terminal(SymbolId(55)),
        ])],
        0,
    ));

    let mut v = GrammarValidator::new();
    let r = v.validate(&g);

    assert!(
        has_error(&r, |e| matches!(
            e,
            ValidationError::UndefinedSymbol { symbol, .. } if *symbol == SymbolId(55)
        )),
        "expected UndefinedSymbol inside Sequence, got: {:?}",
        r.errors
    );
}

// ---------------------------------------------------------------------------
// Epsilon-only rule
// ---------------------------------------------------------------------------

#[test]
fn epsilon_only_rule_validates_without_panic() {
    let mut g = Grammar::new("eps".into());

    g.add_rule(make_rule(0, vec![Symbol::Epsilon], 0));

    let mut v = GrammarValidator::new();
    let _r = v.validate(&g);
    // Validation should not panic on an epsilon-only grammar.
}

// ---------------------------------------------------------------------------
// Validation statistics
// ---------------------------------------------------------------------------

#[test]
fn stats_reflect_grammar_structure() {
    let mut g = Grammar::new("stats".into());
    let (tid, tok) = make_token(1, "x", "x");
    g.tokens.insert(tid, tok);

    // Two rules for same LHS
    g.add_rule(make_rule(0, vec![Symbol::Terminal(SymbolId(1))], 0));
    g.add_rule(make_rule(
        0,
        vec![Symbol::Terminal(SymbolId(1)), Symbol::Terminal(SymbolId(1))],
        1,
    ));

    let mut v = GrammarValidator::new();
    let r = v.validate(&g);

    assert_eq!(r.stats.total_tokens, 1);
    assert_eq!(r.stats.total_rules, 2);
    assert_eq!(r.stats.max_rule_length, 2);
}

// ---------------------------------------------------------------------------
// Validator is reusable across multiple grammars
// ---------------------------------------------------------------------------

#[test]
fn validator_reuse_clears_state() {
    let mut v = GrammarValidator::new();

    // First: empty grammar -> error
    let g1 = Grammar::new("g1".into());
    let r1 = v.validate(&g1);
    assert!(!r1.errors.is_empty());

    // Second: valid grammar -> no errors
    let mut g2 = Grammar::new("g2".into());
    let (tid, tok) = make_token(1, "x", "x");
    g2.tokens.insert(tid, tok);
    g2.add_rule(make_rule(0, vec![Symbol::Terminal(SymbolId(1))], 0));
    let r2 = v.validate(&g2);
    assert!(r2.errors.is_empty(), "errors leaked: {:?}", r2.errors);
}
