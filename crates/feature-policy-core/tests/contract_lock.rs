//! Contract lock test - verifies that public API remains stable.

use adze_feature_policy_core::{ParserBackend, ParserFeatureProfile};

/// Verify all public types exist and have expected structure.
#[test]
fn test_contract_lock_types() {
    // Verify ParserFeatureProfile struct exists with expected fields
    let profile = ParserFeatureProfile {
        pure_rust: true,
        tree_sitter_standard: false,
        tree_sitter_c2rust: false,
        glr: false,
    };

    // Verify Debug trait is implemented
    let _debug = format!("{profile:?}");

    // Verify Clone trait is implemented
    let _cloned = profile;

    // Verify Copy trait is implemented
    let _copied: ParserFeatureProfile = profile;

    // Verify PartialEq trait is implemented
    assert_eq!(profile, profile);

    // Verify Eq trait is implemented
    let profile2 = ParserFeatureProfile {
        pure_rust: true,
        tree_sitter_standard: false,
        tree_sitter_c2rust: false,
        glr: false,
    };
    assert_eq!(profile, profile2);

    // Verify Hash trait is implemented (can use in HashSet)
    use std::collections::HashSet;
    let mut set = HashSet::new();
    set.insert(profile);

    // Verify Display trait is implemented
    let _display = format!("{profile}");
}

/// Verify all public methods exist with expected signatures.
#[test]
fn test_contract_lock_methods() {
    // Verify current method exists
    let _profile = ParserFeatureProfile::current();

    // Verify resolve_backend method exists
    let profile = ParserFeatureProfile {
        pure_rust: true,
        tree_sitter_standard: false,
        tree_sitter_c2rust: false,
        glr: false,
    };
    let _backend: ParserBackend = profile.resolve_backend(false);

    // Verify has_pure_rust method exists
    let has_pure: bool = profile.has_pure_rust();
    assert!(has_pure);

    // Verify has_glr method exists
    let has_glr: bool = profile.has_glr();
    assert!(!has_glr);

    // Verify has_tree_sitter method exists
    let has_ts: bool = profile.has_tree_sitter();
    assert!(!has_ts);
}

/// Verify re-exported ParserBackend type.
#[test]
fn test_contract_lock_reexports() {
    // Verify ParserBackend enum is re-exported with all variants
    let _tree_sitter = ParserBackend::TreeSitter;
    let _pure_rust = ParserBackend::PureRust;
    let _glr = ParserBackend::GLR;

    // Verify ParserBackend methods
    let name: &'static str = ParserBackend::TreeSitter.name();
    assert!(!name.is_empty());

    assert!(ParserBackend::GLR.is_glr());
    assert!(!ParserBackend::TreeSitter.is_glr());
    assert!(!ParserBackend::PureRust.is_glr());
}

/// Verify Display implementation for ParserFeatureProfile.
#[test]
fn test_contract_lock_display() {
    let profile = ParserFeatureProfile {
        pure_rust: true,
        tree_sitter_standard: false,
        tree_sitter_c2rust: false,
        glr: false,
    };
    let display = format!("{profile}");
    assert!(display.contains("pure-rust"));

    let empty_profile = ParserFeatureProfile {
        pure_rust: false,
        tree_sitter_standard: false,
        tree_sitter_c2rust: false,
        glr: false,
    };
    let empty_display = format!("{empty_profile}");
    assert_eq!(empty_display, "none");
}
