//! BDD tests for feature-policy-contract facade crate.
//!
//! These tests verify the public API behavior using Given/When/Then style.

use adze_feature_policy_contract::{ParserBackend, ParserFeatureProfile};

// =============================================================================
// ParserBackend Tests
// =============================================================================

#[test]
fn given_tree_sitter_backend_when_checking_is_glr_then_returns_false() {
    // Given
    let backend = ParserBackend::TreeSitter;

    // When
    let result = backend.is_glr();

    // Then
    assert!(!result);
}

#[test]
fn given_pure_rust_backend_when_checking_is_glr_then_returns_false() {
    // Given
    let backend = ParserBackend::PureRust;

    // When
    let result = backend.is_glr();

    // Then
    assert!(!result);
}

#[test]
fn given_glr_backend_when_checking_is_glr_then_returns_true() {
    // Given
    let backend = ParserBackend::GLR;

    // When
    let result = backend.is_glr();

    // Then
    assert!(result);
}

#[test]
fn given_tree_sitter_backend_when_checking_is_pure_rust_then_returns_false() {
    // Given
    let backend = ParserBackend::TreeSitter;

    // When
    let result = backend.is_pure_rust();

    // Then
    assert!(!result);
}

#[test]
fn given_pure_rust_backend_when_checking_is_pure_rust_then_returns_true() {
    // Given
    let backend = ParserBackend::PureRust;

    // When
    let result = backend.is_pure_rust();

    // Then
    assert!(result);
}

#[test]
fn given_glr_backend_when_checking_is_pure_rust_then_returns_true() {
    // Given
    let backend = ParserBackend::GLR;

    // When
    let result = backend.is_pure_rust();

    // Then
    assert!(result);
}

#[test]
fn given_any_backend_when_getting_name_then_returns_non_empty_string() {
    // Given
    let backends = [
        ParserBackend::TreeSitter,
        ParserBackend::PureRust,
        ParserBackend::GLR,
    ];

    // When / Then
    for backend in backends {
        assert!(!backend.name().is_empty());
    }
}

#[test]
fn given_tree_sitter_backend_when_displaying_then_shows_c_runtime() {
    // Given
    let backend = ParserBackend::TreeSitter;

    // When
    let display = format!("{}", backend);

    // Then
    assert!(display.contains("tree-sitter"));
    assert!(display.contains("C runtime"));
}

#[test]
fn given_pure_rust_backend_when_displaying_then_shows_pure_rust_lr() {
    // Given
    let backend = ParserBackend::PureRust;

    // When
    let display = format!("{}", backend);

    // Then
    assert!(display.contains("pure-Rust"));
    assert!(display.contains("LR"));
}

#[test]
fn given_glr_backend_when_displaying_then_shows_pure_rust_glr() {
    // Given
    let backend = ParserBackend::GLR;

    // When
    let display = format!("{}", backend);

    // Then
    assert!(display.contains("pure-Rust"));
    assert!(display.contains("GLR"));
}

#[test]
fn given_backend_when_cloning_then_equals_original() {
    // Given
    let original = ParserBackend::GLR;

    // When
    let cloned = original;

    // Then
    assert_eq!(original, cloned);
}

// =============================================================================
// ParserFeatureProfile Tests
// =============================================================================

#[test]
fn given_current_profile_when_accessing_then_returns_valid_profile() {
    // Given / When
    let profile = ParserFeatureProfile::current();

    // Then
    let _ = format!("{:?}", profile);
    let _ = format!("{}", profile);
}

#[test]
fn given_current_profile_when_resolving_backend_without_conflicts_then_returns_valid_backend() {
    // Given
    let profile = ParserFeatureProfile::current();

    // When
    let backend = profile.resolve_backend(false);

    // Then
    assert!(!backend.name().is_empty());
}

#[test]
fn given_current_profile_when_checking_has_methods_then_returns_booleans() {
    // Given
    let profile = ParserFeatureProfile::current();

    // When / Then - These should return booleans without panicking
    let _ = profile.has_pure_rust();
    let _ = profile.has_glr();
    let _ = profile.has_tree_sitter();
}

#[test]
fn given_profile_with_glr_when_resolving_backend_then_returns_glr() {
    // Given
    let profile = ParserFeatureProfile {
        pure_rust: true,
        tree_sitter_standard: false,
        tree_sitter_c2rust: false,
        glr: true,
    };

    // When
    let backend = profile.resolve_backend(false);

    // Then
    assert!(backend.is_glr());
}

#[test]
fn given_profile_with_glr_and_conflicts_when_resolving_backend_then_returns_glr() {
    // Given
    let profile = ParserFeatureProfile {
        pure_rust: true,
        tree_sitter_standard: false,
        tree_sitter_c2rust: false,
        glr: true,
    };

    // When
    let backend = profile.resolve_backend(true);

    // Then
    assert!(backend.is_glr());
}
