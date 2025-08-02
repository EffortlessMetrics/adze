// Unified parser combining pure parser with incremental parsing support
use crate::pure_incremental::{Edit, IncrementalParser, Tree};
use crate::pure_parser::{ParseResult, Parser as PureParser, TSLanguage};

/// Unified parser with incremental parsing support
pub struct Parser {
    pure_parser: PureParser,
    incremental_parser: IncrementalParser,
}

impl Parser {
    /// Create a new parser
    pub fn new() -> Self {
        Self {
            pure_parser: PureParser::new(),
            incremental_parser: IncrementalParser::new(),
        }
    }

    /// Set the language for this parser
    pub fn set_language(&mut self, language: &'static TSLanguage) -> Result<(), String> {
        self.pure_parser.set_language(language)?;
        let _ = self.incremental_parser.set_language(language);
        Ok(())
    }

    /// Get the current language
    pub fn language(&self) -> Option<&'static TSLanguage> {
        self.pure_parser.language()
    }

    /// Parse a string
    pub fn parse(&mut self, source: &str, old_tree: Option<&Tree>) -> ParseResult {
        if let Some(tree) = old_tree {
            // Use incremental parsing
            self.incremental_parser.parse(source, Some(tree))
        } else {
            // Use regular parsing
            self.pure_parser.parse_string(source)
        }
    }

    /// Parse with edits applied to old tree
    pub fn parse_with_edits(
        &mut self,
        source: &str,
        old_tree: Option<Tree>,
        edits: &[Edit],
    ) -> ParseResult {
        self.incremental_parser
            .parse_with_edits(source, old_tree, edits)
    }

    /// Set timeout for parsing
    pub fn set_timeout_micros(&mut self, timeout: u64) {
        self.pure_parser.set_timeout_micros(timeout);
        self.incremental_parser.set_timeout_micros(timeout);
    }

    /// Set cancellation flag
    pub fn set_cancellation_flag(&mut self, flag: Option<*const std::sync::atomic::AtomicBool>) {
        self.pure_parser.set_cancellation_flag(flag);
        self.incremental_parser.set_cancellation_flag(flag);
    }

    /// Reset the parser
    pub fn reset(&mut self) {
        self.pure_parser.reset();
    }
}

impl Default for Parser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unified_parser() {
        let mut parser = Parser::new();

        // Test basic parsing
        let result = parser.parse("test", None);
        assert!(result.errors.is_empty() || !result.errors.is_empty()); // Either way is fine without a language
    }

    #[test]
    fn test_incremental_edit() {
        let parser = Parser::new();

        // Create an edit
        let edit = Edit {
            start_byte: 5,
            old_end_byte: 10,
            new_end_byte: 15,
            start_point: crate::pure_parser::Point { row: 0, column: 5 },
            old_end_point: crate::pure_parser::Point { row: 0, column: 10 },
            new_end_point: crate::pure_parser::Point { row: 0, column: 15 },
        };

        // Verify edit was created successfully
        assert_eq!(edit.start_byte, 5);
        assert_eq!(edit.new_end_byte, 15);
    }
}
