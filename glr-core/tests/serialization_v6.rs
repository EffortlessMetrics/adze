//! ParseTable serialization v6 comprehensive tests.
//!
//! Covers roundtrip fidelity, byte format properties, format version,
//! invalid input handling, grammar differentiation, state count preservation,
//! action preservation, and edge cases.
//!
//! Run with:
//!   cargo test -p adze-glr-core --test serialization_v6 --features serialization -- --test-threads=2

#![cfg(feature = "serialization")]

use adze_glr_core::serialization::{DeserializationError, PARSE_TABLE_FORMAT_VERSION};
use adze_glr_core::{Action, FirstFollowSets, ParseTable, StateId, build_lr1_automaton};
use adze_ir::builder::GrammarBuilder;
use adze_ir::{Grammar, RuleId};

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

/// Build a minimal ParseTable with a single cell containing the given actions.
fn make_action_table(actions: Vec<Action>) -> ParseTable {
    use std::collections::BTreeMap;
    let mut sym_to_idx = BTreeMap::new();
    sym_to_idx.insert(adze_glr_core::SymbolId(0), 0);
    ParseTable {
        action_table: vec![vec![actions]],
        state_count: 1,
        symbol_count: 1,
        index_to_symbol: vec![adze_glr_core::SymbolId(0)],
        symbol_to_index: sym_to_idx,
        ..Default::default()
    }
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

/// S -> a S b | epsilon (nested)
fn nested_grammar() -> Grammar {
    GrammarBuilder::new("nested")
        .token("a", "a")
        .token("b", "b")
        .rule("S", vec!["a", "S", "b"])
        .rule("S", vec![])
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

// ===================================================================
// 1. Roundtrip: to_bytes → from_bytes produces equal table (8 tests)
// ===================================================================

#[test]
fn roundtrip_single_token_fields() {
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
fn roundtrip_seq_symbol_maps() {
    let table = build_table(&seq_grammar());
    let restored = roundtrip(&table);
    assert_eq!(table.symbol_to_index, restored.symbol_to_index);
    assert_eq!(table.index_to_symbol, restored.index_to_symbol);
}

#[test]
fn roundtrip_two_nt_nonterminal_index() {
    let table = build_table(&two_nt_grammar());
    let restored = roundtrip(&table);
    assert_eq!(table.nonterminal_to_index, restored.nonterminal_to_index);
    assert_eq!(table.goto_indexing, restored.goto_indexing);
}

#[test]
fn roundtrip_right_recursive_extras() {
    let table = build_table(&right_recursive_grammar());
    let restored = roundtrip(&table);
    assert_eq!(table.extras, restored.extras);
    assert_eq!(table.lex_modes.len(), restored.lex_modes.len());
}

#[test]
fn roundtrip_arithmetic_full() {
    let table = build_table(&arithmetic_grammar());
    let restored = roundtrip(&table);
    assert_eq!(table.state_count, restored.state_count);
    assert_eq!(table.symbol_count, restored.symbol_count);
    assert_eq!(table.action_table, restored.action_table);
    assert_eq!(table.goto_table, restored.goto_table);
    assert_eq!(table.rules.len(), restored.rules.len());
    assert_eq!(table.field_names, restored.field_names);
    assert_eq!(table.field_map, restored.field_map);
}

// ===================================================================
// 2. Byte output is non-empty (8 tests)
// ===================================================================

#[test]
fn bytes_nonempty_single_token() {
    let bytes = build_table(&single_token_grammar()).to_bytes().expect("s");
    assert!(!bytes.is_empty());
}

#[test]
fn bytes_nonempty_two_alt() {
    let bytes = build_table(&two_alt_grammar()).to_bytes().expect("s");
    assert!(!bytes.is_empty());
}

#[test]
fn bytes_nonempty_chain() {
    let bytes = build_table(&chain_grammar()).to_bytes().expect("s");
    assert!(!bytes.is_empty());
}

#[test]
fn bytes_nonempty_expr() {
    let bytes = build_table(&expr_grammar()).to_bytes().expect("s");
    assert!(!bytes.is_empty());
}

#[test]
fn bytes_nonempty_arithmetic() {
    let bytes = build_table(&arithmetic_grammar()).to_bytes().expect("s");
    assert!(!bytes.is_empty());
}

#[test]
fn bytes_nonempty_five_alt() {
    let bytes = build_table(&five_alt_grammar()).to_bytes().expect("s");
    assert!(!bytes.is_empty());
}

#[test]
fn bytes_nonempty_left_recursive() {
    let bytes = build_table(&left_recursive_grammar())
        .to_bytes()
        .expect("s");
    assert!(!bytes.is_empty());
}

#[test]
fn bytes_nonempty_nested() {
    let bytes = build_table(&nested_grammar()).to_bytes().expect("s");
    assert!(!bytes.is_empty());
}

// ===================================================================
// 3. Format version present in bytes (8 tests)
// ===================================================================

#[test]
fn format_version_constant_is_two() {
    assert_eq!(PARSE_TABLE_FORMAT_VERSION, 2);
}

#[test]
fn format_version_nonzero() {
    assert_ne!(PARSE_TABLE_FORMAT_VERSION, 0);
}

#[test]
fn version_mismatch_rejected_future() {
    let table = build_table(&single_token_grammar());
    let bytes = table.to_bytes().expect("serialize");
    let tampered = replace_version_in(&bytes, PARSE_TABLE_FORMAT_VERSION + 1);
    let err = ParseTable::from_bytes(&tampered).unwrap_err();
    match err {
        DeserializationError::IncompatibleVersion { expected, actual } => {
            assert_eq!(expected, PARSE_TABLE_FORMAT_VERSION);
            assert_eq!(actual, PARSE_TABLE_FORMAT_VERSION + 1);
        }
        other => panic!("expected IncompatibleVersion, got: {other}"),
    }
}

#[test]
fn version_mismatch_rejected_past() {
    let table = build_table(&chain_grammar());
    let bytes = table.to_bytes().expect("serialize");
    let tampered = replace_version_in(&bytes, 1);
    assert!(ParseTable::from_bytes(&tampered).is_err());
}

#[test]
fn version_zero_rejected() {
    let table = build_table(&expr_grammar());
    let bytes = table.to_bytes().expect("serialize");
    let tampered = replace_version_in(&bytes, 0);
    assert!(ParseTable::from_bytes(&tampered).is_err());
}

#[test]
fn version_u32_max_rejected() {
    let table = build_table(&two_alt_grammar());
    let bytes = table.to_bytes().expect("serialize");
    let tampered = replace_version_in(&bytes, u32::MAX);
    assert!(ParseTable::from_bytes(&tampered).is_err());
}

#[test]
fn version_three_rejected() {
    let table = build_table(&seq_grammar());
    let bytes = table.to_bytes().expect("serialize");
    let tampered = replace_version_in(&bytes, 3);
    assert!(ParseTable::from_bytes(&tampered).is_err());
}

#[test]
fn version_preserved_through_double_roundtrip() {
    let table = build_table(&arithmetic_grammar());
    let b1 = table.to_bytes().expect("r1");
    let t2 = ParseTable::from_bytes(&b1).expect("d1");
    let b2 = t2.to_bytes().expect("r2");
    let t3 = ParseTable::from_bytes(&b2).expect("d2");
    let b3 = t3.to_bytes().expect("r3");
    assert_eq!(b1, b2);
    assert_eq!(b2, b3);
}

// ===================================================================
// 4. Invalid bytes produce errors (8 tests)
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
fn error_all_zeros_32() {
    assert!(ParseTable::from_bytes(&[0u8; 32]).is_err());
}

#[test]
fn error_all_ones_64() {
    assert!(ParseTable::from_bytes(&[0xFF; 64]).is_err());
}

#[test]
fn error_pseudorandom_garbage() {
    let garbage: Vec<u8> = (0u8..128)
        .map(|i| i.wrapping_mul(37).wrapping_add(13))
        .collect();
    assert!(ParseTable::from_bytes(&garbage).is_err());
}

#[test]
fn error_truncated_valid() {
    let table = build_table(&single_token_grammar());
    let bytes = table.to_bytes().expect("serialize");
    let half = bytes.len() / 2;
    assert!(ParseTable::from_bytes(&bytes[..half]).is_err());
}

#[test]
fn error_valid_version_garbage_data() {
    let tampered = craft_versioned_bytes(PARSE_TABLE_FORMAT_VERSION, &[0xFF; 50]);
    assert!(ParseTable::from_bytes(&tampered).is_err());
}

// ===================================================================
// 5. Different grammars produce different bytes (8 tests)
// ===================================================================

#[test]
fn diff_single_vs_two_alt() {
    let b1 = build_table(&single_token_grammar()).to_bytes().expect("s1");
    let b2 = build_table(&two_alt_grammar()).to_bytes().expect("s2");
    assert_ne!(b1, b2);
}

#[test]
fn diff_chain_vs_expr() {
    let b1 = build_table(&chain_grammar()).to_bytes().expect("s1");
    let b2 = build_table(&expr_grammar()).to_bytes().expect("s2");
    assert_ne!(b1, b2);
}

#[test]
fn diff_seq_vs_two_nt() {
    let b1 = build_table(&seq_grammar()).to_bytes().expect("s1");
    let b2 = build_table(&two_nt_grammar()).to_bytes().expect("s2");
    assert_ne!(b1, b2);
}

#[test]
fn diff_arithmetic_vs_five_alt() {
    let b1 = build_table(&arithmetic_grammar()).to_bytes().expect("s1");
    let b2 = build_table(&five_alt_grammar()).to_bytes().expect("s2");
    assert_ne!(b1, b2);
}

#[test]
fn diff_left_recursive_vs_right_recursive() {
    let b1 = build_table(&left_recursive_grammar())
        .to_bytes()
        .expect("s1");
    let b2 = build_table(&right_recursive_grammar())
        .to_bytes()
        .expect("s2");
    assert_ne!(b1, b2);
}

#[test]
fn diff_nested_vs_chain() {
    let b1 = build_table(&nested_grammar()).to_bytes().expect("s1");
    let b2 = build_table(&chain_grammar()).to_bytes().expect("s2");
    assert_ne!(b1, b2);
}

#[test]
fn diff_three_nt_vs_stmt_list() {
    let b1 = build_table(&three_nt_chain_grammar())
        .to_bytes()
        .expect("s1");
    let b2 = build_table(&statement_list_grammar())
        .to_bytes()
        .expect("s2");
    assert_ne!(b1, b2);
}

#[test]
fn diff_same_grammar_same_bytes() {
    let b1 = build_table(&arithmetic_grammar()).to_bytes().expect("s1");
    let b2 = build_table(&arithmetic_grammar()).to_bytes().expect("s2");
    assert_eq!(b1, b2, "same grammar must produce same bytes");
}

// ===================================================================
// 6. State count preserved through serialization (8 tests)
// ===================================================================

#[test]
fn state_count_preserved_single_token() {
    let table = build_table(&single_token_grammar());
    assert_eq!(table.state_count, roundtrip(&table).state_count);
}

#[test]
fn state_count_preserved_two_alt() {
    let table = build_table(&two_alt_grammar());
    assert_eq!(table.state_count, roundtrip(&table).state_count);
}

#[test]
fn state_count_preserved_chain() {
    let table = build_table(&chain_grammar());
    assert_eq!(table.state_count, roundtrip(&table).state_count);
}

#[test]
fn state_count_preserved_expr() {
    let table = build_table(&expr_grammar());
    assert_eq!(table.state_count, roundtrip(&table).state_count);
}

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
fn state_count_preserved_left_recursive() {
    let table = build_table(&left_recursive_grammar());
    assert_eq!(table.state_count, roundtrip(&table).state_count);
}

#[test]
fn state_count_nonzero_after_roundtrip() {
    let table = build_table(&arithmetic_grammar());
    let restored = roundtrip(&table);
    assert!(restored.state_count > 0);
}

// ===================================================================
// 7. Actions preserved through serialization (8 tests)
// ===================================================================

#[test]
fn actions_preserved_single_token() {
    let table = build_table(&single_token_grammar());
    let restored = roundtrip(&table);
    assert_eq!(table.action_table, restored.action_table);
}

#[test]
fn actions_preserved_two_alt() {
    let table = build_table(&two_alt_grammar());
    let restored = roundtrip(&table);
    assert_eq!(table.action_table, restored.action_table);
}

#[test]
fn actions_preserved_arithmetic() {
    let table = build_table(&arithmetic_grammar());
    let restored = roundtrip(&table);
    assert_eq!(table.action_table, restored.action_table);
}

#[test]
fn actions_preserved_five_alt_per_state() {
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

#[test]
fn actions_shift_variant_preserved() {
    let table = make_action_table(vec![Action::Shift(StateId(7))]);
    let restored = roundtrip(&table);
    assert_eq!(restored.action_table[0][0], vec![Action::Shift(StateId(7))]);
}

#[test]
fn actions_reduce_variant_preserved() {
    let table = make_action_table(vec![Action::Reduce(RuleId(3))]);
    let restored = roundtrip(&table);
    assert_eq!(restored.action_table[0][0], vec![Action::Reduce(RuleId(3))]);
}

#[test]
fn actions_accept_variant_preserved() {
    let table = make_action_table(vec![Action::Accept]);
    let restored = roundtrip(&table);
    assert_eq!(restored.action_table[0][0], vec![Action::Accept]);
}

#[test]
fn actions_error_variant_preserved() {
    let table = make_action_table(vec![Action::Error]);
    let restored = roundtrip(&table);
    assert_eq!(restored.action_table[0][0], vec![Action::Error]);
}

// ===================================================================
// 8. Edge cases: minimal, large, corrupted (8 tests)
// ===================================================================

#[test]
fn edge_default_table_roundtrips() {
    let table = ParseTable::default();
    let bytes = table.to_bytes().expect("serialize");
    let restored = ParseTable::from_bytes(&bytes).expect("deserialize");
    assert_eq!(table.state_count, restored.state_count);
    assert_eq!(table.symbol_count, restored.symbol_count);
}

#[test]
fn edge_statement_list_complex() {
    let g = statement_list_grammar();
    let table = build_table(&g);
    let restored = roundtrip(&table);
    assert_eq!(table.state_count, restored.state_count);
    assert_eq!(table.symbol_count, restored.symbol_count);
    assert_eq!(table.action_table, restored.action_table);
    assert_eq!(table.goto_table, restored.goto_table);
}

#[test]
fn edge_nested_epsilon_grammar() {
    let table = build_table(&nested_grammar());
    let restored = roundtrip(&table);
    assert_eq!(table.state_count, restored.state_count);
    assert_eq!(table.rules.len(), restored.rules.len());
}

#[test]
fn edge_corrupted_middle_no_panic() {
    let table = build_table(&arithmetic_grammar());
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
fn edge_trailing_junk_no_panic() {
    let table = build_table(&single_token_grammar());
    let mut bytes = table.to_bytes().expect("serialize");
    bytes.extend_from_slice(&[0xDE, 0xAD, 0xBE, 0xEF]);
    // postcard may or may not tolerate trailing data — no panic is the contract
    let _ = ParseTable::from_bytes(&bytes);
}

#[test]
fn edge_one_byte_short() {
    let table = build_table(&chain_grammar());
    let bytes = table.to_bytes().expect("serialize");
    assert!(ParseTable::from_bytes(&bytes[..bytes.len() - 1]).is_err());
}

#[test]
fn edge_serialized_size_bounded() {
    let table = build_table(&arithmetic_grammar());
    let bytes = table.to_bytes().expect("serialize");
    assert!(
        bytes.len() < 100_000,
        "serialized size {} exceeds 100KB",
        bytes.len()
    );
}

#[test]
fn edge_empty_inner_data_rejected() {
    let tampered = craft_versioned_bytes(PARSE_TABLE_FORMAT_VERSION, &[]);
    assert!(ParseTable::from_bytes(&tampered).is_err());
}
