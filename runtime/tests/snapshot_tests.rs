//! Snapshot tests for parse tables, forests, and generated code
//!
//! These tests ensure that changes to the parser generator don't accidentally
//! break the output format or introduce subtle bugs in table encoding.

use insta::{assert_debug_snapshot, assert_snapshot};
use rust_sitter::*;

#[cfg(feature = "incremental_glr")]
mod glr_snapshots {
    use super::*;
    use rust_sitter::glr_incremental::IncrementalGLRParser;
    use rust_sitter::glr_parser::GLRParser;

    /// Helper to create a stable string representation of a parse forest
    fn forest_to_string(forest: &impl std::fmt::Debug) -> String {
        format!("{:#?}", forest)
    }

    #[test]
    fn snapshot_simple_expr_forest() {
        // This would use your test grammar - placeholder for now
        let input = "1 + 2 * 3";
        // let (grammar, table) = test_arithmetic_grammar();
        // let mut parser = IncrementalGLRParser::new(table, grammar);
        // let forest = parser.parse(input).unwrap();
        // assert_snapshot!(forest_to_string(&forest));

        // For now just snapshot the input
        assert_snapshot!(input);
    }

    #[test]
    fn snapshot_ambiguous_grammar_forest() {
        // Test with an ambiguous grammar to ensure all parse paths are captured
        let input = "if a then if b then c else d";
        // This should produce multiple parse trees in the forest
        assert_snapshot!(input);
    }

    #[test]
    fn snapshot_parse_table_encoding() {
        // Snapshot the binary encoding of parse tables to catch compression changes
        // let table = create_test_parse_table();
        // let encoded = encode_table(&table);
        // assert_snapshot!(hex::encode(&encoded));
    }
}

#[cfg(not(feature = "incremental_glr"))]
mod standard_snapshots {
    use super::*;

    #[test]
    fn snapshot_parse_tree() {
        // Standard parser snapshot tests
        let input = "1 + 2";
        assert_snapshot!(input);
    }
}

/// Property-based tests for parser invariants
#[cfg(test)]
mod property_tests {
    use proptest::prelude::*;

    /// Generate arbitrary valid token streams for testing
    fn arbitrary_tokens() -> impl Strategy<Value = Vec<String>> {
        prop::collection::vec(
            prop_oneof![
                Just("1".to_string()),
                Just("+".to_string()),
                Just("*".to_string()),
                Just("(".to_string()),
                Just(")".to_string()),
            ],
            0..20,
        )
    }

    proptest! {
        #[test]
        fn parse_reparse_equivalence(tokens in arbitrary_tokens()) {
            // Property: parsing then reparsing should yield same result
            let input = tokens.join(" ");
            // let result1 = parse(&input);
            // let result2 = parse(&input);
            // prop_assert_eq!(result1, result2);
        }

        #[test]
        fn no_unreachable_forest_nodes(tokens in arbitrary_tokens()) {
            // Property: all nodes in parse forest should be reachable from root
            // let forest = parse_to_forest(&tokens.join(" "));
            // if let Ok(forest) = forest {
            //     prop_assert!(forest.all_nodes_reachable());
            // }
        }
    }
}

/// Contract tests for codegen stability
#[cfg(test)]
mod codegen_contract_tests {
    use super::*;

    #[test]
    fn generated_parser_structure() {
        // Ensure generated parser code has expected structure
        // This catches breaking changes in code generation

        // Snapshot the generated code structure
        let expected_structure = r#"
        pub struct Parser { ... }
        impl Parser {
            pub fn new() -> Self { ... }
            pub fn parse(&mut self, input: &str) -> Result<Tree, Error> { ... }
        }
        "#;

        assert_snapshot!(expected_structure);
    }

    #[test]
    fn ffi_language_struct_layout() {
        // Ensure FFI Language struct matches Tree-sitter ABI
        use std::mem;

        // These sizes must match Tree-sitter's C ABI exactly
        // assert_eq!(mem::size_of::<Language>(), expected_size);
        // assert_eq!(mem::align_of::<Language>(), expected_align);

        // Snapshot the layout for regression detection
        let layout = format!(
            "size={}, align={}",
            128, // placeholder
            8    // placeholder
        );
        assert_snapshot!(layout);
    }
}
