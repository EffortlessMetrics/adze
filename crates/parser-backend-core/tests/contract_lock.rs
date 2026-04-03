//! Contract lock test - verifies that public API remains stable.

use adze_parser_backend_core::ParserBackend;

/// Verify all public types exist and have expected structure.
#[test]
fn test_contract_lock_types() {
    // Verify ParserBackend enum exists with all variants
    let tree_sitter = ParserBackend::TreeSitter;
    let pure_rust = ParserBackend::PureRust;
    let glr = ParserBackend::GLR;

    // Verify Debug trait is implemented
    let _debug = format!("{tree_sitter:?}");

    // Verify Clone trait is implemented
    let _cloned = tree_sitter;

    // Verify Copy trait is implemented
    let _copied: ParserBackend = tree_sitter;

    // Verify PartialEq trait is implemented
    assert_eq!(pure_rust, pure_rust);
    assert_ne!(tree_sitter, glr);

    // Verify Eq trait is implemented
    assert_eq!(glr, glr);

    // Verify Display trait is implemented
    let _display = format!("{tree_sitter}");
}

/// Verify all public methods exist with expected signatures.
#[test]
fn test_contract_lock_methods() {
    // Verify select method exists
    let _backend = ParserBackend::select(false);

    // Verify is_glr method exists
    assert!(ParserBackend::GLR.is_glr());
    assert!(!ParserBackend::TreeSitter.is_glr());
    assert!(!ParserBackend::PureRust.is_glr());

    // Verify is_pure_rust method exists
    assert!(ParserBackend::PureRust.is_pure_rust());
    assert!(ParserBackend::GLR.is_pure_rust());
    assert!(!ParserBackend::TreeSitter.is_pure_rust());

    // Verify name method exists and returns &'static str
    let name: &'static str = ParserBackend::TreeSitter.name();
    assert!(!name.is_empty());

    assert_eq!(ParserBackend::TreeSitter.name(), "tree-sitter C runtime");
    assert_eq!(ParserBackend::PureRust.name(), "pure-Rust LR parser");
    assert_eq!(ParserBackend::GLR.name(), "pure-Rust GLR parser");
}

/// Verify Display implementation matches name method.
#[test]
fn test_contract_lock_display() {
    for backend in [
        ParserBackend::TreeSitter,
        ParserBackend::PureRust,
        ParserBackend::GLR,
    ] {
        assert_eq!(format!("{backend}"), backend.name());
    }
}
