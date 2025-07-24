// Lexer integration for GLR parser
// This module provides tokenization for GLR parsing

use rust_sitter_ir::{Grammar, Token, TokenPattern, SymbolId};
use regex::Regex;

/// Token with position information
#[derive(Debug, Clone)]
pub struct TokenWithPosition {
    pub symbol_id: SymbolId,
    pub text: String,
    pub byte_offset: usize,
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
                    if m.start() == 0 {
                        Some(m.len())
                    } else {
                        None
                    }
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
                        Err(e) => return Err(format!("Invalid regex for token {}: {}", token.name, e)),
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
                let text = self.input[self.position..self.position + len].to_string();
                self.position += len;
                
                return Some(TokenWithPosition {
                    symbol_id: *symbol_id,
                    text,
                    byte_offset: start_pos,
                    byte_length: len,
                });
            }
        }
        
        // No token matched - return error token or skip character
        // For now, skip one character and try again
        self.position += 1;
        self.next_token()
    }
    
    /// Skip whitespace characters
    fn skip_whitespace(&mut self) {
        while self.position < self.input.len() {
            let ch = self.input.chars().nth(self.position);
            match ch {
                Some(' ') | Some('\t') | Some('\n') | Some('\r') => {
                    self.position += 1;
                }
                _ => break,
            }
        }
    }
    
    /// Reset lexer to beginning
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
pub fn tokenize_and_parse<F>(
    grammar: &Grammar,
    input: &str,
    mut parse_fn: F,
) -> Result<(), String>
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
    use rust_sitter_ir::{Grammar, Token, TokenPattern};
    
    #[test]
    fn test_literal_token_matching() {
        let mut grammar = Grammar::new("test".to_string());
        
        // Add literal tokens
        grammar.tokens.insert(SymbolId(1), Token {
            name: "plus".to_string(),
            pattern: TokenPattern::String("+".to_string()),
            fragile: false,
        });
        
        grammar.tokens.insert(SymbolId(2), Token {
            name: "minus".to_string(),
            pattern: TokenPattern::String("-".to_string()),
            fragile: false,
        });
        
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
        grammar.tokens.insert(SymbolId(1), Token {
            name: "number".to_string(),
            pattern: TokenPattern::Regex(r"\d+".to_string()),
            fragile: false,
        });
        
        grammar.tokens.insert(SymbolId(2), Token {
            name: "identifier".to_string(),
            pattern: TokenPattern::Regex(r"[a-zA-Z]\w*".to_string()),
            fragile: false,
        });
        
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
        grammar.tokens.insert(SymbolId(1), Token {
            name: "number".to_string(),
            pattern: TokenPattern::Regex(r"\d+".to_string()),
            fragile: false,
        });
        
        grammar.tokens.insert(SymbolId(2), Token {
            name: "plus".to_string(),
            pattern: TokenPattern::String("+".to_string()),
            fragile: false,
        });
        
        let mut lexer = GLRLexer::new(&grammar, "1 + 2 + 3".to_string()).unwrap();
        let tokens = lexer.tokenize_all();
        
        assert_eq!(tokens.len(), 5);
        assert_eq!(tokens[0].text, "1");
        assert_eq!(tokens[1].text, "+");
        assert_eq!(tokens[2].text, "2");
        assert_eq!(tokens[3].text, "+");
        assert_eq!(tokens[4].text, "3");
    }
}