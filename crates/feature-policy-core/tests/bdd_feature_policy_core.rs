//! BDD tests for feature-policy-core crate.
//!
//! These tests verify the public API behavior using Given/When/Then style.

use adze_feature_policy_core::{ParserBackend, ParserFeatureProfile};

// =============================================================================
// ParserFeatureProfile Creation Tests
// =============================================================================

#[test]
fn given_no_flags_when_creating_profile_then_all_false() {
    // Given / When
    let profile = ParserFeatureProfile {
        pure_rust: false,
        tree_sitter_standard: false,
        tree_sitter_c2rust: false,
        glr: false,
    };

    // Then
    assert!(!profile.has_pure_rust());
    assert!(!profile.has_glr());
    assert!(!profile.has_tree_sitter());
}

#[test]
fn given_pure_rust_flag_only_when_creating_profile_then_has_pure_rust_true() {
    // Given / When
    let profile = ParserFeatureProfile {
        pure_rust: true,
        tree_sitter_standard: false,
        tree_sitter_c2rust: false,
        glr: false,
    };

    // Then
    assert!(profile.has_pure_rust());
    assert!(!profile.has_glr());
    assert!(!profile.has_tree_sitter());
}

#[test]
fn given_glr_flag_when_creating_profile_then_has_glr_true() {
    // Given / When
    let profile = ParserFeatureProfile {
        pure_rust: false,
        tree_sitter_standard: false,
        tree_sitter_c2rust: false,
        glr: true,
    };

    // Then
    assert!(profile.has_glr());
}

#[test]
fn given_tree_sitter_standard_when_creating_profile_then_has_tree_sitter_true() {
    // Given / When
    let profile = ParserFeatureProfile {
        pure_rust: false,
        tree_sitter_standard: true,
        tree_sitter_c2rust: false,
        glr: false,
    };

    // Then
    assert!(profile.has_tree_sitter());
}

#[test]
fn given_tree_sitter_c2rust_when_creating_profile_then_has_tree_sitter_true() {
    // Given / When
    let profile = ParserFeatureProfile {
        pure_rust: false,
        tree_sitter_standard: false,
        tree_sitter_c2rust: true,
        glr: false,
    };

    // Then
    assert!(profile.has_tree_sitter());
}

#[test]
fn given_both_tree_sitter_flags_when_creating_profile_then_has_tree_sitter_true() {
    // Given / When
    let profile = ParserFeatureProfile {
        pure_rust: false,
        tree_sitter_standard: true,
        tree_sitter_c2rust: true,
        glr: false,
    };

    // Then
    assert!(profile.has_tree_sitter());
}

// =============================================================================
// ParserFeatureProfile Backend Resolution Tests
// =============================================================================

#[test]
fn given_glr_enabled_when_resolving_backend_then_returns_glr() {
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
    assert_eq!(backend, ParserBackend::GLR);
}

#[test]
fn given_glr_enabled_with_conflicts_when_resolving_backend_then_returns_glr() {
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
    assert_eq!(backend, ParserBackend::GLR);
}

#[test]
fn given_pure_rust_only_without_conflicts_when_resolving_backend_then_returns_pure_rust() {
    // Given
    let profile = ParserFeatureProfile {
        pure_rust: true,
        tree_sitter_standard: false,
        tree_sitter_c2rust: false,
        glr: false,
    };

    // When
    let backend = profile.resolve_backend(false);

    // Then
    assert_eq!(backend, ParserBackend::PureRust);
}

#[test]
fn given_no_features_when_resolving_backend_then_returns_tree_sitter() {
    // Given
    let profile = ParserFeatureProfile {
        pure_rust: false,
        tree_sitter_standard: false,
        tree_sitter_c2rust: false,
        glr: false,
    };

    // When
    let backend = profile.resolve_backend(false);

    // Then
    assert_eq!(backend, ParserBackend::TreeSitter);
}

// =============================================================================
// ParserFeatureProfile Display Tests
// =============================================================================

#[test]
fn given_no_features_when_displaying_then_shows_none() {
    // Given
    let profile = ParserFeatureProfile {
        pure_rust: false,
        tree_sitter_standard: false,
        tree_sitter_c2rust: false,
        glr: false,
    };

    // When
    let display = format!("{}", profile);

    // Then
    assert_eq!(display, "none");
}

#[test]
fn given_pure_rust_only_when_displaying_then_shows_pure_rust() {
    // Given
    let profile = ParserFeatureProfile {
        pure_rust: true,
        tree_sitter_standard: false,
        tree_sitter_c2rust: false,
        glr: false,
    };

    // When
    let display = format!("{}", profile);

    // Then
    assert_eq!(display, "pure-rust");
}

#[test]
fn given_glr_only_when_displaying_then_shows_glr() {
    // Given
    let profile = ParserFeatureProfile {
        pure_rust: false,
        tree_sitter_standard: false,
        tree_sitter_c2rust: false,
        glr: true,
    };

    // When
    let display = format!("{}", profile);

    // Then
    assert_eq!(display, "glr");
}

#[test]
fn given_multiple_features_when_displaying_then_shows_comma_separated() {
    // Given
    let profile = ParserFeatureProfile {
        pure_rust: true,
        tree_sitter_standard: false,
        tree_sitter_c2rust: false,
        glr: true,
    };

    // When
    let display = format!("{}", profile);

    // Then
    assert!(display.contains("pure-rust"));
    assert!(display.contains("glr"));
    assert!(display.contains(","));
}

// =============================================================================
// ParserFeatureProfile Current Tests
// =============================================================================

#[test]
fn given_current_profile_when_accessing_then_matches_cfg_flags() {
    // Given / When
    let profile = ParserFeatureProfile::current();

    // Then
    assert_eq!(profile.pure_rust, cfg!(feature = "pure-rust"));
    assert_eq!(
        profile.tree_sitter_standard,
        cfg!(feature = "tree-sitter-standard")
    );
    assert_eq!(
        profile.tree_sitter_c2rust,
        cfg!(feature = "tree-sitter-c2rust")
    );
    assert_eq!(profile.glr, cfg!(feature = "glr"));
}

// =============================================================================
// ParserBackend Tests
// =============================================================================

#[test]
fn given_tree_sitter_backend_when_checking_predicates_then_correct_results() {
    // Given
    let backend = ParserBackend::TreeSitter;

    // When / Then
    assert!(!backend.is_glr());
    assert!(!backend.is_pure_rust());
}

#[test]
fn given_pure_rust_backend_when_checking_predicates_then_correct_results() {
    // Given
    let backend = ParserBackend::PureRust;

    // When / Then
    assert!(!backend.is_glr());
    assert!(backend.is_pure_rust());
}

#[test]
fn given_glr_backend_when_checking_predicates_then_correct_results() {
    // Given
    let backend = ParserBackend::GLR;

    // When / Then
    assert!(backend.is_glr());
    assert!(backend.is_pure_rust());
}

#[test]
fn given_any_backend_when_getting_name_then_returns_non_empty() {
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
