//! Comprehensive serde roundtrip tests (v2) for all adze-ir types.
//!
//! 50+ tests covering JSON and bincode roundtrips for Grammar, Rule, Symbol,
//! all ID types, PrecedenceKind, Associativity, nested symbols, edge cases, etc.

use adze_ir::{
    AliasSequence, Associativity, ConflictDeclaration, ConflictResolution, ExternalToken, FieldId,
    Grammar, Precedence, PrecedenceKind, ProductionId, Rule, RuleId, StateId, Symbol, SymbolId,
    SymbolMetadata, Token, TokenPattern, builder::GrammarBuilder,
};
use indexmap::IndexMap;

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

fn bincode_roundtrip<
    T: serde::Serialize + serde::de::DeserializeOwned + PartialEq + std::fmt::Debug,
>(
    val: &T,
) {
    let bytes = bincode::serialize(val).expect("bincode serialize");
    let back: T = bincode::deserialize(&bytes).expect("bincode deserialize");
    assert_eq!(*val, back);
}

fn both_roundtrip<
    T: serde::Serialize + serde::de::DeserializeOwned + PartialEq + std::fmt::Debug,
>(
    val: &T,
) {
    json_roundtrip(val);
    bincode_roundtrip(val);
}

// ---------------------------------------------------------------------------
// 1-5: ID types
// ---------------------------------------------------------------------------

#[test]
fn test_symbol_id_roundtrip() {
    both_roundtrip(&SymbolId(0));
    both_roundtrip(&SymbolId(1));
    both_roundtrip(&SymbolId(u16::MAX));
}

#[test]
fn test_rule_id_roundtrip() {
    both_roundtrip(&RuleId(0));
    both_roundtrip(&RuleId(u16::MAX));
}

#[test]
fn test_state_id_roundtrip() {
    both_roundtrip(&StateId(0));
    both_roundtrip(&StateId(u16::MAX));
}

#[test]
fn test_field_id_roundtrip() {
    both_roundtrip(&FieldId(0));
    both_roundtrip(&FieldId(u16::MAX));
}

#[test]
fn test_production_id_roundtrip() {
    both_roundtrip(&ProductionId(0));
    both_roundtrip(&ProductionId(u16::MAX));
}

// ---------------------------------------------------------------------------
// 6-12: Symbol variants
// ---------------------------------------------------------------------------

#[test]
fn test_symbol_terminal() {
    both_roundtrip(&Symbol::Terminal(SymbolId(42)));
}

#[test]
fn test_symbol_nonterminal() {
    both_roundtrip(&Symbol::NonTerminal(SymbolId(7)));
}

#[test]
fn test_symbol_external() {
    both_roundtrip(&Symbol::External(SymbolId(99)));
}

#[test]
fn test_symbol_optional() {
    both_roundtrip(&Symbol::Optional(Box::new(Symbol::Terminal(SymbolId(1)))));
}

#[test]
fn test_symbol_repeat() {
    both_roundtrip(&Symbol::Repeat(Box::new(Symbol::NonTerminal(SymbolId(3)))));
}

#[test]
fn test_symbol_repeat_one() {
    both_roundtrip(&Symbol::RepeatOne(Box::new(Symbol::Terminal(SymbolId(5)))));
}

#[test]
fn test_symbol_epsilon() {
    both_roundtrip(&Symbol::Epsilon);
}

// ---------------------------------------------------------------------------
// 13-16: Symbol collections
// ---------------------------------------------------------------------------

#[test]
fn test_symbol_choice_empty() {
    both_roundtrip(&Symbol::Choice(vec![]));
}

#[test]
fn test_symbol_choice_multiple() {
    both_roundtrip(&Symbol::Choice(vec![
        Symbol::Terminal(SymbolId(1)),
        Symbol::NonTerminal(SymbolId(2)),
        Symbol::Epsilon,
    ]));
}

#[test]
fn test_symbol_sequence_empty() {
    both_roundtrip(&Symbol::Sequence(vec![]));
}

#[test]
fn test_symbol_sequence_multiple() {
    both_roundtrip(&Symbol::Sequence(vec![
        Symbol::Terminal(SymbolId(10)),
        Symbol::Terminal(SymbolId(20)),
        Symbol::NonTerminal(SymbolId(30)),
    ]));
}

// ---------------------------------------------------------------------------
// 17-20: Nested symbols
// ---------------------------------------------------------------------------

#[test]
fn test_symbol_optional_of_repeat() {
    both_roundtrip(&Symbol::Optional(Box::new(Symbol::Repeat(Box::new(
        Symbol::Terminal(SymbolId(1)),
    )))));
}

#[test]
fn test_symbol_choice_of_sequences() {
    both_roundtrip(&Symbol::Choice(vec![
        Symbol::Sequence(vec![
            Symbol::Terminal(SymbolId(1)),
            Symbol::Terminal(SymbolId(2)),
        ]),
        Symbol::Sequence(vec![Symbol::NonTerminal(SymbolId(3))]),
    ]));
}

#[test]
fn test_symbol_deeply_nested_3_levels() {
    let sym = Symbol::Optional(Box::new(Symbol::Repeat(Box::new(Symbol::Choice(vec![
        Symbol::Terminal(SymbolId(1)),
        Symbol::Epsilon,
    ])))));
    both_roundtrip(&sym);
}

#[test]
fn test_symbol_deeply_nested_5_levels() {
    let sym = Symbol::Sequence(vec![Symbol::Optional(Box::new(Symbol::RepeatOne(
        Box::new(Symbol::Choice(vec![Symbol::Repeat(Box::new(
            Symbol::Terminal(SymbolId(42)),
        ))])),
    )))]);
    both_roundtrip(&sym);
}

// ---------------------------------------------------------------------------
// 21-23: PrecedenceKind
// ---------------------------------------------------------------------------

#[test]
fn test_precedence_kind_static() {
    both_roundtrip(&PrecedenceKind::Static(0));
    both_roundtrip(&PrecedenceKind::Static(i16::MAX));
    both_roundtrip(&PrecedenceKind::Static(i16::MIN));
}

#[test]
fn test_precedence_kind_dynamic() {
    both_roundtrip(&PrecedenceKind::Dynamic(1));
    both_roundtrip(&PrecedenceKind::Dynamic(-1));
}

#[test]
fn test_precedence_kind_option() {
    both_roundtrip(&Some(PrecedenceKind::Static(5)));
    both_roundtrip(&None::<PrecedenceKind>);
}

// ---------------------------------------------------------------------------
// 24-26: Associativity
// ---------------------------------------------------------------------------

#[test]
fn test_associativity_left() {
    both_roundtrip(&Associativity::Left);
}

#[test]
fn test_associativity_right() {
    both_roundtrip(&Associativity::Right);
}

#[test]
fn test_associativity_none() {
    both_roundtrip(&Associativity::None);
}

// ---------------------------------------------------------------------------
// 27-28: Token and TokenPattern
// ---------------------------------------------------------------------------

#[test]
fn test_token_string_pattern() {
    let tok = Token {
        name: "PLUS".into(),
        pattern: TokenPattern::String("+".into()),
        fragile: false,
    };
    both_roundtrip(&tok);
}

#[test]
fn test_token_regex_pattern_fragile() {
    let tok = Token {
        name: "NUMBER".into(),
        pattern: TokenPattern::Regex(r"\d+".into()),
        fragile: true,
    };
    both_roundtrip(&tok);
}

// ---------------------------------------------------------------------------
// 29-30: Rule
// ---------------------------------------------------------------------------

#[test]
fn test_rule_simple() {
    let rule = Rule {
        lhs: SymbolId(1),
        rhs: vec![Symbol::Terminal(SymbolId(2))],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    };
    both_roundtrip(&rule);
}

#[test]
fn test_rule_with_precedence_and_fields() {
    let rule = Rule {
        lhs: SymbolId(10),
        rhs: vec![
            Symbol::NonTerminal(SymbolId(10)),
            Symbol::Terminal(SymbolId(11)),
            Symbol::NonTerminal(SymbolId(10)),
        ],
        precedence: Some(PrecedenceKind::Static(2)),
        associativity: Some(Associativity::Left),
        fields: vec![(FieldId(0), 0), (FieldId(1), 2)],
        production_id: ProductionId(3),
    };
    both_roundtrip(&rule);
}

// ---------------------------------------------------------------------------
// 31-32: Precedence and ConflictDeclaration
// ---------------------------------------------------------------------------

#[test]
fn test_precedence_struct() {
    let prec = Precedence {
        level: 5,
        associativity: Associativity::Right,
        symbols: vec![SymbolId(1), SymbolId(2)],
    };
    both_roundtrip(&prec);
}

#[test]
fn test_conflict_declaration_variants() {
    let conflicts = vec![
        ConflictDeclaration {
            symbols: vec![SymbolId(1), SymbolId(2)],
            resolution: ConflictResolution::GLR,
        },
        ConflictDeclaration {
            symbols: vec![SymbolId(3)],
            resolution: ConflictResolution::Precedence(PrecedenceKind::Dynamic(3)),
        },
        ConflictDeclaration {
            symbols: vec![],
            resolution: ConflictResolution::Associativity(Associativity::None),
        },
    ];
    both_roundtrip(&conflicts);
}

// ---------------------------------------------------------------------------
// 33-34: ExternalToken and AliasSequence
// ---------------------------------------------------------------------------

#[test]
fn test_external_token() {
    let ext = ExternalToken {
        name: "INDENT".into(),
        symbol_id: SymbolId(100),
    };
    both_roundtrip(&ext);
}

#[test]
fn test_alias_sequence() {
    let seq = AliasSequence {
        aliases: vec![Some("identifier".into()), None, Some("type".into())],
    };
    both_roundtrip(&seq);
}

// ---------------------------------------------------------------------------
// 35: SymbolMetadata
// ---------------------------------------------------------------------------

#[test]
fn test_symbol_metadata() {
    let meta = SymbolMetadata {
        visible: true,
        named: false,
        hidden: true,
        terminal: true,
    };
    both_roundtrip(&meta);
}

// ---------------------------------------------------------------------------
// 36-40: Grammar via builder – roundtrip
// ---------------------------------------------------------------------------

#[test]
fn test_grammar_minimal_builder() {
    let g = GrammarBuilder::new("test")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    both_roundtrip(&g);
}

#[test]
fn test_grammar_default_empty() {
    let g = Grammar::default();
    both_roundtrip(&g);
}

#[test]
fn test_grammar_new_empty() {
    let g = Grammar::new("empty".into());
    both_roundtrip(&g);
}

#[test]
fn test_grammar_multiple_rules() {
    let g = GrammarBuilder::new("multi")
        .token("NUM", r"\d+")
        .token("+", "+")
        .token("*", "*")
        .rule("expr", vec!["expr", "+", "expr"])
        .rule("expr", vec!["expr", "*", "expr"])
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    both_roundtrip(&g);
}

#[test]
fn test_grammar_with_extras_and_externals() {
    let g = GrammarBuilder::new("ext")
        .token("WS", r"\s+")
        .token("INDENT", "INDENT")
        .token("a", "a")
        .extra("WS")
        .external("INDENT")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    both_roundtrip(&g);
}

// ---------------------------------------------------------------------------
// 41-42: Grammar with precedence rules
// ---------------------------------------------------------------------------

#[test]
fn test_grammar_with_precedence_rules() {
    let g = GrammarBuilder::new("prec")
        .token("NUM", r"\d+")
        .token("+", "+")
        .token("*", "*")
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "*", "expr"], 2, Associativity::Left)
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    both_roundtrip(&g);
}

#[test]
fn test_grammar_python_like() {
    let g = GrammarBuilder::python_like();
    both_roundtrip(&g);
}

// ---------------------------------------------------------------------------
// 43-44: Grammar with fields and alias sequences (manual construction)
// ---------------------------------------------------------------------------

#[test]
fn test_grammar_with_fields() {
    let mut g = GrammarBuilder::new("fields")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a", "b"])
        .start("s")
        .build();
    g.fields.insert(FieldId(0), "alpha".into());
    g.fields.insert(FieldId(1), "beta".into());
    both_roundtrip(&g);
}

#[test]
fn test_grammar_with_alias_sequences() {
    let mut g = GrammarBuilder::new("alias")
        .token("x", "x")
        .rule("s", vec!["x"])
        .start("s")
        .build();
    g.alias_sequences.insert(
        ProductionId(0),
        AliasSequence {
            aliases: vec![Some("ident".into())],
        },
    );
    both_roundtrip(&g);
}

// ---------------------------------------------------------------------------
// 45: Grammar javascript-like preset
// ---------------------------------------------------------------------------

#[test]
fn test_grammar_javascript_like() {
    let g = GrammarBuilder::javascript_like();
    both_roundtrip(&g);
}

// ---------------------------------------------------------------------------
// 46-48: Edge cases – max u16, empty collections, unicode
// ---------------------------------------------------------------------------

#[test]
fn test_id_types_max_u16() {
    let ids: (SymbolId, RuleId, StateId, FieldId, ProductionId) = (
        SymbolId(u16::MAX),
        RuleId(u16::MAX),
        StateId(u16::MAX),
        FieldId(u16::MAX),
        ProductionId(u16::MAX),
    );
    both_roundtrip(&ids);
}

#[test]
fn test_symbol_vec_empty() {
    let v: Vec<Symbol> = vec![];
    both_roundtrip(&v);
}

#[test]
fn test_grammar_unicode_name() {
    let g = Grammar::new("日本語文法".into());
    both_roundtrip(&g);
}

// ---------------------------------------------------------------------------
// 49: Large symbol vector
// ---------------------------------------------------------------------------

#[test]
fn test_large_symbol_vector() {
    let syms: Vec<Symbol> = (0u16..200)
        .map(|i| {
            if i % 2 == 0 {
                Symbol::Terminal(SymbolId(i))
            } else {
                Symbol::NonTerminal(SymbolId(i))
            }
        })
        .collect();
    both_roundtrip(&syms);
}

// ---------------------------------------------------------------------------
// 50: JSON pretty-print roundtrip preserves equality
// ---------------------------------------------------------------------------

#[test]
fn test_json_pretty_roundtrip() {
    let g = GrammarBuilder::new("pretty")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let pretty = serde_json::to_string_pretty(&g).expect("pretty serialize");
    let back: Grammar = serde_json::from_str(&pretty).expect("pretty deserialize");
    assert_eq!(g, back);
}

// ---------------------------------------------------------------------------
// 51-52: ConflictResolution variants
// ---------------------------------------------------------------------------

#[test]
fn test_conflict_resolution_glr() {
    both_roundtrip(&ConflictResolution::GLR);
}

#[test]
fn test_conflict_resolution_precedence_and_assoc() {
    both_roundtrip(&ConflictResolution::Precedence(PrecedenceKind::Static(-10)));
    both_roundtrip(&ConflictResolution::Associativity(Associativity::Right));
}

// ---------------------------------------------------------------------------
// 53: Rule with dynamic precedence
// ---------------------------------------------------------------------------

#[test]
fn test_rule_dynamic_precedence() {
    let rule = Rule {
        lhs: SymbolId(5),
        rhs: vec![
            Symbol::Terminal(SymbolId(6)),
            Symbol::NonTerminal(SymbolId(7)),
        ],
        precedence: Some(PrecedenceKind::Dynamic(-3)),
        associativity: Some(Associativity::Right),
        fields: vec![(FieldId(u16::MAX), 0)],
        production_id: ProductionId(99),
    };
    both_roundtrip(&rule);
}

// ---------------------------------------------------------------------------
// 54: Multiple rules for same LHS
// ---------------------------------------------------------------------------

#[test]
fn test_grammar_multiple_alternatives() {
    let g = GrammarBuilder::new("alt")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("s", vec!["a"])
        .rule("s", vec!["b"])
        .rule("s", vec!["c"])
        .rule("s", vec!["a", "b", "c"])
        .start("s")
        .build();
    both_roundtrip(&g);
}

// ---------------------------------------------------------------------------
// 55: Grammar with empty (epsilon) rule
// ---------------------------------------------------------------------------

#[test]
fn test_grammar_epsilon_rule() {
    let g = GrammarBuilder::new("eps")
        .token("a", "a")
        .rule("s", vec![])
        .rule("s", vec!["a"])
        .start("s")
        .build();
    both_roundtrip(&g);
}

// ---------------------------------------------------------------------------
// 56: Bincode size is smaller than JSON
// ---------------------------------------------------------------------------

#[test]
fn test_bincode_smaller_than_json() {
    let g = GrammarBuilder::new("size")
        .token("NUM", r"\d+")
        .token("+", "+")
        .rule("expr", vec!["NUM", "+", "NUM"])
        .start("expr")
        .build();
    let json_len = serde_json::to_string(&g).unwrap().len();
    let bin_len = bincode::serialize(&g).unwrap().len();
    assert!(
        bin_len < json_len,
        "bincode ({bin_len}) should be smaller than json ({json_len})"
    );
}

// ---------------------------------------------------------------------------
// 57: Roundtrip stability – double roundtrip
// ---------------------------------------------------------------------------

#[test]
fn test_double_json_roundtrip() {
    let g = GrammarBuilder::new("double")
        .token("x", "x")
        .rule("s", vec!["x"])
        .start("s")
        .build();
    let j1 = serde_json::to_string(&g).unwrap();
    let g2: Grammar = serde_json::from_str(&j1).unwrap();
    let j2 = serde_json::to_string(&g2).unwrap();
    let g3: Grammar = serde_json::from_str(&j2).unwrap();
    assert_eq!(g, g2);
    assert_eq!(g2, g3);
    assert_eq!(j1, j2);
}

// ---------------------------------------------------------------------------
// 58: Token pattern variants
// ---------------------------------------------------------------------------

#[test]
fn test_token_pattern_variants() {
    both_roundtrip(&TokenPattern::String("hello".into()));
    both_roundtrip(&TokenPattern::Regex(r"[a-z]+".into()));
    both_roundtrip(&TokenPattern::String(String::new()));
    both_roundtrip(&TokenPattern::Regex(String::new()));
}

// ---------------------------------------------------------------------------
// 59: Vec of rules roundtrip
// ---------------------------------------------------------------------------

#[test]
fn test_vec_of_rules() {
    let rules = vec![
        Rule {
            lhs: SymbolId(1),
            rhs: vec![Symbol::Epsilon],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(0),
        },
        Rule {
            lhs: SymbolId(1),
            rhs: vec![
                Symbol::Terminal(SymbolId(2)),
                Symbol::NonTerminal(SymbolId(1)),
            ],
            precedence: Some(PrecedenceKind::Static(1)),
            associativity: Some(Associativity::Left),
            fields: vec![(FieldId(0), 0)],
            production_id: ProductionId(1),
        },
    ];
    both_roundtrip(&rules);
}

// ---------------------------------------------------------------------------
// 60: IndexMap ordering preservation
// ---------------------------------------------------------------------------

#[test]
fn test_indexmap_order_preserved() {
    let mut g = Grammar::new("ordered".into());
    // Insert fields in a specific order
    g.fields.insert(FieldId(0), "alpha".into());
    g.fields.insert(FieldId(1), "beta".into());
    g.fields.insert(FieldId(2), "gamma".into());

    let json = serde_json::to_string(&g).unwrap();
    let back: Grammar = serde_json::from_str(&json).unwrap();

    let keys: Vec<_> = back.fields.keys().copied().collect();
    assert_eq!(keys, vec![FieldId(0), FieldId(1), FieldId(2)]);
    let vals: Vec<_> = back.fields.values().cloned().collect();
    assert_eq!(vals, vec!["alpha", "beta", "gamma"]);
}
