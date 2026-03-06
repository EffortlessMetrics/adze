//! Converter v4: comprehensive tests for the adze-tool grammar conversion pipeline.
//!
//! 57 tests covering:
//! 1. GrammarBuilder → build_parser round-trip (10 tests)
//! 2. JSON → build_parser_from_json pipeline (10 tests)
//! 3. BuildResult / BuildStats validation (8 tests)
//! 4. GrammarConverter sample grammar (5 tests)
//! 5. GrammarVisualizer output (6 tests)
//! 6. Precedence and associativity (6 tests)
//! 7. Extras, externals, inline, supertype (6 tests)
//! 8. Error and edge cases (6 tests)

use adze_ir::Associativity;
use adze_ir::Grammar;
use adze_ir::builder::GrammarBuilder;
use adze_tool::GrammarConverter;
use adze_tool::pure_rust_builder::{
    BuildOptions, BuildResult, build_parser, build_parser_from_json,
};
use adze_tool::visualization::GrammarVisualizer;
use serde_json::json;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn opts() -> BuildOptions {
    BuildOptions {
        out_dir: "/tmp/adze-converter-v4".to_string(),
        emit_artifacts: false,
        compress_tables: false,
    }
}

fn build_json(value: &serde_json::Value) -> anyhow::Result<BuildResult> {
    build_parser_from_json(serde_json::to_string(value).unwrap(), opts())
}

fn simple_expr_grammar() -> Grammar {
    GrammarBuilder::new("simple_expr")
        .token("NUMBER", r"\d+")
        .token("+", "+")
        .rule("expr", vec!["expr", "+", "expr"])
        .rule("expr", vec!["NUMBER"])
        .start("expr")
        .build()
}

fn minimal_json(name: &str, rule_type: &str, rule_value: &str) -> serde_json::Value {
    json!({
        "name": name,
        "rules": {
            "start": { "type": rule_type, "value": rule_value }
        }
    })
}

fn two_rule_json(name: &str) -> serde_json::Value {
    json!({
        "name": name,
        "rules": {
            "program": { "type": "SYMBOL", "name": "item" },
            "item": { "type": "PATTERN", "value": "[a-z]+" }
        }
    })
}

// ===========================================================================
// 1. GrammarBuilder → build_parser round-trip (10 tests)
// ===========================================================================

#[test]
fn builder_simple_expr_builds_ok() {
    let grammar = simple_expr_grammar();
    assert!(build_parser(grammar, opts()).is_ok());
}

#[test]
fn builder_preserves_grammar_name() {
    let grammar = simple_expr_grammar();
    let result = build_parser(grammar, opts()).unwrap();
    assert_eq!(result.grammar_name, "simple_expr");
}

#[test]
fn builder_produces_nonempty_parser_code() {
    let grammar = simple_expr_grammar();
    let result = build_parser(grammar, opts()).unwrap();
    assert!(!result.parser_code.is_empty());
}

#[test]
fn builder_produces_valid_node_types_json() {
    let grammar = simple_expr_grammar();
    let result = build_parser(grammar, opts()).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&result.node_types_json).unwrap();
    assert!(parsed.is_array());
}

#[test]
fn builder_single_token_grammar() {
    let grammar = GrammarBuilder::new("single_tok")
        .token("WORD", r"[a-z]+")
        .rule("root", vec!["WORD"])
        .start("root")
        .build();
    let result = build_parser(grammar, opts()).unwrap();
    assert_eq!(result.grammar_name, "single_tok");
    assert!(result.build_stats.state_count > 0);
}

#[test]
fn builder_two_alternatives() {
    let grammar = GrammarBuilder::new("two_alt")
        .token("A", "a")
        .token("B", "b")
        .rule("root", vec!["A"])
        .rule("root", vec!["B"])
        .start("root")
        .build();
    assert!(build_parser(grammar, opts()).is_ok());
}

#[test]
fn builder_chained_rules() {
    let grammar = GrammarBuilder::new("chain")
        .token("X", "x")
        .rule("top", vec!["mid"])
        .rule("mid", vec!["X"])
        .start("top")
        .build();
    let result = build_parser(grammar, opts()).unwrap();
    assert!(!result.parser_code.is_empty());
}

#[test]
fn builder_three_token_sequence() {
    let grammar = GrammarBuilder::new("three_seq")
        .token("A", "a")
        .token("B", "b")
        .token("C", "c")
        .rule("root", vec!["A", "B", "C"])
        .start("root")
        .build();
    assert!(build_parser(grammar, opts()).is_ok());
}

#[test]
fn builder_nullable_start() {
    let grammar = GrammarBuilder::new("nullable_start")
        .token("X", "x")
        .rule("root", vec![])
        .rule("root", vec!["X"])
        .start("root")
        .build();
    assert!(build_parser(grammar, opts()).is_ok());
}

#[test]
fn builder_recursive_grammar() {
    let grammar = GrammarBuilder::new("recursive")
        .token("(", "(")
        .token(")", ")")
        .token("ID", r"[a-z]+")
        .rule("expr", vec!["(", "expr", ")"])
        .rule("expr", vec!["ID"])
        .start("expr")
        .build();
    let result = build_parser(grammar, opts()).unwrap();
    assert!(result.build_stats.state_count >= 2);
}

// ===========================================================================
// 2. JSON → build_parser_from_json pipeline (10 tests)
// ===========================================================================

#[test]
fn json_string_literal_builds() {
    let g = minimal_json("v4_str", "STRING", "hello");
    assert!(build_json(&g).is_ok());
}

#[test]
fn json_pattern_builds() {
    let g = minimal_json("v4_pat", "PATTERN", "[0-9]+");
    assert!(build_json(&g).is_ok());
}

#[test]
fn json_preserves_name() {
    let g = two_rule_json("v4_named");
    let r = build_json(&g).unwrap();
    assert_eq!(r.grammar_name, "v4_named");
}

#[test]
fn json_choice_builds() {
    let g = json!({
        "name": "v4_choice",
        "rules": {
            "start": {
                "type": "CHOICE",
                "members": [
                    { "type": "STRING", "value": "yes" },
                    { "type": "STRING", "value": "no" }
                ]
            }
        }
    });
    assert!(build_json(&g).is_ok());
}

#[test]
fn json_seq_builds() {
    let g = json!({
        "name": "v4_seq",
        "rules": {
            "start": {
                "type": "SEQ",
                "members": [
                    { "type": "STRING", "value": "a" },
                    { "type": "STRING", "value": "b" }
                ]
            }
        }
    });
    assert!(build_json(&g).is_ok());
}

#[test]
fn json_repeat_builds() {
    let g = json!({
        "name": "v4_rep",
        "rules": {
            "start": {
                "type": "REPEAT",
                "content": { "type": "STRING", "value": "r" }
            }
        }
    });
    assert!(build_json(&g).is_ok());
}

#[test]
fn json_repeat1_builds() {
    let g = json!({
        "name": "v4_rep1",
        "rules": {
            "start": {
                "type": "REPEAT1",
                "content": { "type": "PATTERN", "value": "[a-z]+" }
            }
        }
    });
    assert!(build_json(&g).is_ok());
}

#[test]
fn json_optional_builds() {
    let g = json!({
        "name": "v4_opt",
        "rules": {
            "start": {
                "type": "SEQ",
                "members": [
                    { "type": "STRING", "value": "x" },
                    {
                        "type": "CHOICE",
                        "members": [
                            { "type": "STRING", "value": "y" },
                            { "type": "BLANK" }
                        ]
                    }
                ]
            }
        }
    });
    assert!(build_json(&g).is_ok());
}

#[test]
fn json_blank_builds() {
    let g = json!({
        "name": "v4_blank",
        "rules": {
            "start": { "type": "BLANK" }
        }
    });
    assert!(build_json(&g).is_ok());
}

#[test]
fn json_token_wrapping_builds() {
    let g = json!({
        "name": "v4_tok",
        "rules": {
            "start": {
                "type": "TOKEN",
                "content": { "type": "PATTERN", "value": "[0-9]+" }
            }
        }
    });
    assert!(build_json(&g).is_ok());
}

// ===========================================================================
// 3. BuildResult / BuildStats validation (8 tests)
// ===========================================================================

#[test]
fn stats_state_count_positive() {
    let grammar = simple_expr_grammar();
    let result = build_parser(grammar, opts()).unwrap();
    assert!(result.build_stats.state_count > 0);
}

#[test]
fn stats_symbol_count_positive() {
    let grammar = simple_expr_grammar();
    let result = build_parser(grammar, opts()).unwrap();
    assert!(result.build_stats.symbol_count > 0);
}

#[test]
fn stats_conflict_cells_non_negative() {
    let grammar = simple_expr_grammar();
    let result = build_parser(grammar, opts()).unwrap();
    // conflict_cells is usize, always >= 0, just verify it compiles and is accessible
    let _count = result.build_stats.conflict_cells;
}

#[test]
fn result_parser_path_is_string() {
    let grammar = simple_expr_grammar();
    let result = build_parser(grammar, opts()).unwrap();
    // parser_path is a string field — just verify it's accessible
    let _path = &result.parser_path;
}

#[test]
fn stats_more_symbols_than_tokens_for_expr() {
    // An expression grammar has tokens + non-terminals, so symbol_count >= 2
    let grammar = simple_expr_grammar();
    let result = build_parser(grammar, opts()).unwrap();
    assert!(result.build_stats.symbol_count >= 2);
}

#[test]
fn json_stats_state_count_grows_with_complexity() {
    let simple = minimal_json("simple_sc", "STRING", "a");
    let complex = json!({
        "name": "complex_sc",
        "rules": {
            "start": {
                "type": "SEQ",
                "members": [
                    { "type": "STRING", "value": "a" },
                    {
                        "type": "CHOICE",
                        "members": [
                            { "type": "STRING", "value": "b" },
                            { "type": "STRING", "value": "c" }
                        ]
                    },
                    { "type": "STRING", "value": "d" }
                ]
            }
        }
    });
    let r_simple = build_json(&simple).unwrap();
    let r_complex = build_json(&complex).unwrap();
    assert!(r_complex.build_stats.state_count >= r_simple.build_stats.state_count);
}

#[test]
fn deterministic_build_produces_same_stats() {
    let g = two_rule_json("v4_det");
    let r1 = build_json(&g).unwrap();
    let r2 = build_json(&g).unwrap();
    assert_eq!(r1.build_stats.state_count, r2.build_stats.state_count);
    assert_eq!(r1.build_stats.symbol_count, r2.build_stats.symbol_count);
}

#[test]
fn node_types_json_contains_array_entries() {
    let grammar = simple_expr_grammar();
    let result = build_parser(grammar, opts()).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&result.node_types_json).unwrap();
    let arr = parsed.as_array().unwrap();
    assert!(!arr.is_empty());
}

// ===========================================================================
// 4. GrammarConverter sample grammar (5 tests)
// ===========================================================================

#[test]
fn sample_grammar_has_tokens() {
    let grammar = GrammarConverter::create_sample_grammar();
    assert!(!grammar.tokens.is_empty());
}

#[test]
fn sample_grammar_has_rules() {
    let grammar = GrammarConverter::create_sample_grammar();
    assert!(!grammar.rules.is_empty());
}

#[test]
fn sample_grammar_name_is_sample() {
    let grammar = GrammarConverter::create_sample_grammar();
    assert_eq!(grammar.name, "sample");
}

#[test]
fn sample_grammar_has_fields() {
    let grammar = GrammarConverter::create_sample_grammar();
    assert!(!grammar.fields.is_empty());
}

#[test]
fn sample_grammar_has_precedence_on_addition() {
    let grammar = GrammarConverter::create_sample_grammar();
    let has_prec = grammar
        .rules
        .values()
        .any(|rules| rules.iter().any(|r| r.precedence.is_some()));
    assert!(has_prec);
}

// ===========================================================================
// 5. GrammarVisualizer output (6 tests)
// ===========================================================================

#[test]
fn visualizer_dot_contains_digraph() {
    let grammar = GrammarConverter::create_sample_grammar();
    let viz = GrammarVisualizer::new(grammar);
    let dot = viz.to_dot();
    assert!(dot.contains("digraph"));
}

#[test]
fn visualizer_dot_contains_terminals() {
    let grammar = GrammarConverter::create_sample_grammar();
    let viz = GrammarVisualizer::new(grammar);
    let dot = viz.to_dot();
    assert!(dot.contains("ellipse"));
}

#[test]
fn visualizer_dot_contains_nonterminals() {
    let grammar = GrammarConverter::create_sample_grammar();
    let viz = GrammarVisualizer::new(grammar);
    let dot = viz.to_dot();
    assert!(dot.contains("lightgreen"));
}

#[test]
fn visualizer_railroad_contains_svg() {
    let grammar = GrammarConverter::create_sample_grammar();
    let viz = GrammarVisualizer::new(grammar);
    let svg = viz.to_railroad_svg();
    assert!(svg.contains("<svg"));
}

#[test]
fn visualizer_dot_from_builder_grammar() {
    let grammar = simple_expr_grammar();
    let viz = GrammarVisualizer::new(grammar);
    let dot = viz.to_dot();
    assert!(dot.contains("digraph"));
    assert!(!dot.is_empty());
}

#[test]
fn visualizer_railroad_from_builder_grammar() {
    let grammar = simple_expr_grammar();
    let viz = GrammarVisualizer::new(grammar);
    let svg = viz.to_railroad_svg();
    assert!(svg.contains("<svg"));
    assert!(svg.contains("</svg>"));
}

// ===========================================================================
// 6. Precedence and associativity (6 tests)
// ===========================================================================

#[test]
fn precedence_grammar_builds_ok() {
    let grammar = GrammarBuilder::new("prec_ok")
        .token("NUM", r"\d+")
        .token("+", "+")
        .token("*", "*")
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "*", "expr"], 2, Associativity::Left)
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    assert!(build_parser(grammar, opts()).is_ok());
}

#[test]
fn precedence_right_assoc_builds() {
    let grammar = GrammarBuilder::new("prec_right")
        .token("NUM", r"\d+")
        .token("^", "^")
        .rule_with_precedence("expr", vec!["expr", "^", "expr"], 3, Associativity::Right)
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    assert!(build_parser(grammar, opts()).is_ok());
}

#[test]
fn precedence_json_prec_left_builds() {
    let g = json!({
        "name": "v4_prec_left",
        "rules": {
            "start": {
                "type": "PREC_LEFT",
                "value": 1,
                "content": {
                    "type": "SEQ",
                    "members": [
                        { "type": "SYMBOL", "name": "start" },
                        { "type": "STRING", "value": "+" },
                        { "type": "SYMBOL", "name": "start" }
                    ]
                }
            }
        }
    });
    assert!(build_json(&g).is_ok());
}

#[test]
fn precedence_json_prec_right_builds() {
    let g = json!({
        "name": "v4_prec_right",
        "rules": {
            "start": {
                "type": "PREC_RIGHT",
                "value": 2,
                "content": {
                    "type": "SEQ",
                    "members": [
                        { "type": "SYMBOL", "name": "start" },
                        { "type": "STRING", "value": "^" },
                        { "type": "SYMBOL", "name": "start" }
                    ]
                }
            }
        }
    });
    assert!(build_json(&g).is_ok());
}

#[test]
fn precedence_multiple_levels_stats() {
    let grammar = GrammarBuilder::new("multi_prec")
        .token("NUM", r"\d+")
        .token("+", "+")
        .token("*", "*")
        .token("-", "-")
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "-", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "*", "expr"], 2, Associativity::Left)
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    let result = build_parser(grammar, opts()).unwrap();
    assert!(result.build_stats.state_count >= 2);
}

#[test]
fn precedence_json_prec_dynamic_builds() {
    let g = json!({
        "name": "v4_prec_dyn",
        "rules": {
            "start": {
                "type": "PREC_DYNAMIC",
                "value": 1,
                "content": { "type": "STRING", "value": "a" }
            }
        }
    });
    assert!(build_json(&g).is_ok());
}

// ===========================================================================
// 7. Extras, externals, inline, supertype (6 tests)
// ===========================================================================

#[test]
fn json_extras_whitespace_builds() {
    let g = json!({
        "name": "v4_extras",
        "rules": {
            "start": { "type": "SYMBOL", "name": "word" },
            "word": { "type": "PATTERN", "value": "[a-z]+" }
        },
        "extras": [
            { "type": "PATTERN", "value": "\\s+" }
        ]
    });
    assert!(build_json(&g).is_ok());
}

#[test]
fn json_inline_rules_accepted() {
    let g = json!({
        "name": "v4_inline",
        "rules": {
            "start": { "type": "SYMBOL", "name": "inner" },
            "inner": { "type": "STRING", "value": "z" }
        },
        "inline": ["inner"]
    });
    assert!(build_json(&g).is_ok());
}

#[test]
fn json_conflicts_declaration_accepted() {
    let g = json!({
        "name": "v4_conflict",
        "rules": {
            "start": {
                "type": "CHOICE",
                "members": [
                    { "type": "SYMBOL", "name": "alpha" },
                    { "type": "SYMBOL", "name": "beta" }
                ]
            },
            "alpha": { "type": "PATTERN", "value": "[a-z]+" },
            "beta": { "type": "PATTERN", "value": "[a-z]+" }
        },
        "conflicts": [["alpha", "beta"]]
    });
    assert!(build_json(&g).is_ok());
}

#[test]
fn json_supertypes_accepted() {
    let g = json!({
        "name": "v4_super",
        "rules": {
            "start": { "type": "SYMBOL", "name": "expression" },
            "expression": {
                "type": "CHOICE",
                "members": [
                    { "type": "SYMBOL", "name": "literal" },
                    { "type": "SYMBOL", "name": "ident" }
                ]
            },
            "literal": { "type": "PATTERN", "value": "[0-9]+" },
            "ident": { "type": "PATTERN", "value": "[a-z]+" }
        },
        "supertypes": ["expression"]
    });
    assert!(build_json(&g).is_ok());
}

#[test]
fn builder_extra_token() {
    let grammar = GrammarBuilder::new("with_extra")
        .token("WORD", r"[a-z]+")
        .token("WS", r"[ \t]+")
        .extra("WS")
        .rule("root", vec!["WORD"])
        .start("root")
        .build();
    assert!(!grammar.extras.is_empty());
}

#[test]
fn builder_inline_rule() {
    let grammar = GrammarBuilder::new("with_inline")
        .token("X", "x")
        .rule("outer", vec!["inner"])
        .rule("inner", vec!["X"])
        .inline("inner")
        .start("outer")
        .build();
    assert!(!grammar.inline_rules.is_empty());
}

// ===========================================================================
// 8. Error and edge cases (6 tests)
// ===========================================================================

#[test]
fn invalid_json_returns_error() {
    let result = build_parser_from_json("not valid json {{{".to_string(), opts());
    assert!(result.is_err());
}

#[test]
fn empty_json_object_returns_error() {
    let result = build_parser_from_json("{}".to_string(), opts());
    assert!(result.is_err());
}

#[test]
fn json_missing_rules_key_returns_error() {
    let g = json!({ "name": "no_rules" });
    assert!(build_json(&g).is_err());
}

#[test]
fn json_empty_rules_returns_error() {
    let g = json!({
        "name": "empty_rules",
        "rules": {}
    });
    assert!(build_json(&g).is_err());
}

#[test]
fn json_unknown_rule_type_returns_error() {
    let g = json!({
        "name": "bad_type",
        "rules": {
            "start": { "type": "UNKNOWN_TYPE", "value": "x" }
        }
    });
    assert!(build_json(&g).is_err());
}

#[test]
fn json_deeply_nested_seq_builds() {
    let g = json!({
        "name": "v4_deep",
        "rules": {
            "start": {
                "type": "SEQ",
                "members": [
                    {
                        "type": "SEQ",
                        "members": [
                            {
                                "type": "SEQ",
                                "members": [
                                    { "type": "STRING", "value": "a" },
                                    { "type": "STRING", "value": "b" }
                                ]
                            },
                            { "type": "STRING", "value": "c" }
                        ]
                    },
                    { "type": "STRING", "value": "d" }
                ]
            }
        }
    });
    assert!(build_json(&g).is_ok());
}
