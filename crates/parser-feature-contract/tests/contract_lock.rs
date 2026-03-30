//! Contract lock test - verifies that public API remains stable.

use adze_parser_feature_contract::{ParserBackend, ParserFeatureProfile};

/// Verify all public types exist and have expected structure.
#[test]
fn test_contract_lock_types() {
    // Verify ParserBackend enum exists with expected variants
    let _tree_sitter = ParserBackend::TreeSitter;
    let _pure_rust = ParserBackend::PureRust;
    let _glr = ParserBackend::GLR;

    // Verify ParserFeatureProfile struct exists
    let _profile = ParserFeatureProfile::current();
}

/// Verify ParserBackend methods exist.
#[test]
fn test_contract_lock_backend_methods() {
    // Verify name method exists
    let name = ParserBackend::TreeSitter.name();
    assert!(!name.is_empty());

    // Verify is_glr method exists
    let _ = ParserBackend::GLR.is_glr();

    // Verify is_pure_rust method exists
    let _ = ParserBackend::PureRust.is_pure_rust();

    // Verify select method exists
    let _ = ParserBackend::select(false);
}

/// Verify ParserFeatureProfile methods exist.
#[test]
fn test_contract_lock_profile_methods() {
    // Verify current method exists
    let _ = ParserFeatureProfile::current();

    // Verify resolve_backend method exists
    let profile = ParserFeatureProfile::current();
    let _ = profile.resolve_backend(false);

    // Verify has_glr method exists
    let _ = profile.has_glr();
}

/// Verify trait implementations exist.
#[test]
fn test_contract_lock_traits() {
    // Verify Display trait is implemented for ParserBackend
    let display = format!("{}", ParserBackend::TreeSitter);
    assert!(!display.is_empty());

    // Verify Debug trait is implemented for ParserFeatureProfile
    let profile = ParserFeatureProfile::current();
    let debug = format!("{:?}", profile);
    assert!(!debug.is_empty());

    // Verify PartialEq trait is implemented for ParserBackend
    assert_eq!(ParserBackend::TreeSitter, ParserBackend::TreeSitter);

    // Verify PartialEq trait is implemented for ParserFeatureProfile
    let a = ParserFeatureProfile::current();
    let b = ParserFeatureProfile::current();
    assert_eq!(a, b);
}
