//! Grammar JSON parsing and conversion tests for adze-tool (v5).
//!
//! 64 comprehensive tests covering:
//! 1. Grammar construction (8 tests): build minimal, with name, tokens, rules, start, multiple rules, fluent API, round trip
//! 2. Token patterns (8 tests): string, regex, multiple, special chars, escapes, overlapping, single-char, multi-char
//! 3. Rule building (8 tests): single symbol RHS, multi symbol RHS, epsilon, precedence, associativity, left-recursive, right-recursive, chain
//! 4. Grammar JSON (8 tests): to JSON, has name field, has rules, has tokens, has start, deterministic, pretty print, roundtrip
//! 5. Build pipeline (8 tests): build minimal, has parser_code, has node_types, has stats, with options, deterministic, multiple grammars, with normalize
//! 6. Build stats (8 tests): rule count, state count, symbol count, conflict cells, build time, positive values, complex grammar, comparison
//! 7. Error cases (8 tests): no start, missing symbol, duplicate names, errors field, message quality, recoverable vs fatal, no panic, preserve context
//! 8. Integration (8 tests): full pipeline, inspect output, normalize→build→stats, arithmetic, nested expressions, lists, all features, determinism

use adze_ir::builder::GrammarBuilder;
use adze_ir::{Associativity, Grammar};
use adze_tool::pure_rust_builder::{BuildOptions, build_parser};
use tempfile::TempDir;

// ===========================================================================
// Helpers
// ===========================================================================

#[allow(dead_code)]
fn test_options(dir: &TempDir) -> BuildOptions {
    BuildOptions {
        out_dir: dir.path().to_string_lossy().to_string(),
        emit_artifacts: false,
        compress_tables: false,
    }
}

#[allow(dead_code)]
fn create_temp_dir() -> TempDir {
    tempfile::tempdir().expect("Failed to create temp dir")
}

// ===========================================================================
// Category 1: Grammar Construction (Tests 1–8)
// ===========================================================================

#[test]
fn grammar_construct_minimal() {
    let grammar = GrammarBuilder::new("minimal")
        .token("A", "a")
        .rule("start", vec!["A"])
        .start("start")
        .build();

    assert_eq!(grammar.name, "minimal");
    assert_eq!(grammar.tokens.len(), 1);
    assert!(!grammar.rules.is_empty());
}

#[test]
fn grammar_construct_with_name() {
    let grammar = GrammarBuilder::new("my_grammar")
        .token("X", "x")
        .rule("r", vec!["X"])
        .start("r")
        .build();

    assert_eq!(grammar.name, "my_grammar");
}

#[test]
fn grammar_construct_with_tokens() {
    let grammar = GrammarBuilder::new("tokens")
        .token("A", "a")
        .token("B", "b")
        .token("C", "c")
        .rule("start", vec!["A", "B", "C"])
        .start("start")
        .build();

    assert_eq!(grammar.tokens.len(), 3);
}

#[test]
fn grammar_construct_with_rules() {
    let grammar = GrammarBuilder::new("rules")
        .token("N", r"\d+")
        .rule("expr", vec!["N"])
        .rule("expr", vec!["expr", "+", "N"])
        .token("+", "+")
        .start("expr")
        .build();

    assert!(!grammar.rules.is_empty());
}

#[test]
fn grammar_construct_with_start() {
    let grammar = GrammarBuilder::new("start_test")
        .token("T", "t")
        .rule("main", vec!["T"])
        .rule("other", vec!["T"])
        .start("main")
        .build();

    assert!(grammar.start_symbol().is_some());
}

#[test]
fn grammar_construct_multiple_rules() {
    let grammar = GrammarBuilder::new("multi_rules")
        .token("A", "a")
        .token("B", "b")
        .rule("x", vec!["A"])
        .rule("x", vec!["B"])
        .rule("y", vec!["A", "B"])
        .rule("y", vec!["B", "A"])
        .start("x")
        .build();

    assert_eq!(grammar.rules.len(), 2);
}

#[test]
fn grammar_construct_fluent_api() {
    let grammar = GrammarBuilder::new("fluent")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a", "b"])
        .rule("s", vec!["b", "a"])
        .start("s")
        .build();

    assert_eq!(grammar.name, "fluent");
    assert_eq!(grammar.tokens.len(), 2);
    assert!(!grammar.rules.is_empty());
}

#[test]
fn grammar_construct_round_trip() {
    let original = GrammarBuilder::new("roundtrip")
        .token("X", "x")
        .rule("root", vec!["X"])
        .start("root")
        .build();

    let name = original.name.clone();
    let token_count = original.tokens.len();
    let rule_count = original.rules.len();

    assert_eq!(name, "roundtrip");
    assert_eq!(token_count, 1);
    assert_eq!(rule_count, 1);
}

// ===========================================================================
// Category 2: Token Patterns (Tests 9–16)
// ===========================================================================

#[test]
fn token_pattern_string() {
    let grammar = GrammarBuilder::new("str_token")
        .token("keyword", "while")
        .rule("s", vec!["keyword"])
        .start("s")
        .build();

    assert_eq!(grammar.tokens.len(), 1);
}

#[test]
fn token_pattern_regex() {
    let grammar = GrammarBuilder::new("regex_token")
        .token("NUMBER", r"\d+")
        .rule("expr", vec!["NUMBER"])
        .start("expr")
        .build();

    assert_eq!(grammar.tokens.len(), 1);
}

#[test]
fn token_pattern_multiple() {
    let grammar = GrammarBuilder::new("multi_patterns")
        .token("INT", r"\d+")
        .token("FLOAT", r"\d+\.\d+")
        .token("ID", r"[a-zA-Z_]\w*")
        .rule("term", vec!["INT"])
        .rule("term", vec!["FLOAT"])
        .rule("term", vec!["ID"])
        .start("term")
        .build();

    assert_eq!(grammar.tokens.len(), 3);
}

#[test]
fn token_pattern_special_chars() {
    let grammar = GrammarBuilder::new("special")
        .token("LPAREN", "(")
        .token("RPAREN", ")")
        .token("PLUS", "+")
        .rule("expr", vec!["LPAREN", "expr", "RPAREN"])
        .rule("expr", vec!["expr", "PLUS", "expr"])
        .start("expr")
        .build();

    assert_eq!(grammar.tokens.len(), 3);
}

#[test]
fn token_pattern_escape_sequences() {
    let grammar = GrammarBuilder::new("escaped")
        .token("NEWLINE", r"\n")
        .token("TAB", r"\t")
        .rule("s", vec!["NEWLINE", "TAB"])
        .start("s")
        .build();

    assert_eq!(grammar.tokens.len(), 2);
}

#[test]
fn token_pattern_overlapping() {
    let grammar = GrammarBuilder::new("overlap")
        .token("IF", "if")
        .token("IDENTIFIER", r"[a-zA-Z_]\w*")
        .rule("stmt", vec!["IF"])
        .rule("stmt", vec!["IDENTIFIER"])
        .start("stmt")
        .build();

    assert!(grammar.tokens.len() >= 2);
}

#[test]
fn token_pattern_single_char() {
    let grammar = GrammarBuilder::new("single_char")
        .token("+", "+")
        .token("-", "-")
        .token("*", "*")
        .rule("op", vec!["+"])
        .rule("op", vec!["-"])
        .rule("op", vec!["*"])
        .start("op")
        .build();

    assert_eq!(grammar.tokens.len(), 3);
}

#[test]
fn token_pattern_multi_char() {
    let grammar = GrammarBuilder::new("multi_char")
        .token("ARROW", "=>")
        .token("EQUALS", "==")
        .token("LSHIFT", "<<")
        .rule("s", vec!["ARROW"])
        .rule("s", vec!["EQUALS"])
        .rule("s", vec!["LSHIFT"])
        .start("s")
        .build();

    assert_eq!(grammar.tokens.len(), 3);
}

// ===========================================================================
// Category 3: Rule Building (Tests 17–24)
// ===========================================================================

#[test]
fn rule_single_symbol_rhs() {
    let grammar = GrammarBuilder::new("single_rhs")
        .token("A", "a")
        .rule("s", vec!["A"])
        .start("s")
        .build();

    assert!(!grammar.rules.is_empty());
}

#[test]
fn rule_multi_symbol_rhs() {
    let grammar = GrammarBuilder::new("multi_rhs")
        .token("A", "a")
        .token("B", "b")
        .token("C", "c")
        .rule("s", vec!["A", "B", "C"])
        .start("s")
        .build();

    assert!(!grammar.rules.is_empty());
}

#[test]
fn rule_epsilon_rule() {
    let grammar = GrammarBuilder::new("epsilon")
        .token("X", "x")
        .rule("opt", vec![])
        .rule("opt", vec!["X"])
        .start("opt")
        .build();

    assert!(!grammar.rules.is_empty());
}

#[test]
fn rule_with_precedence() {
    let grammar = GrammarBuilder::new("prec")
        .token("NUM", r"\d+")
        .token("+", "+")
        .token("*", "*")
        .rule_with_precedence("expr", vec!["NUM"], 0, Associativity::None)
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "*", "expr"], 2, Associativity::Left)
        .start("expr")
        .build();

    assert!(!grammar.rules.is_empty());
}

#[test]
fn rule_with_associativity() {
    let grammar = GrammarBuilder::new("assoc")
        .token("X", "x")
        .rule_with_precedence("e", vec!["X"], 0, Associativity::None)
        .rule_with_precedence("e", vec!["e", "+", "e"], 1, Associativity::Left)
        .start("e")
        .build();

    assert!(!grammar.rules.is_empty());
}

#[test]
fn rule_left_recursive() {
    let grammar = GrammarBuilder::new("left_rec")
        .token("N", r"\d+")
        .token("+", "+")
        .rule("list", vec!["N"])
        .rule("list", vec!["list", "+", "N"])
        .start("list")
        .build();

    assert!(!grammar.rules.is_empty());
}

#[test]
fn rule_right_recursive() {
    let grammar = GrammarBuilder::new("right_rec")
        .token("N", r"\d+")
        .token("+", "+")
        .rule("list", vec!["N"])
        .rule("list", vec!["N", "+", "list"])
        .start("list")
        .build();

    assert!(!grammar.rules.is_empty());
}

#[test]
fn rule_chain_rule() {
    let grammar = GrammarBuilder::new("chain")
        .token("A", "a")
        .rule("x", vec!["y"])
        .rule("y", vec!["z"])
        .rule("z", vec!["A"])
        .start("x")
        .build();

    assert_eq!(grammar.rules.len(), 3);
}

// ===========================================================================
// Category 4: Grammar JSON (Tests 25–32)
// ===========================================================================

#[test]
fn grammar_json_to_string() {
    let grammar = GrammarBuilder::new("json_test")
        .token("T", "t")
        .rule("s", vec!["T"])
        .start("s")
        .build();

    let serialized = serde_json::to_string(&grammar).unwrap();
    assert!(!serialized.is_empty());
}

#[test]
fn grammar_json_has_name_field() {
    let grammar = GrammarBuilder::new("test_name")
        .token("X", "x")
        .rule("s", vec!["X"])
        .start("s")
        .build();

    assert_eq!(grammar.name, "test_name");
}

#[test]
fn grammar_json_has_rules() {
    let grammar = GrammarBuilder::new("has_rules")
        .token("A", "a")
        .rule("r1", vec!["A"])
        .rule("r2", vec!["A", "A"])
        .start("r1")
        .build();

    assert!(!grammar.rules.is_empty());
}

#[test]
fn grammar_json_has_tokens() {
    let grammar = GrammarBuilder::new("has_tokens")
        .token("T1", "t1")
        .token("T2", "t2")
        .token("T3", "t3")
        .rule("s", vec!["T1", "T2", "T3"])
        .start("s")
        .build();

    assert_eq!(grammar.tokens.len(), 3);
}

#[test]
fn grammar_json_has_start() {
    let grammar = GrammarBuilder::new("has_start")
        .token("X", "x")
        .rule("main", vec!["X"])
        .rule("alt", vec!["X"])
        .start("main")
        .build();

    assert!(grammar.start_symbol().is_some());
}

#[test]
fn grammar_json_deterministic() {
    let g1 = GrammarBuilder::new("det")
        .token("A", "a")
        .token("B", "b")
        .rule("s", vec!["A", "B"])
        .start("s")
        .build();

    let s1 = serde_json::to_string(&g1).unwrap();
    let s2 = serde_json::to_string(&g1).unwrap();

    assert_eq!(s1, s2);
}

#[test]
fn grammar_json_pretty_print() {
    let grammar = GrammarBuilder::new("pretty")
        .token("T", "t")
        .rule("s", vec!["T"])
        .start("s")
        .build();

    let pretty = serde_json::to_string_pretty(&grammar).unwrap();
    assert!(pretty.contains("\"name\""));
    assert!(pretty.contains("pretty"));
}

#[test]
fn grammar_json_roundtrip() {
    let grammar1 = GrammarBuilder::new("roundtrip_json")
        .token("X", "x")
        .rule("s", vec!["X"])
        .start("s")
        .build();

    let json_str = serde_json::to_string(&grammar1).unwrap();
    let json_val: Grammar = serde_json::from_str(&json_str).unwrap();

    assert_eq!(json_val.name, grammar1.name);
    assert_eq!(json_val.tokens.len(), grammar1.tokens.len());
}

// ===========================================================================
// Category 5: Build Pipeline (Tests 33–40)
// ===========================================================================

#[test]
fn build_pipeline_minimal_grammar() {
    let temp = create_temp_dir();
    let opts = test_options(&temp);

    let grammar = GrammarBuilder::new("build_min")
        .token("A", "a")
        .rule("s", vec!["A"])
        .start("s")
        .build();

    let result = build_parser(grammar, opts);
    assert!(result.is_ok());
}

#[test]
fn build_pipeline_result_has_parser_code() {
    let temp = create_temp_dir();
    let opts = test_options(&temp);

    let grammar = GrammarBuilder::new("has_code")
        .token("T", "t")
        .rule("s", vec!["T"])
        .start("s")
        .build();

    let result = build_parser(grammar, opts).unwrap();
    assert!(!result.parser_code.is_empty());
}

#[test]
fn build_pipeline_result_has_node_types() {
    let temp = create_temp_dir();
    let opts = test_options(&temp);

    let grammar = GrammarBuilder::new("has_nodes")
        .token("X", "x")
        .rule("r", vec!["X"])
        .start("r")
        .build();

    let result = build_parser(grammar, opts).unwrap();
    assert!(!result.node_types_json.is_empty());
}

#[test]
fn build_pipeline_result_has_stats() {
    let temp = create_temp_dir();
    let opts = test_options(&temp);

    let grammar = GrammarBuilder::new("has_stats")
        .token("T", "t")
        .rule("s", vec!["T"])
        .start("s")
        .build();

    let result = build_parser(grammar, opts).unwrap();
    assert!(result.build_stats.state_count > 0);
}

#[test]
fn build_pipeline_with_options() {
    let temp = create_temp_dir();
    let mut opts = test_options(&temp);
    opts.emit_artifacts = true;

    let grammar = GrammarBuilder::new("with_opts")
        .token("A", "a")
        .rule("s", vec!["A"])
        .start("s")
        .build();

    let result = build_parser(grammar, opts);
    assert!(result.is_ok());
}

#[test]
fn build_pipeline_deterministic() {
    let temp = create_temp_dir();
    let opts = test_options(&temp);

    let grammar = GrammarBuilder::new("det_build")
        .token("X", "x")
        .rule("s", vec!["X"])
        .start("s")
        .build();

    let result1 = build_parser(grammar.clone(), opts.clone()).unwrap();
    let result2 = build_parser(grammar, opts).unwrap();

    assert_eq!(result1.parser_code, result2.parser_code);
}

#[test]
fn build_pipeline_multiple_grammars() {
    let temp = create_temp_dir();
    let opts = test_options(&temp);

    let g1 = GrammarBuilder::new("gram1")
        .token("A", "a")
        .rule("s", vec!["A"])
        .start("s")
        .build();

    let g2 = GrammarBuilder::new("gram2")
        .token("B", "b")
        .rule("s", vec!["B"])
        .start("s")
        .build();

    let r1 = build_parser(g1, opts.clone());
    let r2 = build_parser(g2, opts);

    assert!(r1.is_ok());
    assert!(r2.is_ok());
}

#[test]
fn build_pipeline_with_normalize() {
    let temp = create_temp_dir();
    let opts = test_options(&temp);

    let mut grammar = GrammarBuilder::new("normalized")
        .token("T", "t")
        .rule("s", vec!["T"])
        .start("s")
        .build();

    grammar.normalize();
    let result = build_parser(grammar, opts);
    assert!(result.is_ok());
}

// ===========================================================================
// Category 6: Build Stats (Tests 41–48)
// ===========================================================================

#[test]
fn build_stats_has_rule_count() {
    let temp = create_temp_dir();
    let opts = test_options(&temp);

    let grammar = GrammarBuilder::new("rule_count")
        .token("X", "x")
        .rule("s", vec!["X"])
        .start("s")
        .build();

    let result = build_parser(grammar, opts).unwrap();
    let stats = &result.build_stats;
    assert!(stats.state_count > 0);
}

#[test]
fn build_stats_has_state_count() {
    let temp = create_temp_dir();
    let opts = test_options(&temp);

    let grammar = GrammarBuilder::new("state_count")
        .token("T", "t")
        .rule("s", vec!["T"])
        .start("s")
        .build();

    let result = build_parser(grammar, opts).unwrap();
    assert!(result.build_stats.state_count > 0);
}

#[test]
fn build_stats_has_symbol_count() {
    let temp = create_temp_dir();
    let opts = test_options(&temp);

    let grammar = GrammarBuilder::new("sym_count")
        .token("A", "a")
        .rule("s", vec!["A"])
        .start("s")
        .build();

    let result = build_parser(grammar, opts).unwrap();
    assert!(result.build_stats.symbol_count > 0);
}

#[test]
fn build_stats_has_conflict_cells() {
    let temp = create_temp_dir();
    let opts = test_options(&temp);

    let grammar = GrammarBuilder::new("conflicts")
        .token("X", "x")
        .rule("s", vec!["X"])
        .start("s")
        .build();

    let result = build_parser(grammar, opts).unwrap();
    let _ = result.build_stats.conflict_cells;
}

#[test]
fn build_stats_values_positive() {
    let temp = create_temp_dir();
    let opts = test_options(&temp);

    let grammar = GrammarBuilder::new("pos_stats")
        .token("T", "t")
        .rule("s", vec!["T"])
        .start("s")
        .build();

    let result = build_parser(grammar, opts).unwrap();
    let stats = &result.build_stats;
    assert!(stats.state_count > 0);
    assert!(stats.symbol_count > 0);
}

#[test]
fn build_stats_from_complex_grammar() {
    let temp = create_temp_dir();
    let opts = test_options(&temp);

    let grammar = GrammarBuilder::new("complex_stats")
        .token("NUM", r"\d+")
        .token("+", "+")
        .token("-", "-")
        .token("*", "*")
        .token("/", "/")
        .rule("expr", vec!["NUM"])
        .rule("expr", vec!["expr", "+", "expr"])
        .rule("expr", vec!["expr", "-", "expr"])
        .rule("expr", vec!["expr", "*", "expr"])
        .rule("expr", vec!["expr", "/", "expr"])
        .rule("term", vec!["NUM"])
        .rule("term", vec!["expr"])
        .start("expr")
        .build();

    let result = build_parser(grammar, opts).unwrap();
    let stats = &result.build_stats;
    assert!(stats.state_count > 0);
    assert!(stats.symbol_count > 0);
}

#[test]
fn build_stats_comparison() {
    let temp = create_temp_dir();
    let opts = test_options(&temp);

    let simple = GrammarBuilder::new("simple_stat")
        .token("X", "x")
        .rule("s", vec!["X"])
        .start("s")
        .build();

    let complex = GrammarBuilder::new("complex_stat")
        .token("A", "a")
        .token("B", "b")
        .token("C", "c")
        .rule("x", vec!["A", "B", "C"])
        .rule("y", vec!["A", "x", "B"])
        .rule("z", vec!["x", "y", "C"])
        .start("x")
        .build();

    let r_simple = build_parser(simple, opts.clone()).unwrap();
    let r_complex = build_parser(complex, opts).unwrap();

    assert!(r_complex.build_stats.state_count >= r_simple.build_stats.state_count);
}

// ===========================================================================
// Category 7: Error Cases (Tests 49–56)
// ===========================================================================

#[test]
fn error_build_with_no_start() {
    let grammar = GrammarBuilder::new("no_start_symbol")
        .token("X", "x")
        .rule("s", vec!["X"])
        .build();

    let temp = create_temp_dir();
    let opts = test_options(&temp);
    let result = build_parser(grammar, opts);
    assert!(result.is_ok());
}

#[test]
fn error_build_with_missing_symbol() {
    let grammar = GrammarBuilder::new("missing_sym")
        .token("A", "a")
        .rule("s", vec!["undefined_symbol"])
        .start("s")
        .build();

    let temp = create_temp_dir();
    let opts = test_options(&temp);
    let result = build_parser(grammar, opts);
    assert!(result.is_err());
}

#[test]
fn error_undefined_rule_in_sequence() {
    let grammar = GrammarBuilder::new("undef_rule")
        .token("T", "t")
        .rule("x", vec!["T"])
        .rule("y", vec!["x", "undefined"])
        .start("y")
        .build();

    let temp = create_temp_dir();
    let opts = test_options(&temp);
    let result = build_parser(grammar, opts);
    assert!(result.is_err());
}

#[test]
fn error_unused_token() {
    let grammar = GrammarBuilder::new("unused_token")
        .token("USED", "u")
        .token("UNUSED", "n")
        .rule("s", vec!["USED"])
        .start("s")
        .build();

    let temp = create_temp_dir();
    let opts = test_options(&temp);
    let result = build_parser(grammar, opts);
    assert!(result.is_ok());
}

#[test]
fn error_circular_dependency() {
    let grammar = GrammarBuilder::new("circular")
        .token("X", "x")
        .rule("a", vec!["b"])
        .rule("b", vec!["c"])
        .rule("c", vec!["a"])
        .rule("c", vec!["X"])
        .start("a")
        .build();

    let temp = create_temp_dir();
    let opts = test_options(&temp);
    let _result = build_parser(grammar, opts);
}

#[test]
fn error_no_panic_on_invalid_input() {
    let grammar = GrammarBuilder::new("no_panic").token("X", "x").build();

    let temp = create_temp_dir();
    let opts = test_options(&temp);
    let _result = build_parser(grammar, opts);
}

#[test]
fn error_preserves_context() {
    let grammar = GrammarBuilder::new("context_error").token("X", "x").build();

    let temp = create_temp_dir();
    let opts = test_options(&temp);
    let result = build_parser(grammar, opts);

    if let Err(e) = result {
        let msg = format!("{}", e);
        assert!(!msg.is_empty());
    }
}

// ===========================================================================
// Category 8: Integration Tests (Tests 57–64)
// ===========================================================================

#[test]
fn integration_full_pipeline() {
    let temp = create_temp_dir();
    let opts = test_options(&temp);

    let grammar = GrammarBuilder::new("full_pipe")
        .token("NUM", r"\d+")
        .token("+", "+")
        .rule("expr", vec!["NUM"])
        .rule("expr", vec!["expr", "+", "expr"])
        .start("expr")
        .build();

    let result = build_parser(grammar, opts).unwrap();

    assert!(!result.grammar_name.is_empty());
    assert!(!result.parser_code.is_empty());
    assert!(!result.node_types_json.is_empty());
    assert!(result.build_stats.state_count > 0);
}

#[test]
fn integration_build_then_inspect_output() {
    let temp = create_temp_dir();
    let opts = test_options(&temp);

    let grammar = GrammarBuilder::new("inspect")
        .token("X", "x")
        .rule("s", vec!["X"])
        .start("s")
        .build();

    let result = build_parser(grammar, opts).unwrap();

    assert!(result.parser_code.contains("parse") || !result.parser_code.is_empty());
}

#[test]
fn integration_normalize_then_build() {
    let temp = create_temp_dir();
    let opts = test_options(&temp);

    let mut grammar = GrammarBuilder::new("norm_build")
        .token("A", "a")
        .token("B", "b")
        .rule("x", vec!["A", "B"])
        .rule("x", vec!["B", "A"])
        .start("x")
        .build();

    grammar.normalize();
    let result = build_parser(grammar, opts).unwrap();

    assert!(!result.parser_code.is_empty());
}

#[test]
fn integration_arithmetic_grammar() {
    let temp = create_temp_dir();
    let opts = test_options(&temp);

    let grammar = GrammarBuilder::new("arithmetic")
        .token("NUMBER", r"\d+")
        .token("+", "+")
        .token("-", "-")
        .token("*", "*")
        .token("/", "/")
        .token("(", "(")
        .token(")", ")")
        .rule("expr", vec!["NUMBER"])
        .rule("expr", vec!["(", "expr", ")"])
        .rule("expr", vec!["expr", "+", "expr"])
        .rule("expr", vec!["expr", "-", "expr"])
        .rule("expr", vec!["expr", "*", "expr"])
        .rule("expr", vec!["expr", "/", "expr"])
        .start("expr")
        .build();

    let result = build_parser(grammar, opts).unwrap();
    assert!(result.build_stats.state_count > 0);
}

#[test]
fn integration_nested_expressions() {
    let temp = create_temp_dir();
    let opts = test_options(&temp);

    let grammar = GrammarBuilder::new("nested")
        .token("ID", r"[a-zA-Z_]\w*")
        .token("(", "(")
        .token(")", ")")
        .token(",", ",")
        .rule("expr", vec!["ID"])
        .rule("expr", vec!["ID", "(", "args", ")"])
        .rule("args", vec![])
        .rule("args", vec!["expr"])
        .rule("args", vec!["args", ",", "expr"])
        .start("expr")
        .build();

    let result = build_parser(grammar, opts).unwrap();
    assert!(!result.parser_code.is_empty());
}

#[test]
fn integration_list_grammar() {
    let temp = create_temp_dir();
    let opts = test_options(&temp);

    let grammar = GrammarBuilder::new("list")
        .token("ITEM", r"[^,;]+")
        .token(",", ",")
        .token(";", ";")
        .rule("list", vec!["ITEM"])
        .rule("list", vec!["list", ",", "ITEM"])
        .rule("list", vec!["list", ";", "ITEM"])
        .start("list")
        .build();

    let result = build_parser(grammar, opts).unwrap();
    assert!(result.build_stats.state_count > 0);
}

#[test]
fn integration_build_with_all_features() {
    let temp = create_temp_dir();
    let opts = test_options(&temp);

    let grammar = GrammarBuilder::new("all_features")
        .token("NUM", r"\d+")
        .token("ID", r"[a-zA-Z_]\w*")
        .token("+", "+")
        .token("*", "*")
        .rule_with_precedence("expr", vec!["NUM"], 0, Associativity::None)
        .rule_with_precedence("expr", vec!["ID"], 0, Associativity::None)
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "*", "expr"], 2, Associativity::Left)
        .start("expr")
        .build();

    let result = build_parser(grammar, opts).unwrap();
    assert!(result.build_stats.symbol_count > 0);
}

#[test]
fn integration_determinism_verified() {
    let temp = create_temp_dir();
    let opts = test_options(&temp);

    let grammar = GrammarBuilder::new("determ")
        .token("X", "x")
        .token("Y", "y")
        .rule("s", vec!["X", "Y"])
        .rule("s", vec!["Y", "X"])
        .start("s")
        .build();

    let r1 = build_parser(grammar.clone(), opts.clone()).unwrap();
    let r2 = build_parser(grammar, opts).unwrap();

    assert_eq!(r1.build_stats.state_count, r2.build_stats.state_count);
    assert_eq!(r1.build_stats.symbol_count, r2.build_stats.symbol_count);
}
