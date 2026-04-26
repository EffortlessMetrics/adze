//! Lexer specialized for GLR parsing.
#![cfg_attr(feature = "strict_docs", allow(missing_docs))]

// Lexer integration for GLR parser
// This module provides tokenization for GLR parsing

use adze_ir::{Grammar, SymbolId, TokenPattern};
use regex::Regex;

/// Token with position information
#[derive(Debug, Clone)]
pub struct TokenWithPosition {
    pub symbol_id: SymbolId,
    pub text: String,
    pub byte_offset: usize,
    #[allow(dead_code)]
    pub byte_length: usize,
}

/// GLR-specific lexer that produces tokens for the GLR parser
pub struct GLRLexer {
    /// Compiled regex patterns for tokens
    token_patterns: Vec<(SymbolId, TokenMatcher)>,
    /// Input text
    input: String,
    /// Current position in input
    position: usize,
}

/// Token matching strategy
enum TokenMatcher {
    Literal(String),
    Regex(Regex),
}

impl TokenMatcher {
    fn matches_at(&self, input: &str, pos: usize) -> Option<usize> {
        // Safety: ensure we're at a valid UTF-8 boundary
        if !input.is_char_boundary(pos) {
            return None;
        }

        match self {
            TokenMatcher::Literal(s) => {
                if input[pos..].starts_with(s) {
                    Some(s.len())
                } else {
                    None
                }
            }
            TokenMatcher::Regex(re) => {
                // Ensure regex matches at start of string slice
                if let Some(m) = re.find(&input[pos..]) {
                    if m.start() == 0 { Some(m.len()) } else { None }
                } else {
                    None
                }
            }
        }
    }
}

impl GLRLexer {
    /// Create a new lexer from a grammar
    pub fn new(grammar: &Grammar, input: String) -> Result<Self, String> {
        let mut token_patterns = Vec::new();

        // Compile token patterns
        for (symbol_id, token) in &grammar.tokens {
            let matcher = match &token.pattern {
                TokenPattern::String(s) => TokenMatcher::Literal(s.clone()),
                TokenPattern::Regex(pattern) => {
                    // Add ^ anchor if not present to ensure matching at position
                    let anchored_pattern = if pattern.starts_with('^') {
                        pattern.clone()
                    } else {
                        format!("^{}", pattern)
                    };

                    match Regex::new(&anchored_pattern) {
                        Ok(re) => TokenMatcher::Regex(re),
                        Err(e) => {
                            let name = grammar
                                .rule_names
                                .get(symbol_id)
                                .map(|s| s.as_str())
                                .unwrap_or("unknown");
                            return Err(format!("Invalid regex for token {}: {}", name, e));
                        }
                    }
                }
            };
            token_patterns.push((*symbol_id, matcher));
        }

        // Sort by symbol ID for consistent matching order
        token_patterns.sort_by_key(|(id, _)| id.0);

        Ok(Self {
            token_patterns,
            input,
            position: 0,
        })
    }

    /// Get the next token from input
    pub fn next_token(&mut self) -> Option<TokenWithPosition> {
        // Skip whitespace
        self.skip_whitespace();

        if self.position >= self.input.len() {
            return None;
        }

        let start_pos = self.position;

        // Try each token pattern
        for (symbol_id, matcher) in &self.token_patterns {
            if let Some(len) = matcher.matches_at(&self.input, self.position) {
                if len == 0 {
                    // Prevent no-progress loops from zero-width token patterns.
                    continue;
                }
                // Ensure we're not splitting a UTF-8 sequence
                let end_pos = self.position + len;
                if !self.input.is_char_boundary(end_pos) {
                    continue;
                }

                let text = self.input[self.position..end_pos].to_string();
                self.position = end_pos;

                return Some(TokenWithPosition {
                    symbol_id: *symbol_id,
                    text,
                    byte_offset: start_pos,
                    byte_length: len,
                });
            }
        }

        // No token matched - skip one UTF-8 character and try again
        // Find the next character boundary
        let mut next_pos = self.position + 1;
        while next_pos < self.input.len() && !self.input.is_char_boundary(next_pos) {
            next_pos += 1;
        }
        self.position = next_pos;

        if self.position < self.input.len() {
            self.next_token()
        } else {
            None
        }
    }

    /// Skip whitespace characters
    fn skip_whitespace(&mut self) {
        let input_chars: Vec<char> = self.input.chars().collect();
        let mut char_pos = 0;
        let mut byte_pos = 0;

        // Find current position in characters
        for (i, ch) in self.input.chars().enumerate() {
            if byte_pos >= self.position {
                char_pos = i;
                break;
            }
            byte_pos += ch.len_utf8();
        }

        // Skip whitespace characters
        while char_pos < input_chars.len() {
            match input_chars[char_pos] {
                ' ' | '\t' | '\n' | '\r' => {
                    self.position += input_chars[char_pos].len_utf8();
                    char_pos += 1;
                }
                _ => break,
            }
        }
    }

    /// Reset lexer to beginning
    #[allow(dead_code)]
    pub fn reset(&mut self) {
        self.position = 0;
    }

    /// Get all tokens from input
    pub fn tokenize_all(&mut self) -> Vec<TokenWithPosition> {
        let mut tokens = Vec::new();
        while let Some(token) = self.next_token() {
            tokens.push(token);
        }
        tokens
    }
}

/// Helper to tokenize input and feed to GLR parser
#[allow(dead_code)]
pub fn tokenize_and_parse<F>(grammar: &Grammar, input: &str, mut parse_fn: F) -> Result<(), String>
where
    F: FnMut(SymbolId, &str, usize),
{
    let mut lexer = GLRLexer::new(grammar, input.to_string())?;

    while let Some(token) = lexer.next_token() {
        parse_fn(token.symbol_id, &token.text, token.byte_offset);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use adze_ir::{Grammar, Token, TokenPattern};

    #[test]
    fn test_literal_token_matching() {
        let mut grammar = Grammar::new("test".to_string());

        // Add literal tokens
        grammar.tokens.insert(
            SymbolId(1),
            Token {
                name: "plus".to_string(),
                pattern: TokenPattern::String("+".to_string()),
                fragile: false,
            },
        );

        grammar.tokens.insert(
            SymbolId(2),
            Token {
                name: "minus".to_string(),
                pattern: TokenPattern::String("-".to_string()),
                fragile: false,
            },
        );

        let mut lexer = GLRLexer::new(&grammar, "+ - +".to_string()).unwrap();

        let token1 = lexer.next_token().unwrap();
        assert_eq!(token1.symbol_id, SymbolId(1));
        assert_eq!(token1.text, "+");

        let token2 = lexer.next_token().unwrap();
        assert_eq!(token2.symbol_id, SymbolId(2));
        assert_eq!(token2.text, "-");

        let token3 = lexer.next_token().unwrap();
        assert_eq!(token3.symbol_id, SymbolId(1));
        assert_eq!(token3.text, "+");

        assert!(lexer.next_token().is_none());
    }

    #[test]
    fn test_regex_token_matching() {
        let mut grammar = Grammar::new("test".to_string());

        // Add regex tokens
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
                name: "identifier".to_string(),
                pattern: TokenPattern::Regex(r"[a-zA-Z]\w*".to_string()),
                fragile: false,
            },
        );

        let mut lexer = GLRLexer::new(&grammar, "123 hello 456 world".to_string()).unwrap();

        let tokens = lexer.tokenize_all();
        assert_eq!(tokens.len(), 4);

        assert_eq!(tokens[0].symbol_id, SymbolId(1));
        assert_eq!(tokens[0].text, "123");

        assert_eq!(tokens[1].symbol_id, SymbolId(2));
        assert_eq!(tokens[1].text, "hello");

        assert_eq!(tokens[2].symbol_id, SymbolId(1));
        assert_eq!(tokens[2].text, "456");

        assert_eq!(tokens[3].symbol_id, SymbolId(2));
        assert_eq!(tokens[3].text, "world");
    }

    #[test]
    fn test_mixed_tokens() {
        let mut grammar = Grammar::new("test".to_string());

        // Add mixed token types
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
                name: "plus".to_string(),
                pattern: TokenPattern::String("+".to_string()),
                fragile: false,
            },
        );

        let mut lexer = GLRLexer::new(&grammar, "1 + 2 + 3".to_string()).unwrap();
        let tokens = lexer.tokenize_all();

        assert_eq!(tokens.len(), 5);
        assert_eq!(tokens[0].text, "1");
        assert_eq!(tokens[1].text, "+");
        assert_eq!(tokens[2].text, "2");
        assert_eq!(tokens[3].text, "+");
        assert_eq!(tokens[4].text, "3");
    }

    #[test]
    fn test_utf8_boundary_safety() {
        let mut grammar = Grammar::new("test".to_string());
        grammar.tokens.insert(
            SymbolId(1),
            Token {
                name: "word".to_string(),
                pattern: TokenPattern::Regex(r"[a-zA-Z]+".to_string()),
                fragile: false,
            },
        );

        // Test input that caused fuzzer panic - byte 0xBE at invalid UTF-8 boundary
        let input = vec![190u8, 0, 0];
        let input_str = String::from_utf8_lossy(&input).to_string();

        // This should not panic
        let mut lexer = GLRLexer::new(&grammar, input_str).unwrap();
        let tokens = lexer.tokenize_all();

        // Should handle the invalid UTF-8 gracefully
        assert_eq!(tokens.len(), 0); // No valid tokens in malformed input
    }

    #[test]
    fn test_multibyte_character_handling() {
        let mut grammar = Grammar::new("test".to_string());

        // Add a pattern that matches individual letters
        grammar.tokens.insert(
            SymbolId(1),
            Token {
                name: "letter".to_string(),
                pattern: TokenPattern::Regex(r"[a-zA-Z]".to_string()),
                fragile: false,
            },
        );

        // Input with multi-byte UTF-8 emoji
        let input = "a🦀b".to_string();

        let mut lexer = GLRLexer::new(&grammar, input).unwrap();
        let tokens = lexer.tokenize_all();

        // Should tokenize only the ASCII letters, skipping the emoji
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].text, "a");
        assert_eq!(tokens[1].text, "b");
    }

    #[test]
    fn test_zero_length_regex_is_rejected_without_looping() {
        let mut grammar = Grammar::new("zero_regex".to_string());
        grammar.tokens.insert(
            SymbolId(1),
            Token {
                name: "empty".to_string(),
                pattern: TokenPattern::Regex(String::new()),
                fragile: false,
            },
        );

        let mut lexer = GLRLexer::new(&grammar, "abc".to_string()).unwrap();
        let tokens = lexer.tokenize_all();
        assert!(tokens.is_empty());
    }

    #[test]
    fn test_zero_length_literal_is_rejected_without_looping() {
        let mut grammar = Grammar::new("zero_literal".to_string());
        grammar.tokens.insert(
            SymbolId(1),
            Token {
                name: "empty".to_string(),
                pattern: TokenPattern::String(String::new()),
                fragile: false,
            },
        );
        grammar.tokens.insert(
            SymbolId(2),
            Token {
                name: "alpha".to_string(),
                pattern: TokenPattern::Regex(r"[a-z]+".to_string()),
                fragile: false,
            },
        );

        let mut lexer = GLRLexer::new(&grammar, "abc".to_string()).unwrap();
        let tokens = lexer.tokenize_all();
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].symbol_id, SymbolId(2));
        assert_eq!(tokens[0].text, "abc");
    }
}
