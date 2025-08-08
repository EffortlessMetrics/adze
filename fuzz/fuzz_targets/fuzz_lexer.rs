#![no_main]

use libfuzzer_sys::fuzz_target;
use rust_sitter::*;

fuzz_target!(|data: &[u8]| {
    // Convert random bytes to UTF-8 string (lossy is fine for fuzzing)
    let input = String::from_utf8_lossy(data);
    
    // Try to tokenize the input
    // This should never panic, only return errors
    let _ = tokenize(&input);
    
    // Additional invariants to check:
    // - Tokenizer should handle all valid UTF-8
    // - Tokenizer should not allocate unbounded memory
    // - Tokenizer should complete in reasonable time
});

/// Helper function to tokenize input (placeholder - adapt to your lexer API)
fn tokenize(input: &str) -> Result<Vec<Token>, LexError> {
    // Your actual tokenization logic here
    Ok(vec![])
}

#[derive(Debug)]
struct Token {
    kind: TokenKind,
    text: String,
    span: (usize, usize),
}

#[derive(Debug)]
enum TokenKind {
    Number,
    Operator,
    Identifier,
    // ... other token types
}

#[derive(Debug)]
struct LexError;