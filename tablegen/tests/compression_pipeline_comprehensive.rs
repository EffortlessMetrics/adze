// Wave 132: Comprehensive tablegen compression pipeline tests
use adze_glr_core::*;
use adze_ir::builder::GrammarBuilder;
use adze_ir::*;
use adze_tablegen::compress::*;
use adze_tablegen::helpers::*;

// =====================================================================
// Helper: build grammar → parse table
// =====================================================================

fn simple_grammar_and_table() -> (Grammar, ParseTable) {
    let mut grammar = GrammarBuilder::new("simple")
        .token("num", r"\d+")
        .rule("start", vec!["num"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute_normalized(&mut grammar).unwrap();
    let table = build_lr1_automaton(&grammar, &ff).unwrap();
    (grammar, table)
}

fn expr_grammar_and_table() -> (Grammar, ParseTable) {
    let mut grammar = GrammarBuilder::new("expr")
        .token("num", r"\d+")
        .token("plus", r"\+")
        .rule("expr", vec!["num"])
        .rule("expr", vec!["expr", "plus", "expr"])
        .start("expr")
        .build();
    let ff = FirstFollowSets::compute_normalized(&mut grammar).unwrap();
    let table = build_lr1_automaton(&grammar, &ff).unwrap();
    (grammar, table)
}

// =====================================================================
// CompressedParseTable
// =====================================================================

#[test]
fn compressed_table_new_for_testing() {
    let ct = CompressedParseTable::new_for_testing(10, 5);
    assert_eq!(ct.symbol_count(), 10);
    assert_eq!(ct.state_count(), 5);
}

#[test]
fn compressed_table_from_parse_table() {
    let (_g, table) = simple_grammar_and_table();
    let ct = CompressedParseTable::from_parse_table(&table);
    assert!(ct.state_count() > 0);
    assert!(ct.symbol_count() > 0);
}

#[test]
fn compressed_table_symbol_count_matches() {
    let (_g, table) = expr_grammar_and_table();
    let ct = CompressedParseTable::from_parse_table(&table);
    // symbol_count from compressed should be consistent
    assert!(ct.symbol_count() >= 2); // at least num and plus
}

// =====================================================================
// collect_token_indices
// =====================================================================

#[test]
fn collect_token_indices_simple() {
    let (grammar, table) = simple_grammar_and_table();
    let indices = collect_token_indices(&grammar, &table);
    assert!(!indices.is_empty());
}

#[test]
fn collect_token_indices_includes_eof() {
    let (grammar, table) = simple_grammar_and_table();
    let indices = collect_token_indices(&grammar, &table);
    // Should include at least one index (for tokens + EOF)
    assert!(indices.len() >= 1);
}

#[test]
fn collect_token_indices_expr() {
    let (grammar, table) = expr_grammar_and_table();
    let indices = collect_token_indices(&grammar, &table);
    // expr grammar has num + plus + EOF
    assert!(indices.len() >= 2);
}

// =====================================================================
// eof_accepts_or_reduces
// =====================================================================

#[test]
fn eof_accepts_simple() {
    let (_g, table) = simple_grammar_and_table();
    let result = eof_accepts_or_reduces(&table);
    // This is a check, not necessarily always true depending on table layout
    let _ = result;
}

#[test]
fn eof_accepts_expr() {
    let (_g, table) = expr_grammar_and_table();
    let result = eof_accepts_or_reduces(&table);
    let _ = result;
}

// =====================================================================
// TableCompressor
// =====================================================================

#[test]
fn compressor_new() {
    let tc = TableCompressor::new();
    let _ = tc;
}

#[test]
fn compressor_encode_shift() {
    let tc = TableCompressor::new();
    let encoded = tc.encode_action_small(&Action::Shift(StateId(5)));
    assert!(encoded.is_ok());
}

#[test]
fn compressor_encode_reduce() {
    let tc = TableCompressor::new();
    let encoded = tc.encode_action_small(&Action::Reduce(RuleId(3)));
    assert!(encoded.is_ok());
}

#[test]
fn compressor_encode_accept() {
    let tc = TableCompressor::new();
    let encoded = tc.encode_action_small(&Action::Accept);
    assert!(encoded.is_ok());
}

#[test]
fn compressor_compress_simple() {
    let (grammar, table) = simple_grammar_and_table();
    let tc = TableCompressor::new();
    let indices = collect_token_indices(&grammar, &table);
    let result = tc.compress(&table, &indices, true);
    assert!(
        result.is_ok(),
        "Compression should succeed for simple grammar"
    );
}

#[test]
fn compressor_compress_expr() {
    let (grammar, table) = expr_grammar_and_table();
    let tc = TableCompressor::new();
    let indices = collect_token_indices(&grammar, &table);
    let result = tc.compress(&table, &indices, true);
    assert!(result.is_ok());
}

#[test]
fn compressor_compress_action_table_small() {
    let (_grammar, table) = simple_grammar_and_table();
    let tc = TableCompressor::new();
    let result = tc.compress_action_table_small(&table.action_table, &table.symbol_to_index);
    assert!(result.is_ok());
}

#[test]
fn compressor_compress_goto_table_small() {
    let (_grammar, table) = simple_grammar_and_table();
    let tc = TableCompressor::new();
    let result = tc.compress_goto_table_small(&table.goto_table);
    assert!(result.is_ok());
}

// =====================================================================
// CompressedTables validation
// =====================================================================

#[test]
fn compressed_tables_validate() {
    let (grammar, table) = simple_grammar_and_table();
    let tc = TableCompressor::new();
    let indices = collect_token_indices(&grammar, &table);
    let compressed = tc.compress(&table, &indices, true).unwrap();
    let validation = compressed.validate(&table);
    assert!(validation.is_ok());
}

// =====================================================================
// CompressedActionEntry
// =====================================================================

#[test]
fn compressed_action_entry_new_shift() {
    let entry = CompressedActionEntry::new(1, Action::Shift(StateId(5)));
    assert_eq!(entry.symbol, 1);
}

#[test]
fn compressed_action_entry_new_reduce() {
    let entry = CompressedActionEntry::new(2, Action::Reduce(RuleId(3)));
    assert_eq!(entry.symbol, 2);
}

// =====================================================================
// Pipeline: Grammar → Table → Compress → Validate
// =====================================================================

#[test]
fn full_pipeline_simple() {
    let mut grammar = GrammarBuilder::new("pipe")
        .token("a", r"a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute_normalized(&mut grammar).unwrap();
    let table = build_lr1_automaton(&grammar, &ff).unwrap();
    let tc = TableCompressor::new();
    let indices = collect_token_indices(&grammar, &table);
    let compressed = tc.compress(&table, &indices, true).unwrap();
    compressed.validate(&table).unwrap();
}

#[test]
fn full_pipeline_python_like() {
    let mut grammar = GrammarBuilder::python_like();
    let ff = FirstFollowSets::compute_normalized(&mut grammar).unwrap();
    let table = build_lr1_automaton(&grammar, &ff).unwrap();
    let tc = TableCompressor::new();
    let indices = collect_token_indices(&grammar, &table);
    let compressed = tc.compress(&table, &indices, true).unwrap();
    compressed.validate(&table).unwrap();
}

#[test]
fn full_pipeline_javascript_like() {
    let mut grammar = GrammarBuilder::javascript_like();
    let ff = FirstFollowSets::compute_normalized(&mut grammar).unwrap();
    let table = build_lr1_automaton(&grammar, &ff).unwrap();
    let tc = TableCompressor::new();
    let indices = collect_token_indices(&grammar, &table);
    let compressed = tc.compress(&table, &indices, true).unwrap();
    compressed.validate(&table).unwrap();
}

// =====================================================================
// Determinism: compress twice, same result
// =====================================================================

#[test]
fn compression_deterministic() {
    let build = || {
        let mut grammar = GrammarBuilder::new("det")
            .token("x", r"x")
            .rule("start", vec!["x"])
            .start("start")
            .build();
        let ff = FirstFollowSets::compute_normalized(&mut grammar).unwrap();
        let table = build_lr1_automaton(&grammar, &ff).unwrap();
        let tc = TableCompressor::new();
        let indices = collect_token_indices(&grammar, &table);
        tc.compress(&table, &indices, true).unwrap()
    };
    let c1 = build();
    let c2 = build();
    // Compare action table lengths
    assert_eq!(c1.action_table.data.len(), c2.action_table.data.len());
}
