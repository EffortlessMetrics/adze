//! Parser implementation with Tree-sitter-compatible API

use crate::builder::forest_to_tree;
use crate::engine::parse_full as engine_parse_full;
#[cfg(all(feature = "glr-core", feature = "incremental"))]
use crate::engine::parse_incremental as engine_parse_incremental;
use crate::{error::ParseError, language::Language, tree::Tree};
use std::time::Duration;

/// A parser that can parse text using a Language
#[derive(Debug)]
pub struct Parser {
    language: Option<Language>,
    timeout: Option<Duration>,
    #[cfg(feature = "arenas")]
    arena: Option<bumpalo::Bump>,
}

impl Parser {
    /// Create a new parser
    pub fn new() -> Self {
        Self {
            language: None,
            timeout: None,
            #[cfg(feature = "arenas")]
            arena: None,
        }
    }

    /// Set the language for parsing
    ///
    /// In GLR mode, validates that the language provides a parse table and tokenizer.
    pub fn set_language(&mut self, language: Language) -> Result<(), ParseError> {
        #[cfg(feature = "glr-core")]
        {
            if language.parse_table.is_none() {
                return Err(ParseError::with_msg("Language has no parse table"));
            }
            if language.tokenize.is_none() {
                return Err(ParseError::with_msg("Language has no tokenizer"));
            }
        }
        self.language = Some(language);
        Ok(())
    }

    /// Get the current language
    pub fn language(&self) -> Option<&Language> {
        self.language.as_ref()
    }

    /// Set a timeout for parsing operations
    pub fn set_timeout(&mut self, timeout: Duration) {
        self.timeout = Some(timeout);
    }

    /// Get the current timeout
    pub fn timeout(&self) -> Option<Duration> {
        self.timeout
    }

    /// Parse the given input text
    ///
    /// If `old_tree` is provided, performs incremental parsing.
    pub fn parse(
        &mut self,
        input: impl AsRef<[u8]>,
        old_tree: Option<&Tree>,
    ) -> Result<Tree, ParseError> {
        let language_ptr =
            self.language.as_ref().ok_or(ParseError::no_language())? as *const Language;

        let input = input.as_ref();

        // SAFETY: we only read from the language while holding an immutable reference
        let language = unsafe { &*language_ptr };

        let tree = if let Some(old) = old_tree {
            // Incremental parsing path
            self.parse_incremental(language, input, old)?
        } else {
            // Full parse
            self.parse_full(language, input)?
        };
        let mut tree = tree;
        tree.set_language(language.clone());
        tree.set_source(input.to_vec());
        Ok(tree)
    }

    /// Parse with UTF-8 string input
    pub fn parse_utf8(&mut self, input: &str, old_tree: Option<&Tree>) -> Result<Tree, ParseError> {
        self.parse(input.as_bytes(), old_tree)
    }

    fn parse_full(&mut self, language: &Language, input: &[u8]) -> Result<Tree, ParseError> {
        #[cfg(feature = "glr-core")]
        {
            let forest = engine_parse_full(language, input)?;
            Ok(forest_to_tree(forest))
        }

        #[cfg(not(feature = "glr-core"))]
        {
            let _ = (language, input);
            Err(ParseError::with_msg("GLR core feature not enabled"))
        }
    }

    #[cfg(feature = "incremental")]
    fn parse_incremental(
        &mut self,
        language: &Language,
        input: &[u8],
        old_tree: &Tree,
    ) -> Result<Tree, ParseError> {
        #[cfg(feature = "glr-core")]
        {
            if let Some(old_src) = old_tree.source_bytes() {
                if old_src == input {
                    return Ok(old_tree.clone());
                }
            }
            let forest = engine_parse_incremental(language, input, old_tree)?;
            Ok(forest_to_tree(forest))
        }

        #[cfg(not(feature = "glr-core"))]
        {
            let _ = (language, input, old_tree);
            Err(ParseError::with_msg("GLR core feature not enabled"))
        }
    }

    #[cfg(not(feature = "incremental"))]
    fn parse_incremental(
        &mut self,
        language: &Language,
        input: &[u8],
        _old_tree: &Tree,
    ) -> Result<Tree, ParseError> {
        // Fall back to full parse when incremental is disabled
        self.parse_full(language, input)
    }

    /// Reset the parser state
    pub fn reset(&mut self) {
        #[cfg(feature = "arenas")]
        if let Some(arena) = &mut self.arena {
            arena.reset();
        }
    }
}

impl Default for Parser {
    fn default() -> Self {
        Self::new()
    }
}
