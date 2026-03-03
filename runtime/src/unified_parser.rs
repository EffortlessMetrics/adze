//! Unified parser interface for different backends.
#![cfg_attr(feature = "strict_docs", allow(missing_docs))]
#![allow(dead_code)] // TODO(Phase 2 Day 5): Fix lifetime issues with Tree<'arena>

// Unified parser API - hides implementation complexity behind a clean interface
// This is the main public-facing API for adze parsing
// NOTE: This module needs updates for Tree<'arena> integration (Day 5)

use crate::parser_v4;
use crate::pure_parser::TSLanguage;
use anyhow::Result;
use std::fmt;

/// The main parser struct - provides a unified interface for all parsing needs
pub struct Parser {
    inner: Option<parser_v4::Parser>,
    language: Option<&'static TSLanguage>,
    language_name: Option<String>,
    timeout_micros: Option<u64>,
}

impl Parser {
    /// Create a new parser instance
    pub fn new() -> Self {
        Parser {
            inner: None,
            language: None,
            language_name: None,
            timeout_micros: None,
        }
    }

    /// Set the language for parsing
    ///
    /// # Arguments
    /// * `language` - The Tree-sitter language definition
    ///
    /// # Returns
    /// * `Ok(())` on success
    /// * `Err` if the language cannot be loaded or decoded
    pub fn set_language(&mut self, language: &'static TSLanguage) -> Result<()> {
        self.set_language_with_name(language, "unknown")
    }

    /// Set the language for parsing with a specific language name
    ///
    /// # Arguments
    /// * `language` - The Tree-sitter language definition
    /// * `name` - The name of the language (used for scanner registry lookup)
    ///
    /// # Returns
    /// * `Ok(())` on success
    /// * `Err` if the language cannot be loaded or decoded
    pub fn set_language_with_name(
        &mut self,
        language: &'static TSLanguage,
        name: &str,
    ) -> Result<()> {
        let language_name = name.to_string();

        // Create a V4 (GLR) parser for this language
        let v4_parser = parser_v4::Parser::from_language(language, language_name.clone());

        // Apply any previously set timeout
        if let Some(_timeout) = self.timeout_micros {
            // TODO: Add timeout support to parser_v4
            // v4_parser.set_timeout(timeout);
        }

        self.inner = Some(v4_parser);
        self.language = Some(language);
        self.language_name = Some(language_name);

        Ok(())
    }

    /// Get the name of the currently loaded language
    pub fn language_name(&self) -> Option<&str> {
        self.language_name.as_deref()
    }

    /// Check if a language has been set
    pub fn has_language(&self) -> bool {
        self.inner.is_some()
    }

    /// Parse source code into a syntax tree
    ///
    /// # Arguments
    /// * `source` - The source code to parse
    /// * `_old_tree` - Previous tree for incremental parsing (currently unused)
    ///
    /// # Returns
    /// * `Some(Tree)` on successful parse
    /// * `None` if parsing fails, is not yet supported for this backend, or no language is set
    pub fn parse<'a>(
        &'a mut self,
        source: &str,
        _old_tree: Option<&parser_v4::Tree<'a>>,
    ) -> Option<parser_v4::Tree<'a>> {
        self.parse_with_old_tree(source.as_bytes(), None, None)
    }

    /// Parse source code with incremental parsing support
    ///
    /// # Arguments
    /// * `source` - The source code to parse
    /// * `old_tree` - Previous tree for incremental parsing
    /// * `edit` - The edit that was applied to transform the old source to the new source
    ///
    /// # Returns
    /// * `Some(Tree)` on successful parse
    /// * `None` if parsing fails, if incremental parse is not yet fully supported, or no language is set
    ///
    /// # Note
    /// Currently falls back to full reparse. GLR-aware incremental parsing is being implemented.
    pub fn parse_with_old_tree(
        &mut self,
        source: &[u8],
        _old_tree: Option<&parser_v4::Tree>,
        _edit: Option<&crate::pure_incremental::Edit>,
    ) -> Option<parser_v4::Tree<'_>> {
        if let Some(ref mut parser) = self.inner {
            // TODO: Implement GLR-aware incremental parsing
            // For now, just do a full reparse
            let source_str = std::str::from_utf8(source).ok()?;
            parser.parse(source_str).ok()
        } else {
            None
        }
    }

    /// Reparse source code incrementally after an edit
    ///
    /// # Arguments
    /// * `source` - The new source code after the edit
    /// * `old_tree` - The previous parse tree
    /// * `edit` - The edit that was applied to transform the old source to the new source
    ///
    /// # Returns
    /// * `Some(Tree)` on successful reparse
    /// * `None` if reparsing fails or no language is set
    ///
    /// # Note
    /// This is the main API for GLR-aware incremental parsing.
    pub fn reparse<'a>(
        &mut self,
        source: &[u8],
        old_tree: &parser_v4::Tree<'a>,
        edit: &crate::pure_incremental::Edit,
    ) -> Option<parser_v4::Tree<'a>> {
        // Get the inner parser if it exists
        if let Some(ref inner_parser) = self.inner {
            // Now we can access the grammar and table using the new getter methods
            let grammar = inner_parser.grammar();
            let parse_table = inner_parser.parse_table();

            // Delegate to the incremental reparse implementation
            crate::glr_incremental::reparse(grammar, parse_table, source, old_tree, edit)
        } else {
            // No language set, cannot reparse
            None
        }
    }

    /// Parse source code with detailed error information
    ///
    /// # Arguments
    /// * `source` - The source code to parse
    ///
    /// # Returns
    /// * `Ok(Tree)` on successful parse
    /// * `Err` with details on internal parser errors or custom-lexer incompatibility
    pub fn parse_with_error(&mut self, source: &str) -> Result<parser_v4::Tree<'_>> {
        if let Some(ref mut parser) = self.inner {
            parser.parse(source)
        } else {
            Err(anyhow::anyhow!(
                "No language set: call set_language() before parsing"
            ))
        }
    }

    /// Set a timeout for parsing operations
    ///
    /// # Arguments
    /// * `timeout_micros` - Timeout in microseconds (0 = no timeout)
    pub fn set_timeout_micros(&mut self, timeout_micros: u64) {
        self.timeout_micros = Some(timeout_micros);
        // TODO: Add timeout support to parser_v4
        // if let Some(ref mut parser) = self.inner {
        //     parser.set_timeout(timeout_micros);
        // }
    }

    /// Reset the parser state
    ///
    /// This clears any internal state and prepares the parser for a fresh parse
    pub fn reset(&mut self) {
        if let Some(ref mut parser) = self.inner {
            parser.reset();
        }
    }

    /// Get GLR parser statistics
    ///
    /// Returns performance statistics about the GLR parsing process,
    /// including fork/merge counts and memory usage metrics.
    /// Returns None if no language is set.
    pub fn get_glr_stats(&self) -> Option<&crate::glr_forest::GLRStats> {
        self.inner.as_ref().map(|parser| parser.get_glr_stats())
    }
}

impl Default for Parser {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for Parser {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Parser")
            .field("language", &self.language_name)
            .field("has_timeout", &self.timeout_micros.is_some())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser_creation() {
        let parser = Parser::new();
        assert!(!parser.has_language());
        assert_eq!(parser.language_name(), None);
    }

    #[test]
    fn test_parse_without_language() {
        let mut parser = Parser::new();
        let result = parser.parse("test", None);
        assert!(result.is_none());
    }

    #[test]
    fn given_parser_without_language_when_parse_with_error_then_returns_explicit_message() {
        // Given
        let mut parser = Parser::new();

        // When
        let err = parser
            .parse_with_error("x = 1")
            .expect_err("parse should fail without language");

        // Then
        assert!(err.to_string().contains("No language set"));
    }

    #[test]
    fn given_timeout_set_when_debug_formatted_then_debug_includes_timeout_flag() {
        // Given
        let mut parser = Parser::new();
        parser.set_timeout_micros(5_000);

        // When
        let debug = format!("{:?}", parser);

        // Then
        assert!(debug.contains("language: None"));
        assert!(debug.contains("has_timeout: true"));
    }

    #[test]
    fn given_default_parser_when_created_then_state_matches_new_parser() {
        // Given
        let parser = Parser::default();

        // Then
        assert!(!parser.has_language());
        assert_eq!(parser.language_name(), None);
        assert!(parser.get_glr_stats().is_none());
    }
}
