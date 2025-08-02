// User-friendly error reporting for the GLR parser
use crate::glr_parser::GLRParser;
use crate::subtree::Subtree;
use rust_sitter_ir::SymbolId;
use std::fmt;
use std::sync::Arc;

/// Parse error with location and context
#[derive(Debug, Clone)]
pub struct ParseError {
    /// Line number (1-indexed)
    pub line: usize,
    /// Column number (1-indexed)
    pub column: usize,
    /// The unexpected token
    pub unexpected_token: Option<String>,
    /// Expected tokens/symbols
    pub expected: Vec<String>,
    /// Additional context
    pub context: String,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Parse error at {}:{}: ", self.line, self.column)?;

        if let Some(ref token) = self.unexpected_token {
            write!(f, "unexpected token '{}'", token)?;
        } else {
            write!(f, "unexpected end of input")?;
        }

        if !self.expected.is_empty() {
            write!(f, ", expected one of: {}", self.expected.join(", "))?;
        }

        if !self.context.is_empty() {
            write!(f, " ({})", self.context)?;
        }

        Ok(())
    }
}

/// Error reporter that tracks parse state and generates helpful messages
pub struct ErrorReporter {
    /// Input text for error context
    input: String,
    /// Current line number
    current_line: usize,
    /// Current column number
    current_column: usize,
    /// Token positions
    token_positions: Vec<(usize, usize, usize, usize)>, // (start_line, start_col, end_line, end_col)
}

impl ErrorReporter {
    pub fn new(input: String) -> Self {
        Self {
            input,
            current_line: 1,
            current_column: 1,
            token_positions: Vec::new(),
        }
    }

    /// Record a token position
    pub fn record_token(&mut self, token: &str, _byte_offset: usize) {
        let start_line = self.current_line;
        let start_col = self.current_column;

        // Update position based on token content
        for ch in token.chars() {
            if ch == '\n' {
                self.current_line += 1;
                self.current_column = 1;
            } else {
                self.current_column += 1;
            }
        }

        let end_line = self.current_line;
        let end_col = self.current_column;

        self.token_positions
            .push((start_line, start_col, end_line, end_col));
    }

    /// Generate error at current position
    pub fn error_at_current(&self, parser: &GLRParser, unexpected: Option<String>) -> ParseError {
        let expected = self.get_expected_tokens(parser);

        ParseError {
            line: self.current_line,
            column: self.current_column,
            unexpected_token: unexpected,
            expected,
            context: self.get_context(),
        }
    }

    /// Get expected tokens from parser state
    fn get_expected_tokens(&self, _parser: &GLRParser) -> Vec<String> {
        // In a real implementation, this would examine the parse table
        // to determine valid tokens at the current state
        vec![] // Placeholder
    }

    /// Get context around the error
    fn get_context(&self) -> String {
        // Extract a line or snippet around the error position
        let lines: Vec<&str> = self.input.lines().collect();
        if self.current_line > 0 && self.current_line <= lines.len() {
            let line = lines[self.current_line - 1];
            let marker = " ".repeat(self.current_column.saturating_sub(1)) + "^";
            format!("\n{}\n{}", line, marker)
        } else {
            String::new()
        }
    }
}

/// Extension trait for GLRParser to add error reporting
pub trait ErrorReportingExt {
    fn parse_with_errors(
        &mut self,
        tokens: Vec<(SymbolId, String)>,
    ) -> Result<Subtree, Vec<ParseError>>;
}

impl ErrorReportingExt for GLRParser {
    fn parse_with_errors(
        &mut self,
        tokens: Vec<(SymbolId, String)>,
    ) -> Result<Subtree, Vec<ParseError>> {
        let mut errors = Vec::new();
        let mut reporter = ErrorReporter::new(String::new());

        for (symbol_id, token_text) in tokens {
            reporter.record_token(&token_text, 0);

            // Try to process the token
            let initial_stack_count = self.stack_count();
            self.process_token(symbol_id, &token_text, 0);

            // Check if all stacks died (parse error)
            if self.stack_count() == 0 && initial_stack_count > 0 {
                errors.push(reporter.error_at_current(self, Some(token_text.clone())));
                return Err(errors);
            }
        }

        self.process_eof();

        if let Some(tree) = self.get_best_parse() {
            Ok(Arc::try_unwrap(tree).unwrap_or_else(|arc| (*arc).clone()))
        } else {
            errors.push(reporter.error_at_current(self, None));
            Err(errors)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let error = ParseError {
            line: 3,
            column: 15,
            unexpected_token: Some("foo".to_string()),
            expected: vec!["number".to_string(), "string".to_string()],
            context: "in object member".to_string(),
        };

        let display = format!("{}", error);
        assert!(display.contains("3:15"));
        assert!(display.contains("unexpected token 'foo'"));
        assert!(display.contains("expected one of: number, string"));
        assert!(display.contains("(in object member)"));
    }

    #[test]
    fn test_error_reporter() {
        let mut reporter = ErrorReporter::new("{\n  \"key\": \n}".to_string());

        reporter.record_token("{", 0);
        assert_eq!(reporter.current_line, 1);
        assert_eq!(reporter.current_column, 2);

        reporter.record_token("\n", 1);
        assert_eq!(reporter.current_line, 2);
        assert_eq!(reporter.current_column, 1);

        reporter.record_token("\"key\"", 4);
        assert_eq!(reporter.current_line, 2);
        assert_eq!(reporter.current_column, 6);
    }
}
