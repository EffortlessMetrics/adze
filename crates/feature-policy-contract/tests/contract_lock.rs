//! Contract lock tests - verify API stability
//! These tests ensure the public API remains stable.

#[cfg(test)]
mod contract_lock {
    use adze_feature_policy_contract::*;

    #[test]
    fn contract_lock_types() {
        // Verify types exist and are accessible
        let _backend = ParserBackend::TreeSitter;
        let _backend = ParserBackend::PureRust;
        let _backend = ParserBackend::GLR;

        let _profile = ParserFeatureProfile::current();
    }

    #[test]
    fn contract_lock_functions() {
        // Verify type methods exist and are callable
        let profile = ParserFeatureProfile::current();

        // ParserFeatureProfile methods
        let _has_pure_rust = profile.has_pure_rust();
        let _has_glr = profile.has_glr();
        let _has_tree_sitter = profile.has_tree_sitter();

        // ParserBackend methods
        let backend = ParserBackend::GLR;
        let _name = backend.name();
        let _is_glr = backend.is_glr();
        let _is_pure_rust = backend.is_pure_rust();

        // Resolve backend (without conflicts to avoid potential panic)
        let _resolved = profile.resolve_backend(false);
    }

    #[test]
    fn contract_lock_traits() {
        // Verify traits are implemented
        let backend = ParserBackend::GLR;

        // Debug trait
        let _debug = format!("{:?}", backend);

        // Display trait
        let _display = format!("{}", backend);

        // Clone trait
        let _cloned = backend;

        // PartialEq trait
        assert_eq!(backend, ParserBackend::GLR);

        let profile = ParserFeatureProfile::current();
        let _debug = format!("{:?}", profile);
        let _display = format!("{}", profile);
    }
}
