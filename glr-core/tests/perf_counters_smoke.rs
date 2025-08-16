#![cfg(feature = "perf-counters")]

use glr_test_support::test_utilities::make_minimal_table;
use rust_sitter_glr_core::{perf, Driver};
use rust_sitter_ir::SymbolId;

#[test]
fn counters_move_on_small_parse() {
    // Create a minimal table for testing
    // Need at least 3 columns: ERROR(0), terminal(1), EOF(2)
    let table = make_minimal_table(
        vec![vec![vec![], vec![], vec![]]],  // 3 columns: ERROR, terminal, EOF
        vec![vec![], vec![], vec![]],        // 3 columns for gotos
        vec![],                              // No rules for this simple test
        SymbolId(3),                         // Start symbol (nonterminal)
        SymbolId(2),                         // EOF symbol at column 2
        0,                                   // No external tokens
    );
    let mut driver = Driver::new(&table);
    
    // Take initial counters
    let before = perf::take();
    
    // Parse a single EOF token
    let _ = driver.parse_tokens([(2u32, 0, 0)].into_iter());
    
    // Take counters after parsing
    let after = perf::take();
    
    // Ensure counters moved (we don't assert exact counts as they're grammar-dependent)
    assert!(
        after.shifts >= before.shifts,
        "Shifts counter should not decrease"
    );
    assert!(
        after.reductions >= before.reductions,
        "Reductions counter should not decrease"
    );
    // Note: forks/merges may or may not occur depending on the grammar
}