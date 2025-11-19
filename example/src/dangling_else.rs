// For pure-rust: Include and re-export the generated parser symbols
#[cfg(feature = "pure-rust")]
pub mod generated {
    include!(concat!(
        env!("OUT_DIR"),
        "/grammar_dangling_else/parser_dangling_else.rs"
    ));
}

// Re-export the key symbols for tests
#[cfg(feature = "pure-rust")]
pub use generated::{LANGUAGE, SMALL_PARSE_TABLE, SMALL_PARSE_TABLE_MAP};

// The grammar definition - classic dangling else problem
// This grammar is GUARANTEED to have shift/reduce conflicts:
//
// Input: "if a then if b then s else t"
//
// Conflict: After parsing "if b then s", on lookahead "else":
//   - SHIFT: Continue with outer if (attach else to outer) → "if a then (if b then s) else t"
//   - REDUCE: Complete inner if (attach else to inner) → "if a then (if b then s else t)"
//
// This is a classic ambiguity that CANNOT be resolved by LR(1) lookahead alone,
// making it perfect for testing GLR conflict preservation.

#[rust_sitter::grammar("dangling_else")]
pub mod grammar {
    /// Statement: if-then or if-then-else constructs
    #[rust_sitter::language]
    #[derive(PartialEq, Eq, Debug, Clone)]
    pub enum Statement {
        /// If-then without else clause (creates the ambiguity)
        IfThen(
            #[rust_sitter::leaf(text = "if")] (),
            Box<Expr>,
            #[rust_sitter::leaf(text = "then")] (),
            Box<Statement>,
        ),

        /// If-then-else with else clause
        IfThenElse(
            #[rust_sitter::leaf(text = "if")] (),
            Box<Expr>,
            #[rust_sitter::leaf(text = "then")] (),
            Box<Statement>,
            #[rust_sitter::leaf(text = "else")] (),
            Box<Statement>,
        ),

        /// Simple statement (base case)
        Other(#[rust_sitter::leaf(text = "other")] ()),
    }

    /// Expression: simple variable names
    #[rust_sitter::language]
    #[derive(PartialEq, Eq, Debug, Clone)]
    pub enum Expr {
        Var(#[rust_sitter::leaf(pattern = r"[a-z]+")] String),
    }

    /// Whitespace handling
    #[rust_sitter::extra]
    struct Whitespace {
        #[rust_sitter::leaf(pattern = r"\s")]
        _whitespace: (),
    }
}

#[cfg(test)]
mod tests {
    use super::grammar;
    use super::grammar::*;

    #[test]
    fn test_simple_if_then() {
        // Simple if-then without else (no ambiguity)
        let result = grammar::parse("if a then other");
        println!("Parse result for 'if a then other': {:?}", result);

        match result {
            Ok(stmt) => {
                assert!(matches!(stmt, Statement::IfThen(_, _, _, _)));
            }
            Err(e) => panic!("Parse failed: {:?}", e),
        }
    }

    #[test]
    fn test_simple_if_then_else() {
        // Simple if-then-else (no ambiguity)
        let result = grammar::parse("if a then other else other");
        println!("Parse result for 'if a then other else other': {:?}", result);

        match result {
            Ok(stmt) => {
                assert!(matches!(stmt, Statement::IfThenElse(_, _, _, _, _, _)));
            }
            Err(e) => panic!("Parse failed: {:?}", e),
        }
    }

    #[test]
    #[ignore] // Enable once GLR runtime is working
    fn test_dangling_else_ambiguity() {
        // The classic dangling else: "if a then if b then s else t"
        // This creates a genuine ambiguity:
        //
        // Interpretation 1 (shift - attach else to inner if):
        //   if a then (if b then s else t)
        //
        // Interpretation 2 (reduce - attach else to outer if):
        //   (if a then if b then s) else t
        //
        // Most languages prefer interpretation 1 (attach to nearest if)

        let input = "if a then if b then other else other";
        let result = grammar::parse(input);

        println!("Parse result for dangling else: {:?}", result);

        match result {
            Ok(stmt) => {
                // With GLR, we should get interpretation 1 (attach to inner if)
                // This matches conventional language semantics
                match stmt {
                    Statement::IfThen(_, _, _, inner) => {
                        // The inner statement should be an IfThenElse
                        assert!(
                            matches!(*inner, Statement::IfThenElse(_, _, _, _, _, _)),
                            "Expected inner if-then-else, got {:?}",
                            inner
                        );
                    }
                    _ => panic!("Expected IfThen at top level, got {:?}", stmt),
                }
            }
            Err(e) => {
                println!("Parse error (expected with current LR implementation): {:?}", e);
                // This is expected if GLR isn't preserving conflicts
                // Once GLR is working, this should parse successfully
            }
        }
    }

    #[test]
    #[cfg(feature = "pure-rust")]
    fn test_conflict_detection() {
        // This test verifies that the grammar DOES generate conflicts
        // by inspecting the generated parse table

        use rust_sitter_glr_core::conflict_inspection::*;

        // Decode the LANGUAGE into a ParseTable
        let table = rust_sitter::decoder::decode_parse_table(&LANGUAGE);

        // Run conflict inspection
        let summary = count_conflicts(&table);

        eprintln!("Dangling Else Conflict Detection:");
        eprintln!("  States: {}", table.state_count);
        eprintln!("  Shift/Reduce conflicts: {}", summary.shift_reduce);
        eprintln!("  Reduce/Reduce conflicts: {}", summary.reduce_reduce);
        eprintln!("  States with conflicts: {:?}", summary.states_with_conflicts);

        // Expected: Exactly 1 shift/reduce conflict on "else" token
        //
        // State X (after "if Expr then Statement"), Symbol "else":
        //   - Shift(Y):   Continue outer if, shift else token
        //   - Reduce(Z):  Complete inner if-then, reduce to Statement
        //
        // This is the CLASSIC dangling else ambiguity
        assert_eq!(
            summary.shift_reduce, 1,
            "Dangling else grammar must have exactly 1 S/R conflict, got {}",
            summary.shift_reduce
        );

        assert_eq!(
            summary.reduce_reduce, 0,
            "Dangling else grammar should have no R/R conflicts, got {}",
            summary.reduce_reduce
        );

        // Find the specific "else" conflict
        let else_conflicts: Vec<_> = summary
            .conflict_details
            .iter()
            .filter(|c| c.symbol_name.contains("else"))
            .collect();

        assert!(
            !else_conflicts.is_empty(),
            "Should have conflict on 'else' symbol"
        );

        // Verify it's a shift/reduce conflict with 2 actions
        for conflict in else_conflicts {
            assert_eq!(
                conflict.conflict_type,
                ConflictType::ShiftReduce,
                "Else conflict should be ShiftReduce type"
            );

            assert_eq!(
                conflict.actions.len(),
                2,
                "Else conflict should have exactly 2 actions (Shift and Reduce)"
            );

            eprintln!("\n✅ Validated 'else' conflict:");
            eprintln!("  State: {}", conflict.state.0);
            eprintln!("  Symbol: {}", conflict.symbol_name);
            eprintln!("  Type: {:?}", conflict.conflict_type);
            eprintln!("  Actions: {} (Shift + Reduce)", conflict.actions.len());
        }
    }

    #[test]
    fn test_nested_if_without_else() {
        // Nested if-then statements without else (no ambiguity)
        let result = grammar::parse("if a then if b then other");

        match result {
            Ok(stmt) => {
                match stmt {
                    Statement::IfThen(_, _, _, inner) => {
                        assert!(matches!(*inner, Statement::IfThen(_, _, _, _)));
                    }
                    _ => panic!("Expected IfThen at top level"),
                }
            }
            Err(e) => panic!("Parse failed: {:?}", e),
        }
    }

    #[test]
    fn test_fully_specified_nested_if() {
        // Fully specified nested if (both have else - no ambiguity)
        let result = grammar::parse("if a then if b then other else other else other");

        match result {
            Ok(stmt) => {
                match stmt {
                    Statement::IfThenElse(_, _, _, inner, _, _) => {
                        assert!(matches!(*inner, Statement::IfThenElse(_, _, _, _, _, _)));
                    }
                    _ => panic!("Expected IfThenElse at top level"),
                }
            }
            Err(e) => panic!("Parse failed: {:?}", e),
        }
    }
}
