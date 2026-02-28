#![no_main]

use adze::glr_lexer::GLRLexer;
use adze_ir::{Grammar, SymbolId, Token, TokenPattern};
use libfuzzer_sys::fuzz_target;

fn create_test_grammar() -> Grammar {
    let mut grammar = Grammar::new("fuzz_lexer".to_string());

    grammar.tokens.insert(
        SymbolId(1),
        Token {
            name: "number".to_string(),
            pattern: TokenPattern::Regex(r"\d+".to_string()),
            fragile: false,
        },
    );
    grammar.tokens.insert(
        SymbolId(2),
        Token {
            name: "word".to_string(),
            pattern: TokenPattern::Regex(r"[a-zA-Z_]\w*".to_string()),
            fragile: false,
        },
    );
    grammar.tokens.insert(
        SymbolId(3),
        Token {
            name: "operator".to_string(),
            pattern: TokenPattern::Regex(r"[+\-*/=<>!&|^~%]+".to_string()),
            fragile: false,
        },
    );
    grammar.tokens.insert(
        SymbolId(4),
        Token {
            name: "space".to_string(),
            pattern: TokenPattern::Regex(r"\s+".to_string()),
            fragile: false,
        },
    );
    grammar.tokens.insert(
        SymbolId(5),
        Token {
            name: "punctuation".to_string(),
            pattern: TokenPattern::Regex(r"[(){}\[\];,.]".to_string()),
            fragile: false,
        },
    );

    grammar
}

static TEST_GRAMMAR: std::sync::LazyLock<Grammar> =
    std::sync::LazyLock::new(create_test_grammar);

fuzz_target!(|data: &[u8]| {
    let input = String::from_utf8_lossy(data);

    if input.len() > 10_000 {
        return;
    }

    // Fuzz the GLR lexer with arbitrary input - must never panic
    match GLRLexer::new(&TEST_GRAMMAR, input.to_string()) {
        Ok(mut lexer) => {
            let _tokens = lexer.tokenize_all();
        }
        Err(_) => {}
    }
});
