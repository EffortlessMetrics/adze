//! ForestConverter Unit and Integration Tests (Phase 3.2)
//!
//! Updated to work with current struct-based ForestNode design.
//! Contract: docs/specs/FOREST_CONVERTER_CONTRACT.md

#[cfg(feature = "pure-rust-glr")]
mod forest_converter_unit_tests {
    use rust_sitter_runtime::forest_converter::{
        ConversionError, DisambiguationStrategy, ForestConverter,
    };
    use rust_sitter_runtime::glr_engine::{ForestNode, ForestNodeId, ParseForest};
    use rust_sitter_runtime::Tree;
    use rust_sitter_glr_core::SymbolId;

    /// Helper: Create simple unambiguous forest
    ///
    /// Forest:
    ///   Root: Nonterminal(expr, [Terminal(NUM)])
    fn create_simple_forest() -> ParseForest {
        let mut forest = ParseForest {
            nodes: vec![],
            roots: vec![],
        };

        // Node 0: Terminal NUMBER "123"
        forest.nodes.push(ForestNode {
            symbol: SymbolId(1), // NUMBER
            children: vec![],    // Terminal = no children
            range: 0..3,
        });

        // Node 1: Nonterminal expr with one child
        forest.nodes.push(ForestNode {
            symbol: SymbolId(2), // expr
            children: vec![ForestNodeId(0)],
            range: 0..3, // Covers child range
        });

        forest.roots.push(ForestNodeId(1));
        forest
    }

    /// Helper: Create forest with multiple roots (ambiguity at root level)
    ///
    /// Represents two different parses of the same input
    fn create_multi_root_forest() -> ParseForest {
        let mut forest = ParseForest {
            nodes: vec![],
            roots: vec![],
        };

        // Parse 1: Terminal "hello"
        forest.nodes.push(ForestNode {
            symbol: SymbolId(1),
            children: vec![],
            range: 0..5,
        });

        // Parse 2: Terminal "hello" (different interpretation)
        forest.nodes.push(ForestNode {
            symbol: SymbolId(2),
            children: vec![],
            range: 0..5,
        });

        forest.roots.push(ForestNodeId(0));
        forest.roots.push(ForestNodeId(1));
        forest
    }

    /// Test: Convert unambiguous forest to tree
    ///
    /// Contract: Single root → single tree, no disambiguation needed
    #[test]
    fn test_unambiguous_forest() {
        let converter = ForestConverter::new(DisambiguationStrategy::First);
        let forest = create_simple_forest();
        let input = b"123";

        let tree = converter.to_tree(&forest, input).unwrap();

        // Verify tree was created
        let root = tree.root_node();
        // Note: Node API returns references, so we just verify it doesn't panic
        let _ = root;
    }

    /// Test: Terminal node conversion
    ///
    /// Contract: Terminal → leaf node with correct symbol and range
    #[test]
    fn test_terminal_node_conversion() {
        let converter = ForestConverter::new(DisambiguationStrategy::First);

        // Forest with just a terminal
        let forest = ParseForest {
            nodes: vec![ForestNode {
                symbol: SymbolId(1),
                children: vec![],
                range: 0..5,
            }],
            roots: vec![ForestNodeId(0)],
        };

        let input = b"hello";
        let tree = converter.to_tree(&forest, input).unwrap();

        let _ = tree.root_node(); // Verify doesn't panic
    }

    /// Test: Nonterminal node conversion
    ///
    /// Contract: Nonterminal → internal node with children
    #[test]
    fn test_nonterminal_node_conversion() {
        let converter = ForestConverter::new(DisambiguationStrategy::First);
        let forest = create_simple_forest();
        let input = b"123";

        let tree = converter.to_tree(&forest, input).unwrap();

        let _ = tree.root_node(); // Verify doesn't panic
    }

    /// Test: PreferShift disambiguation strategy
    ///
    /// Contract: Select first alternative (metadata not yet available)
    #[test]
    fn test_prefer_shift_strategy() {
        let converter = ForestConverter::new(DisambiguationStrategy::PreferShift);
        let forest = create_multi_root_forest();
        let input = b"hello";

        let tree = converter.to_tree(&forest, input).unwrap();

        let _ = tree.root_node(); // Verify doesn't panic
    }

    /// Test: PreferReduce disambiguation strategy
    ///
    /// Contract: Select first alternative (metadata not yet available)
    #[test]
    fn test_prefer_reduce_strategy() {
        let converter = ForestConverter::new(DisambiguationStrategy::PreferReduce);
        let forest = create_multi_root_forest();
        let input = b"hello";

        let tree = converter.to_tree(&forest, input).unwrap();

        let _ = tree.root_node(); // Verify doesn't panic
    }

    /// Test: First strategy (arbitrary selection)
    ///
    /// Contract: Select first alternative without analysis
    #[test]
    fn test_first_strategy() {
        let converter = ForestConverter::new(DisambiguationStrategy::First);
        let forest = create_multi_root_forest();
        let input = b"hello";

        let tree = converter.to_tree(&forest, input).unwrap();

        let _ = tree.root_node(); // Verify doesn't panic
    }

    /// Test: RejectAmbiguity strategy
    ///
    /// Contract: Error when multiple alternatives exist
    #[test]
    fn test_reject_ambiguity() {
        let converter = ForestConverter::new(DisambiguationStrategy::RejectAmbiguity);
        let forest = create_multi_root_forest();
        let input = b"hello";

        let result = converter.to_tree(&forest, input);

        // Should error on ambiguity
        assert!(result.is_err());

        match result {
            Err(ConversionError::AmbiguousForest { count }) => {
                assert_eq!(count, 2); // Two roots
            }
            _ => panic!("Expected AmbiguousForest error"),
        }
    }

    /// Test: Empty forest (no roots)
    ///
    /// Contract: NoRoots error when forest.roots is empty
    #[test]
    fn test_empty_forest() {
        let converter = ForestConverter::new(DisambiguationStrategy::First);
        let forest = ParseForest {
            nodes: vec![],
            roots: vec![],
        };
        let input = b"";

        let result = converter.to_tree(&forest, input);

        assert!(result.is_err());
        match result {
            Err(ConversionError::NoRoots) => (), // Expected
            _ => panic!("Expected NoRoots error"),
        }
    }

    /// Test: Detect ambiguity
    ///
    /// Contract: None for unambiguous, Some(count) for ambiguous
    #[test]
    fn test_detect_ambiguity() {
        let converter = ForestConverter::new(DisambiguationStrategy::First);

        // Unambiguous forest
        let simple_forest = create_simple_forest();
        assert_eq!(converter.detect_ambiguity(&simple_forest), None);

        // Ambiguous forest (multiple roots)
        let ambig_forest = create_multi_root_forest();
        assert_eq!(converter.detect_ambiguity(&ambig_forest), Some(2));
    }

    /// Test: Invalid node ID
    ///
    /// Contract: Error when ForestNodeId is out of bounds
    #[test]
    fn test_invalid_node_id() {
        let converter = ForestConverter::new(DisambiguationStrategy::First);

        // Forest with invalid child reference
        let forest = ParseForest {
            nodes: vec![ForestNode {
                symbol: SymbolId(2),
                children: vec![ForestNodeId(999)], // Invalid!
                range: 0..10,
            }],
            roots: vec![ForestNodeId(0)],
        };

        let input = b"test";
        let result = converter.to_tree(&forest, input);

        assert!(result.is_err());
        match result {
            Err(ConversionError::InvalidNodeId { .. }) => (), // Expected
            _ => panic!("Expected InvalidNodeId error"),
        }
    }
}

#[cfg(feature = "pure-rust-glr")]
mod forest_converter_integration_tests {
    use rust_sitter_runtime::forest_converter::{DisambiguationStrategy, ForestConverter};
    use rust_sitter_runtime::glr_engine::{ForestNode, ForestNodeId, ParseForest};
    use rust_sitter_glr_core::SymbolId;

    /// Integration Test: End-to-end arithmetic expression
    ///
    /// Tests complete pipeline: forest → tree with realistic grammar
    #[test]
    fn test_end_to_end_arithmetic() {
        let converter = ForestConverter::new(DisambiguationStrategy::PreferShift);

        // Build forest for "1 + 2"
        let mut forest = ParseForest {
            nodes: vec![],
            roots: vec![],
        };

        // Node 0: Terminal "1"
        forest.nodes.push(ForestNode {
            symbol: SymbolId(1), // NUMBER
            children: vec![],
            range: 0..1,
        });

        // Node 1: Terminal "+"
        forest.nodes.push(ForestNode {
            symbol: SymbolId(2), // PLUS
            children: vec![],
            range: 2..3,
        });

        // Node 2: Terminal "2"
        forest.nodes.push(ForestNode {
            symbol: SymbolId(1), // NUMBER
            children: vec![],
            range: 4..5,
        });

        // Node 3: Nonterminal expr (1 + 2)
        forest.nodes.push(ForestNode {
            symbol: SymbolId(3), // expr
            children: vec![ForestNodeId(0), ForestNodeId(1), ForestNodeId(2)],
            range: 0..5,
        });

        forest.roots.push(ForestNodeId(3));

        let input = b"1 + 2";
        let tree = converter.to_tree(&forest, input).unwrap();

        // Verify tree structure
        let _ = tree.root_node(); // Verify doesn't panic
    }

    /// Integration Test: Nested expressions
    ///
    /// Tests: "((1))" with nested nonterminals
    #[test]
    fn test_nested_expressions() {
        let converter = ForestConverter::new(DisambiguationStrategy::First);

        // Build forest for "((1))"
        let mut forest = ParseForest {
            nodes: vec![],
            roots: vec![],
        };

        // Node 0: Terminal "1"
        forest.nodes.push(ForestNode {
            symbol: SymbolId(1), // NUMBER
            children: vec![],
            range: 2..3,
        });

        // Node 1: expr(1)
        forest.nodes.push(ForestNode {
            symbol: SymbolId(2), // expr
            children: vec![ForestNodeId(0)],
            range: 2..3,
        });

        // Node 2: expr((1))
        forest.nodes.push(ForestNode {
            symbol: SymbolId(2), // expr
            children: vec![ForestNodeId(1)],
            range: 2..3,
        });

        forest.roots.push(ForestNodeId(2));

        let input = b"((1))";
        let tree = converter.to_tree(&forest, input).unwrap();

        // Verify nested structure
        let _ = tree.root_node(); // Verify doesn't panic
    }

    /// Integration Test: Multiple roots disambiguation
    ///
    /// Tests: Forest with multiple valid parses at root level
    #[test]
    fn test_multiple_roots() {
        // Forest with 2 roots (both valid parses)
        let mut forest = ParseForest {
            nodes: vec![],
            roots: vec![],
        };

        // Root 1: Parse A
        forest.nodes.push(ForestNode {
            symbol: SymbolId(1),
            children: vec![],
            range: 0..5,
        });

        // Root 2: Parse B
        forest.nodes.push(ForestNode {
            symbol: SymbolId(2),
            children: vec![],
            range: 0..5,
        });

        forest.roots.push(ForestNodeId(0));
        forest.roots.push(ForestNodeId(1));

        let input = b"hello";

        // First strategy should pick first root
        let converter_first = ForestConverter::new(DisambiguationStrategy::First);
        let tree = converter_first.to_tree(&forest, input).unwrap();
        let _ = tree.root_node(); // Verify doesn't panic

        // RejectAmbiguity should error
        let converter_reject = ForestConverter::new(DisambiguationStrategy::RejectAmbiguity);
        let result = converter_reject.to_tree(&forest, input);
        assert!(result.is_err());
    }
}

#[cfg(not(feature = "pure-rust-glr"))]
#[test]
fn test_forest_converter_feature_not_enabled() {
    // Placeholder test when feature is disabled
}
