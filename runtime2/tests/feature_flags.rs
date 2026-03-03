//! Feature flag behavior tests.
//!
//! Validates that the runtime behaves correctly under the current feature set,
//! and that compile-time gates produce the expected API surface.

use adze_runtime::Tree;

// ---------------------------------------------------------------------------
// EditError is only available with incremental_glr
// ---------------------------------------------------------------------------

#[cfg(feature = "incremental_glr")]
mod incremental_glr_enabled {
    use adze_runtime::{EditError, InputEdit, Point, Tree};

    #[test]
    fn edit_error_type_is_available() {
        // EditError variants are accessible
        let err = EditError::ArithmeticOverflow;
        assert_eq!(
            err.to_string(),
            "Arithmetic overflow during tree edit operation"
        );

        let err2 = EditError::ArithmeticUnderflow;
        assert_eq!(
            err2.to_string(),
            "Arithmetic underflow during tree edit operation"
        );

        let err3 = EditError::InvalidRange {
            start: 10,
            old_end: 5,
        };
        assert!(err3.to_string().contains("Invalid edit range"));
    }

    #[test]
    fn edit_error_is_std_error() {
        let err: Box<dyn std::error::Error> = Box::new(EditError::ArithmeticOverflow);
        assert!(!err.to_string().is_empty());
    }

    #[test]
    fn edit_error_clone_and_eq() {
        let a = EditError::ArithmeticOverflow;
        let b = a.clone();
        assert_eq!(a, b);
    }

    #[test]
    fn tree_edit_is_available() {
        let mut tree = Tree::new_stub();
        let edit = InputEdit {
            start_byte: 0,
            old_end_byte: 0,
            new_end_byte: 5,
            start_position: Point::new(0, 0),
            old_end_position: Point::new(0, 0),
            new_end_position: Point::new(0, 5),
        };
        // Zero-length insertion on empty stub should succeed
        tree.edit(&edit).unwrap();
    }

    #[test]
    fn edit_invalid_range_returns_error() {
        let mut tree = Tree::new_stub();
        let edit = InputEdit {
            start_byte: 10,
            old_end_byte: 5, // invalid: less than start
            new_end_byte: 15,
            start_position: Point::new(0, 10),
            old_end_position: Point::new(0, 5),
            new_end_position: Point::new(0, 15),
        };
        let result = tree.edit(&edit);
        assert!(matches!(result, Err(EditError::InvalidRange { .. })));
    }
}

// ---------------------------------------------------------------------------
// Without incremental_glr, Tree::edit is not available (compile-time check).
// We verify parsing still works.
// ---------------------------------------------------------------------------

#[test]
fn tree_stub_works_regardless_of_features() {
    let tree = Tree::new_stub();
    assert_eq!(tree.root_node().child_count(), 0);
    assert_eq!(tree.root_kind(), 0);
}

// ---------------------------------------------------------------------------
// GLR-core feature: parse table in Language
// ---------------------------------------------------------------------------

#[cfg(feature = "glr-core")]
mod glr_core_enabled {
    use adze_runtime::test_helpers::stub_language;

    #[test]
    fn stub_language_has_parse_table() {
        let lang = stub_language();
        // With glr-core, parse_table is Option<&'static ParseTable>
        assert!(lang.parse_table.is_some());
    }
}

#[cfg(not(feature = "glr-core"))]
mod glr_core_disabled {
    use adze_runtime::{Parser, test_helpers::stub_language};

    #[test]
    fn stub_language_builds_without_glr() {
        let lang = stub_language();
        // Without glr-core, parse_table is the local ParseTable struct
        assert_eq!(lang.parse_table.state_count, 0);
    }

    #[test]
    fn parse_fails_gracefully_without_glr_core() {
        let mut parser = Parser::new();
        let lang = stub_language();
        parser.set_language(lang).unwrap();
        let result = parser.parse(b"test", None);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not enabled"));
    }
}

// ---------------------------------------------------------------------------
// Governance / feature profile helpers are always available
// ---------------------------------------------------------------------------

#[test]
fn parser_feature_profile_is_available() {
    let _profile = adze_runtime::parser_feature_profile_for_current_runtime2();
}

#[test]
fn current_backend_helpers() {
    let _backend_no_conflicts = adze_runtime::current_backend_for(false);
    let _backend_with_conflicts = adze_runtime::current_backend_for(true);
    let _backend_r2 = adze_runtime::current_backend_for_runtime2(false);
}

// ---------------------------------------------------------------------------
// External scanner types are always available (feature only gates FFI structs)
// ---------------------------------------------------------------------------

#[test]
fn external_scanner_types_available() {
    use adze_runtime::ScanResult;
    let result = ScanResult {
        token_type: 1,
        bytes_consumed: 5,
    };
    assert_eq!(result.token_type, 1);
    assert_eq!(result.bytes_consumed, 5);
    let r2 = result; // Copy
    assert_eq!(r2, result);
}

// ---------------------------------------------------------------------------
// Pure-rust feature
// ---------------------------------------------------------------------------

#[cfg(feature = "pure-rust")]
mod pure_rust_enabled {
    use adze_runtime::Parser;

    #[test]
    fn parser_starts_in_lr_mode() {
        let parser = Parser::new();
        assert!(!parser.is_glr_mode());
    }
}
