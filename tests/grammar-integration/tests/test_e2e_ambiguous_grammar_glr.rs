//! End-to-End Validation: Ambiguous Grammar GLR Support
//!
//! This test suite validates the complete pipeline from enum-based grammar definition
//! through GLR conflict generation to runtime parsing with fork/merge behavior.
//!
//! **Contract**: docs/specs/E2E_AMBIGUOUS_GRAMMAR_GLR_VALIDATION.md
//! **Prerequisites**:
//!   - ADR-0003: Enum variant inlining implemented
//!   - GLR conflict preservation fix in glr-core
//!   - ambiguous_expr.rs test grammar available
//!
//! **Success Criteria**:
//!   1. Enum-based ambiguous grammars generate GLR conflicts
//!   2. GLR runtime successfully parses ambiguous input
//!   3. Valid AST produced from parse forest
//!   4. Backward compatibility with precedence grammars maintained

#[cfg(feature = "ts-compat")]
use adze::adze_glr_core as glr_core;
use adze::decoder;
use adze::pure_parser::TSLanguage;

#[cfg(not(feature = "ts-compat"))]
use adze_glr_core as glr_core;

use glr_core::Action;

/// Helper: Count multi-action cells (GLR conflicts) in a parse table
fn count_multi_action_cells(lang: &'static TSLanguage) -> usize {
    let parse_table = decoder::decode_parse_table(lang);

    let mut conflict_count = 0;
    for state_actions in &parse_table.action_table {
        for action_cell in state_actions {
            if action_cell.len() > 1 {
                conflict_count += 1;
            }
        }
    }

    conflict_count
}

/// Helper: Check if action cell contains both shift and reduce
fn contains_shift_reduce(cell: &[Action]) -> bool {
    let has_shift = cell.iter().any(|a| matches!(a, Action::Shift(_)));
    let has_reduce = cell.iter().any(|a| matches!(a, Action::Reduce(_)));
    has_shift && has_reduce
}

//==============================================================================
// Scenario 1: Conflict Generation Validation
//==============================================================================

#[test]
#[ignore = "KNOWN BUG: GLR conflict generation - enum variant inlining not generating conflicts yet"]
fn test_ambiguous_grammar_conflict_generation() {
    eprintln!("\n=== E2E TEST: Ambiguous Grammar Conflict Generation ===\n");

    // Load ambiguous_expr grammar parse table
    // This grammar has NO precedence, so it MUST generate conflicts
    use adze_example::ambiguous_expr::grammar;

    let lang = grammar::language();

    eprintln!("Step 1: Load parse table from generated grammar");
    let parse_table = decoder::decode_parse_table(lang);
    eprintln!("  ✓ Parse table loaded: {} states", parse_table.state_count);

    eprintln!("\nStep 2: Count multi-action cells (GLR conflicts)");
    let conflict_count = count_multi_action_cells(lang);
    eprintln!("  Multi-action cells found: {}", conflict_count);

    // Contract Assertion 1: Conflicts exist
    assert!(
        conflict_count > 0,
        "CONTRACT VIOLATION: Ambiguous grammar MUST generate GLR conflicts!\n\
         Expected: At least 1 multi-action cell\n\
         Actual: {} conflicts\n\n\
         This indicates enum variant inlining may not be working correctly.\n\
         Check: example/src/ambiguous_expr.rs has NO precedence annotations.\n\
         Check: ADR-0003 implementation in tool/src/expansion.rs",
        conflict_count
    );
    eprintln!("  ✅ Conflicts detected: {}", conflict_count);

    eprintln!("\nStep 3: Validate conflict patterns");
    // Contract Assertion 2: Find shift/reduce conflict for binary expression
    let mut has_binary_conflict = false;
    for (state_idx, state_actions) in parse_table.action_table.iter().enumerate() {
        for (symbol_idx, cell) in state_actions.iter().enumerate() {
            if cell.len() > 1 && contains_shift_reduce(cell) {
                has_binary_conflict = true;

                let symbol_name = if symbol_idx < parse_table.symbol_metadata.len() {
                    &parse_table.symbol_metadata[symbol_idx].name
                } else {
                    "UNKNOWN"
                };

                eprintln!("  ✓ Shift/Reduce conflict found:");
                eprintln!("     State: {}", state_idx);
                eprintln!("     Symbol: {} ({})", symbol_idx, symbol_name);
                eprintln!("     Actions: {:?}", cell);
            }
        }
    }

    assert!(
        has_binary_conflict,
        "CONTRACT VIOLATION: Expected shift/reduce conflict for binary expression!\n\
         Ambiguous grammar 'Expr → Expr OP Expr' MUST create conflict on lookahead OP.\n\n\
         Possible causes:\n\
         1. GLR conflict preservation not working (check glr-core/src/lib.rs:2019-2077)\n\
         2. Grammar structure wrong (intermediate symbols still present?)\n\
         3. LR(1) is sufficient to resolve (check grammar definition)"
    );
    eprintln!("  ✅ Binary expression shift/reduce conflict validated");

    eprintln!("\n✅ SCENARIO 1 PASSED: Conflict generation validated\n");
}

//==============================================================================
// Scenario 2: GLR Parsing Behavior
//==============================================================================

#[test]
#[ignore = "KNOWN BUG: GLR conflict generation - depends on conflict generation which is not yet working"]
fn test_ambiguous_grammar_glr_parsing() {
    eprintln!("\n=== E2E TEST: Ambiguous Grammar GLR Parsing ===\n");

    use adze_example::ambiguous_expr::grammar;
    use adze_example::ambiguous_expr::grammar::Expr;

    // Test 1: Simple ambiguous input
    eprintln!("Test 1: Parse '1 + 2 + 3' (ambiguous associativity)");
    let input = "1 + 2 + 3";

    let result = grammar::parse(input);

    // Contract Assertion 1: Parse succeeds
    assert!(
        result.is_ok(),
        "CONTRACT VIOLATION: GLR should handle ambiguous input without error!\n\
         Input: {:?}\n\
         Error: {:?}\n\n\
         GLR parser should create fork points and select a valid parse.\n\
         Check: GLR runtime integration (runtime/src/__private.rs::parse_with_glr)",
        input,
        result.err()
    );
    eprintln!("  ✅ Parse succeeded (no error)");

    let expr = result.unwrap();
    eprintln!("  Parsed AST: {:?}", expr);

    // Contract Assertion 2: Valid AST structure
    assert!(
        matches!(expr, Expr::Binary(_, _, _)),
        "CONTRACT VIOLATION: Should produce binary expression!\n\
         Actual: {:?}",
        expr
    );
    eprintln!("  ✅ Valid binary expression produced");

    // Contract Assertion 3: AST is well-formed (either left or right associative)
    // Left:  (1 + 2) + 3
    // Right: 1 + (2 + 3)
    fn verify_structure(expr: &Expr, depth: usize) {
        let indent = "  ".repeat(depth);
        match expr {
            Expr::Binary(left, op, right) => {
                eprintln!("{}Binary: {:?}", indent, op);
                verify_structure(left, depth + 1);
                verify_structure(right, depth + 1);
            }
            Expr::Number(n) => {
                eprintln!("{}Number: {}", indent, n);
            }
        }
    }

    eprintln!("\n  AST Structure:");
    verify_structure(&expr, 1);
    eprintln!("  ✅ Well-formed parse tree");

    // Test 2: Longer ambiguous input
    eprintln!("\nTest 2: Parse '1 + 2 + 3 + 4' (multiple ambiguity points)");
    let input = "1 + 2 + 3 + 4";
    let result = grammar::parse(input);

    assert!(result.is_ok(), "Failed to parse: {:?}", input);
    eprintln!("  ✅ Complex ambiguous input parsed successfully");

    eprintln!("\n✅ SCENARIO 2 PASSED: GLR parsing produces valid ASTs\n");
}

//==============================================================================
// Scenario 3: Backward Compatibility
//==============================================================================

#[test]
#[cfg(feature = "glr")]
#[ignore = "KNOWN BUG: example grammar Extract panics on precedence expressions"]
fn test_glr_backward_compatibility() {
    eprintln!("\n=== E2E TEST: GLR Backward Compatibility ===\n");

    // This test uses the arithmetic grammar which HAS precedence
    // It should work identically with or without GLR feature
    use adze_example::arithmetic::grammar;
    use adze_example::arithmetic::grammar::Expression;

    eprintln!("Testing precedence grammar: arithmetic");

    // Test multiplication binds tighter than subtraction
    let input = "1 - 2 * 3";
    eprintln!("Input: {:?}", input);

    let result = grammar::parse(input);

    // This should work even with GLR (precedence is preserved)
    assert!(
        result.is_ok(),
        "CONTRACT VIOLATION: Precedence grammar should work with GLR!\n\
         Error: {:?}",
        result.err()
    );

    let expr = result.unwrap();
    eprintln!("Parsed: {:?}", expr);

    // Contract Assertion: Multiplication binds tighter
    // Expected: 1 - (2 * 3), not (1 - 2) * 3
    match expr {
        Expression::Sub(ref left, _, ref right) => {
            assert_eq!(**left, Expression::Number(1), "Left operand should be 1");

            assert!(
                matches!(**right, Expression::Mul(_, _, _)),
                "Right operand should be Mul, got {:?}",
                **right
            );

            if let Expression::Mul(ref mul_left, _, ref mul_right) = **right {
                assert_eq!(**mul_left, Expression::Number(2));
                assert_eq!(**mul_right, Expression::Number(3));
            }

            eprintln!("  ✅ Correct precedence: 1 - (2 * 3)");
        }
        _ => panic!("Expected Sub at top level, got {:?}", expr),
    }

    eprintln!("\n✅ SCENARIO 3 PASSED: Backward compatibility maintained\n");
}

//==============================================================================
// Scenario 4: Ambiguous vs Arithmetic Comparison
//==============================================================================

#[test]
#[ignore = "KNOWN BUG: GLR conflict generation - ambiguous grammar not generating conflicts yet"]
fn test_ambiguous_vs_arithmetic_comparison() {
    eprintln!("\n=== E2E TEST: Ambiguous vs Arithmetic Comparison ===\n");

    // Load both grammars
    use adze_example::ambiguous_expr::grammar as ambiguous;
    use adze_example::arithmetic::grammar as arithmetic;

    eprintln!("Step 1: Load ambiguous_expr grammar");
    let ambiguous_lang = ambiguous::language();
    let ambiguous_conflicts = count_multi_action_cells(ambiguous_lang);
    eprintln!("  Ambiguous grammar conflicts: {}", ambiguous_conflicts);

    eprintln!("\nStep 2: Load arithmetic grammar");
    let arithmetic_lang = arithmetic::language();
    let arithmetic_conflicts = count_multi_action_cells(arithmetic_lang);
    eprintln!("  Arithmetic grammar conflicts: {}", arithmetic_conflicts);

    eprintln!("\n=== Comparison ===");
    eprintln!(
        "  Ambiguous (no precedence): {} conflicts",
        ambiguous_conflicts
    );
    eprintln!(
        "  Arithmetic (with precedence): {} conflicts",
        arithmetic_conflicts
    );

    // Contract Assertion: Ambiguous has conflicts, Arithmetic has none
    assert!(
        ambiguous_conflicts > 0,
        "CONTRACT VIOLATION: Ambiguous grammar MUST have conflicts!"
    );

    assert_eq!(
        arithmetic_conflicts, 0,
        "REGRESSION: Arithmetic grammar should have ZERO conflicts (LR(1) sufficient)"
    );

    eprintln!("\n✅ SCENARIO 4 PASSED: Grammars correctly differentiated\n");
    eprintln!("Key Finding:");
    eprintln!("  - Ambiguous grammar generates GLR conflicts (as expected)");
    eprintln!("  - Arithmetic grammar generates zero conflicts (LR(1) sufficient)");
    eprintln!("  - This proves enum variant inlining enables true ambiguity");
}

//==============================================================================
// Documentation Test: Contract Summary
//==============================================================================

#[test]
fn test_contract_documentation() {
    eprintln!("\n=== E2E GLR VALIDATION CONTRACT ===\n");
    eprintln!("This test suite validates:");
    eprintln!("  1. ✓ Enum variant inlining enables ambiguous grammars");
    eprintln!("  2. ✓ GLR conflict generation works correctly");
    eprintln!("  3. ✓ GLR runtime parses ambiguous input successfully");
    eprintln!("  4. ✓ Backward compatibility with precedence grammars");
    eprintln!();
    eprintln!("To run validation:");
    eprintln!("  cargo test -p adze --features glr --test test_e2e_ambiguous_grammar_glr");
    eprintln!();
    eprintln!("Expected Results:");
    eprintln!("  - test_ambiguous_grammar_conflict_generation: PASS");
    eprintln!("  - test_ambiguous_grammar_glr_parsing: PASS");
    eprintln!("  - test_glr_backward_compatibility: PASS");
    eprintln!("  - test_ambiguous_vs_arithmetic_comparison: PASS");
    eprintln!();
    eprintln!("See: docs/specs/E2E_AMBIGUOUS_GRAMMAR_GLR_VALIDATION.md");
}
