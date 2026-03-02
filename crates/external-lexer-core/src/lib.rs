//! Shared byte-oriented lexer cursor state for external scanners.

#![forbid(unsafe_op_in_unsafe_fn)]
#![deny(missing_docs)]
#![cfg_attr(feature = "strict_api", deny(unreachable_pub))]
#![cfg_attr(not(feature = "strict_api"), warn(unreachable_pub))]
#![cfg_attr(feature = "strict_docs", deny(missing_docs))]
#![cfg_attr(not(feature = "strict_docs"), allow(missing_docs))]

use adze_linecol_core::LineCol;

/// Mutable cursor state used by external scanner adapters.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LexerCursor {
    /// Current byte position in the input.
    pub position: usize,
    /// Current 0-based line number.
    pub line: u32,
    /// Byte offset for the start of the current line.
    pub line_start: usize,
    /// End byte offset of the current token.
    pub token_end: usize,
}

impl LexerCursor {
    /// Construct a cursor at `position` and compute line metadata.
    #[must_use]
    pub fn new(input: &[u8], position: usize) -> Self {
        let tracker = LineCol::at_position(input, position);
        Self {
            position,
            line: tracker.line as u32,
            line_start: tracker.line_start,
            token_end: position,
        }
    }

    /// Get current 0-based byte column.
    #[must_use]
    pub fn column(&self) -> u32 {
        self.position.saturating_sub(self.line_start) as u32
    }

    /// Advance one logical step through input, supporting CR/LF/CRLF line endings.
    pub fn advance(&mut self, input: &[u8], skip: bool) {
        if self.position >= input.len() {
            return;
        }

        let byte = input[self.position];
        self.position += 1;

        if byte == b'\n' {
            self.line += 1;
            self.line_start = self.position;
        } else if byte == b'\r' {
            if self.position < input.len() && input[self.position] == b'\n' {
                self.position += 1;
            }
            self.line += 1;
            self.line_start = self.position;
        }

        if !skip && self.token_end < self.position {
            self.token_end = self.position;
        }
    }

    /// Set token end to current cursor position.
    pub fn mark_end(&mut self) {
        self.token_end = self.position;
    }
}
