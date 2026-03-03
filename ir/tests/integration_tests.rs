//! Comprehensive integration tests for the adze-ir crate.
//!
//! Tests the full pipeline: construction → validation → normalization → optimization,
//! serialization roundtrips, complex grammar patterns, and symbol registry correctness.

use adze_ir::builder::GrammarBuilder;
use adze_ir::optimizer::{GrammarOptimizer, optimize_grammar};
use adze_ir::validation::GrammarValidator;
use adze_ir::*;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Build a minimal arithmetic grammar via the builder API.
fn arithmetic_grammar() -> Grammar {
    GrammarBuilder::new("arithmetic")
        .token("NUMBER", r"\d+")
        .token("+", "+")
        .token("-", "-")
        .token("*", "*")
        .token("/", "/")
        .token("(", "(")
        .token(")", ")")
        .rule("expr", vec!["expr", "+", "expr"])
        .rule("expr", vec!["expr", "-", "expr"])
        .rule("expr", vec!["expr", "*", "expr"])
        .rule("expr", vec!["expr", "/", "expr"])
        .rule("expr", vec!["(", "expr", ")"])
        .rule("expr", vec!["NUMBER"])
        .start("expr")
        .build()
}

/// Build a grammar that exercises nested optionals inside repeats inside choices.
fn complex_nested_grammar() -> Grammar {
    let mut grammar = Grammar::new("complex_nested".to_string());

    // Tokens
    let tok_a = SymbolId(1);
    let tok_b = SymbolId(2);
    let tok_c = SymbolId(3);
    grammar.tokens.insert(
        tok_a,
        Token {
            name: "a".into(),
            pattern: TokenPattern::String("a".into()),
            fragile: false,
        },
    );
    grammar.tokens.insert(
        tok_b,
        Token {
            name: "b".into(),
            pattern: TokenPattern::String("b".into()),
            fragile: false,
        },
    );
    grammar.tokens.insert(
        tok_c,
        Token {
            name: "c".into(),
            pattern: TokenPattern::String("c".into()),
            fragile: false,
        },
    );

    let start = SymbolId(10);
    grammar.rule_names.insert(start, "start".into());

    // start → Choice(Repeat(Optional(a)), Sequence(b, c))
    // This nests Optional inside Repeat inside Choice.
    let nested_symbol = Symbol::Choice(vec![
        Symbol::Repeat(Box::new(Symbol::Optional(Box::new(Symbol::Terminal(
            tok_a,
        ))))),
        Symbol::Sequence(vec![Symbol::Terminal(tok_b), Symbol::Terminal(tok_c)]),
    ]);

    grammar.add_rule(Rule {
        lhs: start,
        rhs: vec![nested_symbol],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });

    grammar
}

// ===========================================================================
// 1. Full grammar construction and normalization pipeline
// ===========================================================================

#[test]
fn test_full_construction_and_normalization() {
    let mut grammar = arithmetic_grammar();

    // Before normalization the grammar should be valid (builder produces simple symbols).
    assert!(grammar.validate().is_ok());

    let rules_before = grammar.rules.values().map(|v| v.len()).sum::<usize>();

    // Normalize (no-op for already-flat grammars, but must not panic).
    grammar.normalize();

    let rules_after = grammar.rules.values().map(|v| v.len()).sum::<usize>();
    // Flat grammars should stay the same size.
    assert_eq!(rules_before, rules_after);
}

#[test]
fn test_normalization_expands_complex_symbols() {
    let mut grammar = complex_nested_grammar();

    let lhs_count_before = grammar.rules.len();

    grammar.normalize();

    // Normalization should have created auxiliary rules (new LHS symbols).
    assert!(
        grammar.rules.len() > lhs_count_before,
        "Expected new auxiliary rules after normalization, but LHS count did not increase"
    );

    // After normalization every RHS symbol must be Terminal, NonTerminal, External, or Epsilon.
    for rule in grammar.all_rules() {
        for sym in &rule.rhs {
            match sym {
                Symbol::Terminal(_)
                | Symbol::NonTerminal(_)
                | Symbol::External(_)
                | Symbol::Epsilon => {}
                other => panic!("Found complex symbol after normalization: {:?}", other),
            }
        }
    }
}

// ===========================================================================
// 2. Complex grammar patterns (nested optionals in repeats in choices)
// ===========================================================================

#[test]
fn test_nested_optional_in_repeat_in_choice() {
    let mut grammar = complex_nested_grammar();
    grammar.normalize();

    // Collect all auxiliary symbol IDs (≥ 1000 offset from original max).
    let aux_ids: Vec<SymbolId> = grammar
        .rules
        .keys()
        .copied()
        .filter(|id| id.0 >= 1000)
        .collect();

    // We expect at least three auxiliary rules:
    //   one for Optional(a), one for Repeat(...), one for Choice(...)
    assert!(
        aux_ids.len() >= 3,
        "Expected at least 3 auxiliary symbols, got {}",
        aux_ids.len()
    );
}

#[test]
fn test_repeat_one_normalization() {
    let mut grammar = Grammar::new("repeat_one".into());

    let tok = SymbolId(1);
    grammar.tokens.insert(
        tok,
        Token {
            name: "x".into(),
            pattern: TokenPattern::String("x".into()),
            fragile: false,
        },
    );
    let start = SymbolId(10);
    grammar.rule_names.insert(start, "items".into());
    grammar.add_rule(Rule {
        lhs: start,
        rhs: vec![Symbol::RepeatOne(Box::new(Symbol::Terminal(tok)))],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });

    grammar.normalize();

    // RepeatOne(x) should produce aux → aux x | x (two rules for the aux symbol).
    let aux_rules: Vec<&Rule> = grammar.all_rules().filter(|r| r.lhs.0 >= 1000).collect();
    assert!(
        aux_rules.len() >= 2,
        "RepeatOne should generate at least 2 auxiliary rules, got {}",
        aux_rules.len()
    );
}

// ===========================================================================
// 3. Symbol registry correctness after normalization
// ===========================================================================

#[test]
fn test_symbol_registry_after_normalization() {
    let mut grammar = complex_nested_grammar();
    grammar.normalize();

    let registry = grammar.build_registry();

    // Registry must contain at least the tokens.
    assert!(registry.get_id("a").is_some());
    assert!(registry.get_id("b").is_some());
    assert!(registry.get_id("c").is_some());

    // EOF is always symbol 0.
    assert_eq!(registry.get_id("end"), Some(SymbolId(0)));
}

#[test]
fn test_registry_determinism() {
    let g1 = arithmetic_grammar();
    let g2 = arithmetic_grammar();

    let r1 = g1.build_registry();
    let r2 = g2.build_registry();

    // Two identical grammars must produce identical registries.
    for (name, info1) in r1.iter() {
        let info2_id = r2
            .get_id(name)
            .unwrap_or_else(|| panic!("missing symbol '{}'", name));
        assert_eq!(info1.id, info2_id, "ID mismatch for '{}'", name);
    }
    assert_eq!(r1.len(), r2.len());
}

#[test]
fn test_get_or_build_registry_caches() {
    let mut grammar = arithmetic_grammar();

    let len1 = grammar.get_or_build_registry().len();
    let len2 = grammar.get_or_build_registry().len();
    assert_eq!(len1, len2, "Cached registry should be identical");
}

// ===========================================================================
// 4. Validation → normalization → optimization pipeline
// ===========================================================================

#[test]
fn test_validation_normalization_optimization_pipeline() {
    let mut grammar = GrammarBuilder::javascript_like();

    // Step 1: validate
    let mut validator = GrammarValidator::new();
    let result = validator.validate(&grammar);
    // EmptyGrammar should NOT appear.
    assert!(
        !result
            .errors
            .iter()
            .any(|e| matches!(e, validation::ValidationError::EmptyGrammar)),
        "Non-empty grammar reported as empty"
    );

    // Step 2: normalize (JS-like grammar is already flat, should be no-op).
    grammar.normalize();

    // Step 3: optimize
    let grammar = optimize_grammar(grammar).expect("optimization should not fail");

    // Grammar should still have rules.
    assert!(!grammar.rules.is_empty(), "Optimization removed all rules");
}

#[test]
fn test_optimizer_stats() {
    let mut grammar = GrammarBuilder::javascript_like();
    let mut optimizer = GrammarOptimizer::new();
    let stats = optimizer.optimize(&mut grammar);

    // Stats should be populated (exact values may vary).
    // Just check they are non-negative and the grammar is still valid.
    assert!(stats.removed_unused_symbols <= 100);
    assert!(!grammar.rules.is_empty());
}

#[test]
fn test_validation_catches_empty_grammar() {
    let grammar = Grammar::new("empty".into());
    let mut validator = GrammarValidator::new();
    let result = validator.validate(&grammar);
    assert!(
        result
            .errors
            .iter()
            .any(|e| matches!(e, validation::ValidationError::EmptyGrammar)),
        "Expected EmptyGrammar error"
    );
}

// ===========================================================================
// 5. Serialization → deserialization roundtrip for complex grammars
// ===========================================================================

#[test]
fn test_serialization_roundtrip_simple() {
    let grammar = arithmetic_grammar();
    let json = serde_json::to_string(&grammar).expect("serialize");
    let deserialized: Grammar = serde_json::from_str(&json).expect("deserialize");

    assert_eq!(grammar.name, deserialized.name);
    assert_eq!(grammar.tokens.len(), deserialized.tokens.len());
    assert_eq!(grammar.rules.len(), deserialized.rules.len());
    for (id, rules) in &grammar.rules {
        let deser_rules = deserialized.rules.get(id).expect("missing rule LHS");
        assert_eq!(rules.len(), deser_rules.len());
    }
}

#[test]
fn test_serialization_roundtrip_complex_symbols() {
    let grammar = complex_nested_grammar();
    let json = serde_json::to_string_pretty(&grammar).expect("serialize");
    let deserialized: Grammar = serde_json::from_str(&json).expect("deserialize");

    assert_eq!(grammar.name, deserialized.name);

    // The complex RHS symbols must survive the roundtrip.
    let original_rule = grammar.all_rules().next().unwrap();
    let deser_rule = deserialized.all_rules().next().unwrap();
    assert_eq!(original_rule.rhs, deser_rule.rhs);
}

#[test]
fn test_serialization_roundtrip_after_normalization() {
    let mut grammar = complex_nested_grammar();
    grammar.normalize();

    let json = serde_json::to_string(&grammar).expect("serialize");
    let deserialized: Grammar = serde_json::from_str(&json).expect("deserialize");

    // Normalized grammar should roundtrip with the same rule count.
    let original_count: usize = grammar.rules.values().map(|v| v.len()).sum();
    let deser_count: usize = deserialized.rules.values().map(|v| v.len()).sum();
    assert_eq!(original_count, deser_count);
}

#[test]
fn test_serialization_roundtrip_with_externals() {
    let grammar = GrammarBuilder::python_like();
    let json = serde_json::to_string(&grammar).expect("serialize");
    let deserialized: Grammar = serde_json::from_str(&json).expect("deserialize");

    assert_eq!(grammar.externals.len(), deserialized.externals.len());
    for (orig, deser) in grammar.externals.iter().zip(deserialized.externals.iter()) {
        assert_eq!(orig.name, deser.name);
        assert_eq!(orig.symbol_id, deser.symbol_id);
    }
}

#[test]
fn test_serialization_roundtrip_precedences() {
    let grammar = GrammarBuilder::new("prec_test")
        .token("NUMBER", r"\d+")
        .token("+", "+")
        .token("*", "*")
        .rule("expr", vec!["NUMBER"])
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "*", "expr"], 2, Associativity::Left)
        .precedence(1, Associativity::Left, vec!["+"])
        .precedence(2, Associativity::Left, vec!["*"])
        .start("expr")
        .build();

    let json = serde_json::to_string(&grammar).expect("serialize");
    let deserialized: Grammar = serde_json::from_str(&json).expect("deserialize");

    assert_eq!(grammar.precedences.len(), deserialized.precedences.len());
    for (orig, deser) in grammar
        .precedences
        .iter()
        .zip(deserialized.precedences.iter())
    {
        assert_eq!(orig.level, deser.level);
        assert_eq!(orig.associativity, deser.associativity);
        assert_eq!(orig.symbols, deser.symbols);
    }
}

// ===========================================================================
// 6. Grammar merging / composition patterns
// ===========================================================================

#[test]
fn test_grammar_merge_rules() {
    // Simulate merging rules from two grammar fragments.
    let mut base = Grammar::new("merged".into());
    let tok_num = SymbolId(1);
    let tok_plus = SymbolId(2);
    let tok_star = SymbolId(3);
    base.tokens.insert(
        tok_num,
        Token {
            name: "NUMBER".into(),
            pattern: TokenPattern::Regex(r"\d+".into()),
            fragile: false,
        },
    );
    base.tokens.insert(
        tok_plus,
        Token {
            name: "+".into(),
            pattern: TokenPattern::String("+".into()),
            fragile: false,
        },
    );
    base.tokens.insert(
        tok_star,
        Token {
            name: "*".into(),
            pattern: TokenPattern::String("*".into()),
            fragile: false,
        },
    );

    let expr = SymbolId(10);
    base.rule_names.insert(expr, "expr".into());

    // Fragment 1: addition
    base.add_rule(Rule {
        lhs: expr,
        rhs: vec![
            Symbol::NonTerminal(expr),
            Symbol::Terminal(tok_plus),
            Symbol::NonTerminal(expr),
        ],
        precedence: Some(PrecedenceKind::Static(1)),
        associativity: Some(Associativity::Left),
        fields: vec![],
        production_id: ProductionId(0),
    });

    // Fragment 2: multiplication
    base.add_rule(Rule {
        lhs: expr,
        rhs: vec![
            Symbol::NonTerminal(expr),
            Symbol::Terminal(tok_star),
            Symbol::NonTerminal(expr),
        ],
        precedence: Some(PrecedenceKind::Static(2)),
        associativity: Some(Associativity::Left),
        fields: vec![],
        production_id: ProductionId(1),
    });

    // Fragment 3: base case
    base.add_rule(Rule {
        lhs: expr,
        rhs: vec![Symbol::Terminal(tok_num)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(2),
    });

    assert_eq!(base.get_rules_for_symbol(expr).unwrap().len(), 3);
    assert!(base.validate().is_ok());
}

#[test]
fn test_grammar_composition_with_shared_tokens() {
    // Two grammars share a NUMBER token; merge should not duplicate.
    let mut grammar = Grammar::new("composed".into());

    let tok_num = SymbolId(1);
    grammar.tokens.insert(
        tok_num,
        Token {
            name: "NUMBER".into(),
            pattern: TokenPattern::Regex(r"\d+".into()),
            fragile: false,
        },
    );

    let sym_a = SymbolId(10);
    let sym_b = SymbolId(11);
    grammar.rule_names.insert(sym_a, "group_a".into());
    grammar.rule_names.insert(sym_b, "group_b".into());

    grammar.add_rule(Rule {
        lhs: sym_a,
        rhs: vec![Symbol::Terminal(tok_num)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    grammar.add_rule(Rule {
        lhs: sym_b,
        rhs: vec![Symbol::Terminal(tok_num)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(1),
    });

    // Both rules reference the same token; only one token entry.
    assert_eq!(grammar.tokens.len(), 1);
    assert!(grammar.validate().is_ok());
}

// ===========================================================================
// 7. Precedence and associativity configuration
// ===========================================================================

#[test]
fn test_precedence_levels_and_associativity() {
    let grammar = GrammarBuilder::new("prec")
        .token("NUMBER", r"\d+")
        .token("+", "+")
        .token("*", "*")
        .token("^", "^")
        .rule("expr", vec!["NUMBER"])
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "*", "expr"], 2, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "^", "expr"], 3, Associativity::Right)
        .start("expr")
        .build();

    let expr_id = grammar.find_symbol_by_name("expr").expect("expr symbol");
    let rules = grammar.get_rules_for_symbol(expr_id).unwrap();

    // Find the power rule (prec=3, right-assoc).
    let power_rule = rules
        .iter()
        .find(|r| r.precedence == Some(PrecedenceKind::Static(3)))
        .expect("power rule");
    assert_eq!(power_rule.associativity, Some(Associativity::Right));

    // All precedence-bearing rules should have their levels.
    let precs: Vec<Option<PrecedenceKind>> = rules.iter().map(|r| r.precedence).collect();
    assert!(precs.contains(&Some(PrecedenceKind::Static(1))));
    assert!(precs.contains(&Some(PrecedenceKind::Static(2))));
    assert!(precs.contains(&Some(PrecedenceKind::Static(3))));
}

#[test]
fn test_dynamic_precedence() {
    let mut grammar = Grammar::new("dyn_prec".into());

    let tok = SymbolId(1);
    grammar.tokens.insert(
        tok,
        Token {
            name: "x".into(),
            pattern: TokenPattern::String("x".into()),
            fragile: false,
        },
    );

    let start = SymbolId(10);
    grammar.rule_names.insert(start, "s".into());
    grammar.add_rule(Rule {
        lhs: start,
        rhs: vec![Symbol::Terminal(tok)],
        precedence: Some(PrecedenceKind::Dynamic(5)),
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });

    assert!(grammar.validate().is_ok());
    let json = serde_json::to_string(&grammar).unwrap();
    let deser: Grammar = serde_json::from_str(&json).unwrap();
    let rule = deser.all_rules().next().unwrap();
    assert_eq!(rule.precedence, Some(PrecedenceKind::Dynamic(5)));
}

#[test]
fn test_conflict_declarations() {
    let mut grammar = Grammar::new("conflicts".into());

    let tok_a = SymbolId(1);
    let tok_b = SymbolId(2);
    grammar.tokens.insert(
        tok_a,
        Token {
            name: "a".into(),
            pattern: TokenPattern::String("a".into()),
            fragile: false,
        },
    );
    grammar.tokens.insert(
        tok_b,
        Token {
            name: "b".into(),
            pattern: TokenPattern::String("b".into()),
            fragile: false,
        },
    );

    let sym_s = SymbolId(10);
    grammar.rule_names.insert(sym_s, "s".into());
    grammar.add_rule(Rule {
        lhs: sym_s,
        rhs: vec![Symbol::Terminal(tok_a)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });

    grammar.conflicts.push(ConflictDeclaration {
        symbols: vec![tok_a, tok_b],
        resolution: ConflictResolution::GLR,
    });

    assert_eq!(grammar.conflicts.len(), 1);
    assert_eq!(grammar.conflicts[0].resolution, ConflictResolution::GLR);

    let json = serde_json::to_string(&grammar).unwrap();
    let deser: Grammar = serde_json::from_str(&json).unwrap();
    assert_eq!(deser.conflicts.len(), 1);
}

// ===========================================================================
// 8. External token integration with grammar
// ===========================================================================

#[test]
fn test_external_tokens_in_rules() {
    let grammar = GrammarBuilder::python_like();

    // The Python-like grammar declares INDENT and DEDENT as externals.
    assert!(grammar.externals.len() >= 2);
    let ext_names: Vec<&str> = grammar.externals.iter().map(|e| e.name.as_str()).collect();
    assert!(ext_names.contains(&"INDENT"));
    assert!(ext_names.contains(&"DEDENT"));
}

#[test]
fn test_external_token_validation() {
    let mut grammar = Grammar::new("ext_test".into());

    let ext_id = SymbolId(50);
    grammar.externals.push(ExternalToken {
        name: "HEREDOC".into(),
        symbol_id: ext_id,
    });

    let start = SymbolId(10);
    let tok_a = SymbolId(1);
    grammar.tokens.insert(
        tok_a,
        Token {
            name: "a".into(),
            pattern: TokenPattern::String("a".into()),
            fragile: false,
        },
    );
    grammar.rule_names.insert(start, "s".into());

    // Rule referencing the external token should validate.
    grammar.add_rule(Rule {
        lhs: start,
        rhs: vec![Symbol::Terminal(tok_a), Symbol::External(ext_id)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });

    assert!(grammar.validate().is_ok());
}

#[test]
fn test_external_token_in_registry() {
    let mut grammar = Grammar::new("ext_reg".into());

    let ext_id = SymbolId(50);
    grammar.externals.push(ExternalToken {
        name: "TEMPLATE_LITERAL".into(),
        symbol_id: ext_id,
    });

    let tok = SymbolId(1);
    grammar.tokens.insert(
        tok,
        Token {
            name: "x".into(),
            pattern: TokenPattern::String("x".into()),
            fragile: false,
        },
    );
    let start = SymbolId(10);
    grammar.rule_names.insert(start, "s".into());
    grammar.add_rule(Rule {
        lhs: start,
        rhs: vec![Symbol::Terminal(tok)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });

    let registry = grammar.build_registry();
    assert!(
        registry.get_id("TEMPLATE_LITERAL").is_some(),
        "External token should appear in registry"
    );
}

// ===========================================================================
// Additional: edge-case and smoke tests
// ===========================================================================

#[test]
fn test_normalize_idempotent() {
    let mut grammar = complex_nested_grammar();
    grammar.normalize();
    let count1: usize = grammar.rules.values().map(|v| v.len()).sum();

    grammar.normalize();
    let count2: usize = grammar.rules.values().map(|v| v.len()).sum();

    assert_eq!(count1, count2, "Normalization should be idempotent");
}

#[test]
fn test_find_symbol_by_name() {
    let grammar = arithmetic_grammar();
    assert!(grammar.find_symbol_by_name("expr").is_some());
    assert!(grammar.find_symbol_by_name("nonexistent").is_none());
}

#[test]
fn test_start_symbol_detection() {
    let grammar = GrammarBuilder::python_like();
    // The Python-like grammar sets "module" as start via builder.
    // start_symbol() should find the first rule symbol.
    assert!(grammar.start_symbol().is_some());
}

#[test]
fn test_empty_terminal_check() {
    let mut grammar = Grammar::new("empty_tok".into());
    grammar.tokens.insert(
        SymbolId(1),
        Token {
            name: "BAD".into(),
            pattern: TokenPattern::String(String::new()),
            fragile: false,
        },
    );
    assert!(grammar.check_empty_terminals().is_err());
}

#[test]
fn test_serialization_roundtrip_default_grammar() {
    let grammar = Grammar::default();
    let json = serde_json::to_string(&grammar).unwrap();
    let deser: Grammar = serde_json::from_str(&json).unwrap();
    assert_eq!(grammar.name, deser.name);
    assert!(deser.rules.is_empty());
}

#[test]
fn test_validation_stats_populated() {
    let grammar = GrammarBuilder::javascript_like();
    let mut validator = GrammarValidator::new();
    let result = validator.validate(&grammar);

    assert!(result.stats.total_rules > 0);
    assert!(result.stats.total_tokens > 0);
    assert!(result.stats.total_symbols > 0);
    assert!(result.stats.max_rule_length > 0);
}

#[test]
fn test_fragile_token_roundtrip() {
    let grammar = GrammarBuilder::new("fragile")
        .fragile_token("ERROR_RECOVERY", ".*")
        .token("ID", r"[a-z]+")
        .rule("s", vec!["ID"])
        .start("s")
        .build();

    let json = serde_json::to_string(&grammar).unwrap();
    let deser: Grammar = serde_json::from_str(&json).unwrap();

    let fragile_count = deser.tokens.values().filter(|t| t.fragile).count();
    assert_eq!(fragile_count, 1);
}

#[test]
fn test_extras_survive_roundtrip() {
    let grammar = GrammarBuilder::new("extras")
        .token("WHITESPACE", r"[ \t]+")
        .token("ID", r"[a-z]+")
        .extra("WHITESPACE")
        .rule("s", vec!["ID"])
        .start("s")
        .build();

    assert!(!grammar.extras.is_empty());

    let json = serde_json::to_string(&grammar).unwrap();
    let deser: Grammar = serde_json::from_str(&json).unwrap();
    assert_eq!(grammar.extras.len(), deser.extras.len());
}
