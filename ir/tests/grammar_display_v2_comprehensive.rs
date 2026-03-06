use adze_ir::builder::GrammarBuilder;
use adze_ir::{
    AliasSequence, Associativity, ConflictDeclaration, ConflictResolution, ExternalToken, FieldId,
    Grammar, Precedence, PrecedenceKind, ProductionId, Rule, RuleId, StateId, Symbol, SymbolId,
    SymbolMetadata, Token, TokenPattern,
};

// ── Helpers ──────────────────────────────────────────────────────────────────

fn simple_grammar() -> Grammar {
    GrammarBuilder::new("simple")
        .token("NUM", r"\d+")
        .token("+", "+")
        .rule("expr", vec!["NUM", "+", "NUM"])
        .start("expr")
        .build()
}

fn calc_grammar() -> Grammar {
    GrammarBuilder::new("calc")
        .token("NUMBER", r"\d+")
        .token("+", "+")
        .token("*", "*")
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "*", "expr"], 2, Associativity::Left)
        .rule("expr", vec!["NUMBER"])
        .start("expr")
        .build()
}

fn multi_rule_grammar() -> Grammar {
    GrammarBuilder::new("multi")
        .token("ID", r"[a-z]+")
        .token(";", ";")
        .token("(", "(")
        .token(")", ")")
        .rule("program", vec!["stmt"])
        .rule("program", vec!["program", "stmt"])
        .rule("stmt", vec!["expr", ";"])
        .rule("expr", vec!["ID"])
        .rule("expr", vec!["(", "expr", ")"])
        .start("program")
        .build()
}

// ===========================================================================
// 1. Grammar Debug format (8 tests)
// ===========================================================================

#[test]
fn test_grammar_debug_contains_name() {
    let g = simple_grammar();
    let dbg = format!("{g:?}");
    assert!(
        dbg.contains("simple"),
        "Debug output should contain grammar name"
    );
}

#[test]
fn test_grammar_debug_contains_rules_key() {
    let g = simple_grammar();
    let dbg = format!("{g:?}");
    assert!(dbg.contains("rules"), "Debug output should contain 'rules'");
}

#[test]
fn test_grammar_debug_contains_tokens_key() {
    let g = simple_grammar();
    let dbg = format!("{g:?}");
    assert!(
        dbg.contains("tokens"),
        "Debug output should contain 'tokens'"
    );
}

#[test]
fn test_grammar_debug_pretty_is_multiline() {
    let g = simple_grammar();
    let dbg = format!("{g:#?}");
    assert!(dbg.lines().count() > 1, "Pretty debug should be multiline");
}

#[test]
fn test_grammar_debug_empty_grammar() {
    let g = Grammar::default();
    let dbg = format!("{g:?}");
    assert!(
        dbg.contains("Grammar"),
        "Debug of default grammar should contain 'Grammar'"
    );
}

#[test]
fn test_grammar_debug_contains_precedences() {
    let g = calc_grammar();
    let dbg = format!("{g:?}");
    assert!(
        dbg.contains("precedences"),
        "Debug should contain 'precedences'"
    );
}

#[test]
fn test_grammar_debug_multi_rule_contains_program() {
    let g = multi_rule_grammar();
    let dbg = format!("{g:?}");
    assert!(
        dbg.contains("program"),
        "Debug should contain rule name 'program'"
    );
}

#[test]
fn test_grammar_debug_format_is_deterministic() {
    let g1 = simple_grammar();
    let g2 = simple_grammar();
    assert_eq!(format!("{g1:?}"), format!("{g2:?}"));
}

// ===========================================================================
// 2. Grammar Clone (8 tests)
// ===========================================================================

#[test]
fn test_grammar_clone_equals_original() {
    let g = simple_grammar();
    let cloned = g.clone();
    assert_eq!(g, cloned);
}

#[test]
fn test_grammar_clone_name_matches() {
    let g = calc_grammar();
    let cloned = g.clone();
    assert_eq!(g.name, cloned.name);
}

#[test]
fn test_grammar_clone_tokens_match() {
    let g = calc_grammar();
    let cloned = g.clone();
    assert_eq!(g.tokens.len(), cloned.tokens.len());
}

#[test]
fn test_grammar_clone_rules_match() {
    let g = multi_rule_grammar();
    let cloned = g.clone();
    assert_eq!(g.rules.len(), cloned.rules.len());
}

#[test]
fn test_grammar_clone_is_independent() {
    let g = simple_grammar();
    let mut cloned = g.clone();
    cloned.name = "modified".to_string();
    assert_ne!(g.name, cloned.name);
}

#[test]
fn test_grammar_clone_debug_matches() {
    let g = calc_grammar();
    let cloned = g.clone();
    assert_eq!(format!("{g:?}"), format!("{cloned:?}"));
}

#[test]
fn test_grammar_clone_empty() {
    let g = Grammar::default();
    let cloned = g.clone();
    assert_eq!(g, cloned);
}

#[test]
fn test_grammar_clone_with_externals() {
    let g = GrammarBuilder::new("ext")
        .token("X", "x")
        .external("INDENT")
        .external("DEDENT")
        .rule("top", vec!["X"])
        .start("top")
        .build();
    let cloned = g.clone();
    assert_eq!(g.externals.len(), cloned.externals.len());
    assert_eq!(g, cloned);
}

// ===========================================================================
// 3. Symbol Display/Debug (8 tests)
// ===========================================================================

#[test]
fn test_symbol_terminal_debug() {
    let sym = Symbol::Terminal(SymbolId(0));
    let dbg = format!("{sym:?}");
    assert!(dbg.contains("Terminal"));
    assert!(dbg.contains("SymbolId(0)"));
}

#[test]
fn test_symbol_nonterminal_debug() {
    let sym = Symbol::NonTerminal(SymbolId(5));
    let dbg = format!("{sym:?}");
    assert!(dbg.contains("NonTerminal"));
    assert!(dbg.contains("SymbolId(5)"));
}

#[test]
fn test_symbol_epsilon_debug() {
    let sym = Symbol::Epsilon;
    assert_eq!(format!("{sym:?}"), "Epsilon");
}

#[test]
fn test_symbol_optional_debug() {
    let sym = Symbol::Optional(Box::new(Symbol::Terminal(SymbolId(1))));
    let dbg = format!("{sym:?}");
    assert!(dbg.contains("Optional"));
    assert!(dbg.contains("Terminal"));
}

#[test]
fn test_symbol_repeat_debug() {
    let sym = Symbol::Repeat(Box::new(Symbol::NonTerminal(SymbolId(2))));
    let dbg = format!("{sym:?}");
    assert!(dbg.contains("Repeat"));
}

#[test]
fn test_symbol_repeat_one_debug() {
    let sym = Symbol::RepeatOne(Box::new(Symbol::Terminal(SymbolId(3))));
    let dbg = format!("{sym:?}");
    assert!(dbg.contains("RepeatOne"));
}

#[test]
fn test_symbol_choice_debug() {
    let sym = Symbol::Choice(vec![
        Symbol::Terminal(SymbolId(0)),
        Symbol::Terminal(SymbolId(1)),
    ]);
    let dbg = format!("{sym:?}");
    assert!(dbg.contains("Choice"));
}

#[test]
fn test_symbol_sequence_debug() {
    let sym = Symbol::Sequence(vec![
        Symbol::Terminal(SymbolId(0)),
        Symbol::NonTerminal(SymbolId(1)),
    ]);
    let dbg = format!("{sym:?}");
    assert!(dbg.contains("Sequence"));
    assert!(dbg.contains("Terminal"));
    assert!(dbg.contains("NonTerminal"));
}

// ===========================================================================
// 4. Token Display/Debug (5 tests)
// ===========================================================================

#[test]
fn test_token_debug_string_pattern() {
    let tok = Token {
        name: "PLUS".to_string(),
        pattern: TokenPattern::String("+".to_string()),
        fragile: false,
    };
    let dbg = format!("{tok:?}");
    assert!(dbg.contains("PLUS"));
    assert!(dbg.contains("String"));
}

#[test]
fn test_token_debug_regex_pattern() {
    let tok = Token {
        name: "NUMBER".to_string(),
        pattern: TokenPattern::Regex(r"\d+".to_string()),
        fragile: false,
    };
    let dbg = format!("{tok:?}");
    assert!(dbg.contains("NUMBER"));
    assert!(dbg.contains("Regex"));
}

#[test]
fn test_token_debug_fragile_flag() {
    let tok = Token {
        name: "ERR".to_string(),
        pattern: TokenPattern::String("?".to_string()),
        fragile: true,
    };
    let dbg = format!("{tok:?}");
    assert!(dbg.contains("fragile: true"));
}

#[test]
fn test_token_pattern_string_debug() {
    let pat = TokenPattern::String("hello".to_string());
    let dbg = format!("{pat:?}");
    assert!(dbg.contains("String"));
    assert!(dbg.contains("hello"));
}

#[test]
fn test_token_pattern_regex_debug() {
    let pat = TokenPattern::Regex(r"[a-z]+".to_string());
    let dbg = format!("{pat:?}");
    assert!(dbg.contains("Regex"));
    assert!(dbg.contains("[a-z]+"));
}

// ===========================================================================
// 5. Rule Display/Debug (5 tests)
// ===========================================================================

#[test]
fn test_rule_debug_contains_lhs() {
    let rule = Rule {
        lhs: SymbolId(0),
        rhs: vec![Symbol::Terminal(SymbolId(1))],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    };
    let dbg = format!("{rule:?}");
    assert!(dbg.contains("lhs"));
    assert!(dbg.contains("SymbolId(0)"));
}

#[test]
fn test_rule_debug_contains_rhs() {
    let rule = Rule {
        lhs: SymbolId(0),
        rhs: vec![Symbol::Terminal(SymbolId(1)), Symbol::Terminal(SymbolId(2))],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    };
    let dbg = format!("{rule:?}");
    assert!(dbg.contains("rhs"));
    assert!(dbg.contains("SymbolId(1)"));
    assert!(dbg.contains("SymbolId(2)"));
}

#[test]
fn test_rule_debug_with_precedence() {
    let rule = Rule {
        lhs: SymbolId(0),
        rhs: vec![Symbol::Terminal(SymbolId(1))],
        precedence: Some(PrecedenceKind::Static(5)),
        associativity: Some(Associativity::Left),
        fields: vec![],
        production_id: ProductionId(0),
    };
    let dbg = format!("{rule:?}");
    assert!(dbg.contains("Static"));
    assert!(dbg.contains("Left"));
}

#[test]
fn test_rule_debug_with_fields() {
    let rule = Rule {
        lhs: SymbolId(0),
        rhs: vec![Symbol::Terminal(SymbolId(1))],
        precedence: None,
        associativity: None,
        fields: vec![(FieldId(0), 0)],
        production_id: ProductionId(1),
    };
    let dbg = format!("{rule:?}");
    assert!(dbg.contains("FieldId(0)"));
    assert!(dbg.contains("ProductionId(1)"));
}

#[test]
fn test_rule_debug_empty_rhs() {
    let rule = Rule {
        lhs: SymbolId(10),
        rhs: vec![],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    };
    let dbg = format!("{rule:?}");
    assert!(dbg.contains("SymbolId(10)"));
}

// ===========================================================================
// 6. Various type Debug roundtrips (8 tests)
// ===========================================================================

#[test]
fn test_symbol_id_display_format() {
    assert_eq!(format!("{}", SymbolId(42)), "Symbol(42)");
}

#[test]
fn test_rule_id_display_format() {
    assert_eq!(format!("{}", RuleId(7)), "Rule(7)");
}

#[test]
fn test_state_id_display_format() {
    assert_eq!(format!("{}", StateId(100)), "State(100)");
}

#[test]
fn test_field_id_display_format() {
    assert_eq!(format!("{}", FieldId(3)), "Field(3)");
}

#[test]
fn test_production_id_display_format() {
    assert_eq!(format!("{}", ProductionId(0)), "Production(0)");
}

#[test]
fn test_associativity_left_debug() {
    assert_eq!(format!("{:?}", Associativity::Left), "Left");
}

#[test]
fn test_associativity_right_debug() {
    assert_eq!(format!("{:?}", Associativity::Right), "Right");
}

#[test]
fn test_associativity_none_debug() {
    assert_eq!(format!("{:?}", Associativity::None), "None");
}

// ===========================================================================
// 7. Grammar equality (5 tests)
// ===========================================================================

#[test]
fn test_grammar_equality_same_builder() {
    let g1 = simple_grammar();
    let g2 = simple_grammar();
    assert_eq!(g1, g2);
}

#[test]
fn test_grammar_inequality_different_names() {
    let g1 = GrammarBuilder::new("a")
        .token("X", "x")
        .rule("r", vec!["X"])
        .start("r")
        .build();
    let g2 = GrammarBuilder::new("b")
        .token("X", "x")
        .rule("r", vec!["X"])
        .start("r")
        .build();
    assert_ne!(g1, g2);
}

#[test]
fn test_grammar_inequality_different_tokens() {
    let g1 = GrammarBuilder::new("g")
        .token("A", "a")
        .rule("r", vec!["A"])
        .start("r")
        .build();
    let g2 = GrammarBuilder::new("g")
        .token("B", "b")
        .rule("r", vec!["B"])
        .start("r")
        .build();
    assert_ne!(g1, g2);
}

#[test]
fn test_grammar_equality_default() {
    let g1 = Grammar::default();
    let g2 = Grammar::default();
    assert_eq!(g1, g2);
}

#[test]
fn test_grammar_inequality_extra_rule() {
    let g1 = GrammarBuilder::new("g")
        .token("X", "x")
        .rule("r", vec!["X"])
        .start("r")
        .build();
    let g2 = GrammarBuilder::new("g")
        .token("X", "x")
        .rule("r", vec!["X"])
        .rule("r", vec!["X", "X"])
        .start("r")
        .build();
    assert_ne!(g1, g2);
}

// ===========================================================================
// 8. Edge cases (8 tests)
// ===========================================================================

#[test]
fn test_symbol_id_zero_display() {
    assert_eq!(SymbolId(0).to_string(), "Symbol(0)");
}

#[test]
fn test_symbol_id_max_display() {
    assert_eq!(SymbolId(u16::MAX).to_string(), "Symbol(65535)");
}

#[test]
fn test_grammar_default_name_is_empty() {
    let g = Grammar::default();
    assert!(g.name.is_empty());
}

#[test]
fn test_grammar_default_has_no_rules() {
    let g = Grammar::default();
    assert!(g.rules.is_empty());
}

#[test]
fn test_grammar_default_has_no_tokens() {
    let g = Grammar::default();
    assert!(g.tokens.is_empty());
}

#[test]
fn test_deep_nested_symbol_debug() {
    let sym = Symbol::Optional(Box::new(Symbol::Repeat(Box::new(Symbol::Choice(vec![
        Symbol::Terminal(SymbolId(0)),
        Symbol::Epsilon,
    ])))));
    let dbg = format!("{sym:?}");
    assert!(dbg.contains("Optional"));
    assert!(dbg.contains("Repeat"));
    assert!(dbg.contains("Choice"));
    assert!(dbg.contains("Epsilon"));
}

#[test]
fn test_precedence_kind_static_debug() {
    let pk = PrecedenceKind::Static(10);
    assert_eq!(format!("{pk:?}"), "Static(10)");
}

#[test]
fn test_precedence_kind_dynamic_debug() {
    let pk = PrecedenceKind::Dynamic(-3);
    assert_eq!(format!("{pk:?}"), "Dynamic(-3)");
}

// ===========================================================================
// Bonus: additional coverage to exceed 55 tests
// ===========================================================================

#[test]
fn test_external_token_debug() {
    let ext = ExternalToken {
        name: "INDENT".to_string(),
        symbol_id: SymbolId(99),
    };
    let dbg = format!("{ext:?}");
    assert!(dbg.contains("INDENT"));
    assert!(dbg.contains("SymbolId(99)"));
}

#[test]
fn test_alias_sequence_debug() {
    let seq = AliasSequence {
        aliases: vec![Some("alias1".to_string()), None],
    };
    let dbg = format!("{seq:?}");
    assert!(dbg.contains("alias1"));
    assert!(dbg.contains("None"));
}

#[test]
fn test_conflict_declaration_debug() {
    let cd = ConflictDeclaration {
        symbols: vec![SymbolId(1), SymbolId(2)],
        resolution: ConflictResolution::GLR,
    };
    let dbg = format!("{cd:?}");
    assert!(dbg.contains("GLR"));
    assert!(dbg.contains("SymbolId(1)"));
}

#[test]
fn test_conflict_resolution_precedence_debug() {
    let cr = ConflictResolution::Precedence(PrecedenceKind::Static(5));
    let dbg = format!("{cr:?}");
    assert!(dbg.contains("Precedence"));
    assert!(dbg.contains("Static(5)"));
}

#[test]
fn test_conflict_resolution_associativity_debug() {
    let cr = ConflictResolution::Associativity(Associativity::Right);
    let dbg = format!("{cr:?}");
    assert!(dbg.contains("Associativity"));
    assert!(dbg.contains("Right"));
}

#[test]
fn test_precedence_struct_debug() {
    let p = Precedence {
        level: 3,
        associativity: Associativity::Left,
        symbols: vec![SymbolId(10)],
    };
    let dbg = format!("{p:?}");
    assert!(dbg.contains("level: 3"));
    assert!(dbg.contains("Left"));
}

#[test]
fn test_symbol_metadata_debug() {
    let meta = SymbolMetadata {
        visible: true,
        named: true,
        hidden: false,
        terminal: true,
    };
    let dbg = format!("{meta:?}");
    assert!(dbg.contains("visible: true"));
    assert!(dbg.contains("terminal: true"));
}

#[test]
fn test_grammar_clone_preserves_extras() {
    let g = GrammarBuilder::new("ws")
        .token("X", "x")
        .token("WS", r"\s+")
        .extra("WS")
        .rule("top", vec!["X"])
        .start("top")
        .build();
    let cloned = g.clone();
    assert_eq!(g.extras.len(), cloned.extras.len());
    assert_eq!(g, cloned);
}

#[test]
fn test_grammar_clone_preserves_rule_names() {
    let g = multi_rule_grammar();
    let cloned = g.clone();
    assert_eq!(g.rule_names.len(), cloned.rule_names.len());
    for (id, name) in &g.rule_names {
        assert_eq!(cloned.rule_names.get(id), Some(name));
    }
}

#[test]
fn test_symbol_clone_equality() {
    let original = Symbol::Choice(vec![
        Symbol::Terminal(SymbolId(0)),
        Symbol::NonTerminal(SymbolId(1)),
    ]);
    let cloned = original.clone();
    assert_eq!(original, cloned);
}

#[test]
fn test_token_clone_equality() {
    let original = Token {
        name: "FOO".to_string(),
        pattern: TokenPattern::Regex(r"\w+".to_string()),
        fragile: false,
    };
    let cloned = original.clone();
    assert_eq!(original, cloned);
}

#[test]
fn test_rule_clone_equality() {
    let original = Rule {
        lhs: SymbolId(0),
        rhs: vec![Symbol::Terminal(SymbolId(1)), Symbol::Terminal(SymbolId(2))],
        precedence: Some(PrecedenceKind::Static(1)),
        associativity: Some(Associativity::Left),
        fields: vec![(FieldId(0), 0)],
        production_id: ProductionId(5),
    };
    let cloned = original.clone();
    assert_eq!(original, cloned);
}

#[test]
fn test_symbol_external_debug() {
    let sym = Symbol::External(SymbolId(77));
    let dbg = format!("{sym:?}");
    assert!(dbg.contains("External"));
    assert!(dbg.contains("SymbolId(77)"));
}

#[test]
fn test_grammar_with_fragile_token_debug() {
    let g = GrammarBuilder::new("frag")
        .token("OK", "ok")
        .fragile_token("BAD", "bad")
        .rule("top", vec!["OK"])
        .start("top")
        .build();
    let dbg = format!("{g:?}");
    assert!(dbg.contains("frag"));
    assert!(dbg.contains("fragile: true"));
}

#[test]
fn test_grammar_python_like_debug() {
    let g = GrammarBuilder::python_like();
    let dbg = format!("{g:?}");
    assert!(dbg.contains("python_like"));
}

#[test]
fn test_grammar_javascript_like_debug() {
    let g = GrammarBuilder::javascript_like();
    let dbg = format!("{g:?}");
    assert!(dbg.contains("javascript_like"));
}
