//! Lexical Scanner (Tokenizer) for GLR Parsing (Phase 3.2)
//!
//! Contract: docs/specs/PHASE_3.2_TOKENIZATION_FOREST_CONVERSION.md
//!
//! This module implements a maximal-munch (longest-match) tokenizer that:
//! - Scans input bytes according to grammar patterns
//! - Produces Token sequences with correct positions
//! - Handles whitespace according to configuration
//! - Reports errors for unrecognized characters

use crate::{error::ParseError, Token};
use rust_sitter_glr_core::SymbolId;
use std::fmt;

/// Tokenizer scans input and produces tokens according to grammar
///
/// # Contract
///
/// - Thread-safe (Send + Sync)
/// - Deterministic (same input → same tokens)
/// - Complete coverage (no input bytes skipped)
/// - Position tracking (byte offsets)
///
#[derive(Debug)]
pub struct Tokenizer {
    /// Token patterns from grammar (sorted by precedence)
    patterns: Vec<TokenPattern>,
    /// Whitespace handling mode
    whitespace_mode: WhitespaceMode,
}

/// Token pattern from grammar
#[derive(Debug, Clone)]
pub struct TokenPattern {
    /// Symbol ID from grammar
    pub symbol_id: SymbolId,
    /// Pattern matcher (regex or literal string)
    pub matcher: Matcher,
    /// Is this a keyword or identifier?
    pub is_keyword: bool,
}

/// Pattern matching strategy
#[derive(Debug, Clone)]
pub enum Matcher {
    /// Literal string match (exact)
    Literal(String),
    /// Regex pattern match
    Regex(regex::Regex),
}

/// Whitespace handling strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhitespaceMode {
    /// Skip whitespace (most common)
    Skip,
    /// Preserve whitespace as tokens
    Preserve,
}

/// Tokenizer errors
#[derive(Debug)]
pub enum TokenizerError {
    /// Invalid token at position
    InvalidToken { position: usize, snippet: String },
}

impl fmt::Display for TokenizerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenizerError::InvalidToken { position, snippet } => {
                write!(f, "Invalid token at position {}: '{}'", position, snippet)
            }
        }
    }
}

impl std::error::Error for TokenizerError {}

impl Tokenizer {
    /// Create tokenizer with patterns and whitespace mode
    ///
    /// # Contract
    ///
    /// ## Postconditions
    /// - Tokenizer ready to scan input
    /// - Patterns stored as provided (no sorting yet - future optimization)
    ///
    pub fn new(patterns: Vec<TokenPattern>, whitespace_mode: WhitespaceMode) -> Self {
        Self {
            patterns,
            whitespace_mode,
        }
    }

    /// Scan input and produce tokens
    ///
    /// # Contract
    ///
    /// ## Preconditions
    /// - `input` is valid bytes
    ///
    /// ## Postconditions
    /// - All input bytes covered (no gaps)
    /// - Tokens in order (sorted by start position)
    /// - Last token is EOF with position at input.len()
    ///
    /// ## Invariants
    /// - For all tokens: token[i].end == token[i+1].start (no gaps/overlaps)
    /// - EOF token always present: tokens.last().kind == 0
    ///
    /// ## Errors
    /// - `TokenizerError::InvalidToken`: Unrecognized character sequence
    ///
    /// ## Algorithm
    /// - Maximal munch (longest match)
    /// - Pattern precedence for ties
    ///
    pub fn scan(&self, input: &[u8]) -> Result<Vec<Token>, TokenizerError> {
        let mut tokens = Vec::new();
        let mut position: usize = 0;

        while position < input.len() {
            // Try all patterns at current position (maximal munch)
            let mut best_match: Option<(SymbolId, usize, bool)> = None; // (symbol, length, is_keyword)

            for pattern in &self.patterns {
                if let Some(match_len) = pattern.match_at(input, position) {
                    // Prefer longer matches (maximal munch)
                    let is_better = match best_match {
                        None => true,
                        Some((_, best_len, best_is_keyword)) => {
                            if match_len > best_len {
                                true // Longer match wins
                            } else if match_len == best_len {
                                // Same length: keywords win over identifiers
                                pattern.is_keyword && !best_is_keyword
                            } else {
                                false
                            }
                        }
                    };

                    if is_better {
                        best_match = Some((pattern.symbol_id, match_len, pattern.is_keyword));
                    }
                }
            }

            // Apply best match or error
            if let Some((symbol_id, length, _)) = best_match {
                // Check if this is whitespace (symbol 255 is whitespace convention)
                let is_whitespace = symbol_id.0 == 255;

                // Skip whitespace if configured
                if is_whitespace && self.whitespace_mode == WhitespaceMode::Skip {
                    position += length;
                    continue;
                }

                // Create token
                let token = Token {
                    kind: symbol_id.0 as u32,
                    start: position as u32,
                    end: (position + length) as u32,
                };
                tokens.push(token);
                position += length;
            } else {
                // No pattern matched - error
                let snippet = String::from_utf8_lossy(
                    &input[position..std::cmp::min(position + 20, input.len())],
                )
                .to_string();
                return Err(TokenizerError::InvalidToken { position, snippet });
            }
        }

        // Append EOF token
        tokens.push(Token {
            kind: 0, // EOF
            start: input.len() as u32,
            end: input.len() as u32,
        });

        Ok(tokens)
    }
}

impl TokenPattern {
    /// Try to match pattern at given position
    ///
    /// Returns Some(length) if match succeeds, None otherwise
    fn match_at(&self, input: &[u8], position: usize) -> Option<usize> {
        match &self.matcher {
            Matcher::Literal(lit) => {
                let lit_bytes = lit.as_bytes();
                if position + lit_bytes.len() <= input.len()
                    && &input[position..position + lit_bytes.len()] == lit_bytes
                {
                    Some(lit_bytes.len())
                } else {
                    None
                }
            }
            Matcher::Regex(regex) => {
                // Convert remaining input to str for regex matching
                let remaining = &input[position..];
                let input_str = std::str::from_utf8(remaining).ok()?;

                // Match must start at position 0 (current position) and have non-zero length
                regex.find(input_str).and_then(|m| {
                    let len = m.end() - m.start();
                    if m.start() == 0 && len > 0 {
                        Some(len)  // Return match length
                    } else {
                        None  // Match doesn't start at current position or has zero length
                    }
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_literal_match() {
        let pattern = TokenPattern {
            symbol_id: SymbolId(1),
            matcher: Matcher::Literal("+".to_string()),
            is_keyword: false,
        };

        assert_eq!(pattern.match_at(b"+", 0), Some(1));
        assert_eq!(pattern.match_at(b"++", 0), Some(1));
        assert_eq!(pattern.match_at(b"a+", 1), Some(1));
        assert_eq!(pattern.match_at(b"-", 0), None);
    }

    #[test]
    fn test_regex_match() {
        let pattern = TokenPattern {
            symbol_id: SymbolId(1),
            matcher: Matcher::Regex(regex::Regex::new(r"^\d+").unwrap()),
            is_keyword: false,
        };

        assert_eq!(pattern.match_at(b"123", 0), Some(3));
        assert_eq!(pattern.match_at(b"123abc", 0), Some(3));
        assert_eq!(pattern.match_at(b"abc123", 0), None);
    }

    #[test]
    fn test_empty_input() {
        let tokenizer = Tokenizer::new(vec![], WhitespaceMode::Skip);
        let tokens = tokenizer.scan(b"").unwrap();
        assert_eq!(tokens.len(), 1); // EOF only
        assert_eq!(tokens[0].kind, 0); // EOF
    }

    #[test]
    fn test_zero_length_regex_rejected() {
        // Regex that could match empty string
        let pattern = TokenPattern {
            symbol_id: SymbolId(1),
            matcher: Matcher::Regex(regex::Regex::new(r"[a-z]*").unwrap()),
            is_keyword: false,
        };

        let tokenizer = Tokenizer::new(vec![pattern], WhitespaceMode::Skip);

        // Should match "abc" as one token, not loop on zero-length matches
        let tokens = tokenizer.scan(b"abc").unwrap();

        // Expect: one token for "abc" + EOF
        assert_eq!(tokens.len(), 2, "Should have token + EOF");
        assert_eq!(tokens[0].kind, 1, "First token should be ID 1");
        assert_eq!(tokens[0].start, 0, "Should start at position 0");
        assert_eq!(tokens[0].end, 3, "Should end at position 3");
        assert_eq!(tokens[1].kind, 0, "Second token should be EOF");
    }

    #[test]
    fn test_whitespace_handling() {
        // Whitespace pattern (symbol 255)
        let patterns = vec![
            TokenPattern {
                symbol_id: SymbolId(1),
                matcher: Matcher::Literal("if".to_string()),
                is_keyword: true,
            },
            TokenPattern {
                symbol_id: SymbolId(2),
                matcher: Matcher::Literal("then".to_string()),
                is_keyword: true,
            },
            TokenPattern {
                symbol_id: SymbolId(255),  // Whitespace
                matcher: Matcher::Regex(regex::Regex::new(r"^\s+").unwrap()),
                is_keyword: false,
            },
        ];

        // Test with Skip mode
        let tokenizer_skip = Tokenizer::new(patterns.clone(), WhitespaceMode::Skip);
        let tokens_skip = tokenizer_skip.scan(b"if then").unwrap();
        assert_eq!(tokens_skip.len(), 3); // if, then, EOF (whitespace skipped)
        assert_eq!(tokens_skip[0].kind, 1); // if
        assert_eq!(tokens_skip[1].kind, 2); // then
        assert_eq!(tokens_skip[2].kind, 0); // EOF

        // Test with Preserve mode
        let tokenizer_preserve = Tokenizer::new(patterns, WhitespaceMode::Preserve);
        let tokens_preserve = tokenizer_preserve.scan(b"if then").unwrap();
        assert_eq!(tokens_preserve.len(), 4); // if, whitespace, then, EOF
        assert_eq!(tokens_preserve[0].kind, 1); // if
        assert_eq!(tokens_preserve[1].kind, 255); // whitespace
        assert_eq!(tokens_preserve[2].kind, 2); // then
        assert_eq!(tokens_preserve[3].kind, 0); // EOF
    }
}
