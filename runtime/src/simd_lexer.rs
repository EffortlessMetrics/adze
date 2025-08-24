// SIMD-accelerated lexer for rust-sitter
// Uses portable_simd for cross-platform SIMD operations

use std::simd::*;
use crate::lexer::{Token as LexerToken};
use rust_sitter_ir::{TokenPattern, SymbolId};
use anyhow::Result;

/// SIMD-accelerated lexer for fast token scanning
pub struct SimdLexer {
    patterns: Vec<CompiledPattern>,
    literals: Vec<LiteralPattern>,
}

/// Compiled pattern for SIMD matching
#[derive(Debug, Clone)]
struct CompiledPattern {
    symbol_id: SymbolId,
    pattern_type: PatternType,
}

#[derive(Debug, Clone)]
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
    /// Complex regex (fallback to non-SIMD)
    Regex(regex::Regex),
}

/// Literal patterns sorted by length for efficient matching
#[derive(Debug, Clone)]
struct LiteralPattern {
    symbol_id: SymbolId,
    bytes: Vec<u8>,
}

/// SIMD-optimized character class matcher
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
    
    #[inline]
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
        
        // Compile patterns for SIMD optimization
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
                        compiled_patterns.push(CompiledPattern {
                            symbol_id,
                            pattern_type: PatternType::Regex(regex::Regex::new(r).unwrap()),
                        });
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
    
    /// Try to optimize a regex pattern for SIMD
    fn optimize_regex(pattern: &str) -> Option<PatternType> {
        match pattern {
            r"\s+" => Some(PatternType::Whitespace),
            r"\d+" => Some(PatternType::Digit),
            r"[a-zA-Z_][a-zA-Z0-9_]*" => Some(PatternType::Identifier),
            _ => {
                // Try to parse as character class
                if pattern.starts_with('[') && pattern.ends_with(']') {
                    Self::parse_char_class(&pattern[1..pattern.len()-1])
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
        let mut chars = pattern.chars();
        
        while let Some(ch) = chars.next() {
            if ch == '-' && chars.as_str().len() > 0 {
                // Range
                if let Some(end) = chars.next() {
                    // Assume previous char exists
                    // This is simplified - real implementation would track state
                    continue;
                }
            } else {
                matcher.add_char(ch);
            }
        }
        
        Some(matcher)
    }
    
    /// Scan for the next token using SIMD acceleration
    pub fn scan(&self, input: &[u8], start: usize) -> Option<LexerToken> {
        if start >= input.len() {
            return None;
        }
        
        let remaining = &input[start..];
        
        // First, try literal matching with SIMD
        if let Some(token) = self.scan_literals_simd(remaining, start) {
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
    
    /// SIMD-accelerated literal string matching
    fn scan_literals_simd(&self, input: &[u8], start: usize) -> Option<LexerToken> {
        const LANES: usize = 32; // 256-bit SIMD
        
        for literal in &self.literals {
            if literal.bytes.len() > input.len() {
                continue;
            }
            
            // For short literals, use simple comparison
            if literal.bytes.len() < LANES {
                if input.starts_with(&literal.bytes) {
                    return Some(LexerToken {
                        symbol: literal.symbol_id,
                        start,
                        end: start + literal.bytes.len(),
                        text: literal.bytes.clone(),
                    });
                }
                continue;
            }
            
            // For longer literals, use SIMD
            if self.simd_compare(&input[..literal.bytes.len()], &literal.bytes) {
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
    
    /// SIMD comparison of two byte slices
    #[inline]
    fn simd_compare(&self, a: &[u8], b: &[u8]) -> bool {
        if a.len() != b.len() {
            return false;
        }
        
        const LANES: usize = 32;
        let chunks = a.len() / LANES;
        
        // SIMD comparison for aligned chunks
        for i in 0..chunks {
            let offset = i * LANES;
            let a_chunk = Simd::<u8, LANES>::from_slice(&a[offset..]);
            let b_chunk = Simd::<u8, LANES>::from_slice(&b[offset..]);
            
            if a_chunk != b_chunk {
                return false;
            }
        }
        
        // Handle remaining bytes
        let remainder = chunks * LANES;
        &a[remainder..] == &b[remainder..]
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
            PatternType::CharClass(matcher) => {
                self.match_char_class_simd(matcher, input)
            }
            PatternType::Whitespace => {
                self.match_whitespace_simd(input)
            }
            PatternType::Digit => {
                self.match_digits_simd(input)
            }
            PatternType::Identifier => {
                self.match_identifier_simd(input)
            }
            PatternType::Regex(regex) => {
                // Fallback to regex engine
                let text = std::str::from_utf8(input).ok()?;
                regex.find(text).map(|m| m.end())
            }
        }
    }
    
    /// SIMD-accelerated whitespace matching
    fn match_whitespace_simd(&self, input: &[u8]) -> Option<usize> {
        const LANES: usize = 32;
        let space = Simd::splat(b' ');
        let tab = Simd::splat(b'\t');
        let newline = Simd::splat(b'\n');
        let cr = Simd::splat(b'\r');
        
        let mut len = 0;
        let chunks = input.len() / LANES;
        
        // Process SIMD chunks
        for i in 0..chunks {
            let offset = i * LANES;
            let chunk = Simd::<u8, LANES>::from_slice(&input[offset..]);
            
            // Check if all lanes are whitespace
            let is_space = chunk.simd_eq(space);
            let is_tab = chunk.simd_eq(tab);
            let is_newline = chunk.simd_eq(newline);
            let is_cr = chunk.simd_eq(cr);
            
            let is_whitespace = is_space | is_tab | is_newline | is_cr;
            let mask = is_whitespace.to_bitmask();
            
            // Count trailing whitespace characters
            let whitespace_count = mask.trailing_ones() as usize;
            len += whitespace_count;
            
            if whitespace_count < LANES {
                break;
            }
        }
        
        // Handle remaining bytes
        let remainder_start = chunks * LANES + (len % LANES);
        for &byte in &input[remainder_start..] {
            if byte == b' ' || byte == b'\t' || byte == b'\n' || byte == b'\r' {
                len += 1;
            } else {
                break;
            }
        }
        
        if len > 0 {
            Some(len)
        } else {
            None
        }
    }
    
    /// SIMD-accelerated digit matching
    fn match_digits_simd(&self, input: &[u8]) -> Option<usize> {
        const LANES: usize = 32;
        let zero = Simd::splat(b'0');
        let nine = Simd::splat(b'9');
        
        let mut len = 0;
        let chunks = input.len() / LANES;
        
        // Process SIMD chunks
        for i in 0..chunks {
            let offset = i * LANES;
            let chunk = Simd::<u8, LANES>::from_slice(&input[offset..]);
            
            // Check if all lanes are digits (0-9)
            let ge_zero = chunk.simd_ge(zero);
            let le_nine = chunk.simd_le(nine);
            let is_digit = ge_zero & le_nine;
            let mask = is_digit.to_bitmask();
            
            // Count trailing digits
            let digit_count = mask.trailing_ones() as usize;
            len += digit_count;
            
            if digit_count < LANES {
                break;
            }
        }
        
        // Handle remaining bytes
        let remainder_start = chunks * LANES + (len % LANES);
        for &byte in &input[remainder_start..] {
            if byte >= b'0' && byte <= b'9' {
                len += 1;
            } else {
                break;
            }
        }
        
        if len > 0 {
            Some(len)
        } else {
            None
        }
    }
    
    /// SIMD-accelerated identifier matching
    fn match_identifier_simd(&self, input: &[u8]) -> Option<usize> {
        if input.is_empty() {
            return None;
        }
        
        // First character must be letter or underscore
        let first = input[0];
        if !((first >= b'a' && first <= b'z') || 
             (first >= b'A' && first <= b'Z') || 
             first == b'_') {
            return None;
        }
        
        const LANES: usize = 32;
        let lower_a = Simd::splat(b'a');
        let lower_z = Simd::splat(b'z');
        let upper_a = Simd::splat(b'A');
        let upper_z = Simd::splat(b'Z');
        let zero = Simd::splat(b'0');
        let nine = Simd::splat(b'9');
        let underscore = Simd::splat(b'_');
        
        let mut len = 1; // Already validated first character
        let remaining = &input[1..];
        let chunks = remaining.len() / LANES;
        
        // Process SIMD chunks
        for i in 0..chunks {
            let offset = i * LANES;
            let chunk = Simd::<u8, LANES>::from_slice(&remaining[offset..]);
            
            // Check if character is valid identifier character
            let is_lower = chunk.simd_ge(lower_a) & chunk.simd_le(lower_z);
            let is_upper = chunk.simd_ge(upper_a) & chunk.simd_le(upper_z);
            let is_digit = chunk.simd_ge(zero) & chunk.simd_le(nine);
            let is_underscore = chunk.simd_eq(underscore);
            
            let is_valid = is_lower | is_upper | is_digit | is_underscore;
            let mask = is_valid.to_bitmask();
            
            // Count trailing valid characters
            let valid_count = mask.trailing_ones() as usize;
            len += valid_count;
            
            if valid_count < LANES {
                break;
            }
        }
        
        // Handle remaining bytes
        let remainder_start = 1 + chunks * LANES + ((len - 1) % LANES);
        for &byte in &input[remainder_start..] {
            if (byte >= b'a' && byte <= b'z') ||
               (byte >= b'A' && byte <= b'Z') ||
               (byte >= b'0' && byte <= b'9') ||
               byte == b'_' {
                len += 1;
            } else {
                break;
            }
        }
        
        Some(len)
    }
    
    /// SIMD-accelerated character class matching
    fn match_char_class_simd(&self, matcher: &CharClassMatcher, input: &[u8]) -> Option<usize> {
        const LANES: usize = 32;
        let mut len = 0;
        
        // For now, use scalar matching
        // TODO: Optimize with SIMD lookup tables
        for &byte in input {
            if matcher.matches(byte) {
                len += 1;
            } else {
                break;
            }
        }
        
        if len > 0 {
            Some(len)
        } else {
            None
        }
    }
}

/// Benchmarking utilities
#[cfg(test)]
mod bench {
    use super::*;
    use std::time::Instant;
    
    pub fn benchmark_lexer(lexer: &SimdLexer, input: &[u8], iterations: usize) -> f64 {
        let start = Instant::now();
        
        for _ in 0..iterations {
            let mut pos = 0;
            while let Some(token) = lexer.scan(input, pos) {
                pos = token.end;
            }
        }
        
        let duration = start.elapsed();
        duration.as_secs_f64() / iterations as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_simd_whitespace() {
        let patterns = vec![
            (SymbolId(1), TokenPattern::Regex(r"\s+".to_string())),
        ];
        let lexer = SimdLexer::new(&patterns);
        
        let input = b"    \t\n  hello";
        let token = lexer.scan(input, 0).unwrap();
        assert_eq!(token.symbol, SymbolId(1));
        assert_eq!(token.end, 8);
    }
    
    #[test]
    fn test_simd_digits() {
        let patterns = vec![
            (SymbolId(2), TokenPattern::Regex(r"\d+".to_string())),
        ];
        let lexer = SimdLexer::new(&patterns);
        
        let input = b"12345abc";
        let token = lexer.scan(input, 0).unwrap();
        assert_eq!(token.symbol, SymbolId(2));
        assert_eq!(token.end, 5);
    }
    
    #[test]
    fn test_simd_identifier() {
        let patterns = vec![
            (SymbolId(3), TokenPattern::Regex(r"[a-zA-Z_][a-zA-Z0-9_]*".to_string())),
        ];
        let lexer = SimdLexer::new(&patterns);
        
        let input = b"hello_world123 ";
        let token = lexer.scan(input, 0).unwrap();
        assert_eq!(token.symbol, SymbolId(3));
        assert_eq!(token.end, 14);
    }
    
    #[test]
    fn test_simd_literals() {
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
}
