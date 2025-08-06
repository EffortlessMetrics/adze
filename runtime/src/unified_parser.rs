// Unified parser API - hides implementation complexity behind a clean interface
// This is the main public-facing API for rust-sitter parsing

use crate::parser_v4;
use crate::pure_parser::TSLanguage;
use anyhow::Result;
use std::fmt;

/// The main parser struct - provides a unified interface for all parsing needs
pub struct Parser {
    inner: Option<parser_v4::Parser>,
    language_name: Option<String>,
    timeout_micros: Option<u64>,
}

impl Parser {
    /// Create a new parser instance
    pub fn new() -> Self {
        Parser {
            inner: None,
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
        // For now, use a default language name
        // TODO: Extract from symbol_names or other metadata if needed
        let language_name = "unknown".to_string();

        // Create a V4 (GLR) parser for this language
        let v4_parser = parser_v4::Parser::from_language(language, language_name.clone());
        
        // Apply any previously set timeout
        if let Some(_timeout) = self.timeout_micros {
            // TODO: Add timeout support to parser_v4
            // v4_parser.set_timeout(timeout);
        }
        
        self.inner = Some(v4_parser);
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
    /// * `old_tree` - Optional previous tree for incremental parsing (not yet supported)
    /// 
    /// # Returns
    /// * `Some(Tree)` on successful parse
    /// * `None` if parsing fails or no language is set
    pub fn parse(&mut self, source: &str, _old_tree: Option<&parser_v4::Tree>) -> Option<parser_v4::Tree> {
        if let Some(ref mut parser) = self.inner {
            // For now, ignore old_tree (incremental parsing to be added)
            parser.parse(source).ok()
        } else {
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
    /// * `Err` with details about what went wrong
    pub fn parse_with_error(&mut self, source: &str) -> Result<parser_v4::Tree> {
        if let Some(ref mut parser) = self.inner {
            parser.parse(source)
        } else {
            Err(anyhow::anyhow!("No language set"))
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
}
