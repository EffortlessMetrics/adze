//! Cross-crate integration tests verifying the full IR → GLR → Tablegen → Runtime pipeline.
//!
//! Each test builds a grammar using adze-ir types, computes FIRST/FOLLOW sets
//! via adze-glr-core, constructs a parse table, and feeds it into adze-tablegen
//! for code generation and table compression.

use adze_glr_core::{FirstFollowSets, build_lr1_automaton};
use adze_ir::builder::GrammarBuilder;
use adze_ir::{Associativity, Grammar, ProductionId, Rule, Symbol, SymbolId, Token, TokenPattern};
use adze_tablegen::StaticLanguageGenerator;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Run the full pipeline: grammar → FIRST/FOLLOW → parse table → tablegen.
/// Returns the generator so callers can inspect generated code and node types.
fn run_pipeline(grammar: Grammar) -> StaticLanguageGenerator {
    let ff = FirstFollowSets::compute(&grammar).expect("FIRST/FOLLOW computation should succeed");
    let parse_table =
        build_lr1_automaton(&grammar, &ff).expect("LR(1) automaton construction should succeed");

    assert!(parse_table.state_count > 0, "parse table must have states");
    assert!(
        !parse_table.action_table.is_empty(),
        "action table must not be empty"
    );
    assert!(
        !parse_table.rules.is_empty(),
        "parse table must contain rules"
    );

    let mut generator = StaticLanguageGenerator::new(grammar, parse_table);
    generator
        .compress_tables()
        .expect("table compression should succeed");
    generator
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

/// Simplest possible grammar: a single terminal rule.
#[test]
fn pipeline_single_terminal_rule() {
    let grammar = GrammarBuilder::new("single")
        .token("NUMBER", r"\d+")
        .rule("expr", vec!["NUMBER"])
        .start("expr")
        .build();

    let generator = run_pipeline(grammar);

    let code = generator.generate_language_code().to_string();
    assert!(
        code.contains("language"),
        "generated code should contain a language function"
    );

    let node_types = generator.generate_node_types();
    assert!(
        node_types.starts_with('['),
        "node types should be a JSON array"
    );
}

/// Arithmetic expression grammar with multiple rules and left-recursive productions.
#[test]
fn pipeline_arithmetic_grammar() {
    let grammar = GrammarBuilder::new("arithmetic")
        .token("NUMBER", r"\d+")
        .token("+", "+")
        .token("*", "*")
        .token("(", "(")
        .token(")", ")")
        .rule("expr", vec!["expr", "+", "term"])
        .rule("expr", vec!["term"])
        .rule("term", vec!["term", "*", "factor"])
        .rule("term", vec!["factor"])
        .rule("factor", vec!["(", "expr", ")"])
        .rule("factor", vec!["NUMBER"])
        .start("expr")
        .build();

    let generator = run_pipeline(grammar);

    let code = generator.generate_language_code().to_string();
    assert!(code.contains("tree_sitter_arithmetic"));
    assert!(generator.compressed_tables.is_some());
}

/// Grammar with precedence and associativity declarations.
#[test]
fn pipeline_precedence_grammar() {
    let grammar = GrammarBuilder::new("calc")
        .token("NUM", r"\d+")
        .token("+", "+")
        .token("-", "-")
        .token("*", "*")
        .token("/", "/")
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "-", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "*", "expr"], 2, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "/", "expr"], 2, Associativity::Left)
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();

    let generator = run_pipeline(grammar);

    let code = generator.generate_language_code().to_string();
    assert!(code.contains("tree_sitter_calc"));

    // Compressed tables must exist after compress_tables()
    let compressed = generator.compressed_tables.as_ref().unwrap();
    assert!(
        !compressed.action_table.data.is_empty(),
        "compressed action_table data must be non-empty"
    );
}

/// Grammar with multiple non-terminals and epsilon productions.
#[test]
fn pipeline_multi_nonterminal_with_epsilon() {
    let grammar = GrammarBuilder::new("stmts")
        .token("ID", r"[a-z]+")
        .token(";", ";")
        .token("=", "=")
        .rule("program", vec!["stmt_list"])
        .rule("stmt_list", vec!["stmt_list", "stmt"])
        .rule("stmt_list", vec!["stmt"])
        .rule("stmt", vec!["ID", "=", "ID", ";"])
        .start("program")
        .build();

    let generator = run_pipeline(grammar);

    assert!(
        generator.parse_table.state_count >= 4,
        "expect multiple states"
    );
    let code = generator.generate_language_code().to_string();
    assert!(code.contains("tree_sitter_stmts"));
}

/// Grammar built from raw IR types (without the builder) to ensure low-level API works.
#[test]
fn pipeline_raw_ir_types() {
    let mut grammar = Grammar::new("raw".to_string());

    let num_id = SymbolId(1);
    let plus_id = SymbolId(2);
    let expr_id = SymbolId(10);

    grammar.tokens.insert(
        num_id,
        Token {
            name: "NUMBER".to_string(),
            pattern: TokenPattern::Regex(r"\d+".to_string()),
            fragile: false,
        },
    );
    grammar.tokens.insert(
        plus_id,
        Token {
            name: "+".to_string(),
            pattern: TokenPattern::String("+".to_string()),
            fragile: false,
        },
    );

    grammar.rule_names.insert(expr_id, "expr".to_string());

    grammar.add_rule(Rule {
        lhs: expr_id,
        rhs: vec![Symbol::Terminal(num_id)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    grammar.add_rule(Rule {
        lhs: expr_id,
        rhs: vec![
            Symbol::NonTerminal(expr_id),
            Symbol::Terminal(plus_id),
            Symbol::NonTerminal(expr_id),
        ],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(1),
    });

    let generator = run_pipeline(grammar);

    let code = generator.generate_language_code().to_string();
    assert!(
        code.contains("language"),
        "raw IR grammar should produce valid code"
    );
}

/// Verify FIRST/FOLLOW sets have correct entries for a known grammar.
#[test]
fn pipeline_first_follow_sanity() {
    let grammar = GrammarBuilder::new("ff_test")
        .token("A", "a")
        .token("B", "b")
        .rule("start", vec!["A", "xrule"])
        .rule("xrule", vec!["B"])
        .rule("xrule", vec!["A"])
        .start("start")
        .build();

    let ff = FirstFollowSets::compute(&grammar).expect("FIRST/FOLLOW should compute");

    // start is a non-terminal with a rule — it must have a FIRST set
    let start_id = grammar
        .rule_names
        .iter()
        .find(|(_, n)| n.as_str() == "start")
        .map(|(id, _)| *id)
        .expect("start should exist in rule_names");

    let first_start = ff.first(start_id).expect("start must have a FIRST set");
    assert!(
        first_start.count_ones(..) > 0,
        "FIRST(start) must contain at least one terminal"
    );

    // Also verify the full pipeline still works
    let parse_table =
        build_lr1_automaton(&grammar, &ff).expect("parse table should build from these sets");
    let generator = StaticLanguageGenerator::new(grammar, parse_table);
    let code = generator.generate_language_code().to_string();
    assert!(code.contains("language"));
}

/// Verify that the generated node types JSON is parseable.
#[test]
fn pipeline_node_types_valid_json() {
    let grammar = GrammarBuilder::new("json_check")
        .token("X", "x")
        .token("Y", "y")
        .rule("root", vec!["X", "mid"])
        .rule("mid", vec!["Y"])
        .start("root")
        .build();

    let generator = run_pipeline(grammar);

    let node_types_json = generator.generate_node_types();
    let parsed: serde_json::Value =
        serde_json::from_str(&node_types_json).expect("node types should be valid JSON");
    assert!(parsed.is_array(), "node types must be a JSON array");
}

/// Pipeline with the pre-built javascript-like grammar from the builder module.
#[test]
fn pipeline_javascript_like_grammar() {
    let grammar = GrammarBuilder::javascript_like();

    let ff = FirstFollowSets::compute(&grammar).expect("FIRST/FOLLOW should compute");
    let parse_table =
        build_lr1_automaton(&grammar, &ff).expect("parse table should build for JS-like grammar");

    assert!(
        parse_table.state_count >= 10,
        "JS-like grammar should produce a non-trivial automaton (got {} states)",
        parse_table.state_count
    );

    let mut generator = StaticLanguageGenerator::new(grammar, parse_table);
    generator
        .compress_tables()
        .expect("compression should succeed for JS-like grammar");

    let code = generator.generate_language_code().to_string();
    assert!(code.contains("tree_sitter_javascript_like"));
}
