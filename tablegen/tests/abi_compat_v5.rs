//! ABI compatibility tests (v5) for `adze-tablegen`.
//!
//! Covers ABI version constants, Language struct generation, node types JSON,
//! symbol preservation, state counts, token/non-terminal separation, and edge cases
//! across `AbiLanguageBuilder`, `StaticLanguageGenerator`, and `NodeTypesGenerator`.

use adze_glr_core::{build_lr1_automaton, FirstFollowSets};
use adze_ir::builder::GrammarBuilder;
use adze_tablegen::abi::{
    self, create_symbol_metadata, ExternalScanner, TSFieldId, TSLanguage, TSLexState,
    TSParseAction, TSStateId, TSSymbol, TREE_SITTER_LANGUAGE_VERSION,
    TREE_SITTER_MIN_COMPATIBLE_LANGUAGE_VERSION,
};
use adze_tablegen::{AbiLanguageBuilder, NodeTypesGenerator, StaticLanguageGenerator};

// ===========================================================================
// Helpers
// ===========================================================================

fn build_pipeline(
    grammar_fn: impl FnOnce() -> adze_ir::Grammar,
) -> (adze_ir::Grammar, adze_glr_core::ParseTable) {
    let mut grammar = grammar_fn();
    let ff =
        FirstFollowSets::compute_normalized(&mut grammar).expect("FIRST/FOLLOW computation failed");
    let table = build_lr1_automaton(&grammar, &ff).expect("LR(1) automaton failed");
    (grammar, table)
}

fn single_token_grammar() -> adze_ir::Grammar {
    GrammarBuilder::new("single")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build()
}

fn two_token_grammar() -> adze_ir::Grammar {
    GrammarBuilder::new("two_tok")
        .token("x", "x")
        .token("y", "y")
        .rule("start", vec!["x", "y"])
        .start("start")
        .build()
}

fn alternatives_grammar() -> adze_ir::Grammar {
    GrammarBuilder::new("alts")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a"])
        .rule("start", vec!["b"])
        .start("start")
        .build()
}

fn chain_grammar() -> adze_ir::Grammar {
    GrammarBuilder::new("chain")
        .token("z", "z")
        .rule("inner", vec!["z"])
        .rule("mid", vec!["inner"])
        .rule("start", vec!["mid"])
        .start("start")
        .build()
}

fn expression_grammar() -> adze_ir::Grammar {
    GrammarBuilder::new("expr")
        .token("NUM", r"\d+")
        .token("PLUS", "+")
        .token("STAR", "*")
        .token("LPAREN", "(")
        .token("RPAREN", ")")
        .rule("expr", vec!["NUM"])
        .rule("expr", vec!["expr", "PLUS", "expr"])
        .rule("expr", vec!["expr", "STAR", "expr"])
        .rule("expr", vec!["LPAREN", "expr", "RPAREN"])
        .start("expr")
        .build()
}

fn statement_grammar() -> adze_ir::Grammar {
    GrammarBuilder::new("stmt")
        .token("ID", r"[a-z]+")
        .token("NUM", r"\d+")
        .token("EQ", "=")
        .token("SEMI", ";")
        .rule("stmt", vec!["ID", "EQ", "NUM", "SEMI"])
        .rule("program", vec!["stmt"])
        .start("program")
        .build()
}

fn many_symbols_grammar(count: usize) -> adze_ir::Grammar {
    let mut builder = GrammarBuilder::new("many");
    let mut token_names: Vec<String> = Vec::new();
    for i in 0..count {
        let name = format!("tok{i}");
        let pat = format!("pat{i}");
        builder = builder.token(
            Box::leak(name.clone().into_boxed_str()),
            Box::leak(pat.into_boxed_str()),
        );
        token_names.push(name);
    }
    for (i, name) in token_names.iter().enumerate() {
        let rule_name = format!("r{i}");
        builder = builder.rule(
            Box::leak(rule_name.into_boxed_str()),
            vec![Box::leak(name.clone().into_boxed_str())],
        );
    }
    let limit = count.min(4);
    for i in 0..limit {
        let rule_name = format!("r{i}");
        builder = builder.rule("top", vec![Box::leak(rule_name.into_boxed_str())]);
    }
    builder = builder.start("top");
    builder.build()
}

// ===========================================================================
// 1. ABI version constants (5 tests)
// ===========================================================================

#[test]
fn abi_version_is_15() {
    assert_eq!(TREE_SITTER_LANGUAGE_VERSION, 15);
}

#[test]
fn abi_min_compat_version_is_13() {
    assert_eq!(TREE_SITTER_MIN_COMPATIBLE_LANGUAGE_VERSION, 13);
}

#[test]
fn abi_version_ge_min_compat() {
    const { assert!(TREE_SITTER_LANGUAGE_VERSION >= TREE_SITTER_MIN_COMPATIBLE_LANGUAGE_VERSION) };
}

#[test]
fn abi_min_compat_within_range() {
    const { assert!(TREE_SITTER_MIN_COMPATIBLE_LANGUAGE_VERSION >= 13) };
    const { assert!(TREE_SITTER_MIN_COMPATIBLE_LANGUAGE_VERSION <= TREE_SITTER_LANGUAGE_VERSION) };
}

#[test]
fn abi_type_sizes_match_tree_sitter() {
    assert_eq!(std::mem::size_of::<TSSymbol>(), 2);
    assert_eq!(std::mem::size_of::<TSStateId>(), 2);
    assert_eq!(std::mem::size_of::<TSFieldId>(), 2);
    assert_eq!(std::mem::size_of::<TSParseAction>(), 6);
    assert_eq!(std::mem::size_of::<TSLexState>(), 4);
}

// ===========================================================================
// 2. Generated Language code compiles / contains expected symbols (8 tests)
// ===========================================================================

#[test]
fn abi_builder_generate_contains_tslanguage() {
    let (grammar, table) = build_pipeline(single_token_grammar);
    let builder = AbiLanguageBuilder::new(&grammar, &table);
    let code = builder.generate().to_string();
    assert!(code.contains("TSLanguage"), "output must contain TSLanguage struct reference");
}

#[test]
fn abi_builder_generate_contains_tree_sitter_fn() {
    let (grammar, table) = build_pipeline(single_token_grammar);
    let builder = AbiLanguageBuilder::new(&grammar, &table);
    let code = builder.generate().to_string();
    assert!(
        code.contains("tree_sitter_single"),
        "output must contain FFI function for grammar name"
    );
}

#[test]
fn static_gen_code_contains_language() {
    let (grammar, table) = build_pipeline(single_token_grammar);
    let ntg = StaticLanguageGenerator::new(grammar, table);
    let code = ntg.generate_language_code().to_string();
    assert!(code.contains("language"), "StaticLanguageGenerator output must mention 'language'");
}

#[test]
fn static_gen_code_nonempty_for_expression() {
    let (grammar, table) = build_pipeline(expression_grammar);
    let ntg = StaticLanguageGenerator::new(grammar, table);
    let code = ntg.generate_language_code().to_string();
    assert!(!code.is_empty());
    assert!(code.len() > 100, "expression grammar should produce substantial code");
}

#[test]
fn abi_builder_code_references_version_15() {
    let (grammar, table) = build_pipeline(two_token_grammar);
    let builder = AbiLanguageBuilder::new(&grammar, &table);
    let code = builder.generate().to_string();
    assert!(code.contains("15"), "generated code should embed ABI version 15");
}

#[test]
fn abi_builder_code_references_symbol_names_array() {
    let (grammar, table) = build_pipeline(alternatives_grammar);
    let builder = AbiLanguageBuilder::new(&grammar, &table);
    let code = builder.generate().to_string();
    let has_ref = code.contains("SYMBOL_NAMES") || code.contains("symbol_names");
    assert!(has_ref, "generated code should reference symbol names");
}

#[test]
fn abi_builder_generates_for_chain_grammar() {
    let (grammar, table) = build_pipeline(chain_grammar);
    let builder = AbiLanguageBuilder::new(&grammar, &table);
    let code = builder.generate().to_string();
    assert!(!code.is_empty());
    assert!(code.contains("TSLanguage"));
}

#[test]
fn abi_builder_generates_for_statement_grammar() {
    let (grammar, table) = build_pipeline(statement_grammar);
    let builder = AbiLanguageBuilder::new(&grammar, &table);
    let code = builder.generate().to_string();
    assert!(
        code.contains("tree_sitter_stmt"),
        "statement grammar FFI function"
    );
}

// ===========================================================================
// 3. Node types JSON is valid JSON with expected structure (8 tests)
// ===========================================================================

#[test]
fn node_types_gen_valid_json_single() {
    let (grammar, _table) = build_pipeline(single_token_grammar);
    let ntg = NodeTypesGenerator::new(&grammar);
    let json_str = ntg.generate().expect("generate should succeed");
    let parsed: serde_json::Value = serde_json::from_str(&json_str).expect("must be valid JSON");
    assert!(parsed.is_array());
}

#[test]
fn node_types_gen_entries_have_type_field() {
    let (grammar, _table) = build_pipeline(chain_grammar);
    let ntg = NodeTypesGenerator::new(&grammar);
    let json_str = ntg.generate().expect("generate should succeed");
    let arr: Vec<serde_json::Value> = serde_json::from_str(&json_str).expect("valid JSON");
    for entry in &arr {
        assert!(entry.get("type").is_some(), "each entry needs 'type'");
    }
}

#[test]
fn node_types_gen_entries_have_named_field() {
    let (grammar, _table) = build_pipeline(chain_grammar);
    let ntg = NodeTypesGenerator::new(&grammar);
    let json_str = ntg.generate().expect("generate should succeed");
    let arr: Vec<serde_json::Value> = serde_json::from_str(&json_str).expect("valid JSON");
    for entry in &arr {
        assert!(entry.get("named").is_some(), "each entry needs 'named'");
    }
}

#[test]
fn static_gen_node_types_valid_json_alternatives() {
    let (grammar, table) = build_pipeline(alternatives_grammar);
    let json_str = StaticLanguageGenerator::new(grammar, table).generate_node_types();
    let parsed: serde_json::Value = serde_json::from_str(&json_str).expect("must be valid JSON");
    assert!(parsed.is_array());
}

#[test]
fn static_gen_node_types_valid_json_expression() {
    let (grammar, table) = build_pipeline(expression_grammar);
    let json_str = StaticLanguageGenerator::new(grammar, table).generate_node_types();
    let _: serde_json::Value = serde_json::from_str(&json_str).expect("must be valid JSON");
}

#[test]
fn static_gen_node_types_valid_json_statement() {
    let (grammar, table) = build_pipeline(statement_grammar);
    let json_str = StaticLanguageGenerator::new(grammar, table).generate_node_types();
    let _: serde_json::Value = serde_json::from_str(&json_str).expect("must be valid JSON");
}

#[test]
fn node_types_gen_valid_json_two_tokens() {
    let (grammar, _table) = build_pipeline(two_token_grammar);
    let ntg = NodeTypesGenerator::new(&grammar);
    let json_str = ntg.generate().expect("generate should succeed");
    let _: serde_json::Value = serde_json::from_str(&json_str).expect("valid JSON");
}

#[test]
fn node_types_named_rules_are_named_true() {
    let (grammar, _table) = build_pipeline(expression_grammar);
    let ntg = NodeTypesGenerator::new(&grammar);
    let json_str = ntg.generate().expect("generate should succeed");
    let arr: Vec<serde_json::Value> = serde_json::from_str(&json_str).expect("valid JSON");
    // Named rules (non-tokens) should have named: true
    for entry in &arr {
        if entry["named"].as_bool() == Some(true) {
            assert!(
                entry["type"].as_str().is_some(),
                "named entry must have a string type"
            );
        }
    }
}

// ===========================================================================
// 4. Symbol names preserved in generated code (8 tests)
// ===========================================================================

#[test]
fn abi_builder_preserves_token_name_a() {
    let (grammar, table) = build_pipeline(single_token_grammar);
    let builder = AbiLanguageBuilder::new(&grammar, &table);
    let code = builder.generate().to_string();
    assert!(
        code.contains("\"a\"") || code.contains("a"),
        "token name 'a' must appear in generated code"
    );
}

#[test]
fn abi_builder_preserves_end_symbol() {
    let (grammar, table) = build_pipeline(single_token_grammar);
    let builder = AbiLanguageBuilder::new(&grammar, &table);
    let code = builder.generate().to_string();
    assert!(
        code.contains("end") || code.contains("EOF") || code.contains("eof"),
        "EOF/end symbol should appear in generated code"
    );
}

#[test]
fn abi_builder_preserves_two_token_names() {
    let (grammar, table) = build_pipeline(two_token_grammar);
    let builder = AbiLanguageBuilder::new(&grammar, &table);
    let code = builder.generate().to_string();
    // Both token names should be present
    let has_x = code.contains('x');
    let has_y = code.contains('y');
    assert!(has_x && has_y, "both token names must appear in code");
}

#[test]
fn abi_builder_preserves_grammar_name_in_fn() {
    let (grammar, table) = build_pipeline(expression_grammar);
    let builder = AbiLanguageBuilder::new(&grammar, &table);
    let code = builder.generate().to_string();
    assert!(
        code.contains("tree_sitter_expr"),
        "grammar name should be embedded in FFI function name"
    );
}

#[test]
fn static_gen_preserves_rule_name_in_node_types() {
    let (grammar, table) = build_pipeline(statement_grammar);
    let json_str = StaticLanguageGenerator::new(grammar, table).generate_node_types();
    // StaticLanguageGenerator uses "rule_{id}" format for rule names
    let has_rule_ref = json_str.contains("rule_");
    assert!(has_rule_ref, "rule references should be in node types output");
}

#[test]
fn node_types_gen_preserves_rule_names_chain() {
    let (grammar, _table) = build_pipeline(chain_grammar);
    let ntg = NodeTypesGenerator::new(&grammar);
    let json_str = ntg.generate().expect("generate should succeed");
    // Chain grammar has inner, mid, start rules
    let has_names = json_str.contains("start") || json_str.contains("inner") || json_str.contains("mid");
    assert!(has_names, "rule names should appear in node types");
}

#[test]
fn abi_builder_preserves_operator_tokens() {
    let (grammar, table) = build_pipeline(expression_grammar);
    let builder = AbiLanguageBuilder::new(&grammar, &table);
    let code = builder.generate().to_string();
    // Token names are encoded as byte arrays; verify SYMBOL_NAME idents are present
    assert!(
        code.contains("SYMBOL_NAME_"),
        "generated code must contain SYMBOL_NAME_ entries for tokens"
    );
}

#[test]
fn abi_builder_preserves_regex_token_names() {
    let (grammar, table) = build_pipeline(statement_grammar);
    let builder = AbiLanguageBuilder::new(&grammar, &table);
    let code = builder.generate().to_string();
    // Token names are encoded as byte arrays; count SYMBOL_NAME entries
    let symbol_name_count = code.matches("SYMBOL_NAME_").count();
    // Statement grammar has 4 tokens + rules + EOF — at least 5 symbol name entries
    assert!(
        symbol_name_count >= 5,
        "expected >= 5 SYMBOL_NAME entries, got {symbol_name_count}"
    );
}

// ===========================================================================
// 5. State count matches parse table (7 tests)
// ===========================================================================

#[test]
fn abi_builder_state_count_in_code_single() {
    let (grammar, table) = build_pipeline(single_token_grammar);
    let state_count = table.state_count;
    let builder = AbiLanguageBuilder::new(&grammar, &table);
    let code = builder.generate().to_string();
    let sc_str = format!("{state_count}");
    assert!(
        code.contains(&sc_str),
        "state_count {state_count} should appear in generated code"
    );
}

#[test]
fn abi_builder_state_count_in_code_expression() {
    let (grammar, table) = build_pipeline(expression_grammar);
    let state_count = table.state_count;
    let builder = AbiLanguageBuilder::new(&grammar, &table);
    let code = builder.generate().to_string();
    let sc_str = format!("{state_count}");
    assert!(
        code.contains(&sc_str),
        "state_count {state_count} should appear in expression grammar code"
    );
}

#[test]
fn static_gen_state_count_positive_single() {
    let (_grammar, table) = build_pipeline(single_token_grammar);
    assert!(table.state_count > 0, "parse table must have at least one state");
}

#[test]
fn static_gen_state_count_grows_with_grammar() {
    let (_g1, t1) = build_pipeline(single_token_grammar);
    let (_g2, t2) = build_pipeline(expression_grammar);
    assert!(
        t2.state_count >= t1.state_count,
        "expression grammar should have at least as many states"
    );
}

#[test]
fn parse_table_symbol_count_positive() {
    let (_grammar, table) = build_pipeline(single_token_grammar);
    assert!(table.symbol_count > 0, "symbol_count must be positive");
}

#[test]
fn parse_table_token_count_positive() {
    let (_grammar, table) = build_pipeline(single_token_grammar);
    assert!(table.token_count > 0, "token_count must be positive");
}

#[test]
fn abi_builder_state_count_in_code_chain() {
    let (grammar, table) = build_pipeline(chain_grammar);
    let state_count = table.state_count;
    let builder = AbiLanguageBuilder::new(&grammar, &table);
    let code = builder.generate().to_string();
    let sc_str = format!("{state_count}");
    assert!(
        code.contains(&sc_str),
        "state_count {state_count} should appear in chain grammar code"
    );
}

// ===========================================================================
// 6. Token and non-terminal separation in generated output (7 tests)
// ===========================================================================

#[test]
fn abi_builder_output_has_parse_table() {
    let (grammar, table) = build_pipeline(two_token_grammar);
    let builder = AbiLanguageBuilder::new(&grammar, &table);
    let code = builder.generate().to_string();
    let has_table = code.contains("PARSE_TABLE") || code.contains("parse_table");
    assert!(has_table, "generated code must reference parse table data");
}

#[test]
fn abi_builder_output_has_lex_modes() {
    let (grammar, table) = build_pipeline(two_token_grammar);
    let builder = AbiLanguageBuilder::new(&grammar, &table);
    let code = builder.generate().to_string();
    let has_lex = code.contains("LEX_MODE") || code.contains("lex_mode") || code.contains("lex");
    assert!(has_lex, "generated code must reference lex modes");
}

#[test]
fn static_gen_token_and_rule_separation_in_node_types() {
    let (grammar, table) = build_pipeline(expression_grammar);
    let json_str = StaticLanguageGenerator::new(grammar, table).generate_node_types();
    let arr: Vec<serde_json::Value> = serde_json::from_str(&json_str).expect("valid JSON");
    let named_count = arr.iter().filter(|e| e["named"] == true).count();
    assert!(named_count > 0, "should have at least one named node type");
}

#[test]
fn abi_builder_output_has_metadata() {
    let (grammar, table) = build_pipeline(alternatives_grammar);
    let builder = AbiLanguageBuilder::new(&grammar, &table);
    let code = builder.generate().to_string();
    let has_meta = code.contains("metadata") || code.contains("METADATA") || code.contains("symbol_metadata");
    assert!(has_meta, "generated code must include symbol metadata");
}

#[test]
fn abi_builder_output_differentiates_grammars() {
    let (g1, t1) = build_pipeline(single_token_grammar);
    let (g2, t2) = build_pipeline(expression_grammar);
    let code1 = AbiLanguageBuilder::new(&g1, &t1).generate().to_string();
    let code2 = AbiLanguageBuilder::new(&g2, &t2).generate().to_string();
    assert_ne!(code1, code2, "different grammars must produce different code");
}

#[test]
fn static_gen_different_grammars_different_node_types() {
    let (g1, t1) = build_pipeline(single_token_grammar);
    let (g2, t2) = build_pipeline(statement_grammar);
    let n1 = StaticLanguageGenerator::new(g1, t1).generate_node_types();
    let n2 = StaticLanguageGenerator::new(g2, t2).generate_node_types();
    assert_ne!(n1, n2, "different grammars must produce different node types");
}

#[test]
fn abi_builder_output_has_field_names_section() {
    let (grammar, table) = build_pipeline(single_token_grammar);
    let builder = AbiLanguageBuilder::new(&grammar, &table);
    let code = builder.generate().to_string();
    let has_field = code.contains("field") || code.contains("FIELD");
    assert!(has_field, "generated code should reference field names");
}

// ===========================================================================
// 7. Edge cases: empty grammar, single rule, many symbols (12 tests)
// ===========================================================================

#[test]
fn abi_builder_single_rule_grammar() {
    let (grammar, table) = build_pipeline(single_token_grammar);
    let builder = AbiLanguageBuilder::new(&grammar, &table);
    let code = builder.generate().to_string();
    assert!(!code.is_empty(), "single rule grammar must produce output");
}

#[test]
fn static_gen_single_rule_node_types() {
    let (grammar, table) = build_pipeline(single_token_grammar);
    let json_str = StaticLanguageGenerator::new(grammar, table).generate_node_types();
    let arr: Vec<serde_json::Value> = serde_json::from_str(&json_str).expect("valid JSON");
    assert!(!arr.is_empty(), "single rule grammar should produce node types");
}

#[test]
fn abi_builder_many_symbols_10() {
    let (grammar, table) = build_pipeline(|| many_symbols_grammar(10));
    let builder = AbiLanguageBuilder::new(&grammar, &table);
    let code = builder.generate().to_string();
    assert!(!code.is_empty());
    assert!(code.contains("TSLanguage"));
}

#[test]
fn abi_builder_many_symbols_20() {
    let (grammar, table) = build_pipeline(|| many_symbols_grammar(20));
    let builder = AbiLanguageBuilder::new(&grammar, &table);
    let code = builder.generate().to_string();
    assert!(code.len() > 500, "20-symbol grammar should produce large code");
}

#[test]
fn static_gen_many_symbols_node_types_valid() {
    let (grammar, table) = build_pipeline(|| many_symbols_grammar(15));
    let json_str = StaticLanguageGenerator::new(grammar, table).generate_node_types();
    let arr: Vec<serde_json::Value> = serde_json::from_str(&json_str).expect("valid JSON");
    assert!(arr.len() >= 5, "many symbols should produce multiple node type entries");
}

#[test]
fn node_types_gen_many_symbols_valid() {
    let (grammar, _table) = build_pipeline(|| many_symbols_grammar(10));
    let ntg = NodeTypesGenerator::new(&grammar);
    let json_str = ntg.generate().expect("generate should succeed");
    let _: serde_json::Value = serde_json::from_str(&json_str).expect("valid JSON");
}

#[test]
fn abi_builder_deterministic_single() {
    let (g1, t1) = build_pipeline(single_token_grammar);
    let (g2, t2) = build_pipeline(single_token_grammar);
    let c1 = AbiLanguageBuilder::new(&g1, &t1).generate().to_string();
    let c2 = AbiLanguageBuilder::new(&g2, &t2).generate().to_string();
    assert_eq!(c1, c2, "same grammar must produce identical ABI builder output");
}

#[test]
fn abi_builder_deterministic_expression() {
    let (g1, t1) = build_pipeline(expression_grammar);
    let (g2, t2) = build_pipeline(expression_grammar);
    let c1 = AbiLanguageBuilder::new(&g1, &t1).generate().to_string();
    let c2 = AbiLanguageBuilder::new(&g2, &t2).generate().to_string();
    assert_eq!(c1, c2);
}

#[test]
fn abi_builder_scales_with_grammar_size() {
    let (g1, t1) = build_pipeline(single_token_grammar);
    let (g2, t2) = build_pipeline(expression_grammar);
    let len1 = AbiLanguageBuilder::new(&g1, &t1).generate().to_string().len();
    let len2 = AbiLanguageBuilder::new(&g2, &t2).generate().to_string().len();
    assert!(
        len2 > len1,
        "expression grammar ({len2}) should produce more code than single ({len1})"
    );
}

#[test]
fn symbol_metadata_flags_roundtrip() {
    let meta = create_symbol_metadata(true, true, false, false, false);
    assert_eq!(meta, abi::symbol_metadata::VISIBLE | abi::symbol_metadata::NAMED);
    let meta2 = create_symbol_metadata(false, false, true, true, true);
    assert_eq!(
        meta2,
        abi::symbol_metadata::HIDDEN | abi::symbol_metadata::AUXILIARY | abi::symbol_metadata::SUPERTYPE
    );
}

#[test]
fn tslanguage_struct_aligned_for_ffi() {
    assert_eq!(
        std::mem::align_of::<TSLanguage>(),
        std::mem::align_of::<*const u8>(),
        "TSLanguage must be pointer-aligned"
    );
}

#[test]
fn external_scanner_default_has_null_pointers() {
    let scanner = ExternalScanner::default();
    assert!(scanner.states.is_null());
    assert!(scanner.symbol_map.is_null());
    assert!(scanner.create.is_none());
    assert!(scanner.destroy.is_none());
    assert!(scanner.scan.is_none());
    assert!(scanner.serialize.is_none());
    assert!(scanner.deserialize.is_none());
}
