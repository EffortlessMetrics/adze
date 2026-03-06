//! Comprehensive serialization v3 tests for ParseTable.
//!
//! Covers roundtrip fidelity, field preservation, actions/goto integrity,
//! invalid-bytes error handling, and deterministic serialization.
//!
//! Run with:
//!   cargo test -p adze-glr-core --test serialization_v3 --features serialization

#![cfg(feature = "serialization")]

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

/// S -> a
fn simple_grammar() -> Grammar {
    GrammarBuilder::new("simple")
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

/// S -> item, item -> a
fn chain_grammar() -> Grammar {
    GrammarBuilder::new("chain")
        .token("a", "a")
        .rule("S", vec!["item"])
        .rule("item", vec!["a"])
        .start("S")
        .build()
}

/// E -> E + E | n  (ambiguous)
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

/// S -> left right, left -> a, right -> b
fn two_nt_grammar() -> Grammar {
    GrammarBuilder::new("two_nt")
        .token("a", "a")
        .token("b", "b")
        .rule("S", vec!["left", "right"])
        .rule("left", vec!["a"])
        .rule("right", vec!["b"])
        .start("S")
        .build()
}

/// list -> a list | a  (right-recursive)
fn right_recursive_grammar() -> Grammar {
    GrammarBuilder::new("right_rec")
        .token("a", "a")
        .rule("list", vec!["a", "list"])
        .rule("list", vec!["a"])
        .start("list")
        .build()
}

/// S -> a b c  (three-token sequence)
fn triple_seq_grammar() -> Grammar {
    GrammarBuilder::new("triple_seq")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("S", vec!["a", "b", "c"])
        .start("S")
        .build()
}

fn roundtrip(table: &ParseTable) -> ParseTable {
    let bytes = table.to_bytes().expect("serialize");
    ParseTable::from_bytes(&bytes).expect("deserialize")
}

// ===================================================================
// 1. Roundtrip serialization (8 tests)
// ===================================================================

#[test]
fn roundtrip_simple_grammar() {
    let table = build_table(&simple_grammar());
    let restored = roundtrip(&table);
    assert_eq!(table.state_count, restored.state_count);
    assert_eq!(table.symbol_count, restored.symbol_count);
    assert_eq!(table.eof_symbol, restored.eof_symbol);
}

#[test]
fn roundtrip_two_alt_grammar() {
    let table = build_table(&two_alt_grammar());
    let restored = roundtrip(&table);
    assert_eq!(table.state_count, restored.state_count);
    assert_eq!(table.action_table, restored.action_table);
}

#[test]
fn roundtrip_chain_grammar() {
    let table = build_table(&chain_grammar());
    let restored = roundtrip(&table);
    assert_eq!(table.goto_table, restored.goto_table);
    assert_eq!(table.action_table, restored.action_table);
}

#[test]
fn roundtrip_expr_grammar() {
    let table = build_table(&expr_grammar());
    let restored = roundtrip(&table);
    assert_eq!(table.state_count, restored.state_count);
    assert_eq!(table.symbol_count, restored.symbol_count);
}

#[test]
fn roundtrip_seq_grammar() {
    let table = build_table(&seq_grammar());
    let restored = roundtrip(&table);
    assert_eq!(table.action_table, restored.action_table);
    assert_eq!(table.goto_table, restored.goto_table);
}

#[test]
fn roundtrip_two_nt_grammar() {
    let table = build_table(&two_nt_grammar());
    let restored = roundtrip(&table);
    assert_eq!(table.rules.len(), restored.rules.len());
    for (a, b) in table.rules.iter().zip(restored.rules.iter()) {
        assert_eq!(a.lhs, b.lhs);
        assert_eq!(a.rhs_len, b.rhs_len);
    }
    assert_eq!(table.eof_symbol, restored.eof_symbol);
}

#[test]
fn roundtrip_right_recursive_grammar() {
    let table = build_table(&right_recursive_grammar());
    let restored = roundtrip(&table);
    assert_eq!(table.state_count, restored.state_count);
    assert_eq!(table.action_table, restored.action_table);
}

#[test]
fn roundtrip_triple_seq_grammar() {
    let table = build_table(&triple_seq_grammar());
    let restored = roundtrip(&table);
    assert_eq!(table.state_count, restored.state_count);
    assert_eq!(table.goto_table, restored.goto_table);
}

// ===================================================================
// 2. State count preserved (8 tests)
// ===================================================================

#[test]
fn state_count_preserved_simple() {
    let table = build_table(&simple_grammar());
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
fn state_count_preserved_seq() {
    let table = build_table(&seq_grammar());
    assert_eq!(table.state_count, roundtrip(&table).state_count);
}

#[test]
fn state_count_preserved_two_nt() {
    let table = build_table(&two_nt_grammar());
    assert_eq!(table.state_count, roundtrip(&table).state_count);
}

#[test]
fn state_count_preserved_right_recursive() {
    let table = build_table(&right_recursive_grammar());
    assert_eq!(table.state_count, roundtrip(&table).state_count);
}

#[test]
fn state_count_preserved_triple_seq() {
    let table = build_table(&triple_seq_grammar());
    assert_eq!(table.state_count, roundtrip(&table).state_count);
}

// ===================================================================
// 3. Symbol count preserved (8 tests)
// ===================================================================

#[test]
fn symbol_count_preserved_simple() {
    let table = build_table(&simple_grammar());
    assert_eq!(table.symbol_count, roundtrip(&table).symbol_count);
}

#[test]
fn symbol_count_preserved_two_alt() {
    let table = build_table(&two_alt_grammar());
    assert_eq!(table.symbol_count, roundtrip(&table).symbol_count);
}

#[test]
fn symbol_count_preserved_chain() {
    let table = build_table(&chain_grammar());
    assert_eq!(table.symbol_count, roundtrip(&table).symbol_count);
}

#[test]
fn symbol_count_preserved_expr() {
    let table = build_table(&expr_grammar());
    assert_eq!(table.symbol_count, roundtrip(&table).symbol_count);
}

#[test]
fn symbol_count_preserved_seq() {
    let table = build_table(&seq_grammar());
    assert_eq!(table.symbol_count, roundtrip(&table).symbol_count);
}

#[test]
fn symbol_count_preserved_two_nt() {
    let table = build_table(&two_nt_grammar());
    assert_eq!(table.symbol_count, roundtrip(&table).symbol_count);
}

#[test]
fn symbol_count_preserved_right_recursive() {
    let table = build_table(&right_recursive_grammar());
    assert_eq!(table.symbol_count, roundtrip(&table).symbol_count);
}

#[test]
fn symbol_count_preserved_triple_seq() {
    let table = build_table(&triple_seq_grammar());
    assert_eq!(table.symbol_count, roundtrip(&table).symbol_count);
}

// ===================================================================
// 4. Actions preserved (7 tests)
// ===================================================================

fn assert_all_actions_match(original: &ParseTable, restored: &ParseTable, grammar: &Grammar) {
    let eof = original.eof();
    for st in 0..original.state_count {
        let sid = StateId(st as u16);
        // Check EOF actions
        assert_eq!(
            original.actions(sid, eof),
            restored.actions(sid, eof),
            "actions mismatch at state {st} for EOF"
        );
        // Check all token actions
        for (&sym, _) in &grammar.tokens {
            assert_eq!(
                original.actions(sid, sym),
                restored.actions(sid, sym),
                "actions mismatch at state {st} for symbol {sym:?}"
            );
        }
    }
}

#[test]
fn actions_preserved_simple() {
    let g = simple_grammar();
    let table = build_table(&g);
    let restored = roundtrip(&table);
    assert_all_actions_match(&table, &restored, &g);
}

#[test]
fn actions_preserved_two_alt() {
    let g = two_alt_grammar();
    let table = build_table(&g);
    let restored = roundtrip(&table);
    assert_all_actions_match(&table, &restored, &g);
}

#[test]
fn actions_preserved_chain() {
    let g = chain_grammar();
    let table = build_table(&g);
    let restored = roundtrip(&table);
    assert_all_actions_match(&table, &restored, &g);
}

#[test]
fn actions_preserved_expr() {
    let g = expr_grammar();
    let table = build_table(&g);
    let restored = roundtrip(&table);
    assert_all_actions_match(&table, &restored, &g);
}

#[test]
fn actions_preserved_seq() {
    let g = seq_grammar();
    let table = build_table(&g);
    let restored = roundtrip(&table);
    assert_all_actions_match(&table, &restored, &g);
}

#[test]
fn actions_preserved_two_nt() {
    let g = two_nt_grammar();
    let table = build_table(&g);
    let restored = roundtrip(&table);
    assert_all_actions_match(&table, &restored, &g);
}

#[test]
fn actions_preserved_right_recursive() {
    let g = right_recursive_grammar();
    let table = build_table(&g);
    let restored = roundtrip(&table);
    assert_all_actions_match(&table, &restored, &g);
}

// ===================================================================
// 5. Goto preserved (8 tests)
// ===================================================================

fn assert_all_gotos_match(original: &ParseTable, restored: &ParseTable, grammar: &Grammar) {
    for st in 0..original.state_count {
        let sid = StateId(st as u16);
        for (nt_sym, _) in &grammar.rule_names {
            assert_eq!(
                original.goto(sid, *nt_sym),
                restored.goto(sid, *nt_sym),
                "goto mismatch at state {st} for nonterminal {nt_sym:?}"
            );
        }
    }
}

#[test]
fn goto_preserved_simple() {
    let g = simple_grammar();
    let table = build_table(&g);
    let restored = roundtrip(&table);
    assert_all_gotos_match(&table, &restored, &g);
}

#[test]
fn goto_preserved_two_alt() {
    let g = two_alt_grammar();
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

#[test]
fn goto_preserved_expr() {
    let g = expr_grammar();
    let table = build_table(&g);
    let restored = roundtrip(&table);
    assert_all_gotos_match(&table, &restored, &g);
}

#[test]
fn goto_preserved_seq() {
    let g = seq_grammar();
    let table = build_table(&g);
    let restored = roundtrip(&table);
    assert_all_gotos_match(&table, &restored, &g);
}

#[test]
fn goto_preserved_two_nt() {
    let g = two_nt_grammar();
    let table = build_table(&g);
    let restored = roundtrip(&table);
    assert_all_gotos_match(&table, &restored, &g);
}

#[test]
fn goto_preserved_right_recursive() {
    let g = right_recursive_grammar();
    let table = build_table(&g);
    let restored = roundtrip(&table);
    assert_all_gotos_match(&table, &restored, &g);
}

#[test]
fn goto_preserved_triple_seq() {
    let g = triple_seq_grammar();
    let table = build_table(&g);
    let restored = roundtrip(&table);
    assert_all_gotos_match(&table, &restored, &g);
}

// ===================================================================
// 6. Invalid bytes handling (8 tests)
// ===================================================================

#[test]
fn invalid_bytes_empty() {
    assert!(ParseTable::from_bytes(&[]).is_err());
}

#[test]
fn invalid_bytes_single_byte() {
    assert!(ParseTable::from_bytes(&[0x42]).is_err());
}

#[test]
fn invalid_bytes_two_bytes() {
    assert!(ParseTable::from_bytes(&[0x00, 0x01]).is_err());
}

#[test]
fn invalid_bytes_all_zeros() {
    assert!(ParseTable::from_bytes(&[0u8; 16]).is_err());
}

#[test]
fn invalid_bytes_all_ones() {
    assert!(ParseTable::from_bytes(&[0xFF; 32]).is_err());
}

#[test]
fn invalid_bytes_garbage() {
    let garbage: Vec<u8> = (0..64).map(|i| (i * 37 + 13) as u8).collect();
    assert!(ParseTable::from_bytes(&garbage).is_err());
}

#[test]
fn invalid_bytes_truncated_valid() {
    let table = build_table(&simple_grammar());
    let bytes = table.to_bytes().expect("serialize");
    // Truncate to half
    let half = bytes.len() / 2;
    assert!(ParseTable::from_bytes(&bytes[..half]).is_err());
}

#[test]
fn invalid_bytes_corrupted_middle() {
    let table = build_table(&simple_grammar());
    let mut bytes = table.to_bytes().expect("serialize");
    // Corrupt the middle of the serialized data
    if bytes.len() > 4 {
        let mid = bytes.len() / 2;
        bytes[mid] ^= 0xFF;
        bytes[mid + 1] ^= 0xFF;
    }
    // May or may not deserialize, but should not panic
    let _ = ParseTable::from_bytes(&bytes);
}

// ===================================================================
// 7. Determinism (8 tests)
// ===================================================================

fn assert_deterministic(grammar: &Grammar) {
    let table = build_table(grammar);
    let bytes1 = table.to_bytes().expect("serialize first");
    let bytes2 = table.to_bytes().expect("serialize second");
    assert_eq!(bytes1, bytes2, "serialization must be deterministic");
}

#[test]
fn determinism_simple() {
    assert_deterministic(&simple_grammar());
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
fn determinism_seq() {
    assert_deterministic(&seq_grammar());
}

#[test]
fn determinism_two_nt() {
    assert_deterministic(&two_nt_grammar());
}

#[test]
fn determinism_right_recursive() {
    assert_deterministic(&right_recursive_grammar());
}

#[test]
fn determinism_triple_seq() {
    assert_deterministic(&triple_seq_grammar());
}
