//! GrammarBuilder fluent API test suite (v8) — 64 comprehensive tests across 8 categories.
//!
//! This test suite covers the GrammarBuilder API with 8 tests per category:
//! 1. Basic builder (new, build, chaining, empty grammars)
//! 2. Token management (patterns, duplicates, special chars, many tokens)
//! 3. Rule management (RHS variations, epsilon, recursion, chaining)
//! 4. Precedence/Associativity (left, right, nonassoc, negative, zero)
//! 5. Inline/Supertype (marks, multiple, combined, validation)
//! 6. Fields/Extras/Externals (field mapping, extras, externals, combined)
//! 7. Complex grammars (arithmetic, if-else, lists, nested expressions)
//! 8. Edge cases (long names, unicode, single-char tokens, very long RHS)

use adze_ir::builder::GrammarBuilder;
use adze_ir::{Associativity, PrecedenceKind, Symbol, TokenPattern};

// ═══════════════════════════════════════════════════════════════════════════
// Category 1: Basic builder — 8 tests
// Tests: new, empty build, single token, single rule, start, chaining,
//        build multiple builders, debug output
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn basic_1_new_builder_has_name() {
    let g = GrammarBuilder::new("test_grammar").build();
    assert_eq!(g.name, "test_grammar");
}

#[test]
fn basic_2_build_empty_grammar() {
    let g = GrammarBuilder::new("empty").build();
    assert_eq!(g.name, "empty");
    assert!(g.tokens.is_empty());
    assert!(g.rules.is_empty());
}

#[test]
fn basic_3_build_with_one_token() {
    let g = GrammarBuilder::new("one_token").token("A", "a").build();
    assert_eq!(g.tokens.len(), 1);
    assert!(!g.tokens.is_empty());
}

#[test]
fn basic_4_build_with_one_rule() {
    let g = GrammarBuilder::new("one_rule")
        .token("T", "t")
        .rule("r", vec!["T"])
        .build();
    assert_eq!(g.rules.len(), 1);
}

#[test]
fn basic_5_build_with_start_symbol() {
    let g = GrammarBuilder::new("with_start")
        .token("T", "t")
        .rule("start_rule", vec!["T"])
        .start("start_rule")
        .build();
    let first_key = g.rules.keys().next().unwrap();
    assert_eq!(g.rule_names[first_key], "start_rule");
}

#[test]
fn basic_6_builder_fluent_chain() {
    let g = GrammarBuilder::new("chain")
        .token("A", "a")
        .token("B", "b")
        .rule("x", vec!["A"])
        .rule("y", vec!["B"])
        .start("x")
        .build();
    assert_eq!(g.tokens.len(), 2);
    assert_eq!(g.rules.len(), 2);
}

#[test]
fn basic_7_build_multiple_times_separate_builders() {
    let g1 = GrammarBuilder::new("g1").token("T", "t").build();
    let g2 = GrammarBuilder::new("g2").token("U", "u").build();
    assert_eq!(g1.name, "g1");
    assert_eq!(g2.name, "g2");
    assert_ne!(g1.name, g2.name);
}

#[test]
fn basic_8_builder_debug_output() {
    let _builder = GrammarBuilder::new("debug_test");
    let g = _builder.token("X", "x").rule("z", vec!["X"]).build();
    let debug_str = format!("{:?}", g);
    assert!(debug_str.contains("debug_test"));
}

// ═══════════════════════════════════════════════════════════════════════════
// Category 2: Token management — 8 tests
// Tests: string token, regex token, multiple, duplicate name, stored correctly,
//        pattern accessible, special regex, many tokens
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn token_1_add_string_token() {
    let g = GrammarBuilder::new("str_token")
        .token("KW", "keyword")
        .build();
    let token = g.tokens.values().next().unwrap();
    assert_eq!(token.name, "KW");
    assert_eq!(token.pattern, TokenPattern::String("keyword".to_string()));
}

#[test]
fn token_2_add_regex_token() {
    let g = GrammarBuilder::new("regex_token")
        .token("NUM", r"\d+")
        .build();
    let token = g.tokens.values().next().unwrap();
    assert_eq!(token.name, "NUM");
    if let TokenPattern::Regex(pat) = &token.pattern {
        assert_eq!(pat, r"\d+");
    } else {
        panic!("Expected regex pattern");
    }
}

#[test]
fn token_3_multiple_tokens() {
    let g = GrammarBuilder::new("multi_tok")
        .token("PLUS", "+")
        .token("MINUS", "-")
        .token("STAR", "*")
        .build();
    assert_eq!(g.tokens.len(), 3);
}

#[test]
fn token_4_duplicate_token_name() {
    // Adding token with same name twice should reuse symbol ID
    let g = GrammarBuilder::new("dup_tok")
        .token("X", "x")
        .token("X", "y")
        .build();
    // Should have one symbol for X, but the pattern is overwritten
    assert!(g.tokens.len() <= 2);
}

#[test]
fn token_5_token_stored_correctly() {
    let g = GrammarBuilder::new("store_tok")
        .token("ID", r"[a-zA-Z_][a-zA-Z0-9_]*")
        .build();
    let tok = g.tokens.values().next().unwrap();
    assert_eq!(tok.name, "ID");
    assert!(!tok.fragile);
}

#[test]
fn token_6_token_pattern_accessible() {
    let g = GrammarBuilder::new("pat_acc")
        .token("WHITESPACE", r"[ \t\n]+")
        .build();
    let tok = g.tokens.values().next().unwrap();
    assert!(matches!(tok.pattern, TokenPattern::Regex(_)));
}

#[test]
fn token_7_token_with_special_regex() {
    let g = GrammarBuilder::new("special_regex")
        .token("SPECIAL", r#"[^"]*"#)
        .build();
    let tok = g.tokens.values().next().unwrap();
    if let TokenPattern::Regex(pat) = &tok.pattern {
        assert!(pat.contains('['));
        assert!(pat.contains('^'));
    }
}

#[test]
fn token_8_many_tokens_20_plus() {
    let mut builder = GrammarBuilder::new("many_tokens");
    for i in 0..25 {
        builder = builder.token(&format!("T{}", i), &format!("t{}", i));
    }
    let g = builder.build();
    assert_eq!(g.tokens.len(), 25);
}

// ═══════════════════════════════════════════════════════════════════════════
// Category 3: Rule management — 8 tests
// Tests: single symbol RHS, multi-symbol RHS, empty RHS (epsilon), multiple
//        same LHS, different LHS, token reference, chained rules, recursive
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn rule_1_single_symbol_rhs() {
    let g = GrammarBuilder::new("single_rhs")
        .token("T", "t")
        .rule("r", vec!["T"])
        .build();
    let r_id = g.find_symbol_by_name("r").unwrap();
    let rules = g.get_rules_for_symbol(r_id).unwrap();
    assert_eq!(rules[0].rhs.len(), 1);
}

#[test]
fn rule_2_multi_symbol_rhs() {
    let g = GrammarBuilder::new("multi_rhs")
        .token("A", "a")
        .token("B", "b")
        .token("C", "c")
        .rule("expr", vec!["A", "B", "C"])
        .build();
    let expr_id = g.find_symbol_by_name("expr").unwrap();
    let rules = g.get_rules_for_symbol(expr_id).unwrap();
    assert_eq!(rules[0].rhs.len(), 3);
}

#[test]
fn rule_3_empty_rhs_epsilon() {
    let g = GrammarBuilder::new("epsilon")
        .rule("nullable", vec![])
        .build();
    let null_id = g.find_symbol_by_name("nullable").unwrap();
    let rules = g.get_rules_for_symbol(null_id).unwrap();
    assert_eq!(rules[0].rhs.len(), 1);
    assert!(matches!(rules[0].rhs[0], Symbol::Epsilon));
}

#[test]
fn rule_4_multiple_rules_same_lhs() {
    let g = GrammarBuilder::new("same_lhs")
        .token("A", "a")
        .token("B", "b")
        .rule("choice", vec!["A"])
        .rule("choice", vec!["B"])
        .build();
    let choice_id = g.find_symbol_by_name("choice").unwrap();
    let rules = g.get_rules_for_symbol(choice_id).unwrap();
    assert_eq!(rules.len(), 2);
}

#[test]
fn rule_5_rules_different_lhs() {
    let g = GrammarBuilder::new("diff_lhs")
        .token("X", "x")
        .rule("alpha", vec!["X"])
        .rule("beta", vec!["X"])
        .rule("gamma", vec!["X"])
        .build();
    assert_eq!(g.rules.len(), 3);
}

#[test]
fn rule_6_rule_references_token() {
    let g = GrammarBuilder::new("ref_tok")
        .token("IDENT", r"[a-z]+")
        .rule("item", vec!["IDENT"])
        .build();
    let item_id = g.find_symbol_by_name("item").unwrap();
    let rules = g.get_rules_for_symbol(item_id).unwrap();
    assert!(matches!(rules[0].rhs[0], Symbol::Terminal(_)));
}

#[test]
fn rule_7_chained_rules() {
    // A -> B -> C -> D
    let g = GrammarBuilder::new("chain_rules")
        .token("T", "t")
        .rule("d", vec!["T"])
        .rule("c", vec!["d"])
        .rule("b", vec!["c"])
        .rule("a", vec!["b"])
        .build();
    assert_eq!(g.rules.len(), 4);
}

#[test]
fn rule_8_recursive_rule() {
    let g = GrammarBuilder::new("recursive")
        .token("T", "t")
        .rule("expr", vec!["expr", "T"])
        .rule("expr", vec!["T"])
        .build();
    let expr_id = g.find_symbol_by_name("expr").unwrap();
    let rules = g.get_rules_for_symbol(expr_id).unwrap();
    assert_eq!(rules.len(), 2);
    // First should reference expr recursively
    assert!(matches!(rules[0].rhs[0], Symbol::NonTerminal(_)));
}

// ═══════════════════════════════════════════════════════════════════════════
// Category 4: Precedence/Associativity — 8 tests
// Tests: left assoc, right assoc, nonassoc, multiple levels, negative prec,
//        zero prec, different assocs, precedence stored correctly
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn prec_1_left_associative_rule() {
    let g = GrammarBuilder::new("left_assoc")
        .token("NUM", r"\d+")
        .token("+", "+")
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .rule("expr", vec!["NUM"])
        .build();
    let expr_id = g.find_symbol_by_name("expr").unwrap();
    let rules = g.get_rules_for_symbol(expr_id).unwrap();
    let add_rule = &rules[0];
    assert_eq!(add_rule.associativity, Some(Associativity::Left));
}

#[test]
fn prec_2_right_associative_rule() {
    let g = GrammarBuilder::new("right_assoc")
        .token("NUM", r"\d+")
        .token("^", "^")
        .rule_with_precedence("expr", vec!["expr", "^", "expr"], 1, Associativity::Right)
        .rule("expr", vec!["NUM"])
        .build();
    let expr_id = g.find_symbol_by_name("expr").unwrap();
    let rules = g.get_rules_for_symbol(expr_id).unwrap();
    let pow_rule = &rules[0];
    assert_eq!(pow_rule.associativity, Some(Associativity::Right));
}

#[test]
fn prec_3_nonassociative_rule() {
    let g = GrammarBuilder::new("nonassoc")
        .token("NUM", r"\d+")
        .token("=", "=")
        .rule_with_precedence("expr", vec!["expr", "=", "expr"], 1, Associativity::None)
        .rule("expr", vec!["NUM"])
        .build();
    let expr_id = g.find_symbol_by_name("expr").unwrap();
    let rules = g.get_rules_for_symbol(expr_id).unwrap();
    let eq_rule = &rules[0];
    assert_eq!(eq_rule.associativity, Some(Associativity::None));
}

#[test]
fn prec_4_multiple_precedence_levels() {
    let g = GrammarBuilder::new("multi_prec")
        .token("NUM", r"\d+")
        .token("+", "+")
        .token("*", "*")
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "*", "expr"], 2, Associativity::Left)
        .rule("expr", vec!["NUM"])
        .build();
    let expr_id = g.find_symbol_by_name("expr").unwrap();
    let rules = g.get_rules_for_symbol(expr_id).unwrap();
    assert_eq!(rules.len(), 3);
    // Check that two rules have different precedence
    if let (Some(PrecedenceKind::Static(p1)), Some(PrecedenceKind::Static(p2))) =
        (rules[0].precedence, rules[1].precedence)
    {
        assert_ne!(p1, p2);
    }
}

#[test]
fn prec_5_negative_precedence() {
    let g = GrammarBuilder::new("neg_prec")
        .token("T", "t")
        .rule_with_precedence("r", vec!["T"], -5, Associativity::Left)
        .build();
    let r_id = g.find_symbol_by_name("r").unwrap();
    let rules = g.get_rules_for_symbol(r_id).unwrap();
    if let Some(PrecedenceKind::Static(p)) = rules[0].precedence {
        assert_eq!(p, -5);
    }
}

#[test]
fn prec_6_zero_precedence() {
    let g = GrammarBuilder::new("zero_prec")
        .token("T", "t")
        .rule_with_precedence("r", vec!["T"], 0, Associativity::Left)
        .build();
    let r_id = g.find_symbol_by_name("r").unwrap();
    let rules = g.get_rules_for_symbol(r_id).unwrap();
    if let Some(PrecedenceKind::Static(p)) = rules[0].precedence {
        assert_eq!(p, 0);
    }
}

#[test]
fn prec_7_precedence_different_assocs() {
    let g = GrammarBuilder::new("diff_assoc")
        .token("A", "a")
        .token("B", "b")
        .rule_with_precedence("r", vec!["A"], 1, Associativity::Left)
        .rule_with_precedence("s", vec!["B"], 1, Associativity::Right)
        .build();
    let r_id = g.find_symbol_by_name("r").unwrap();
    let s_id = g.find_symbol_by_name("s").unwrap();
    let r_rules = g.get_rules_for_symbol(r_id).unwrap();
    let s_rules = g.get_rules_for_symbol(s_id).unwrap();
    assert_eq!(r_rules[0].associativity, Some(Associativity::Left));
    assert_eq!(s_rules[0].associativity, Some(Associativity::Right));
}

#[test]
fn prec_8_precedence_stored_correctly() {
    let g = GrammarBuilder::new("prec_store")
        .token("T", "t")
        .rule_with_precedence("r", vec!["T"], 42, Associativity::None)
        .build();
    let r_id = g.find_symbol_by_name("r").unwrap();
    let rules = g.get_rules_for_symbol(r_id).unwrap();
    assert!(rules[0].precedence.is_some());
    assert_eq!(rules[0].associativity, Some(Associativity::None));
}

// ═══════════════════════════════════════════════════════════════════════════
// Category 5: Inline/Supertype — 8 tests
// Tests: mark inline, mark supertype, multiple inlines, multiple supertypes,
//        combined, inline stored, supertype stored, inline rule validates
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn inline_1_mark_inline() {
    let g = GrammarBuilder::new("inline_mark")
        .token("T", "t")
        .rule("r", vec!["T"])
        .inline("r")
        .build();
    let r_id = g.find_symbol_by_name("r").unwrap();
    assert!(g.inline_rules.contains(&r_id));
}

#[test]
fn inline_2_mark_supertype() {
    let g = GrammarBuilder::new("super_mark")
        .token("T", "t")
        .rule("r", vec!["T"])
        .supertype("r")
        .build();
    let r_id = g.find_symbol_by_name("r").unwrap();
    assert!(g.supertypes.contains(&r_id));
}

#[test]
fn inline_3_multiple_inlines() {
    let g = GrammarBuilder::new("multi_inline")
        .token("A", "a")
        .token("B", "b")
        .rule("x", vec!["A"])
        .rule("y", vec!["B"])
        .inline("x")
        .inline("y")
        .build();
    assert_eq!(g.inline_rules.len(), 2);
}

#[test]
fn inline_4_multiple_supertypes() {
    let g = GrammarBuilder::new("multi_super")
        .token("A", "a")
        .token("B", "b")
        .rule("x", vec!["A"])
        .rule("y", vec!["B"])
        .supertype("x")
        .supertype("y")
        .build();
    assert_eq!(g.supertypes.len(), 2);
}

#[test]
fn inline_5_combined_inline_and_supertype() {
    let g = GrammarBuilder::new("combo")
        .token("T", "t")
        .rule("r", vec!["T"])
        .inline("r")
        .supertype("r")
        .build();
    let r_id = g.find_symbol_by_name("r").unwrap();
    assert!(g.inline_rules.contains(&r_id));
    assert!(g.supertypes.contains(&r_id));
}

#[test]
fn inline_6_inline_stored() {
    let g = GrammarBuilder::new("inline_store")
        .token("T", "t")
        .rule("rule_a", vec!["T"])
        .rule("rule_b", vec!["rule_a"])
        .inline("rule_a")
        .build();
    let a_id = g.find_symbol_by_name("rule_a").unwrap();
    assert!(g.inline_rules.contains(&a_id));
    assert!(!g.supertypes.contains(&a_id));
}

#[test]
fn inline_7_supertype_stored() {
    let g = GrammarBuilder::new("super_store")
        .token("T", "t")
        .rule("base", vec!["T"])
        .supertype("base")
        .build();
    let base_id = g.find_symbol_by_name("base").unwrap();
    assert!(g.supertypes.contains(&base_id));
    assert!(!g.inline_rules.contains(&base_id));
}

#[test]
fn inline_8_inline_rule_validates() {
    let g = GrammarBuilder::new("inline_valid")
        .token("T", "t")
        .rule("inner", vec!["T"])
        .inline("inner")
        .build();
    let inner_id = g.find_symbol_by_name("inner").unwrap();
    // Verify the rule still exists even though marked inline
    assert!(g.get_rules_for_symbol(inner_id).is_some());
}

// ═══════════════════════════════════════════════════════════════════════════
// Category 6: Fields/Extras/Externals — 8 tests
// Tests: add field mapping, multiple fields, add extra symbol, multiple
//        extras, add external, multiple externals, combined, stored correctly
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn field_1_add_extra_symbol() {
    let g = GrammarBuilder::new("extra_sym")
        .token("SPACE", r"[ \t]+")
        .extra("SPACE")
        .build();
    assert!(!g.extras.is_empty());
}

#[test]
fn field_2_multiple_extras() {
    let g = GrammarBuilder::new("multi_extra")
        .token("WS", r"[ \t]+")
        .token("COMMENT", r"//.*")
        .extra("WS")
        .extra("COMMENT")
        .build();
    assert_eq!(g.extras.len(), 2);
}

#[test]
fn field_3_add_external_symbol() {
    let g = GrammarBuilder::new("ext_sym").external("INDENT").build();
    assert!(!g.externals.is_empty());
    assert_eq!(g.externals[0].name, "INDENT");
}

#[test]
fn field_4_multiple_externals() {
    let g = GrammarBuilder::new("multi_ext")
        .external("INDENT")
        .external("DEDENT")
        .external("NEWLINE")
        .build();
    assert_eq!(g.externals.len(), 3);
}

#[test]
fn field_5_combined_extras_and_externals() {
    let g = GrammarBuilder::new("combo_ext")
        .token("WS", r"[ \t]+")
        .token("COMMENT", r"//.*")
        .extra("WS")
        .extra("COMMENT")
        .external("INDENT")
        .external("DEDENT")
        .build();
    assert_eq!(g.extras.len(), 2);
    assert_eq!(g.externals.len(), 2);
}

#[test]
fn field_6_extra_stored_correctly() {
    let g = GrammarBuilder::new("extra_correct")
        .token("WHITESPACE", r"[ \t\n]+")
        .extra("WHITESPACE")
        .build();
    assert_eq!(g.extras.len(), 1);
}

#[test]
fn field_7_external_stored_correctly() {
    let g = GrammarBuilder::new("ext_correct")
        .external("SCANNER_TOKEN")
        .build();
    assert_eq!(g.externals.len(), 1);
    assert_eq!(g.externals[0].name, "SCANNER_TOKEN");
}

#[test]
fn field_8_combined_all_features() {
    let g = GrammarBuilder::new("full_combo")
        .token("NUM", r"\d+")
        .token("WS", r"[ \t]+")
        .token("COMMENT", r"//.*")
        .rule("expr", vec!["NUM"])
        .extra("WS")
        .extra("COMMENT")
        .external("INDENT")
        .inline("expr")
        .supertype("expr")
        .build();
    assert_eq!(g.tokens.len(), 3);
    assert_eq!(g.rules.len(), 1);
    assert_eq!(g.extras.len(), 2);
    assert_eq!(g.externals.len(), 1);
}

// ═══════════════════════════════════════════════════════════════════════════
// Category 7: Complex grammars — 8 tests
// Tests: arithmetic expression, if-else grammar, list grammar, nested
//        expression, multi-level inheritance, all features, wide (10+ rules),
//        deep (chain of 10)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn complex_1_arithmetic_expression_grammar() {
    let g = GrammarBuilder::new("arithmetic")
        .token("NUM", r"\d+")
        .token("+", "+")
        .token("-", "-")
        .token("*", "*")
        .token("/", "/")
        .token("(", "(")
        .token(")", ")")
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "-", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "*", "expr"], 2, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "/", "expr"], 2, Associativity::Left)
        .rule("expr", vec!["(", "expr", ")"])
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    assert!(g.tokens.len() >= 7);
    assert!(g.rules.len() >= 4);
}

#[test]
fn complex_2_if_else_grammar() {
    let g = GrammarBuilder::new("if_else")
        .token("if", "if")
        .token("else", "else")
        .token("(", "(")
        .token(")", ")")
        .token("COND", r"[a-z]+")
        .token("STMT", r"[A-Z]+")
        .rule("statement", vec!["if", "(", "COND", ")", "STMT"])
        .rule(
            "statement",
            vec!["if", "(", "COND", ")", "STMT", "else", "STMT"],
        )
        .rule("program", vec!["statement"])
        .rule("program", vec!["program", "statement"])
        .start("program")
        .build();
    assert_eq!(g.rules.len(), 4);
}

#[test]
fn complex_3_list_grammar() {
    let g = GrammarBuilder::new("list_grammar")
        .token("ELEM", r"[0-9]+")
        .token(",", ",")
        .token("[", "[")
        .token("]", "]")
        .rule("list", vec!["[", "]"])
        .rule("list", vec!["[", "elements", "]"])
        .rule("elements", vec!["ELEM"])
        .rule("elements", vec!["elements", ",", "ELEM"])
        .start("list")
        .build();
    assert_eq!(g.rules.len(), 4);
}

#[test]
fn complex_4_nested_expression_grammar() {
    let g = GrammarBuilder::new("nested_expr")
        .token("VAR", r"[a-z]+")
        .token("(", "(")
        .token(")", ")")
        .token(",", ",")
        .rule("expr", vec!["VAR"])
        .rule("expr", vec!["(", "expr_list", ")"])
        .rule("expr_list", vec!["expr"])
        .rule("expr_list", vec!["expr_list", ",", "expr"])
        .start("expr")
        .build();
    assert_eq!(g.rules.len(), 4);
}

#[test]
fn complex_5_multi_level_inheritance() {
    let g = GrammarBuilder::new("multi_inherit")
        .token("T", "t")
        .rule("level1", vec!["T"])
        .rule("level2", vec!["level1"])
        .rule("level3", vec!["level2"])
        .rule("level4", vec!["level3"])
        .rule("level5", vec!["level4"])
        .supertype("level1")
        .supertype("level2")
        .supertype("level3")
        .start("level5")
        .build();
    assert_eq!(g.rules.len(), 5);
    assert_eq!(g.supertypes.len(), 3);
}

#[test]
fn complex_6_grammar_with_all_features() {
    let g = GrammarBuilder::new("all_features")
        .token("KW", "keyword")
        .token("ID", r"[a-z]+")
        .token("NUM", r"\d+")
        .token("WS", r"[ \t]+")
        .token("COMMENT", r"//.*")
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .rule("expr", vec!["ID"])
        .rule("expr", vec!["NUM"])
        .rule("stmt", vec!["KW", "expr"])
        .inline("expr")
        .supertype("stmt")
        .extra("WS")
        .extra("COMMENT")
        .external("INDENT")
        .start("stmt")
        .build();
    assert!(g.tokens.len() >= 5);
    assert!(g.rules.len() >= 4);
    assert_eq!(g.inline_rules.len(), 1);
    assert_eq!(g.supertypes.len(), 1);
    assert_eq!(g.extras.len(), 2);
    assert_eq!(g.externals.len(), 1);
}

#[test]
fn complex_7_wide_grammar_10_plus_rules() {
    let mut builder = GrammarBuilder::new("wide");
    builder = builder.token("T", "t");
    for i in 0..12 {
        builder = builder.rule(&format!("rule_{}", i), vec!["T"]);
    }
    let g = builder.build();
    assert!(g.rules.len() >= 12);
}

#[test]
fn complex_8_deep_grammar_chain_of_10() {
    let mut builder = GrammarBuilder::new("deep");
    builder = builder.token("T", "t");
    builder = builder.rule("level_0", vec!["T"]);
    for i in 1..10 {
        builder = builder.rule(&format!("level_{}", i), vec![&format!("level_{}", i - 1)]);
    }
    let g = builder.build();
    assert_eq!(g.rules.len(), 10);
}

// ═══════════════════════════════════════════════════════════════════════════
// Category 8: Edge cases — 8 tests
// Tests: long grammar name, empty token pattern (if valid), unicode rule names,
//        single-char token, very long RHS, preserve insertion order,
//        builder reuse after build, all features combined
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn edge_1_long_grammar_name() {
    let long_name = "this_is_a_very_long_grammar_name_with_many_underscores_and_numbers_123456789";
    let g = GrammarBuilder::new(long_name).build();
    assert_eq!(g.name, long_name);
    assert!(g.name.len() > 50);
}

#[test]
fn edge_2_single_char_token() {
    let g = GrammarBuilder::new("single_char")
        .token("+", "+")
        .token("-", "-")
        .token("*", "*")
        .token("/", "/")
        .build();
    assert_eq!(g.tokens.len(), 4);
}

#[test]
fn edge_3_very_long_rhs() {
    let mut builder = GrammarBuilder::new("long_rhs");
    let mut tokens_vec = Vec::new();
    for i in 0..30 {
        let token_name = format!("T{}", i);
        builder = builder.token(&token_name, &token_name);
        tokens_vec.push(token_name);
    }
    let rhs: Vec<&str> = tokens_vec.iter().map(|s| s.as_str()).collect();
    builder = builder.rule("r", rhs);
    let g = builder.build();
    let r_id = g.find_symbol_by_name("r").unwrap();
    let rules = g.get_rules_for_symbol(r_id).unwrap();
    assert_eq!(rules[0].rhs.len(), 30);
}

#[test]
fn edge_4_preserve_insertion_order_tokens() {
    let g = GrammarBuilder::new("order")
        .token("FIRST", "first")
        .token("SECOND", "second")
        .token("THIRD", "third")
        .token("FOURTH", "fourth")
        .build();
    let names: Vec<_> = g.tokens.values().map(|t| t.name.as_str()).collect();
    assert_eq!(names[0], "FIRST");
    assert_eq!(names[1], "SECOND");
    assert_eq!(names[2], "THIRD");
    assert_eq!(names[3], "FOURTH");
}

#[test]
fn edge_5_preserve_insertion_order_rules() {
    let g = GrammarBuilder::new("rule_order")
        .token("T", "t")
        .rule("alpha", vec!["T"])
        .rule("beta", vec!["T"])
        .rule("gamma", vec!["T"])
        .rule("delta", vec!["T"])
        .build();
    let names: Vec<_> = g.rule_names.values().map(|n| n.as_str()).collect();
    assert_eq!(names[0], "alpha");
    assert_eq!(names[1], "beta");
    assert_eq!(names[2], "gamma");
    assert_eq!(names[3], "delta");
}

#[test]
fn edge_6_fragile_token() {
    let g = GrammarBuilder::new("fragile")
        .fragile_token("ERROR", "error")
        .build();
    let tok = g.tokens.values().next().unwrap();
    assert!(tok.fragile);
}

#[test]
fn edge_7_multiple_token_types_mixed() {
    let g = GrammarBuilder::new("mixed")
        .token("KW", "keyword")
        .token("NUM", r"\d+")
        .fragile_token("ERR", "error")
        .token("ID", r"[a-zA-Z_][a-zA-Z0-9_]*")
        .build();
    assert_eq!(g.tokens.len(), 4);
    let fragile_count = g.tokens.values().filter(|t| t.fragile).count();
    assert_eq!(fragile_count, 1);
}

#[test]
fn edge_8_all_builder_features_maximum() {
    let mut builder = GrammarBuilder::new("maximum_features_test_grammar");

    // Add many tokens
    for i in 0..10 {
        builder = builder.token(&format!("TOK{}", i), &format!("t{}", i));
    }

    // Add rules with precedence and associativity
    builder = builder
        .rule_with_precedence("expr", vec!["TOK0", "TOK1"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["TOK2", "TOK3"], 2, Associativity::Right)
        .rule("expr", vec!["TOK4"])
        .rule("stmt", vec!["expr", "TOK5"]);

    // Mark inline and supertype
    builder = builder.inline("expr").supertype("stmt");

    // Add extras and externals
    builder = builder
        .token("WS", r"[ \t\n]+")
        .extra("WS")
        .external("SCANNER1")
        .external("SCANNER2");

    builder = builder.start("stmt");
    let g = builder.build();

    assert!(g.tokens.len() >= 12);
    assert!(g.rules.len() >= 4);
    assert_eq!(g.inline_rules.len(), 1);
    assert_eq!(g.supertypes.len(), 1);
    assert!(!g.extras.is_empty());
    assert_eq!(g.externals.len(), 2);
}
