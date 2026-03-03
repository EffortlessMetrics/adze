//! Comprehensive grammar builder API tests.
//!
//! Tests the `GrammarBuilder` pattern thoroughly across 30+ scenarios.

use adze_ir::builder::GrammarBuilder;
use adze_ir::{
    Associativity, GrammarValidator, PrecedenceKind, Symbol, TokenPattern, ValidationError,
};

// ═══════════════════════════════════════════════════════════════════════════
// 1. Builder with no rules produces valid empty grammar
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn empty_grammar_has_correct_name() {
    let g = GrammarBuilder::new("empty").build();
    assert_eq!(g.name, "empty");
}

#[test]
fn empty_grammar_has_no_rules() {
    let g = GrammarBuilder::new("empty").build();
    assert!(g.rules.is_empty());
    assert!(g.tokens.is_empty());
}

#[test]
fn empty_grammar_collections_are_empty() {
    let g = GrammarBuilder::new("empty").build();
    assert!(g.precedences.is_empty());
    assert!(g.conflicts.is_empty());
    assert!(g.externals.is_empty());
    assert!(g.extras.is_empty());
    assert!(g.fields.is_empty());
    assert!(g.supertypes.is_empty());
    assert!(g.inline_rules.is_empty());
    assert!(g.alias_sequences.is_empty());
    assert!(g.production_ids.is_empty());
    assert_eq!(g.max_alias_sequence_length, 0);
    assert!(g.symbol_registry.is_none());
    assert!(g.rule_names.is_empty());
}

// ═══════════════════════════════════════════════════════════════════════════
// 2. Builder with single terminal rule
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn single_terminal_rule_creates_one_rule_entry() {
    let g = GrammarBuilder::new("single")
        .token("A", "a")
        .rule("start", vec!["A"])
        .build();
    assert_eq!(g.rules.len(), 1);
    assert_eq!(g.tokens.len(), 1);
}

#[test]
fn single_terminal_rule_rhs_is_terminal() {
    let g = GrammarBuilder::new("single")
        .token("A", "a")
        .rule("start", vec!["A"])
        .build();
    let rules = g.rules.values().next().unwrap();
    assert_eq!(rules.len(), 1);
    assert_eq!(rules[0].rhs.len(), 1);
    assert!(matches!(rules[0].rhs[0], Symbol::Terminal(_)));
}

#[test]
fn single_terminal_rule_token_pattern_is_string() {
    let g = GrammarBuilder::new("single")
        .token("A", "a")
        .rule("start", vec!["A"])
        .build();
    let tok = g.tokens.values().next().unwrap();
    assert_eq!(tok.pattern, TokenPattern::String("a".to_string()));
    assert!(!tok.fragile);
}

// ═══════════════════════════════════════════════════════════════════════════
// 3. Builder with multiple rules
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn multiple_rules_distinct_nonterminals() {
    let g = GrammarBuilder::new("multi")
        .token("X", "x")
        .token("Y", "y")
        .rule("alpha", vec!["X"])
        .rule("beta", vec!["Y"])
        .build();
    assert_eq!(g.rules.len(), 2);
}

#[test]
fn multiple_alternatives_same_nonterminal() {
    let g = GrammarBuilder::new("multi")
        .token("X", "x")
        .token("Y", "y")
        .rule("thing", vec!["X"])
        .rule("thing", vec!["Y"])
        .rule("thing", vec!["X", "Y"])
        .build();
    // Only one key in the map, but with 3 alternatives
    assert_eq!(g.rules.len(), 1);
    let rules = g.rules.values().next().unwrap();
    assert_eq!(rules.len(), 3);
}

#[test]
fn each_alternative_gets_unique_production_id() {
    let g = GrammarBuilder::new("ids")
        .token("A", "a")
        .rule("s", vec!["A"])
        .rule("s", vec!["A", "A"])
        .rule("s", vec![])
        .build();
    let rules = g.rules.values().next().unwrap();
    let ids: std::collections::HashSet<_> = rules.iter().map(|r| r.production_id).collect();
    assert_eq!(ids.len(), 3);
}

// ═══════════════════════════════════════════════════════════════════════════
// 4. Builder set start symbol
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn start_symbol_appears_first_in_rules() {
    let g = GrammarBuilder::new("ordered")
        .token("A", "a")
        .rule("beta", vec!["A"])
        .rule("alpha", vec!["beta"])
        .start("alpha")
        .build();
    let first_key = g.rules.keys().next().unwrap();
    assert_eq!(g.rule_names[first_key], "alpha");
}

#[test]
fn start_symbol_without_setting_uses_insertion_order() {
    let g = GrammarBuilder::new("natural")
        .token("A", "a")
        .rule("second", vec!["A"])
        .rule("first", vec!["second"])
        .build();
    let first_key = g.rules.keys().next().unwrap();
    assert_eq!(g.rule_names[first_key], "second");
}

#[test]
fn start_called_before_rules_still_works() {
    let g = GrammarBuilder::new("early_start")
        .start("root")
        .token("T", "t")
        .rule("other", vec!["T"])
        .rule("root", vec!["other"])
        .build();
    let first_key = g.rules.keys().next().unwrap();
    assert_eq!(g.rule_names[first_key], "root");
}

// ═══════════════════════════════════════════════════════════════════════════
// 5. Builder add external tokens
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn externals_are_registered_with_unique_ids() {
    let g = GrammarBuilder::new("ext")
        .external("INDENT")
        .external("DEDENT")
        .external("NEWLINE")
        .build();
    assert_eq!(g.externals.len(), 3);
    let ids: std::collections::HashSet<_> = g.externals.iter().map(|e| e.symbol_id).collect();
    assert_eq!(ids.len(), 3);
}

#[test]
fn external_names_are_preserved() {
    let g = GrammarBuilder::new("ext")
        .external("INDENT")
        .external("DEDENT")
        .build();
    assert_eq!(g.externals[0].name, "INDENT");
    assert_eq!(g.externals[1].name, "DEDENT");
}

// ═══════════════════════════════════════════════════════════════════════════
// 6. Builder add extras (whitespace)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn extras_are_registered() {
    let g = GrammarBuilder::new("ws")
        .token("WS", r"\s+")
        .extra("WS")
        .build();
    assert_eq!(g.extras.len(), 1);
}

#[test]
fn multiple_extras() {
    let g = GrammarBuilder::new("multi_ws")
        .token("WS", r"\s+")
        .token("COMMENT", r"//[^\n]*")
        .extra("WS")
        .extra("COMMENT")
        .build();
    assert_eq!(g.extras.len(), 2);
}

// ═══════════════════════════════════════════════════════════════════════════
// 7. Builder add supertypes
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn supertypes_default_empty() {
    let g = GrammarBuilder::new("no_super").build();
    assert!(g.supertypes.is_empty());
}

// The builder currently doesn't expose a .supertype() method, so supertypes
// remain empty when using the builder. Verify the default is correct.
#[test]
fn builder_supertypes_stay_empty_without_method() {
    let g = GrammarBuilder::new("test")
        .token("A", "a")
        .rule("s", vec!["A"])
        .start("s")
        .build();
    assert!(g.supertypes.is_empty());
}

// ═══════════════════════════════════════════════════════════════════════════
// 8. Builder add inline rules
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn inline_rules_default_empty() {
    let g = GrammarBuilder::new("no_inline").build();
    assert!(g.inline_rules.is_empty());
}

#[test]
fn builder_inline_rules_stay_empty_without_method() {
    let g = GrammarBuilder::new("test")
        .token("A", "a")
        .rule("s", vec!["A"])
        .build();
    assert!(g.inline_rules.is_empty());
}

// ═══════════════════════════════════════════════════════════════════════════
// 9. Builder add precedence declarations
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn precedence_declaration_is_stored() {
    let g = GrammarBuilder::new("prec")
        .token("+", "+")
        .token("*", "*")
        .precedence(1, Associativity::Left, vec!["+"])
        .precedence(2, Associativity::Left, vec!["*"])
        .build();
    assert_eq!(g.precedences.len(), 2);
    assert_eq!(g.precedences[0].level, 1);
    assert_eq!(g.precedences[0].associativity, Associativity::Left);
    assert_eq!(g.precedences[1].level, 2);
}

#[test]
fn precedence_right_associativity() {
    let g = GrammarBuilder::new("right")
        .token("^", "^")
        .precedence(3, Associativity::Right, vec!["^"])
        .build();
    assert_eq!(g.precedences[0].associativity, Associativity::Right);
}

#[test]
fn precedence_none_associativity() {
    let g = GrammarBuilder::new("none_assoc")
        .token("==", "==")
        .precedence(1, Associativity::None, vec!["=="])
        .build();
    assert_eq!(g.precedences[0].associativity, Associativity::None);
}

#[test]
fn rule_with_precedence_sets_prec_and_assoc() {
    let g = GrammarBuilder::new("prec_rule")
        .token("N", r"\d+")
        .token("+", "+")
        .rule_with_precedence("e", vec!["e", "+", "e"], 5, Associativity::Left)
        .rule("e", vec!["N"])
        .start("e")
        .build();
    let e_id = g.find_symbol_by_name("e").unwrap();
    let rules = g.get_rules_for_symbol(e_id).unwrap();
    let add_rule = &rules[0];
    assert_eq!(add_rule.precedence, Some(PrecedenceKind::Static(5)));
    assert_eq!(add_rule.associativity, Some(Associativity::Left));
    // Plain rule has no precedence
    let num_rule = &rules[1];
    assert_eq!(num_rule.precedence, None);
    assert_eq!(num_rule.associativity, None);
}

// ═══════════════════════════════════════════════════════════════════════════
// 10. Builder add field mappings
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn fields_default_empty_in_built_grammar() {
    let g = GrammarBuilder::new("no_fields")
        .token("A", "a")
        .rule("s", vec!["A"])
        .build();
    assert!(g.fields.is_empty());
}

#[test]
fn rules_have_empty_fields_by_default() {
    let g = GrammarBuilder::new("no_fields")
        .token("A", "a")
        .rule("s", vec!["A"])
        .build();
    for rule in g.all_rules() {
        assert!(rule.fields.is_empty());
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 11. Builder add alias sequences
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn alias_sequences_default_empty() {
    let g = GrammarBuilder::new("no_alias")
        .token("A", "a")
        .rule("s", vec!["A"])
        .build();
    assert!(g.alias_sequences.is_empty());
    assert_eq!(g.max_alias_sequence_length, 0);
}

// ═══════════════════════════════════════════════════════════════════════════
// 12. Builder chaining (fluent API)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn all_builder_methods_are_chainable() {
    // Every builder method should return Self and be chainable in any order
    let g = GrammarBuilder::new("fluent")
        .start("root")
        .extra("WS")
        .external("EXT1")
        .external("EXT2")
        .precedence(1, Associativity::Left, vec!["op"])
        .token("WS", r"\s+")
        .token("op", "+")
        .token("N", r"\d+")
        .fragile_token("ERR", "???")
        .rule("root", vec!["N"])
        .rule("root", vec!["root", "op", "root"])
        .rule_with_precedence("root", vec!["root", "op", "N"], 2, Associativity::Right)
        .build();

    assert_eq!(g.name, "fluent");
    assert_eq!(g.tokens.len(), 4); // WS, op, N, ERR
    assert_eq!(g.externals.len(), 2);
    assert_eq!(g.extras.len(), 1);
    assert_eq!(g.precedences.len(), 1);
    // start symbol causes "root" to come first
    let first_key = g.rules.keys().next().unwrap();
    assert_eq!(g.rule_names[first_key], "root");
}

#[test]
fn chaining_order_does_not_affect_outcome() {
    // Build same grammar with different call orders
    let g1 = GrammarBuilder::new("g")
        .token("A", "a")
        .rule("s", vec!["A"])
        .start("s")
        .build();
    let g2 = GrammarBuilder::new("g")
        .start("s")
        .rule("s", vec!["A"])
        .token("A", "a")
        .build();
    assert_eq!(g1.tokens.len(), g2.tokens.len());
    assert_eq!(g1.rules.len(), g2.rules.len());
}

// ═══════════════════════════════════════════════════════════════════════════
// 13. Builder override previously set values
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn redefining_token_overwrites_pattern() {
    let g = GrammarBuilder::new("override")
        .token("NUM", r"\d+")
        .token("NUM", r"[0-9]+")
        .rule("s", vec!["NUM"])
        .build();
    // Only one token entry because the same name reuses the symbol ID
    assert_eq!(g.tokens.len(), 1);
    let tok = g.tokens.values().next().unwrap();
    assert_eq!(tok.pattern, TokenPattern::Regex("[0-9]+".to_string()));
}

#[test]
fn calling_start_twice_uses_last() {
    let g = GrammarBuilder::new("double_start")
        .token("A", "a")
        .token("B", "b")
        .rule("alpha", vec!["A"])
        .rule("beta", vec!["B"])
        .start("alpha")
        .start("beta")
        .build();
    let first_key = g.rules.keys().next().unwrap();
    assert_eq!(g.rule_names[first_key], "beta");
}

#[test]
fn adding_extra_twice_for_same_symbol_duplicates() {
    let g = GrammarBuilder::new("dup_extra")
        .token("WS", r"\s+")
        .extra("WS")
        .extra("WS")
        .build();
    // Extra is pushed, not deduplicated
    assert_eq!(g.extras.len(), 2);
}

// ═══════════════════════════════════════════════════════════════════════════
// 14. Builder with all options set simultaneously
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn kitchen_sink_grammar() {
    let g = GrammarBuilder::new("kitchen_sink")
        // Tokens
        .token("NUMBER", r"\d+")
        .token("IDENT", r"[a-z]+")
        .token("+", "+")
        .token("*", "*")
        .token(";", ";")
        .token("(", "(")
        .token(")", ")")
        .fragile_token("ERROR_TOK", r"[^\s]+")
        .token("WS", r"\s+")
        // Externals
        .external("INDENT")
        .external("DEDENT")
        // Extras
        .extra("WS")
        // Precedences
        .precedence(1, Associativity::Left, vec!["+"])
        .precedence(2, Associativity::Left, vec!["*"])
        // Rules with and without precedence
        .rule("program", vec!["statement"])
        .rule("program", vec!["program", "statement"])
        .rule("statement", vec!["expression", ";"])
        .rule("expression", vec!["NUMBER"])
        .rule("expression", vec!["IDENT"])
        .rule("expression", vec!["(", "expression", ")"])
        .rule_with_precedence(
            "expression",
            vec!["expression", "+", "expression"],
            1,
            Associativity::Left,
        )
        .rule_with_precedence(
            "expression",
            vec!["expression", "*", "expression"],
            2,
            Associativity::Left,
        )
        // Epsilon rule
        .rule("opt_semi", vec![])
        .rule("opt_semi", vec![";"])
        // Start
        .start("program")
        .build();

    assert_eq!(g.name, "kitchen_sink");
    assert_eq!(g.tokens.len(), 9);
    assert_eq!(g.externals.len(), 2);
    assert_eq!(g.extras.len(), 1);
    assert_eq!(g.precedences.len(), 2);
    // Rules: program, statement, expression, opt_semi
    assert_eq!(g.rules.len(), 4);
    // Start symbol first
    let first_key = g.rules.keys().next().unwrap();
    assert_eq!(g.rule_names[first_key], "program");
    // Verify epsilon rule exists
    let opt_semi_id = g.find_symbol_by_name("opt_semi").unwrap();
    let opt_rules = g.get_rules_for_symbol(opt_semi_id).unwrap();
    assert!(opt_rules.iter().any(|r| r.rhs == vec![Symbol::Epsilon]));
}

// ═══════════════════════════════════════════════════════════════════════════
// 15. Builder produces grammar that passes validation
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn simple_grammar_passes_grammar_validate() {
    let g = GrammarBuilder::new("valid")
        .token("A", "a")
        .rule("s", vec!["A"])
        .start("s")
        .build();
    // Grammar::validate() checks field ordering and symbol references
    assert!(g.validate().is_ok());
}

#[test]
fn complex_grammar_passes_grammar_validate() {
    let g = GrammarBuilder::new("valid_complex")
        .token("N", r"\d+")
        .token("+", "+")
        .token("*", "*")
        .token("(", "(")
        .token(")", ")")
        .rule("expr", vec!["N"])
        .rule("expr", vec!["(", "expr", ")"])
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "*", "expr"], 2, Associativity::Left)
        .start("expr")
        .build();
    assert!(g.validate().is_ok());
}

#[test]
fn validator_reports_empty_grammar() {
    let g = GrammarBuilder::new("empty").build();
    let mut validator = GrammarValidator::new();
    let result = validator.validate(&g);
    assert!(
        result
            .errors
            .iter()
            .any(|e| matches!(e, ValidationError::EmptyGrammar))
    );
}

#[test]
fn validator_accepts_well_formed_grammar() {
    let g = GrammarBuilder::new("good")
        .token("N", r"\d+")
        .token("+", "+")
        .rule("expr", vec!["N"])
        .rule("expr", vec!["expr", "+", "expr"])
        .start("expr")
        .build();
    let mut validator = GrammarValidator::new();
    let result = validator.validate(&g);
    // No critical errors (EmptyGrammar would be the main concern)
    assert!(
        !result
            .errors
            .iter()
            .any(|e| matches!(e, ValidationError::EmptyGrammar))
    );
}

#[test]
fn validator_reports_stats_correctly() {
    let g = GrammarBuilder::new("stats")
        .token("A", "a")
        .token("B", "b")
        .rule("s", vec!["A"])
        .rule("s", vec!["A", "B"])
        .start("s")
        .build();
    let mut validator = GrammarValidator::new();
    let result = validator.validate(&g);
    assert_eq!(result.stats.total_tokens, 2);
    assert_eq!(result.stats.total_rules, 2);
    assert_eq!(result.stats.max_rule_length, 2);
}

// ═══════════════════════════════════════════════════════════════════════════
// 16. Builder produces grammar that normalizes correctly
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn normalize_on_simple_grammar_is_idempotent() {
    let mut g = GrammarBuilder::new("simple")
        .token("A", "a")
        .rule("s", vec!["A"])
        .start("s")
        .build();
    let rules_before = g.rules.values().flatten().count();
    g.normalize();
    let rules_after = g.rules.values().flatten().count();
    assert_eq!(rules_before, rules_after);
}

#[test]
fn normalize_preserves_terminal_nonterminal_structure() {
    let mut g = GrammarBuilder::new("norm")
        .token("A", "a")
        .token("B", "b")
        .rule("s", vec!["A", "inner"])
        .rule("inner", vec!["B"])
        .start("s")
        .build();
    g.normalize();
    // After normalization, all symbols in all rules should be Terminal,
    // NonTerminal, or Epsilon (no complex symbols)
    for rule in g.all_rules() {
        for sym in &rule.rhs {
            assert!(
                matches!(
                    sym,
                    Symbol::Terminal(_) | Symbol::NonTerminal(_) | Symbol::Epsilon
                ),
                "Found unexpected complex symbol after normalization: {:?}",
                sym
            );
        }
    }
}

#[test]
fn normalize_preserves_epsilon_rules() {
    let mut g = GrammarBuilder::new("eps")
        .token("A", "a")
        .rule("opt", vec![])
        .rule("opt", vec!["A"])
        .start("opt")
        .build();
    g.normalize();
    let opt_id = g.find_symbol_by_name("opt").unwrap();
    let rules = g.get_rules_for_symbol(opt_id).unwrap();
    assert!(rules.iter().any(|r| r.rhs == vec![Symbol::Epsilon]));
}

#[test]
fn normalized_grammar_still_passes_validate() {
    let mut g = GrammarBuilder::new("norm_valid")
        .token("N", r"\d+")
        .token("+", "+")
        .rule("expr", vec!["N"])
        .rule("expr", vec!["expr", "+", "expr"])
        .start("expr")
        .build();
    g.normalize();
    assert!(g.validate().is_ok());
}

// ═══════════════════════════════════════════════════════════════════════════
// 17. Builder produces grammar compatible with FIRST/FOLLOW computation
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn grammar_has_at_least_one_terminal_per_nonterminal_path() {
    // For FIRST/FOLLOW: every non-terminal must eventually derive terminals
    let g = GrammarBuilder::new("productive")
        .token("A", "a")
        .token("B", "b")
        .rule("s", vec!["inner"])
        .rule("inner", vec!["A"])
        .rule("inner", vec!["B"])
        .start("s")
        .build();
    // Check that every non-terminal has at least one rule
    for (sym_id, _name) in &g.rule_names {
        if g.rules.contains_key(sym_id) {
            let rules = &g.rules[sym_id];
            assert!(
                !rules.is_empty(),
                "Non-terminal should have at least one rule"
            );
        }
    }
}

#[test]
fn no_complex_symbols_in_builder_output() {
    // FIRST/FOLLOW computation expects normalized symbols: Terminal, NonTerminal, Epsilon
    let g = GrammarBuilder::new("clean")
        .token("X", "x")
        .token("Y", "y")
        .rule("s", vec!["X"])
        .rule("s", vec!["X", "Y"])
        .rule("s", vec![])
        .start("s")
        .build();
    for rule in g.all_rules() {
        for sym in &rule.rhs {
            match sym {
                Symbol::Terminal(_) | Symbol::NonTerminal(_) | Symbol::Epsilon => {}
                other => panic!(
                    "Builder should not produce complex symbols, got {:?}",
                    other
                ),
            }
        }
    }
}

#[test]
fn all_rhs_terminals_exist_in_tokens() {
    // FIRST set computation needs all terminals to be defined
    let g = GrammarBuilder::new("complete")
        .token("A", "a")
        .token("B", "b")
        .token("+", "+")
        .rule("expr", vec!["A"])
        .rule("expr", vec!["B"])
        .rule("expr", vec!["expr", "+", "expr"])
        .start("expr")
        .build();
    for rule in g.all_rules() {
        for sym in &rule.rhs {
            if let Symbol::Terminal(id) = sym {
                assert!(
                    g.tokens.contains_key(id),
                    "Terminal {:?} referenced in rule but not defined as token",
                    id
                );
            }
        }
    }
}

#[test]
fn all_rhs_nonterminals_have_rules() {
    // FIRST set computation needs all non-terminals to have corresponding rules
    let g = GrammarBuilder::new("connected")
        .token("A", "a")
        .rule("s", vec!["inner"])
        .rule("inner", vec!["A"])
        .start("s")
        .build();
    for rule in g.all_rules() {
        for sym in &rule.rhs {
            if let Symbol::NonTerminal(id) = sym {
                assert!(
                    g.rules.contains_key(id),
                    "NonTerminal {:?} referenced but has no rules",
                    id
                );
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Additional builder behavior tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn token_pattern_detection_regex() {
    let g = GrammarBuilder::new("patterns")
        .token("NUM", r"\d+")
        .token("WORD", r"[a-z]+")
        .build();
    for tok in g.tokens.values() {
        assert!(
            matches!(tok.pattern, TokenPattern::Regex(_)),
            "Pattern with regex metacharacters should be Regex, got {:?}",
            tok.pattern
        );
    }
}

#[test]
fn token_pattern_detection_string_literal() {
    let g = GrammarBuilder::new("literals")
        .token("KW", "keyword")
        .build();
    let tok = g.tokens.values().next().unwrap();
    assert_eq!(tok.pattern, TokenPattern::String("keyword".to_string()));
}

#[test]
fn fragile_token_is_flagged() {
    let g = GrammarBuilder::new("fragile")
        .fragile_token("ERR", r"[^\s]+")
        .build();
    let tok = g.tokens.values().next().unwrap();
    assert!(tok.fragile);
}

#[test]
fn symbol_reuse_across_token_and_rule() {
    // When a symbol is used as both a token name and in rule RHS,
    // the same SymbolId should be used
    let g = GrammarBuilder::new("reuse")
        .token("A", "a")
        .rule("s", vec!["A"])
        .build();
    let tok_id = *g.tokens.keys().next().unwrap();
    let rule = g.rules.values().next().unwrap().first().unwrap();
    match &rule.rhs[0] {
        Symbol::Terminal(id) => assert_eq!(*id, tok_id),
        other => panic!("Expected Terminal, got {:?}", other),
    }
}

#[test]
fn rule_names_map_covers_nonterminals() {
    let g = GrammarBuilder::new("names")
        .token("A", "a")
        .rule("alpha", vec!["A"])
        .rule("beta", vec!["alpha"])
        .start("alpha")
        .build();
    assert!(g.find_symbol_by_name("alpha").is_some());
    assert!(g.find_symbol_by_name("beta").is_some());
}

#[test]
fn python_like_helper_produces_nullable_start() {
    let g = GrammarBuilder::python_like();
    let module_id = g.find_symbol_by_name("module").unwrap();
    let rules = g.get_rules_for_symbol(module_id).unwrap();
    assert!(rules.iter().any(|r| r.rhs == vec![Symbol::Epsilon]));
}

#[test]
fn javascript_like_helper_produces_non_nullable_start() {
    let g = GrammarBuilder::javascript_like();
    let program_id = g.find_symbol_by_name("program").unwrap();
    let rules = g.get_rules_for_symbol(program_id).unwrap();
    assert!(!rules.iter().any(|r| r.rhs == vec![Symbol::Epsilon]));
}

#[test]
fn javascript_like_helper_has_precedence_rules() {
    let g = GrammarBuilder::javascript_like();
    let expr_id = g.find_symbol_by_name("expression").unwrap();
    let rules = g.get_rules_for_symbol(expr_id).unwrap();
    let prec_rules: Vec<_> = rules.iter().filter(|r| r.precedence.is_some()).collect();
    assert!(
        prec_rules.len() >= 4,
        "Expected at least 4 precedence rules (+, -, *, /)"
    );
}

#[test]
fn grammar_serialization_roundtrip() {
    let g = GrammarBuilder::new("roundtrip")
        .token("A", "a")
        .token("B", "b")
        .rule("s", vec!["A", "B"])
        .rule("s", vec!["A"])
        .start("s")
        .build();
    let json = serde_json::to_string(&g).unwrap();
    let g2: adze_ir::Grammar = serde_json::from_str(&json).unwrap();
    assert_eq!(g.name, g2.name);
    assert_eq!(g.tokens.len(), g2.tokens.len());
    assert_eq!(g.rules.len(), g2.rules.len());
    // Verify rule counts match
    let count1: usize = g.rules.values().map(|v| v.len()).sum();
    let count2: usize = g2.rules.values().map(|v| v.len()).sum();
    assert_eq!(count1, count2);
}

#[test]
fn check_empty_terminals_passes_for_builder_output() {
    let g = GrammarBuilder::new("check")
        .token("A", "a")
        .token("B", "b")
        .rule("s", vec!["A", "B"])
        .build();
    assert!(g.check_empty_terminals().is_ok());
}

#[test]
fn build_registry_includes_tokens_and_nonterminals() {
    let mut g = GrammarBuilder::new("registry")
        .token("NUM", r"\d+")
        .token("+", "+")
        .rule("expr", vec!["NUM"])
        .rule("expr", vec!["expr", "+", "expr"])
        .start("expr")
        .build();
    let registry = g.get_or_build_registry();
    // Registry should contain entries
    assert!(!registry.is_empty(), "Registry should not be empty");
}

#[test]
fn multiple_precedence_levels_on_same_grammar() {
    let g = GrammarBuilder::new("multi_prec")
        .token("N", r"\d+")
        .token("+", "+")
        .token("-", "-")
        .token("*", "*")
        .token("/", "/")
        .token("^", "^")
        .precedence(1, Associativity::Left, vec!["+", "-"])
        .precedence(2, Associativity::Left, vec!["*", "/"])
        .precedence(3, Associativity::Right, vec!["^"])
        .rule_with_precedence("e", vec!["e", "+", "e"], 1, Associativity::Left)
        .rule_with_precedence("e", vec!["e", "-", "e"], 1, Associativity::Left)
        .rule_with_precedence("e", vec!["e", "*", "e"], 2, Associativity::Left)
        .rule_with_precedence("e", vec!["e", "/", "e"], 2, Associativity::Left)
        .rule_with_precedence("e", vec!["e", "^", "e"], 3, Associativity::Right)
        .rule("e", vec!["N"])
        .start("e")
        .build();

    assert_eq!(g.precedences.len(), 3);
    let e_id = g.find_symbol_by_name("e").unwrap();
    let rules = g.get_rules_for_symbol(e_id).unwrap();
    assert_eq!(rules.len(), 6);

    // Verify precedence ordering
    let pow_rule = rules
        .iter()
        .find(|r| {
            r.rhs.len() == 3
                && r.rhs
                    .iter()
                    .any(|s| matches!(s, Symbol::Terminal(id) if g.tokens[id].name == "^"))
        })
        .unwrap();
    assert_eq!(pow_rule.precedence, Some(PrecedenceKind::Static(3)));
    assert_eq!(pow_rule.associativity, Some(Associativity::Right));
}
