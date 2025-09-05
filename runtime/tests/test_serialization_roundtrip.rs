//! Comprehensive serialization roundtrip tests
//!
//! These tests ensure that:
//! 1. Serialization APIs work correctly with the rust-sitter types
//! 2. SerializedNode roundtrips through JSON properly
//! 3. TreeSerializer produces valid output
//! 4. Unicode edge cases are handled correctly
//! 5. Various serialization formats work as expected

#[cfg(feature = "serialization")]
use rust_sitter::serialization::*;

#[cfg(test)]
mod roundtrip_tests {
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
mod unicode_tests {
    #[cfg(feature = "serialization")]
    use super::*;

    /// Test 5: Unicode handling in serialization
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
    #[cfg(feature = "serialization")]
    use super::*;

    /// Test 6: Error and missing node serialization
    #[test]
    #[cfg(feature = "serialization")]
    fn test_error_and_missing_nodes() {
        let error_node = SerializedNode {
            kind: "ERROR".to_string(),
            is_named: false,
            field_name: None,
            start_position: (1, 5),
            end_position: (1, 10),
            start_byte: 15,
            end_byte: 20,
            text: Some("invalid".to_string()),
            children: vec![],
            is_error: true,
            is_missing: false,
        };

        let missing_node = SerializedNode {
            kind: "identifier".to_string(),
            is_named: true,
            field_name: Some("name".to_string()),
            start_position: (2, 0),
            end_position: (2, 0),
            start_byte: 30,
            end_byte: 30,
            text: None,
            children: vec![],
            is_error: false,
            is_missing: true,
        };

        // Test error node roundtrip
        let error_json = serde_json::to_string(&error_node).unwrap();
        let decoded_error: SerializedNode = serde_json::from_str(&error_json).unwrap();

        assert!(decoded_error.is_error);
        assert!(!decoded_error.is_missing);
        assert_eq!(error_node.kind, decoded_error.kind);

        // Test missing node roundtrip
        let missing_json = serde_json::to_string(&missing_node).unwrap();
        let decoded_missing: SerializedNode = serde_json::from_str(&missing_json).unwrap();

        assert!(!decoded_missing.is_error);
        assert!(decoded_missing.is_missing);
        assert_eq!(missing_node.start_byte, decoded_missing.start_byte);
        assert_eq!(missing_node.end_byte, decoded_missing.end_byte);
    }
}

#[cfg(test)]
mod performance_tests {
    #[cfg(feature = "serialization")]
    use super::*;

    /// Test 7: Large tree serialization performance
    #[test]
    #[cfg(feature = "serialization")]
    fn test_large_tree_serialization() {
        let mut large_node = SerializedNode {
            kind: "root".to_string(),
            is_named: true,
            field_name: None,
            start_position: (0, 0),
            end_position: (1000, 0),
            start_byte: 0,
            end_byte: 10000,
            text: None,
            children: vec![],
            is_error: false,
            is_missing: false,
        };

        // Create a large number of child nodes
        for i in 0..1000 {
            large_node.children.push(SerializedNode {
                kind: format!("node_{}", i),
                is_named: true,
                field_name: Some(format!("field_{}", i)),
                start_position: (i / 100, i % 100),
                end_position: (i / 100, (i % 100) + 5),
                start_byte: i * 10,
                end_byte: i * 10 + 5,
                text: Some(format!("text_{}", i)),
                children: vec![],
                is_error: false,
                is_missing: false,
            });
        }

        // Serialize (should complete in reasonable time)
        let start = std::time::Instant::now();
        let json = serde_json::to_string(&large_node).unwrap();
        let serialize_time = start.elapsed();

        // Deserialize
        let start = std::time::Instant::now();
        let decoded: SerializedNode = serde_json::from_str(&json).unwrap();
        let deserialize_time = start.elapsed();

        // Basic verification
        assert_eq!(large_node.kind, decoded.kind);
        assert_eq!(large_node.children.len(), decoded.children.len());

        // Performance checks (should be sub-second for 1000 nodes)
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

// Feature-gated tests that only run when serialization is enabled
#[cfg(not(feature = "serialization"))]
#[test]
fn test_serialization_feature_disabled() {
    // This test exists to ensure the test suite runs even when serialization is disabled
    // In that case, the serialization-dependent tests are skipped but we should still
    // have at least one passing test. The fact that this compiles and runs confirms
    // proper feature gating.
}
