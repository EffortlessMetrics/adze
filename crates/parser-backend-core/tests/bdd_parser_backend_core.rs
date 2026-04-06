use adze_parser_backend_core::ParserBackend;
#[cfg(feature = "pure-rust")]
use adze_parser_backend_core::ParserBackendSelection;

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
fn selecting_with_conflicts_without_glr_panics() {
    match ParserBackend::select_contract(true) {
        ParserBackendSelection::Backend(_) => {
            panic!("unexpected backend result from select_contract for conflicting grammar")
        }
        ParserBackendSelection::ConflictsRequireGlr => {
            assert!(std::panic::catch_unwind(|| ParserBackend::select(true)).is_err());
        }
    }
}

#[cfg(not(any(feature = "pure-rust", feature = "glr")))]
#[test]
fn selecting_without_pure_rust_defaults_to_treesitter() {
    assert_eq!(ParserBackend::select(false), ParserBackend::TreeSitter);
}
