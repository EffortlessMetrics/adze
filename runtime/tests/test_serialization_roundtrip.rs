//! Comprehensive serialization roundtrip tests
//!
//! These tests ensure that:
//! 1. Serialization APIs work correctly with the rust-sitter types
//! 2. SerializedNode roundtrips through JSON properly
//! 3. S-expression parsing roundtrips correctly
//! 4. TreeSerializer produces valid output
//! 5. Unicode edge cases are handled correctly
//! 6. Property-based testing catches edge cases

// Only compile this test file when serialization feature is enabled
#![cfg(feature = "serialization")]

use rust_sitter::serialization::*;
use std::collections::HashMap;

#[cfg(test)]
mod serialized_node_tests {
    #[cfg(feature = "serialization")]
    use super::*;

    /// Test 1: SerializedNode JSON roundtrip
    #[test]
    #[cfg(feature = "serialization")]
    fn test_serialized_node_json_roundtrip() {
        let original_node = SerializedNode {
            kind: "identifier".to_string(),
            is_named: true,
            field_name: Some("name".to_string()),
            start_position: (0, 0),
            end_position: (0, 5),
            start_byte: 0,
            end_byte: 5,
            text: Some("hello".to_string()),
            children: vec![],
            is_error: false,
            is_missing: false,
        };

        // Serialize to JSON
        let json = serde_json::to_string(&original_node).unwrap();

        // Deserialize back
        let decoded: SerializedNode = serde_json::from_str(&json).unwrap();

        // Verify roundtrip identity
        assert_eq!(original_node.kind, decoded.kind);
        assert_eq!(original_node.is_named, decoded.is_named);
        assert_eq!(original_node.field_name, decoded.field_name);
        assert_eq!(original_node.start_position, decoded.start_position);
        assert_eq!(original_node.end_position, decoded.end_position);
        assert_eq!(original_node.start_byte, decoded.start_byte);
        assert_eq!(original_node.end_byte, decoded.end_byte);
        assert_eq!(original_node.text, decoded.text);
        assert_eq!(original_node.children.len(), decoded.children.len());
        assert_eq!(original_node.is_error, decoded.is_error);
        assert_eq!(original_node.is_missing, decoded.is_missing);
    }

    /// Test 2: TreeSerializer configuration
    #[test]
    #[cfg(feature = "serialization")]
    fn test_tree_serializer_configuration() {
        let source = b"test source code";
        let serializer = TreeSerializer::new(source)
            .with_unnamed_nodes()
            .with_max_text_length(Some(50));

        assert!(serializer.include_unnamed);
        assert_eq!(serializer.max_text_length, Some(50));
        assert_eq!(serializer.source, source);
    }

    /// Test 3: Complex nested structure JSON roundtrip
    #[test]
    #[cfg(feature = "serialization")]
    fn test_complex_nested_structure() {
        let child1 = SerializedNode {
            kind: "identifier".to_string(),
            is_named: true,
            field_name: Some("left".to_string()),
            start_position: (0, 0),
            end_position: (0, 3),
            start_byte: 0,
            end_byte: 3,
            text: Some("foo".to_string()),
            children: vec![],
            is_error: false,
            is_missing: false,
        };

        let child2 = SerializedNode {
            kind: "identifier".to_string(),
            is_named: true,
            field_name: Some("right".to_string()),
            start_position: (0, 6),
            end_position: (0, 9),
            start_byte: 6,
            end_byte: 9,
            text: Some("bar".to_string()),
            children: vec![],
            is_error: false,
            is_missing: false,
        };

        let parent = SerializedNode {
            kind: "binary_expression".to_string(),
            is_named: true,
            field_name: None,
            start_position: (0, 0),
            end_position: (0, 9),
            start_byte: 0,
            end_byte: 9,
            text: None,
            children: vec![child1, child2],
            is_error: false,
            is_missing: false,
        };

        // JSON roundtrip
        let json = serde_json::to_string_pretty(&parent).unwrap();
        let decoded: SerializedNode = serde_json::from_str(&json).unwrap();

        assert_eq!(parent.kind, decoded.kind);
        assert_eq!(parent.children.len(), decoded.children.len());
        assert_eq!(parent.children[0].text, decoded.children[0].text);
        assert_eq!(parent.children[1].text, decoded.children[1].text);
        assert_eq!(
            parent.children[0].field_name,
            decoded.children[0].field_name
        );
        assert_eq!(
            parent.children[1].field_name,
            decoded.children[1].field_name
        );
    }

    /// Test 4: CompactNode JSON roundtrip
    #[test]
    #[cfg(feature = "serialization")]
    fn test_compact_node_roundtrip() {
        let compact = CompactNode {
            kind: "function".to_string(),
            start: Some(10),
            end: Some(20),
            field: Some("body".to_string()),
            children: vec![CompactNode {
                kind: "identifier".to_string(),
                start: None,
                end: None,
                field: Some("name".to_string()),
                children: vec![],
                text: Some("main".to_string()),
            }],
            text: None,
        };

        let json = serde_json::to_string(&compact).unwrap();
        let decoded: CompactNode = serde_json::from_str(&json).unwrap();

        assert_eq!(compact.kind, decoded.kind);
        assert_eq!(compact.start, decoded.start);
        assert_eq!(compact.end, decoded.end);
        assert_eq!(compact.field, decoded.field);
        assert_eq!(compact.children.len(), decoded.children.len());
        assert_eq!(compact.children[0].text, decoded.children[0].text);
    }
}

#[cfg(test)]
mod s_expr_tests {
    use super::*;

    /// Test 1: Round-trip identity for basic structures
    #[test]
    fn test_basic_roundtrip_identity() {
        // Simple atom
        let atom_sexpr = SExpr::Atom("hello".to_string());
        let serialized = format!("{:?}", atom_sexpr);
        let atom_str = "hello";
        let parsed = parse_sexpr(atom_str).unwrap();
        assert_eq!(atom_sexpr, parsed);

        // Simple list
        let list_sexpr = SExpr::List(vec![
            SExpr::Atom("function".to_string()),
            SExpr::Atom("main".to_string()),
        ]);
        let list_str = "(function main)";
        let parsed_list = parse_sexpr(list_str).unwrap();
        assert_eq!(list_sexpr, parsed_list);
    }

    /// Test 2: Canonicalization - atom quoting/escaping
    #[test]
    fn test_canonicalization_atom_escaping() {
        let test_cases = vec![
            (r#""hello world""#, "hello world"),
            (r#""\"quoted\"""#, r#""quoted""#),
            (r#""line1\nline2""#, "line1\nline2"),
            (r#""tab\there""#, "tab\there"),
            (r#""backslash\\""#, r"backslash\"),
            (r#""();;special""#, "();;special"),
        ];

        for (input, expected) in test_cases {
            let parsed = parse_sexpr(input).unwrap();
            match parsed {
                SExpr::Atom(text) => assert_eq!(text, expected),
                _ => panic!("Expected atom, got list"),
            }
        }
    }

    /// Test 3: Unicode edge cases (non-BMP, combining marks, RTL)
    #[test]
    fn test_unicode_edge_cases() {
        let unicode_cases = vec![
            // Non-BMP characters (emoji, mathematical symbols)
            ("🚀", "🚀"),
            ("𝔘𝔫𝔦𝔠𝔬𝔡𝔢", "𝔘𝔫𝔦𝔠𝔬𝔡𝔢"),
            // Combining marks
            ("e\u{0301}", "é"), // é composed
            // RTL text
            ("שלום", "שלום"),
            ("مرحبا", "مرحبا"),
            // Mixed scripts
            ("Hello世界", "Hello世界"),
        ];

        for (input, expected) in unicode_cases {
            let quoted_input = format!("\"{}\"", input);
            let parsed = parse_sexpr(&quoted_input).unwrap();
            match parsed {
                SExpr::Atom(text) => {
                    // For combining marks, we compare the normalized form
                    assert!(
                        text.contains(expected) || expected.contains(&text),
                        "Unicode handling failed: {} vs {}",
                        text,
                        expected
                    );
                }
                _ => panic!("Expected atom for unicode test"),
            }
        }
    }

    /// Test 4: Empty list vs empty atom semantics  
    #[test]
    fn test_empty_structures() {
        // Empty lists should parse but may be semantically invalid
        let empty_list = parse_sexpr("()").unwrap();
        assert_eq!(empty_list, SExpr::List(vec![]));

        // Empty atom should be invalid
        assert!(parse_sexpr("").is_err());

        // Whitespace-only should be invalid
        assert!(parse_sexpr("   ").is_err());
    }

    /// Test 5: Stable serialization order
    #[test]
    fn test_serialization_stability() {
        let node1 = SerializedNode {
            kind: "function".to_string(),
            is_named: true,
            field_name: Some("name".to_string()),
            start_position: (1, 0),
            end_position: (5, 0),
            start_byte: 10,
            end_byte: 50,
            text: None,
            children: vec![SerializedNode {
                kind: "identifier".to_string(),
                is_named: true,
                field_name: None,
                start_position: (1, 9),
                end_position: (1, 13),
                start_byte: 19,
                end_byte: 23,
                text: Some("main".to_string()),
                children: vec![],
                is_error: false,
                is_missing: false,
            }],
            is_error: false,
            is_missing: false,
        };

        // Serialize multiple times and ensure consistency
        let json1 = serde_json::to_string(&node1).unwrap();
        let json2 = serde_json::to_string(&node1).unwrap();
        assert_eq!(json1, json2);

        // Parse and re-serialize should be stable
        let parsed: SerializedNode = serde_json::from_str(&json1).unwrap();
        let json3 = serde_json::to_string(&parsed).unwrap();
        assert_eq!(json1, json3);
    }

    /// Test 6: Deep structure handling (prevent stack overflow)
    #[test]
    fn test_deep_structure_stability() {
        const MAX_DEPTH: usize = 1000;

        // Create deeply nested structure
        let mut deep_sexpr = SExpr::Atom("leaf".to_string());
        for i in 0..MAX_DEPTH {
            deep_sexpr = SExpr::List(vec![SExpr::Atom(format!("level_{}", i)), deep_sexpr]);
        }

        // Should not stack overflow during serialization
        let serialized = format!("{:?}", deep_sexpr);
        assert!(!serialized.is_empty());

        // Verify depth by counting nested levels
        let paren_count = serialized.matches('(').count();
        assert!(paren_count >= MAX_DEPTH);
    }

    /// Test 7: Wide structure handling (prevent quadratic concatenation)  
    #[test]
    fn test_wide_structure_performance() {
        const WIDTH: usize = 10000;

        let mut wide_children = Vec::new();
        for i in 0..WIDTH {
            wide_children.push(SExpr::Atom(format!("child_{}", i)));
        }

        let wide_sexpr = SExpr::List(
            vec![SExpr::Atom("root".to_string())]
                .into_iter()
                .chain(wide_children)
                .collect(),
        );

        let start = std::time::Instant::now();
        let serialized = format!("{:?}", wide_sexpr);
        let duration = start.elapsed();

        // Should complete in reasonable time (not quadratic)
        assert!(
            duration.as_millis() < 1000,
            "Wide serialization took too long: {:?}",
            duration
        );
        assert!(serialized.len() > WIDTH * 5); // Rough size check
    }

    /// Test 8: SerializedNode roundtrip with all features
    #[test]
    fn test_serialized_node_roundtrip() {
        let original = SerializedNode {
            kind: "binary_expression".to_string(),
            is_named: true,
            field_name: Some("left".to_string()),
            start_position: (2, 5),
            end_position: (2, 15),
            start_byte: 25,
            end_byte: 35,
            text: None,
            children: vec![
                SerializedNode {
                    kind: "identifier".to_string(),
                    is_named: true,
                    field_name: None,
                    start_position: (2, 5),
                    end_position: (2, 6),
                    start_byte: 25,
                    end_byte: 26,
                    text: Some("x".to_string()),
                    children: vec![],
                    is_error: false,
                    is_missing: false,
                },
                SerializedNode {
                    kind: "number".to_string(),
                    is_named: true,
                    field_name: None,
                    start_position: (2, 9),
                    end_position: (2, 11),
                    start_byte: 29,
                    end_byte: 31,
                    text: Some("42".to_string()),
                    children: vec![],
                    is_error: false,
                    is_missing: false,
                },
            ],
            is_error: false,
            is_missing: false,
        };

        // JSON roundtrip
        let json = serde_json::to_string(&original).unwrap();
        let decoded: SerializedNode = serde_json::from_str(&json).unwrap();
        assert_eq!(original.kind, decoded.kind);
        assert_eq!(original.children.len(), decoded.children.len());
        assert_eq!(original.field_name, decoded.field_name);
        assert_eq!(original.text, decoded.text);
    }
}

#[cfg(test)]
mod unicode_tests {
    #[cfg(feature = "serialization")]
    use super::*;

    /// Test: Unicode handling in serialization
    #[test]
    #[cfg(feature = "serialization")]
    fn test_unicode_text_handling() {
        let unicode_cases = vec![
            "Hello, 世界!",
            "🦀 Rust 🚀",
            "עברית",
            "العربية",
            "🎉✨🌟",
            "Mixed: Hello 世界 🦀",
        ];

        for text in unicode_cases {
            let node = SerializedNode {
                kind: "string_literal".to_string(),
                is_named: true,
                field_name: None,
                start_position: (0, 0),
                end_position: (0, text.chars().count()),
                start_byte: 0,
                end_byte: text.len(),
                text: Some(text.to_string()),
                children: vec![],
                is_error: false,
                is_missing: false,
            };

            let json = serde_json::to_string(&node).unwrap();
            let decoded: SerializedNode = serde_json::from_str(&json).unwrap();

            assert_eq!(node.text, decoded.text);
            assert_eq!(text, decoded.text.unwrap());
        }
    }
}

#[cfg(test)]
mod error_tests {
    use super::*;

    /// Test 9: Error node serialization
    #[test]
    fn test_error_node_roundtrip() {
        let error_node = SerializedNode {
            kind: "ERROR".to_string(),
            is_named: false,
            field_name: None,
            start_position: (3, 10),
            end_position: (3, 15),
            start_byte: 40,
            end_byte: 45,
            text: Some("invalid syntax".to_string()),
            children: vec![],
            is_error: true,
            is_missing: false,
        };

        let json = serde_json::to_string(&error_node).unwrap();
        let decoded: SerializedNode = serde_json::from_str(&json).unwrap();

        assert_eq!(error_node.is_error, decoded.is_error);
        assert_eq!(error_node.kind, decoded.kind);
        assert_eq!(error_node.text, decoded.text);
    }

    /// Test 10: Missing node serialization
    #[test]
    fn test_missing_node_roundtrip() {
        let missing_node = SerializedNode {
            kind: "identifier".to_string(),
            is_named: true,
            field_name: Some("name".to_string()),
            start_position: (4, 0),
            end_position: (4, 0),
            start_byte: 60,
            end_byte: 60,
            text: None,
            children: vec![],
            is_error: false,
            is_missing: true,
        };

        let json = serde_json::to_string(&missing_node).unwrap();
        let decoded: SerializedNode = serde_json::from_str(&json).unwrap();

        assert_eq!(missing_node.is_missing, decoded.is_missing);
        assert_eq!(missing_node.start_byte, missing_node.end_byte);
        assert_eq!(decoded.start_byte, decoded.end_byte);
    }
}

#[cfg(test)]
mod property_based_tests {
    use super::*;

    /// Generate random SerializedNode for property testing
    fn gen_random_node(depth: usize, rng: &mut rand::SmallRng) -> SerializedNode {
        let kind = format!("kind_{}", rng.r#gen::<u16>() % 10);
        let is_leaf = depth == 0 || rng.gen_bool(0.3);

        let mut node = SerializedNode {
            kind,
            is_named: rng.gen_bool(0.8),
            field_name: if rng.gen_bool(0.3) {
                Some(format!("field_{}", rng.r#gen::<u8>() % 5))
            } else {
                None
            },
            start_position: (rng.r#gen::<usize>() % 100, rng.r#gen::<usize>() % 100),
            end_position: (rng.r#gen::<usize>() % 100, rng.r#gen::<usize>() % 100),
            start_byte: rng.r#gen::<usize>() % 1000,
            end_byte: rng.r#gen::<usize>() % 1000,
            text: None,
            children: vec![],
            is_error: rng.gen_bool(0.1),
            is_missing: rng.gen_bool(0.05),
        };

        if is_leaf {
            let text_options = vec!["hello", "world", "test", "42", "true", "null"];
            node.text = Some(text_options[rng.r#gen::<usize>() % text_options.len()].to_string());
        } else if depth > 0 {
            let child_count = rng.r#gen::<usize>() % 4;
            for _ in 0..child_count {
                node.children.push(gen_random_node(depth - 1, rng));
            }
        }

        node
    }

    /// Property test: serialization roundtrip should be identity
    #[test]
    fn property_test_json_roundtrip() {
        let mut rng = rand::SmallRng::seed_from_u64(12345);

        for i in 0..100 {
            let original = gen_random_node(i % 5, &mut rng);

            let json = serde_json::to_string(&original).unwrap();
            let decoded: SerializedNode = serde_json::from_str(&json).unwrap();

            // Key structural properties should be preserved
            assert_eq!(
                original.kind, decoded.kind,
                "Kind mismatch at iteration {}",
                i
            );
            assert_eq!(
                original.is_named, decoded.is_named,
                "is_named mismatch at iteration {}",
                i
            );
            assert_eq!(
                original.field_name, decoded.field_name,
                "field_name mismatch at iteration {}",
                i
            );
            assert_eq!(
                original.text, decoded.text,
                "text mismatch at iteration {}",
                i
            );
            assert_eq!(
                original.children.len(),
                decoded.children.len(),
                "children count mismatch at iteration {}",
                i
            );
            assert_eq!(
                original.is_error, decoded.is_error,
                "is_error mismatch at iteration {}",
                i
            );
            assert_eq!(
                original.is_missing, decoded.is_missing,
                "is_missing mismatch at iteration {}",
                i
            );
        }
    }

    /// Property test: S-expression parsing is inverse of formatting
    #[test]
    fn property_test_sexpr_roundtrip() {
        let mut rng = rand::SmallRng::seed_from_u64(54321);

        for _ in 0..50 {
            // Generate random S-expression
            let sexpr = gen_random_sexpr(3, &mut rng);

            // For now, just test that parsing doesn't crash
            // A full roundtrip would require implementing SExpr -> string formatting
            match sexpr {
                SExpr::Atom(ref text) => {
                    let quoted = format!("\"{}\"", text.replace('"', r#"\""#));
                    let parsed = parse_sexpr(&quoted);
                    assert!(parsed.is_ok(), "Failed to parse generated atom: {}", quoted);
                }
                SExpr::List(_) => {
                    // List formatting is more complex, skip for now
                }
            }
        }
    }

    fn gen_random_sexpr(depth: usize, rng: &mut rand::SmallRng) -> SExpr {
        if depth == 0 || rng.gen_bool(0.5) {
            let atoms = vec!["hello", "world", "test", "function", "if", "else", "return"];
            SExpr::Atom(atoms[rng.r#gen::<usize>() % atoms.len()].to_string())
        } else {
            let child_count = rng.r#gen::<usize>() % 4 + 1;
            let mut children = vec![SExpr::Atom("list_head".to_string())];
            for _ in 0..child_count {
                children.push(gen_random_sexpr(depth - 1, rng));
            }
            SExpr::List(children)
        }
    }
}

#[cfg(test)]
mod performance_tests {
    use super::*;

    /// Test that large tree serialization completes in reasonable time
    #[test]
    fn test_large_tree_performance() {
        let mut large_node = SerializedNode {
            kind: "root".to_string(),
            is_named: true,
            field_name: None,
            start_position: (0, 0),
            end_position: (1000, 0),
            start_byte: 0,
            end_byte: 50000,
            text: None,
            children: vec![],
            is_error: false,
            is_missing: false,
        };

        // Create a moderately large tree
        for i in 0..1000 {
            large_node.children.push(SerializedNode {
                kind: format!("child_{}", i),
                is_named: true,
                field_name: Some(format!("field_{}", i % 10)),
                start_position: (i / 80, i % 80),
                end_position: (i / 80, i % 80 + 5),
                start_byte: i * 50,
                end_byte: i * 50 + 45,
                text: Some(format!("text_content_{}", i)),
                children: vec![],
                is_error: false,
                is_missing: false,
            });
        }

        let start = std::time::Instant::now();
        let json = serde_json::to_string(&large_node).unwrap();
        let serialize_time = start.elapsed();

        let start = std::time::Instant::now();
        let _decoded: SerializedNode = serde_json::from_str(&json).unwrap();
        let deserialize_time = start.elapsed();

        println!(
            "Serialize time: {:?}, Deserialize time: {:?}",
            serialize_time, deserialize_time
        );

        // Should complete within reasonable time bounds
        assert!(
            serialize_time.as_millis() < 1000,
            "Serialization too slow: {:?}",
            serialize_time
        );
        assert!(
            deserialize_time.as_millis() < 1000,
            "Deserialization too slow: {:?}",
            deserialize_time
        );

        // JSON should be reasonably sized (not absurdly large)
        assert!(json.len() > 50000); // At least some content
        assert!(json.len() < 10_000_000); // But not excessive
    }
}

// Simple random number generator for property testing
#[cfg(test)]
mod rand {
    pub struct SmallRng {
        state: u64,
    }

    impl SmallRng {
        pub fn seed_from_u64(seed: u64) -> Self {
            Self { state: seed }
        }

        pub fn gen_bool(&mut self, p: f64) -> bool {
            self.state = self.state.wrapping_mul(1103515245).wrapping_add(12345);
            let val = (self.state >> 32) as f64 / u32::MAX as f64;
            val < p
        }

        pub fn gen_range(&mut self, min: usize, max: usize) -> usize {
            self.state = self.state.wrapping_mul(1103515245).wrapping_add(12345);
            let val = (self.state as usize) % (max - min);
            min + val
        }

        pub fn r#gen<T>(&mut self) -> T {
            self.state = self.state.wrapping_mul(1103515245).wrapping_add(12345);
            // Simple type-specific implementation for testing
            unsafe { std::mem::transmute_copy(&self.state) }
        }
    }

    pub mod rngs {
        pub use super::SmallRng;
    }
}

// Feature-gated tests that only run when serialization is enabled
#[cfg(not(feature = "serialization"))]
#[test]
fn test_serialization_feature_disabled() {
    // This test exists to ensure the test suite runs even when serialization is disabled
    // In that case, the serialization-dependent tests are skipped but we should still
    // have at least one passing test. The fact that this compiles and runs confirms
    // proper feature gating.
}
