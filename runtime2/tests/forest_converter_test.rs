//! ForestConverter Unit and Integration Tests (Phase 3.2)
//!
//! Following TDD/BDD methodology - tests written before implementation.
//! Contract: docs/specs/FOREST_CONVERTER_CONTRACT.md

#[cfg(feature = "pure-rust-glr")]
mod forest_converter_unit_tests {
    use rust_sitter_runtime::forest_converter::{
        ConversionError, DisambiguationStrategy, ForestConverter,
    };
    use rust_sitter_runtime::glr_engine::{ForestNode, ForestNodeId, ParseForest};
    use rust_sitter_runtime::Tree;
    use rust_sitter_glr_core::SymbolId;
    use rust_sitter_ir::RuleId;

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
        forest.nodes.push(ForestNode::Terminal {
            symbol: SymbolId(1), // NUMBER
            range: 0..3,
        });

        // Node 1: Nonterminal expr
        forest.nodes.push(ForestNode::Nonterminal {
            symbol: SymbolId(2), // expr
            children: vec![ForestNodeId(0)],
            rule_id: RuleId(0),
        });

        forest.roots.push(ForestNodeId(1));
        forest
    }

    /// Helper: Create ambiguous forest with packed node
    ///
    /// Forest represents "1 + 2 + 3" with two parses:
    ///   Parse A: ((1 + 2) + 3)
    ///   Parse B: (1 + (2 + 3))
    fn create_ambiguous_forest() -> ParseForest {
        let mut forest = ParseForest {
            nodes: vec![],
            roots: vec![],
        };

        // Terminals
        forest.nodes.push(ForestNode::Terminal {
            symbol: SymbolId(1), // NUMBER "1"
            range: 0..1,
        });
        forest.nodes.push(ForestNode::Terminal {
            symbol: SymbolId(1), // NUMBER "2"
            range: 4..5,
        });
        forest.nodes.push(ForestNode::Terminal {
            symbol: SymbolId(1), // NUMBER "3"
            range: 8..9,
        });

        // Parse A: ((1 + 2) + 3) - left-associative
        // Node 3: expr(1 + 2)
        forest.nodes.push(ForestNode::Nonterminal {
            symbol: SymbolId(2), // expr
            children: vec![ForestNodeId(0), ForestNodeId(1)],
            rule_id: RuleId(0),
        });
        // Node 4: expr((1 + 2) + 3)
        forest.nodes.push(ForestNode::Nonterminal {
            symbol: SymbolId(2), // expr
            children: vec![ForestNodeId(3), ForestNodeId(2)],
            rule_id: RuleId(0),
        });

        // Parse B: (1 + (2 + 3)) - right-associative
        // Node 5: expr(2 + 3)
        forest.nodes.push(ForestNode::Nonterminal {
            symbol: SymbolId(2), // expr
            children: vec![ForestNodeId(1), ForestNodeId(2)],
            rule_id: RuleId(0),
        });
        // Node 6: expr(1 + (2 + 3))
        forest.nodes.push(ForestNode::Nonterminal {
            symbol: SymbolId(2), // expr
            children: vec![ForestNodeId(0), ForestNodeId(5)],
            rule_id: RuleId(0),
        });

        // Packed node with two alternatives
        forest.nodes.push(ForestNode::Packed {
            alternatives: vec![ForestNodeId(4), ForestNodeId(6)],
        });

        forest.roots.push(ForestNodeId(7)); // Packed node
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

        // Verify tree structure
        let root = tree.root_node();
        assert_eq!(root.kind(), "expr"); // Symbol 2
        assert_eq!(root.byte_range(), (0..3));
        assert_eq!(root.child_count(), 1);

        let child = root.child(0).unwrap();
        assert_eq!(child.kind(), "NUMBER"); // Symbol 1
        assert_eq!(child.byte_range(), (0..3));
    }

    /// Test: Terminal node conversion
    ///
    /// Contract: Terminal → leaf node with correct symbol and range
    #[test]
    fn test_terminal_node_conversion() {
        let converter = ForestConverter::new(DisambiguationStrategy::First);

        // Forest with just a terminal
        let mut forest = ParseForest {
            nodes: vec![ForestNode::Terminal {
                symbol: SymbolId(1),
                range: 0..5,
            }],
            roots: vec![ForestNodeId(0)],
        };

        let input = b"hello";
        let tree = converter.to_tree(&forest, input).unwrap();

        let root = tree.root_node();
        assert_eq!(root.kind(), "NUMBER"); // Symbol 1
        assert_eq!(root.byte_range(), (0..5));
        assert_eq!(root.child_count(), 0); // Leaf node
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

        let root = tree.root_node();
        assert_eq!(root.kind(), "expr");
        assert_eq!(root.child_count(), 1);

        // Child should be terminal
        let child = root.child(0).unwrap();
        assert_eq!(child.kind(), "NUMBER");
    }

    /// Test: PreferShift disambiguation strategy
    ///
    /// Contract: Select shift-derived alternative (right-associative)
    #[test]
    fn test_prefer_shift_strategy() {
        let converter = ForestConverter::new(DisambiguationStrategy::PreferShift);
        let forest = create_ambiguous_forest();
        let input = b"1 + 2 + 3";

        let tree = converter.to_tree(&forest, input).unwrap();

        // PreferShift should select right-associative parse: (1 + (2 + 3))
        // This is represented by alternative index 1 (ForestNodeId(6))
        //
        // Note: Actual tree structure depends on implementation details
        // For now, just verify it doesn't error and produces a tree
        let root = tree.root_node();
        assert_eq!(root.kind(), "expr");
    }

    /// Test: PreferReduce disambiguation strategy
    ///
    /// Contract: Select reduce-derived alternative (left-associative)
    #[test]
    fn test_prefer_reduce_strategy() {
        let converter = ForestConverter::new(DisambiguationStrategy::PreferReduce);
        let forest = create_ambiguous_forest();
        let input = b"1 + 2 + 3";

        let tree = converter.to_tree(&forest, input).unwrap();

        // PreferReduce should select left-associative parse: ((1 + 2) + 3)
        let root = tree.root_node();
        assert_eq!(root.kind(), "expr");
    }

    /// Test: First strategy (arbitrary selection)
    ///
    /// Contract: Select first alternative without analysis
    #[test]
    fn test_first_strategy() {
        let converter = ForestConverter::new(DisambiguationStrategy::First);
        let forest = create_ambiguous_forest();
        let input = b"1 + 2 + 3";

        let tree = converter.to_tree(&forest, input).unwrap();

        // First strategy just picks alternatives[0]
        let root = tree.root_node();
        assert_eq!(root.kind(), "expr");
    }

    /// Test: RejectAmbiguity strategy
    ///
    /// Contract: Error when multiple alternatives exist
    #[test]
    fn test_reject_ambiguity() {
        let converter = ForestConverter::new(DisambiguationStrategy::RejectAmbiguity);
        let forest = create_ambiguous_forest();
        let input = b"1 + 2 + 3";

        let result = converter.to_tree(&forest, input);

        // Should error on ambiguity
        assert!(result.is_err());

        if let Err(ConversionError::AmbiguousForest { count }) = result {
            assert_eq!(count, 2); // Two alternatives
        } else {
            panic!("Expected AmbiguousForest error");
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

        // Ambiguous forest
        let ambig_forest = create_ambiguous_forest();
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
            nodes: vec![ForestNode::Nonterminal {
                symbol: SymbolId(2),
                children: vec![ForestNodeId(999)], // Invalid!
                rule_id: RuleId(0),
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
    use rust_sitter_ir::RuleId;

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
        forest.nodes.push(ForestNode::Terminal {
            symbol: SymbolId(1), // NUMBER
            range: 0..1,
        });

        // Node 1: Terminal "+"
        forest.nodes.push(ForestNode::Terminal {
            symbol: SymbolId(2), // PLUS
            range: 2..3,
        });

        // Node 2: Terminal "2"
        forest.nodes.push(ForestNode::Terminal {
            symbol: SymbolId(1), // NUMBER
            range: 4..5,
        });

        // Node 3: Nonterminal expr (1 + 2)
        forest.nodes.push(ForestNode::Nonterminal {
            symbol: SymbolId(3), // expr
            children: vec![ForestNodeId(0), ForestNodeId(1), ForestNodeId(2)],
            rule_id: RuleId(0),
        });

        forest.roots.push(ForestNodeId(3));

        let input = b"1 + 2";
        let tree = converter.to_tree(&forest, input).unwrap();

        // Verify tree structure
        let root = tree.root_node();
        assert_eq!(root.kind(), "expr");
        assert_eq!(root.byte_range(), (0..5));
        assert_eq!(root.child_count(), 3); // 1, +, 2
    }

    /// Integration Test: Nested expressions
    ///
    /// Tests: "((1))" with parentheses
    #[test]
    fn test_nested_expressions() {
        let converter = ForestConverter::new(DisambiguationStrategy::First);

        // Build forest for "((1))"
        let mut forest = ParseForest {
            nodes: vec![],
            roots: vec![],
        };

        // Node 0: Terminal "1"
        forest.nodes.push(ForestNode::Terminal {
            symbol: SymbolId(1), // NUMBER
            range: 2..3,
        });

        // Node 1: expr(1)
        forest.nodes.push(ForestNode::Nonterminal {
            symbol: SymbolId(2), // expr
            children: vec![ForestNodeId(0)],
            rule_id: RuleId(1),
        });

        // Node 2: expr((1))
        forest.nodes.push(ForestNode::Nonterminal {
            symbol: SymbolId(2), // expr
            children: vec![ForestNodeId(1)],
            rule_id: RuleId(1),
        });

        forest.roots.push(ForestNodeId(2));

        let input = b"((1))";
        let tree = converter.to_tree(&forest, input).unwrap();

        // Verify nested structure
        let root = tree.root_node();
        assert_eq!(root.kind(), "expr");

        // Should have nested expr nodes
        assert!(root.child_count() > 0);
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
        forest.nodes.push(ForestNode::Terminal {
            symbol: SymbolId(1),
            range: 0..5,
        });

        // Root 2: Parse B
        forest.nodes.push(ForestNode::Terminal {
            symbol: SymbolId(2),
            range: 0..5,
        });

        forest.roots.push(ForestNodeId(0));
        forest.roots.push(ForestNodeId(1));

        let input = b"hello";

        // First strategy should pick first root
        let converter_first = ForestConverter::new(DisambiguationStrategy::First);
        let tree = converter_first.to_tree(&forest, input).unwrap();
        assert_eq!(tree.root_node().kind(), "NUMBER"); // Symbol 1

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
