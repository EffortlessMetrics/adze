use adze_parser_backend_core::ParserBackend;

#[cfg(feature = "glr")]
#[test]
fn selecting_with_glr_feature_uses_glr_backend() {
    assert_eq!(ParserBackend::select(false), ParserBackend::GLR);
}

#[cfg(not(feature = "glr"))]
#[cfg(feature = "pure-rust")]
#[test]
fn selecting_with_pure_rust_without_conflicts_uses_pure_rust_backend() {
    assert_eq!(ParserBackend::select(false), ParserBackend::PureRust);
}

#[cfg(all(feature = "pure-rust", not(feature = "glr")))]
#[test]
#[should_panic(expected = "Grammar has conflicts but GLR feature is not enabled.")]
fn selecting_with_conflicts_without_glr_panics() {
    let _ = ParserBackend::select(true);
}

#[cfg(not(any(feature = "pure-rust", feature = "glr")))]
#[test]
fn selecting_without_pure_rust_defaults_to_treesitter() {
    assert_eq!(ParserBackend::select(false), ParserBackend::TreeSitter);
}
