// SIMD-accelerated lexer for rust-sitter (stable Rust version)
// Uses manual vectorization techniques for better performance

use crate::lexer::Token as LexerToken;
use rust_sitter_ir::{SymbolId, TokenPattern};

/// SIMD-accelerated lexer using stable Rust features
pub struct SimdLexer {
    patterns: Vec<CompiledPattern>,
    literals: Vec<LiteralPattern>,
}

/// Compiled pattern for fast matching
#[derive(Debug, Clone)]
struct CompiledPattern {
    symbol_id: SymbolId,
    pattern_type: PatternType,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
enum PatternType {
    /// Simple literal string
    Literal(Vec<u8>),
    /// Character class (e.g., [a-zA-Z])
    CharClass(CharClassMatcher),
    /// Whitespace pattern
    Whitespace,
    /// Digit pattern
    Digit,
    /// Identifier pattern
    Identifier,
    /// Complex regex (fallback to regex crate)
    Regex(regex::Regex),
}

/// Literal patterns sorted by length for efficient matching
#[derive(Debug, Clone)]
struct LiteralPattern {
    symbol_id: SymbolId,
    bytes: Vec<u8>,
}

/// Optimized character class matcher using bitmaps
#[derive(Debug, Clone)]
struct CharClassMatcher {
    /// Bitmap for ASCII characters (0-127)
    ascii_bitmap: [u64; 2],
    /// Whether to match non-ASCII characters
    match_non_ascii: bool,
}

impl CharClassMatcher {
    fn new() -> Self {
        Self {
            ascii_bitmap: [0; 2],
            match_non_ascii: false,
        }
    }

    fn add_char(&mut self, ch: char) {
        if ch as u32 <= 127 {
            let byte = ch as u8;
            let idx = (byte / 64) as usize;
            let bit = byte % 64;
            self.ascii_bitmap[idx] |= 1u64 << bit;
        } else {
            self.match_non_ascii = true;
        }
    }

    fn add_range(&mut self, start: char, end: char) {
        for ch in (start as u32)..=(end as u32) {
            if ch <= 127 {
                let byte = ch as u8;
                let idx = (byte / 64) as usize;
                let bit = byte % 64;
                self.ascii_bitmap[idx] |= 1u64 << bit;
            } else {
                self.match_non_ascii = true;
            }
        }
    }

    #[inline(always)]
    fn matches(&self, byte: u8) -> bool {
        if byte > 127 {
            return self.match_non_ascii;
        }
        let idx = (byte / 64) as usize;
        let bit = byte % 64;
        (self.ascii_bitmap[idx] & (1u64 << bit)) != 0
    }
}

impl SimdLexer {
    pub fn new(patterns: &[(SymbolId, TokenPattern)]) -> Self {
        let mut compiled_patterns = Vec::new();
        let mut literals = Vec::new();

        // Compile patterns for optimization
        for &(symbol_id, ref pattern) in patterns {
            match pattern {
                TokenPattern::String(s) => {
                    literals.push(LiteralPattern {
                        symbol_id,
                        bytes: s.as_bytes().to_vec(),
                    });
                }
                TokenPattern::Regex(r) => {
                    // Try to optimize common regex patterns
                    if let Some(optimized) = Self::optimize_regex(r) {
                        compiled_patterns.push(CompiledPattern {
                            symbol_id,
                            pattern_type: optimized,
                        });
                    } else {
                        // Fallback to regex engine
                        if let Ok(regex) = regex::Regex::new(r) {
                            compiled_patterns.push(CompiledPattern {
                                symbol_id,
                                pattern_type: PatternType::Regex(regex),
                            });
                        }
                    }
                }
            }
        }

        // Sort literals by length (longest first) for greedy matching
        literals.sort_by_key(|l| std::cmp::Reverse(l.bytes.len()));

        Self {
            patterns: compiled_patterns,
            literals,
        }
    }

    /// Try to optimize a regex pattern
    fn optimize_regex(pattern: &str) -> Option<PatternType> {
        match pattern {
            r"\s+" => Some(PatternType::Whitespace),
            r"\d+" => Some(PatternType::Digit),
            r"[a-zA-Z_][a-zA-Z0-9_]*" => Some(PatternType::Identifier),
            _ => {
                // Try to parse as character class
                if pattern.starts_with('[') && pattern.ends_with(']') {
                    Self::parse_char_class(&pattern[1..pattern.len() - 1])
                        .map(PatternType::CharClass)
                } else {
                    None
                }
            }
        }
    }

    /// Parse a character class pattern
    fn parse_char_class(pattern: &str) -> Option<CharClassMatcher> {
        let mut matcher = CharClassMatcher::new();
        let mut chars = pattern.chars().peekable();

        while let Some(ch) = chars.next() {
            if chars.peek() == Some(&'-') && chars.clone().nth(1).is_some() {
                // Range pattern
                chars.next(); // consume '-'
                if let Some(end_ch) = chars.next() {
                    matcher.add_range(ch, end_ch);
                }
            } else {
                matcher.add_char(ch);
            }
        }

        Some(matcher)
    }

    /// Scan for the next token using optimizations
    pub fn scan(&self, input: &[u8], start: usize) -> Option<LexerToken> {
        if start >= input.len() {
            return None;
        }

        let remaining = &input[start..];

        // First, try literal matching with optimized comparison
        if let Some(token) = self.scan_literals_fast(remaining, start) {
            return Some(token);
        }

        // Then try pattern matching
        for pattern in &self.patterns {
            if let Some(len) = self.match_pattern(&pattern.pattern_type, remaining) {
                return Some(LexerToken {
                    symbol: pattern.symbol_id,
                    start,
                    end: start + len,
                    text: remaining[..len].to_vec(),
                });
            }
        }

        None
    }

    /// Fast literal string matching
    fn scan_literals_fast(&self, input: &[u8], start: usize) -> Option<LexerToken> {
        for literal in &self.literals {
            if literal.bytes.len() > input.len() {
                continue;
            }

            // Use optimized comparison
            if self.fast_compare(&input[..literal.bytes.len()], &literal.bytes) {
                return Some(LexerToken {
                    symbol: literal.symbol_id,
                    start,
                    end: start + literal.bytes.len(),
                    text: literal.bytes.clone(),
                });
            }
        }

        None
    }

    /// Fast byte comparison using chunks
    #[inline(always)]
    fn fast_compare(&self, a: &[u8], b: &[u8]) -> bool {
        if a.len() != b.len() {
            return false;
        }

        // Process 8 bytes at a time using u64
        let chunks = a.len() / 8;
        let _remainder = a.len() % 8;

        // Compare 8-byte chunks
        for i in 0..chunks {
            let offset = i * 8;
            let a_chunk = u64::from_ne_bytes([
                a[offset],
                a[offset + 1],
                a[offset + 2],
                a[offset + 3],
                a[offset + 4],
                a[offset + 5],
                a[offset + 6],
                a[offset + 7],
            ]);
            let b_chunk = u64::from_ne_bytes([
                b[offset],
                b[offset + 1],
                b[offset + 2],
                b[offset + 3],
                b[offset + 4],
                b[offset + 5],
                b[offset + 6],
                b[offset + 7],
            ]);

            if a_chunk != b_chunk {
                return false;
            }
        }

        // Compare remaining bytes
        let remainder_start = chunks * 8;
        &a[remainder_start..] == &b[remainder_start..]
    }

    /// Match a pattern type against input
    fn match_pattern(&self, pattern_type: &PatternType, input: &[u8]) -> Option<usize> {
        match pattern_type {
            PatternType::Literal(bytes) => {
                if input.starts_with(bytes) {
                    Some(bytes.len())
                } else {
                    None
                }
            }
            PatternType::CharClass(matcher) => self.match_char_class_fast(matcher, input),
            PatternType::Whitespace => self.match_whitespace_fast(input),
            PatternType::Digit => self.match_digits_fast(input),
            PatternType::Identifier => self.match_identifier_fast(input),
            PatternType::Regex(regex) => {
                // Fallback to regex engine
                let text = std::str::from_utf8(input).ok()?;
                regex.find(text).map(|m| m.end())
            }
        }
    }

    /// Fast whitespace matching
    #[inline(always)]
    fn match_whitespace_fast(&self, input: &[u8]) -> Option<usize> {
        let mut len = 0;

        // Unroll loop for better performance
        let mut i = 0;
        while i + 8 <= input.len() {
            let mut all_whitespace = true;
            for j in 0..8 {
                let byte = input[i + j];
                if byte != b' ' && byte != b'\t' && byte != b'\n' && byte != b'\r' {
                    all_whitespace = false;
                    break;
                }
            }

            if all_whitespace {
                i += 8;
                len += 8;
            } else {
                break;
            }
        }

        // Handle remaining bytes
        for &byte in &input[i..] {
            if byte == b' ' || byte == b'\t' || byte == b'\n' || byte == b'\r' {
                len += 1;
            } else {
                break;
            }
        }

        if len > 0 { Some(len) } else { None }
    }

    /// Fast digit matching
    #[inline(always)]
    fn match_digits_fast(&self, input: &[u8]) -> Option<usize> {
        let mut len = 0;

        // Process multiple bytes at once
        let mut i = 0;
        while i + 4 <= input.len() {
            let mut all_digits = true;
            for j in 0..4 {
                let byte = input[i + j];
                if byte < b'0' || byte > b'9' {
                    all_digits = false;
                    break;
                }
            }

            if all_digits {
                i += 4;
                len += 4;
            } else {
                break;
            }
        }

        // Handle remaining bytes
        for &byte in &input[i..] {
            if byte >= b'0' && byte <= b'9' {
                len += 1;
            } else {
                break;
            }
        }

        if len > 0 { Some(len) } else { None }
    }

    /// Fast identifier matching
    #[inline(always)]
    fn match_identifier_fast(&self, input: &[u8]) -> Option<usize> {
        if input.is_empty() {
            return None;
        }

        // First character must be letter or underscore
        let first = input[0];
        if !((first >= b'a' && first <= b'z') || (first >= b'A' && first <= b'Z') || first == b'_')
        {
            return None;
        }

        let mut len = 1;

        // Match remaining characters
        for &byte in &input[1..] {
            if (byte >= b'a' && byte <= b'z')
                || (byte >= b'A' && byte <= b'Z')
                || (byte >= b'0' && byte <= b'9')
                || byte == b'_'
            {
                len += 1;
            } else {
                break;
            }
        }

        Some(len)
    }

    /// Fast character class matching
    #[inline(always)]
    fn match_char_class_fast(&self, matcher: &CharClassMatcher, input: &[u8]) -> Option<usize> {
        let mut len = 0;

        for &byte in input {
            if matcher.matches(byte) {
                len += 1;
            } else {
                break;
            }
        }

        if len > 0 { Some(len) } else { None }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fast_whitespace() {
        let patterns = vec![(SymbolId(1), TokenPattern::Regex(r"\s+".to_string()))];
        let lexer = SimdLexer::new(&patterns);

        let input = b"    \t\n  hello";
        let token = lexer.scan(input, 0).unwrap();
        assert_eq!(token.symbol, SymbolId(1));
        assert_eq!(token.end, 8);
    }

    #[test]
    fn test_fast_digits() {
        let patterns = vec![(SymbolId(2), TokenPattern::Regex(r"\d+".to_string()))];
        let lexer = SimdLexer::new(&patterns);

        let input = b"12345abc";
        let token = lexer.scan(input, 0).unwrap();
        assert_eq!(token.symbol, SymbolId(2));
        assert_eq!(token.end, 5);
    }

    #[test]
    fn test_fast_identifier() {
        let patterns = vec![(
            SymbolId(3),
            TokenPattern::Regex(r"[a-zA-Z_][a-zA-Z0-9_]*".to_string()),
        )];
        let lexer = SimdLexer::new(&patterns);

        let input = b"hello_world123 ";
        let token = lexer.scan(input, 0).unwrap();
        assert_eq!(token.symbol, SymbolId(3));
        assert_eq!(token.end, 14);
    }

    #[test]
    fn test_fast_literals() {
        let patterns = vec![
            (SymbolId(4), TokenPattern::String("function".to_string())),
            (SymbolId(5), TokenPattern::String("func".to_string())),
            (SymbolId(6), TokenPattern::String("fn".to_string())),
        ];
        let lexer = SimdLexer::new(&patterns);

        let input = b"function";
        let token = lexer.scan(input, 0).unwrap();
        assert_eq!(token.symbol, SymbolId(4));
        assert_eq!(token.end, 8);
    }

    #[test]
    fn test_char_class() {
        let patterns = vec![(SymbolId(7), TokenPattern::Regex(r"[a-f0-9]+".to_string()))];
        let lexer = SimdLexer::new(&patterns);

        let input = b"abc123xyz";
        let token = lexer.scan(input, 0).unwrap();
        assert_eq!(token.symbol, SymbolId(7));
        assert_eq!(token.end, 6); // "abc123"
    }
}
