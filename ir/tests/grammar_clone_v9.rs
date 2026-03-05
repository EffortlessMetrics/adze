//! Comprehensive tests for Grammar cloning, equality, and Debug formatting (v9).

use adze_ir::builder::GrammarBuilder;
use adze_ir::{
    AliasSequence, Associativity, ConflictDeclaration, ConflictResolution, ExternalToken, FieldId,
    Grammar, Precedence, ProductionId, Rule, RuleId, Symbol, SymbolId, Token, TokenPattern,
};
use indexmap::IndexMap;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn gc_v9_empty() -> Grammar {
    GrammarBuilder::new("gc_v9_empty").build()
}

fn gc_v9_single_token() -> Grammar {
    GrammarBuilder::new("gc_v9_single_token")
        .token("x", "x")
        .rule("start", vec!["x"])
        .start("start")
        .build()
}

fn gc_v9_two_tokens() -> Grammar {
    GrammarBuilder::new("gc_v9_two_tokens")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a", "b"])
        .start("start")
        .build()
}

fn gc_v9_multi_rule() -> Grammar {
    GrammarBuilder::new("gc_v9_multi_rule")
        .token("x", "x")
        .token("y", "y")
        .token("z", "z")
        .rule("start", vec!["expr"])
        .rule("expr", vec!["x"])
        .rule("expr", vec!["y"])
        .rule("expr", vec!["z"])
        .start("start")
        .build()
}

fn gc_v9_precedence() -> Grammar {
    GrammarBuilder::new("gc_v9_precedence")
        .token("num", r"\d+")
        .token("plus", r"\+")
        .token("star", r"\*")
        .rule("expr", vec!["num"])
        .rule_with_precedence("expr", vec!["expr", "plus", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "star", "expr"], 2, Associativity::Left)
        .start("expr")
        .build()
}

fn gc_v9_complex() -> Grammar {
    let mut g = Grammar::new("gc_v9_complex".to_string());

    // Tokens
    for i in 0u16..5 {
        g.tokens.insert(
            SymbolId(100 + i),
            Token {
                name: format!("T{i}"),
                pattern: TokenPattern::String(format!("t{i}")),
                fragile: i % 2 == 0,
            },
        );
    }

    // Rules
    g.add_rule(Rule {
        lhs: SymbolId(0),
        rhs: vec![
            Symbol::NonTerminal(SymbolId(1)),
            Symbol::Terminal(SymbolId(100)),
        ],
        precedence: Some(adze_ir::PrecedenceKind::Static(1)),
        associativity: Some(Associativity::Left),
        fields: vec![(FieldId(0), 0)],
        production_id: ProductionId(0),
    });
    g.add_rule(Rule {
        lhs: SymbolId(1),
        rhs: vec![Symbol::Terminal(SymbolId(101))],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(1),
    });

    // Precedences
    g.precedences.push(Precedence {
        level: 1,
        associativity: Associativity::Left,
        symbols: vec![SymbolId(100)],
    });
    g.precedences.push(Precedence {
        level: 2,
        associativity: Associativity::Right,
        symbols: vec![SymbolId(101)],
    });

    // Conflicts
    g.conflicts.push(ConflictDeclaration {
        symbols: vec![SymbolId(0), SymbolId(1)],
        resolution: ConflictResolution::GLR,
    });

    // Externals
    g.externals.push(ExternalToken {
        name: "indent".to_string(),
        symbol_id: SymbolId(200),
    });

    // Extras
    g.extras.push(SymbolId(300));

    // Fields
    g.fields.insert(FieldId(0), "left".to_string());
    g.fields.insert(FieldId(1), "right".to_string());

    // Supertypes
    g.supertypes.push(SymbolId(0));

    // Inline rules
    g.inline_rules.push(SymbolId(1));

    // Alias sequences
    let mut aliases = IndexMap::new();
    aliases.insert(
        ProductionId(0),
        AliasSequence {
            aliases: vec![Some("add".to_string()), None],
        },
    );
    g.alias_sequences = aliases;

    // Production IDs
    g.production_ids.insert(RuleId(0), ProductionId(0));
    g.production_ids.insert(RuleId(1), ProductionId(1));

    g.max_alias_sequence_length = 2;

    // Rule names
    g.rule_names.insert(SymbolId(0), "root".to_string());
    g.rule_names.insert(SymbolId(1), "child".to_string());

    g
}

/// Build a grammar with N simple rules, each referencing a dedicated token.
fn gc_v9_n_rules(n: u16) -> Grammar {
    let name = format!("gc_v9_{n}_rules");
    let mut builder = GrammarBuilder::new(&name);
    let mut rhs_names: Vec<String> = Vec::new();

    for i in 0..n {
        let tok_name = format!("t{i}");
        let rule_name = format!("r{i}");
        builder = builder.token(&tok_name, &tok_name);
        builder = builder.rule(&rule_name, vec![&tok_name]);
        rhs_names.push(rule_name);
    }

    let refs: Vec<&str> = rhs_names.iter().map(|s| s.as_str()).collect();
    if !refs.is_empty() {
        builder = builder.rule("start", refs).start("start");
    }

    builder.build()
}

// ===========================================================================
// 1–5. Clone produces independent copy with same data
// ===========================================================================

#[test]
fn test_clone_produces_independent_copy() {
    let g = gc_v9_two_tokens();
    let cloned = g.clone();
    assert_eq!(g, cloned);
}

#[test]
fn test_clone_has_same_name() {
    let g = gc_v9_single_token();
    let cloned = g.clone();
    assert_eq!(g.name, cloned.name);
}

#[test]
fn test_clone_has_same_rule_count() {
    let g = gc_v9_multi_rule();
    let cloned = g.clone();
    assert_eq!(g.all_rules().count(), cloned.all_rules().count());
}

#[test]
fn test_clone_has_same_token_count() {
    let g = gc_v9_two_tokens();
    let cloned = g.clone();
    assert_eq!(g.tokens.len(), cloned.tokens.len());
}

#[test]
fn test_clone_has_same_start_symbol() {
    let g = gc_v9_single_token();
    let cloned = g.clone();
    assert_eq!(g.start_symbol(), cloned.start_symbol());
}

// ===========================================================================
// 6–7. Modifying clone doesn't affect original
// ===========================================================================

#[test]
fn test_clone_normalize_does_not_affect_original() {
    let g = gc_v9_multi_rule();
    let original_count = g.all_rules().count();
    let mut cloned = g.clone();
    let _new_rules = cloned.normalize();
    // Original must still have its original rule count.
    assert_eq!(g.all_rules().count(), original_count);
}

#[test]
fn test_clone_optimize_does_not_affect_original() {
    let g = gc_v9_multi_rule();
    let original_name = g.name.clone();
    let mut cloned = g.clone();
    cloned.optimize();
    cloned.name = "gc_v9_optimized".to_string();
    assert_eq!(g.name, original_name);
}

// ===========================================================================
// 8–10. Debug formatting
// ===========================================================================

#[test]
fn test_debug_format_is_non_empty() {
    let g = gc_v9_single_token();
    let dbg = format!("{g:?}");
    assert!(!dbg.is_empty());
}

#[test]
fn test_debug_format_contains_grammar_name() {
    let g = gc_v9_two_tokens();
    let dbg = format!("{g:?}");
    assert!(
        dbg.contains("gc_v9_two_tokens"),
        "Debug output should contain grammar name, got: {dbg}"
    );
}

#[test]
fn test_debug_format_contains_rules() {
    let g = gc_v9_multi_rule();
    let dbg = format!("{g:?}");
    assert!(
        dbg.contains("rules"),
        "Debug output should contain 'rules', got: {dbg}"
    );
}

// ===========================================================================
// 11. Multiple clones are independent
// ===========================================================================

#[test]
fn test_multiple_clones_independent() {
    let g = gc_v9_two_tokens();
    let mut c1 = g.clone();
    let mut c2 = g.clone();
    let c3 = g.clone();

    c1.name = "gc_v9_clone_a".to_string();
    c2.name = "gc_v9_clone_b".to_string();

    assert_eq!(g.name, "gc_v9_two_tokens");
    assert_eq!(c3.name, "gc_v9_two_tokens");
    assert_ne!(c1.name, c2.name);
    assert_ne!(c1.name, g.name);
}

#[test]
fn test_three_clones_all_equal_to_original() {
    let g = gc_v9_precedence();
    let c1 = g.clone();
    let c2 = g.clone();
    let c3 = g.clone();
    assert_eq!(g, c1);
    assert_eq!(g, c2);
    assert_eq!(g, c3);
}

// ===========================================================================
// 12. Clone of empty grammar
// ===========================================================================

#[test]
fn test_clone_empty_grammar_eq() {
    let g = gc_v9_empty();
    let cloned = g.clone();
    assert_eq!(g, cloned);
}

#[test]
fn test_clone_empty_grammar_name() {
    let g = gc_v9_empty();
    let cloned = g.clone();
    assert_eq!(cloned.name, "gc_v9_empty");
}

#[test]
fn test_clone_empty_grammar_no_rules() {
    let g = gc_v9_empty();
    let cloned = g.clone();
    assert_eq!(cloned.all_rules().count(), 0);
    assert_eq!(g.all_rules().count(), 0);
}

#[test]
fn test_clone_empty_grammar_no_tokens() {
    let g = gc_v9_empty();
    let cloned = g.clone();
    assert!(g.tokens.is_empty());
    assert!(cloned.tokens.is_empty());
}

#[test]
fn test_clone_default_grammar_eq() {
    let g = Grammar::default();
    let cloned = g.clone();
    assert_eq!(g, cloned);
}

// ===========================================================================
// 13. Clone of complex grammar
// ===========================================================================

#[test]
fn test_clone_complex_grammar_eq() {
    let g = gc_v9_complex();
    let cloned = g.clone();
    assert_eq!(g, cloned);
}

#[test]
fn test_clone_complex_grammar_token_count() {
    let g = gc_v9_complex();
    let cloned = g.clone();
    assert_eq!(g.tokens.len(), cloned.tokens.len());
    assert_eq!(g.tokens.len(), 5);
}

#[test]
fn test_clone_complex_grammar_rule_count() {
    let g = gc_v9_complex();
    let cloned = g.clone();
    assert_eq!(g.all_rules().count(), cloned.all_rules().count());
    assert_eq!(g.all_rules().count(), 2);
}

#[test]
fn test_clone_complex_grammar_rule_names() {
    let g = gc_v9_complex();
    let cloned = g.clone();
    assert_eq!(g.rule_names.len(), cloned.rule_names.len());
    for (id, name) in &g.rule_names {
        assert_eq!(cloned.rule_names.get(id), Some(name));
    }
}

#[test]
fn test_clone_complex_grammar_fields() {
    let g = gc_v9_complex();
    let cloned = g.clone();
    assert_eq!(g.fields.len(), cloned.fields.len());
    assert_eq!(cloned.fields[&FieldId(0)], "left");
    assert_eq!(cloned.fields[&FieldId(1)], "right");
}

#[test]
fn test_clone_complex_grammar_alias_sequences() {
    let g = gc_v9_complex();
    let cloned = g.clone();
    assert_eq!(g.alias_sequences.len(), cloned.alias_sequences.len());
    assert_eq!(
        g.max_alias_sequence_length,
        cloned.max_alias_sequence_length
    );
}

#[test]
fn test_clone_complex_grammar_production_ids() {
    let g = gc_v9_complex();
    let cloned = g.clone();
    assert_eq!(g.production_ids.len(), cloned.production_ids.len());
    for (rid, pid) in &g.production_ids {
        assert_eq!(cloned.production_ids.get(rid), Some(pid));
    }
}

#[test]
fn test_clone_complex_grammar_supertypes() {
    let g = gc_v9_complex();
    let cloned = g.clone();
    assert_eq!(g.supertypes, cloned.supertypes);
}

#[test]
fn test_clone_complex_grammar_inline_rules() {
    let g = gc_v9_complex();
    let cloned = g.clone();
    assert_eq!(g.inline_rules, cloned.inline_rules);
}

// ===========================================================================
// 14. Clone preserves precedences
// ===========================================================================

#[test]
fn test_clone_preserves_precedence_count() {
    let g = gc_v9_complex();
    let cloned = g.clone();
    assert_eq!(g.precedences.len(), cloned.precedences.len());
}

#[test]
fn test_clone_preserves_precedence_levels() {
    let g = gc_v9_complex();
    let cloned = g.clone();
    assert_eq!(cloned.precedences[0].level, 1);
    assert_eq!(cloned.precedences[1].level, 2);
}

#[test]
fn test_clone_preserves_precedence_associativity() {
    let g = gc_v9_complex();
    let cloned = g.clone();
    assert_eq!(cloned.precedences[0].associativity, Associativity::Left);
    assert_eq!(cloned.precedences[1].associativity, Associativity::Right);
}

#[test]
fn test_clone_preserves_precedence_symbols() {
    let g = gc_v9_complex();
    let cloned = g.clone();
    assert_eq!(g.precedences[0].symbols, cloned.precedences[0].symbols);
    assert_eq!(g.precedences[1].symbols, cloned.precedences[1].symbols);
}

#[test]
fn test_clone_builder_precedence_grammar() {
    let g = gc_v9_precedence();
    let cloned = g.clone();
    assert_eq!(g, cloned);
    assert_eq!(g.all_rules().count(), cloned.all_rules().count());
}

// ===========================================================================
// 15. Clone preserves conflicts, externals, extras
// ===========================================================================

#[test]
fn test_clone_preserves_conflicts() {
    let g = gc_v9_complex();
    let cloned = g.clone();
    assert_eq!(g.conflicts.len(), cloned.conflicts.len());
    assert_eq!(g.conflicts[0].symbols, cloned.conflicts[0].symbols);
}

#[test]
fn test_clone_preserves_conflict_resolution() {
    let g = gc_v9_complex();
    let cloned = g.clone();
    assert_eq!(g.conflicts[0].resolution, cloned.conflicts[0].resolution);
}

#[test]
fn test_clone_preserves_externals() {
    let g = gc_v9_complex();
    let cloned = g.clone();
    assert_eq!(g.externals.len(), cloned.externals.len());
    assert_eq!(g.externals[0].name, cloned.externals[0].name);
    assert_eq!(g.externals[0].symbol_id, cloned.externals[0].symbol_id);
}

#[test]
fn test_clone_preserves_extras() {
    let g = gc_v9_complex();
    let cloned = g.clone();
    assert_eq!(g.extras, cloned.extras);
}

// ===========================================================================
// 16. Various grammar sizes (1–15 rules)
// ===========================================================================

#[test]
fn test_clone_1_rule_grammar() {
    let g = gc_v9_n_rules(1);
    let cloned = g.clone();
    assert_eq!(g, cloned);
}

#[test]
fn test_clone_2_rule_grammar() {
    let g = gc_v9_n_rules(2);
    let cloned = g.clone();
    assert_eq!(g, cloned);
}

#[test]
fn test_clone_3_rule_grammar() {
    let g = gc_v9_n_rules(3);
    let cloned = g.clone();
    assert_eq!(g, cloned);
}

#[test]
fn test_clone_4_rule_grammar() {
    let g = gc_v9_n_rules(4);
    let cloned = g.clone();
    assert_eq!(g, cloned);
}

#[test]
fn test_clone_5_rule_grammar() {
    let g = gc_v9_n_rules(5);
    let cloned = g.clone();
    assert_eq!(g, cloned);
}

#[test]
fn test_clone_6_rule_grammar() {
    let g = gc_v9_n_rules(6);
    let cloned = g.clone();
    assert_eq!(g, cloned);
}

#[test]
fn test_clone_7_rule_grammar() {
    let g = gc_v9_n_rules(7);
    let cloned = g.clone();
    assert_eq!(g, cloned);
}

#[test]
fn test_clone_8_rule_grammar() {
    let g = gc_v9_n_rules(8);
    let cloned = g.clone();
    assert_eq!(g, cloned);
}

#[test]
fn test_clone_9_rule_grammar() {
    let g = gc_v9_n_rules(9);
    let cloned = g.clone();
    assert_eq!(g, cloned);
}

#[test]
fn test_clone_10_rule_grammar() {
    let g = gc_v9_n_rules(10);
    let cloned = g.clone();
    assert_eq!(g, cloned);
}

#[test]
fn test_clone_11_rule_grammar() {
    let g = gc_v9_n_rules(11);
    let cloned = g.clone();
    assert_eq!(g, cloned);
}

#[test]
fn test_clone_12_rule_grammar() {
    let g = gc_v9_n_rules(12);
    let cloned = g.clone();
    assert_eq!(g, cloned);
}

#[test]
fn test_clone_13_rule_grammar() {
    let g = gc_v9_n_rules(13);
    let cloned = g.clone();
    assert_eq!(g, cloned);
}

#[test]
fn test_clone_14_rule_grammar() {
    let g = gc_v9_n_rules(14);
    let cloned = g.clone();
    assert_eq!(g, cloned);
}

#[test]
fn test_clone_15_rule_grammar() {
    let g = gc_v9_n_rules(15);
    let cloned = g.clone();
    assert_eq!(g, cloned);
}

// ===========================================================================
// Additional clone independence tests
// ===========================================================================

#[test]
fn test_clone_independence_name_mutation() {
    let g = gc_v9_single_token();
    let mut cloned = g.clone();
    cloned.name = "gc_v9_changed".to_string();
    assert_eq!(g.name, "gc_v9_single_token");
    assert_ne!(g, cloned);
}

#[test]
fn test_clone_independence_add_token() {
    let g = gc_v9_single_token();
    let original_token_count = g.tokens.len();
    let mut cloned = g.clone();
    cloned.tokens.insert(
        SymbolId(999),
        Token {
            name: "NEW".to_string(),
            pattern: TokenPattern::String("new".to_string()),
            fragile: false,
        },
    );
    assert_eq!(g.tokens.len(), original_token_count);
    assert_eq!(cloned.tokens.len(), original_token_count + 1);
}

#[test]
fn test_clone_independence_add_rule() {
    let g = gc_v9_single_token();
    let original_rule_count = g.all_rules().count();
    let mut cloned = g.clone();
    cloned.add_rule(Rule {
        lhs: SymbolId(500),
        rhs: vec![Symbol::Epsilon],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(99),
    });
    assert_eq!(g.all_rules().count(), original_rule_count);
    assert!(cloned.all_rules().count() > original_rule_count);
}

#[test]
fn test_clone_independence_add_precedence() {
    let g = gc_v9_complex();
    let original_prec_count = g.precedences.len();
    let mut cloned = g.clone();
    cloned.precedences.push(Precedence {
        level: 99,
        associativity: Associativity::None,
        symbols: vec![],
    });
    assert_eq!(g.precedences.len(), original_prec_count);
    assert_eq!(cloned.precedences.len(), original_prec_count + 1);
}

#[test]
fn test_clone_independence_add_conflict() {
    let g = gc_v9_complex();
    let original_conflict_count = g.conflicts.len();
    let mut cloned = g.clone();
    cloned.conflicts.push(ConflictDeclaration {
        symbols: vec![SymbolId(50)],
        resolution: ConflictResolution::GLR,
    });
    assert_eq!(g.conflicts.len(), original_conflict_count);
    assert_eq!(cloned.conflicts.len(), original_conflict_count + 1);
}

#[test]
fn test_clone_independence_add_external() {
    let g = gc_v9_complex();
    let original_ext_count = g.externals.len();
    let mut cloned = g.clone();
    cloned.externals.push(ExternalToken {
        name: "dedent".to_string(),
        symbol_id: SymbolId(201),
    });
    assert_eq!(g.externals.len(), original_ext_count);
    assert_eq!(cloned.externals.len(), original_ext_count + 1);
}

#[test]
fn test_clone_independence_add_extra() {
    let g = gc_v9_complex();
    let original_extras_count = g.extras.len();
    let mut cloned = g.clone();
    cloned.extras.push(SymbolId(301));
    assert_eq!(g.extras.len(), original_extras_count);
    assert_eq!(cloned.extras.len(), original_extras_count + 1);
}

#[test]
fn test_clone_independence_add_field() {
    let g = gc_v9_complex();
    let original_field_count = g.fields.len();
    let mut cloned = g.clone();
    cloned.fields.insert(FieldId(10), "body".to_string());
    assert_eq!(g.fields.len(), original_field_count);
    assert_eq!(cloned.fields.len(), original_field_count + 1);
}

#[test]
fn test_clone_independence_add_supertype() {
    let g = gc_v9_complex();
    let original_st_count = g.supertypes.len();
    let mut cloned = g.clone();
    cloned.supertypes.push(SymbolId(1));
    assert_eq!(g.supertypes.len(), original_st_count);
    assert_eq!(cloned.supertypes.len(), original_st_count + 1);
}

#[test]
fn test_clone_independence_add_inline() {
    let g = gc_v9_complex();
    let original_inline_count = g.inline_rules.len();
    let mut cloned = g.clone();
    cloned.inline_rules.push(SymbolId(0));
    assert_eq!(g.inline_rules.len(), original_inline_count);
    assert_eq!(cloned.inline_rules.len(), original_inline_count + 1);
}

// ===========================================================================
// Additional Debug formatting tests
// ===========================================================================

#[test]
fn test_debug_format_empty_grammar_non_empty() {
    let g = gc_v9_empty();
    let dbg = format!("{g:?}");
    assert!(!dbg.is_empty());
}

#[test]
fn test_debug_format_empty_grammar_contains_name() {
    let g = gc_v9_empty();
    let dbg = format!("{g:?}");
    assert!(dbg.contains("gc_v9_empty"));
}

#[test]
fn test_debug_format_complex_grammar_contains_name() {
    let g = gc_v9_complex();
    let dbg = format!("{g:?}");
    assert!(dbg.contains("gc_v9_complex"));
}

#[test]
fn test_debug_format_contains_tokens() {
    let g = gc_v9_two_tokens();
    let dbg = format!("{g:?}");
    assert!(dbg.contains("tokens"));
}

#[test]
fn test_debug_format_contains_precedences() {
    let g = gc_v9_complex();
    let dbg = format!("{g:?}");
    assert!(dbg.contains("precedences"));
}

#[test]
fn test_debug_format_contains_conflicts() {
    let g = gc_v9_complex();
    let dbg = format!("{g:?}");
    assert!(dbg.contains("conflicts"));
}

#[test]
fn test_debug_format_contains_externals() {
    let g = gc_v9_complex();
    let dbg = format!("{g:?}");
    assert!(dbg.contains("externals"));
}

#[test]
fn test_debug_format_contains_extras() {
    let g = gc_v9_complex();
    let dbg = format!("{g:?}");
    assert!(dbg.contains("extras"));
}

#[test]
fn test_debug_format_clone_matches_original() {
    let g = gc_v9_multi_rule();
    let cloned = g.clone();
    assert_eq!(format!("{g:?}"), format!("{cloned:?}"));
}

// ===========================================================================
// Equality tests
// ===========================================================================

#[test]
fn test_equality_same_builder_same_grammar() {
    let g1 = gc_v9_single_token();
    let g2 = gc_v9_single_token();
    assert_eq!(g1, g2);
}

#[test]
fn test_inequality_different_names() {
    let g1 = GrammarBuilder::new("gc_v9_alpha")
        .token("x", "x")
        .rule("start", vec!["x"])
        .start("start")
        .build();
    let g2 = GrammarBuilder::new("gc_v9_beta")
        .token("x", "x")
        .rule("start", vec!["x"])
        .start("start")
        .build();
    assert_ne!(g1, g2);
}

#[test]
fn test_inequality_different_token_count() {
    let g1 = gc_v9_single_token();
    let g2 = gc_v9_two_tokens();
    assert_ne!(g1, g2);
}

#[test]
fn test_equality_default_grammars() {
    let g1 = Grammar::default();
    let g2 = Grammar::default();
    assert_eq!(g1, g2);
}

#[test]
fn test_inequality_empty_vs_nonempty() {
    let g1 = gc_v9_empty();
    let g2 = gc_v9_single_token();
    assert_ne!(g1, g2);
}

// ===========================================================================
// N-rule grammar clone details
// ===========================================================================

#[test]
fn test_n_rules_clone_name_matches() {
    for n in 1u16..=5 {
        let g = gc_v9_n_rules(n);
        let cloned = g.clone();
        assert_eq!(g.name, cloned.name);
    }
}

#[test]
fn test_n_rules_clone_token_count_matches() {
    for n in 1u16..=5 {
        let g = gc_v9_n_rules(n);
        let cloned = g.clone();
        assert_eq!(g.tokens.len(), cloned.tokens.len());
    }
}

#[test]
fn test_n_rules_clone_rule_count_matches() {
    for n in 1u16..=5 {
        let g = gc_v9_n_rules(n);
        let cloned = g.clone();
        assert_eq!(g.all_rules().count(), cloned.all_rules().count());
    }
}

#[test]
fn test_n_rules_clone_independence() {
    for n in [3u16, 7, 12] {
        let g = gc_v9_n_rules(n);
        let mut cloned = g.clone();
        cloned.name = format!("gc_v9_{n}_modified");
        assert_ne!(g.name, cloned.name);
    }
}

// ===========================================================================
// Misc edge-case tests
// ===========================================================================

#[test]
fn test_clone_preserves_max_alias_sequence_length() {
    let g = gc_v9_complex();
    let cloned = g.clone();
    assert_eq!(
        g.max_alias_sequence_length,
        cloned.max_alias_sequence_length
    );
}

#[test]
fn test_clone_preserves_symbol_registry_none() {
    let g = gc_v9_single_token();
    let cloned = g.clone();
    assert_eq!(
        g.symbol_registry.is_none(),
        cloned.symbol_registry.is_none()
    );
}

#[test]
fn test_clone_of_clone_equals_original() {
    let g = gc_v9_complex();
    let c1 = g.clone();
    let c2 = c1.clone();
    assert_eq!(g, c2);
}

#[test]
fn test_clone_then_validate() {
    let g = gc_v9_single_token();
    let cloned = g.clone();
    // Both should have the same validation outcome.
    let v1 = g.validate();
    let v2 = cloned.validate();
    assert_eq!(v1.is_ok(), v2.is_ok());
}

#[test]
fn test_debug_format_multi_rule_contains_name() {
    let g = gc_v9_multi_rule();
    let dbg = format!("{g:?}");
    assert!(dbg.contains("gc_v9_multi_rule"));
}

#[test]
fn test_debug_format_precedence_grammar_contains_name() {
    let g = gc_v9_precedence();
    let dbg = format!("{g:?}");
    assert!(dbg.contains("gc_v9_precedence"));
}

#[test]
fn test_clone_equality_reflexive() {
    let g = gc_v9_complex();
    assert_eq!(g, g);
}

#[test]
fn test_clone_equality_symmetric() {
    let g = gc_v9_complex();
    let cloned = g.clone();
    assert_eq!(g, cloned);
    assert_eq!(cloned, g);
}

#[test]
fn test_clone_equality_transitive() {
    let g = gc_v9_complex();
    let c1 = g.clone();
    let c2 = c1.clone();
    assert_eq!(g, c1);
    assert_eq!(c1, c2);
    assert_eq!(g, c2);
}
