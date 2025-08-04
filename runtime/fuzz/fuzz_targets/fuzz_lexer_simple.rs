#![no_main]

use libfuzzer_sys::fuzz_target;
use rust_sitter::glr_lexer::GLRLexer;
use rust_sitter_ir::{Grammar, Token, TokenPattern, SymbolId};

// Create a simple test grammar
fn create_test_grammar() -> Grammar {
    let mut grammar = Grammar::new("fuzz_test".to_string());
    
    // Add some tokens that could match fuzzer input
    grammar.tokens.insert(SymbolId(1), Token {
        name: "number".to_string(),
        pattern: TokenPattern::Regex(r"\d+".to_string()),
        fragile: false,
    });
    
    grammar.tokens.insert(SymbolId(2), Token {
        name: "word".to_string(),
        pattern: TokenPattern::Regex(r"[a-zA-Z]+".to_string()),
        fragile: false,
    });
    
    grammar.tokens.insert(SymbolId(3), Token {
        name: "plus".to_string(),
        pattern: TokenPattern::String("+".to_string()),
        fragile: false,
    });
    
    grammar.tokens.insert(SymbolId(4), Token {
        name: "space".to_string(),
        pattern: TokenPattern::Regex(r"\s+".to_string()),
        fragile: false,
    });
    
    grammar
}

lazy_static::lazy_static! {
    static ref TEST_GRAMMAR: Grammar = create_test_grammar();
}

fuzz_target!(|data: &[u8]| {
    // Convert fuzzer input to string (ignore invalid UTF-8)
    let input = String::from_utf8_lossy(data);
    
    // Skip overly large inputs to avoid memory issues
    if input.len() > 10_000 {
        return;
    }
    
    // Try to create lexer - this should not panic
    match GLRLexer::new(&TEST_GRAMMAR, input.to_string()) {
        Ok(mut lexer) => {
            // Try to tokenize - this should not panic
            let _tokens = lexer.tokenize_all();
        }
        Err(_) => {
            // Lexer creation error is fine
        }
    }
});