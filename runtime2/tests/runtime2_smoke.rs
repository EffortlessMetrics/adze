//! Bounded smoke tests for the `runtime2/` crate surface.
//!
//! These tests intentionally verify only minimal guarantees that are currently
//! supported (builder + parser wiring), without asserting full parser maturity.

use adze_runtime::{Parser, Token, language::SymbolMetadata};

fn leak_parse_table() -> &'static adze_glr_core::ParseTable {
    Box::leak(Box::new(adze_glr_core::ParseTable::default()))
}

#[test]
fn smoke_language_builder_constructs_and_parser_accepts_language() {
    let language = adze_runtime::Language::builder()
        .parse_table(leak_parse_table())
        .symbol_names(vec!["eof".to_string(), "expr".to_string()])
        .symbol_metadata(vec![
            SymbolMetadata {
                is_terminal: true,
                is_visible: false,
                is_supertype: false,
            },
            SymbolMetadata {
                is_terminal: false,
                is_visible: true,
                is_supertype: false,
            },
        ])
        .build()
        .expect("language builder should construct minimal runtime2 language");

    let language = language.with_static_tokens(vec![Token {
        kind: 0,
        start: 0,
        end: 0,
    }]);

    assert_eq!(language.symbol_name(1), Some("expr"));
    assert_eq!(language.symbol_for_name("expr", true), Some(1));

    let mut parser = Parser::new();
    parser
        .set_language(language)
        .expect("runtime2 parser should accept minimally constructed language");
    assert!(parser.language().is_some());
}
