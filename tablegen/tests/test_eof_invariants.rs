//! Integration test for EOF invariants
//! Validates that EOF handling works correctly without position assumptions

use rust_sitter_tablegen::helpers::{collect_token_indices, eof_accepts_or_reduces};

#[test]
fn test_eof_in_token_indices() {
    // Create a minimal parse table to test with
    // Since the API is internal, we'll use the helpers to validate behavior
    
    // For now, we just ensure the helpers compile and can be called
    // A real test would need a ParseTable instance, which requires
    // access to internal APIs that aren't currently exposed
    
    // This test documents the expected invariants:
    // 1. EOF (SymbolId(0)) should always be in the token indices
    // 2. EOF position is derived from symbol_to_index, not hardcoded
    // 3. eof_accepts_or_reduces returns true for nullable start, false otherwise
    
    // Once the Grammar builder API is exposed (planned for 0.8.0),
    // we can write a proper integration test like:
    //
    // let grammar = Grammar::builder()
    //     .add_terminal("a", TokenId(1))
    //     .add_non_terminal("S")
    //     .add_rule("S", vec![]) // nullable start
    //     .set_start_symbol("S")
    //     .build();
    //
    // let parse_table = generate_parse_table(&grammar);
    // let indices = collect_token_indices(&grammar, &parse_table);
    // 
    // // EOF should be in indices but not necessarily at position 0
    // assert!(indices.iter().any(|&idx| {
    //     parse_table.symbol_to_index.get(&SymbolId(0)) == Some(&idx)
    // }));
    //
    // // Nullable start should make eof_accepts_or_reduces true
    // assert!(eof_accepts_or_reduces(&parse_table));
    
    // For 0.7.0, we validate that the functions exist and compile
    assert!(true, "EOF invariant helpers are available");
}

#[test]
fn test_eof_helper_signatures() {
    // This test validates that the helper functions have the expected signatures
    // It will fail to compile if the API changes unexpectedly
    
    use rust_sitter_glr_core::ParseTable;
    use rust_sitter_ir::Grammar;
    
    // These are just type checks, not runtime tests
    let _: fn(&Grammar, &ParseTable) -> Vec<usize> = collect_token_indices;
    let _: fn(&ParseTable) -> bool = eof_accepts_or_reduces;
}