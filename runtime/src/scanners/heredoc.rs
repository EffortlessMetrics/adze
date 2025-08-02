// Heredoc scanner for shell-like languages
// Handles heredoc strings with custom delimiters

use crate::external_scanner::{ExternalScanner, ScanResult};
use rust_sitter_ir::SymbolId;

/// Token indices for heredoc scanner
pub const HEREDOC_START: usize = 0;
pub const HEREDOC_BODY: usize = 1;
pub const HEREDOC_END: usize = 2;

/// Heredoc scanner for shell/Ruby-like languages
pub struct HeredocScanner {
    /// Current heredoc delimiter (if inside one)
    delimiter: Option<String>,
    /// Whether we're at the start of a line
    at_line_start: bool,
}

impl ExternalScanner for HeredocScanner {
    fn new() -> Self {
        HeredocScanner {
            delimiter: None,
            at_line_start: false,
        }
    }

    fn scan(
        &mut self,
        valid_symbols: &[bool],
        input: &[u8],
        position: usize,
    ) -> Option<ScanResult> {
        // If we're inside a heredoc, look for the end delimiter
        if let Some(delimiter) = self.delimiter.clone() {
            if self.at_line_start && valid_symbols.get(HEREDOC_END) == Some(&true) {
                // Check if the line starts with the delimiter
                let delimiter_bytes = delimiter.as_bytes();
                if position + delimiter_bytes.len() <= input.len() {
                    let line_start = &input[position..position + delimiter_bytes.len()];
                    if line_start == delimiter_bytes {
                        // Check if delimiter is followed by newline or EOF
                        let after_delimiter = position + delimiter_bytes.len();
                        if after_delimiter >= input.len()
                            || input[after_delimiter] == b'\n'
                            || input[after_delimiter] == b'\r'
                        {
                            self.delimiter = None;
                            return Some(ScanResult {
                                symbol: SymbolId(HEREDOC_END as u16),
                                length: delimiter_bytes.len(),
                            });
                        }
                    }
                }
            }

            // Otherwise, scan heredoc body
            if valid_symbols.get(HEREDOC_BODY) == Some(&true) {
                let mut length = 0;
                let mut i = position;

                while i < input.len() {
                    if input[i] == b'\n' {
                        length += 1;
                        self.at_line_start = true;
                        break;
                    } else {
                        length += 1;
                        i += 1;
                        self.at_line_start = false;
                    }
                }

                if length > 0 {
                    return Some(ScanResult {
                        symbol: SymbolId(HEREDOC_BODY as u16),
                        length,
                    });
                }
            }
        } else {
            // Look for heredoc start: <<DELIMITER or <<-DELIMITER
            if valid_symbols.get(HEREDOC_START) == Some(&true) {
                if position + 2 <= input.len()
                    && input[position] == b'<'
                    && input[position + 1] == b'<'
                {
                    let mut i = position + 2;
                    let mut _indent_allowed = false;

                    // Check for <<- (indent allowed)
                    if i < input.len() && input[i] == b'-' {
                        _indent_allowed = true;
                        i += 1;
                    }

                    // Skip whitespace
                    while i < input.len() && (input[i] == b' ' || input[i] == b'\t') {
                        i += 1;
                    }

                    // Read delimiter
                    let delimiter_start = i;
                    while i < input.len() {
                        let ch = input[i];
                        if ch == b'\n'
                            || ch == b' '
                            || ch == b'\t'
                            || ch == b';'
                            || ch == b'|'
                            || ch == b'&'
                        {
                            break;
                        }
                        i += 1;
                    }

                    if i > delimiter_start {
                        let delimiter =
                            String::from_utf8_lossy(&input[delimiter_start..i]).to_string();
                        self.delimiter = Some(delimiter);
                        self.at_line_start = false;

                        return Some(ScanResult {
                            symbol: SymbolId(HEREDOC_START as u16),
                            length: i - position,
                        });
                    }
                }
            }
        }

        None
    }

    fn serialize(&self, buffer: &mut Vec<u8>) {
        // Serialize delimiter presence
        if let Some(delimiter) = &self.delimiter {
            buffer.push(1); // Has delimiter
            buffer.extend_from_slice(&(delimiter.len() as u32).to_le_bytes());
            buffer.extend_from_slice(delimiter.as_bytes());
        } else {
            buffer.push(0); // No delimiter
        }

        buffer.push(if self.at_line_start { 1 } else { 0 });
    }

    fn deserialize(&mut self, buffer: &[u8]) {
        if buffer.is_empty() {
            return;
        }

        let mut offset = 0;

        // Read delimiter presence
        if buffer[offset] == 1 {
            offset += 1;

            if offset + 4 <= buffer.len() {
                let len = u32::from_le_bytes([
                    buffer[offset],
                    buffer[offset + 1],
                    buffer[offset + 2],
                    buffer[offset + 3],
                ]) as usize;
                offset += 4;

                if offset + len <= buffer.len() {
                    self.delimiter =
                        Some(String::from_utf8_lossy(&buffer[offset..offset + len]).to_string());
                    offset += len;
                }
            }
        } else {
            self.delimiter = None;
            offset += 1;
        }

        if offset < buffer.len() {
            self.at_line_start = buffer[offset] != 0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_heredoc_basic() {
        let mut scanner = HeredocScanner::new();

        let input = b"<<EOF\nHello World\nEOF\n";
        let valid = vec![true, true, true];

        // Scan heredoc start
        let result = scanner.scan(&valid, input, 0);
        assert_eq!(
            result,
            Some(ScanResult {
                symbol: SymbolId(HEREDOC_START as u16),
                length: 5, // <<EOF
            })
        );
        assert_eq!(scanner.delimiter, Some("EOF".to_string()));

        // Scan heredoc body
        let result = scanner.scan(&valid, input, 6); // After newline
        assert_eq!(
            result,
            Some(ScanResult {
                symbol: SymbolId(HEREDOC_BODY as u16),
                length: 12, // "Hello World\n"
            })
        );

        // Scan heredoc end
        scanner.at_line_start = true;
        let result = scanner.scan(&valid, input, 18); // At "EOF"
        assert_eq!(
            result,
            Some(ScanResult {
                symbol: SymbolId(HEREDOC_END as u16),
                length: 3, // "EOF"
            })
        );
        assert_eq!(scanner.delimiter, None);
    }

    #[test]
    fn test_heredoc_with_indent() {
        let mut scanner = HeredocScanner::new();

        let input = b"<<-MARKER\n  Content\nMARKER\n";
        let valid = vec![true, true, true];

        // Scan heredoc start with indent marker
        let result = scanner.scan(&valid, input, 0);
        assert_eq!(
            result,
            Some(ScanResult {
                symbol: SymbolId(HEREDOC_START as u16),
                length: 9, // <<-MARKER
            })
        );
    }

    #[test]
    fn test_heredoc_serialization() {
        let mut scanner = HeredocScanner::new();
        scanner.delimiter = Some("DELIMITER".to_string());
        scanner.at_line_start = true;

        // Serialize
        let mut buffer = Vec::new();
        scanner.serialize(&mut buffer);

        // Deserialize
        let mut new_scanner = HeredocScanner::new();
        new_scanner.deserialize(&buffer);

        assert_eq!(new_scanner.delimiter, Some("DELIMITER".to_string()));
        assert_eq!(new_scanner.at_line_start, true);
    }
}
