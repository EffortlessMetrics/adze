/// Utilities for tracking line and column positions in text

/// Track line and column position while iterating through text
#[derive(Debug, Clone, Copy)]
pub struct LineCol {
    pub line: usize,
    pub line_start: usize, // Byte offset of the start of the current line
}

impl LineCol {
    /// Create a new LineCol tracker starting at line 0
    pub fn new() -> Self {
        Self {
            line: 0,
            line_start: 0,
        }
    }

    /// Calculate line and line_start for a given position in input
    pub fn at_position(input: &[u8], position: usize) -> Self {
        let mut tracker = Self::new();

        for i in 0..position.min(input.len()) {
            if input[i] == b'\n' {
                tracker.advance_line(i + 1);
            } else if input[i] == b'\r' {
                // Handle CR and CRLF
                if i + 1 < input.len() && input[i + 1] == b'\n' {
                    // CRLF - skip the LF part later
                    continue;
                }
                tracker.advance_line(i + 1);
            }
        }

        tracker
    }

    /// Advance to a new line at the given byte offset
    pub fn advance_line(&mut self, new_line_start: usize) {
        self.line += 1;
        self.line_start = new_line_start;
    }

    /// Process a byte and update line tracking if it's a newline
    /// Returns true if the line was advanced
    pub fn process_byte(&mut self, byte: u8, next_byte: Option<u8>, current_offset: usize) -> bool {
        match byte {
            b'\n' => {
                self.advance_line(current_offset + 1);
                true
            }
            b'\r' => {
                // Only advance if not part of CRLF
                if next_byte != Some(b'\n') {
                    self.advance_line(current_offset + 1);
                    true
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    /// Get the column (byte offset from line start) for a given position
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
    fn test_basic_newline_tracking() {
        let input = b"hello\nworld\n";
        let tracker = LineCol::at_position(input, 6);
        assert_eq!(tracker.line, 1);
        assert_eq!(tracker.line_start, 6);
        assert_eq!(tracker.column(8), 2); // 'or' in world
    }

    #[test]
    fn test_crlf_handling() {
        let input = b"hello\r\nworld\r\n";
        let tracker = LineCol::at_position(input, 7);
        assert_eq!(tracker.line, 1);
        assert_eq!(tracker.line_start, 7);
        assert_eq!(tracker.column(9), 2); // 'or' in world
    }

    #[test]
    fn test_cr_only() {
        let input = b"hello\rworld\r";
        let tracker = LineCol::at_position(input, 6);
        assert_eq!(tracker.line, 1);
        assert_eq!(tracker.line_start, 6);
    }

    #[test]
    fn test_process_byte() {
        let mut tracker = LineCol::new();

        // Regular character
        assert!(!tracker.process_byte(b'a', None, 0));
        assert_eq!(tracker.line, 0);

        // LF
        assert!(tracker.process_byte(b'\n', None, 5));
        assert_eq!(tracker.line, 1);
        assert_eq!(tracker.line_start, 6);

        // CR not followed by LF
        assert!(tracker.process_byte(b'\r', Some(b'x'), 10));
        assert_eq!(tracker.line, 2);
        assert_eq!(tracker.line_start, 11);

        // CR followed by LF (CRLF)
        assert!(!tracker.process_byte(b'\r', Some(b'\n'), 15));
        assert_eq!(tracker.line, 2); // No advance for CR in CRLF
    }
}
