//! Insta snapshot tests for IR normalization, validation, optimization,
//! and symbol ID allocation.

use adze_ir::builder::GrammarBuilder;
use adze_ir::optimizer::GrammarOptimizer;
use adze_ir::validation::GrammarValidator;
use adze_ir::*;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Render rules in a deterministic, human-readable form suitable for snapshots.
fn render_rules(grammar: &Grammar) -> String {
    let mut lines = Vec::new();
    for (lhs, rules) in &grammar.rules {
        for rule in rules {
            let rhs: Vec<String> = rule.rhs.iter().map(|s| format!("{s:?}")).collect();
            lines.push(format!("  {lhs} -> {}", rhs.join(" ")));
        }
    }
    lines.sort();
    lines.join("\n")
}

/// Render validation errors as strings.
fn render_errors(errors: &[adze_ir::validation::ValidationError]) -> String {
    let mut msgs: Vec<String> = errors.iter().map(|e| e.to_string()).collect();
    msgs.sort();
    msgs.join("\n")
}

/// Render validation warnings as strings.
fn render_warnings(warnings: &[adze_ir::validation::ValidationWarning]) -> String {
    let mut msgs: Vec<String> = warnings.iter().map(|w| w.to_string()).collect();
    msgs.sort();
    msgs.join("\n")
}

/// Collect all symbol IDs that appear as rule LHS after normalization.
fn collect_lhs_ids(grammar: &Grammar) -> Vec<u16> {
    let mut ids: Vec<u16> = grammar.rules.keys().map(|id| id.0).collect();
    ids.sort();
    ids
}

// ===========================================================================
// 1. Normalization snapshots
// ===========================================================================

#[test]
fn normalize_simple_grammar() {
    let mut grammar = Grammar::new("simple".into());
    // expr -> NUMBER
    grammar.tokens.insert(
        SymbolId(1),
        Token {
            name: "NUMBER".into(),
            pattern: TokenPattern::Regex(r"\d+".into()),
            fragile: false,
        },
    );
    grammar.add_rule(Rule {
        lhs: SymbolId(10),
        rhs: vec![Symbol::Terminal(SymbolId(1))],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    grammar.rule_names.insert(SymbolId(10), "expr".into());

    grammar.normalize();
    insta::assert_snapshot!("normalize_simple", render_rules(&grammar));
}

#[test]
fn normalize_optional() {
    let mut grammar = Grammar::new("optional".into());
    grammar.tokens.insert(
        SymbolId(1),
        Token {
            name: "A".into(),
            pattern: TokenPattern::String("a".into()),
            fragile: false,
        },
    );
    grammar.tokens.insert(
        SymbolId(2),
        Token {
            name: "B".into(),
            pattern: TokenPattern::String("b".into()),
            fragile: false,
        },
    );
    // S -> A B?
    grammar.add_rule(Rule {
        lhs: SymbolId(10),
        rhs: vec![
            Symbol::Terminal(SymbolId(1)),
            Symbol::Optional(Box::new(Symbol::Terminal(SymbolId(2)))),
        ],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    grammar.rule_names.insert(SymbolId(10), "S".into());

    grammar.normalize();
    insta::assert_snapshot!("normalize_optional", render_rules(&grammar));
}

#[test]
fn normalize_repeat() {
    let mut grammar = Grammar::new("repeat".into());
    grammar.tokens.insert(
        SymbolId(1),
        Token {
            name: "X".into(),
            pattern: TokenPattern::String("x".into()),
            fragile: false,
        },
    );
    // S -> X*
    grammar.add_rule(Rule {
        lhs: SymbolId(10),
        rhs: vec![Symbol::Repeat(Box::new(Symbol::Terminal(SymbolId(1))))],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    grammar.rule_names.insert(SymbolId(10), "S".into());

    grammar.normalize();
    insta::assert_snapshot!("normalize_repeat", render_rules(&grammar));
}

#[test]
fn normalize_repeat_one() {
    let mut grammar = Grammar::new("repeat_one".into());
    grammar.tokens.insert(
        SymbolId(1),
        Token {
            name: "Y".into(),
            pattern: TokenPattern::String("y".into()),
            fragile: false,
        },
    );
    // S -> Y+
    grammar.add_rule(Rule {
        lhs: SymbolId(10),
        rhs: vec![Symbol::RepeatOne(Box::new(Symbol::Terminal(SymbolId(1))))],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    grammar.rule_names.insert(SymbolId(10), "S".into());

    grammar.normalize();
    insta::assert_snapshot!("normalize_repeat_one", render_rules(&grammar));
}

#[test]
fn normalize_choice() {
    let mut grammar = Grammar::new("choice".into());
    grammar.tokens.insert(
        SymbolId(1),
        Token {
            name: "A".into(),
            pattern: TokenPattern::String("a".into()),
            fragile: false,
        },
    );
    grammar.tokens.insert(
        SymbolId(2),
        Token {
            name: "B".into(),
            pattern: TokenPattern::String("b".into()),
            fragile: false,
        },
    );
    // S -> (A | B)
    grammar.add_rule(Rule {
        lhs: SymbolId(10),
        rhs: vec![Symbol::Choice(vec![
            Symbol::Terminal(SymbolId(1)),
            Symbol::Terminal(SymbolId(2)),
        ])],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    grammar.rule_names.insert(SymbolId(10), "S".into());

    grammar.normalize();
    insta::assert_snapshot!("normalize_choice", render_rules(&grammar));
}

#[test]
fn normalize_sequence() {
    let mut grammar = Grammar::new("sequence".into());
    grammar.tokens.insert(
        SymbolId(1),
        Token {
            name: "A".into(),
            pattern: TokenPattern::String("a".into()),
            fragile: false,
        },
    );
    grammar.tokens.insert(
        SymbolId(2),
        Token {
            name: "B".into(),
            pattern: TokenPattern::String("b".into()),
            fragile: false,
        },
    );
    grammar.tokens.insert(
        SymbolId(3),
        Token {
            name: "C".into(),
            pattern: TokenPattern::String("c".into()),
            fragile: false,
        },
    );
    // S -> seq(A, B, C)
    grammar.add_rule(Rule {
        lhs: SymbolId(10),
        rhs: vec![Symbol::Sequence(vec![
            Symbol::Terminal(SymbolId(1)),
            Symbol::Terminal(SymbolId(2)),
            Symbol::Terminal(SymbolId(3)),
        ])],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    grammar.rule_names.insert(SymbolId(10), "S".into());

    grammar.normalize();
    insta::assert_snapshot!("normalize_sequence", render_rules(&grammar));
}

#[test]
fn normalize_nested_optional_repeat() {
    let mut grammar = Grammar::new("nested".into());
    grammar.tokens.insert(
        SymbolId(1),
        Token {
            name: "T".into(),
            pattern: TokenPattern::String("t".into()),
            fragile: false,
        },
    );
    // S -> (T*)?   — optional repeat
    grammar.add_rule(Rule {
        lhs: SymbolId(10),
        rhs: vec![Symbol::Optional(Box::new(Symbol::Repeat(Box::new(
            Symbol::Terminal(SymbolId(1)),
        ))))],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    grammar.rule_names.insert(SymbolId(10), "S".into());

    grammar.normalize();
    insta::assert_snapshot!("normalize_nested_optional_repeat", render_rules(&grammar));
}

// ===========================================================================
// 2. Validation error / warning snapshots
// ===========================================================================

#[test]
fn validate_empty_grammar() {
    let grammar = Grammar::new("empty".into());
    let mut validator = GrammarValidator::new();
    let result = validator.validate(&grammar);
    insta::assert_snapshot!(
        "validate_empty_grammar_errors",
        render_errors(&result.errors)
    );
}

#[test]
fn validate_undefined_symbol() {
    let mut grammar = Grammar::new("undef".into());
    // Rule references SymbolId(99) which has no token or rule entry
    grammar.add_rule(Rule {
        lhs: SymbolId(1),
        rhs: vec![Symbol::NonTerminal(SymbolId(99))],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    grammar.rule_names.insert(SymbolId(1), "start".into());

    let mut validator = GrammarValidator::new();
    let result = validator.validate(&grammar);
    insta::assert_snapshot!(
        "validate_undefined_symbol_errors",
        render_errors(&result.errors)
    );
}

#[test]
fn validate_well_formed_grammar() {
    let grammar = GrammarBuilder::new("good")
        .token("NUMBER", r"\d+")
        .token("+", "+")
        .rule("expr", vec!["expr", "+", "expr"])
        .rule("expr", vec!["NUMBER"])
        .start("expr")
        .build();

    let mut validator = GrammarValidator::new();
    let result = validator.validate(&grammar);
    insta::assert_snapshot!(
        "validate_well_formed_warnings",
        render_warnings(&result.warnings)
    );
}

// ===========================================================================
// 3. Optimizer before / after snapshots
// ===========================================================================

#[test]
fn optimizer_simple_grammar() {
    let mut grammar = GrammarBuilder::new("opt_test")
        .token("NUMBER", r"\d+")
        .token("+", "+")
        .token("*", "*")
        .rule("source_file", vec!["expr"])
        .rule("expr", vec!["expr", "+", "expr"])
        .rule("expr", vec!["expr", "*", "expr"])
        .rule("expr", vec!["NUMBER"])
        .start("source_file")
        .build();

    let before = render_rules(&grammar);

    let mut optimizer = GrammarOptimizer::new();
    let stats = optimizer.optimize(&mut grammar);

    let after = render_rules(&grammar);

    insta::assert_snapshot!("optimizer_before", before);
    insta::assert_snapshot!("optimizer_after", after);
    insta::assert_snapshot!(
        "optimizer_stats",
        format!(
            "removed_unused={} inlined={} merged_tokens={} opt_left_rec={} elim_unit={}",
            stats.removed_unused_symbols,
            stats.inlined_rules,
            stats.merged_tokens,
            stats.optimized_left_recursion,
            stats.eliminated_unit_rules,
        )
    );
}

// ===========================================================================
// 4. Symbol ID allocation after normalization
// ===========================================================================

#[test]
fn symbol_ids_after_normalize_simple() {
    let mut grammar = Grammar::new("ids_simple".into());
    grammar.tokens.insert(
        SymbolId(1),
        Token {
            name: "A".into(),
            pattern: TokenPattern::String("a".into()),
            fragile: false,
        },
    );
    grammar.add_rule(Rule {
        lhs: SymbolId(5),
        rhs: vec![Symbol::Terminal(SymbolId(1))],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    grammar.rule_names.insert(SymbolId(5), "S".into());

    grammar.normalize();
    insta::assert_snapshot!(
        "symbol_ids_simple",
        format!("{:?}", collect_lhs_ids(&grammar))
    );
}

#[test]
fn symbol_ids_after_normalize_complex() {
    let mut grammar = Grammar::new("ids_complex".into());
    grammar.tokens.insert(
        SymbolId(1),
        Token {
            name: "A".into(),
            pattern: TokenPattern::String("a".into()),
            fragile: false,
        },
    );
    grammar.tokens.insert(
        SymbolId(2),
        Token {
            name: "B".into(),
            pattern: TokenPattern::String("b".into()),
            fragile: false,
        },
    );
    // S -> A? B*
    grammar.add_rule(Rule {
        lhs: SymbolId(10),
        rhs: vec![
            Symbol::Optional(Box::new(Symbol::Terminal(SymbolId(1)))),
            Symbol::Repeat(Box::new(Symbol::Terminal(SymbolId(2)))),
        ],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    grammar.rule_names.insert(SymbolId(10), "S".into());

    grammar.normalize();
    let ids = collect_lhs_ids(&grammar);
    // Auxiliary IDs should start at max_existing + 1000
    insta::assert_snapshot!("symbol_ids_complex", format!("{ids:?}"));
}

#[test]
fn symbol_ids_gap_preserved() {
    let mut grammar = Grammar::new("ids_gap".into());
    grammar.tokens.insert(
        SymbolId(1),
        Token {
            name: "X".into(),
            pattern: TokenPattern::String("x".into()),
            fragile: false,
        },
    );
    grammar.tokens.insert(
        SymbolId(2),
        Token {
            name: "Y".into(),
            pattern: TokenPattern::String("y".into()),
            fragile: false,
        },
    );
    grammar.tokens.insert(
        SymbolId(3),
        Token {
            name: "Z".into(),
            pattern: TokenPattern::String("z".into()),
            fragile: false,
        },
    );
    // S -> (X | Y) Z+
    grammar.add_rule(Rule {
        lhs: SymbolId(20),
        rhs: vec![
            Symbol::Choice(vec![
                Symbol::Terminal(SymbolId(1)),
                Symbol::Terminal(SymbolId(2)),
            ]),
            Symbol::RepeatOne(Box::new(Symbol::Terminal(SymbolId(3)))),
        ],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    grammar.rule_names.insert(SymbolId(20), "S".into());

    grammar.normalize();
    let ids = collect_lhs_ids(&grammar);
    // Aux IDs start at 20 + 1000 = 1020
    insta::assert_snapshot!("symbol_ids_gap", format!("{ids:?}"));
}

// ===========================================================================
// 5. JSON serialization round-trip snapshot
// ===========================================================================

#[test]
fn grammar_json_roundtrip() {
    let grammar = GrammarBuilder::new("json_test")
        .token("NUMBER", r"\d+")
        .token("+", "+")
        .rule("expr", vec!["expr", "+", "expr"])
        .rule("expr", vec!["NUMBER"])
        .start("expr")
        .build();

    let json = serde_json::to_string_pretty(&grammar).expect("serialize");
    let roundtrip: Grammar = serde_json::from_str(&json).expect("deserialize");
    let json2 = serde_json::to_string_pretty(&roundtrip).expect("re-serialize");
    assert_eq!(json, json2, "JSON round-trip should be identical");
    insta::assert_snapshot!("grammar_json_roundtrip", json);
}

// ===========================================================================
// 6. Grammar Debug output snapshots
// ===========================================================================

#[test]
fn grammar_debug_simple() {
    let grammar = GrammarBuilder::new("debug_simple")
        .token("NUMBER", r"\d+")
        .token("+", "+")
        .rule("expr", vec!["expr", "+", "expr"])
        .rule("expr", vec!["NUMBER"])
        .start("expr")
        .build();

    insta::assert_debug_snapshot!("grammar_debug_simple", grammar);
}

#[test]
fn grammar_debug_with_precedence() {
    let grammar = GrammarBuilder::new("debug_prec")
        .token("NUMBER", r"\d+")
        .token("+", "+")
        .token("*", "*")
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "*", "expr"], 2, Associativity::Left)
        .rule("expr", vec!["NUMBER"])
        .start("expr")
        .build();

    insta::assert_debug_snapshot!("grammar_debug_with_precedence", grammar);
}

#[test]
fn grammar_debug_empty() {
    let grammar = Grammar::new("empty_debug".into());
    insta::assert_debug_snapshot!("grammar_debug_empty", grammar);
}

#[test]
fn grammar_debug_with_extras() {
    let grammar = GrammarBuilder::new("debug_extras")
        .token("NUM", r"\d+")
        .token("WS", r"\s+")
        .extra("WS")
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();

    insta::assert_debug_snapshot!("grammar_debug_with_extras", grammar);
}
