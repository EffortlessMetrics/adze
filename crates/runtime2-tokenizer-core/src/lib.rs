//! Token and lexical scanner primitives used by `adze-runtime` pure-Rust GLR parsing.

#![warn(missing_docs)]
#![forbid(unsafe_op_in_unsafe_fn)]

use adze_glr_core::SymbolId;
use std::fmt;

/// A lexical token the GLR engine consumes.
#[derive(Debug, Clone, Copy)]
pub struct Token {
    /// Symbol id (terminal) in the grammar.
    pub kind: u32,
    /// Byte range (half-open).
    pub start: u32,
    /// Byte offset where the token ends.
    pub end: u32,
}

/// Tokenizer scans input and produces tokens according to grammar.
#[derive(Debug)]
pub struct Tokenizer {
    /// Token patterns from grammar (sorted by precedence)
    patterns: Vec<TokenPattern>,
    /// Whitespace handling mode
    whitespace_mode: WhitespaceMode,
}

/// Token pattern from grammar.
#[derive(Debug, Clone)]
pub struct TokenPattern {
    /// Symbol ID from grammar
    pub symbol_id: SymbolId,
    /// Pattern matcher (regex or literal string)
    pub matcher: Matcher,
    /// Is this a keyword or identifier?
    pub is_keyword: bool,
}

/// Pattern matching strategy.
#[derive(Debug, Clone)]
pub enum Matcher {
    /// Literal string match (exact)
    Literal(String),
    /// Regex pattern match
    Regex(regex::Regex),
}

/// Whitespace handling strategy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhitespaceMode {
    /// Skip whitespace (most common)
    Skip,
    /// Preserve whitespace as tokens
    Preserve,
}

/// Tokenizer errors.
#[derive(Debug)]
pub enum TokenizerError {
    /// Invalid token at position
    InvalidToken {
        /// Byte offset where the unrecognized character was found
        position: usize,
        /// Short snippet of the unrecognized input (up to 20 bytes)
        snippet: String,
    },
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
    /// Create tokenizer with patterns and whitespace mode.
    #[must_use]
    pub fn new(patterns: Vec<TokenPattern>, whitespace_mode: WhitespaceMode) -> Self {
        Self {
            patterns,
            whitespace_mode,
        }
    }

    /// Scan input and produce tokens.
    pub fn scan(&self, input: &[u8]) -> Result<Vec<Token>, TokenizerError> {
        let mut tokens = Vec::new();
        let mut position: usize = 0;

        while position < input.len() {
            let mut best_match: Option<(SymbolId, usize, bool)> = None;

            for pattern in &self.patterns {
                if let Some(match_len) = pattern.match_at(input, position) {
                    let is_better = match best_match {
                        None => true,
                        Some((_, best_len, best_is_keyword)) => {
                            if match_len > best_len {
                                true
                            } else if match_len == best_len {
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

            if let Some((symbol_id, length, _)) = best_match {
                let is_whitespace = symbol_id.0 == 255;
                if is_whitespace && self.whitespace_mode == WhitespaceMode::Skip {
                    position += length;
                    continue;
                }

                let token = Token {
                    kind: symbol_id.0 as u32,
                    start: position as u32,
                    end: (position + length) as u32,
                };
                tokens.push(token);
                position += length;
            } else {
                let snippet = String::from_utf8_lossy(
                    &input[position..std::cmp::min(position + 20, input.len())],
                )
                .to_string();
                return Err(TokenizerError::InvalidToken { position, snippet });
            }
        }

        tokens.push(Token {
            kind: 0,
            start: input.len() as u32,
            end: input.len() as u32,
        });

        Ok(tokens)
    }
}

impl TokenPattern {
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
                let remaining = &input[position..];
                let input_str = std::str::from_utf8(remaining).ok()?;
                regex.find(input_str).and_then(|m| {
                    if m.start() == 0 {
                        Some(m.end() - m.start())
                    } else {
                        None
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
    fn test_empty_input() {
        let tokenizer = Tokenizer::new(vec![], WhitespaceMode::Skip);
        let tokens = tokenizer.scan(b"").unwrap();
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].kind, 0);
    }
}
