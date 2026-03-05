//! GrammarBuilder fluent API test suite (v6) — 68 tests across 8 categories.

use adze_ir::builder::GrammarBuilder;
use adze_ir::{Associativity, PrecedenceKind, Symbol, TokenPattern};

// ═══════════════════════════════════════════════════════════════════════════
// Category 1: Basic builder — token + rule + start + build (10 tests)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_basic_single_token_single_rule() {
    let g = GrammarBuilder::new("minimal")
        .token("A", "a")
        .rule("root", vec!["A"])
        .start("root")
        .build();
    assert_eq!(g.name, "minimal");
    assert_eq!(g.tokens.len(), 1);
    assert_eq!(g.rules.len(), 1);
}

#[test]
fn test_basic_two_tokens_one_rule() {
    let g = GrammarBuilder::new("pair")
        .token("X", "x")
        .token("Y", "y")
        .rule("pair", vec!["X", "Y"])
        .start("pair")
        .build();
    assert_eq!(g.tokens.len(), 2);
    let pair_id = g.find_symbol_by_name("pair").unwrap();
    let rules = g.get_rules_for_symbol(pair_id).unwrap();
    assert_eq!(rules[0].rhs.len(), 2);
}

#[test]
fn test_basic_multiple_rules_different_lhs() {
    let g = GrammarBuilder::new("multi")
        .token("A", "a")
        .token("B", "b")
        .rule("alpha", vec!["A"])
        .rule("beta", vec!["B"])
        .start("alpha")
        .build();
    assert_eq!(g.rules.len(), 2);
}

#[test]
fn test_basic_start_reorders_rules() {
    let g = GrammarBuilder::new("order")
        .token("T", "t")
        .rule("second", vec!["T"])
        .rule("first", vec!["second"])
        .start("first")
        .build();
    let first_key = g.rules.keys().next().unwrap();
    assert_eq!(g.rule_names[first_key], "first");
}

#[test]
fn test_basic_build_without_start() {
    let g = GrammarBuilder::new("nostart")
        .token("T", "t")
        .rule("r", vec!["T"])
        .build();
    // Should still build without panic
    assert_eq!(g.rules.len(), 1);
}

#[test]
fn test_basic_name_preserved() {
    let g = GrammarBuilder::new("my_grammar_2024").build();
    assert_eq!(g.name, "my_grammar_2024");
}

#[test]
fn test_basic_token_string_pattern() {
    let g = GrammarBuilder::new("str").token("KW", "keyword").build();
    let tok = g.tokens.values().next().unwrap();
    assert_eq!(tok.pattern, TokenPattern::String("keyword".to_string()));
}

#[test]
fn test_basic_token_regex_pattern() {
    let g = GrammarBuilder::new("rgx").token("NUM", r"\d+").build();
    let tok = g.tokens.values().next().unwrap();
    assert_eq!(tok.pattern, TokenPattern::Regex(r"\d+".to_string()));
}

#[test]
fn test_basic_rule_references_token_as_terminal() {
    let g = GrammarBuilder::new("term")
        .token("IDENT", r"[a-z]+")
        .rule("item", vec!["IDENT"])
        .start("item")
        .build();
    let item_id = g.find_symbol_by_name("item").unwrap();
    let rules = g.get_rules_for_symbol(item_id).unwrap();
    assert!(matches!(rules[0].rhs[0], Symbol::Terminal(_)));
}

#[test]
fn test_basic_rule_references_rule_as_nonterminal() {
    let g = GrammarBuilder::new("nt")
        .token("T", "t")
        .rule("leaf", vec!["T"])
        .rule("wrapper", vec!["leaf"])
        .start("wrapper")
        .build();
    let wrapper_id = g.find_symbol_by_name("wrapper").unwrap();
    let rules = g.get_rules_for_symbol(wrapper_id).unwrap();
    assert!(matches!(rules[0].rhs[0], Symbol::NonTerminal(_)));
}

// ═══════════════════════════════════════════════════════════════════════════
// Category 2: Precedence — rule_with_precedence (8 tests)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_prec_left_associative() {
    let g = GrammarBuilder::new("left")
        .token("N", r"\d+")
        .token("+", "+")
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .rule("expr", vec!["N"])
        .start("expr")
        .build();
    let expr_id = g.find_symbol_by_name("expr").unwrap();
    let rules = g.get_rules_for_symbol(expr_id).unwrap();
    assert_eq!(rules[0].precedence, Some(PrecedenceKind::Static(1)));
    assert_eq!(rules[0].associativity, Some(Associativity::Left));
}

#[test]
fn test_prec_right_associative() {
    let g = GrammarBuilder::new("right")
        .token("N", r"\d+")
        .token("^", "^")
        .rule_with_precedence("expr", vec!["expr", "^", "expr"], 3, Associativity::Right)
        .rule("expr", vec!["N"])
        .start("expr")
        .build();
    let expr_id = g.find_symbol_by_name("expr").unwrap();
    let rules = g.get_rules_for_symbol(expr_id).unwrap();
    assert_eq!(rules[0].associativity, Some(Associativity::Right));
}

#[test]
fn test_prec_none_associative() {
    let g = GrammarBuilder::new("none_assoc")
        .token("N", r"\d+")
        .token("==", "==")
        .rule_with_precedence("cmp", vec!["cmp", "==", "cmp"], 0, Associativity::None)
        .rule("cmp", vec!["N"])
        .start("cmp")
        .build();
    let cmp_id = g.find_symbol_by_name("cmp").unwrap();
    let rules = g.get_rules_for_symbol(cmp_id).unwrap();
    assert_eq!(rules[0].associativity, Some(Associativity::None));
}

#[test]
fn test_prec_negative_level() {
    let g = GrammarBuilder::new("neg")
        .token("T", "t")
        .rule_with_precedence("r", vec!["T"], -5, Associativity::Left)
        .start("r")
        .build();
    let r_id = g.find_symbol_by_name("r").unwrap();
    let rules = g.get_rules_for_symbol(r_id).unwrap();
    assert_eq!(rules[0].precedence, Some(PrecedenceKind::Static(-5)));
}

#[test]
fn test_prec_zero_level() {
    let g = GrammarBuilder::new("zero")
        .token("T", "t")
        .rule_with_precedence("r", vec!["T"], 0, Associativity::Left)
        .start("r")
        .build();
    let r_id = g.find_symbol_by_name("r").unwrap();
    let rules = g.get_rules_for_symbol(r_id).unwrap();
    assert_eq!(rules[0].precedence, Some(PrecedenceKind::Static(0)));
}

#[test]
fn test_prec_mixed_with_plain_rule() {
    let g = GrammarBuilder::new("mixed")
        .token("N", r"\d+")
        .token("+", "+")
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .rule("expr", vec!["N"])
        .start("expr")
        .build();
    let expr_id = g.find_symbol_by_name("expr").unwrap();
    let rules = g.get_rules_for_symbol(expr_id).unwrap();
    assert!(rules[0].precedence.is_some());
    assert!(rules[1].precedence.is_none());
}

#[test]
fn test_prec_multiple_levels_ordered() {
    let g = GrammarBuilder::new("levels")
        .token("N", r"\d+")
        .token("+", "+")
        .token("*", "*")
        .token("^", "^")
        .rule_with_precedence("e", vec!["e", "+", "e"], 1, Associativity::Left)
        .rule_with_precedence("e", vec!["e", "*", "e"], 2, Associativity::Left)
        .rule_with_precedence("e", vec!["e", "^", "e"], 3, Associativity::Right)
        .rule("e", vec!["N"])
        .start("e")
        .build();
    let e_id = g.find_symbol_by_name("e").unwrap();
    let rules = g.get_rules_for_symbol(e_id).unwrap();
    let levels: Vec<i16> = rules
        .iter()
        .filter_map(|r| match r.precedence {
            Some(PrecedenceKind::Static(l)) => Some(l),
            _ => Option::None,
        })
        .collect();
    assert_eq!(levels, vec![1, 2, 3]);
}

#[test]
fn test_prec_high_level_value() {
    let g = GrammarBuilder::new("high")
        .token("T", "t")
        .rule_with_precedence("r", vec!["T"], i16::MAX, Associativity::Left)
        .start("r")
        .build();
    let r_id = g.find_symbol_by_name("r").unwrap();
    let rules = g.get_rules_for_symbol(r_id).unwrap();
    assert_eq!(rules[0].precedence, Some(PrecedenceKind::Static(i16::MAX)));
}

// ═══════════════════════════════════════════════════════════════════════════
// Category 3: Inline and supertype marking (8 tests)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_inline_single_rule() {
    let g = GrammarBuilder::new("inl")
        .token("T", "t")
        .rule("helper", vec!["T"])
        .rule("main", vec!["helper"])
        .inline("helper")
        .start("main")
        .build();
    assert_eq!(g.inline_rules.len(), 1);
}

#[test]
fn test_inline_multiple_rules() {
    let g = GrammarBuilder::new("inl2")
        .token("T", "t")
        .rule("h1", vec!["T"])
        .rule("h2", vec!["T"])
        .rule("main", vec!["h1"])
        .inline("h1")
        .inline("h2")
        .start("main")
        .build();
    assert_eq!(g.inline_rules.len(), 2);
}

#[test]
fn test_inline_symbol_id_matches() {
    let g = GrammarBuilder::new("inl_id")
        .token("T", "t")
        .rule("helper", vec!["T"])
        .rule("top", vec!["helper"])
        .inline("helper")
        .start("top")
        .build();
    let helper_id = g.find_symbol_by_name("helper").unwrap();
    assert!(g.inline_rules.contains(&helper_id));
}

#[test]
fn test_supertype_single() {
    let g = GrammarBuilder::new("sup")
        .token("T", "t")
        .rule("expression", vec!["T"])
        .supertype("expression")
        .start("expression")
        .build();
    assert_eq!(g.supertypes.len(), 1);
}

#[test]
fn test_supertype_multiple() {
    let g = GrammarBuilder::new("sup2")
        .token("T", "t")
        .rule("expression", vec!["T"])
        .rule("statement", vec!["expression"])
        .supertype("expression")
        .supertype("statement")
        .start("statement")
        .build();
    assert_eq!(g.supertypes.len(), 2);
}

#[test]
fn test_supertype_symbol_id_matches() {
    let g = GrammarBuilder::new("sup_id")
        .token("T", "t")
        .rule("node", vec!["T"])
        .supertype("node")
        .start("node")
        .build();
    let node_id = g.find_symbol_by_name("node").unwrap();
    assert!(g.supertypes.contains(&node_id));
}

#[test]
fn test_inline_and_supertype_independent() {
    let g = GrammarBuilder::new("both")
        .token("T", "t")
        .rule("helper", vec!["T"])
        .rule("base", vec!["T"])
        .rule("top", vec!["helper"])
        .inline("helper")
        .supertype("base")
        .start("top")
        .build();
    assert_eq!(g.inline_rules.len(), 1);
    assert_eq!(g.supertypes.len(), 1);
    let helper_id = g.find_symbol_by_name("helper").unwrap();
    let base_id = g.find_symbol_by_name("base").unwrap();
    assert!(g.inline_rules.contains(&helper_id));
    assert!(g.supertypes.contains(&base_id));
}

#[test]
fn test_inline_does_not_remove_rule() {
    let g = GrammarBuilder::new("keep")
        .token("T", "t")
        .rule("helper", vec!["T"])
        .rule("main", vec!["helper"])
        .inline("helper")
        .start("main")
        .build();
    let helper_id = g.find_symbol_by_name("helper").unwrap();
    assert!(g.get_rules_for_symbol(helper_id).is_some());
}

// ═══════════════════════════════════════════════════════════════════════════
// Category 4: Extras, externals, precedence declarations (8 tests)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_extra_single() {
    let g = GrammarBuilder::new("ws")
        .token("WS", r"\s+")
        .extra("WS")
        .token("T", "t")
        .rule("r", vec!["T"])
        .start("r")
        .build();
    assert_eq!(g.extras.len(), 1);
}

#[test]
fn test_extra_multiple() {
    let g = GrammarBuilder::new("ws2")
        .token("WS", r"\s+")
        .token("COMMENT", r"//[^\n]*")
        .extra("WS")
        .extra("COMMENT")
        .token("T", "t")
        .rule("r", vec!["T"])
        .start("r")
        .build();
    assert_eq!(g.extras.len(), 2);
}

#[test]
fn test_external_single() {
    let g = GrammarBuilder::new("ext")
        .external("INDENT")
        .token("T", "t")
        .rule("r", vec!["T"])
        .start("r")
        .build();
    assert_eq!(g.externals.len(), 1);
    assert_eq!(g.externals[0].name, "INDENT");
}

#[test]
fn test_external_multiple() {
    let g = GrammarBuilder::new("ext2")
        .external("INDENT")
        .external("DEDENT")
        .external("NEWLINE")
        .token("T", "t")
        .rule("r", vec!["T"])
        .start("r")
        .build();
    assert_eq!(g.externals.len(), 3);
    assert_eq!(g.externals[0].name, "INDENT");
    assert_eq!(g.externals[1].name, "DEDENT");
    assert_eq!(g.externals[2].name, "NEWLINE");
}

#[test]
fn test_external_unique_symbol_ids() {
    let g = GrammarBuilder::new("ext_ids")
        .external("A_EXT")
        .external("B_EXT")
        .build();
    assert_ne!(g.externals[0].symbol_id, g.externals[1].symbol_id);
}

#[test]
fn test_precedence_declaration() {
    let g = GrammarBuilder::new("prec_decl")
        .token("+", "+")
        .token("*", "*")
        .precedence(1, Associativity::Left, vec!["+"])
        .precedence(2, Associativity::Left, vec!["*"])
        .token("N", r"\d+")
        .rule("e", vec!["N"])
        .start("e")
        .build();
    assert_eq!(g.precedences.len(), 2);
    assert_eq!(g.precedences[0].level, 1);
    assert_eq!(g.precedences[1].level, 2);
}

#[test]
fn test_precedence_declaration_with_multiple_symbols() {
    let g = GrammarBuilder::new("prec_multi")
        .token("+", "+")
        .token("-", "-")
        .precedence(1, Associativity::Left, vec!["+", "-"])
        .token("N", r"\d+")
        .rule("e", vec!["N"])
        .start("e")
        .build();
    assert_eq!(g.precedences.len(), 1);
    assert_eq!(g.precedences[0].symbols.len(), 2);
}

#[test]
fn test_extras_and_externals_together() {
    let g = GrammarBuilder::new("combo")
        .token("WS", r"\s+")
        .token("T", "t")
        .extra("WS")
        .external("INDENT")
        .external("DEDENT")
        .rule("r", vec!["T"])
        .start("r")
        .build();
    assert_eq!(g.extras.len(), 1);
    assert_eq!(g.externals.len(), 2);
}

// ═══════════════════════════════════════════════════════════════════════════
// Category 5: Builder edge/error cases (8 tests)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_error_duplicate_token_overwrites() {
    // Second .token() with same name overwrites the first
    let g = GrammarBuilder::new("dup_tok")
        .token("T", "first")
        .token("T", "second")
        .rule("r", vec!["T"])
        .start("r")
        .build();
    let tok = g.tokens.values().find(|t| t.name == "T").unwrap();
    assert_eq!(tok.pattern, TokenPattern::String("second".to_string()));
}

#[test]
fn test_error_empty_grammar_builds() {
    let g = GrammarBuilder::new("empty").build();
    assert!(g.rules.is_empty());
    assert!(g.tokens.is_empty());
}

#[test]
fn test_error_no_start_still_builds() {
    let g = GrammarBuilder::new("nostart")
        .token("T", "t")
        .rule("r", vec!["T"])
        .build();
    assert_eq!(g.rules.len(), 1);
}

#[test]
fn test_error_rule_with_undefined_token_treated_as_nonterminal() {
    // A symbol not registered as a token is treated as NonTerminal
    let g = GrammarBuilder::new("undef")
        .rule("r", vec!["UNKNOWN"])
        .start("r")
        .build();
    let r_id = g.find_symbol_by_name("r").unwrap();
    let rules = g.get_rules_for_symbol(r_id).unwrap();
    assert!(matches!(rules[0].rhs[0], Symbol::NonTerminal(_)));
}

#[test]
fn test_error_start_before_rules() {
    // Setting start before defining rules should still work
    let g = GrammarBuilder::new("early_start")
        .start("root")
        .token("T", "t")
        .rule("root", vec!["T"])
        .build();
    let first_key = g.rules.keys().next().unwrap();
    assert_eq!(g.rule_names[first_key], "root");
}

#[test]
fn test_error_extra_before_token_defined() {
    // Extra can reference a symbol before its token is defined
    let g = GrammarBuilder::new("early_extra")
        .extra("WS")
        .token("WS", r"\s+")
        .token("T", "t")
        .rule("r", vec!["T"])
        .start("r")
        .build();
    assert_eq!(g.extras.len(), 1);
}

#[test]
fn test_error_rule_with_same_lhs_and_rhs() {
    // Direct left recursion: A → A
    let g = GrammarBuilder::new("self_ref")
        .token("T", "t")
        .rule("r", vec!["r"])
        .rule("r", vec!["T"])
        .start("r")
        .build();
    let r_id = g.find_symbol_by_name("r").unwrap();
    let rules = g.get_rules_for_symbol(r_id).unwrap();
    assert_eq!(rules.len(), 2);
}

#[test]
fn test_error_fragile_and_normal_tokens_coexist() {
    let g = GrammarBuilder::new("mixed_tok")
        .token("NORMAL", "normal")
        .fragile_token("FRAG", r"[^\s]+")
        .rule("r", vec!["NORMAL"])
        .start("r")
        .build();
    let normal = g.tokens.values().find(|t| t.name == "NORMAL").unwrap();
    let frag = g.tokens.values().find(|t| t.name == "FRAG").unwrap();
    assert!(!normal.fragile);
    assert!(frag.fragile);
}

// ═══════════════════════════════════════════════════════════════════════════
// Category 6: Grammar properties after build (8 tests)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_prop_rule_count() {
    let g = GrammarBuilder::new("count")
        .token("A", "a")
        .token("B", "b")
        .rule("r1", vec!["A"])
        .rule("r2", vec!["B"])
        .rule("r3", vec!["A", "B"])
        .start("r1")
        .build();
    // rules map groups by LHS, so 3 distinct LHS = 3 entries
    assert_eq!(g.rules.len(), 3);
}

#[test]
fn test_prop_token_count() {
    let g = GrammarBuilder::new("tok_count")
        .token("A", "a")
        .token("B", "b")
        .token("C", "c")
        .rule("r", vec!["A"])
        .start("r")
        .build();
    assert_eq!(g.tokens.len(), 3);
}

#[test]
fn test_prop_grammar_name() {
    let g = GrammarBuilder::new("test_lang_v2").build();
    assert_eq!(g.name, "test_lang_v2");
}

#[test]
fn test_prop_start_symbol_first_rule() {
    let g = GrammarBuilder::new("start_test")
        .token("T", "t")
        .rule("beta", vec!["T"])
        .rule("alpha", vec!["beta"])
        .start("alpha")
        .build();
    // When start is set, first key in rules map is the start symbol
    let first_key = g.rules.keys().next().unwrap();
    let first_name = g.rule_names.get(first_key).unwrap();
    assert_eq!(first_name, "alpha");
}

#[test]
fn test_prop_all_rules_iterator() {
    let g = GrammarBuilder::new("iter")
        .token("A", "a")
        .token("B", "b")
        .rule("r1", vec!["A"])
        .rule("r1", vec!["B"])
        .rule("r2", vec!["A", "B"])
        .start("r1")
        .build();
    // 2 alternatives for r1 + 1 for r2 = 3 total
    assert_eq!(g.all_rules().count(), 3);
}

#[test]
fn test_prop_production_ids_unique() {
    let g = GrammarBuilder::new("prod")
        .token("A", "a")
        .token("B", "b")
        .rule("r", vec!["A"])
        .rule("r", vec!["B"])
        .rule("r", vec!["A", "B"])
        .start("r")
        .build();
    let r_id = g.find_symbol_by_name("r").unwrap();
    let rules = g.get_rules_for_symbol(r_id).unwrap();
    let ids: Vec<_> = rules.iter().map(|r| r.production_id).collect();
    let unique: std::collections::HashSet<_> = ids.iter().collect();
    assert_eq!(unique.len(), ids.len());
}

#[test]
fn test_prop_rule_names_populated() {
    let g = GrammarBuilder::new("names")
        .token("T", "t")
        .rule("alpha", vec!["T"])
        .rule("beta", vec!["alpha"])
        .start("alpha")
        .build();
    assert!(g.find_symbol_by_name("alpha").is_some());
    assert!(g.find_symbol_by_name("beta").is_some());
}

#[test]
fn test_prop_fields_empty_by_default() {
    let g = GrammarBuilder::new("fields")
        .token("T", "t")
        .rule("r", vec!["T"])
        .start("r")
        .build();
    assert!(g.fields.is_empty());
    let r_id = g.find_symbol_by_name("r").unwrap();
    let rules = g.get_rules_for_symbol(r_id).unwrap();
    assert!(rules[0].fields.is_empty());
}

// ═══════════════════════════════════════════════════════════════════════════
// Category 7: Complex grammars (8 tests)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_complex_arithmetic() {
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
    let expr_id = g.find_symbol_by_name("expr").unwrap();
    let rules = g.get_rules_for_symbol(expr_id).unwrap();
    assert_eq!(rules.len(), 6);
    assert_eq!(g.tokens.len(), 7);
}

#[test]
fn test_complex_json_like() {
    let g = GrammarBuilder::new("json")
        .token("STRING", r#""[^"]*""#)
        .token("NUMBER", r"-?\d+(\.\d+)?")
        .token("TRUE", "true")
        .token("FALSE", "false")
        .token("NULL", "null")
        .token("{", "{")
        .token("}", "}")
        .token("[", "[")
        .token("]", "]")
        .token(":", ":")
        .token(",", ",")
        .rule("value", vec!["STRING"])
        .rule("value", vec!["NUMBER"])
        .rule("value", vec!["TRUE"])
        .rule("value", vec!["FALSE"])
        .rule("value", vec!["NULL"])
        .rule("value", vec!["object"])
        .rule("value", vec!["array"])
        .rule("object", vec!["{", "}"])
        .rule("object", vec!["{", "members", "}"])
        .rule("members", vec!["pair"])
        .rule("members", vec!["members", ",", "pair"])
        .rule("pair", vec!["STRING", ":", "value"])
        .rule("array", vec!["[", "]"])
        .rule("array", vec!["[", "elements", "]"])
        .rule("elements", vec!["value"])
        .rule("elements", vec!["elements", ",", "value"])
        .start("value")
        .build();
    assert_eq!(g.rules.len(), 6); // value, object, members, pair, array, elements
    assert_eq!(g.tokens.len(), 11);
}

#[test]
fn test_complex_nested_expressions() {
    let g = GrammarBuilder::new("nested")
        .token("ID", r"[a-z]+")
        .token("(", "(")
        .token(")", ")")
        .token(",", ",")
        .rule("call", vec!["ID", "(", "args", ")"])
        .rule("call", vec!["ID", "(", ")"])
        .rule("args", vec!["expr"])
        .rule("args", vec!["args", ",", "expr"])
        .rule("expr", vec!["ID"])
        .rule("expr", vec!["call"])
        .start("call")
        .build();
    assert_eq!(g.rules.len(), 3); // call, args, expr
}

#[test]
fn test_complex_statement_language() {
    let g = GrammarBuilder::new("lang")
        .token("ID", r"[a-z]+")
        .token("NUM", r"\d+")
        .token("=", "=")
        .token(";", ";")
        .token("if", "if")
        .token("else", "else")
        .token("{", "{")
        .token("}", "}")
        .rule("program", vec!["stmt"])
        .rule("program", vec!["program", "stmt"])
        .rule("stmt", vec!["assign"])
        .rule("stmt", vec!["if_stmt"])
        .rule("assign", vec!["ID", "=", "expr", ";"])
        .rule("if_stmt", vec!["if", "expr", "block"])
        .rule("if_stmt", vec!["if", "expr", "block", "else", "block"])
        .rule("block", vec!["{", "program", "}"])
        .rule("expr", vec!["ID"])
        .rule("expr", vec!["NUM"])
        .start("program")
        .build();
    assert_eq!(g.rules.len(), 6); // program, stmt, assign, if_stmt, block, expr
    let program_id = g.find_symbol_by_name("program").unwrap();
    let first_key = g.rules.keys().next().unwrap();
    assert_eq!(*first_key, program_id);
}

#[test]
fn test_complex_python_like_preset() {
    let g = GrammarBuilder::python_like();
    assert_eq!(g.name, "python_like");
    assert!(!g.externals.is_empty());
    assert!(!g.extras.is_empty());
    let module_id = g.find_symbol_by_name("module").unwrap();
    let module_rules = g.get_rules_for_symbol(module_id).unwrap();
    assert!(module_rules.len() >= 2);
}

#[test]
fn test_complex_javascript_like_preset() {
    let g = GrammarBuilder::javascript_like();
    assert_eq!(g.name, "javascript_like");
    assert!(!g.extras.is_empty());
    // Should have precedence rules for expressions
    let expr_rules: Vec<_> = g.all_rules().filter(|r| r.precedence.is_some()).collect();
    assert!(expr_rules.len() >= 4);
}

#[test]
fn test_complex_many_alternatives_single_lhs() {
    let mut builder = GrammarBuilder::new("many_alt");
    builder = builder.token("A", "a");
    for i in 0..20 {
        let name = format!("t{i}");
        builder = builder.token(&name, &name);
    }
    for i in 0..20 {
        let name = format!("t{i}");
        builder = builder.rule("root", vec![&name]);
    }
    builder = builder.start("root");
    let g = builder.build();
    let root_id = g.find_symbol_by_name("root").unwrap();
    let rules = g.get_rules_for_symbol(root_id).unwrap();
    assert_eq!(rules.len(), 20);
}

#[test]
fn test_complex_deep_nonterminal_chain() {
    let g = GrammarBuilder::new("chain")
        .token("T", "t")
        .rule("a", vec!["b"])
        .rule("b", vec!["c"])
        .rule("c", vec!["leaf"])
        .rule("leaf", vec!["T"])
        .start("a")
        .build();
    assert_eq!(g.rules.len(), 4);
    let a_id = g.find_symbol_by_name("a").unwrap();
    let a_rules = g.get_rules_for_symbol(a_id).unwrap();
    assert!(matches!(a_rules[0].rhs[0], Symbol::NonTerminal(_)));
}

// ═══════════════════════════════════════════════════════════════════════════
// Category 8: Edge cases — epsilon, single token, many (10 tests)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_edge_epsilon_rule() {
    let g = GrammarBuilder::new("eps")
        .rule("empty", vec![])
        .start("empty")
        .build();
    let empty_id = g.find_symbol_by_name("empty").unwrap();
    let rules = g.get_rules_for_symbol(empty_id).unwrap();
    assert_eq!(rules[0].rhs, vec![Symbol::Epsilon]);
}

#[test]
fn test_edge_epsilon_alongside_nonempty() {
    let g = GrammarBuilder::new("eps2")
        .token("T", "t")
        .rule("opt", vec![])
        .rule("opt", vec!["T"])
        .start("opt")
        .build();
    let opt_id = g.find_symbol_by_name("opt").unwrap();
    let rules = g.get_rules_for_symbol(opt_id).unwrap();
    assert_eq!(rules.len(), 2);
    let has_eps = rules.iter().any(|r| r.rhs == vec![Symbol::Epsilon]);
    let has_nonempty = rules
        .iter()
        .any(|r| r.rhs.len() == 1 && matches!(r.rhs[0], Symbol::Terminal(_)));
    assert!(has_eps);
    assert!(has_nonempty);
}

#[test]
fn test_edge_single_token_grammar() {
    let g = GrammarBuilder::new("single")
        .token("T", "t")
        .rule("r", vec!["T"])
        .start("r")
        .build();
    assert_eq!(g.tokens.len(), 1);
    assert_eq!(g.rules.len(), 1);
    assert_eq!(g.all_rules().count(), 1);
}

#[test]
fn test_edge_rule_with_long_rhs() {
    let g = GrammarBuilder::new("long_rhs")
        .token("A", "a")
        .token("B", "b")
        .token("C", "c")
        .token("D", "d")
        .token("E", "e")
        .token("F", "f")
        .token("G", "g")
        .rule("r", vec!["A", "B", "C", "D", "E", "F", "G"])
        .start("r")
        .build();
    let r_id = g.find_symbol_by_name("r").unwrap();
    let rules = g.get_rules_for_symbol(r_id).unwrap();
    assert_eq!(rules[0].rhs.len(), 7);
}

#[test]
fn test_edge_token_with_special_chars() {
    let g = GrammarBuilder::new("special")
        .token("(", "(")
        .token(")", ")")
        .token("{", "{")
        .token("}", "}")
        .token("[", "[")
        .token("]", "]")
        .token(";", ";")
        .token(",", ",")
        .build();
    assert_eq!(g.tokens.len(), 8);
}

#[test]
fn test_edge_same_token_reregistered() {
    // Calling .token() with an existing name reuses the SymbolId
    let g = GrammarBuilder::new("reuse")
        .token("T", "first")
        .token("T", "second")
        .rule("r", vec!["T"])
        .start("r")
        .build();
    // Only 1 token entry since same name → same id → overwritten
    assert_eq!(g.tokens.len(), 1);
}

#[test]
fn test_edge_all_rules_count_with_alternatives() {
    let g = GrammarBuilder::new("alts")
        .token("A", "a")
        .token("B", "b")
        .token("C", "c")
        .rule("x", vec!["A"])
        .rule("x", vec!["B"])
        .rule("x", vec!["C"])
        .rule("x", vec!["A", "B"])
        .rule("x", vec!["A", "C"])
        .start("x")
        .build();
    // 5 productions, all under one LHS
    assert_eq!(g.rules.len(), 1);
    assert_eq!(g.all_rules().count(), 5);
}

#[test]
fn test_edge_grammar_serialization_roundtrip() {
    let g = GrammarBuilder::new("roundtrip")
        .token("N", r"\d+")
        .token("+", "+")
        .rule_with_precedence("e", vec!["e", "+", "e"], 1, Associativity::Left)
        .rule("e", vec!["N"])
        .extra("WS")
        .token("WS", r"\s+")
        .external("EXT")
        .start("e")
        .build();
    let json = serde_json::to_string(&g).expect("serialize");
    let g2: adze_ir::Grammar = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(g.name, g2.name);
    assert_eq!(g.tokens.len(), g2.tokens.len());
    assert_eq!(g.rules.len(), g2.rules.len());
    assert_eq!(g.extras.len(), g2.extras.len());
    assert_eq!(g.externals.len(), g2.externals.len());
}

#[test]
fn test_edge_empty_name() {
    let g = GrammarBuilder::new("").build();
    assert_eq!(g.name, "");
}

#[test]
fn test_edge_chaining_order_irrelevant() {
    // Verify grammar is correct regardless of method call order
    let g1 = GrammarBuilder::new("order1")
        .token("T", "t")
        .rule("r", vec!["T"])
        .start("r")
        .extra("WS")
        .token("WS", r"\s+")
        .external("EXT")
        .build();
    let g2 = GrammarBuilder::new("order2")
        .external("EXT")
        .extra("WS")
        .token("WS", r"\s+")
        .start("r")
        .token("T", "t")
        .rule("r", vec!["T"])
        .build();
    assert_eq!(g1.tokens.len(), g2.tokens.len());
    assert_eq!(g1.rules.len(), g2.rules.len());
    assert_eq!(g1.extras.len(), g2.extras.len());
    assert_eq!(g1.externals.len(), g2.externals.len());
}
