//! Property-based tests for ParserFeatureProfile and ParserBackend.

use proptest::prelude::*;

use adze_feature_policy_core::{ParserBackend, ParserFeatureProfile};

// ---------------------------------------------------------------------------
// Strategies
// ---------------------------------------------------------------------------

/// Generate arbitrary ParserFeatureProfile values.
fn arb_profile() -> impl Strategy<Value = ParserFeatureProfile> {
    (any::<bool>(), any::<bool>(), any::<bool>(), any::<bool>()).prop_map(
        |(pure_rust, tree_sitter_standard, tree_sitter_c2rust, glr)| ParserFeatureProfile {
            pure_rust,
            tree_sitter_standard,
            tree_sitter_c2rust,
            glr,
        },
    )
}

/// Generate profiles with GLR disabled (for testing non-GLR backend resolution).
fn arb_profile_without_glr() -> impl Strategy<Value = ParserFeatureProfile> {
    (any::<bool>(), any::<bool>(), any::<bool>()).prop_map(
        |(pure_rust, tree_sitter_standard, tree_sitter_c2rust)| ParserFeatureProfile {
            pure_rust,
            tree_sitter_standard,
            tree_sitter_c2rust,
            glr: false,
        },
    )
}

// ---------------------------------------------------------------------------
// 1 – Copy and Clone semantics
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn profile_copy_preserves_all_fields(p in arb_profile()) {
        let p2 = p;
        prop_assert_eq!(p.pure_rust, p2.pure_rust);
        prop_assert_eq!(p.tree_sitter_standard, p2.tree_sitter_standard);
        prop_assert_eq!(p.tree_sitter_c2rust, p2.tree_sitter_c2rust);
        prop_assert_eq!(p.glr, p2.glr);
    }

    #[test]
    fn profile_clone_equals_original(p in arb_profile()) {
        let cloned = p;
        prop_assert_eq!(p, cloned);
    }
}

// ---------------------------------------------------------------------------
// 2 – PartialEq / Eq
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn profile_eq_reflexive(p in arb_profile()) {
        prop_assert_eq!(p, p);
    }

    #[test]
    fn profile_eq_symmetric(a in arb_profile(), b in arb_profile()) {
        prop_assert_eq!(a == b, b == a);
    }

    #[test]
    fn profile_eq_transitive(a in arb_profile(), b in arb_profile(), c in arb_profile()) {
        if a == b && b == c {
            prop_assert_eq!(a, c);
        }
    }
}

// ---------------------------------------------------------------------------
// 3 – Hash consistency
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn profile_hash_consistent(p in arb_profile()) {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher1 = DefaultHasher::new();
        p.hash(&mut hasher1);
        let hash1 = hasher1.finish();

        let mut hasher2 = DefaultHasher::new();
        p.hash(&mut hasher2);
        let hash2 = hasher2.finish();

        prop_assert_eq!(hash1, hash2);
    }

    #[test]
    fn profile_equal_implies_equal_hash(a in arb_profile(), b in arb_profile()) {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        if a == b {
            let mut hasher_a = DefaultHasher::new();
            a.hash(&mut hasher_a);
            let hash_a = hasher_a.finish();

            let mut hasher_b = DefaultHasher::new();
            b.hash(&mut hasher_b);
            let hash_b = hasher_b.finish();

            prop_assert_eq!(hash_a, hash_b);
        }
    }
}

// ---------------------------------------------------------------------------
// 4 – Accessor methods
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn has_pure_rust_matches_field(p in arb_profile()) {
        prop_assert_eq!(p.has_pure_rust(), p.pure_rust);
    }

    #[test]
    fn has_glr_matches_field(p in arb_profile()) {
        prop_assert_eq!(p.has_glr(), p.glr);
    }

    #[test]
    fn has_tree_sitter_is_or_of_backends(p in arb_profile()) {
        prop_assert_eq!(
            p.has_tree_sitter(),
            p.tree_sitter_standard || p.tree_sitter_c2rust
        );
    }
}

// ---------------------------------------------------------------------------
// 5 – Backend resolution (without GLR)
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn resolve_backend_glr_takes_priority(p in arb_profile()) {
        if p.glr {
            prop_assert_eq!(p.resolve_backend(false), ParserBackend::GLR);
            prop_assert_eq!(p.resolve_backend(true), ParserBackend::GLR);
        }
    }

    #[test]
    fn resolve_backend_pure_rust_without_conflicts(p in arb_profile_without_glr()) {
        if p.pure_rust {
            prop_assert_eq!(p.resolve_backend(false), ParserBackend::PureRust);
        }
    }

    #[test]
    fn resolve_backend_tree_sitter_fallback(p in arb_profile_without_glr()) {
        if !p.pure_rust {
            prop_assert_eq!(p.resolve_backend(false), ParserBackend::TreeSitter);
            prop_assert_eq!(p.resolve_backend(true), ParserBackend::TreeSitter);
        }
    }
}

// ---------------------------------------------------------------------------
// 6 – Display implementation
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn display_never_panics(p in arb_profile()) {
        let _s = format!("{p}");
        // Just checking that Display doesn't panic
    }

    #[test]
    fn display_is_deterministic(p in arb_profile()) {
        let s1 = format!("{p}");
        let s2 = format!("{p}");
        prop_assert_eq!(s1, s2);
    }
}

// ---------------------------------------------------------------------------
// 7 – ParserBackend tests
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn backend_display_never_panics(backend in prop_oneof![
        Just(ParserBackend::TreeSitter),
        Just(ParserBackend::PureRust),
        Just(ParserBackend::GLR),
    ]) {
        let _s = format!("{backend}");
    }

    #[test]
    fn backend_eq_reflexive(backend in prop_oneof![
        Just(ParserBackend::TreeSitter),
        Just(ParserBackend::PureRust),
        Just(ParserBackend::GLR),
    ]) {
        prop_assert_eq!(backend, backend);
    }
}

// ---------------------------------------------------------------------------
// 8 – All 16 profile combinations
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn all_profile_combinations_valid(bits in 0u8..16u8) {
        let p = ParserFeatureProfile {
            pure_rust: bits & 0b1000 != 0,
            tree_sitter_standard: bits & 0b0100 != 0,
            tree_sitter_c2rust: bits & 0b0010 != 0,
            glr: bits & 0b0001 != 0,
        };

        // Verify accessor consistency
        prop_assert_eq!(p.has_pure_rust(), p.pure_rust);
        prop_assert_eq!(p.has_glr(), p.glr);
        prop_assert_eq!(p.has_tree_sitter(), p.tree_sitter_standard || p.tree_sitter_c2rust);

        // Verify backend resolution priority
        if p.glr {
            prop_assert_eq!(p.resolve_backend(false), ParserBackend::GLR);
            prop_assert_eq!(p.resolve_backend(true), ParserBackend::GLR);
        } else if p.pure_rust {
            // Without conflicts, pure-rust is used
            prop_assert_eq!(p.resolve_backend(false), ParserBackend::PureRust);
        } else {
            prop_assert_eq!(p.resolve_backend(false), ParserBackend::TreeSitter);
            prop_assert_eq!(p.resolve_backend(true), ParserBackend::TreeSitter);
        }
    }
}
