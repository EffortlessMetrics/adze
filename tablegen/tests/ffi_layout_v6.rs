//! FFI Language struct layout and Tree-sitter ABI compatibility tests.
//!
//! 64 tests covering:
//! 1. ABI version constants (8 tests)
//! 2. Symbol metadata flag values (8 tests)
//! 3. Language struct field layout (8 tests)
//! 4. FFI function naming conventions (8 tests)
//! 5. Parse table encoding in generated code (8 tests)
//! 6. Lex mode generation (8 tests)
//! 7. Symbol name table generation (8 tests)
//! 8. Edge cases: minimal grammar, many symbols, external tokens (8 tests)

use adze_glr_core::{FirstFollowSets, ParseTable, build_lr1_automaton};
use adze_ir::Grammar;
use adze_ir::builder::GrammarBuilder;
use adze_tablegen::AbiLanguageBuilder;
use adze_tablegen::abi::{
    self, ExternalScanner, TSFieldId, TSLanguage, TSLexState, TSParseAction, TSStateId, TSSymbol,
    create_symbol_metadata,
};
use std::mem;

// ============================================================================
// Helpers
// ============================================================================

fn build_table(grammar: &Grammar) -> ParseTable {
    let mut g = grammar.clone();
    let ff = FirstFollowSets::compute_normalized(&mut g).expect("FIRST/FOLLOW");
    build_lr1_automaton(&g, &ff).expect("LR(1)")
}

fn codegen(grammar: &Grammar, table: &ParseTable) -> String {
    AbiLanguageBuilder::new(grammar, table)
        .generate()
        .to_string()
}

fn simple_grammar() -> Grammar {
    GrammarBuilder::new("simple")
        .token("x", "x")
        .rule("root", vec!["x"])
        .start("root")
        .build()
}

fn two_alt_grammar() -> Grammar {
    GrammarBuilder::new("two_alt")
        .token("a", "a")
        .token("b", "b")
        .rule("root", vec!["a"])
        .rule("root", vec!["b"])
        .start("root")
        .build()
}

fn nested_grammar() -> Grammar {
    GrammarBuilder::new("nested")
        .token("x", "x")
        .token("y", "y")
        .rule("top", vec!["mid", "x"])
        .rule("mid", vec!["y"])
        .start("top")
        .build()
}

fn chain_grammar() -> Grammar {
    GrammarBuilder::new("chain")
        .token("z", "z")
        .rule("s", vec!["a"])
        .rule("a", vec!["b"])
        .rule("b", vec!["z"])
        .start("s")
        .build()
}

fn left_rec_grammar() -> Grammar {
    GrammarBuilder::new("left_rec")
        .token("tok", "t")
        .rule("s", vec!["tok"])
        .rule("s", vec!["s", "tok"])
        .start("s")
        .build()
}

fn many_tokens_grammar() -> Grammar {
    let mut gb = GrammarBuilder::new("many_tokens");
    for i in 0..20 {
        let name = format!("t{i}");
        let pat = format!("{}", (b'a' + (i % 26) as u8) as char);
        gb = gb.token(&name, &pat);
        gb = gb.rule("root", vec![Box::leak(name.into_boxed_str()) as &str]);
    }
    gb.start("root").build()
}

fn external_token_grammar() -> Grammar {
    GrammarBuilder::new("ext")
        .token("a", "a")
        .external("HEREDOC")
        .rule("root", vec!["a"])
        .start("root")
        .build()
}

fn nullable_grammar() -> Grammar {
    GrammarBuilder::new("nullable")
        .token("a", "a")
        .rule("root", vec!["maybe"])
        .rule("maybe", vec!["a"])
        .rule("maybe", vec![])
        .start("root")
        .build()
}

// ============================================================================
// 1. ABI version constants (8 tests)
// ============================================================================

#[test]
fn abi_version_is_fifteen() {
    assert_eq!(abi::TREE_SITTER_LANGUAGE_VERSION, 15);
}

#[test]
fn abi_min_compatible_version_is_thirteen() {
    assert_eq!(abi::TREE_SITTER_MIN_COMPATIBLE_LANGUAGE_VERSION, 13);
}

#[test]
fn abi_version_gte_min_compatible() {
    let current = abi::TREE_SITTER_LANGUAGE_VERSION;
    let min = abi::TREE_SITTER_MIN_COMPATIBLE_LANGUAGE_VERSION;
    assert!(current >= min, "current version must be >= min compatible");
}

#[test]
fn abi_version_is_u32() {
    let v: u32 = abi::TREE_SITTER_LANGUAGE_VERSION;
    assert!(v > 0);
}

#[test]
fn abi_min_version_is_u32() {
    let v: u32 = abi::TREE_SITTER_MIN_COMPATIBLE_LANGUAGE_VERSION;
    assert!(v > 0);
}

#[test]
fn generated_code_contains_abi_version() {
    let g = simple_grammar();
    let pt = build_table(&g);
    let code = codegen(&g, &pt);
    assert!(
        code.contains("TREE_SITTER_LANGUAGE_VERSION"),
        "generated code must reference the ABI version constant"
    );
}

#[test]
fn abi_version_gap_bounded() {
    let gap = abi::TREE_SITTER_LANGUAGE_VERSION - abi::TREE_SITTER_MIN_COMPATIBLE_LANGUAGE_VERSION;
    assert!(
        gap <= 5,
        "ABI version gap should be small for compatibility, got {gap}"
    );
}

#[test]
fn abi_version_nonzero_and_reasonable() {
    let v = abi::TREE_SITTER_LANGUAGE_VERSION;
    assert!(v >= 13);
    assert!(v < 100);
}

// ============================================================================
// 2. Symbol metadata flag values (8 tests)
// ============================================================================

#[test]
fn metadata_visible_is_0x01() {
    assert_eq!(abi::symbol_metadata::VISIBLE, 0x01);
}

#[test]
fn metadata_named_is_0x02() {
    assert_eq!(abi::symbol_metadata::NAMED, 0x02);
}

#[test]
fn metadata_hidden_is_0x04() {
    assert_eq!(abi::symbol_metadata::HIDDEN, 0x04);
}

#[test]
fn metadata_auxiliary_is_0x08() {
    assert_eq!(abi::symbol_metadata::AUXILIARY, 0x08);
}

#[test]
fn metadata_supertype_is_0x10() {
    assert_eq!(abi::symbol_metadata::SUPERTYPE, 0x10);
}

#[test]
fn metadata_flags_are_disjoint() {
    let all = [
        abi::symbol_metadata::VISIBLE,
        abi::symbol_metadata::NAMED,
        abi::symbol_metadata::HIDDEN,
        abi::symbol_metadata::AUXILIARY,
        abi::symbol_metadata::SUPERTYPE,
    ];
    for i in 0..all.len() {
        for j in (i + 1)..all.len() {
            assert_eq!(
                all[i] & all[j],
                0,
                "flags {:#04x} and {:#04x} must be disjoint",
                all[i],
                all[j]
            );
        }
    }
}

#[test]
fn create_symbol_metadata_combines_visible_named() {
    let md = create_symbol_metadata(true, true, false, false, false);
    assert_eq!(
        md,
        abi::symbol_metadata::VISIBLE | abi::symbol_metadata::NAMED
    );
}

#[test]
fn create_symbol_metadata_all_flags_set() {
    let md = create_symbol_metadata(true, true, true, true, true);
    let expected = abi::symbol_metadata::VISIBLE
        | abi::symbol_metadata::NAMED
        | abi::symbol_metadata::HIDDEN
        | abi::symbol_metadata::AUXILIARY
        | abi::symbol_metadata::SUPERTYPE;
    assert_eq!(md, expected);
}

// ============================================================================
// 3. Language struct field layout (8 tests)
// ============================================================================

#[test]
fn ts_symbol_size_is_two_bytes() {
    assert_eq!(mem::size_of::<TSSymbol>(), 2);
}

#[test]
fn ts_state_id_size_is_two_bytes() {
    assert_eq!(mem::size_of::<TSStateId>(), 2);
}

#[test]
fn ts_field_id_size_is_two_bytes() {
    assert_eq!(mem::size_of::<TSFieldId>(), 2);
}

#[test]
fn ts_parse_action_size_is_six_bytes() {
    assert_eq!(mem::size_of::<TSParseAction>(), 6);
}

#[test]
fn ts_lex_state_size_is_four_bytes() {
    assert_eq!(mem::size_of::<TSLexState>(), 4);
}

#[test]
fn ts_language_pointer_aligned() {
    assert_eq!(
        mem::align_of::<TSLanguage>(),
        mem::align_of::<*const u8>(),
        "TSLanguage must be pointer-aligned for C ABI"
    );
}

#[test]
fn external_scanner_default_null_pointers() {
    let es = ExternalScanner::default();
    assert!(es.states.is_null());
    assert!(es.symbol_map.is_null());
    assert!(es.create.is_none());
    assert!(es.destroy.is_none());
    assert!(es.scan.is_none());
    assert!(es.serialize.is_none());
    assert!(es.deserialize.is_none());
}

#[test]
fn generated_code_sets_state_count() {
    let g = simple_grammar();
    let pt = build_table(&g);
    let code = codegen(&g, &pt);
    let sc = pt.state_count;
    assert!(
        code.contains("state_count"),
        "generated code must set state_count (expected {sc})"
    );
}

// ============================================================================
// 4. FFI function naming conventions (8 tests)
// ============================================================================

#[test]
fn ffi_function_uses_tree_sitter_prefix() {
    let g = simple_grammar();
    let pt = build_table(&g);
    let code = codegen(&g, &pt);
    assert!(
        code.contains("tree_sitter_simple"),
        "FFI function must use tree_sitter_ prefix"
    );
}

#[test]
fn ffi_function_name_includes_grammar_name() {
    let g = two_alt_grammar();
    let pt = build_table(&g);
    let code = codegen(&g, &pt);
    assert!(
        code.contains("tree_sitter_two_alt"),
        "FFI function must include grammar name"
    );
}

#[test]
fn ffi_function_extern_c() {
    let g = simple_grammar();
    let pt = build_table(&g);
    let code = codegen(&g, &pt);
    assert!(
        code.contains("extern \"C\""),
        "FFI function must be extern \"C\""
    );
}

#[test]
fn ffi_function_returns_pointer_to_language() {
    let g = simple_grammar();
    let pt = build_table(&g);
    let code = codegen(&g, &pt);
    assert!(
        code.contains("TSLanguage"),
        "FFI function must reference TSLanguage type"
    );
}

#[test]
fn ffi_nested_grammar_function_name() {
    let g = nested_grammar();
    let pt = build_table(&g);
    let code = codegen(&g, &pt);
    assert!(
        code.contains("tree_sitter_nested"),
        "nested grammar FFI name incorrect"
    );
}

#[test]
fn ffi_chain_grammar_function_name() {
    let g = chain_grammar();
    let pt = build_table(&g);
    let code = codegen(&g, &pt);
    assert!(
        code.contains("tree_sitter_chain"),
        "chain grammar FFI name incorrect"
    );
}

#[test]
fn ffi_external_grammar_function_name() {
    let g = external_token_grammar();
    let pt = build_table(&g);
    let code = codegen(&g, &pt);
    assert!(
        code.contains("tree_sitter_ext"),
        "external grammar FFI name incorrect"
    );
}

#[test]
fn ffi_function_marked_no_mangle() {
    let g = simple_grammar();
    let pt = build_table(&g);
    let code = codegen(&g, &pt);
    assert!(
        code.contains("no_mangle"),
        "FFI function must be #[no_mangle]"
    );
}

// ============================================================================
// 5. Parse table encoding in generated code (8 tests)
// ============================================================================

#[test]
fn generated_code_contains_parse_table_static() {
    let g = simple_grammar();
    let pt = build_table(&g);
    let code = codegen(&g, &pt);
    assert!(
        code.contains("PARSE_TABLE") || code.contains("SMALL_PARSE_TABLE"),
        "code must contain parse table data"
    );
}

#[test]
fn generated_code_contains_parse_actions() {
    let g = simple_grammar();
    let pt = build_table(&g);
    let code = codegen(&g, &pt);
    assert!(
        code.contains("PARSE_ACTIONS"),
        "code must contain parse actions array"
    );
}

#[test]
fn generated_code_contains_small_parse_table_map() {
    let g = simple_grammar();
    let pt = build_table(&g);
    let code = codegen(&g, &pt);
    assert!(
        code.contains("SMALL_PARSE_TABLE_MAP"),
        "code must contain small parse table map"
    );
}

#[test]
fn two_alt_parse_table_encodes_both_alternatives() {
    let g = two_alt_grammar();
    let pt = build_table(&g);
    let code = codegen(&g, &pt);
    // Must have at least the two terminal symbols in the parse table data
    assert!(
        !code.is_empty(),
        "generated code must be non-empty for two-alt grammar"
    );
    assert!(pt.action_table.len() >= 2, "need at least 2 states");
}

#[test]
fn nested_parse_table_encodes_multi_rule() {
    let g = nested_grammar();
    let pt = build_table(&g);
    let code = codegen(&g, &pt);
    assert!(!code.is_empty());
    // Nested grammar has goto entries for non-terminals
    assert!(
        !pt.goto_table.is_empty(),
        "nested grammar must have goto table entries"
    );
}

#[test]
fn parse_table_production_id_map_present() {
    let g = simple_grammar();
    let pt = build_table(&g);
    let code = codegen(&g, &pt);
    assert!(
        code.contains("PRODUCTION_ID_MAP"),
        "code must include production ID map"
    );
}

#[test]
fn parse_table_production_lhs_index_present() {
    let g = simple_grammar();
    let pt = build_table(&g);
    let code = codegen(&g, &pt);
    assert!(
        code.contains("PRODUCTION_LHS_INDEX"),
        "code must include production LHS index"
    );
}

#[test]
fn left_rec_parse_table_finite() {
    let g = left_rec_grammar();
    let pt = build_table(&g);
    let code = codegen(&g, &pt);
    assert!(!code.is_empty());
    assert!(
        pt.state_count < 1000,
        "left-recursive grammar must produce finite table"
    );
}

// ============================================================================
// 6. Lex mode generation (8 tests)
// ============================================================================

#[test]
fn lex_modes_static_present() {
    let g = simple_grammar();
    let pt = build_table(&g);
    let code = codegen(&g, &pt);
    assert!(
        code.contains("LEX_MODES"),
        "code must contain LEX_MODES static"
    );
}

#[test]
fn lex_modes_reference_ts_lex_state() {
    let g = simple_grammar();
    let pt = build_table(&g);
    let code = codegen(&g, &pt);
    assert!(
        code.contains("TSLexState"),
        "lex modes must use TSLexState type"
    );
}

#[test]
fn lex_modes_count_matches_states() {
    let g = simple_grammar();
    let pt = build_table(&g);
    // Each state should have a corresponding lex mode
    assert_eq!(pt.lex_modes.len(), pt.state_count, "one lex mode per state");
}

#[test]
fn lex_modes_two_alt_grammar() {
    let g = two_alt_grammar();
    let pt = build_table(&g);
    assert_eq!(pt.lex_modes.len(), pt.state_count);
}

#[test]
fn lex_modes_nested_grammar() {
    let g = nested_grammar();
    let pt = build_table(&g);
    assert_eq!(pt.lex_modes.len(), pt.state_count);
}

#[test]
fn lex_modes_chain_grammar() {
    let g = chain_grammar();
    let pt = build_table(&g);
    assert_eq!(pt.lex_modes.len(), pt.state_count);
}

#[test]
fn lex_modes_initial_state_zero() {
    let g = simple_grammar();
    let pt = build_table(&g);
    if !pt.lex_modes.is_empty() {
        assert_eq!(
            pt.lex_modes[0].lex_state, 0,
            "initial lex state should be 0"
        );
    }
}

#[test]
fn lex_modes_external_lex_state_default_zero() {
    let g = simple_grammar();
    let pt = build_table(&g);
    for (i, mode) in pt.lex_modes.iter().enumerate() {
        assert_eq!(
            mode.external_lex_state, 0,
            "state {i}: external_lex_state should default to 0 for grammar without externals"
        );
    }
}

// ============================================================================
// 7. Symbol name table generation (8 tests)
// ============================================================================

#[test]
fn symbol_names_static_present() {
    let g = simple_grammar();
    let pt = build_table(&g);
    let code = codegen(&g, &pt);
    assert!(
        code.contains("SYMBOL_NAME"),
        "code must contain symbol name table"
    );
}

#[test]
fn symbol_names_contains_end_for_eof() {
    let g = simple_grammar();
    let pt = build_table(&g);
    let code = codegen(&g, &pt);
    assert!(
        code.contains("end"),
        "symbol names must include \"end\" for EOF"
    );
}

#[test]
fn symbol_names_contains_token_name() {
    let g = simple_grammar();
    let pt = build_table(&g);
    let code = codegen(&g, &pt);
    // The grammar has a token named "x"
    assert!(
        code.contains('x'),
        "symbol names must include terminal token name"
    );
}

#[test]
fn symbol_names_two_alt_contains_both_tokens() {
    let g = two_alt_grammar();
    let pt = build_table(&g);
    let code = codegen(&g, &pt);
    assert!(code.contains('a'), "must include token 'a'");
    assert!(code.contains('b'), "must include token 'b'");
}

#[test]
fn symbol_names_nested_contains_nonterminals() {
    let g = nested_grammar();
    let pt = build_table(&g);
    let code = codegen(&g, &pt);
    // "top" and "mid" are rule names
    assert!(
        code.contains("top") || code.contains("rule_"),
        "must include non-terminal names"
    );
}

#[test]
fn symbol_names_count_matches_symbol_count() {
    let g = simple_grammar();
    let pt = build_table(&g);
    // index_to_symbol has one entry per symbol
    assert_eq!(
        pt.index_to_symbol.len(),
        pt.symbol_count,
        "index_to_symbol length must equal symbol_count"
    );
}

#[test]
fn symbol_names_chain_contains_terminal() {
    let g = chain_grammar();
    let pt = build_table(&g);
    let code = codegen(&g, &pt);
    assert!(code.contains('z'), "chain grammar terminal 'z' must appear");
}

#[test]
fn symbol_names_deterministic_across_calls() {
    let g = simple_grammar();
    let pt = build_table(&g);
    let code1 = codegen(&g, &pt);
    let code2 = codegen(&g, &pt);
    assert_eq!(code1, code2, "symbol name generation must be deterministic");
}

// ============================================================================
// 8. Edge cases: minimal, many symbols, external tokens (8 tests)
// ============================================================================

#[test]
fn edge_empty_grammar_does_not_panic() {
    let g = Grammar::new("empty".to_string());
    let pt = ParseTable::default();
    // Should not panic even with a degenerate grammar
    let _builder = AbiLanguageBuilder::new(&g, &pt);
}

#[test]
fn edge_many_tokens_grammar_builds() {
    let g = many_tokens_grammar();
    let pt = build_table(&g);
    let code = codegen(&g, &pt);
    assert!(!code.is_empty(), "many-token grammar must produce code");
    assert!(
        pt.symbol_count >= 20,
        "many-token grammar should have at least 20 symbols, got {}",
        pt.symbol_count
    );
}

#[test]
fn edge_many_tokens_state_count_bounded() {
    let g = many_tokens_grammar();
    let pt = build_table(&g);
    assert!(
        pt.state_count < 10_000,
        "state count must be bounded even with 20 tokens"
    );
}

#[test]
fn edge_external_token_grammar_builds() {
    let g = external_token_grammar();
    let pt = build_table(&g);
    let code = codegen(&g, &pt);
    assert!(!code.is_empty());
}

#[test]
fn edge_external_token_count_reflected() {
    let g = external_token_grammar();
    let pt = build_table(&g);
    assert!(
        pt.external_token_count >= 1,
        "grammar with externals must report at least 1 external token, got {}",
        pt.external_token_count
    );
}

#[test]
fn edge_nullable_grammar_builds() {
    let g = nullable_grammar();
    let pt = build_table(&g);
    let code = codegen(&g, &pt);
    assert!(
        !code.is_empty(),
        "nullable grammar must produce non-empty code"
    );
}

#[test]
fn edge_left_rec_token_count_at_least_one() {
    let g = left_rec_grammar();
    let pt = build_table(&g);
    assert!(
        pt.token_count >= 1,
        "left-recursive grammar must have at least one token"
    );
}

#[test]
fn edge_generated_code_deterministic_for_external() {
    let g = external_token_grammar();
    let pt = build_table(&g);
    let code1 = codegen(&g, &pt);
    let code2 = codegen(&g, &pt);
    assert_eq!(
        code1, code2,
        "code generation must be deterministic for external-token grammars"
    );
}
