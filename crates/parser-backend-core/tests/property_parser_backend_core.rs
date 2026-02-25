use adze_parser_backend_core::ParserBackend;
use proptest::prelude::*;
use std::panic::catch_unwind;

fn expected_backend(has_conflicts: bool) -> Option<ParserBackend> {
    if cfg!(feature = "glr") {
        Some(ParserBackend::GLR)
    } else if cfg!(feature = "pure-rust") {
        if has_conflicts {
            None
        } else {
            Some(ParserBackend::PureRust)
        }
    } else {
        Some(ParserBackend::TreeSitter)
    }
}

proptest! {
    #[test]
    fn select_matches_model(has_conflicts in any::<bool>()) {
        let expected = expected_backend(has_conflicts);
        let actual = catch_unwind(|| ParserBackend::select(has_conflicts));

        match (actual, expected) {
            (Ok(actual), Some(expected)) => {
                assert_eq!(actual, expected);
            }
            (Err(_), None) => {}
            (Ok(_), None) => {
                panic!("selection unexpectedly succeeded when conflicts should panic");
            }
            (Err(_), Some(expected)) => {
                panic!("selection should return {:?} for conflicts={}", expected, has_conflicts);
            }
        }
    }
}
