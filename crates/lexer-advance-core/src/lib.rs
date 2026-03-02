//! Core helpers for advancing byte-oriented lexers while tracking line metadata.

#![forbid(unsafe_op_in_unsafe_fn)]
#![deny(missing_docs)]
#![cfg_attr(feature = "strict_api", deny(unreachable_pub))]
#![cfg_attr(not(feature = "strict_api"), warn(unreachable_pub))]
#![cfg_attr(feature = "strict_docs", deny(missing_docs))]
#![cfg_attr(not(feature = "strict_docs"), allow(missing_docs))]

/// Mutable line-tracking metadata for a byte-oriented lexer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LineState {
    /// Current zero-based line index.
    pub line: u32,
    /// Byte offset where the current line starts.
    pub line_start: usize,
}

impl LineState {
    /// Construct a line-state snapshot.
    #[must_use]
    pub const fn new(line: u32, line_start: usize) -> Self {
        Self { line, line_start }
    }
}

/// Advance a lexer by one logical unit, honoring `\n`, `\r`, and `\r\n`.
///
/// Returns the new byte position after consuming one byte or one CRLF sequence.
///
/// * For `\n`: increments `line` and sets `line_start` to the byte after `\n`.
/// * For `\r\n`: consumes both bytes, increments `line`, and sets `line_start`
///   to the byte after the `\n`.
/// * For bare `\r`: increments `line` and sets `line_start` to the byte after `\r`.
/// * For any other byte: advances by one byte with no line metadata change.
#[must_use]
pub fn advance_position(input: &[u8], position: usize, state: &mut LineState) -> usize {
    if position >= input.len() {
        return position;
    }

    let byte = input[position];
    let mut new_position = position + 1;

    if byte == b'\n' {
        state.line += 1;
        state.line_start = new_position;
    } else if byte == b'\r' {
        if new_position < input.len() && input[new_position] == b'\n' {
            new_position += 1;
        }
        state.line += 1;
        state.line_start = new_position;
    }

    new_position
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normal_byte_advances_without_line_change() {
        let input = b"abc";
        let mut state = LineState::new(2, 3);

        let pos = advance_position(input, 1, &mut state);

        assert_eq!(pos, 2);
        assert_eq!(state, LineState::new(2, 3));
    }

    #[test]
    fn lf_advances_and_updates_line_state() {
        let input = b"a\n";
        let mut state = LineState::new(0, 0);

        let pos = advance_position(input, 1, &mut state);

        assert_eq!(pos, 2);
        assert_eq!(state, LineState::new(1, 2));
    }

    #[test]
    fn crlf_consumes_both_bytes() {
        let input = b"x\r\ny";
        let mut state = LineState::new(0, 0);

        let pos = advance_position(input, 1, &mut state);

        assert_eq!(pos, 3);
        assert_eq!(state, LineState::new(1, 3));
    }

    #[test]
    fn bare_cr_advances_one_byte() {
        let input = b"x\ry";
        let mut state = LineState::new(0, 0);

        let pos = advance_position(input, 1, &mut state);

        assert_eq!(pos, 2);
        assert_eq!(state, LineState::new(1, 2));
    }
}
