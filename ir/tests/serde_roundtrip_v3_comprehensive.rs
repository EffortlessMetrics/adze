//! Comprehensive serde_json roundtrip tests (v3) for all adze-ir types.
//!
//! 70+ tests covering JSON roundtrips for Grammar, Rule, Symbol, all ID types,
//! PrecedenceKind, Associativity, Token, edge cases, pretty-print, and cross-format.

use adze_ir::builder::GrammarBuilder;
use adze_ir::*;
use serde_json;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn json_roundtrip<
    T: serde::Serialize + serde::de::DeserializeOwned + PartialEq + std::fmt::Debug,
>(
    val: &T,
) {
    let json = serde_json::to_string(val).expect("serialize");
    let back: T = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(*val, back);
}

fn json_pretty_roundtrip<
    T: serde::Serialize + serde::de::DeserializeOwned + PartialEq + std::fmt::Debug,
>(
    val: &T,
) {
    let json = serde_json::to_string_pretty(val).expect("pretty serialize");
    let back: T = serde_json::from_str(&json).expect("pretty deserialize");
    assert_eq!(*val, back);
}

fn make_simple_grammar() -> Grammar {
    GrammarBuilder::new("simple")
        .token("PLUS", "+")
        .token("NUM", "[0-9]+")
        .rule("expr", vec!["NUM", "PLUS", "NUM"])
        .build()
}

// ===========================================================================
// 1. Simple type roundtrips (15 tests)
// ===========================================================================

#[test]
fn test_symbol_id_zero() {
    json_roundtrip(&SymbolId(0));
}

#[test]
fn test_symbol_id_max() {
    json_roundtrip(&SymbolId(u16::MAX));
}

#[test]
fn test_symbol_id_mid() {
    json_roundtrip(&SymbolId(1000));
}

#[test]
fn test_rule_id_zero() {
    json_roundtrip(&RuleId(0));
}

#[test]
fn test_rule_id_max() {
    json_roundtrip(&RuleId(u16::MAX));
}

#[test]
fn test_state_id_zero() {
    json_roundtrip(&StateId(0));
}

#[test]
fn test_state_id_max() {
    json_roundtrip(&StateId(u16::MAX));
}

#[test]
fn test_field_id_zero() {
    json_roundtrip(&FieldId(0));
}

#[test]
fn test_field_id_max() {
    json_roundtrip(&FieldId(u16::MAX));
}

#[test]
fn test_production_id_zero() {
    json_roundtrip(&ProductionId(0));
}

#[test]
fn test_production_id_max() {
    json_roundtrip(&ProductionId(u16::MAX));
}

#[test]
fn test_symbol_terminal() {
    json_roundtrip(&Symbol::Terminal(SymbolId(5)));
}

#[test]
fn test_symbol_nonterminal() {
    json_roundtrip(&Symbol::NonTerminal(SymbolId(10)));
}

#[test]
fn test_symbol_epsilon() {
    json_roundtrip(&Symbol::Epsilon);
}

#[test]
fn test_symbol_external() {
    json_roundtrip(&Symbol::External(SymbolId(42)));
}

// ===========================================================================
// 2. Grammar roundtrip preserves structure (10 tests)
// ===========================================================================

#[test]
fn test_grammar_name_preserved() {
    let grammar = make_simple_grammar();
    let json = serde_json::to_string(&grammar).expect("serialize");
    let back: Grammar = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(grammar.name, back.name);
}

#[test]
fn test_grammar_rules_count_preserved() {
    let grammar = make_simple_grammar();
    let json = serde_json::to_string(&grammar).expect("serialize");
    let back: Grammar = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(grammar.rules.len(), back.rules.len());
}

#[test]
fn test_grammar_tokens_count_preserved() {
    let grammar = make_simple_grammar();
    let json = serde_json::to_string(&grammar).expect("serialize");
    let back: Grammar = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(grammar.tokens.len(), back.tokens.len());
}

#[test]
fn test_grammar_rule_names_preserved() {
    let grammar = make_simple_grammar();
    let json = serde_json::to_string(&grammar).expect("serialize");
    let back: Grammar = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(grammar.rule_names, back.rule_names);
}

#[test]
fn test_grammar_full_equality() {
    let grammar = make_simple_grammar();
    json_roundtrip(&grammar);
}

#[test]
fn test_grammar_extras_preserved() {
    let grammar = GrammarBuilder::new("with_extras")
        .token("WS", "\\s+")
        .token("NUM", "[0-9]+")
        .rule("expr", vec!["NUM"])
        .extra("WS")
        .build();
    let json = serde_json::to_string(&grammar).expect("serialize");
    let back: Grammar = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(grammar.extras, back.extras);
}

#[test]
fn test_grammar_precedences_preserved() {
    let grammar = GrammarBuilder::new("prec_grammar")
        .token("PLUS", "+")
        .token("STAR", "*")
        .token("NUM", "[0-9]+")
        .rule("expr", vec!["NUM", "PLUS", "NUM"])
        .precedence(1, Associativity::Left, vec!["PLUS"])
        .precedence(2, Associativity::Left, vec!["STAR"])
        .build();
    let json = serde_json::to_string(&grammar).expect("serialize");
    let back: Grammar = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(grammar.precedences, back.precedences);
}

#[test]
fn test_grammar_fields_preserved() {
    let grammar = make_simple_grammar();
    let json = serde_json::to_string(&grammar).expect("serialize");
    let back: Grammar = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(grammar.fields, back.fields);
}

#[test]
fn test_grammar_inline_rules_preserved() {
    let grammar = make_simple_grammar();
    let json = serde_json::to_string(&grammar).expect("serialize");
    let back: Grammar = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(grammar.inline_rules, back.inline_rules);
}

#[test]
fn test_grammar_supertypes_preserved() {
    let grammar = make_simple_grammar();
    let json = serde_json::to_string(&grammar).expect("serialize");
    let back: Grammar = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(grammar.supertypes, back.supertypes);
}

// ===========================================================================
// 3. Complex grammar roundtrip (10 tests)
// ===========================================================================

#[test]
fn test_multi_rule_grammar_roundtrip() {
    let grammar = GrammarBuilder::new("multi")
        .token("NUM", "[0-9]+")
        .token("PLUS", "+")
        .token("MINUS", "-")
        .rule("expr", vec!["NUM", "PLUS", "NUM"])
        .rule("sub_expr", vec!["NUM", "MINUS", "NUM"])
        .build();
    json_roundtrip(&grammar);
}

#[test]
fn test_grammar_with_precedence_roundtrip() {
    let grammar = GrammarBuilder::new("prec")
        .token("NUM", "[0-9]+")
        .token("PLUS", "+")
        .token("STAR", "*")
        .rule("expr", vec!["NUM", "PLUS", "NUM"])
        .rule("mul", vec!["NUM", "STAR", "NUM"])
        .precedence(1, Associativity::Left, vec!["PLUS"])
        .precedence(2, Associativity::Left, vec!["STAR"])
        .build();
    json_roundtrip(&grammar);
}

#[test]
fn test_grammar_with_extras_roundtrip() {
    let grammar = GrammarBuilder::new("extras")
        .token("WS", "\\s+")
        .token("NUM", "[0-9]+")
        .token("PLUS", "+")
        .rule("expr", vec!["NUM", "PLUS", "NUM"])
        .extra("WS")
        .build();
    json_roundtrip(&grammar);
}

#[test]
fn test_grammar_with_right_assoc_roundtrip() {
    let grammar = GrammarBuilder::new("right_assoc")
        .token("NUM", "[0-9]+")
        .token("EXP", "**")
        .rule("power", vec!["NUM", "EXP", "NUM"])
        .precedence(1, Associativity::Right, vec!["EXP"])
        .build();
    json_roundtrip(&grammar);
}

#[test]
fn test_grammar_with_none_assoc_roundtrip() {
    let grammar = GrammarBuilder::new("none_assoc")
        .token("NUM", "[0-9]+")
        .token("CMP", "==")
        .rule("comparison", vec!["NUM", "CMP", "NUM"])
        .precedence(1, Associativity::None, vec!["CMP"])
        .build();
    json_roundtrip(&grammar);
}

#[test]
fn test_grammar_with_externals_roundtrip() {
    let grammar = GrammarBuilder::new("ext")
        .token("NUM", "[0-9]+")
        .rule("expr", vec!["NUM"])
        .external("INDENT")
        .build();
    json_roundtrip(&grammar);
}

#[test]
fn test_grammar_with_rule_precedence_roundtrip() {
    let grammar = GrammarBuilder::new("rule_prec")
        .token("NUM", "[0-9]+")
        .token("PLUS", "+")
        .rule_with_precedence("expr", vec!["NUM", "PLUS", "NUM"], 5, Associativity::Left)
        .build();
    json_roundtrip(&grammar);
}

#[test]
fn test_python_like_grammar_roundtrip() {
    let grammar = GrammarBuilder::python_like();
    json_roundtrip(&grammar);
}

#[test]
fn test_javascript_like_grammar_roundtrip() {
    let grammar = GrammarBuilder::javascript_like();
    json_roundtrip(&grammar);
}

#[test]
fn test_complex_grammar_all_features() {
    let grammar = GrammarBuilder::new("complex")
        .token("NUM", "[0-9]+")
        .token("ID", "[a-zA-Z_]+")
        .token("PLUS", "+")
        .token("STAR", "*")
        .token("WS", "\\s+")
        .fragile_token("COMMENT", "//.*")
        .rule("expr", vec!["NUM", "PLUS", "NUM"])
        .rule("mul", vec!["NUM", "STAR", "NUM"])
        .rule("id_expr", vec!["ID"])
        .rule_with_precedence(
            "prec_expr",
            vec!["NUM", "PLUS", "NUM"],
            3,
            Associativity::Left,
        )
        .extra("WS")
        .precedence(1, Associativity::Left, vec!["PLUS"])
        .precedence(2, Associativity::Left, vec!["STAR"])
        .external("INDENT")
        .build();
    json_roundtrip(&grammar);
}

// ===========================================================================
// 4. Token roundtrip (8 tests)
// ===========================================================================

#[test]
fn test_token_string_pattern_roundtrip() {
    let tok = Token {
        name: "PLUS".to_string(),
        pattern: TokenPattern::String("+".to_string()),
        fragile: false,
    };
    json_roundtrip(&tok);
}

#[test]
fn test_token_regex_pattern_roundtrip() {
    let tok = Token {
        name: "NUM".to_string(),
        pattern: TokenPattern::Regex("[0-9]+".to_string()),
        fragile: false,
    };
    json_roundtrip(&tok);
}

#[test]
fn test_token_fragile_roundtrip() {
    let tok = Token {
        name: "COMMENT".to_string(),
        pattern: TokenPattern::Regex("//.*".to_string()),
        fragile: true,
    };
    json_roundtrip(&tok);
}

#[test]
fn test_token_empty_name_roundtrip() {
    let tok = Token {
        name: String::new(),
        pattern: TokenPattern::String(String::new()),
        fragile: false,
    };
    json_roundtrip(&tok);
}

#[test]
fn test_token_complex_regex_roundtrip() {
    let tok = Token {
        name: "STRING".to_string(),
        pattern: TokenPattern::Regex(r#""([^"\\]|\\.)*""#.to_string()),
        fragile: false,
    };
    json_roundtrip(&tok);
}

#[test]
fn test_token_unicode_name_roundtrip() {
    let tok = Token {
        name: "日本語トークン".to_string(),
        pattern: TokenPattern::String("→".to_string()),
        fragile: false,
    };
    json_roundtrip(&tok);
}

#[test]
fn test_token_pattern_string_vs_regex_distinct() {
    let str_tok = Token {
        name: "T".to_string(),
        pattern: TokenPattern::String("abc".to_string()),
        fragile: false,
    };
    let regex_tok = Token {
        name: "T".to_string(),
        pattern: TokenPattern::Regex("abc".to_string()),
        fragile: false,
    };
    let str_json = serde_json::to_string(&str_tok).unwrap();
    let regex_json = serde_json::to_string(&regex_tok).unwrap();
    assert_ne!(str_json, regex_json);
    json_roundtrip(&str_tok);
    json_roundtrip(&regex_tok);
}

#[test]
fn test_token_fragile_flag_preserved() {
    let fragile = Token {
        name: "X".to_string(),
        pattern: TokenPattern::String("x".to_string()),
        fragile: true,
    };
    let not_fragile = Token {
        name: "X".to_string(),
        pattern: TokenPattern::String("x".to_string()),
        fragile: false,
    };
    let j1 = serde_json::to_string(&fragile).unwrap();
    let j2 = serde_json::to_string(&not_fragile).unwrap();
    assert_ne!(j1, j2);
    let back1: Token = serde_json::from_str(&j1).unwrap();
    let back2: Token = serde_json::from_str(&j2).unwrap();
    assert!(back1.fragile);
    assert!(!back2.fragile);
}

// ===========================================================================
// 5. Rule roundtrip (8 tests)
// ===========================================================================

#[test]
fn test_rule_simple_roundtrip() {
    let rule = Rule {
        lhs: SymbolId(0),
        rhs: vec![Symbol::Terminal(SymbolId(1))],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    };
    json_roundtrip(&rule);
}

#[test]
fn test_rule_with_static_precedence_roundtrip() {
    let rule = Rule {
        lhs: SymbolId(0),
        rhs: vec![
            Symbol::NonTerminal(SymbolId(1)),
            Symbol::Terminal(SymbolId(2)),
            Symbol::NonTerminal(SymbolId(3)),
        ],
        precedence: Some(PrecedenceKind::Static(5)),
        associativity: Some(Associativity::Left),
        fields: vec![],
        production_id: ProductionId(1),
    };
    json_roundtrip(&rule);
}

#[test]
fn test_rule_with_dynamic_precedence_roundtrip() {
    let rule = Rule {
        lhs: SymbolId(0),
        rhs: vec![Symbol::Terminal(SymbolId(1))],
        precedence: Some(PrecedenceKind::Dynamic(10)),
        associativity: Some(Associativity::Right),
        fields: vec![],
        production_id: ProductionId(2),
    };
    json_roundtrip(&rule);
}

#[test]
fn test_rule_with_fields_roundtrip() {
    let rule = Rule {
        lhs: SymbolId(0),
        rhs: vec![
            Symbol::Terminal(SymbolId(1)),
            Symbol::NonTerminal(SymbolId(2)),
        ],
        precedence: None,
        associativity: None,
        fields: vec![(FieldId(0), 0), (FieldId(1), 1)],
        production_id: ProductionId(3),
    };
    json_roundtrip(&rule);
}

#[test]
fn test_rule_with_epsilon_rhs_roundtrip() {
    let rule = Rule {
        lhs: SymbolId(0),
        rhs: vec![Symbol::Epsilon],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(4),
    };
    json_roundtrip(&rule);
}

#[test]
fn test_rule_with_none_associativity_roundtrip() {
    let rule = Rule {
        lhs: SymbolId(10),
        rhs: vec![Symbol::Terminal(SymbolId(1)), Symbol::Terminal(SymbolId(2))],
        precedence: Some(PrecedenceKind::Static(1)),
        associativity: Some(Associativity::None),
        fields: vec![],
        production_id: ProductionId(5),
    };
    json_roundtrip(&rule);
}

#[test]
fn test_rule_long_rhs_roundtrip() {
    let rhs: Vec<Symbol> = (0..20).map(|i| Symbol::Terminal(SymbolId(i))).collect();
    let rule = Rule {
        lhs: SymbolId(100),
        rhs,
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(6),
    };
    json_roundtrip(&rule);
}

#[test]
fn test_rule_mixed_symbols_roundtrip() {
    let rule = Rule {
        lhs: SymbolId(0),
        rhs: vec![
            Symbol::Terminal(SymbolId(1)),
            Symbol::NonTerminal(SymbolId(2)),
            Symbol::External(SymbolId(3)),
            Symbol::Epsilon,
        ],
        precedence: Some(PrecedenceKind::Dynamic(-1)),
        associativity: Some(Associativity::Left),
        fields: vec![(FieldId(0), 0), (FieldId(2), 2)],
        production_id: ProductionId(7),
    };
    json_roundtrip(&rule);
}

// ===========================================================================
// 6. Pretty JSON roundtrip (5 tests)
// ===========================================================================

#[test]
fn test_grammar_pretty_roundtrip() {
    let grammar = make_simple_grammar();
    json_pretty_roundtrip(&grammar);
}

#[test]
fn test_rule_pretty_roundtrip() {
    let rule = Rule {
        lhs: SymbolId(0),
        rhs: vec![
            Symbol::Terminal(SymbolId(1)),
            Symbol::NonTerminal(SymbolId(2)),
        ],
        precedence: Some(PrecedenceKind::Static(3)),
        associativity: Some(Associativity::Left),
        fields: vec![(FieldId(0), 0)],
        production_id: ProductionId(0),
    };
    json_pretty_roundtrip(&rule);
}

#[test]
fn test_token_pretty_roundtrip() {
    let tok = Token {
        name: "IDENT".to_string(),
        pattern: TokenPattern::Regex("[a-z]+".to_string()),
        fragile: false,
    };
    json_pretty_roundtrip(&tok);
}

#[test]
fn test_symbol_pretty_roundtrip() {
    let sym = Symbol::Optional(Box::new(Symbol::Terminal(SymbolId(5))));
    json_pretty_roundtrip(&sym);
}

#[test]
fn test_complex_grammar_pretty_roundtrip() {
    let grammar = GrammarBuilder::new("pretty_test")
        .token("NUM", "[0-9]+")
        .token("PLUS", "+")
        .token("STAR", "*")
        .token("WS", "\\s+")
        .rule("expr", vec!["NUM", "PLUS", "NUM"])
        .rule("mul", vec!["NUM", "STAR", "NUM"])
        .extra("WS")
        .precedence(1, Associativity::Left, vec!["PLUS"])
        .precedence(2, Associativity::Left, vec!["STAR"])
        .build();
    json_pretty_roundtrip(&grammar);
}

// ===========================================================================
// 7. Cross-format: compact → pretty → compact (5 tests)
// ===========================================================================

fn cross_format_roundtrip<
    T: serde::Serialize + serde::de::DeserializeOwned + PartialEq + std::fmt::Debug,
>(
    val: &T,
) {
    let compact = serde_json::to_string(val).expect("compact");
    let parsed: T = serde_json::from_str(&compact).expect("from compact");
    let pretty = serde_json::to_string_pretty(&parsed).expect("pretty");
    let parsed2: T = serde_json::from_str(&pretty).expect("from pretty");
    let compact2 = serde_json::to_string(&parsed2).expect("compact2");
    assert_eq!(compact, compact2);
    assert_eq!(*val, parsed2);
}

#[test]
fn test_cross_format_grammar() {
    cross_format_roundtrip(&make_simple_grammar());
}

#[test]
fn test_cross_format_rule() {
    let rule = Rule {
        lhs: SymbolId(0),
        rhs: vec![
            Symbol::Terminal(SymbolId(1)),
            Symbol::NonTerminal(SymbolId(2)),
        ],
        precedence: Some(PrecedenceKind::Static(1)),
        associativity: Some(Associativity::Left),
        fields: vec![],
        production_id: ProductionId(0),
    };
    cross_format_roundtrip(&rule);
}

#[test]
fn test_cross_format_token() {
    let tok = Token {
        name: "NUM".to_string(),
        pattern: TokenPattern::Regex("[0-9]+".to_string()),
        fragile: false,
    };
    cross_format_roundtrip(&tok);
}

#[test]
fn test_cross_format_symbol_nested() {
    let sym = Symbol::Repeat(Box::new(Symbol::Choice(vec![
        Symbol::Terminal(SymbolId(1)),
        Symbol::NonTerminal(SymbolId(2)),
    ])));
    cross_format_roundtrip(&sym);
}

#[test]
fn test_cross_format_python_like() {
    cross_format_roundtrip(&GrammarBuilder::python_like());
}

// ===========================================================================
// 8. Edge cases (9 tests)
// ===========================================================================

#[test]
fn test_empty_grammar_roundtrip() {
    let grammar = Grammar::default();
    json_roundtrip(&grammar);
}

#[test]
fn test_grammar_unicode_name_roundtrip() {
    let grammar = GrammarBuilder::new("日本語文法")
        .token("数字", "[0-9]+")
        .rule("式", vec!["数字"])
        .build();
    json_roundtrip(&grammar);
}

#[test]
fn test_grammar_emoji_name_roundtrip() {
    let grammar = GrammarBuilder::new("🦀grammar🦀")
        .token("TOK", "x")
        .rule("start", vec!["TOK"])
        .build();
    json_roundtrip(&grammar);
}

#[test]
fn test_large_grammar_roundtrip() {
    let mut builder = GrammarBuilder::new("large");
    for i in 0..50 {
        builder = builder.token(&format!("T{i}"), &format!("t{i}"));
    }
    builder = builder.rule("start", vec!["T0"]);
    let grammar = builder.build();
    json_roundtrip(&grammar);
}

#[test]
fn test_many_rules_grammar_roundtrip() {
    let mut builder = GrammarBuilder::new("many_rules");
    builder = builder.token("A", "a").token("B", "b").token("C", "c");
    for i in 0..20 {
        let name = format!("rule_{i}");
        builder = builder.rule(&name, vec!["A", "B", "C"]);
    }
    let grammar = builder.build();
    json_roundtrip(&grammar);
}

#[test]
fn test_deeply_nested_symbol_roundtrip() {
    let sym = Symbol::Optional(Box::new(Symbol::Repeat(Box::new(Symbol::RepeatOne(
        Box::new(Symbol::Choice(vec![
            Symbol::Terminal(SymbolId(1)),
            Symbol::Sequence(vec![
                Symbol::NonTerminal(SymbolId(2)),
                Symbol::External(SymbolId(3)),
            ]),
        ])),
    )))));
    json_roundtrip(&sym);
}

#[test]
fn test_symbol_choice_empty_roundtrip() {
    let sym = Symbol::Choice(vec![]);
    json_roundtrip(&sym);
}

#[test]
fn test_symbol_sequence_empty_roundtrip() {
    let sym = Symbol::Sequence(vec![]);
    json_roundtrip(&sym);
}

#[test]
fn test_json_value_intermediate_roundtrip() {
    // Serialize to serde_json::Value, then back to Grammar
    let grammar = make_simple_grammar();
    let value = serde_json::to_value(&grammar).expect("to value");
    let back: Grammar = serde_json::from_value(value).expect("from value");
    assert_eq!(grammar, back);
}

// ===========================================================================
// Additional type roundtrips to reach 70+ tests
// ===========================================================================

#[test]
fn test_precedence_kind_static_roundtrip() {
    json_roundtrip(&PrecedenceKind::Static(0));
    json_roundtrip(&PrecedenceKind::Static(-1));
    json_roundtrip(&PrecedenceKind::Static(i16::MAX));
}

#[test]
fn test_precedence_kind_dynamic_roundtrip() {
    json_roundtrip(&PrecedenceKind::Dynamic(0));
    json_roundtrip(&PrecedenceKind::Dynamic(-100));
    json_roundtrip(&PrecedenceKind::Dynamic(i16::MIN));
}

#[test]
fn test_associativity_left_roundtrip() {
    json_roundtrip(&Associativity::Left);
}

#[test]
fn test_associativity_right_roundtrip() {
    json_roundtrip(&Associativity::Right);
}

#[test]
fn test_associativity_none_roundtrip() {
    json_roundtrip(&Associativity::None);
}

#[test]
fn test_precedence_struct_roundtrip() {
    let prec = Precedence {
        level: 5,
        associativity: Associativity::Left,
        symbols: vec![SymbolId(1), SymbolId(2)],
    };
    json_roundtrip(&prec);
}

#[test]
fn test_conflict_declaration_roundtrip() {
    let cd = ConflictDeclaration {
        symbols: vec![SymbolId(1), SymbolId(2)],
        resolution: ConflictResolution::Precedence(PrecedenceKind::Static(3)),
    };
    json_roundtrip(&cd);
}

#[test]
fn test_conflict_resolution_assoc_roundtrip() {
    let cd = ConflictDeclaration {
        symbols: vec![SymbolId(10)],
        resolution: ConflictResolution::Associativity(Associativity::Right),
    };
    json_roundtrip(&cd);
}

#[test]
fn test_conflict_resolution_glr_roundtrip() {
    let cd = ConflictDeclaration {
        symbols: vec![SymbolId(1), SymbolId(2), SymbolId(3)],
        resolution: ConflictResolution::GLR,
    };
    json_roundtrip(&cd);
}

#[test]
fn test_external_token_roundtrip() {
    let ext = ExternalToken {
        name: "INDENT".to_string(),
        symbol_id: SymbolId(99),
    };
    json_roundtrip(&ext);
}

#[test]
fn test_alias_sequence_roundtrip() {
    let alias = AliasSequence {
        aliases: vec![Some("alias1".to_string()), None, Some("alias2".to_string())],
    };
    json_roundtrip(&alias);
}

#[test]
fn test_alias_sequence_empty_roundtrip() {
    let alias = AliasSequence { aliases: vec![] };
    json_roundtrip(&alias);
}

#[test]
fn test_token_pattern_string_roundtrip() {
    json_roundtrip(&TokenPattern::String("hello".to_string()));
}

#[test]
fn test_token_pattern_regex_roundtrip() {
    json_roundtrip(&TokenPattern::Regex("[a-z]+".to_string()));
}

#[test]
fn test_symbol_optional_roundtrip() {
    json_roundtrip(&Symbol::Optional(Box::new(Symbol::Terminal(SymbolId(1)))));
}

#[test]
fn test_symbol_repeat_roundtrip() {
    json_roundtrip(&Symbol::Repeat(Box::new(Symbol::NonTerminal(SymbolId(2)))));
}

#[test]
fn test_symbol_repeat_one_roundtrip() {
    json_roundtrip(&Symbol::RepeatOne(Box::new(Symbol::Terminal(SymbolId(3)))));
}

#[test]
fn test_symbol_choice_roundtrip() {
    json_roundtrip(&Symbol::Choice(vec![
        Symbol::Terminal(SymbolId(1)),
        Symbol::NonTerminal(SymbolId(2)),
        Symbol::Epsilon,
    ]));
}

#[test]
fn test_symbol_sequence_roundtrip() {
    json_roundtrip(&Symbol::Sequence(vec![
        Symbol::Terminal(SymbolId(1)),
        Symbol::NonTerminal(SymbolId(2)),
    ]));
}
