//! BDD-style tests for parser-feature-contract crate.
//!
//! Tests follow the Given/When/Then pattern to verify public API behavior.

use adze_parser_feature_contract::{ParserBackend, ParserFeatureProfile};

#[test]
fn given_tree_sitter_backend_when_calling_name_then_returns_non_empty() {
    // Given / When
    let name = ParserBackend::TreeSitter.name();

    // Then
    assert!(!name.is_empty());
}

#[test]
fn given_pure_rust_backend_when_calling_name_then_returns_non_empty() {
    // Given / When
    let name = ParserBackend::PureRust.name();

    // Then
    assert!(!name.is_empty());
}

#[test]
fn given_glr_backend_when_calling_name_then_returns_non_empty() {
    // Given / When
    let name = ParserBackend::GLR.name();

    // Then
    assert!(!name.is_empty());
}

#[test]
fn given_glr_backend_when_checking_is_glr_then_returns_true() {
    // Given / When
    let result = ParserBackend::GLR.is_glr();

    // Then
    assert!(result);
}

#[test]
fn given_tree_sitter_backend_when_checking_is_glr_then_returns_false() {
    // Given / When
    let result = ParserBackend::TreeSitter.is_glr();

    // Then
    assert!(!result);
}

#[test]
fn given_pure_rust_backend_when_checking_is_pure_rust_then_returns_true() {
    // Given / When
    let result = ParserBackend::PureRust.is_pure_rust();

    // Then
    assert!(result);
}

#[test]
fn given_glr_backend_when_checking_is_pure_rust_then_returns_true() {
    // Given / When
    let result = ParserBackend::GLR.is_pure_rust();

    // Then
    assert!(result);
}

#[test]
fn given_tree_sitter_backend_when_checking_is_pure_rust_then_returns_false() {
    // Given / When
    let result = ParserBackend::TreeSitter.is_pure_rust();

    // Then
    assert!(!result);
}

#[test]
fn given_same_backends_when_comparing_equality_then_returns_true() {
    // Given
    let a = ParserBackend::TreeSitter;
    let b = ParserBackend::TreeSitter;

    // When
    let result = a == b;

    // Then
    assert!(result);
}

#[test]
fn given_different_backends_when_comparing_equality_then_returns_false() {
    // Given
    let a = ParserBackend::TreeSitter;
    let b = ParserBackend::GLR;

    // When
    let result = a == b;

    // Then
    assert!(!result);
}

#[test]
fn given_current_profile_when_calling_resolve_backend_then_returns_valid_backend() {
    // Given
    let profile = ParserFeatureProfile::current();

    // When
    let backend = profile.resolve_backend(false);

    // Then
    assert!(!backend.name().is_empty());
}

#[test]
fn given_current_profile_when_calling_has_glr_then_returns_boolean() {
    // Given
    let profile = ParserFeatureProfile::current();

    // When
    let has_glr = profile.has_glr();

    // Then
    // Verify the method returns a boolean value (type check only)
    let _: bool = has_glr;
}

#[test]
fn given_two_current_profiles_when_comparing_then_they_are_equal() {
    // Given
    let a = ParserFeatureProfile::current();
    let b = ParserFeatureProfile::current();

    // When
    let result = a == b;

    // Then
    assert!(result);
}

#[test]
fn given_profile_when_formatting_display_then_returns_non_empty() {
    // Given
    let profile = ParserFeatureProfile::current();

    // When
    let display = format!("{}", profile);

    // Then
    assert!(!display.is_empty());
}

#[test]
fn given_select_false_when_calling_select_then_returns_valid_backend() {
    // Given / When
    let backend = ParserBackend::select(false);

    // Then
    assert!(!backend.name().is_empty());
}

#[test]
fn given_select_true_when_calling_select_then_returns_valid_backend() {
    // Given / When
    let profile = ParserFeatureProfile::current();
    let select_result = std::panic::catch_unwind(|| ParserBackend::select(true));
    let profile_result = std::panic::catch_unwind(|| profile.resolve_backend(true));

    assert_eq!(select_result.is_ok(), profile_result.is_ok());

    if let (Ok(select_backend), Ok(profile_backend)) = (select_result, profile_result) {
        assert_eq!(select_backend, profile_backend);
        assert!(!select_backend.name().is_empty());
    }
}
