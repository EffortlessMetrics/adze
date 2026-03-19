// Comprehensive tests for parser-backend-core
use adze_parser_backend_core::ParserBackend;

#[test]
fn all_variants_distinct() {
    let variants = [
        ParserBackend::TreeSitter,
        ParserBackend::PureRust,
        ParserBackend::GLR,
    ];
    for i in 0..variants.len() {
        for j in (i + 1)..variants.len() {
            assert_ne!(variants[i], variants[j]);
        }
    }
}

#[test]
fn name_non_empty() {
    for backend in [
        ParserBackend::TreeSitter,
        ParserBackend::PureRust,
        ParserBackend::GLR,
    ] {
        assert!(!backend.name().is_empty());
    }
}

#[test]
fn display_matches_name() {
    for backend in [
        ParserBackend::TreeSitter,
        ParserBackend::PureRust,
        ParserBackend::GLR,
    ] {
        assert_eq!(format!("{}", backend), backend.name());
    }
}

#[test]
fn select_matches_feature_contract() {
    #[cfg(feature = "glr")]
    {
        assert_eq!(ParserBackend::select(false), ParserBackend::GLR);
        assert_eq!(ParserBackend::select(true), ParserBackend::GLR);
    }

    #[cfg(all(feature = "pure-rust", not(feature = "glr")))]
    {
        assert_eq!(ParserBackend::select(false), ParserBackend::PureRust);
        let result = std::panic::catch_unwind(|| ParserBackend::select(true));
        assert!(
            result.is_err(),
            "pure-rust without glr must reject conflicting grammars"
        );
    }

    #[cfg(not(any(feature = "pure-rust", feature = "glr")))]
    {
        assert_eq!(ParserBackend::select(false), ParserBackend::TreeSitter);
        assert_eq!(ParserBackend::select(true), ParserBackend::TreeSitter);
    }
}

#[test]
fn glr_is_pure_rust() {
    assert!(ParserBackend::GLR.is_pure_rust());
    assert!(ParserBackend::GLR.is_glr());
}

#[test]
fn tree_sitter_not_pure_rust() {
    assert!(!ParserBackend::TreeSitter.is_pure_rust());
    assert!(!ParserBackend::TreeSitter.is_glr());
}

#[test]
fn pure_rust_not_glr() {
    assert!(ParserBackend::PureRust.is_pure_rust());
    assert!(!ParserBackend::PureRust.is_glr());
}

#[test]
fn clone_independence() {
    let a = ParserBackend::GLR;
    let b = a;
    assert_eq!(a, b);
}

#[test]
fn debug_all_variants() {
    for backend in [
        ParserBackend::TreeSitter,
        ParserBackend::PureRust,
        ParserBackend::GLR,
    ] {
        let d = format!("{:?}", backend);
        assert!(!d.is_empty());
    }
}
