//! Shared parse error location type.

#![forbid(unsafe_op_in_unsafe_fn)]
#![deny(missing_docs)]
#![cfg_attr(feature = "strict_api", deny(unreachable_pub))]
#![cfg_attr(not(feature = "strict_api"), warn(unreachable_pub))]
#![cfg_attr(feature = "strict_docs", deny(missing_docs))]
#![cfg_attr(not(feature = "strict_docs"), allow(missing_docs))]

use adze_linecol_core::LineCol;

/// Location information for a parse error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ErrorLocation {
    /// Byte offset in the input.
    pub byte_offset: usize,
    /// Line number (1-indexed).
    pub line: usize,
    /// Column number (1-indexed).
    pub column: usize,
}

impl ErrorLocation {
    /// Create an explicit error location.
    #[must_use]
    pub const fn new(byte_offset: usize, line: usize, column: usize) -> Self {
        Self {
            byte_offset,
            line,
            column,
        }
    }

    /// Compute line/column information from a byte offset.
    ///
    /// Line and column values are 1-indexed.
    #[must_use]
    pub fn from_byte_offset(input: &[u8], byte_offset: usize) -> Self {
        let tracker = LineCol::at_position(input, byte_offset);
        Self {
            byte_offset,
            line: tracker.line + 1,
            column: tracker.column(byte_offset) + 1,
        }
    }
}

impl std::fmt::Display for ErrorLocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.line, self.column)
    }
}

#[cfg(test)]
mod tests {
    use super::ErrorLocation;

    #[test]
    fn from_offset_uses_one_indexed_positions() {
        let loc = ErrorLocation::from_byte_offset(b"ab\ncd", 4);
        assert_eq!(loc.byte_offset, 4);
        assert_eq!(loc.line, 2);
        assert_eq!(loc.column, 2);
    }

    #[test]
    fn display_is_line_col() {
        let loc = ErrorLocation::new(0, 3, 7);
        assert_eq!(loc.to_string(), "3:7");
    }
}
