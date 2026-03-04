// Comprehensive tests for feature-policy-core
use adze_feature_policy_core::{ParserBackend, ParserFeatureProfile};

// ---------------------------------------------------------------------------
// ParserFeatureProfile construction
// ---------------------------------------------------------------------------

#[test]
fn profile_current() {
    let p = ParserFeatureProfile::current();
    // Should reflect current build features
    let _ = format!("{:?}", p);
}

#[test]
fn profile_fields_accessible() {
    let p = ParserFeatureProfile {
        pure_rust: true,
        tree_sitter_standard: false,
        tree_sitter_c2rust: true,
        glr: false,
    };
    assert!(p.pure_rust);
    assert!(!p.tree_sitter_standard);
    assert!(p.tree_sitter_c2rust);
    assert!(!p.glr);
}

// ---------------------------------------------------------------------------
// has_* predicates
// ---------------------------------------------------------------------------

#[test]
fn has_pure_rust() {
    let p = ParserFeatureProfile {
        pure_rust: true,
        tree_sitter_standard: false,
        tree_sitter_c2rust: false,
        glr: false,
    };
    assert!(p.has_pure_rust());
}

#[test]
fn has_glr() {
    let p = ParserFeatureProfile {
        pure_rust: false,
        tree_sitter_standard: false,
        tree_sitter_c2rust: false,
        glr: true,
    };
    assert!(p.has_glr());
}

#[test]
fn has_tree_sitter() {
    let p = ParserFeatureProfile {
        pure_rust: false,
        tree_sitter_standard: true,
        tree_sitter_c2rust: false,
        glr: false,
    };
    assert!(p.has_tree_sitter());
    let p2 = ParserFeatureProfile {
        pure_rust: false,
        tree_sitter_standard: false,
        tree_sitter_c2rust: true,
        glr: false,
    };
    assert!(p2.has_tree_sitter());
}

#[test]
fn has_no_tree_sitter() {
    let p = ParserFeatureProfile {
        pure_rust: true,
        tree_sitter_standard: false,
        tree_sitter_c2rust: false,
        glr: false,
    };
    assert!(!p.has_tree_sitter());
}

// ---------------------------------------------------------------------------
// resolve_backend
// ---------------------------------------------------------------------------

#[test]
fn resolve_backend_glr_with_conflicts() {
    let p = ParserFeatureProfile {
        pure_rust: false,
        tree_sitter_standard: false,
        tree_sitter_c2rust: false,
        glr: true,
    };
    assert_eq!(p.resolve_backend(true), ParserBackend::GLR);
}

#[test]
fn resolve_backend_pure_rust() {
    let p = ParserFeatureProfile {
        pure_rust: true,
        tree_sitter_standard: false,
        tree_sitter_c2rust: false,
        glr: false,
    };
    assert_eq!(p.resolve_backend(false), ParserBackend::PureRust);
}

#[test]
fn resolve_backend_tree_sitter() {
    let p = ParserFeatureProfile {
        pure_rust: false,
        tree_sitter_standard: true,
        tree_sitter_c2rust: false,
        glr: false,
    };
    assert_eq!(p.resolve_backend(false), ParserBackend::TreeSitter);
}

// ---------------------------------------------------------------------------
// ParserBackend
// ---------------------------------------------------------------------------

#[test]
fn backend_glr() {
    let b = ParserBackend::GLR;
    let _ = format!("{:?}", b);
}

#[test]
fn backend_pure_rust() {
    let b = ParserBackend::PureRust;
    let _ = format!("{:?}", b);
}

#[test]
fn backend_tree_sitter() {
    let b = ParserBackend::TreeSitter;
    let _ = format!("{:?}", b);
}

#[test]
fn backend_eq() {
    assert_eq!(ParserBackend::GLR, ParserBackend::GLR);
    assert_eq!(ParserBackend::PureRust, ParserBackend::PureRust);
    assert_eq!(ParserBackend::TreeSitter, ParserBackend::TreeSitter);
}

#[test]
fn backend_ne() {
    assert_ne!(ParserBackend::GLR, ParserBackend::PureRust);
    assert_ne!(ParserBackend::GLR, ParserBackend::TreeSitter);
    assert_ne!(ParserBackend::PureRust, ParserBackend::TreeSitter);
}

// ---------------------------------------------------------------------------
// Profile debug/clone
// ---------------------------------------------------------------------------

#[test]
fn profile_debug() {
    let p = ParserFeatureProfile::current();
    let d = format!("{:?}", p);
    assert!(d.contains("ParserFeatureProfile"));
}

#[test]
fn profile_clone() {
    let p = ParserFeatureProfile {
        pure_rust: true,
        tree_sitter_standard: false,
        tree_sitter_c2rust: true,
        glr: false,
    };
    let p2 = p;
    assert_eq!(p.pure_rust, p2.pure_rust);
    assert_eq!(p.glr, p2.glr);
}

// ---------------------------------------------------------------------------
// All combinations
// ---------------------------------------------------------------------------

#[test]
fn all_false_profile() {
    let p = ParserFeatureProfile {
        pure_rust: false,
        tree_sitter_standard: false,
        tree_sitter_c2rust: false,
        glr: false,
    };
    assert!(!p.has_pure_rust());
    assert!(!p.has_glr());
    assert!(!p.has_tree_sitter());
}

#[test]
fn all_true_profile() {
    let p = ParserFeatureProfile {
        pure_rust: true,
        tree_sitter_standard: true,
        tree_sitter_c2rust: true,
        glr: true,
    };
    assert!(p.has_pure_rust());
    assert!(p.has_glr());
    assert!(p.has_tree_sitter());
}
