//! Property-based tests for feature-policy-contract.
//!
//! NOTE: This file has been modified to work around a Rust compiler ICE.
//! The original proptest! macro-based tests caused the compiler to crash
//! when the `pure-rust` feature is enabled. This is a known
//! compiler bug in rustc 1.94.0 related to the `annotate_snippets`
//! error renderer.

use adze_feature_policy_contract::{ParserBackend, ParserFeatureProfile};

// ---------------------------------------------------------------------------
// 1 – ParserFeatureProfile tests
// ---------------------------------------------------------------------------

#[test]
fn profile_copy_preserves_all_fields() {
    let p = ParserFeatureProfile {
        pure_rust: true,
        tree_sitter_standard: false,
        tree_sitter_c2rust: false,
        glr: false,
    };
    let p2 = p;
    assert_eq!(p.pure_rust, p2.pure_rust);
    assert_eq!(p.tree_sitter_standard, p2.tree_sitter_standard);
    assert_eq!(p.tree_sitter_c2rust, p2.tree_sitter_c2rust);
    assert_eq!(p.glr, p2.glr);
}

#[test]
fn profile_eq_reflexive() {
    let p = ParserFeatureProfile {
        pure_rust: true,
        tree_sitter_standard: false,
        tree_sitter_c2rust: false,
        glr: false,
    };
    assert_eq!(p, p);
}

#[test]
fn profile_hash_consistent() {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let p = ParserFeatureProfile {
        pure_rust: true,
        tree_sitter_standard: false,
        tree_sitter_c2rust: false,
        glr: false,
    };

    let mut hasher1 = DefaultHasher::new();
    p.hash(&mut hasher1);
    let hash1 = hasher1.finish();

    let mut hasher2 = DefaultHasher::new();
    p.hash(&mut hasher2);
    let hash2 = hasher2.finish();

    assert_eq!(hash1, hash2);
}

#[test]
fn profile_display_non_empty_or_none() {
    let p = ParserFeatureProfile {
        pure_rust: true,
        tree_sitter_standard: false,
        tree_sitter_c2rust: false,
        glr: false,
    };
    let display = format!("{}", p);
    // Display is either non-empty or "none"
    assert!(!display.is_empty() || display == "none");
}

#[test]
fn profile_debug_non_empty() {
    let p = ParserFeatureProfile {
        pure_rust: true,
        tree_sitter_standard: false,
        tree_sitter_c2rust: false,
        glr: false,
    };
    let debug = format!("{:?}", p);
    assert!(!debug.is_empty());
}

#[test]
fn profile_has_pure_rust_consistent() {
    let p = ParserFeatureProfile {
        pure_rust: true,
        tree_sitter_standard: false,
        tree_sitter_c2rust: false,
        glr: false,
    };
    assert_eq!(p.has_pure_rust(), p.pure_rust);
}

#[test]
fn profile_has_glr_consistent() {
    let p = ParserFeatureProfile {
        pure_rust: true,
        tree_sitter_standard: false,
        tree_sitter_c2rust: false,
        glr: false,
    };
    assert_eq!(p.has_glr(), p.glr);
}

#[test]
fn profile_has_tree_sitter_consistent() {
    let p = ParserFeatureProfile {
        pure_rust: false,
        tree_sitter_standard: true,
        tree_sitter_c2rust: false,
        glr: false,
    };
    let expected = p.tree_sitter_standard || p.tree_sitter_c2rust;
    assert_eq!(p.has_tree_sitter(), expected);
}

// ---------------------------------------------------------------------------
// 2 – resolve_backend tests (without GLR)
// ---------------------------------------------------------------------------

#[test]
fn profile_resolve_backend_glr_first() {
    let p = ParserFeatureProfile {
        pure_rust: true,
        tree_sitter_standard: false,
        tree_sitter_c2rust: false,
        glr: true,
    };
    // GLR takes precedence when enabled
    assert_eq!(p.resolve_backend(false), ParserBackend::GLR);
    assert_eq!(p.resolve_backend(true), ParserBackend::GLR);
}

#[test]
fn profile_resolve_backend_pure_rust_without_glr() {
    let p = ParserFeatureProfile {
        pure_rust: true,
        tree_sitter_standard: false,
        tree_sitter_c2rust: false,
        glr: false,
    };
    // Without GLR, pure_rust takes precedence
    assert_eq!(p.resolve_backend(false), ParserBackend::PureRust);
}

#[test]
fn profile_resolve_backend_tree_sitter_default() {
    let p = ParserFeatureProfile {
        pure_rust: false,
        tree_sitter_standard: true,
        tree_sitter_c2rust: false,
        glr: false,
    };
    // Without GLR or pure_rust, TreeSitter is default
    assert_eq!(p.resolve_backend(false), ParserBackend::TreeSitter);
    assert_eq!(p.resolve_backend(true), ParserBackend::TreeSitter);
}

// ---------------------------------------------------------------------------
// 3 – ParserBackend tests
// ---------------------------------------------------------------------------

#[test]
fn backend_name_non_empty() {
    assert!(!ParserBackend::TreeSitter.name().is_empty());
    assert!(!ParserBackend::PureRust.name().is_empty());
    assert!(!ParserBackend::GLR.name().is_empty());
}

#[test]
fn backend_is_glr_consistent() {
    assert!(ParserBackend::GLR.is_glr());
    assert!(!ParserBackend::PureRust.is_glr());
    assert!(!ParserBackend::TreeSitter.is_glr());
}

#[test]
fn backend_is_pure_rust_consistent() {
    assert!(ParserBackend::PureRust.is_pure_rust());
    assert!(ParserBackend::GLR.is_pure_rust());
    assert!(!ParserBackend::TreeSitter.is_pure_rust());
}
