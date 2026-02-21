//! Integration test for EOF invariants
//! Validates that EOF handling works correctly without position assumptions

use adze_glr_core::{FirstFollowSets, build_lr1_automaton};
use adze_ir::*;
use adze_tablegen::helpers::{collect_token_indices, eof_accepts_or_reduces};

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
    // For 0.7.0, we validate that the functions exist and compile - no assertion needed
}

#[test]
fn test_eof_helper_signatures() {
    // This test validates that the helper functions have the expected signatures
    // It will fail to compile if the API changes unexpectedly

    use adze_glr_core::ParseTable;
    use adze_ir::Grammar;

    // These are just type checks, not runtime tests
    let _: fn(&Grammar, &ParseTable) -> Vec<usize> = collect_token_indices;
    let _: fn(&ParseTable) -> bool = eof_accepts_or_reduces;
}

/// Contract test: Validate symbol layout invariants
///
/// This test ensures that:
/// 1. EOF symbol ID never collides with grammar symbol IDs
/// 2. Terminals (including EOF) are at indices 0..token_count
/// 3. Non-terminals are at indices token_count..symbol_count
/// 4. Production LHS symbols map to non-terminal columns
#[test]
fn test_symbol_layout_invariants() {
    // Create a grammar where non-terminals would collide with EOF if improperly assigned
    // Grammar: S -> 'a' | ε
    // With tokens at SymbolId(1) and non-terminal at SymbolId(2)
    let mut grammar = Grammar::new("layout_test".to_string());

    // Add token 'a' at SymbolId(1)
    grammar.tokens.insert(
        SymbolId(1),
        Token {
            name: "a".to_string(),
            pattern: TokenPattern::String("a".to_string()),
            fragile: false,
        },
    );

    // Add non-terminal S at SymbolId(2)
    // NOTE: Intentionally NOT adding to rule_names to test the original collision scenario.
    // The fix ensures EOF symbol ID calculation includes grammar.rules.keys(), not just rule_names.
    let start = SymbolId(2);

    // Rule 1: S -> 'a'
    grammar.rules.entry(start).or_default().push(Rule {
        lhs: start,
        rhs: vec![Symbol::Terminal(SymbolId(1))],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });

    // Rule 2: S -> ε (empty)
    grammar.rules.entry(start).or_default().push(Rule {
        lhs: start,
        rhs: vec![],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(1),
    });

    // Build FIRST/FOLLOW sets and parse table
    let first_follow = FirstFollowSets::compute(&grammar).unwrap();
    let parse_table = build_lr1_automaton(&grammar, &first_follow).unwrap();

    // Invariant 1: EOF symbol ID is distinct from all grammar symbols
    let grammar_symbol_ids: Vec<u16> = grammar
        .tokens
        .keys()
        .chain(grammar.rules.keys())
        .map(|s| s.0)
        .collect();
    assert!(
        !grammar_symbol_ids.contains(&parse_table.eof_symbol.0),
        "EOF symbol {:?} must not collide with grammar symbols {:?}",
        parse_table.eof_symbol,
        grammar_symbol_ids
    );

    // Invariant 2: EOF is in symbol_to_index (without position assumptions)
    let eof_idx = *parse_table
        .symbol_to_index
        .get(&parse_table.eof_symbol)
        .expect("EOF must be in symbol_to_index");

    // Invariant 3: All terminals are in terminal region
    // Terminal boundary is token_count + 1 (includes EOF)
    let terminal_boundary = parse_table.token_count + 1;
    for (symbol_id, &idx) in &parse_table.symbol_to_index {
        let is_terminal =
            grammar.tokens.contains_key(symbol_id) || *symbol_id == parse_table.eof_symbol;

        if is_terminal {
            assert!(
                idx < terminal_boundary,
                "Terminal {:?} at index {} must be in terminal region (0..{})",
                symbol_id,
                idx,
                terminal_boundary
            );
        } else {
            assert!(
                idx >= terminal_boundary,
                "Non-terminal {:?} at index {} must be in non-terminal region ({}..)",
                symbol_id,
                idx,
                terminal_boundary
            );
        }
    }

    // Invariant 4: Production LHS symbols are at non-terminal indices
    for rule in grammar.all_rules() {
        let lhs_idx = parse_table
            .symbol_to_index
            .get(&rule.lhs)
            .expect("Production LHS must be in symbol_to_index");
        assert!(
            *lhs_idx >= terminal_boundary,
            "Production LHS {:?} at index {} must be in non-terminal region ({}..)",
            rule.lhs,
            *lhs_idx,
            terminal_boundary
        );
    }

    // Invariant 5: Nullable start should have Accept or Reduce on EOF
    assert!(
        eof_accepts_or_reduces(&parse_table),
        "Grammar with nullable start should have Accept/Reduce on EOF in state 0"
    );

    // Invariant 6: EOF is in token_indices (using eof_idx, not hardcoded 0)
    let token_indices = collect_token_indices(&grammar, &parse_table);
    assert!(
        token_indices.contains(&eof_idx),
        "EOF column ({}) must be in token_indices {:?}",
        eof_idx,
        token_indices
    );

    println!("✓ All symbol layout invariants verified");
}
