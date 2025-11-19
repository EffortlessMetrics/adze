//! Parser implementation with Tree-sitter-compatible API

#[cfg(feature = "glr-core")]
use crate::builder::forest_to_tree;
#[cfg(feature = "glr-core")]
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
    /// GLR mode state (Phase 3.1)
    #[cfg(feature = "pure-rust-glr")]
    glr_state: Option<GLRState>,
}

/// GLR parsing state (pure-Rust mode, bypasses TSLanguage)
#[cfg(feature = "pure-rust-glr")]
#[derive(Debug)]
struct GLRState {
    /// Direct reference to ParseTable from glr-core
    parse_table: &'static rust_sitter_glr_core::ParseTable,
    /// Symbol metadata for tree construction
    symbol_metadata: Vec<crate::language::SymbolMetadata>,
}

impl Parser {
    /// Create a new parser
    pub fn new() -> Self {
        Self {
            language: None,
            timeout: None,
            #[cfg(feature = "arenas")]
            arena: None,
            #[cfg(feature = "pure-rust-glr")]
            glr_state: None,
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
        if language.symbol_metadata.is_empty() {
            return Err(ParseError::with_msg("Language has no symbol metadata"));
        }
        // TODO: Validate language version compatibility
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
            self.parse_incremental(language, input, old)?
        } else {
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
        #[cfg(all(feature = "glr-core", feature = "incremental"))]
        {
            // Optimization: return early if input hasn't changed
            if let Some(old_src) = old_tree.source_bytes()
                && old_src == input
            {
                return Ok(old_tree.clone());
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

    /// Set the GLR parse table directly (pure-Rust mode, bypasses TSLanguage)
    ///
    /// This is the Phase 3.1 API that enables GLR parsing without TSLanguage encoding.
    ///
    /// # Contract
    ///
    /// - `table` must satisfy ParseTable invariants (see CONFLICT_INSPECTION_API.md)
    /// - `table.state_count > 0`
    /// - `table.action_table.len() == table.state_count`
    /// - Multi-action cells are preserved (GLR conflicts)
    ///
    /// # Mode Switching
    ///
    /// Calling this method switches the parser to GLR mode. Subsequent calls to
    /// `set_language()` will switch back to LR mode.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use rust_sitter_runtime::Parser;
    /// use rust_sitter_glr_core::build_lr1_automaton;
    ///
    /// let mut parser = Parser::new();
    /// parser.set_glr_table(&PARSE_TABLE)?;
    /// parser.set_symbol_metadata(metadata)?;
    /// let tree = parser.parse(b"1 + 2 + 3", None)?;
    /// ```
    ///
    /// # Errors
    ///
    /// - `ParseError::InvalidTable`: If table violates invariants
    ///
    #[cfg(feature = "pure-rust-glr")]
    #[cfg_attr(docsrs, doc(cfg(feature = "pure-rust-glr")))]
    pub fn set_glr_table(
        &mut self,
        table: &'static rust_sitter_glr_core::ParseTable,
    ) -> Result<(), ParseError> {
        // Validate ParseTable invariants
        if table.state_count == 0 {
            return Err(ParseError::with_msg("ParseTable has 0 states"));
        }

        if table.action_table.len() != table.state_count {
            return Err(ParseError::with_msg(&format!(
                "ParseTable invariant violation: state_count ({}) != action_table.len() ({})",
                table.state_count,
                table.action_table.len()
            )));
        }

        // Create GLR state
        self.glr_state = Some(GLRState {
            parse_table: table,
            symbol_metadata: Vec::new(), // Will be set by set_symbol_metadata()
        });

        // Clear LR mode state (mode switching)
        self.language = None;

        Ok(())
    }

    /// Set symbol metadata for GLR mode
    ///
    /// Symbol metadata is needed for tree construction in GLR mode.
    ///
    /// # Contract
    ///
    /// - Should be called after `set_glr_table()`
    /// - `metadata.len()` should match symbol count in ParseTable
    ///
    /// # Errors
    ///
    /// - `ParseError::NoGLRState`: If `set_glr_table()` was not called first
    ///
    #[cfg(feature = "pure-rust-glr")]
    #[cfg_attr(docsrs, doc(cfg(feature = "pure-rust-glr")))]
    pub fn set_symbol_metadata(
        &mut self,
        metadata: Vec<crate::language::SymbolMetadata>,
    ) -> Result<(), ParseError> {
        let glr_state = self
            .glr_state
            .as_mut()
            .ok_or_else(|| ParseError::with_msg("No GLR state: call set_glr_table() first"))?;

        glr_state.symbol_metadata = metadata;
        Ok(())
    }

    /// Check if parser is in GLR mode
    ///
    /// Returns `true` if `set_glr_table()` was called and GLR mode is active.
    #[cfg(feature = "pure-rust-glr")]
    #[cfg_attr(docsrs, doc(cfg(feature = "pure-rust-glr")))]
    pub fn is_glr_mode(&self) -> bool {
        self.glr_state.is_some()
    }
}

impl Default for Parser {
    fn default() -> Self {
        Self::new()
    }
}
