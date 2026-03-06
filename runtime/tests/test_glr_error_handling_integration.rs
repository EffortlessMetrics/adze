//! Integration tests for GLR parser error handling
//!
//! This test suite verifies that the GLR parser properly handles errors through
//! its public API, focusing on the error types and display functionality.

#[cfg(feature = "ts-compat")]
use adze::adze_ir as ir;
use adze::glr_parser::GLRError;

#[cfg(not(feature = "ts-compat"))]
use adze_ir as ir;

use ir::ProductionId;

/// Test error message formatting and display
#[test]
fn test_glr_error_display() {
    let error = GLRError::ComplexSymbolNotNormalized {
        symbol_type: "Optional".to_string(),
        production_id: ProductionId(42),
        position: 3,
    };

    let error_msg = format!("{}", error);
    assert!(error_msg.contains("Complex symbol 'Optional' not normalized"));
    assert!(error_msg.contains("rule ProductionId(42)"));
    assert!(error_msg.contains("position 3"));
    assert!(error_msg.contains("normalized before GLR parsing"));
}

/// Test different symbol type error messages
#[test]
fn test_different_symbol_types() {
    let test_cases = vec![
        ("Optional", ProductionId(1)),
        ("Repeat", ProductionId(2)),
        ("RepeatOne", ProductionId(3)),
        ("Choice", ProductionId(4)),
        ("Sequence", ProductionId(5)),
        ("Epsilon", ProductionId(6)),
    ];

    for (symbol_type, production_id) in test_cases {
        let error = GLRError::ComplexSymbolNotNormalized {
            symbol_type: symbol_type.to_string(),
            production_id,
            position: 0,
        };

        let error_msg = format!("{}", error);
        assert!(error_msg.contains(&format!("Complex symbol '{}' not normalized", symbol_type)));
        assert!(error_msg.contains(&format!("rule {:?}", production_id)));
    }
}

/// Test error context and source chain
#[test]
fn test_glr_error_source_chain() {
    let error = GLRError::ComplexSymbolNotNormalized {
        symbol_type: "Choice".to_string(),
        production_id: ProductionId(7),
        position: 0,
    };

    // Verify it implements Error trait properly
    let _source = std::error::Error::source(&error);
    // ComplexSymbolNotNormalized doesn't have an underlying source, so this should be None
    assert!(_source.is_none());
}

/// Test error trait implementation
#[test]
fn test_error_trait_implementation() {
    let error = GLRError::ComplexSymbolNotNormalized {
        symbol_type: "Sequence".to_string(),
        production_id: ProductionId(100),
        position: 5,
    };

    // Should implement Debug
    let debug_str = format!("{:?}", error);
    assert!(debug_str.contains("ComplexSymbolNotNormalized"));

    // Should implement std::error::Error
    let error_ref: &dyn std::error::Error = &error;
    let _display = error_ref.to_string();
}

/// Test that error messages are helpful for developers
#[test]
fn test_error_messages_are_helpful() {
    let error = GLRError::ComplexSymbolNotNormalized {
        symbol_type: "Optional".to_string(),
        production_id: ProductionId(42),
        position: 3,
    };

    let error_msg = format!("{}", error);

    // Should include actionable information
    assert!(error_msg.contains("normalized before GLR parsing"));
    assert!(error_msg.contains("position 3")); // Helps locate the problem
    assert!(error_msg.contains("rule ProductionId(42)")); // Identifies the rule
    assert!(error_msg.contains("Optional")); // Identifies the symbol type
}
