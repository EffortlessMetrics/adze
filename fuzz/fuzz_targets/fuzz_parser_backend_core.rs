#![no_main]

use adze_parser_backend_core::ParserBackend;
use libfuzzer_sys::fuzz_target;
use std::panic::catch_unwind;

fn expected(has_conflicts: bool) -> Option<ParserBackend> {
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

fuzz_target!(|data: &[u8]| {
    if data.is_empty() {
        return;
    }

    let has_conflicts = (data[0] & 1) == 0;
    let actual = catch_unwind(|| ParserBackend::select(has_conflicts));
    let expected = expected(has_conflicts);

    match (actual, expected) {
        (Ok(actual), Some(expected)) => {
            assert_eq!(actual, expected);
        }
        (Err(_), None) => {}
        (Ok(_), None) => {
            panic!("selection should panic for conflicting grammar when glr is unavailable");
        }
        (Err(_), Some(expected)) => {
            panic!(
                "selection should return {:?} for conflicts={}",
                expected, has_conflicts
            );
        }
    }
});
