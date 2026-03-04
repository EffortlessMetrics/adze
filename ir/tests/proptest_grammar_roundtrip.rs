//! Property-based tests for Grammar roundtrip serialization.
//!
//! 40+ tests covering JSON roundtrip fidelity, clone equality,
//! double-roundtrip stability, and builder-constructed grammars.

use adze_ir::builder::GrammarBuilder;
use adze_ir::{
    AliasSequence, Associativity, ConflictDeclaration, ConflictResolution, ExternalToken, FieldId,
    Grammar, Precedence, PrecedenceKind, ProductionId, Rule, Symbol, SymbolId, Token, TokenPattern,
};
use proptest::prelude::*;

// ===========================================================================
// Strategies
// ===========================================================================

fn arb_symbol_id() -> impl Strategy<Value = SymbolId> {
    (1u16..200).prop_map(SymbolId)
}

fn arb_field_id() -> impl Strategy<Value = FieldId> {
    (0u16..50).prop_map(FieldId)
}

fn arb_production_id() -> impl Strategy<Value = ProductionId> {
    (0u16..50).prop_map(ProductionId)
}

fn arb_associativity() -> impl Strategy<Value = Associativity> {
    prop_oneof![
        Just(Associativity::Left),
        Just(Associativity::Right),
        Just(Associativity::None),
    ]
}

fn arb_precedence_kind() -> impl Strategy<Value = PrecedenceKind> {
    prop_oneof![
        (-50i16..50).prop_map(PrecedenceKind::Static),
        (-50i16..50).prop_map(PrecedenceKind::Dynamic),
    ]
}

fn arb_token_pattern() -> impl Strategy<Value = TokenPattern> {
    prop_oneof![
        "[a-zA-Z0-9]{1,8}".prop_map(TokenPattern::String),
        "[a-zA-Z0-9.+*]{1,8}".prop_map(TokenPattern::Regex),
    ]
}

fn arb_symbol_leaf() -> impl Strategy<Value = Symbol> {
    prop_oneof![
        arb_symbol_id().prop_map(Symbol::Terminal),
        arb_symbol_id().prop_map(Symbol::NonTerminal),
        arb_symbol_id().prop_map(Symbol::External),
        Just(Symbol::Epsilon),
    ]
}

fn arb_symbol() -> impl Strategy<Value = Symbol> {
    arb_symbol_leaf().prop_recursive(2, 10, 3, |inner| {
        prop_oneof![
            inner.clone().prop_map(|s| Symbol::Optional(Box::new(s))),
            inner.clone().prop_map(|s| Symbol::Repeat(Box::new(s))),
            inner.clone().prop_map(|s| Symbol::RepeatOne(Box::new(s))),
            prop::collection::vec(inner.clone(), 1..3).prop_map(Symbol::Choice),
            prop::collection::vec(inner, 1..3).prop_map(Symbol::Sequence),
        ]
    })
}

fn arb_token() -> impl Strategy<Value = Token> {
    (
        "[a-zA-Z_][a-zA-Z0-9_]{0,6}",
        arb_token_pattern(),
        any::<bool>(),
    )
        .prop_map(|(name, pattern, fragile)| Token {
            name,
            pattern,
            fragile,
        })
}

fn arb_rule() -> impl Strategy<Value = Rule> {
    (
        arb_symbol_id(),
        prop::collection::vec(arb_symbol(), 1..5),
        prop::option::of(arb_precedence_kind()),
        prop::option::of(arb_associativity()),
        prop::collection::vec((arb_field_id(), 0usize..8), 0..3),
        arb_production_id(),
    )
        .prop_map(
            |(lhs, rhs, precedence, associativity, fields, production_id)| Rule {
                lhs,
                rhs,
                precedence,
                associativity,
                fields,
                production_id,
            },
        )
}

fn arb_external_token() -> impl Strategy<Value = ExternalToken> {
    ("[a-zA-Z_][a-zA-Z0-9_]{0,6}", arb_symbol_id())
        .prop_map(|(name, symbol_id)| ExternalToken { name, symbol_id })
}

fn arb_precedence() -> impl Strategy<Value = Precedence> {
    (
        -50i16..50,
        arb_associativity(),
        prop::collection::vec(arb_symbol_id(), 0..4),
    )
        .prop_map(|(level, associativity, symbols)| Precedence {
            level,
            associativity,
            symbols,
        })
}

fn arb_conflict_resolution() -> impl Strategy<Value = ConflictResolution> {
    prop_oneof![
        arb_precedence_kind().prop_map(ConflictResolution::Precedence),
        arb_associativity().prop_map(ConflictResolution::Associativity),
        Just(ConflictResolution::GLR),
    ]
}

fn arb_conflict_declaration() -> impl Strategy<Value = ConflictDeclaration> {
    (
        prop::collection::vec(arb_symbol_id(), 1..4),
        arb_conflict_resolution(),
    )
        .prop_map(|(symbols, resolution)| ConflictDeclaration {
            symbols,
            resolution,
        })
}

fn _arb_alias_sequence() -> impl Strategy<Value = AliasSequence> {
    prop::collection::vec(
        prop::option::of("[a-zA-Z]{1,6}".prop_map(String::from)),
        0..4,
    )
    .prop_map(|aliases| AliasSequence { aliases })
}

/// Arbitrary grammar with all fields populated.
fn arb_grammar() -> impl Strategy<Value = Grammar> {
    (
        "[a-zA-Z][a-zA-Z0-9]{0,8}",                                  // name
        prop::collection::vec(arb_rule(), 1..4),                     // rules
        prop::collection::vec((arb_symbol_id(), arb_token()), 1..4), // tokens
        prop::collection::vec(arb_precedence(), 0..3),               // precedences
        prop::collection::vec(arb_conflict_declaration(), 0..3),     // conflicts
        prop::collection::vec(arb_external_token(), 0..3),           // externals
        prop::collection::vec(arb_symbol_id(), 0..3),                // extras
        prop::collection::vec(arb_symbol_id(), 0..3),                // supertypes
        prop::collection::vec(arb_symbol_id(), 0..3),                // inline_rules
        0usize..8,                                                   // max_alias_sequence_length
    )
        .prop_map(
            |(
                name,
                rules,
                toks,
                precs,
                conflicts,
                exts,
                extras,
                supertypes,
                inline_rules,
                max_alias,
            )| {
                let mut g = Grammar::new(name);
                for r in rules {
                    g.add_rule(r);
                }
                for (id, tok) in toks {
                    g.tokens.insert(id, tok);
                }
                g.precedences = precs;
                g.conflicts = conflicts;
                g.externals = exts;
                g.extras = extras;
                g.supertypes = supertypes;
                g.inline_rules = inline_rules;
                g.max_alias_sequence_length = max_alias;
                g
            },
        )
}

/// Minimal grammar: name + one token + one rule.
fn arb_minimal_grammar() -> impl Strategy<Value = Grammar> {
    (
        "[a-zA-Z][a-zA-Z0-9]{0,8}",
        arb_symbol_id(),
        arb_token(),
        arb_rule(),
    )
        .prop_map(|(name, tok_id, tok, rule)| {
            let mut g = Grammar::new(name);
            g.tokens.insert(tok_id, tok);
            g.add_rule(rule);
            g
        })
}

// ===========================================================================
// Helpers
// ===========================================================================

fn json_roundtrip(g: &Grammar) -> Grammar {
    let json = serde_json::to_string(g).expect("serialize");
    serde_json::from_str(&json).expect("deserialize")
}

fn total_rule_count(g: &Grammar) -> usize {
    g.rules.values().map(|v| v.len()).sum()
}

// ===========================================================================
// Property tests: JSON roundtrip
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    // 1
    #[test]
    fn prop_any_grammar_serializes_to_valid_json(g in arb_grammar()) {
        let json = serde_json::to_string(&g).unwrap();
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();
        prop_assert!(value.is_object());
    }

    // 2
    #[test]
    fn prop_roundtrip_preserves_name(g in arb_grammar()) {
        let g2 = json_roundtrip(&g);
        prop_assert_eq!(&g.name, &g2.name);
    }

    // 3
    #[test]
    fn prop_roundtrip_preserves_token_count(g in arb_grammar()) {
        let g2 = json_roundtrip(&g);
        prop_assert_eq!(g.tokens.len(), g2.tokens.len());
    }

    // 4
    #[test]
    fn prop_roundtrip_preserves_rule_count(g in arb_grammar()) {
        let g2 = json_roundtrip(&g);
        prop_assert_eq!(total_rule_count(&g), total_rule_count(&g2));
    }

    // 5
    #[test]
    fn prop_clone_equals_original(g in arb_grammar()) {
        let g2 = g.clone();
        prop_assert_eq!(&g, &g2);
    }

    // 6
    #[test]
    fn prop_double_roundtrip_identical_json(g in arb_grammar()) {
        let j1 = serde_json::to_string(&g).unwrap();
        let g2: Grammar = serde_json::from_str(&j1).unwrap();
        let j2 = serde_json::to_string(&g2).unwrap();
        prop_assert_eq!(&j1, &j2);
    }

    // 7
    #[test]
    fn prop_roundtrip_preserves_precedences(g in arb_grammar()) {
        let g2 = json_roundtrip(&g);
        prop_assert_eq!(&g.precedences, &g2.precedences);
    }

    // 8
    #[test]
    fn prop_roundtrip_preserves_externals(g in arb_grammar()) {
        let g2 = json_roundtrip(&g);
        prop_assert_eq!(&g.externals, &g2.externals);
    }

    // 9
    #[test]
    fn prop_roundtrip_preserves_extras(g in arb_grammar()) {
        let g2 = json_roundtrip(&g);
        prop_assert_eq!(&g.extras, &g2.extras);
    }

    // 10
    #[test]
    fn prop_roundtrip_preserves_conflicts(g in arb_grammar()) {
        let g2 = json_roundtrip(&g);
        prop_assert_eq!(&g.conflicts, &g2.conflicts);
    }

    // 11
    #[test]
    fn prop_roundtrip_preserves_supertypes(g in arb_grammar()) {
        let g2 = json_roundtrip(&g);
        prop_assert_eq!(&g.supertypes, &g2.supertypes);
    }

    // 12
    #[test]
    fn prop_roundtrip_preserves_inline_rules(g in arb_grammar()) {
        let g2 = json_roundtrip(&g);
        prop_assert_eq!(&g.inline_rules, &g2.inline_rules);
    }

    // 13
    #[test]
    fn prop_roundtrip_preserves_max_alias_length(g in arb_grammar()) {
        let g2 = json_roundtrip(&g);
        prop_assert_eq!(g.max_alias_sequence_length, g2.max_alias_sequence_length);
    }

    // 14
    #[test]
    fn prop_roundtrip_preserves_rule_names(g in arb_grammar()) {
        let g2 = json_roundtrip(&g);
        prop_assert_eq!(&g.rule_names, &g2.rule_names);
    }

    // 15
    #[test]
    fn prop_roundtrip_preserves_fields(g in arb_grammar()) {
        let g2 = json_roundtrip(&g);
        prop_assert_eq!(&g.fields, &g2.fields);
    }

    // 16
    #[test]
    fn prop_roundtrip_full_equality(g in arb_grammar()) {
        let g2 = json_roundtrip(&g);
        prop_assert_eq!(&g, &g2);
    }

    // 17
    #[test]
    fn prop_roundtrip_preserves_token_names(g in arb_grammar()) {
        let g2 = json_roundtrip(&g);
        let names1: Vec<_> = g.tokens.values().map(|t| &t.name).collect();
        let names2: Vec<_> = g2.tokens.values().map(|t| &t.name).collect();
        prop_assert_eq!(names1, names2);
    }

    // 18
    #[test]
    fn prop_roundtrip_preserves_token_fragile(g in arb_grammar()) {
        let g2 = json_roundtrip(&g);
        let f1: Vec<_> = g.tokens.values().map(|t| t.fragile).collect();
        let f2: Vec<_> = g2.tokens.values().map(|t| t.fragile).collect();
        prop_assert_eq!(f1, f2);
    }

    // 19
    #[test]
    fn prop_roundtrip_preserves_rule_lhs_ids(g in arb_grammar()) {
        let g2 = json_roundtrip(&g);
        let ids1: Vec<_> = g.rules.keys().collect();
        let ids2: Vec<_> = g2.rules.keys().collect();
        prop_assert_eq!(ids1, ids2);
    }

    // 20
    #[test]
    fn prop_roundtrip_preserves_production_ids(g in arb_grammar()) {
        let g2 = json_roundtrip(&g);
        prop_assert_eq!(&g.production_ids, &g2.production_ids);
    }

    // 21
    #[test]
    fn prop_pretty_json_roundtrip(g in arb_grammar()) {
        let pretty = serde_json::to_string_pretty(&g).unwrap();
        let g2: Grammar = serde_json::from_str(&pretty).unwrap();
        prop_assert_eq!(&g, &g2);
    }

    // 22
    #[test]
    fn prop_roundtrip_preserves_alias_sequences(g in arb_grammar()) {
        let g2 = json_roundtrip(&g);
        prop_assert_eq!(&g.alias_sequences, &g2.alias_sequences);
    }

    // 23
    #[test]
    fn prop_minimal_grammar_roundtrips(g in arb_minimal_grammar()) {
        let g2 = json_roundtrip(&g);
        prop_assert_eq!(&g, &g2);
    }

    // 24
    #[test]
    fn prop_serialized_json_contains_name(g in arb_grammar()) {
        let json = serde_json::to_string(&g).unwrap();
        let needle = format!("\"name\":\"{}\"", g.name);
        prop_assert!(json.contains(&needle), "JSON missing name field");
    }

    // 25
    #[test]
    fn prop_clone_of_roundtrip_equals_original(g in arb_grammar()) {
        let g2 = json_roundtrip(&g).clone();
        prop_assert_eq!(&g, &g2);
    }

    // 26
    #[test]
    fn prop_roundtrip_preserves_token_ids(g in arb_grammar()) {
        let g2 = json_roundtrip(&g);
        let ids1: Vec<_> = g.tokens.keys().collect();
        let ids2: Vec<_> = g2.tokens.keys().collect();
        prop_assert_eq!(ids1, ids2);
    }

    // 27
    #[test]
    fn prop_roundtrip_preserves_external_names(g in arb_grammar()) {
        let g2 = json_roundtrip(&g);
        let n1: Vec<_> = g.externals.iter().map(|e| &e.name).collect();
        let n2: Vec<_> = g2.externals.iter().map(|e| &e.name).collect();
        prop_assert_eq!(n1, n2);
    }

    // 28
    #[test]
    fn prop_roundtrip_preserves_rule_rhs_lengths(g in arb_grammar()) {
        let g2 = json_roundtrip(&g);
        let lens1: Vec<Vec<usize>> = g.rules.values()
            .map(|rs| rs.iter().map(|r| r.rhs.len()).collect())
            .collect();
        let lens2: Vec<Vec<usize>> = g2.rules.values()
            .map(|rs| rs.iter().map(|r| r.rhs.len()).collect())
            .collect();
        prop_assert_eq!(lens1, lens2);
    }

    // 29
    #[test]
    fn prop_json_value_roundtrip(g in arb_grammar()) {
        let value = serde_json::to_value(&g).unwrap();
        let g2: Grammar = serde_json::from_value(value).unwrap();
        prop_assert_eq!(&g, &g2);
    }

    // 30
    #[test]
    fn prop_roundtrip_preserves_conflict_count(g in arb_grammar()) {
        let g2 = json_roundtrip(&g);
        prop_assert_eq!(g.conflicts.len(), g2.conflicts.len());
    }
}

// ===========================================================================
// Unit tests: builder-constructed grammars
// ===========================================================================

// 31
#[test]
fn unit_arithmetic_grammar_roundtrip() {
    let g = GrammarBuilder::new("arith")
        .token("NUM", r"\d+")
        .token("+", "+")
        .rule("expr", vec!["NUM", "+", "NUM"])
        .start("expr")
        .build();
    let g2 = json_roundtrip(&g);
    assert_eq!(g, g2);
}

// 32
#[test]
fn unit_empty_rhs_epsilon_roundtrip() {
    let g = GrammarBuilder::new("nullable")
        .token("A", "a")
        .rule("start", vec![])
        .rule("start", vec!["A"])
        .start("start")
        .build();
    let g2 = json_roundtrip(&g);
    assert_eq!(g, g2);
}

// 33
#[test]
fn unit_precedence_roundtrip() {
    let g = GrammarBuilder::new("prec")
        .token("N", r"\d+")
        .token("+", "+")
        .token("*", "*")
        .rule_with_precedence("e", vec!["e", "+", "e"], 1, Associativity::Left)
        .rule_with_precedence("e", vec!["e", "*", "e"], 2, Associativity::Left)
        .rule("e", vec!["N"])
        .start("e")
        .build();
    let g2 = json_roundtrip(&g);
    assert_eq!(g, g2);
}

// 34
#[test]
fn unit_python_like_roundtrip() {
    let g = GrammarBuilder::python_like();
    let g2 = json_roundtrip(&g);
    assert_eq!(g, g2);
}

// 35
#[test]
fn unit_javascript_like_roundtrip() {
    let g = GrammarBuilder::javascript_like();
    let g2 = json_roundtrip(&g);
    assert_eq!(g, g2);
}

// 36
#[test]
fn unit_clone_equals_builder_grammar() {
    let g = GrammarBuilder::new("ctest")
        .token("X", "x")
        .rule("s", vec!["X"])
        .start("s")
        .build();
    assert_eq!(g, g.clone());
}

// 37
#[test]
fn unit_double_roundtrip_builder_grammar() {
    let g = GrammarBuilder::new("dbl")
        .token("A", "a")
        .token("B", "b")
        .rule("s", vec!["A", "B"])
        .start("s")
        .build();
    let j1 = serde_json::to_string(&g).unwrap();
    let g2: Grammar = serde_json::from_str(&j1).unwrap();
    let j2 = serde_json::to_string(&g2).unwrap();
    assert_eq!(j1, j2);
}

// 38
#[test]
fn unit_many_tokens_roundtrip() {
    let mut b = GrammarBuilder::new("many_tok");
    for i in 0..20 {
        b = b.token(&format!("T{i}"), &format!("t{i}"));
    }
    // Need at least one rule with non-empty RHS
    b = b.rule("s", vec!["T0"]).start("s");
    let g = b.build();
    let g2 = json_roundtrip(&g);
    assert_eq!(g, g2);
}

// 39
#[test]
fn unit_many_rules_roundtrip() {
    let mut b = GrammarBuilder::new("many_rules")
        .token("A", "a")
        .token("B", "b")
        .token("C", "c");
    for _ in 0..10 {
        b = b.rule("s", vec!["A"]);
        b = b.rule("s", vec!["A", "B"]);
        b = b.rule("s", vec!["A", "B", "C"]);
    }
    b = b.start("s");
    let g = b.build();
    let g2 = json_roundtrip(&g);
    assert_eq!(g, g2);
}

// 40
#[test]
fn unit_fragile_token_roundtrip() {
    let g = GrammarBuilder::new("fragile")
        .fragile_token("SEMI", ";")
        .token("ID", r"[a-z]+")
        .rule("s", vec!["ID", "SEMI"])
        .start("s")
        .build();
    let g2 = json_roundtrip(&g);
    assert_eq!(g, g2);
    assert!(g2.tokens.values().any(|t| t.fragile));
}

// 41
#[test]
fn unit_external_scanner_roundtrip() {
    let g = GrammarBuilder::new("ext")
        .token("INDENT", "INDENT")
        .token("DEDENT", "DEDENT")
        .external("INDENT")
        .external("DEDENT")
        .token("ID", r"[a-z]+")
        .rule("s", vec!["ID"])
        .start("s")
        .build();
    let g2 = json_roundtrip(&g);
    assert_eq!(g.externals.len(), 2);
    assert_eq!(g, g2);
}

// 42
#[test]
fn unit_extras_roundtrip() {
    let g = GrammarBuilder::new("ws")
        .token("WS", r"\s+")
        .extra("WS")
        .token("A", "a")
        .rule("s", vec!["A"])
        .start("s")
        .build();
    let g2 = json_roundtrip(&g);
    assert_eq!(g.extras.len(), 1);
    assert_eq!(g, g2);
}

// 43
#[test]
fn unit_right_assoc_roundtrip() {
    let g = GrammarBuilder::new("rassoc")
        .token("N", r"\d+")
        .token("^", "^")
        .rule_with_precedence("e", vec!["e", "^", "e"], 3, Associativity::Right)
        .rule("e", vec!["N"])
        .start("e")
        .build();
    let g2 = json_roundtrip(&g);
    assert_eq!(g, g2);
}

// 44
#[test]
fn unit_none_assoc_roundtrip() {
    let g = GrammarBuilder::new("nassoc")
        .token("N", r"\d+")
        .token("=", "=")
        .rule_with_precedence("e", vec!["e", "=", "e"], 1, Associativity::None)
        .rule("e", vec!["N"])
        .start("e")
        .build();
    let g2 = json_roundtrip(&g);
    assert_eq!(g, g2);
}

// 45
#[test]
fn unit_grammar_default_roundtrip() {
    let g = Grammar::default();
    let g2 = json_roundtrip(&g);
    assert_eq!(g, g2);
}

// 46
#[test]
fn unit_grammar_new_roundtrip() {
    let g = Grammar::new("empty".to_string());
    let g2 = json_roundtrip(&g);
    assert_eq!(g, g2);
    assert_eq!(g2.name, "empty");
}

// 47
#[test]
fn unit_serialized_json_is_valid_object() {
    let g = GrammarBuilder::new("check")
        .token("X", "x")
        .rule("s", vec!["X"])
        .start("s")
        .build();
    let json = serde_json::to_string(&g).unwrap();
    let v: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert!(v.is_object());
    assert!(v.get("name").is_some());
    assert!(v.get("tokens").is_some());
    assert!(v.get("rules").is_some());
}

// 48
#[test]
fn unit_multiple_lhs_roundtrip() {
    let g = GrammarBuilder::new("multi")
        .token("A", "a")
        .token("B", "b")
        .rule("x", vec!["A"])
        .rule("y", vec!["B"])
        .rule("s", vec!["x", "y"])
        .start("s")
        .build();
    let g2 = json_roundtrip(&g);
    assert_eq!(g, g2);
}

// 49
#[test]
fn unit_precedence_declaration_roundtrip() {
    let g = GrammarBuilder::new("pdecl")
        .token("N", r"\d+")
        .token("+", "+")
        .token("*", "*")
        .precedence(1, Associativity::Left, vec!["+"])
        .precedence(2, Associativity::Left, vec!["*"])
        .rule("e", vec!["N"])
        .start("e")
        .build();
    let g2 = json_roundtrip(&g);
    assert_eq!(g.precedences.len(), 2);
    assert_eq!(g, g2);
}

// 50
#[test]
fn unit_pretty_vs_compact_roundtrip() {
    let g = GrammarBuilder::new("fmt")
        .token("A", "a")
        .rule("s", vec!["A"])
        .start("s")
        .build();
    let compact: Grammar = serde_json::from_str(&serde_json::to_string(&g).unwrap()).unwrap();
    let pretty: Grammar = serde_json::from_str(&serde_json::to_string_pretty(&g).unwrap()).unwrap();
    assert_eq!(compact, pretty);
}
