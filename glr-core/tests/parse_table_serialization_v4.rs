//! ParseTable serialization v4 comprehensive tests.
//!
//! Covers roundtrip fidelity, byte format properties, determinism,
//! error handling, and complex grammars.
//!
//! Run with:
//!   cargo test -p adze-glr-core --test parse_table_serialization_v4 --features serialization -- --test-threads=2

#![cfg(feature = "serialization")]

use adze_glr_core::serialization::{DeserializationError, PARSE_TABLE_FORMAT_VERSION};
use adze_glr_core::{FirstFollowSets, ParseTable, build_lr1_automaton};
use adze_ir::builder::GrammarBuilder;
use adze_ir::*;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn build_table(grammar: &Grammar) -> ParseTable {
    let ff = FirstFollowSets::compute(grammar).expect("FIRST/FOLLOW");
    build_lr1_automaton(grammar, &ff).expect("automaton construction")
}

fn roundtrip(table: &ParseTable) -> ParseTable {
    let bytes = table.to_bytes().expect("serialize");
    ParseTable::from_bytes(&bytes).expect("deserialize")
}

fn craft_versioned_bytes(version: u32, inner_data: &[u8]) -> Vec<u8> {
    #[derive(serde::Serialize)]
    struct Wrapper {
        version: u32,
        data: Vec<u8>,
    }
    let w = Wrapper {
        version,
        data: inner_data.to_vec(),
    };
    postcard::to_stdvec(&w).expect("encode wrapper")
}

fn replace_version_in(valid_bytes: &[u8], new_version: u32) -> Vec<u8> {
    #[derive(serde::Deserialize)]
    struct WrapperDe {
        #[allow(dead_code)]
        version: u32,
        data: Vec<u8>,
    }
    #[derive(serde::Serialize)]
    struct WrapperSe {
        version: u32,
        data: Vec<u8>,
    }
    let original: WrapperDe = postcard::from_bytes(valid_bytes).expect("decode");
    let replaced = WrapperSe {
        version: new_version,
        data: original.data,
    };
    postcard::to_stdvec(&replaced).expect("re-encode")
}

// ---------------------------------------------------------------------------
// Grammar factories
// ---------------------------------------------------------------------------

/// S -> a
fn single_token_grammar() -> Grammar {
    GrammarBuilder::new("single_tok")
        .token("a", "a")
        .rule("S", vec!["a"])
        .start("S")
        .build()
}

/// S -> a | b
fn two_alt_grammar() -> Grammar {
    GrammarBuilder::new("two_alt")
        .token("a", "a")
        .token("b", "b")
        .rule("S", vec!["a"])
        .rule("S", vec!["b"])
        .start("S")
        .build()
}

/// S -> A, A -> a
fn chain_grammar() -> Grammar {
    GrammarBuilder::new("chain")
        .token("a", "a")
        .rule("S", vec!["A"])
        .rule("A", vec!["a"])
        .start("S")
        .build()
}

/// E -> E + E | n (ambiguous)
fn expr_grammar() -> Grammar {
    GrammarBuilder::new("expr")
        .token("n", r"\d+")
        .token("+", r"\+")
        .rule("E", vec!["E", "+", "E"])
        .rule("E", vec!["n"])
        .start("E")
        .build()
}

/// S -> a b
fn seq_grammar() -> Grammar {
    GrammarBuilder::new("seq")
        .token("a", "a")
        .token("b", "b")
        .rule("S", vec!["a", "b"])
        .start("S")
        .build()
}

/// S -> L R, L -> a, R -> b
fn two_nt_grammar() -> Grammar {
    GrammarBuilder::new("two_nt")
        .token("a", "a")
        .token("b", "b")
        .rule("S", vec!["L", "R"])
        .rule("L", vec!["a"])
        .rule("R", vec!["b"])
        .start("S")
        .build()
}

/// list -> a list | a (right-recursive)
fn right_recursive_grammar() -> Grammar {
    GrammarBuilder::new("right_rec")
        .token("a", "a")
        .rule("list", vec!["a", "list"])
        .rule("list", vec!["a"])
        .start("list")
        .build()
}

/// S -> a b c
fn triple_seq_grammar() -> Grammar {
    GrammarBuilder::new("triple_seq")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("S", vec!["a", "b", "c"])
        .start("S")
        .build()
}

/// E -> E + T | T, T -> T * F | F, F -> n | ( E )
fn arithmetic_grammar() -> Grammar {
    GrammarBuilder::new("arith")
        .token("n", r"\d+")
        .token("+", r"\+")
        .token("*", r"\*")
        .token("(", r"\(")
        .token(")", r"\)")
        .rule("E", vec!["E", "+", "T"])
        .rule("E", vec!["T"])
        .rule("T", vec!["T", "*", "F"])
        .rule("T", vec!["F"])
        .rule("F", vec!["n"])
        .rule("F", vec!["(", "E", ")"])
        .start("E")
        .build()
}

/// S -> a | b | c | d | e
fn five_alt_grammar() -> Grammar {
    GrammarBuilder::new("five_alt")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .token("d", "d")
        .token("e", "e")
        .rule("S", vec!["a"])
        .rule("S", vec!["b"])
        .rule("S", vec!["c"])
        .rule("S", vec!["d"])
        .rule("S", vec!["e"])
        .start("S")
        .build()
}

/// S -> A B C, A -> a, B -> b, C -> c
fn three_nt_chain_grammar() -> Grammar {
    GrammarBuilder::new("three_nt_chain")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("S", vec!["A", "B", "C"])
        .rule("A", vec!["a"])
        .rule("B", vec!["b"])
        .rule("C", vec!["c"])
        .start("S")
        .build()
}

/// S -> S a | a (left-recursive)
fn left_recursive_grammar() -> Grammar {
    GrammarBuilder::new("left_rec")
        .token("a", "a")
        .rule("S", vec!["S", "a"])
        .rule("S", vec!["a"])
        .start("S")
        .build()
}

/// S -> a S b | epsilon (nested parens style)
fn nested_grammar() -> Grammar {
    GrammarBuilder::new("nested")
        .token("a", "a")
        .token("b", "b")
        .rule("S", vec!["a", "S", "b"])
        .rule("S", vec![])
        .start("S")
        .build()
}

// ===================================================================
// 1. Roundtrip serialization — full field fidelity (10 tests)
// ===================================================================

#[test]
fn roundtrip_single_token_all_fields() {
    let table = build_table(&single_token_grammar());
    let restored = roundtrip(&table);
    assert_eq!(table.state_count, restored.state_count);
    assert_eq!(table.symbol_count, restored.symbol_count);
    assert_eq!(table.eof_symbol, restored.eof_symbol);
    assert_eq!(table.start_symbol, restored.start_symbol);
    assert_eq!(table.token_count, restored.token_count);
    assert_eq!(table.initial_state, restored.initial_state);
}

#[test]
fn roundtrip_two_alt_action_table() {
    let table = build_table(&two_alt_grammar());
    let restored = roundtrip(&table);
    assert_eq!(table.action_table, restored.action_table);
}

#[test]
fn roundtrip_chain_goto_table() {
    let table = build_table(&chain_grammar());
    let restored = roundtrip(&table);
    assert_eq!(table.goto_table, restored.goto_table);
}

#[test]
fn roundtrip_expr_preserves_rules() {
    let table = build_table(&expr_grammar());
    let restored = roundtrip(&table);
    assert_eq!(table.rules.len(), restored.rules.len());
    for (a, b) in table.rules.iter().zip(restored.rules.iter()) {
        assert_eq!(a.lhs, b.lhs);
        assert_eq!(a.rhs_len, b.rhs_len);
    }
}

#[test]
fn roundtrip_seq_symbol_to_index() {
    let table = build_table(&seq_grammar());
    let restored = roundtrip(&table);
    assert_eq!(table.symbol_to_index, restored.symbol_to_index);
    assert_eq!(table.index_to_symbol, restored.index_to_symbol);
}

#[test]
fn roundtrip_two_nt_nonterminal_map() {
    let table = build_table(&two_nt_grammar());
    let restored = roundtrip(&table);
    assert_eq!(table.nonterminal_to_index, restored.nonterminal_to_index);
    assert_eq!(table.goto_indexing, restored.goto_indexing);
}

#[test]
fn roundtrip_right_recursive_extras_and_lex() {
    let table = build_table(&right_recursive_grammar());
    let restored = roundtrip(&table);
    assert_eq!(table.extras, restored.extras);
    assert_eq!(table.lex_modes.len(), restored.lex_modes.len());
}

#[test]
fn roundtrip_triple_seq_field_names() {
    let table = build_table(&triple_seq_grammar());
    let restored = roundtrip(&table);
    assert_eq!(table.field_names, restored.field_names);
    assert_eq!(table.field_map, restored.field_map);
}

#[test]
fn roundtrip_arithmetic_complex() {
    let table = build_table(&arithmetic_grammar());
    let restored = roundtrip(&table);
    assert_eq!(table.state_count, restored.state_count);
    assert_eq!(table.symbol_count, restored.symbol_count);
    assert_eq!(table.action_table, restored.action_table);
    assert_eq!(table.goto_table, restored.goto_table);
}

#[test]
fn roundtrip_five_alt_all_actions() {
    let g = five_alt_grammar();
    let table = build_table(&g);
    let restored = roundtrip(&table);
    for st in 0..table.state_count {
        let sid = StateId(st as u16);
        for (&sym, _) in &g.tokens {
            assert_eq!(
                table.actions(sid, sym),
                restored.actions(sid, sym),
                "state {st} sym {sym:?}"
            );
        }
    }
}

// ===================================================================
// 2. Byte format properties (8 tests)
// ===================================================================

#[test]
fn bytes_are_non_empty() {
    let table = build_table(&single_token_grammar());
    let bytes = table.to_bytes().expect("serialize");
    assert!(!bytes.is_empty());
}

#[test]
fn larger_grammar_produces_larger_bytes() {
    let small = build_table(&single_token_grammar())
        .to_bytes()
        .expect("small");
    let large = build_table(&arithmetic_grammar())
        .to_bytes()
        .expect("large");
    assert!(
        large.len() > small.len(),
        "arithmetic ({}) should be larger than single token ({})",
        large.len(),
        small.len()
    );
}

#[test]
fn double_roundtrip_yields_identical_bytes() {
    let table = build_table(&expr_grammar());
    let bytes1 = table.to_bytes().expect("first");
    let table2 = ParseTable::from_bytes(&bytes1).expect("deserialize");
    let bytes2 = table2.to_bytes().expect("second");
    assert_eq!(bytes1, bytes2);
}

#[test]
fn triple_roundtrip_stable() {
    let table = build_table(&arithmetic_grammar());
    let b1 = table.to_bytes().expect("r1");
    let t2 = ParseTable::from_bytes(&b1).expect("d1");
    let b2 = t2.to_bytes().expect("r2");
    let t3 = ParseTable::from_bytes(&b2).expect("d2");
    let b3 = t3.to_bytes().expect("r3");
    assert_eq!(b1, b2);
    assert_eq!(b2, b3);
}

#[test]
fn format_version_is_positive() {
    assert_ne!(PARSE_TABLE_FORMAT_VERSION, 0);
}

#[test]
fn format_version_equals_two() {
    assert_eq!(PARSE_TABLE_FORMAT_VERSION, 2);
}

#[test]
fn serialized_size_bounded() {
    let table = build_table(&arithmetic_grammar());
    let bytes = table.to_bytes().expect("serialize");
    // A reasonable grammar should serialize to < 100KB
    assert!(
        bytes.len() < 100_000,
        "serialized size {} exceeds 100KB",
        bytes.len()
    );
}

#[test]
fn different_grammars_different_bytes() {
    let b1 = build_table(&single_token_grammar()).to_bytes().expect("s1");
    let b2 = build_table(&expr_grammar()).to_bytes().expect("s2");
    assert_ne!(b1, b2);
}

// ===================================================================
// 3. Determinism (8 tests)
// ===================================================================

fn assert_deterministic(grammar: &Grammar) {
    let table = build_table(grammar);
    let bytes1 = table.to_bytes().expect("first");
    let bytes2 = table.to_bytes().expect("second");
    assert_eq!(bytes1, bytes2, "serialization must be deterministic");
}

#[test]
fn determinism_single_token() {
    assert_deterministic(&single_token_grammar());
}

#[test]
fn determinism_two_alt() {
    assert_deterministic(&two_alt_grammar());
}

#[test]
fn determinism_chain() {
    assert_deterministic(&chain_grammar());
}

#[test]
fn determinism_expr() {
    assert_deterministic(&expr_grammar());
}

#[test]
fn determinism_arithmetic() {
    assert_deterministic(&arithmetic_grammar());
}

#[test]
fn determinism_five_alt() {
    assert_deterministic(&five_alt_grammar());
}

#[test]
fn determinism_three_nt_chain() {
    assert_deterministic(&three_nt_chain_grammar());
}

#[test]
fn determinism_left_recursive() {
    assert_deterministic(&left_recursive_grammar());
}

// ===================================================================
// 4. Error handling — invalid bytes (10 tests)
// ===================================================================

#[test]
fn error_empty_bytes() {
    assert!(ParseTable::from_bytes(&[]).is_err());
}

#[test]
fn error_single_byte() {
    assert!(ParseTable::from_bytes(&[0x42]).is_err());
}

#[test]
fn error_two_bytes() {
    assert!(ParseTable::from_bytes(&[0xDE, 0xAD]).is_err());
}

#[test]
fn error_all_zeros() {
    assert!(ParseTable::from_bytes(&[0u8; 32]).is_err());
}

#[test]
fn error_all_ones() {
    assert!(ParseTable::from_bytes(&[0xFF; 64]).is_err());
}

#[test]
fn error_random_garbage() {
    let garbage: Vec<u8> = (0u8..128)
        .map(|i| i.wrapping_mul(37).wrapping_add(13))
        .collect();
    assert!(ParseTable::from_bytes(&garbage).is_err());
}

#[test]
fn error_truncated_valid_bytes() {
    let table = build_table(&single_token_grammar());
    let bytes = table.to_bytes().expect("serialize");
    let half = bytes.len() / 2;
    assert!(ParseTable::from_bytes(&bytes[..half]).is_err());
}

#[test]
fn error_wrong_version() {
    let table = build_table(&single_token_grammar());
    let bytes = table.to_bytes().expect("serialize");
    let tampered = replace_version_in(&bytes, PARSE_TABLE_FORMAT_VERSION + 1);
    let result = ParseTable::from_bytes(&tampered);
    assert!(result.is_err());
    match result.unwrap_err() {
        DeserializationError::IncompatibleVersion { expected, actual } => {
            assert_eq!(expected, PARSE_TABLE_FORMAT_VERSION);
            assert_eq!(actual, PARSE_TABLE_FORMAT_VERSION + 1);
        }
        other => panic!("expected IncompatibleVersion, got: {other}"),
    }
}

#[test]
fn error_version_zero() {
    let table = build_table(&single_token_grammar());
    let bytes = table.to_bytes().expect("serialize");
    let tampered = replace_version_in(&bytes, 0);
    assert!(ParseTable::from_bytes(&tampered).is_err());
}

#[test]
fn error_valid_version_corrupted_data() {
    let tampered = craft_versioned_bytes(PARSE_TABLE_FORMAT_VERSION, &[0xFF; 50]);
    assert!(ParseTable::from_bytes(&tampered).is_err());
}

// ===================================================================
// 5. Error handling — edge cases (5 tests)
// ===================================================================

#[test]
fn error_empty_inner_data() {
    let tampered = craft_versioned_bytes(PARSE_TABLE_FORMAT_VERSION, &[]);
    assert!(ParseTable::from_bytes(&tampered).is_err());
}

#[test]
fn error_version_u32_max() {
    let table = build_table(&single_token_grammar());
    let bytes = table.to_bytes().expect("serialize");
    let tampered = replace_version_in(&bytes, u32::MAX);
    assert!(ParseTable::from_bytes(&tampered).is_err());
}

#[test]
fn error_corrupted_middle_no_panic() {
    let table = build_table(&expr_grammar());
    let mut bytes = table.to_bytes().expect("serialize");
    if bytes.len() > 8 {
        let mid = bytes.len() / 2;
        bytes[mid] ^= 0xFF;
        bytes[mid + 1] ^= 0xAA;
        bytes[mid + 2] ^= 0x55;
    }
    // Must not panic — may succeed or fail
    let _ = ParseTable::from_bytes(&bytes);
}

#[test]
fn error_trailing_junk_no_panic() {
    let table = build_table(&single_token_grammar());
    let mut bytes = table.to_bytes().expect("serialize");
    bytes.extend_from_slice(&[0xDE, 0xAD, 0xBE, 0xEF]);
    // postcard may or may not tolerate trailing data — no panic is the contract
    let _ = ParseTable::from_bytes(&bytes);
}

#[test]
fn error_one_byte_short() {
    let table = build_table(&single_token_grammar());
    let bytes = table.to_bytes().expect("serialize");
    assert!(ParseTable::from_bytes(&bytes[..bytes.len() - 1]).is_err());
}

// ===================================================================
// 6. State count preservation across grammars (6 tests)
// ===================================================================

#[test]
fn state_count_preserved_arithmetic() {
    let table = build_table(&arithmetic_grammar());
    assert_eq!(table.state_count, roundtrip(&table).state_count);
}

#[test]
fn state_count_preserved_five_alt() {
    let table = build_table(&five_alt_grammar());
    assert_eq!(table.state_count, roundtrip(&table).state_count);
}

#[test]
fn state_count_preserved_three_nt_chain() {
    let table = build_table(&three_nt_chain_grammar());
    assert_eq!(table.state_count, roundtrip(&table).state_count);
}

#[test]
fn state_count_preserved_left_recursive() {
    let table = build_table(&left_recursive_grammar());
    assert_eq!(table.state_count, roundtrip(&table).state_count);
}

#[test]
fn state_count_preserved_nested() {
    let table = build_table(&nested_grammar());
    assert_eq!(table.state_count, roundtrip(&table).state_count);
}

#[test]
fn state_count_nonzero_after_roundtrip() {
    let table = build_table(&arithmetic_grammar());
    let restored = roundtrip(&table);
    assert!(restored.state_count > 0);
}

// ===================================================================
// 7. Goto table preservation (5 tests)
// ===================================================================

fn assert_all_gotos_match(original: &ParseTable, restored: &ParseTable, grammar: &Grammar) {
    for st in 0..original.state_count {
        let sid = StateId(st as u16);
        for nt_sym in grammar.rule_names.keys() {
            assert_eq!(
                original.goto(sid, *nt_sym),
                restored.goto(sid, *nt_sym),
                "goto mismatch state {st} nt {nt_sym:?}"
            );
        }
    }
}

#[test]
fn goto_preserved_arithmetic() {
    let g = arithmetic_grammar();
    let table = build_table(&g);
    let restored = roundtrip(&table);
    assert_all_gotos_match(&table, &restored, &g);
}

#[test]
fn goto_preserved_three_nt_chain() {
    let g = three_nt_chain_grammar();
    let table = build_table(&g);
    let restored = roundtrip(&table);
    assert_all_gotos_match(&table, &restored, &g);
}

#[test]
fn goto_preserved_nested() {
    let g = nested_grammar();
    let table = build_table(&g);
    let restored = roundtrip(&table);
    assert_all_gotos_match(&table, &restored, &g);
}

#[test]
fn goto_preserved_left_recursive() {
    let g = left_recursive_grammar();
    let table = build_table(&g);
    let restored = roundtrip(&table);
    assert_all_gotos_match(&table, &restored, &g);
}

#[test]
fn goto_preserved_chain() {
    let g = chain_grammar();
    let table = build_table(&g);
    let restored = roundtrip(&table);
    assert_all_gotos_match(&table, &restored, &g);
}

// ===================================================================
// 8. Complex grammars (4 tests)
// ===================================================================

/// S -> A B | C, A -> a, B -> b, C -> c
fn overlapping_grammar() -> Grammar {
    GrammarBuilder::new("overlapping")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("S", vec!["A", "B"])
        .rule("S", vec!["C"])
        .rule("A", vec!["a"])
        .rule("B", vec!["b"])
        .rule("C", vec!["c"])
        .start("S")
        .build()
}

/// S -> A, A -> B, B -> C, C -> x
fn deep_chain_grammar() -> Grammar {
    GrammarBuilder::new("deep_chain")
        .token("x", "x")
        .rule("S", vec!["A"])
        .rule("A", vec!["B"])
        .rule("B", vec!["C"])
        .rule("C", vec!["x"])
        .start("S")
        .build()
}

/// stmts -> stmt stmts | stmt, stmt -> ID = NUM ;
fn statement_list_grammar() -> Grammar {
    GrammarBuilder::new("stmt_list")
        .token("ID", r"[a-z]+")
        .token("NUM", r"\d+")
        .token("=", "=")
        .token(";", ";")
        .rule("stmts", vec!["stmt", "stmts"])
        .rule("stmts", vec!["stmt"])
        .rule("stmt", vec!["ID", "=", "NUM", ";"])
        .start("stmts")
        .build()
}

#[test]
fn roundtrip_overlapping_complex() {
    let g = overlapping_grammar();
    let table = build_table(&g);
    let restored = roundtrip(&table);
    assert_eq!(table.state_count, restored.state_count);
    assert_eq!(table.action_table, restored.action_table);
    assert_eq!(table.goto_table, restored.goto_table);
}

#[test]
fn roundtrip_deep_chain() {
    let table = build_table(&deep_chain_grammar());
    let restored = roundtrip(&table);
    assert_eq!(table.state_count, restored.state_count);
    assert_eq!(table.rules.len(), restored.rules.len());
}

#[test]
fn roundtrip_statement_list() {
    let g = statement_list_grammar();
    let table = build_table(&g);
    let restored = roundtrip(&table);
    assert_eq!(table.state_count, restored.state_count);
    assert_eq!(table.symbol_count, restored.symbol_count);
    assert_all_gotos_match(&table, &restored, &g);
}

#[test]
fn complex_grammar_deterministic() {
    let g = statement_list_grammar();
    let table = build_table(&g);
    let b1 = table.to_bytes().expect("s1");
    let b2 = table.to_bytes().expect("s2");
    assert_eq!(b1, b2);
}

// ===================================================================
// 9. Symbol metadata preservation (3 tests)
// ===================================================================

#[test]
fn symbol_metadata_length_preserved() {
    let table = build_table(&arithmetic_grammar());
    let restored = roundtrip(&table);
    assert_eq!(table.symbol_metadata.len(), restored.symbol_metadata.len());
}

#[test]
fn symbol_metadata_fields_preserved() {
    let table = build_table(&two_nt_grammar());
    let restored = roundtrip(&table);
    for (a, b) in table
        .symbol_metadata
        .iter()
        .zip(restored.symbol_metadata.iter())
    {
        assert_eq!(a.name, b.name);
        assert_eq!(a.is_visible, b.is_visible);
        assert_eq!(a.is_named, b.is_named);
        assert_eq!(a.is_terminal, b.is_terminal);
        assert_eq!(a.symbol_id, b.symbol_id);
    }
}

#[test]
fn symbol_metadata_empty_for_simple() {
    let table = build_table(&single_token_grammar());
    let restored = roundtrip(&table);
    // Metadata length must be identical whether populated or empty
    assert_eq!(table.symbol_metadata.len(), restored.symbol_metadata.len());
}

// ===================================================================
// 10. Dynamic precedence and associativity (3 tests)
// ===================================================================

#[test]
fn dynamic_prec_preserved() {
    let table = build_table(&expr_grammar());
    let restored = roundtrip(&table);
    assert_eq!(table.dynamic_prec_by_rule, restored.dynamic_prec_by_rule);
}

#[test]
fn rule_assoc_preserved() {
    let table = build_table(&expr_grammar());
    let restored = roundtrip(&table);
    assert_eq!(table.rule_assoc_by_rule, restored.rule_assoc_by_rule);
}

#[test]
fn alias_sequences_preserved() {
    let table = build_table(&arithmetic_grammar());
    let restored = roundtrip(&table);
    assert_eq!(table.alias_sequences, restored.alias_sequences);
}

// ===================================================================
// 11. External scanner states (2 tests)
// ===================================================================

#[test]
fn external_scanner_states_preserved() {
    let table = build_table(&single_token_grammar());
    let restored = roundtrip(&table);
    assert_eq!(
        table.external_scanner_states,
        restored.external_scanner_states
    );
}

#[test]
fn external_token_count_preserved() {
    let table = build_table(&arithmetic_grammar());
    let restored = roundtrip(&table);
    assert_eq!(table.external_token_count, restored.external_token_count);
}
