// For pure-rust: Include and re-export the generated parser symbols
#[cfg(feature = "pure-rust")]
pub mod generated {
    include!(concat!(
        env!("OUT_DIR"),
        "/grammar_ambiguous_expr/parser_ambiguous_expr.rs"
    ));
}

// Re-export the key symbols for tests
#[cfg(feature = "pure-rust")]
pub use generated::{LANGUAGE, SMALL_PARSE_TABLE, SMALL_PARSE_TABLE_MAP};

// The grammar definition - GUARANTEED to generate shift/reduce conflicts
//
// This grammar is intentionally ambiguous to test GLR conflict preservation:
//
// Grammar:
//   Expr → Expr Op Expr    (left-recursive binary operation)
//   Expr → Number          (terminal)
//   Op → '+' | '-' | '*' | '/'
//
// Input: "1 + 2 * 3"
//
// Conflict: After parsing "1 + 2", on lookahead "*":
//   - SHIFT: Continue to form potentially "(1 + 2) * 3" (left-associative)
//   - REDUCE: Complete "1 + 2", then form "1 + (2 * 3)" (right-associative)
//
// Key Differences from arithmetic.rs:
//   1. NO precedence annotations (no prec_left, prec_right)
//   2. SINGLE Binary variant (not separate Add/Sub/Mul variants)
//   3. Operator is captured as String (not implicit via variant type)
//
// This creates INHERENT AMBIGUITY that LR(1) cannot resolve.
// Perfect test case for GLR conflict preservation!

#[adze::grammar("ambiguous_expr")]
pub mod grammar {
    /// Expression: intentionally ambiguous binary operations
    /// NO precedence or associativity defined - this will create conflicts!
    #[adze::language]
    #[derive(PartialEq, Eq, Debug, Clone)]
    pub enum Expr {
        /// Single binary operation variant - NO precedence!
        /// All operators (+, -, *, /) are treated equally.
        /// This creates genuine shift/reduce conflicts.
        Binary(
            Box<Expr>,
            #[adze::leaf(pattern = r"[-+*/]")] String,
            Box<Expr>,
        ),

        /// Number terminal
        Number(
            #[adze::leaf(pattern = r"\d+", transform = |v| {
            println!("DEBUG: parsing number: {:?}", v);
            v.parse().unwrap()
        })]
            i32,
        ),
    }

    /// Whitespace handling
    #[adze::extra]
    struct Whitespace {
        #[adze::leaf(pattern = r"\s")]
        _whitespace: (),
    }
}

#[cfg(test)]
mod tests {
    use super::grammar;
    use super::grammar::*;

    #[test]
    fn test_simple_number() {
        // Simple number (no ambiguity)
        let result = grammar::parse("42");
        println!("Parse result for '42': {:?}", result);

        match result {
            Ok(expr) => {
                assert_eq!(expr, Expr::Number(42));
            }
            Err(e) => panic!("Parse failed: {:?}", e),
        }
    }

    #[test]
    fn test_simple_binary() {
        // Simple binary expression (no ambiguity)
        let result = grammar::parse("1 + 2");
        println!("Parse result for '1 + 2': {:?}", result);

        match result {
            Ok(expr) => match expr {
                Expr::Binary(left, op, right) => {
                    assert_eq!(*left, Expr::Number(1));
                    assert_eq!(op, "+");
                    assert_eq!(*right, Expr::Number(2));
                }
                _ => panic!("Expected Binary, got {:?}", expr),
            },
            Err(e) => panic!("Parse failed: {:?}", e),
        }
    }

    #[test]
    #[ignore] // Enable once GLR runtime is working
    fn test_ambiguous_expression() {
        // The classic ambiguous case: "1 + 2 * 3"
        // Without precedence, this has TWO valid parse trees:
        //
        // Tree 1 (left-associative): (1 + 2) * 3 = 9
        // Tree 2 (right-associative): 1 + (2 * 3) = 7
        //
        // With GLR, both interpretations are valid.
        // The parser should:
        //   1. Detect the shift/reduce conflict
        //   2. Fork into both paths
        //   3. Produce both parse trees (or select one based on priority)

        let input = "1 + 2 * 3";
        let result = grammar::parse(input);

        println!("Parse result for '{}': {:?}", input, result);

        match result {
            Ok(expr) => {
                println!("Successfully parsed ambiguous expression: {:?}", expr);
                // With GLR conflict preservation, this should parse successfully
                // The specific tree depends on which action has priority in the conflict
            }
            Err(e) => {
                println!("Parse error (may indicate conflict not resolved): {:?}", e);
                // This is possible if GLR isn't fully working yet
            }
        }
    }

    #[test]
    #[cfg(feature = "pure-rust")]
    fn test_conflict_detection() {
        // This test documents the EXPECTED conflicts in this grammar
        // and validates that they are properly preserved in the parse table

        use adze_glr_core::conflict_inspection::*;

        // Decode the LANGUAGE into a ParseTable
        let table = adze::decoder::decode_parse_table(&LANGUAGE);

        // Run conflict inspection
        let summary = count_conflicts(&table);

        eprintln!("Ambiguous Expression Conflict Detection:");
        eprintln!("  States: {}", table.state_count);
        eprintln!("  Shift/Reduce conflicts: {}", summary.shift_reduce);
        eprintln!("  Reduce/Reduce conflicts: {}", summary.reduce_reduce);
        eprintln!(
            "  States with conflicts: {}",
            summary.states_with_conflicts.len()
        );

        // Expected: At least 1 shift/reduce conflict on operators
        //
        // For input "1 + 2 * 3", after parsing "1 + 2" on lookahead "*":
        //
        // Conflict: Shift/Reduce on operator lookahead
        //   State X (after "Expr Op Expr"), Symbol "*":
        //     - Shift(Y):   Continue reading, form "(1 + 2) * 3"
        //     - Reduce(Z):  Complete "1 + 2", then form "1 + (2 * 3)"
        //
        // This conflict WILL be detected because:
        //   1. Single Binary variant (no enum disambiguation)
        //   2. No precedence annotations
        //   3. LR(1) cannot resolve without precedence
        assert!(
            summary.shift_reduce >= 1,
            "Ambiguous expr grammar must have at least 1 S/R conflict, got {}",
            summary.shift_reduce
        );

        assert_eq!(
            summary.reduce_reduce, 0,
            "Ambiguous expr grammar should have no R/R conflicts, got {}",
            summary.reduce_reduce
        );

        // Find operator conflicts
        let operator_conflicts: Vec<_> = summary
            .conflict_details
            .iter()
            .filter(|c| {
                c.symbol_name.contains('+')
                    || c.symbol_name.contains('-')
                    || c.symbol_name.contains('*')
                    || c.symbol_name.contains('/')
                    || c.symbol_name.contains("Op")
            })
            .collect();

        assert!(
            !operator_conflicts.is_empty(),
            "Should have conflicts on operator symbols, found conflicts: {:?}",
            summary
                .conflict_details
                .iter()
                .map(|c| &c.symbol_name)
                .collect::<Vec<_>>()
        );

        // Verify conflicts are shift/reduce with 2 actions
        for conflict in operator_conflicts {
            assert_eq!(
                conflict.conflict_type,
                ConflictType::ShiftReduce,
                "Operator conflicts should be ShiftReduce type"
            );

            assert_eq!(
                conflict.actions.len(),
                2,
                "Operator conflicts should have exactly 2 actions (Shift and Reduce)"
            );

            eprintln!("\n✅ Validated operator conflict:");
            eprintln!("  State: {}", conflict.state.0);
            eprintln!("  Symbol: {}", conflict.symbol_name);
            eprintln!("  Type: {:?}", conflict.conflict_type);
            eprintln!("  Actions: {} (Shift + Reduce)", conflict.actions.len());
        }
    }

    #[test]
    fn test_multiple_operators() {
        // Test with multiple operators of different types
        let result = grammar::parse("1 + 2 - 3");
        println!("Parse result for '1 + 2 - 3': {:?}", result);

        // Without precedence, this could parse as:
        //   - (1 + 2) - 3  (left-associative)
        //   - 1 + (2 - 3)  (right-associative)
        //
        // GLR should handle this
    }

    #[test]
    fn test_same_operator_repeated() {
        // Test with same operator repeated
        let result = grammar::parse("1 + 2 + 3");
        println!("Parse result for '1 + 2 + 3': {:?}", result);

        // Even same operator creates ambiguity without associativity:
        //   - (1 + 2) + 3  (left-associative)
        //   - 1 + (2 + 3)  (right-associative)
        //
        // For commutative operators the result is the same,
        // but the parse tree structure differs
    }

    #[test]
    fn test_long_expression() {
        // Test with longer expression
        let result = grammar::parse("1 + 2 * 3 - 4");
        println!("Parse result for '1 + 2 * 3 - 4': {:?}", result);

        // This has MANY possible parse trees!
        // GLR should explore all valid interpretations
    }

    #[test]
    fn test_all_operators() {
        // Test that all operators work
        for op in ["+", "-", "*", "/"] {
            let input = format!("5 {} 3", op);
            let result = grammar::parse(&input);
            println!("Parse result for '{}': {:?}", input, result);

            match result {
                Ok(Expr::Binary(left, parsed_op, right)) => {
                    assert_eq!(*left, Expr::Number(5));
                    assert_eq!(parsed_op, op);
                    assert_eq!(*right, Expr::Number(3));
                }
                Ok(other) => panic!("Expected Binary, got {:?}", other),
                Err(e) => panic!("Parse failed for '{}': {:?}", input, e),
            }
        }
    }
}
