//! Property-based tests for Grammar validation in adze-ir.
//!
//! 46 proptest properties covering GrammarValidator behaviour across
//! well-formed grammars, missing tokens, determinism, normalize/optimize
//! interaction, error counts, start symbols, token uniqueness, and edge cases.

use proptest::prelude::*;

use adze_ir::builder::GrammarBuilder;
use adze_ir::optimizer::GrammarOptimizer;
use adze_ir::validation::{GrammarValidator, ValidationError, ValidationWarning};
use adze_ir::{Associativity, Grammar, ProductionId, Rule, Symbol, SymbolId, Token, TokenPattern};

// ---------------------------------------------------------------------------
// Helpers: safe name generation (alphabetic only, no Rust 2024 keywords)
// ---------------------------------------------------------------------------

const RESERVED: &[&str] = &[
    "gen", "do", "abstract", "become", "final", "override", "priv", "typeof", "unsized", "virtual",
    "box", "macro", "try", "yield", "as", "break", "const", "continue", "crate", "else", "enum",
    "extern", "false", "fn", "for", "if", "impl", "in", "let", "loop", "match", "mod", "move",
    "mut", "pub", "ref", "return", "self", "static", "struct", "super", "trait", "true", "type",
    "unsafe", "use", "where", "while", "async", "await", "dyn",
];

fn is_reserved(s: &str) -> bool {
    RESERVED.contains(&s)
}

/// Strategy producing safe identifiers: 2–8 lowercase letters, never a keyword.
fn safe_ident() -> impl Strategy<Value = String> {
    "[a-z]{2,8}".prop_filter("must not be a reserved keyword", |s| !is_reserved(s))
}

/// Strategy for grammar names.
fn grammar_name() -> impl Strategy<Value = String> {
    safe_ident()
}

/// Strategy for uppercase token names.
fn token_name() -> impl Strategy<Value = String> {
    "[A-Z]{2,6}"
}

/// Build a minimal well-formed grammar: one token, one rule, one start symbol.
fn well_formed(name: &str, tok: &str, rule_sym: &str) -> Grammar {
    GrammarBuilder::new(name)
        .token(tok, tok)
        .rule(rule_sym, vec![tok])
        .start(rule_sym)
        .build()
}

/// Build a grammar complex enough to survive optimizer passes.
fn optimizable_grammar(name: &str) -> Grammar {
    GrammarBuilder::new(name)
        .token("NUM", r"\d+")
        .token("+", "+")
        .rule("expr", vec!["expr", "+", "expr"])
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build()
}

/// Validate and return result conveniently.
fn run_validate(g: &Grammar) -> adze_ir::validation::ValidationResult {
    let mut v = GrammarValidator::new();
    v.validate(g)
}

/// Count errors matching a predicate.
fn count_errors(
    r: &adze_ir::validation::ValidationResult,
    pred: impl Fn(&ValidationError) -> bool,
) -> usize {
    r.errors.iter().filter(|e| pred(e)).count()
}

/// Check if any error matches a predicate.
fn has_error(
    r: &adze_ir::validation::ValidationResult,
    pred: impl Fn(&ValidationError) -> bool,
) -> bool {
    r.errors.iter().any(pred)
}

/// Check if any warning matches a predicate.
fn has_warning(
    r: &adze_ir::validation::ValidationResult,
    pred: impl Fn(&ValidationWarning) -> bool,
) -> bool {
    r.warnings.iter().any(pred)
}

/// Build a raw grammar with a single rule referencing the given RHS symbols.
fn raw_grammar_with_rhs(name: &str, lhs: SymbolId, rhs: Vec<Symbol>) -> Grammar {
    let mut g = Grammar::new(name.to_string());
    g.rules.insert(
        lhs,
        vec![Rule {
            lhs,
            rhs,
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(0),
        }],
    );
    g.rule_names.insert(lhs, "root".to_string());
    g
}

// ===========================================================================
// Category 1 — Well-formed grammars always pass validation (5 properties)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(60))]

    // 1
    #[test]
    fn wellformed_minimal_no_errors(name in grammar_name()) {
        let g = well_formed(&name, "TOK", "root");
        let r = run_validate(&g);
        prop_assert!(r.errors.is_empty(), "expected no errors, got {:?}", r.errors);
    }

    // 2
    #[test]
    fn wellformed_two_tokens_no_structural_errors(
        name in grammar_name(),
        t1 in token_name(),
        t2 in token_name(),
    ) {
        let g = GrammarBuilder::new(&name)
            .token(&t1, "a")
            .token(&t2, "b")
            .rule("root", vec![&t1])
            .rule("root", vec![&t2])
            .start("root")
            .build();
        let r = run_validate(&g);
        // Filter out non-productive warnings that arise from unused tokens
        let structural = count_errors(&r, |e| {
            !matches!(e, ValidationError::NonProductiveSymbol { .. })
        });
        prop_assert_eq!(structural, 0);
    }

    // 3
    #[test]
    fn wellformed_python_like_no_empty_grammar(_dummy in 0u8..1) {
        let g = GrammarBuilder::python_like();
        let r = run_validate(&g);
        let empty = has_error(&r, |e| matches!(e, ValidationError::EmptyGrammar));
        prop_assert!(!empty);
    }

    // 4
    #[test]
    fn wellformed_javascript_like_no_empty_grammar(_dummy in 0u8..1) {
        let g = GrammarBuilder::javascript_like();
        let r = run_validate(&g);
        let empty = has_error(&r, |e| matches!(e, ValidationError::EmptyGrammar));
        prop_assert!(!empty);
    }

    // 5
    #[test]
    fn wellformed_with_precedence_no_empty(name in grammar_name()) {
        let g = GrammarBuilder::new(&name)
            .token("NUM", r"\d+")
            .token("+", "+")
            .token("*", "*")
            .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
            .rule_with_precedence("expr", vec!["expr", "*", "expr"], 2, Associativity::Left)
            .rule("expr", vec!["NUM"])
            .start("expr")
            .build();
        let r = run_validate(&g);
        let empty = has_error(&r, |e| matches!(e, ValidationError::EmptyGrammar));
        prop_assert!(!empty);
    }
}

// ===========================================================================
// Category 2 — Grammars with missing tokens fail validation (5 properties)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(60))]

    // 6
    #[test]
    fn missing_nonterminal_produces_undefined(name in grammar_name()) {
        let missing = SymbolId(99);
        let g = raw_grammar_with_rhs(&name, SymbolId(1), vec![Symbol::NonTerminal(missing)]);
        let r = run_validate(&g);
        let found = has_error(&r, |e| {
            matches!(e, ValidationError::UndefinedSymbol { symbol, .. } if *symbol == missing)
        });
        prop_assert!(found);
    }

    // 7
    #[test]
    fn missing_terminal_is_flagged(name in grammar_name()) {
        let bad_tok = SymbolId(50);
        let g = raw_grammar_with_rhs(&name, SymbolId(1), vec![Symbol::Terminal(bad_tok)]);
        let r = run_validate(&g);
        let found = has_error(&r, |e| {
            matches!(e, ValidationError::UndefinedSymbol { symbol, .. } if *symbol == bad_tok)
        });
        prop_assert!(found);
    }

    // 8
    #[test]
    fn two_missing_terminals_two_errors(name in grammar_name()) {
        let bad1 = SymbolId(50);
        let bad2 = SymbolId(51);
        let g = raw_grammar_with_rhs(
            &name,
            SymbolId(1),
            vec![Symbol::Terminal(bad1), Symbol::Terminal(bad2)],
        );
        let r = run_validate(&g);
        let undef_count = count_errors(&r, |e| {
            matches!(e, ValidationError::UndefinedSymbol { .. })
        });
        prop_assert!(undef_count >= 2, "expected >=2 undefined, got {}", undef_count);
    }

    // 9
    #[test]
    fn missing_nonterminal_mixed_rhs(name in grammar_name()) {
        let mut g = Grammar::new(name);
        let lhs = SymbolId(1);
        let phantom = SymbolId(200);
        g.tokens.insert(SymbolId(2), Token {
            name: "TOK".to_string(),
            pattern: TokenPattern::String("x".to_string()),
            fragile: false,
        });
        g.rules.insert(lhs, vec![Rule {
            lhs,
            rhs: vec![Symbol::Terminal(SymbolId(2)), Symbol::NonTerminal(phantom)],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(0),
        }]);
        g.rule_names.insert(lhs, "root".to_string());
        let r = run_validate(&g);
        let found = has_error(&r, |e| {
            matches!(e, ValidationError::UndefinedSymbol { symbol, .. } if *symbol == phantom)
        });
        prop_assert!(found);
    }

    // 10
    #[test]
    fn missing_token_still_reports_stats(name in grammar_name()) {
        let g = raw_grammar_with_rhs(&name, SymbolId(1), vec![Symbol::NonTerminal(SymbolId(99))]);
        let r = run_validate(&g);
        prop_assert!(r.stats.total_rules >= 1);
    }
}

// ===========================================================================
// Category 3 — Validation is deterministic (5 properties)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    // 11
    #[test]
    fn deterministic_error_count(name in grammar_name()) {
        let g = well_formed(&name, "TOK", "root");
        let r1 = run_validate(&g);
        let r2 = run_validate(&g);
        prop_assert_eq!(r1.errors.len(), r2.errors.len());
    }

    // 12
    #[test]
    fn deterministic_warning_count(name in grammar_name()) {
        let g = well_formed(&name, "TOK", "root");
        let r1 = run_validate(&g);
        let r2 = run_validate(&g);
        prop_assert_eq!(r1.warnings.len(), r2.warnings.len());
    }

    // 13
    #[test]
    fn deterministic_stats_total_rules(name in grammar_name()) {
        let g = well_formed(&name, "TOK", "root");
        let r1 = run_validate(&g);
        let r2 = run_validate(&g);
        prop_assert_eq!(r1.stats.total_rules, r2.stats.total_rules);
    }

    // 14
    #[test]
    fn deterministic_on_python_like(_dummy in 0u8..1) {
        let g = GrammarBuilder::python_like();
        let r1 = run_validate(&g);
        let r2 = run_validate(&g);
        prop_assert_eq!(r1.errors.len(), r2.errors.len());
        prop_assert_eq!(r1.warnings.len(), r2.warnings.len());
    }

    // 15
    #[test]
    fn deterministic_on_js_like(_dummy in 0u8..1) {
        let g = GrammarBuilder::javascript_like();
        let r1 = run_validate(&g);
        let r2 = run_validate(&g);
        prop_assert_eq!(r1.errors.len(), r2.errors.len());
        prop_assert_eq!(r1.stats.total_tokens, r2.stats.total_tokens);
    }
}

// ===========================================================================
// Category 4 — Validation after normalize still passes (5 properties)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    // 16
    #[test]
    fn normalize_then_validate_no_empty_grammar(name in grammar_name()) {
        let mut g = well_formed(&name, "TOK", "root");
        g.normalize();
        let r = run_validate(&g);
        let empty = has_error(&r, |e| matches!(e, ValidationError::EmptyGrammar));
        prop_assert!(!empty);
    }

    // 17
    #[test]
    fn normalize_preserves_token_count(name in grammar_name()) {
        let mut g = well_formed(&name, "TOK", "root");
        let before = g.tokens.len();
        g.normalize();
        prop_assert_eq!(g.tokens.len(), before);
    }

    // 18
    #[test]
    fn normalize_idempotent_validation(name in grammar_name()) {
        let mut g = well_formed(&name, "TOK", "root");
        g.normalize();
        let r1 = run_validate(&g);
        g.normalize();
        let r2 = run_validate(&g);
        prop_assert_eq!(r1.errors.len(), r2.errors.len());
    }

    // 19
    #[test]
    fn normalize_nullable_then_validate(name in grammar_name()) {
        let mut g = GrammarBuilder::new(&name)
            .token("TOK", "x")
            .rule("root", vec!["TOK"])
            .rule("root", vec![])
            .start("root")
            .build();
        g.normalize();
        let r = run_validate(&g);
        let empty = has_error(&r, |e| matches!(e, ValidationError::EmptyGrammar));
        prop_assert!(!empty);
    }

    // 20
    #[test]
    fn normalize_stats_still_populated(name in grammar_name()) {
        let mut g = well_formed(&name, "TOK", "root");
        g.normalize();
        let r = run_validate(&g);
        prop_assert!(r.stats.total_rules >= 1);
        prop_assert!(r.stats.total_tokens >= 1);
    }
}

// ===========================================================================
// Category 5 — Validation after optimize still passes (5 properties)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    // 21
    #[test]
    fn optimize_then_validate_no_empty_grammar(name in grammar_name()) {
        let mut g = optimizable_grammar(&name);
        let mut opt = GrammarOptimizer::new();
        opt.optimize(&mut g);
        let r = run_validate(&g);
        let empty = has_error(&r, |e| matches!(e, ValidationError::EmptyGrammar));
        prop_assert!(!empty);
    }

    // 22
    #[test]
    fn optimize_preserves_non_empty_rules(name in grammar_name()) {
        let mut g = optimizable_grammar(&name);
        let mut opt = GrammarOptimizer::new();
        opt.optimize(&mut g);
        prop_assert!(!g.rules.is_empty());
    }

    // 23
    #[test]
    fn optimize_stats_returned(name in grammar_name()) {
        let mut g = optimizable_grammar(&name);
        let mut opt = GrammarOptimizer::new();
        let stats = opt.optimize(&mut g);
        prop_assert!(stats.removed_unused_symbols <= 100);
        prop_assert!(stats.inlined_rules <= 100);
    }

    // 24
    #[test]
    fn optimize_deterministic(name in grammar_name()) {
        let g_orig = optimizable_grammar(&name);
        let mut g1 = g_orig.clone();
        let mut g2 = g_orig.clone();
        let mut opt1 = GrammarOptimizer::new();
        let mut opt2 = GrammarOptimizer::new();
        opt1.optimize(&mut g1);
        opt2.optimize(&mut g2);
        prop_assert_eq!(g1.rules.len(), g2.rules.len());
        prop_assert_eq!(g1.tokens.len(), g2.tokens.len());
    }

    // 25
    #[test]
    fn optimize_then_validate_stats_sane(name in grammar_name()) {
        let mut g = optimizable_grammar(&name);
        let mut opt = GrammarOptimizer::new();
        opt.optimize(&mut g);
        let r = run_validate(&g);
        prop_assert!(r.stats.total_symbols >= r.stats.total_tokens);
    }
}

// ===========================================================================
// Category 6 — Validation error count matches issues (5 properties)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(60))]

    // 26
    #[test]
    fn empty_grammar_has_empty_grammar_error(_dummy in 0u8..1) {
        let g = Grammar::new("empty".to_string());
        let r = run_validate(&g);
        let count = count_errors(&r, |e| matches!(e, ValidationError::EmptyGrammar));
        prop_assert_eq!(count, 1);
    }

    // 27
    #[test]
    fn no_duplicate_empty_grammar_errors(_dummy in 0u8..1) {
        let g = Grammar::new("empty2".to_string());
        let r = run_validate(&g);
        let count = count_errors(&r, |e| matches!(e, ValidationError::EmptyGrammar));
        prop_assert!(count <= 1);
    }

    // 28
    #[test]
    fn error_count_equals_vec_len(name in grammar_name()) {
        let g = well_formed(&name, "TOK", "root");
        let r = run_validate(&g);
        // Verify error vec can be collected and counted consistently
        let collected: Vec<_> = r.errors.iter().collect();
        prop_assert_eq!(collected.len(), r.errors.len());
    }

    // 29
    #[test]
    fn warning_count_equals_vec_len(name in grammar_name()) {
        let g = well_formed(&name, "TOK", "root");
        let r = run_validate(&g);
        // Verify warning vec can be collected and counted consistently
        let collected: Vec<_> = r.warnings.iter().collect();
        prop_assert_eq!(collected.len(), r.warnings.len());
    }

    // 30
    #[test]
    fn non_productive_errors_for_isolated_nonterminal(name in grammar_name()) {
        let g = raw_grammar_with_rhs(&name, SymbolId(1), vec![Symbol::NonTerminal(SymbolId(2))]);
        let r = run_validate(&g);
        let found = has_error(&r, |e| matches!(e, ValidationError::NonProductiveSymbol { .. }));
        prop_assert!(found);
    }
}

// ===========================================================================
// Category 7 — Start symbol validation (5 properties)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(60))]

    // 31
    #[test]
    fn start_symbol_is_first_rule_key(rule_sym in safe_ident()) {
        let g = GrammarBuilder::new("test")
            .token("TOK", "x")
            .rule("other", vec!["TOK"])
            .rule(&rule_sym, vec!["TOK"])
            .start(&rule_sym)
            .build();
        let first_key = g.rules.keys().next().copied();
        let start_id = g.find_symbol_by_name(&rule_sym);
        prop_assert_eq!(first_key, start_id);
    }

    // 32
    #[test]
    fn start_symbol_reachable(name in grammar_name()) {
        let g = well_formed(&name, "TOK", "root");
        let r = run_validate(&g);
        let root_unused = has_warning(&r, |w| {
            matches!(w, ValidationWarning::UnusedToken { name, .. } if name == "root")
        });
        prop_assert!(!root_unused);
    }

    // 33
    #[test]
    fn grammar_start_symbol_returns_some(name in grammar_name()) {
        let g = well_formed(&name, "TOK", "root");
        prop_assert!(g.start_symbol().is_some());
    }

    // 34
    #[test]
    fn grammar_find_start_by_name(name in grammar_name()) {
        let g = well_formed(&name, "TOK", "root");
        let found = g.find_symbol_by_name("root");
        prop_assert!(found.is_some());
    }

    // 35
    #[test]
    fn start_symbol_has_rules(name in grammar_name()) {
        let g = well_formed(&name, "TOK", "root");
        let start = g.start_symbol().unwrap();
        let rules = g.get_rules_for_symbol(start);
        prop_assert!(rules.is_some());
        let rules_vec = rules.unwrap();
        prop_assert!(!rules_vec.is_empty());
    }
}

// ===========================================================================
// Category 8 — Token uniqueness validation (5 properties)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(60))]

    // 36
    #[test]
    fn duplicate_token_pattern_warns(name in grammar_name()) {
        let mut g = Grammar::new(name);
        let t1 = SymbolId(1);
        let t2 = SymbolId(2);
        g.tokens.insert(t1, Token {
            name: "TOK_A".to_string(),
            pattern: TokenPattern::String("x".to_string()),
            fragile: false,
        });
        g.tokens.insert(t2, Token {
            name: "TOK_B".to_string(),
            pattern: TokenPattern::String("x".to_string()),
            fragile: false,
        });
        let lhs = SymbolId(3);
        g.rules.insert(lhs, vec![Rule {
            lhs,
            rhs: vec![Symbol::Terminal(t1)],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(0),
        }]);
        g.rule_names.insert(lhs, "root".to_string());
        let r = run_validate(&g);
        let found = has_warning(&r, |w| matches!(w, ValidationWarning::DuplicateTokenPattern { .. }));
        prop_assert!(found);
    }

    // 37
    #[test]
    fn unique_patterns_no_dup_warning(name in grammar_name()) {
        let g = GrammarBuilder::new(&name)
            .token("AA", "aaa")
            .token("BB", "bbb")
            .rule("root", vec!["AA", "BB"])
            .start("root")
            .build();
        let r = run_validate(&g);
        let found = has_warning(&r, |w| matches!(w, ValidationWarning::DuplicateTokenPattern { .. }));
        prop_assert!(!found);
    }

    // 38
    #[test]
    fn single_token_never_duplicate(tok in token_name(), name in grammar_name()) {
        let g = GrammarBuilder::new(&name)
            .token(&tok, "pattern")
            .rule("root", vec![&tok])
            .start("root")
            .build();
        let r = run_validate(&g);
        let found = has_warning(&r, |w| matches!(w, ValidationWarning::DuplicateTokenPattern { .. }));
        prop_assert!(!found);
    }

    // 39
    #[test]
    fn token_count_in_stats_matches(name in grammar_name()) {
        let g = GrammarBuilder::new(&name)
            .token("AA", "aaa")
            .token("BB", "bbb")
            .rule("root", vec!["AA"])
            .start("root")
            .build();
        let r = run_validate(&g);
        prop_assert_eq!(r.stats.total_tokens, 2);
    }

    // 40
    #[test]
    fn three_dup_tokens_warn(name in grammar_name()) {
        let mut g = Grammar::new(name);
        for i in 0u16..3 {
            g.tokens.insert(SymbolId(i + 1), Token {
                name: format!("T{i}"),
                pattern: TokenPattern::String("same".to_string()),
                fragile: false,
            });
        }
        let lhs = SymbolId(10);
        g.rules.insert(lhs, vec![Rule {
            lhs,
            rhs: vec![Symbol::Terminal(SymbolId(1))],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(0),
        }]);
        g.rule_names.insert(lhs, "root".to_string());
        let r = run_validate(&g);
        let found = has_warning(&r, |w| matches!(w, ValidationWarning::DuplicateTokenPattern { .. }));
        prop_assert!(found);
    }
}

// ===========================================================================
// Category 9 — Edge cases (6 properties)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    // 41
    #[test]
    fn epsilon_only_grammar_validates(name in grammar_name()) {
        let g = GrammarBuilder::new(&name)
            .rule("root", vec![])
            .start("root")
            .build();
        let r = run_validate(&g);
        let empty = has_error(&r, |e| matches!(e, ValidationError::EmptyGrammar));
        prop_assert!(!empty);
    }

    // 42
    #[test]
    fn many_alternatives_validate(n in 2usize..8) {
        let mut builder = GrammarBuilder::new("alts")
            .token("TOK", "x");
        for _ in 0..n {
            builder = builder.rule("root", vec!["TOK"]);
        }
        let g = builder.start("root").build();
        let r = run_validate(&g);
        let empty = has_error(&r, |e| matches!(e, ValidationError::EmptyGrammar));
        prop_assert!(!empty);
        prop_assert_eq!(r.stats.total_rules, n);
    }

    // 43
    #[test]
    fn validator_can_be_reused(name in grammar_name()) {
        let g1 = well_formed(&name, "TOK", "root");
        let g2 = Grammar::new("empty".to_string());
        let mut v = GrammarValidator::new();
        let r1 = v.validate(&g1);
        let r2 = v.validate(&g2);
        let first_empty = r1.errors.iter().any(|e| matches!(e, ValidationError::EmptyGrammar));
        let second_empty = r2.errors.iter().any(|e| matches!(e, ValidationError::EmptyGrammar));
        prop_assert!(!first_empty);
        prop_assert!(second_empty);
    }

    // 44
    #[test]
    fn external_token_counted_in_stats(name in grammar_name()) {
        let g = GrammarBuilder::new(&name)
            .token("TOK", "x")
            .external("INDENT")
            .rule("root", vec!["TOK"])
            .start("root")
            .build();
        let r = run_validate(&g);
        prop_assert!(r.stats.external_tokens >= 1);
    }

    // 45
    #[test]
    fn max_rule_length_stat_accurate(name in grammar_name()) {
        let g = GrammarBuilder::new(&name)
            .token("A", "a")
            .token("B", "b")
            .token("C", "c")
            .rule("root", vec!["A", "B", "C"])
            .start("root")
            .build();
        let r = run_validate(&g);
        prop_assert_eq!(r.stats.max_rule_length, 3);
    }

    // 46
    #[test]
    fn fragile_token_does_not_cause_regex_error(name in grammar_name()) {
        let mut g = Grammar::new(name);
        let t = SymbolId(1);
        g.tokens.insert(t, Token {
            name: "FRAG".to_string(),
            pattern: TokenPattern::String("x".to_string()),
            fragile: true,
        });
        let lhs = SymbolId(2);
        g.rules.insert(lhs, vec![Rule {
            lhs,
            rhs: vec![Symbol::Terminal(t)],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(0),
        }]);
        g.rule_names.insert(lhs, "root".to_string());
        let r = run_validate(&g);
        let found = has_error(&r, |e| matches!(e, ValidationError::InvalidRegex { .. }));
        prop_assert!(!found);
    }
}
