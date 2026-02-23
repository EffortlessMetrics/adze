//! Core line/column byte-position tracking utilities.
//!
//! The tracker is byte-oriented and supports `\n`, `\r`, and `\r\n` line endings.

#![forbid(unsafe_op_in_unsafe_fn)]
#![deny(missing_docs)]
#![cfg_attr(feature = "strict_api", deny(unreachable_pub))]
#![cfg_attr(not(feature = "strict_api"), warn(unreachable_pub))]
#![cfg_attr(feature = "strict_docs", deny(missing_docs))]
#![cfg_attr(not(feature = "strict_docs"), allow(missing_docs))]

/// Tracks a zero-based line index and the byte offset where that line starts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LineCol {
    /// Zero-based line index.
    pub line: usize,
    /// Byte offset for the start of the current line.
    pub line_start: usize,
}

impl LineCol {
    /// Create a new tracker at line `0`, byte offset `0`.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            line: 0,
            line_start: 0,
        }
    }

    /// Compute line metadata for a byte position in `input`.
    ///
    /// If `position` is beyond `input.len()`, the end of input is used.
    #[must_use]
    pub fn at_position(input: &[u8], position: usize) -> Self {
        let mut tracker = Self::new();
        let end = position.min(input.len());

        for i in 0..end {
            if input[i] == b'\n' {
                tracker.advance_line(i + 1);
            } else if input[i] == b'\r' {
                // CRLF is counted on the LF byte, not the CR byte.
                if i + 1 < input.len() && input[i + 1] == b'\n' {
                    continue;
                }
                tracker.advance_line(i + 1);
            }
        }

        tracker
    }

    /// Advance to a new line, setting the new line's starting byte offset.
    pub fn advance_line(&mut self, new_line_start: usize) {
        self.line += 1;
        self.line_start = new_line_start;
    }

    /// Process one byte while scanning a stream and update line metadata.
    ///
    /// Returns `true` if the byte advanced to a new line.
    ///
    /// Note: for CRLF, this returns `false` for the CR byte and `true` for the LF byte.
    pub fn process_byte(&mut self, byte: u8, next_byte: Option<u8>, current_offset: usize) -> bool {
        match byte {
            b'\n' => {
                self.advance_line(current_offset + 1);
                true
            }
            b'\r' => {
                if next_byte == Some(b'\n') {
                    false
                } else {
                    self.advance_line(current_offset + 1);
                    true
                }
            }
            _ => false,
        }
    }

    /// Compute a byte-based column for `position`.
    #[must_use]
    pub fn column(&self, position: usize) -> usize {
        position.saturating_sub(self.line_start)
    }
}

impl Default for LineCol {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_newline_tracking() {
        let input = b"hello\nworld\n";
        let tracker = LineCol::at_position(input, 6);
        assert_eq!(tracker.line, 1);
        assert_eq!(tracker.line_start, 6);
        assert_eq!(tracker.column(8), 2);
    }

    #[test]
    fn crlf_handling() {
        let input = b"hello\r\nworld\r\n";
        let tracker = LineCol::at_position(input, 7);
        assert_eq!(tracker.line, 1);
        assert_eq!(tracker.line_start, 7);
        assert_eq!(tracker.column(9), 2);
    }

    #[test]
    fn cr_only_handling() {
        let input = b"hello\rworld\r";
        let tracker = LineCol::at_position(input, 6);
        assert_eq!(tracker.line, 1);
        assert_eq!(tracker.line_start, 6);
    }

    #[test]
    fn process_byte_tracks_line_boundaries() {
        let mut tracker = LineCol::new();

        assert!(!tracker.process_byte(b'a', None, 0));
        assert_eq!(tracker.line, 0);

        assert!(tracker.process_byte(b'\n', None, 5));
        assert_eq!(tracker.line, 1);
        assert_eq!(tracker.line_start, 6);

        assert!(tracker.process_byte(b'\r', Some(b'x'), 10));
        assert_eq!(tracker.line, 2);
        assert_eq!(tracker.line_start, 11);

        assert!(!tracker.process_byte(b'\r', Some(b'\n'), 15));
        assert_eq!(tracker.line, 2);
    }
}
