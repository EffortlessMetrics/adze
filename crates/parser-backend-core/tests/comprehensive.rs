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
fn select_returns_valid_backend() {
    let b1 = ParserBackend::select(false);
    let b2 = ParserBackend::select(true);
    let _ = b1.name();
    let _ = b2.name();
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
