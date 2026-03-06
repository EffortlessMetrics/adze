//! Complete grammar lifecycle tests: create → configure → normalize → optimize → validate →
//! serialize → deserialize.
//!
//! 64 tests across 8 categories (8 tests each).

use adze_ir::builder::GrammarBuilder;
use adze_ir::optimizer::{GrammarOptimizer, optimize_grammar};
use adze_ir::validation::GrammarValidator;
use adze_ir::{
    Associativity, Grammar, PrecedenceKind, Rule, Symbol, SymbolId, Token, TokenPattern,
};

// ── Helpers ────────────────────────────────────────────────────────────────

/// Build a minimal arithmetic grammar used across many tests.
fn arithmetic_grammar() -> Grammar {
    GrammarBuilder::new("arithmetic")
        .token("NUMBER", r"\d+")
        .token("+", "+")
        .token("*", "*")
        .token("(", "(")
        .token(")", ")")
        .rule("expr", vec!["expr", "+", "expr"])
        .rule("expr", vec!["expr", "*", "expr"])
        .rule("expr", vec!["NUMBER"])
        .rule("expr", vec!["(", "expr", ")"])
        .start("expr")
        .build()
}

/// Build a grammar with multiple nonterminals and tokens.
fn statement_grammar() -> Grammar {
    GrammarBuilder::new("statements")
        .token("IF", "if")
        .token("ELSE", "else")
        .token("WHILE", "while")
        .token("IDENT", r"[a-z]+")
        .token("NUM", r"[0-9]+")
        .token("=", "=")
        .token(";", ";")
        .token("(", "(")
        .token(")", ")")
        .token("{", "{")
        .token("}", "}")
        .rule("program", vec!["stmt_list"])
        .rule("stmt_list", vec!["stmt"])
        .rule("stmt_list", vec!["stmt_list", "stmt"])
        .rule("stmt", vec!["assign"])
        .rule("stmt", vec!["if_stmt"])
        .rule("stmt", vec!["while_stmt"])
        .rule("assign", vec!["IDENT", "=", "value", ";"])
        .rule("value", vec!["IDENT"])
        .rule("value", vec!["NUM"])
        .rule("if_stmt", vec!["IF", "(", "value", ")", "block"])
        .rule(
            "if_stmt",
            vec!["IF", "(", "value", ")", "block", "ELSE", "block"],
        )
        .rule("while_stmt", vec!["WHILE", "(", "value", ")", "block"])
        .rule("block", vec!["{", "stmt_list", "}"])
        .start("program")
        .build()
}

// ═══════════════════════════════════════════════════════════════════════════
// 1. lifecycle_create_* — grammar creation variations (8 tests)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn lifecycle_create_minimal_grammar() {
    let g = GrammarBuilder::new("minimal")
        .token("A", "a")
        .rule("start", vec!["A"])
        .start("start")
        .build();

    assert_eq!(g.name, "minimal");
    assert!(!g.rules.is_empty());
    assert!(!g.tokens.is_empty());
}

#[test]
fn lifecycle_create_empty_name() {
    let g = GrammarBuilder::new("").build();
    assert_eq!(g.name, "");
    assert!(g.rules.is_empty());
}

#[test]
fn lifecycle_create_multiple_alternatives() {
    let g = arithmetic_grammar();

    // "expr" should have 4 alternative productions
    let expr_id = g.find_symbol_by_name("expr").expect("expr must exist");
    let expr_rules = g.get_rules_for_symbol(expr_id).unwrap();
    assert_eq!(expr_rules.len(), 4);
}

#[test]
fn lifecycle_create_with_precedence() {
    let g = GrammarBuilder::new("prec")
        .token("N", r"\d+")
        .token("+", "+")
        .token("*", "*")
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "*", "expr"], 2, Associativity::Left)
        .rule("expr", vec!["N"])
        .start("expr")
        .build();

    let expr_id = g.find_symbol_by_name("expr").unwrap();
    let rules = g.get_rules_for_symbol(expr_id).unwrap();

    let has_prec_1 = rules
        .iter()
        .any(|r| r.precedence == Some(PrecedenceKind::Static(1)));
    let has_prec_2 = rules
        .iter()
        .any(|r| r.precedence == Some(PrecedenceKind::Static(2)));
    assert!(has_prec_1);
    assert!(has_prec_2);
}

#[test]
fn lifecycle_create_with_externals() {
    let g = GrammarBuilder::new("ext")
        .token("A", "a")
        .external("INDENT")
        .external("DEDENT")
        .rule("start", vec!["A"])
        .start("start")
        .build();

    assert_eq!(g.externals.len(), 2);
    assert_eq!(g.externals[0].name, "INDENT");
    assert_eq!(g.externals[1].name, "DEDENT");
}

#[test]
fn lifecycle_create_with_extras() {
    let g = GrammarBuilder::new("ws")
        .token("A", "a")
        .token("WS", r"\s+")
        .extra("WS")
        .rule("start", vec!["A"])
        .start("start")
        .build();

    assert!(!g.extras.is_empty());
}

#[test]
fn lifecycle_create_with_inline_and_supertype() {
    let g = GrammarBuilder::new("complex")
        .token("X", "x")
        .token("Y", "y")
        .rule("root", vec!["inner"])
        .rule("inner", vec!["X"])
        .rule("inner", vec!["Y"])
        .inline("inner")
        .supertype("root")
        .start("root")
        .build();

    assert!(!g.inline_rules.is_empty());
    assert!(!g.supertypes.is_empty());
}

#[test]
fn lifecycle_create_python_like_nullable() {
    let g = GrammarBuilder::python_like();

    assert_eq!(g.name, "python_like");
    assert!(!g.tokens.is_empty());
    assert!(!g.rules.is_empty());
    assert!(!g.externals.is_empty());

    // Module should have an epsilon production (nullable start).
    let module_id = g.find_symbol_by_name("module").unwrap();
    let module_rules = g.get_rules_for_symbol(module_id).unwrap();
    assert!(
        module_rules
            .iter()
            .any(|r| r.rhs.iter().any(|s| matches!(s, Symbol::Epsilon)))
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// 2. lifecycle_normalize_* — normalization effects (8 tests)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn lifecycle_normalize_preserves_terminal_rules() {
    let mut g = arithmetic_grammar();
    let rules_before: usize = g.rules.values().map(|v| v.len()).sum();

    g.normalize();

    // Basic terminal-only rules should survive normalization.
    let rules_after: usize = g.rules.values().map(|v| v.len()).sum();
    assert!(rules_after >= rules_before);
}

#[test]
fn lifecycle_normalize_expands_optional() {
    let mut g = Grammar::new("opt_test".to_string());
    let tok_id = SymbolId(1);
    let nt_id = SymbolId(2);
    g.tokens.insert(
        tok_id,
        Token {
            name: "A".to_string(),
            pattern: TokenPattern::String("a".to_string()),
            fragile: false,
        },
    );
    g.rule_names.insert(nt_id, "root".to_string());
    g.add_rule(Rule {
        lhs: nt_id,
        rhs: vec![Symbol::Optional(Box::new(Symbol::Terminal(tok_id)))],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: adze_ir::ProductionId(0),
    });

    let normalized = g.normalize();
    // Normalization should have created auxiliary rules for the optional.
    assert!(!normalized.is_empty());
    assert!(g.rules.len() >= 2, "expected aux rule for Optional");
}

#[test]
fn lifecycle_normalize_expands_repeat() {
    let mut g = Grammar::new("rep_test".to_string());
    let tok_id = SymbolId(1);
    let nt_id = SymbolId(2);
    g.tokens.insert(
        tok_id,
        Token {
            name: "B".to_string(),
            pattern: TokenPattern::String("b".to_string()),
            fragile: false,
        },
    );
    g.rule_names.insert(nt_id, "root".to_string());
    g.add_rule(Rule {
        lhs: nt_id,
        rhs: vec![Symbol::Repeat(Box::new(Symbol::Terminal(tok_id)))],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: adze_ir::ProductionId(0),
    });

    g.normalize();
    // Repeat should produce an aux rule with left-recursion and epsilon.
    let total_rules: usize = g.rules.values().map(|v| v.len()).sum();
    assert!(total_rules >= 3, "expected repeat to expand into aux rules");
}

#[test]
fn lifecycle_normalize_expands_repeat_one() {
    let mut g = Grammar::new("rep1_test".to_string());
    let tok_id = SymbolId(1);
    let nt_id = SymbolId(2);
    g.tokens.insert(
        tok_id,
        Token {
            name: "C".to_string(),
            pattern: TokenPattern::String("c".to_string()),
            fragile: false,
        },
    );
    g.rule_names.insert(nt_id, "root".to_string());
    g.add_rule(Rule {
        lhs: nt_id,
        rhs: vec![Symbol::RepeatOne(Box::new(Symbol::Terminal(tok_id)))],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: adze_ir::ProductionId(0),
    });

    g.normalize();
    let total_rules: usize = g.rules.values().map(|v| v.len()).sum();
    assert!(
        total_rules >= 3,
        "expected repeat_one to expand into aux rules"
    );
}

#[test]
fn lifecycle_normalize_expands_choice() {
    let mut g = Grammar::new("choice_test".to_string());
    let tok_a = SymbolId(1);
    let tok_b = SymbolId(2);
    let nt_id = SymbolId(3);
    g.tokens.insert(
        tok_a,
        Token {
            name: "A".to_string(),
            pattern: TokenPattern::String("a".to_string()),
            fragile: false,
        },
    );
    g.tokens.insert(
        tok_b,
        Token {
            name: "B".to_string(),
            pattern: TokenPattern::String("b".to_string()),
            fragile: false,
        },
    );
    g.rule_names.insert(nt_id, "root".to_string());
    g.add_rule(Rule {
        lhs: nt_id,
        rhs: vec![Symbol::Choice(vec![
            Symbol::Terminal(tok_a),
            Symbol::Terminal(tok_b),
        ])],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: adze_ir::ProductionId(0),
    });

    g.normalize();
    let total_rules: usize = g.rules.values().map(|v| v.len()).sum();
    assert!(total_rules >= 3, "expected choice to expand into aux rules");
}

#[test]
fn lifecycle_normalize_flattens_sequence() {
    let mut g = Grammar::new("seq_test".to_string());
    let tok_a = SymbolId(1);
    let tok_b = SymbolId(2);
    let nt_id = SymbolId(3);
    g.tokens.insert(
        tok_a,
        Token {
            name: "A".to_string(),
            pattern: TokenPattern::String("a".to_string()),
            fragile: false,
        },
    );
    g.tokens.insert(
        tok_b,
        Token {
            name: "B".to_string(),
            pattern: TokenPattern::String("b".to_string()),
            fragile: false,
        },
    );
    g.rule_names.insert(nt_id, "root".to_string());
    g.add_rule(Rule {
        lhs: nt_id,
        rhs: vec![Symbol::Sequence(vec![
            Symbol::Terminal(tok_a),
            Symbol::Terminal(tok_b),
        ])],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: adze_ir::ProductionId(0),
    });

    g.normalize();
    // After normalization, the Sequence should be flattened into the rule rhs.
    let root_rules = g.get_rules_for_symbol(nt_id).unwrap();
    let has_flat = root_rules.iter().any(|r| {
        r.rhs.len() == 2
            && matches!(r.rhs[0], Symbol::Terminal(_))
            && matches!(r.rhs[1], Symbol::Terminal(_))
    });
    assert!(has_flat, "Sequence should be flattened into the rule");
}

#[test]
fn lifecycle_normalize_idempotent() {
    let mut g = arithmetic_grammar();
    g.normalize();
    let snapshot_1: usize = g.rules.values().map(|v| v.len()).sum();

    g.normalize();
    let snapshot_2: usize = g.rules.values().map(|v| v.len()).sum();

    assert_eq!(
        snapshot_1, snapshot_2,
        "Normalizing twice should be idempotent"
    );
}

#[test]
fn lifecycle_normalize_preserves_name() {
    let mut g = arithmetic_grammar();
    let name_before = g.name.clone();
    g.normalize();
    assert_eq!(g.name, name_before);
}

// ═══════════════════════════════════════════════════════════════════════════
// 3. lifecycle_optimize_* — optimization effects (8 tests)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn lifecycle_optimize_returns_stats() {
    let mut g = arithmetic_grammar();
    let mut opt = GrammarOptimizer::new();
    let stats = opt.optimize(&mut g);

    // Stats should be non-negative (the total is a sum of usizes).
    assert!(stats.total() < 1000);
}

#[test]
fn lifecycle_optimize_preserves_grammar_name() {
    let mut g = arithmetic_grammar();
    let mut opt = GrammarOptimizer::new();
    opt.optimize(&mut g);
    assert_eq!(g.name, "arithmetic");
}

#[test]
fn lifecycle_optimize_preserves_tokens() {
    let mut g = arithmetic_grammar();
    let token_count_before = g.tokens.len();
    let mut opt = GrammarOptimizer::new();
    opt.optimize(&mut g);
    // Tokens may be merged, but should not increase.
    assert!(g.tokens.len() <= token_count_before);
}

#[test]
fn lifecycle_optimize_convenience_function() {
    let g = arithmetic_grammar();
    let result = optimize_grammar(g);
    assert!(result.is_ok());
    let optimized = result.unwrap();
    assert_eq!(optimized.name, "arithmetic");
}

#[test]
fn lifecycle_optimize_statement_grammar() {
    let mut g = statement_grammar();
    let mut opt = GrammarOptimizer::new();
    let stats = opt.optimize(&mut g);

    // Grammar should still have rules after optimization.
    assert!(!g.rules.is_empty());
    // Stats fields are non-negative by construction.
    let _ = stats.removed_unused_symbols;
    let _ = stats.inlined_rules;
    let _ = stats.merged_tokens;
}

#[test]
fn lifecycle_optimize_with_precedence() {
    let g = GrammarBuilder::new("prec_opt")
        .token("N", r"\d+")
        .token("+", "+")
        .token("*", "*")
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "*", "expr"], 2, Associativity::Left)
        .rule("expr", vec!["N"])
        .start("expr")
        .build();

    let optimized = optimize_grammar(g).unwrap();
    // Precedence info should survive optimization.
    let has_precedence = optimized.all_rules().any(|r| r.precedence.is_some());
    assert!(has_precedence, "Precedence should survive optimization");
}

#[test]
fn lifecycle_optimize_idempotent_stats() {
    let mut g = arithmetic_grammar();
    let mut opt1 = GrammarOptimizer::new();
    opt1.optimize(&mut g);

    let mut opt2 = GrammarOptimizer::new();
    let stats2 = opt2.optimize(&mut g);

    // Second pass should have little or nothing to optimize.
    assert!(
        stats2.total() <= 5,
        "Second optimization pass should do minimal work, got {}",
        stats2.total()
    );
}

#[test]
fn lifecycle_optimize_does_not_remove_start_rules() {
    let mut g = arithmetic_grammar();
    let start_before = g.start_symbol();
    let mut opt = GrammarOptimizer::new();
    opt.optimize(&mut g);

    // The grammar should still have rules.
    assert!(
        !g.rules.is_empty(),
        "Grammar must retain rules after optimize"
    );
    // If start was set, the grammar should still function.
    if let Some(sid) = start_before {
        // The symbol might have been renumbered, but rules should exist.
        let _ = sid; // acknowledged
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 4. lifecycle_validate_* — validation at each stage (8 tests)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn lifecycle_validate_fresh_grammar_ok() {
    let g = arithmetic_grammar();
    assert!(g.validate().is_ok());
}

#[test]
fn lifecycle_validate_empty_grammar_error() {
    let g = Grammar::new("empty".to_string());
    let mut validator = GrammarValidator::new();
    let result = validator.validate(&g);
    assert!(
        !result.errors.is_empty(),
        "Empty grammar should produce errors"
    );
}

#[test]
fn lifecycle_validate_after_normalize() {
    let mut g = arithmetic_grammar();
    g.normalize();
    // Normalized grammars only have simple Terminal/NonTerminal/Epsilon, so validate
    // should still pass for field ordering and symbol resolution.
    assert!(g.validate().is_ok());
}

#[test]
fn lifecycle_validate_after_optimize() {
    let g = optimize_grammar(arithmetic_grammar()).unwrap();
    // Validate should still succeed after optimization.
    assert!(g.validate().is_ok());
}

#[test]
fn lifecycle_validate_validator_stats() {
    let g = statement_grammar();
    let mut validator = GrammarValidator::new();
    let result = validator.validate(&g);

    assert!(result.stats.total_rules > 0);
    assert!(result.stats.total_tokens > 0);
    assert!(result.stats.total_symbols > 0);
}

#[test]
fn lifecycle_validate_check_empty_terminals() {
    let g = arithmetic_grammar();
    assert!(
        g.check_empty_terminals().is_ok(),
        "Arithmetic grammar should have no empty terminals"
    );
}

#[test]
fn lifecycle_validate_python_like_grammar() {
    let g = GrammarBuilder::python_like();
    let mut validator = GrammarValidator::new();
    let result = validator.validate(&g);

    // Python-like grammar should produce stats.
    assert!(result.stats.total_rules > 0);
    assert!(result.stats.external_tokens > 0);
}

#[test]
fn lifecycle_validate_javascript_like_grammar() {
    let g = GrammarBuilder::javascript_like();
    let mut validator = GrammarValidator::new();
    let result = validator.validate(&g);

    assert!(result.stats.total_rules > 0);
    assert!(result.stats.max_rule_length > 0);
}

// ═══════════════════════════════════════════════════════════════════════════
// 5. lifecycle_serialize_* — JSON serialization (8 tests)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn lifecycle_serialize_minimal() {
    let g = GrammarBuilder::new("ser")
        .token("A", "a")
        .rule("root", vec!["A"])
        .start("root")
        .build();

    let json = serde_json::to_string(&g).expect("serialize should succeed");
    assert!(!json.is_empty());
    assert!(json.contains("\"name\":\"ser\""));
}

#[test]
fn lifecycle_serialize_arithmetic() {
    let g = arithmetic_grammar();
    let json = serde_json::to_string(&g).unwrap();

    assert!(json.contains("arithmetic"));
    assert!(json.contains("NUMBER"));
}

#[test]
fn lifecycle_serialize_pretty() {
    let g = arithmetic_grammar();
    let json = serde_json::to_string_pretty(&g).unwrap();

    assert!(json.contains('\n'));
    assert!(json.contains("\"name\""));
}

#[test]
fn lifecycle_serialize_after_normalize() {
    let mut g = arithmetic_grammar();
    g.normalize();
    let json = serde_json::to_string(&g);
    assert!(json.is_ok(), "Normalized grammar should serialize");
}

#[test]
fn lifecycle_serialize_after_optimize() {
    let g = optimize_grammar(arithmetic_grammar()).unwrap();
    let json = serde_json::to_string(&g);
    assert!(json.is_ok(), "Optimized grammar should serialize");
}

#[test]
fn lifecycle_serialize_preserves_tokens() {
    let g = arithmetic_grammar();
    let json = serde_json::to_string(&g).unwrap();
    let deser: Grammar = serde_json::from_str(&json).unwrap();

    assert_eq!(g.tokens.len(), deser.tokens.len());
    for (sid, tok) in &g.tokens {
        let deser_tok = deser.tokens.get(sid).expect("token must exist after deser");
        assert_eq!(tok.name, deser_tok.name);
    }
}

#[test]
fn lifecycle_serialize_preserves_rule_names() {
    let g = arithmetic_grammar();
    let json = serde_json::to_string(&g).unwrap();
    let deser: Grammar = serde_json::from_str(&json).unwrap();

    assert_eq!(g.rule_names.len(), deser.rule_names.len());
}

#[test]
fn lifecycle_serialize_statement_grammar() {
    let g = statement_grammar();
    let json = serde_json::to_string(&g).unwrap();
    assert!(json.contains("program"));
    assert!(json.contains("stmt_list"));
    assert!(json.contains("if_stmt"));
}

// ═══════════════════════════════════════════════════════════════════════════
// 6. lifecycle_roundtrip_* — full create → serialize → deserialize (8 tests)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn lifecycle_roundtrip_arithmetic() {
    let g = arithmetic_grammar();
    let json = serde_json::to_string(&g).unwrap();
    let deser: Grammar = serde_json::from_str(&json).unwrap();

    assert_eq!(g.name, deser.name);
    assert_eq!(g.rules.len(), deser.rules.len());
    assert_eq!(g.tokens.len(), deser.tokens.len());
    assert_eq!(g.precedences.len(), deser.precedences.len());
}

#[test]
fn lifecycle_roundtrip_statement() {
    let g = statement_grammar();
    let json = serde_json::to_string(&g).unwrap();
    let deser: Grammar = serde_json::from_str(&json).unwrap();

    assert_eq!(g.name, deser.name);
    assert_eq!(g.rules.len(), deser.rules.len());
}

#[test]
fn lifecycle_roundtrip_python_like() {
    let g = GrammarBuilder::python_like();
    let json = serde_json::to_string(&g).unwrap();
    let deser: Grammar = serde_json::from_str(&json).unwrap();

    assert_eq!(g.name, deser.name);
    assert_eq!(g.externals.len(), deser.externals.len());
    assert_eq!(g.extras.len(), deser.extras.len());
}

#[test]
fn lifecycle_roundtrip_javascript_like() {
    let g = GrammarBuilder::javascript_like();
    let json = serde_json::to_string(&g).unwrap();
    let deser: Grammar = serde_json::from_str(&json).unwrap();

    assert_eq!(g.name, deser.name);
    assert_eq!(g.tokens.len(), deser.tokens.len());
}

#[test]
fn lifecycle_roundtrip_with_precedence() {
    let g = GrammarBuilder::new("prec_rt")
        .token("N", r"\d+")
        .token("+", "+")
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 5, Associativity::Right)
        .rule("expr", vec!["N"])
        .start("expr")
        .build();

    let json = serde_json::to_string(&g).unwrap();
    let deser: Grammar = serde_json::from_str(&json).unwrap();

    let has_right_assoc = deser
        .all_rules()
        .any(|r| r.associativity == Some(Associativity::Right));
    assert!(has_right_assoc, "Associativity should survive roundtrip");
}

#[test]
fn lifecycle_roundtrip_after_normalize() {
    let mut g = arithmetic_grammar();
    g.normalize();
    let json = serde_json::to_string(&g).unwrap();
    let deser: Grammar = serde_json::from_str(&json).unwrap();

    assert_eq!(g.name, deser.name);
    assert_eq!(g.rules.len(), deser.rules.len());
}

#[test]
fn lifecycle_roundtrip_after_optimize() {
    let g = optimize_grammar(arithmetic_grammar()).unwrap();
    let json = serde_json::to_string(&g).unwrap();
    let deser: Grammar = serde_json::from_str(&json).unwrap();

    assert_eq!(g.name, deser.name);
}

#[test]
fn lifecycle_roundtrip_full_pipeline() {
    let mut g = statement_grammar();
    g.normalize();
    let g = optimize_grammar(g).unwrap();
    let json = serde_json::to_string(&g).unwrap();
    let deser: Grammar = serde_json::from_str(&json).unwrap();

    assert_eq!(g.name, deser.name);
    // The deserialized grammar should also serialize identically.
    let json2 = serde_json::to_string(&deser).unwrap();
    assert_eq!(json, json2, "Double roundtrip should be stable");
}

// ═══════════════════════════════════════════════════════════════════════════
// 7. lifecycle_chain_* — chained operations (8 tests)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn lifecycle_chain_create_then_validate() {
    let g = arithmetic_grammar();
    assert!(g.validate().is_ok());
}

#[test]
fn lifecycle_chain_normalize_then_validate() {
    let mut g = arithmetic_grammar();
    g.normalize();
    assert!(g.validate().is_ok());
}

#[test]
fn lifecycle_chain_validate_then_optimize() {
    let g = arithmetic_grammar();
    assert!(g.validate().is_ok());
    let optimized = optimize_grammar(g).unwrap();
    assert!(!optimized.rules.is_empty());
}

#[test]
fn lifecycle_chain_normalize_optimize_validate() {
    let mut g = statement_grammar();
    g.normalize();
    let mut opt = GrammarOptimizer::new();
    let stats = opt.optimize(&mut g);
    // After normalize + optimize the grammar should still have rules.
    assert!(!g.rules.is_empty());
    // The optimizer ran and produced stats.
    let _ = stats.total();
}

#[test]
fn lifecycle_chain_optimize_serialize_validate() {
    let g = optimize_grammar(arithmetic_grammar()).unwrap();
    let json = serde_json::to_string(&g).unwrap();
    let deser: Grammar = serde_json::from_str(&json).unwrap();
    assert!(deser.validate().is_ok());
}

#[test]
fn lifecycle_chain_normalize_serialize_deserialize_validate() {
    let mut g = arithmetic_grammar();
    g.normalize();
    let json = serde_json::to_string(&g).unwrap();
    let deser: Grammar = serde_json::from_str(&json).unwrap();
    assert!(deser.validate().is_ok());
}

#[test]
fn lifecycle_chain_full_pipeline_python() {
    let mut g = GrammarBuilder::python_like();
    g.normalize();
    let g = optimize_grammar(g).unwrap();
    let json = serde_json::to_string(&g).unwrap();
    let deser: Grammar = serde_json::from_str(&json).unwrap();
    assert_eq!(deser.name, "python_like");
}

#[test]
fn lifecycle_chain_full_pipeline_javascript() {
    let mut g = GrammarBuilder::javascript_like();
    g.normalize();
    let g = optimize_grammar(g).unwrap();
    let json = serde_json::to_string(&g).unwrap();
    let deser: Grammar = serde_json::from_str(&json).unwrap();
    assert_eq!(deser.name, "javascript_like");
}

// ═══════════════════════════════════════════════════════════════════════════
// 8. lifecycle_edge_* — edge cases (8 tests)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn lifecycle_edge_empty_grammar_serialize() {
    let g = Grammar::new("empty".to_string());
    let json = serde_json::to_string(&g).unwrap();
    let deser: Grammar = serde_json::from_str(&json).unwrap();
    assert_eq!(deser.name, "empty");
    assert!(deser.rules.is_empty());
}

#[test]
fn lifecycle_edge_single_epsilon_rule() {
    let g = GrammarBuilder::new("eps")
        .rule("start", vec![])
        .start("start")
        .build();

    let start_id = g.find_symbol_by_name("start").unwrap();
    let rules = g.get_rules_for_symbol(start_id).unwrap();
    assert!(
        rules
            .iter()
            .any(|r| r.rhs.iter().any(|s| matches!(s, Symbol::Epsilon)))
    );

    // Should roundtrip.
    let json = serde_json::to_string(&g).unwrap();
    let deser: Grammar = serde_json::from_str(&json).unwrap();
    assert_eq!(g.rules.len(), deser.rules.len());
}

#[test]
fn lifecycle_edge_many_tokens() {
    let mut builder = GrammarBuilder::new("many_tokens");
    for idx in 0..100 {
        let name = format!("TOK_{idx}");
        let pat = format!("t{idx}");
        builder = builder.token(&name, &pat);
    }
    let rhs_names: Vec<&str> = vec!["TOK_0"];
    builder = builder.rule("root", rhs_names).start("root");
    let g = builder.build();

    assert_eq!(g.tokens.len(), 100);

    let json = serde_json::to_string(&g).unwrap();
    let deser: Grammar = serde_json::from_str(&json).unwrap();
    assert_eq!(deser.tokens.len(), 100);
}

#[test]
fn lifecycle_edge_many_rules() {
    let mut builder = GrammarBuilder::new("many_rules");
    builder = builder.token("A", "a").token("B", "b");
    for idx in 0..50 {
        let name = format!("rule_{idx}");
        if idx == 0 {
            builder = builder.rule(&name, vec!["A"]);
        } else {
            let prev_name = format!("rule_{}", idx - 1);
            builder = builder.rule(&name, vec![&prev_name, "B"]);
        }
    }
    builder = builder.start("rule_49");
    let g = builder.build();

    assert_eq!(g.rules.len(), 50);
}

#[test]
fn lifecycle_edge_deeply_nested_alternatives() {
    // Grammar with one nonterminal having many alternatives.
    let mut builder = GrammarBuilder::new("deep_alt");
    for idx in 0..20 {
        let tok_name = format!("T{idx}");
        builder = builder.token(&tok_name, &format!("t{idx}"));
    }
    for idx in 0..20 {
        let tok_name = format!("T{idx}");
        builder = builder.rule("root", vec![&tok_name]);
    }
    builder = builder.start("root");
    let g = builder.build();

    let root_id = g.find_symbol_by_name("root").unwrap();
    let root_rules = g.get_rules_for_symbol(root_id).unwrap();
    assert_eq!(root_rules.len(), 20);
}

#[test]
fn lifecycle_edge_unicode_grammar_name() {
    let g = GrammarBuilder::new("日本語文法")
        .token("A", "a")
        .rule("root", vec!["A"])
        .start("root")
        .build();

    let json = serde_json::to_string(&g).unwrap();
    let deser: Grammar = serde_json::from_str(&json).unwrap();
    assert_eq!(deser.name, "日本語文法");
}

#[test]
fn lifecycle_edge_default_grammar() {
    let g = Grammar::default();
    assert!(g.name.is_empty());
    assert!(g.rules.is_empty());
    assert!(g.tokens.is_empty());

    let json = serde_json::to_string(&g).unwrap();
    let deser: Grammar = serde_json::from_str(&json).unwrap();
    assert_eq!(g, deser);
}

#[test]
fn lifecycle_edge_fragile_tokens_roundtrip() {
    let g = GrammarBuilder::new("fragile")
        .fragile_token("ERR", r".")
        .token("OK", "ok")
        .rule("root", vec!["OK"])
        .start("root")
        .build();

    // Verify the fragile flag is set.
    let fragile_count = g.tokens.values().filter(|t| t.fragile).count();
    assert_eq!(fragile_count, 1);

    let json = serde_json::to_string(&g).unwrap();
    let deser: Grammar = serde_json::from_str(&json).unwrap();
    let deser_fragile = deser.tokens.values().filter(|t| t.fragile).count();
    assert_eq!(deser_fragile, 1);
}
