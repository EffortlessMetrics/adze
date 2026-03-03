//! Tests for the testing infrastructure itself.
//!
//! Verifies that test fixture grammars are valid, helper functions work
//! correctly, builders produce expected results, and fixture parsers can
//! handle simple inputs.

use adze_ir::Associativity;
use adze_ir::builder::GrammarBuilder;
use adze_testing::assertions::*;
use adze_testing::fixtures::*;
use adze_testing::grammar_helpers::*;
use adze_testing::snapshots::*;
use adze_testing::{assert_grammar_invalid, assert_grammar_valid};

// ===========================================================================
// 1. Fixture grammars are valid
// ===========================================================================

#[test]
fn trivial_grammar_produces_valid_parse_table() {
    let g = trivial_grammar();
    let table = build_parse_table(&g).expect("trivial grammar should produce a parse table");
    assert!(table.state_count > 0);
    assert_table_consistent(&table);
}

#[test]
fn arithmetic_grammar_produces_valid_parse_table() {
    let g = arithmetic_grammar();
    let table = build_parse_table(&g).expect("arithmetic grammar should produce a parse table");
    assert!(table.state_count > 0);
    assert_table_consistent(&table);
    assert_no_dead_states(&table);
}

#[test]
fn test_grammar_single_rule_is_valid() {
    let g = test_grammar(&[("start", &["A"])]);
    let table = build_parse_table(&g).expect("single-rule grammar should produce a parse table");
    assert!(table.state_count > 0);
}

#[test]
fn test_grammar_multi_rule_is_valid() {
    let g = test_grammar(&[("expr", &["NUMBER"]), ("expr", &["expr", "+", "expr"])]);
    let table = build_parse_table(&g).expect("multi-rule grammar should produce a parse table");
    assert!(table.state_count > 0);
}

#[test]
fn javascript_like_grammar_is_valid() {
    let g = GrammarBuilder::javascript_like();
    let table =
        build_parse_table(&g).expect("javascript-like grammar should produce a parse table");
    assert!(table.state_count >= 10);
    assert_table_consistent(&table);
}

#[test]
fn python_like_grammar_is_valid() {
    let g = GrammarBuilder::python_like();
    let table = build_parse_table(&g).expect("python-like grammar should produce a parse table");
    assert!(table.state_count > 0);
    assert_table_consistent(&table);
}

// ===========================================================================
// 2. Helper functions work correctly
// ===========================================================================

#[test]
fn test_grammar_auto_registers_terminals() {
    let g = test_grammar(&[("start", &["FOO", "BAR"])]);
    assert_has_token(&g, "FOO");
    assert_has_token(&g, "BAR");
}

#[test]
fn test_grammar_distinguishes_terminals_from_nonterminals() {
    let g = test_grammar(&[("expr", &["NUMBER"]), ("expr", &["expr", "+", "expr"])]);
    assert_has_rule(&g, "expr");
    assert_has_token(&g, "NUMBER");
    assert_has_token(&g, "+");
}

#[test]
fn test_grammar_first_lhs_is_start() {
    let g = test_grammar(&[("program", &["stmt"]), ("stmt", &["X"])]);
    assert_start_symbol(&g, "program");
}

#[test]
fn build_parse_table_returns_error_for_empty_grammar() {
    // A Grammar with no rules should fail
    let g = GrammarBuilder::new("empty").build();
    let result = build_parse_table(&g);
    assert!(
        result.is_err(),
        "empty grammar should fail to build parse table"
    );
}

#[test]
fn arithmetic_grammar_has_expected_tokens() {
    let g = arithmetic_grammar();
    assert_has_token(&g, "NUMBER");
    assert_has_token(&g, "+");
    assert_has_token(&g, "-");
    assert_has_token(&g, "*");
    assert_has_token(&g, "/");
    assert_has_token(&g, "(");
    assert_has_token(&g, ")");
}

#[test]
fn arithmetic_grammar_has_expected_start_symbol() {
    let g = arithmetic_grammar();
    assert_start_symbol(&g, "expr");
}

// ===========================================================================
// 3. Builder produces expected results
// ===========================================================================

#[test]
fn grammar_builder_sets_name() {
    let g = GrammarBuilder::new("my_lang")
        .token("X", "x")
        .rule("s", vec!["X"])
        .start("s")
        .build();
    assert_eq!(g.name, "my_lang");
}

#[test]
fn grammar_builder_with_precedence_is_valid() {
    let g = GrammarBuilder::new("prec")
        .token("NUM", r"\d+")
        .token("+", "+")
        .token("*", "*")
        .rule_with_precedence("e", vec!["e", "+", "e"], 1, Associativity::Left)
        .rule_with_precedence("e", vec!["e", "*", "e"], 2, Associativity::Left)
        .rule("e", vec!["NUM"])
        .start("e")
        .build();

    let table = build_parse_table(&g).expect("precedence grammar should build");
    assert!(table.state_count > 0);
    assert_table_consistent(&table);
}

#[test]
fn grammar_builder_chain_produces_valid_grammar() {
    let g = GrammarBuilder::new("chain")
        .token("A", "a")
        .token("B", "b")
        .token("C", "c")
        .rule("start", vec!["A", "middle"])
        .rule("middle", vec!["B", "end"])
        .rule("end", vec!["C"])
        .start("start")
        .build();

    assert_has_rule(&g, "start");
    assert_has_rule(&g, "middle");
    assert_has_rule(&g, "end");
    let table = build_parse_table(&g).expect("chain grammar should build");
    assert!(table.state_count > 0);
}

#[test]
fn test_grammar_helper_sets_grammar_name_to_test() {
    let g = test_grammar(&[("s", &["X"])]);
    assert_eq!(g.name, "test");
}

// ===========================================================================
// 4. Assertion helpers
// ===========================================================================

#[test]
fn assert_rule_count_matches() {
    let g = test_grammar(&[("expr", &["A"]), ("expr", &["B"]), ("expr", &["C"])]);
    assert_rule_count(&g, "expr", 3);
}

#[test]
fn assert_has_production_matches() {
    let g = test_grammar(&[("sum", &["X", "+", "Y"])]);
    assert_has_production(&g, "sum", &["X", "+", "Y"]);
}

#[test]
fn assert_min_states_passes_for_nontrivial_grammar() {
    let g = arithmetic_grammar();
    let table = build_parse_table(&g).unwrap();
    assert_min_states(&table, 1);
}

#[test]
fn assert_grammar_valid_macro_works() {
    let g = trivial_grammar();
    let table = assert_grammar_valid!(g);
    assert!(table.state_count > 0);
}

#[test]
fn assert_grammar_invalid_macro_works() {
    let g = GrammarBuilder::new("empty").build();
    assert_grammar_invalid!(g);
}

// ===========================================================================
// 5. Snapshot helpers
// ===========================================================================

#[test]
fn grammar_snapshot_includes_tokens_and_rules() {
    let g = arithmetic_grammar();
    let snap = grammar_snapshot(&g);
    assert!(snap.contains("Grammar: arithmetic"));
    assert!(snap.contains("Tokens"));
    assert!(snap.contains("Rules"));
    assert!(snap.contains("NUMBER"));
}

#[test]
fn grammar_json_snapshot_is_valid_json() {
    let g = trivial_grammar();
    let json = grammar_json_snapshot(&g);
    let parsed: serde_json::Value = serde_json::from_str(&json).expect("should be valid JSON");
    assert!(parsed.is_object());
}

#[test]
fn parse_table_summary_has_key_fields() {
    let g = trivial_grammar();
    let table = build_parse_table(&g).unwrap();
    let summary = parse_table_summary_snapshot(&table);
    assert!(summary.contains("States:"));
    assert!(summary.contains("Symbols:"));
    assert!(summary.contains("Tokens:"));
    assert!(summary.contains("EOF symbol:"));
    assert!(summary.contains("Start symbol:"));
}

// ===========================================================================
// 6. Fixture loading
// ===========================================================================

#[test]
fn parse_corpus_empty_input() {
    let entries = parse_corpus("");
    assert!(entries.is_empty());
}

#[test]
fn parse_corpus_single_entry() {
    let corpus = "=== Test\ninput\n---\n(output)\n";
    let entries = parse_corpus(corpus);
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].title, "Test");
    assert_eq!(entries[0].input, "input");
    assert_eq!(entries[0].expected, "(output)");
}

#[test]
fn parse_corpus_strips_whitespace_from_expected() {
    let corpus = "=== Trim\ncode\n---\n\n  (tree)\n\n";
    let entries = parse_corpus(corpus);
    assert_eq!(entries[0].expected, "(tree)");
}

#[test]
fn parse_corpus_skips_empty_title() {
    let corpus = "===\ninput\n---\n(out)\n";
    let entries = parse_corpus(corpus);
    assert!(entries.is_empty(), "empty titles should be skipped");
}

#[test]
fn parse_corpus_multiline_input() {
    let corpus = "=== Multi\nline 1\nline 2\nline 3\n---\n(tree)\n";
    let entries = parse_corpus(corpus);
    assert_eq!(entries.len(), 1);
    assert!(entries[0].input.contains("line 1"));
    assert!(entries[0].input.contains("line 3"));
}

#[test]
fn corpus_entry_equality() {
    let a = CorpusEntry {
        title: "Test".to_string(),
        input: "code".to_string(),
        expected: "(tree)".to_string(),
    };
    let b = a.clone();
    assert_eq!(a, b);
}

#[test]
fn write_temp_fixture_creates_nested_dirs() {
    let dir = tempfile::tempdir().unwrap();
    let path = write_temp_fixture(dir.path(), "sub/dir/test.txt", "content").unwrap();
    assert_eq!(std::fs::read_to_string(&path).unwrap(), "content");
}

#[test]
fn fixtures_dir_returns_a_path() {
    let dir = fixtures_dir();
    assert!(!dir.as_os_str().is_empty());
}

// ===========================================================================
// 7. Cross-function integration: grammar → table → snapshot
// ===========================================================================

#[test]
fn full_pipeline_trivial() {
    let g = trivial_grammar();
    let table = build_parse_table(&g).unwrap();
    assert_table_consistent(&table);

    let snap = grammar_snapshot(&g);
    assert!(snap.contains("trivial"));

    let summary = parse_table_summary_snapshot(&table);
    assert!(summary.contains("States:"));
}

#[test]
fn full_pipeline_arithmetic() {
    let g = arithmetic_grammar();
    let table = build_parse_table(&g).unwrap();
    assert_table_consistent(&table);
    assert_no_dead_states(&table);
    assert_has_rule(&g, "expr");
    assert_start_symbol(&g, "expr");

    let snap = grammar_snapshot(&g);
    assert!(snap.contains("arithmetic"));

    let json = grammar_json_snapshot(&g);
    let roundtrip: adze_ir::Grammar = serde_json::from_str(&json).unwrap();
    assert_eq!(roundtrip.name, "arithmetic");
}
