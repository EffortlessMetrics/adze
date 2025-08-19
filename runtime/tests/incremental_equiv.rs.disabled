//! Tests to verify that incremental parsing produces the same results as fresh parsing

#[cfg(all(test, feature = "incremental_glr"))]
mod tests {
    use rust_sitter::parser_v4::{Parser, Tree};
    use rust_sitter::pure_incremental::Edit;
    use rust_sitter_glr_core::ParseTable;
    use rust_sitter_ir::Grammar;

    fn get_arithmetic_parser() -> Parser {
        // Create a minimal arithmetic grammar for testing
        use rust_sitter_glr_core::{Action, StateId, SymbolId};
        use rust_sitter_ir::{Grammar, GrammarRule, Symbol};

        let mut grammar = Grammar::default();
        grammar.rules.push(GrammarRule {
            lhs: SymbolId(1),       // expression
            rhs: vec![SymbolId(0)], // number
            precedence: 0,
            associativity: None,
            is_fragile: false,
            fields: vec![],
        });

        // Create a minimal parse table
        let mut symbol_to_index = std::collections::BTreeMap::new();
        for i in 0..3 {
            symbol_to_index.insert(SymbolId(i as u16), i);
        }

        let table = ParseTable {
            state_count: 2,
            symbol_count: 3,
            action_table: vec![vec![vec![]; 3]; 2],
            goto_table: vec![vec![StateId(0); 3]; 2],
            symbol_metadata: vec![],
            symbol_to_index,
            index_to_symbol: (0..3).map(|i| SymbolId(i as u16)).collect(),
            external_scanner_states: vec![],
            token_count: 2,
            external_token_count: 0,
            eof_symbol: SymbolId(2),
            start_symbol: SymbolId(1),
            initial_state: StateId(0),
            rules: vec![],
            lex_modes: vec![],
            extras: vec![],
            dynamic_prec_by_rule: vec![],
            alias_sequences: vec![],
            field_names: vec![],
            field_map: std::collections::BTreeMap::new(),
            nonterminal_to_index: std::collections::BTreeMap::new(),
            grammar: grammar.clone(),
        };

        Parser::new(grammar, table)
    }

    #[test]
    fn test_fresh_equals_incremental_insert() {
        let parser = get_arithmetic_parser();

        // Initial parse
        let src1 = "1+2";
        let tree1 = parser
            .parse(src1.as_bytes(), None)
            .expect("Failed to parse");

        // Edit: insert "*3" at the end
        let src2 = "1+2*3";
        let edit = Edit {
            start_byte: 3,
            old_end_byte: 3,
            new_end_byte: 5,
            start_point: Default::default(),
            old_end_point: Default::default(),
            new_end_point: Default::default(),
        };

        // Incremental parse
        let tree2_incremental = parser
            .reparse(src2.as_bytes(), &tree1, &edit)
            .expect("Incremental parsing should succeed");

        // Fresh parse
        let tree2_fresh = parser
            .parse(src2.as_bytes(), None)
            .expect("Fresh parsing should succeed");

        // Compare S-expressions
        assert_eq!(
            format!("{:?}", tree2_incremental),
            format!("{:?}", tree2_fresh),
            "Incremental and fresh parse should produce the same tree"
        );
    }

    #[test]
    fn test_fresh_equals_incremental_delete() {
        let parser = get_arithmetic_parser();

        // Initial parse
        let src1 = "1+2*3";
        let tree1 = parser
            .parse(src1.as_bytes(), None)
            .expect("Failed to parse");

        // Edit: delete "*3"
        let src2 = "1+2";
        let edit = Edit {
            start_byte: 3,
            old_end_byte: 5,
            new_end_byte: 3,
            start_point: Default::default(),
            old_end_point: Default::default(),
            new_end_point: Default::default(),
        };

        // Incremental parse
        let tree2_incremental = parser
            .reparse(src2.as_bytes(), &tree1, &edit)
            .expect("Incremental parsing should succeed");

        // Fresh parse
        let tree2_fresh = parser
            .parse(src2.as_bytes(), None)
            .expect("Fresh parsing should succeed");

        // Compare S-expressions
        assert_eq!(
            format!("{:?}", tree2_incremental),
            format!("{:?}", tree2_fresh),
            "Incremental and fresh parse should produce the same tree"
        );
    }

    #[test]
    fn test_fresh_equals_incremental_replace() {
        let parser = get_arithmetic_parser();

        // Initial parse
        let src1 = "1+2";
        let tree1 = parser
            .parse(src1.as_bytes(), None)
            .expect("Failed to parse");

        // Edit: replace "+" with "*"
        let src2 = "1*2";
        let edit = Edit {
            start_byte: 1,
            old_end_byte: 2,
            new_end_byte: 2,
            start_point: Default::default(),
            old_end_point: Default::default(),
            new_end_point: Default::default(),
        };

        // Incremental parse
        let tree2_incremental = parser
            .reparse(src2.as_bytes(), &tree1, &edit)
            .expect("Incremental parsing should succeed");

        // Fresh parse
        let tree2_fresh = parser
            .parse(src2.as_bytes(), None)
            .expect("Fresh parsing should succeed");

        // Compare S-expressions
        assert_eq!(
            format!("{:?}", tree2_incremental),
            format!("{:?}", tree2_fresh),
            "Incremental and fresh parse should produce the same tree"
        );
    }

    #[test]
    fn test_multiple_edits() {
        let parser = get_arithmetic_parser();

        // Test a series of edits
        let test_cases = vec![
            (
                "1",
                "1+2",
                Edit {
                    start_byte: 1,
                    old_end_byte: 1,
                    new_end_byte: 3,
                    start_point: Default::default(),
                    old_end_point: Default::default(),
                    new_end_point: Default::default(),
                },
            ),
            (
                "1+2",
                "1+2+3",
                Edit {
                    start_byte: 3,
                    old_end_byte: 3,
                    new_end_byte: 5,
                    start_point: Default::default(),
                    old_end_point: Default::default(),
                    new_end_point: Default::default(),
                },
            ),
            (
                "1+2+3",
                "(1+2)+3",
                Edit {
                    start_byte: 0,
                    old_end_byte: 3,
                    new_end_byte: 5,
                    start_point: Default::default(),
                    old_end_point: Default::default(),
                    new_end_point: Default::default(),
                },
            ),
            (
                "(1+2)+3",
                "(1+2)*3",
                Edit {
                    start_byte: 5,
                    old_end_byte: 6,
                    new_end_byte: 6,
                    start_point: Default::default(),
                    old_end_point: Default::default(),
                    new_end_point: Default::default(),
                },
            ),
        ];

        let mut prev_tree = parser
            .parse(test_cases[0].0.as_bytes(), None)
            .expect("Initial parse failed");

        for (prev_src, new_src, edit) in test_cases.iter().skip(1) {
            // Incremental parse
            let incremental_tree = parser
                .reparse(new_src.as_bytes(), &prev_tree, edit)
                .expect("Incremental parsing should succeed");

            // Fresh parse
            let fresh_tree = parser
                .parse(new_src.as_bytes(), None)
                .expect("Fresh parsing should succeed");

            // Compare
            assert_eq!(
                format!("{:?}", incremental_tree),
                format!("{:?}", fresh_tree),
                "Trees should match for transition from '{}' to '{}'",
                prev_src,
                new_src
            );

            prev_tree = incremental_tree;
        }
    }
}

#[cfg(not(feature = "incremental_glr"))]
#[test]
fn test_incremental_disabled() {
    // This test verifies that without the feature, reparse returns None
    use rust_sitter::parser_v4::{Parser, Tree};
    use rust_sitter::pure_incremental::Edit;
    use rust_sitter_glr_core::ParseTable;
    use rust_sitter_ir::Grammar;

    // Create a minimal parser (details don't matter since reparse should return None)
    let grammar = Grammar::default();
    let mut symbol_to_index = std::collections::BTreeMap::new();
    symbol_to_index.insert(rust_sitter_glr_core::SymbolId(0), 0);
    let table = ParseTable {
        state_count: 1,
        symbol_count: 1,
        action_table: vec![vec![vec![]]],
        goto_table: vec![vec![]],
        symbol_metadata: vec![],
        symbol_to_index,
        index_to_symbol: vec![rust_sitter_glr_core::SymbolId(0)],
        external_scanner_states: vec![],
        token_count: 0,
        external_token_count: 0,
        eof_symbol: rust_sitter_glr_core::SymbolId(0),
        start_symbol: rust_sitter_glr_core::SymbolId(0),
        initial_state: rust_sitter_glr_core::StateId(0),
        rules: vec![],
        lex_modes: vec![],
        extras: vec![],
        dynamic_prec_by_rule: vec![],
        alias_sequences: vec![],
        field_names: vec![],
        field_map: std::collections::BTreeMap::new(),
        nonterminal_to_index: std::collections::BTreeMap::new(),
        grammar: Grammar::default(),
    };
    let parser = Parser::new(grammar, table, "test".to_string());

    let src = "test";
    let mut parser = parser;
    let tree = parser.parse(src);

    if let Ok(tree) = tree {
        let edit = Edit {
            start_byte: 0,
            old_end_byte: 1,
            new_end_byte: 1,
            start_point: Default::default(),
            old_end_point: Default::default(),
            new_end_point: Default::default(),
        };

        // Without incremental_glr feature, reparse should return None
        assert!(
            parser.reparse(src.as_bytes(), &tree, &edit).is_none(),
            "reparse should return None when incremental_glr feature is disabled"
        );
    }
}
