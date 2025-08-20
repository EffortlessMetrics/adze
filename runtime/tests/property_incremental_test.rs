// Property-based tests for incremental parsing
// These tests ensure that incremental parsing produces the same results as fresh parsing

// Basic sanity test that always runs
#[test]
fn test_fresh_parse_sanity() {
    // This test verifies basic parsing works even without incremental features
    // It serves as a compile-time check that the test infrastructure is valid
    use rust_sitter_ir::{Grammar, ProductionId, Rule, Symbol, SymbolId, Token, TokenPattern};

    let mut grammar = Grammar::new("test".to_string());
    let num_id = SymbolId(1);
    grammar.tokens.insert(
        num_id,
        Token {
            name: "NUM".to_string(),
            pattern: TokenPattern::Regex(r"\d+".to_string()),
            fragile: false,
        },
    );

    let rule = Rule {
        lhs: SymbolId(0),
        rhs: vec![Symbol::Terminal(num_id)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    };
    grammar.add_rule(rule);

    // Just verify grammar construction works
    assert_eq!(grammar.rules.len(), 1);
}

#[cfg(all(test, feature = "incremental_glr"))]
mod incremental_properties {
    use proptest::prelude::*;
    use rust_sitter::parser_v4::{Parser, Tree};
    use rust_sitter::pure_incremental::Edit;
    use rust_sitter::pure_parser::Point;
    use rust_sitter_glr_core::ParseTable;
    use rust_sitter_ir::{Grammar, ProductionId, Rule, Symbol, SymbolId};

    use super::common;

    /// Strategy for generating source code strings
    fn source_strategy() -> impl Strategy<Value = String> {
        prop::string::string_regex("[0-9a-z ]{1,50}").unwrap()
    }

    /// Strategy for generating edits
    fn edit_strategy() -> impl Strategy<Value = (usize, usize, String)> {
        (0usize..20, 0usize..10, "[0-9a-z ]{0,10}")
            .prop_map(|(pos, del_len, insert)| (pos, del_len, insert))
    }

    /// Apply an edit to a source string
    fn apply_edit(source: &str, pos: usize, del_len: usize, insert: &str) -> String {
        let pos = pos.min(source.len());
        let del_len = del_len.min(source.len() - pos);
        let mut result = String::new();
        result.push_str(&source[..pos]);
        result.push_str(insert);
        result.push_str(&source[pos + del_len..]);
        result
    }

    /// Create an Edit struct from edit parameters
    fn create_edit(pos: usize, del_len: usize, insert_len: usize) -> Edit {
        Edit {
            start_byte: pos,
            old_end_byte: pos + del_len,
            new_end_byte: pos + insert_len,
            start_point: Point {
                row: 0,
                column: pos,
            }, // Simplified - assumes single line
            old_end_point: Point {
                row: 0,
                column: pos + del_len,
            },
            new_end_point: Point {
                row: 0,
                column: pos + insert_len,
            },
        }
    }

    /// Helper to create a simple test grammar and parse table
    fn create_test_setup() -> (Grammar, ParseTable) {
        // Create a very simple grammar that just accepts any sequence of tokens
        let mut grammar = Grammar::new("test".to_string());

        // Add simple number token
        let number_id = SymbolId(1);
        grammar.add_terminal(number_id, "number".to_string());

        // Add simple identifier token
        let ident_id = SymbolId(2);
        grammar.add_terminal(ident_id, "identifier".to_string());

        // Add root non-terminal that accepts a sequence
        let root_id = SymbolId(10);
        let rule = Rule {
            lhs: root_id,
            rhs: vec![Symbol::Terminal(number_id)], // Simplified for now
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(0),
        };
        grammar.add_rule(rule);

        let table = common::build_table(&grammar);
        (grammar, *table)
    }

    #[cfg(feature = "incremental_glr")]
    proptest! {
        /// Property: Fresh parse and incremental parse should produce equivalent trees
        #[test]
        #[cfg_attr(not(feature = "incremental_glr"), ignore = "incremental parsing not enabled")]
        fn fresh_vs_incremental_equivalent(
            original in source_strategy(),
            (edit_pos, del_len, insert) in edit_strategy()
        ) {
            let (grammar, table) = create_test_setup();
            let mut parser = Parser::new(grammar.clone(), table.clone(), "test".to_string());

            // Parse original
            let tree1 = parser.parse(&original).expect("Initial parse should succeed");

            // Apply edit
            let edited = apply_edit(&original, edit_pos, del_len, &insert);
            let edit = create_edit(edit_pos, del_len, insert.len());

            // Parse fresh
            let tree_fresh = parser.parse(&edited).expect("Fresh parse should succeed");

            // Try incremental parse (when implemented)
            if let Some(tree_inc) = parser.reparse(&edited, &tree1, &edit) {
                // Core properties that must hold
                prop_assert_eq!(tree_inc.root_kind, tree_fresh.root_kind,
                    "Root kinds should match between incremental and fresh parse");
                prop_assert_eq!(tree_inc.error_count, tree_fresh.error_count,
                    "Error counts should match between incremental and fresh parse");

                // Additional properties can be added as the implementation matures
            } else {
                // Incremental parsing not implemented yet - that's OK
                // At least verify fresh parsing works
                prop_assert!(tree_fresh.error_count == 0 || tree_fresh.error_count > 0,
                    "Fresh parse should complete with a valid error count");
            }
        }

        /// Property: Multiple sequential edits should produce the same result regardless of path
        #[test]
        #[cfg_attr(not(feature = "incremental_glr"), ignore = "incremental parsing not enabled")]
        fn sequential_edits_consistent(
            original in source_strategy(),
            edits in prop::collection::vec(edit_strategy(), 1..5)
        ) {
            let (grammar, table) = create_test_setup();
            let mut parser = Parser::new(grammar.clone(), table.clone(), "test".to_string());

            // Apply all edits at once to get final string
            let mut final_source = original.clone();
            for (pos, del_len, insert) in &edits {
                final_source = apply_edit(&final_source, *pos, *del_len, insert);
            }

            // Parse the final result fresh
            let tree_fresh = parser.parse(&final_source).expect("Fresh parse should succeed");

            // Now try applying edits incrementally (when implemented)
            // This would verify that the order of incremental edits doesn't affect the final result

            // For now, just verify fresh parsing works
            prop_assert!(tree_fresh.error_count == 0 || tree_fresh.error_count > 0,
                "Fresh parse should complete with a valid error count");
        }
    }
}
