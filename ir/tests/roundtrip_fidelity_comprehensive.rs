#![allow(clippy::needless_range_loop)]

//! Comprehensive roundtrip fidelity tests for Grammar construction and manipulation.
//!
//! Verifies that building, reading back, adding/removing rules, symbol IDs,
//! precedence/associativity, field mappings, external tokens, and normalization
//! all preserve data faithfully.

use adze_ir::builder::GrammarBuilder;
use adze_ir::{
    AliasSequence, Associativity, ConflictDeclaration, ConflictResolution, ExternalToken, FieldId,
    Grammar, Precedence, PrecedenceKind, ProductionId, Rule, RuleId, Symbol, SymbolId, Token,
    TokenPattern,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn total_rule_count(g: &Grammar) -> usize {
    g.rules.values().map(|v| v.len()).sum()
}

fn find_symbol_id(g: &Grammar, name: &str) -> SymbolId {
    g.rule_names
        .iter()
        .find(|(_, n)| n.as_str() == name)
        .map(|(id, _)| *id)
        .or_else(|| {
            g.tokens
                .iter()
                .find(|(_, t)| t.name == name)
                .map(|(id, _)| *id)
        })
        .unwrap_or_else(|| panic!("symbol '{name}' not found"))
}

fn build_arithmetic_grammar() -> Grammar {
    GrammarBuilder::new("arithmetic")
        .token("NUMBER", r"\d+")
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
        .rule("expr", vec!["NUMBER"])
        .rule("expr", vec!["(", "expr", ")"])
        .start("expr")
        .build()
}

// ===========================================================================
// 1. Build-then-read-back identity
// ===========================================================================

#[test]
fn build_and_read_back_name() {
    let g = GrammarBuilder::new("my_lang")
        .token("A", "a")
        .rule("start", vec!["A"])
        .start("start")
        .build();
    assert_eq!(g.name, "my_lang");
}

#[test]
fn build_and_read_back_token_count() {
    let g = GrammarBuilder::new("t")
        .token("X", "x")
        .token("Y", "y")
        .token("Z", "z")
        .rule("start", vec!["X"])
        .start("start")
        .build();
    assert_eq!(g.tokens.len(), 3);
}

#[test]
fn build_and_read_back_token_details() {
    let g = GrammarBuilder::new("t")
        .token("NUM", r"\d+")
        .token("+", "+")
        .rule("start", vec!["NUM"])
        .start("start")
        .build();

    let num_token = g.tokens.values().find(|t| t.name == "NUM").unwrap();
    assert_eq!(num_token.pattern, TokenPattern::Regex(r"\d+".to_string()));
    assert!(!num_token.fragile);

    let plus_token = g.tokens.values().find(|t| t.name == "+").unwrap();
    assert_eq!(plus_token.pattern, TokenPattern::String("+".to_string()));
}

#[test]
fn build_and_read_back_rule_count() {
    let g = build_arithmetic_grammar();
    assert_eq!(total_rule_count(&g), 6);
}

#[test]
fn build_and_read_back_rule_rhs_lengths() {
    let g = build_arithmetic_grammar();
    let expr_id = find_symbol_id(&g, "expr");
    let rules = g.get_rules_for_symbol(expr_id).unwrap();

    let mut rhs_lengths: Vec<usize> = rules.iter().map(|r| r.rhs.len()).collect();
    rhs_lengths.sort();
    assert_eq!(rhs_lengths, vec![1, 3, 3, 3, 3, 3]);
}

#[test]
fn build_and_read_back_extras() {
    let g = GrammarBuilder::new("t")
        .token("WS", r"\s+")
        .token("A", "a")
        .extra("WS")
        .rule("start", vec!["A"])
        .start("start")
        .build();
    assert_eq!(g.extras.len(), 1);
    let ws_id = g
        .tokens
        .iter()
        .find(|(_, t)| t.name == "WS")
        .map(|(id, _)| *id)
        .unwrap();
    assert!(g.extras.contains(&ws_id));
}

#[test]
fn build_and_read_back_fragile_token() {
    let g = GrammarBuilder::new("t")
        .fragile_token("SEMI", ";")
        .token("A", "a")
        .rule("start", vec!["A"])
        .start("start")
        .build();
    let semi = g.tokens.values().find(|t| t.name == "SEMI").unwrap();
    assert!(semi.fragile);
}

// ===========================================================================
// 2. Adding/removing rules preserves other rules
// ===========================================================================

#[test]
fn add_rule_preserves_existing_rules() {
    let mut g = build_arithmetic_grammar();
    let original_count = total_rule_count(&g);

    let expr_id = find_symbol_id(&g, "expr");
    let plus_id = g
        .tokens
        .iter()
        .find(|(_, t)| t.name == "+")
        .map(|(id, _)| *id)
        .unwrap();

    let new_rule = Rule {
        lhs: expr_id,
        rhs: vec![
            Symbol::NonTerminal(expr_id),
            Symbol::Terminal(plus_id),
            Symbol::NonTerminal(expr_id),
        ],
        precedence: Some(PrecedenceKind::Static(3)),
        associativity: Some(Associativity::Left),
        fields: vec![],
        production_id: ProductionId(100),
    };
    g.add_rule(new_rule);

    assert_eq!(total_rule_count(&g), original_count + 1);

    let rules = g.get_rules_for_symbol(expr_id).unwrap();
    let has_number_rule = rules.iter().any(|r| r.rhs.len() == 1);
    assert!(has_number_rule, "NUMBER rule should still exist");
}

#[test]
fn remove_one_lhs_preserves_others() {
    let mut g = GrammarBuilder::new("multi")
        .token("A", "a")
        .token("B", "b")
        .rule("start", vec!["A"])
        .rule("other", vec!["B"])
        .start("start")
        .build();

    let other_id = find_symbol_id(&g, "other");
    g.rules.shift_remove(&other_id);

    assert_eq!(g.rules.len(), 1);
    let start_id = find_symbol_id(&g, "start");
    assert!(g.get_rules_for_symbol(start_id).is_some());
    assert!(g.get_rules_for_symbol(other_id).is_none());
}

#[test]
fn add_rule_to_new_lhs_does_not_disturb_existing() {
    let mut g = build_arithmetic_grammar();
    let expr_id = find_symbol_id(&g, "expr");
    let original_expr_rules = g.get_rules_for_symbol(expr_id).unwrap().clone();

    let new_id = SymbolId(999);
    let number_id = g
        .tokens
        .iter()
        .find(|(_, t)| t.name == "NUMBER")
        .map(|(id, _)| *id)
        .unwrap();
    g.add_rule(Rule {
        lhs: new_id,
        rhs: vec![Symbol::Terminal(number_id)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(200),
    });

    let current_expr_rules = g.get_rules_for_symbol(expr_id).unwrap();
    assert_eq!(current_expr_rules.len(), original_expr_rules.len());
    for i in 0..original_expr_rules.len() {
        assert_eq!(current_expr_rules[i], original_expr_rules[i]);
    }
}

// ===========================================================================
// 3. Symbol IDs are correctly assigned and stable
// ===========================================================================

#[test]
fn symbol_ids_start_at_one() {
    let g = GrammarBuilder::new("t")
        .token("A", "a")
        .rule("start", vec!["A"])
        .start("start")
        .build();

    let min_id = g
        .tokens
        .keys()
        .chain(g.rules.keys())
        .map(|id| id.0)
        .min()
        .unwrap();
    assert!(min_id >= 1);
}

#[test]
fn symbol_ids_are_unique_across_tokens_and_rules() {
    let g = build_arithmetic_grammar();
    let mut all_ids: Vec<u16> = g.tokens.keys().map(|id| id.0).collect();
    all_ids.extend(g.rules.keys().map(|id| id.0));
    let unique_count = {
        let mut sorted = all_ids.clone();
        sorted.sort();
        sorted.dedup();
        sorted.len()
    };
    assert_eq!(all_ids.len(), unique_count, "symbol IDs must be unique");
}

#[test]
fn same_token_referenced_twice_yields_same_id() {
    let g = GrammarBuilder::new("t")
        .token("+", "+")
        .token("A", "a")
        .rule("start", vec!["A", "+", "A"])
        .start("start")
        .build();

    let s_id = find_symbol_id(&g, "start");
    let rules = g.get_rules_for_symbol(s_id).unwrap();
    let rhs = &rules[0].rhs;
    let id0 = match &rhs[0] {
        Symbol::Terminal(id) | Symbol::NonTerminal(id) => *id,
        other => panic!("unexpected symbol: {other:?}"),
    };
    let id2 = match &rhs[2] {
        Symbol::Terminal(id) | Symbol::NonTerminal(id) => *id,
        other => panic!("unexpected symbol: {other:?}"),
    };
    assert_eq!(id0, id2, "same name must produce same SymbolId");
}

#[test]
fn production_ids_are_unique_per_grammar() {
    let g = build_arithmetic_grammar();
    let prod_ids: Vec<ProductionId> = g.all_rules().map(|r| r.production_id).collect();
    let mut sorted: Vec<u16> = prod_ids.iter().map(|p| p.0).collect();
    sorted.sort();
    sorted.dedup();
    assert_eq!(
        sorted.len(),
        prod_ids.len(),
        "production IDs must be unique"
    );
}

#[test]
fn rule_names_map_covers_nonterminals() {
    let g = GrammarBuilder::new("t")
        .token("A", "a")
        .token("B", "b")
        .rule("begin", vec!["A"])
        .rule("other", vec!["B"])
        .start("begin")
        .build();

    assert!(g.rule_names.values().any(|n| n == "begin"));
    assert!(g.rule_names.values().any(|n| n == "other"));
}

// ===========================================================================
// 4. Precedence and associativity survive roundtrips
// ===========================================================================

#[test]
fn static_precedence_preserved() {
    let g = build_arithmetic_grammar();
    let expr_id = find_symbol_id(&g, "expr");
    let rules = g.get_rules_for_symbol(expr_id).unwrap();

    let add_rule = rules
        .iter()
        .find(|r| {
            r.rhs.len() == 3
                && r.rhs.iter().any(|s| {
                    matches!(s, Symbol::Terminal(id) if g.tokens.get(id).map(|t| t.name.as_str()) == Some("+"))
                })
        })
        .expect("addition rule should exist");

    assert_eq!(add_rule.precedence, Some(PrecedenceKind::Static(1)));
    assert_eq!(add_rule.associativity, Some(Associativity::Left));
}

#[test]
fn dynamic_precedence_preserved() {
    let mut g = Grammar::new("dyn_prec".to_string());
    let lhs = SymbolId(10);
    let terminal = SymbolId(11);
    g.tokens.insert(
        terminal,
        Token {
            name: "T".to_string(),
            pattern: TokenPattern::String("t".to_string()),
            fragile: false,
        },
    );
    g.add_rule(Rule {
        lhs,
        rhs: vec![Symbol::Terminal(terminal)],
        precedence: Some(PrecedenceKind::Dynamic(42)),
        associativity: Some(Associativity::Right),
        fields: vec![],
        production_id: ProductionId(0),
    });

    let rule = &g.rules[&lhs][0];
    assert_eq!(rule.precedence, Some(PrecedenceKind::Dynamic(42)));
    assert_eq!(rule.associativity, Some(Associativity::Right));
}

#[test]
fn precedence_declarations_preserved() {
    let g = GrammarBuilder::new("prec")
        .token("A", "a")
        .token("B", "b")
        .precedence(1, Associativity::Left, vec!["A"])
        .precedence(2, Associativity::Right, vec!["B"])
        .rule("start", vec!["A"])
        .start("start")
        .build();

    assert_eq!(g.precedences.len(), 2);
    assert_eq!(g.precedences[0].level, 1);
    assert_eq!(g.precedences[0].associativity, Associativity::Left);
    assert_eq!(g.precedences[1].level, 2);
    assert_eq!(g.precedences[1].associativity, Associativity::Right);
}

#[test]
fn multiple_associativity_kinds_coexist() {
    let g = GrammarBuilder::new("assoc")
        .token("A", "a")
        .token("B", "b")
        .token("C", "c")
        .rule_with_precedence("expr", vec!["A"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["B"], 2, Associativity::Right)
        .rule_with_precedence("expr", vec!["C"], 3, Associativity::None)
        .start("expr")
        .build();

    let e_id = find_symbol_id(&g, "expr");
    let rules = g.get_rules_for_symbol(e_id).unwrap();

    assert_eq!(rules[0].associativity, Some(Associativity::Left));
    assert_eq!(rules[1].associativity, Some(Associativity::Right));
    assert_eq!(rules[2].associativity, Some(Associativity::None));
}

#[test]
fn no_precedence_is_none() {
    let g = GrammarBuilder::new("t")
        .token("A", "a")
        .rule("start", vec!["A"])
        .start("start")
        .build();

    let s_id = find_symbol_id(&g, "start");
    let rule = &g.get_rules_for_symbol(s_id).unwrap()[0];
    assert_eq!(rule.precedence, None);
    assert_eq!(rule.associativity, None);
}

// ===========================================================================
// 5. Field mappings are preserved
// ===========================================================================

#[test]
fn field_mappings_on_rules_preserved() {
    let mut g = Grammar::new("fields".to_string());
    let lhs = SymbolId(1);
    let t1 = SymbolId(2);
    let t2 = SymbolId(3);
    g.tokens.insert(
        t1,
        Token {
            name: "A".into(),
            pattern: TokenPattern::String("a".into()),
            fragile: false,
        },
    );
    g.tokens.insert(
        t2,
        Token {
            name: "B".into(),
            pattern: TokenPattern::String("b".into()),
            fragile: false,
        },
    );
    g.fields.insert(FieldId(0), "left".to_string());
    g.fields.insert(FieldId(1), "right".to_string());

    g.add_rule(Rule {
        lhs,
        rhs: vec![Symbol::Terminal(t1), Symbol::Terminal(t2)],
        precedence: None,
        associativity: None,
        fields: vec![(FieldId(0), 0), (FieldId(1), 1)],
        production_id: ProductionId(0),
    });

    let rule = &g.rules[&lhs][0];
    assert_eq!(rule.fields.len(), 2);
    assert_eq!(rule.fields[0], (FieldId(0), 0));
    assert_eq!(rule.fields[1], (FieldId(1), 1));
    assert_eq!(g.fields[&FieldId(0)], "left");
    assert_eq!(g.fields[&FieldId(1)], "right");
}

#[test]
fn empty_fields_preserved() {
    let g = GrammarBuilder::new("t")
        .token("A", "a")
        .rule("start", vec!["A"])
        .start("start")
        .build();
    assert!(g.fields.is_empty());
    let s_id = find_symbol_id(&g, "start");
    let rule = &g.get_rules_for_symbol(s_id).unwrap()[0];
    assert!(rule.fields.is_empty());
}

#[test]
fn grammar_level_fields_map_preserved() {
    let mut g = Grammar::new("f".to_string());
    g.fields.insert(FieldId(0), "alpha".to_string());
    g.fields.insert(FieldId(1), "beta".to_string());
    g.fields.insert(FieldId(2), "gamma".to_string());

    assert_eq!(g.fields.len(), 3);
    let names: Vec<&str> = g.fields.values().map(|s| s.as_str()).collect();
    assert_eq!(names, vec!["alpha", "beta", "gamma"]);
}

// ===========================================================================
// 6. ExternalToken data is preserved
// ===========================================================================

#[test]
fn external_tokens_preserved() {
    let g = GrammarBuilder::new("ext")
        .token("INDENT", "INDENT")
        .token("DEDENT", "DEDENT")
        .token("NEWLINE", r"\n")
        .external("INDENT")
        .external("DEDENT")
        .rule("start", vec!["NEWLINE"])
        .start("start")
        .build();

    assert_eq!(g.externals.len(), 2);
    assert_eq!(g.externals[0].name, "INDENT");
    assert_eq!(g.externals[1].name, "DEDENT");
}

#[test]
fn external_token_symbol_ids_match_token_ids() {
    let g = GrammarBuilder::new("ext")
        .token("INDENT", "INDENT")
        .external("INDENT")
        .token("A", "a")
        .rule("start", vec!["A"])
        .start("start")
        .build();

    let indent_token_id = g
        .tokens
        .iter()
        .find(|(_, t)| t.name == "INDENT")
        .map(|(id, _)| *id)
        .unwrap();
    assert_eq!(g.externals[0].symbol_id, indent_token_id);
}

#[test]
fn external_tokens_independent_of_rules() {
    let mut g = GrammarBuilder::new("ext")
        .token("EXT", "ext")
        .external("EXT")
        .token("A", "a")
        .rule("start", vec!["A"])
        .rule("other", vec!["A"])
        .start("start")
        .build();

    let other_id = find_symbol_id(&g, "other");
    g.rules.shift_remove(&other_id);

    assert_eq!(g.externals.len(), 1);
    assert_eq!(g.externals[0].name, "EXT");
}

#[test]
fn external_token_direct_construction() {
    let mut g = Grammar::new("direct_ext".to_string());
    g.externals.push(ExternalToken {
        name: "TEMPLATE_LITERAL".to_string(),
        symbol_id: SymbolId(50),
    });
    g.externals.push(ExternalToken {
        name: "REGEX_LITERAL".to_string(),
        symbol_id: SymbolId(51),
    });

    assert_eq!(g.externals.len(), 2);
    assert_eq!(g.externals[0].name, "TEMPLATE_LITERAL");
    assert_eq!(g.externals[0].symbol_id, SymbolId(50));
    assert_eq!(g.externals[1].name, "REGEX_LITERAL");
    assert_eq!(g.externals[1].symbol_id, SymbolId(51));
}

// ===========================================================================
// 7. Grammar.normalize() preserves semantics
// ===========================================================================

#[test]
fn normalize_removes_optional_symbols() {
    let mut g = Grammar::new("opt".to_string());
    let s = SymbolId(1);
    let a = SymbolId(2);
    g.tokens.insert(
        a,
        Token {
            name: "A".into(),
            pattern: TokenPattern::String("a".into()),
            fragile: false,
        },
    );
    g.rule_names.insert(s, "S".to_string());

    g.add_rule(Rule {
        lhs: s,
        rhs: vec![Symbol::Optional(Box::new(Symbol::Terminal(a)))],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });

    g.normalize();

    let has_optional = g
        .all_rules()
        .any(|r| r.rhs.iter().any(|sym| matches!(sym, Symbol::Optional(_))));
    assert!(!has_optional, "Optional symbols should be eliminated");
}

#[test]
fn normalize_removes_repeat_symbols() {
    let mut g = Grammar::new("rep".to_string());
    let s = SymbolId(1);
    let a = SymbolId(2);
    g.tokens.insert(
        a,
        Token {
            name: "A".into(),
            pattern: TokenPattern::String("a".into()),
            fragile: false,
        },
    );

    g.add_rule(Rule {
        lhs: s,
        rhs: vec![Symbol::Repeat(Box::new(Symbol::Terminal(a)))],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });

    g.normalize();

    let has_repeat = g
        .all_rules()
        .any(|r| r.rhs.iter().any(|sym| matches!(sym, Symbol::Repeat(_))));
    assert!(!has_repeat, "Repeat symbols should be eliminated");
}

#[test]
fn normalize_removes_repeat_one_symbols() {
    let mut g = Grammar::new("rep1".to_string());
    let s = SymbolId(1);
    let a = SymbolId(2);
    g.tokens.insert(
        a,
        Token {
            name: "A".into(),
            pattern: TokenPattern::String("a".into()),
            fragile: false,
        },
    );

    g.add_rule(Rule {
        lhs: s,
        rhs: vec![Symbol::RepeatOne(Box::new(Symbol::Terminal(a)))],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });

    g.normalize();

    let has_repeat1 = g
        .all_rules()
        .any(|r| r.rhs.iter().any(|sym| matches!(sym, Symbol::RepeatOne(_))));
    assert!(!has_repeat1, "RepeatOne symbols should be eliminated");
}

#[test]
fn normalize_removes_choice_symbols() {
    let mut g = Grammar::new("choice".to_string());
    let s = SymbolId(1);
    let a = SymbolId(2);
    let b = SymbolId(3);
    g.tokens.insert(
        a,
        Token {
            name: "A".into(),
            pattern: TokenPattern::String("a".into()),
            fragile: false,
        },
    );
    g.tokens.insert(
        b,
        Token {
            name: "B".into(),
            pattern: TokenPattern::String("b".into()),
            fragile: false,
        },
    );

    g.add_rule(Rule {
        lhs: s,
        rhs: vec![Symbol::Choice(vec![
            Symbol::Terminal(a),
            Symbol::Terminal(b),
        ])],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });

    g.normalize();

    let has_choice = g
        .all_rules()
        .any(|r| r.rhs.iter().any(|sym| matches!(sym, Symbol::Choice(_))));
    assert!(!has_choice, "Choice symbols should be eliminated");
}

#[test]
fn normalize_flattens_sequence_symbols() {
    let mut g = Grammar::new("seq".to_string());
    let s = SymbolId(1);
    let a = SymbolId(2);
    let b = SymbolId(3);
    g.tokens.insert(
        a,
        Token {
            name: "A".into(),
            pattern: TokenPattern::String("a".into()),
            fragile: false,
        },
    );
    g.tokens.insert(
        b,
        Token {
            name: "B".into(),
            pattern: TokenPattern::String("b".into()),
            fragile: false,
        },
    );

    g.add_rule(Rule {
        lhs: s,
        rhs: vec![Symbol::Sequence(vec![
            Symbol::Terminal(a),
            Symbol::Terminal(b),
        ])],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });

    g.normalize();

    let has_seq = g
        .all_rules()
        .any(|r| r.rhs.iter().any(|sym| matches!(sym, Symbol::Sequence(_))));
    assert!(!has_seq, "Sequence symbols should be flattened");

    let s_rules = g.get_rules_for_symbol(s).unwrap();
    let has_both = s_rules.iter().any(|r| {
        r.rhs.len() == 2
            && matches!(r.rhs[0], Symbol::Terminal(id) if id == a)
            && matches!(r.rhs[1], Symbol::Terminal(id) if id == b)
    });
    assert!(has_both, "Sequence should be flattened into parent rule");
}

#[test]
fn normalize_preserves_simple_rules() {
    let mut g = build_arithmetic_grammar();
    let before_count = total_rule_count(&g);

    g.normalize();

    assert_eq!(total_rule_count(&g), before_count);
}

#[test]
fn normalize_preserves_tokens() {
    let mut g = Grammar::new("tok".to_string());
    let s = SymbolId(1);
    let a = SymbolId(2);
    g.tokens.insert(
        a,
        Token {
            name: "A".into(),
            pattern: TokenPattern::String("a".into()),
            fragile: false,
        },
    );

    g.add_rule(Rule {
        lhs: s,
        rhs: vec![Symbol::Optional(Box::new(Symbol::Terminal(a)))],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });

    g.normalize();

    assert!(g.tokens.contains_key(&a));
    assert_eq!(g.tokens[&a].name, "A");
}

#[test]
fn normalize_preserves_externals() {
    let mut g = Grammar::new("ext_norm".to_string());
    let s = SymbolId(1);
    let a = SymbolId(2);
    g.tokens.insert(
        a,
        Token {
            name: "A".into(),
            pattern: TokenPattern::String("a".into()),
            fragile: false,
        },
    );
    g.externals.push(ExternalToken {
        name: "EXT".to_string(),
        symbol_id: SymbolId(100),
    });

    g.add_rule(Rule {
        lhs: s,
        rhs: vec![Symbol::Optional(Box::new(Symbol::Terminal(a)))],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });

    g.normalize();

    assert_eq!(g.externals.len(), 1);
    assert_eq!(g.externals[0].name, "EXT");
}

#[test]
fn normalize_preserves_precedences() {
    let mut g = Grammar::new("prec_norm".to_string());
    let s = SymbolId(1);
    let a = SymbolId(2);
    g.tokens.insert(
        a,
        Token {
            name: "A".into(),
            pattern: TokenPattern::String("a".into()),
            fragile: false,
        },
    );
    g.precedences.push(Precedence {
        level: 5,
        associativity: Associativity::Left,
        symbols: vec![a],
    });

    g.add_rule(Rule {
        lhs: s,
        rhs: vec![Symbol::Optional(Box::new(Symbol::Terminal(a)))],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });

    g.normalize();

    assert_eq!(g.precedences.len(), 1);
    assert_eq!(g.precedences[0].level, 5);
    assert_eq!(g.precedences[0].associativity, Associativity::Left);
}

// ===========================================================================
// 8. Serde roundtrip
// ===========================================================================

#[test]
fn serde_json_roundtrip_preserves_grammar() {
    let g = build_arithmetic_grammar();
    let json = serde_json::to_string(&g).expect("serialize");
    let g2: Grammar = serde_json::from_str(&json).expect("deserialize");

    assert_eq!(g.name, g2.name);
    assert_eq!(g.tokens.len(), g2.tokens.len());
    assert_eq!(total_rule_count(&g), total_rule_count(&g2));
    assert_eq!(g.precedences.len(), g2.precedences.len());
    assert_eq!(g.externals.len(), g2.externals.len());
    assert_eq!(g.extras.len(), g2.extras.len());

    for (lhs, rules) in &g.rules {
        let rules2 = g2
            .get_rules_for_symbol(*lhs)
            .expect("LHS should exist after roundtrip");
        assert_eq!(rules.len(), rules2.len());
        for i in 0..rules.len() {
            assert_eq!(rules[i], rules2[i]);
        }
    }
}

// ===========================================================================
// 9. Conflict declarations
// ===========================================================================

#[test]
fn conflict_declarations_preserved() {
    let mut g = Grammar::new("conflicts".to_string());
    g.conflicts.push(ConflictDeclaration {
        symbols: vec![SymbolId(1), SymbolId(2)],
        resolution: ConflictResolution::GLR,
    });
    g.conflicts.push(ConflictDeclaration {
        symbols: vec![SymbolId(3)],
        resolution: ConflictResolution::Precedence(PrecedenceKind::Static(10)),
    });

    assert_eq!(g.conflicts.len(), 2);
    assert_eq!(g.conflicts[0].resolution, ConflictResolution::GLR);
    match &g.conflicts[1].resolution {
        ConflictResolution::Precedence(PrecedenceKind::Static(10)) => {}
        other => panic!("expected static precedence, got {other:?}"),
    }
}

// ===========================================================================
// 10. Miscellaneous fidelity
// ===========================================================================

#[test]
fn alias_sequences_preserved() {
    let mut g = Grammar::new("alias".to_string());
    g.alias_sequences.insert(
        ProductionId(0),
        AliasSequence {
            aliases: vec![Some("method_name".into()), None, Some("params".into())],
        },
    );
    g.max_alias_sequence_length = 3;

    assert_eq!(g.alias_sequences.len(), 1);
    let seq = &g.alias_sequences[&ProductionId(0)];
    assert_eq!(seq.aliases.len(), 3);
    assert_eq!(seq.aliases[0], Some("method_name".into()));
    assert_eq!(seq.aliases[1], None);
    assert_eq!(seq.aliases[2], Some("params".into()));
    assert_eq!(g.max_alias_sequence_length, 3);
}

#[test]
fn production_id_mapping_preserved() {
    let mut g = Grammar::new("prod".to_string());
    g.production_ids.insert(RuleId(0), ProductionId(100));
    g.production_ids.insert(RuleId(1), ProductionId(200));

    assert_eq!(g.production_ids.len(), 2);
    assert_eq!(g.production_ids[&RuleId(0)], ProductionId(100));
    assert_eq!(g.production_ids[&RuleId(1)], ProductionId(200));
}

#[test]
fn inline_rules_and_supertypes_preserved() {
    let mut g = Grammar::new("meta".to_string());
    g.inline_rules.push(SymbolId(10));
    g.inline_rules.push(SymbolId(20));
    g.supertypes.push(SymbolId(30));

    assert_eq!(g.inline_rules, vec![SymbolId(10), SymbolId(20)]);
    assert_eq!(g.supertypes, vec![SymbolId(30)]);
}

#[test]
fn python_like_grammar_roundtrip() {
    let g = GrammarBuilder::python_like();

    assert_eq!(g.name, "python_like");
    assert_eq!(g.externals.len(), 2);
    assert_eq!(g.extras.len(), 1);

    let module_id = find_symbol_id(&g, "module");
    let module_rules = g.get_rules_for_symbol(module_id).unwrap();
    let has_epsilon = module_rules.iter().any(|r| r.rhs == vec![Symbol::Epsilon]);
    assert!(has_epsilon, "module should have epsilon production");

    let json = serde_json::to_string(&g).unwrap();
    let g2: Grammar = serde_json::from_str(&json).unwrap();
    assert_eq!(g2.externals.len(), 2);
    assert_eq!(total_rule_count(&g), total_rule_count(&g2));
}

#[test]
fn javascript_like_grammar_roundtrip() {
    let g = GrammarBuilder::javascript_like();

    assert_eq!(g.name, "javascript_like");
    assert!(g.extras.len() >= 1);

    let expr_id = find_symbol_id(&g, "expression");
    let expr_rules = g.get_rules_for_symbol(expr_id).unwrap();
    let prec_rules: Vec<_> = expr_rules
        .iter()
        .filter(|r| r.precedence.is_some())
        .collect();
    assert_eq!(
        prec_rules.len(),
        4,
        "should have 4 precedented expression rules"
    );

    let json = serde_json::to_string(&g).unwrap();
    let g2: Grammar = serde_json::from_str(&json).unwrap();
    let expr_rules2 = g2.get_rules_for_symbol(expr_id).unwrap();
    assert_eq!(expr_rules.len(), expr_rules2.len());
}
