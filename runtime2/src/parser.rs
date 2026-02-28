//! Parser implementation with Tree-sitter-compatible API

#[cfg(feature = "glr-core")]
use crate::builder::forest_to_tree;
#[cfg(feature = "glr-core")]
use crate::engine::parse_full as engine_parse_full;
#[cfg(all(feature = "glr-core", feature = "incremental_glr"))]
use crate::engine::parse_incremental as engine_parse_incremental;
use crate::{error::ParseError, language::Language, tree::Tree};
#[cfg(all(feature = "pure-rust", feature = "serialization"))]
use adze_parsetable_metadata::{FORMAT_VERSION, MAGIC_NUMBER, ParsetableMetadata};
use std::time::Duration;

/// A parser that can parse text using a Language
#[derive(Debug)]
pub struct Parser {
    language: Option<Language>,
    timeout: Option<Duration>,
    #[cfg(feature = "arenas")]
    arena: Option<bumpalo::Bump>,
    /// GLR mode state (Phase 3.1)
    #[cfg(feature = "pure-rust")]
    glr_state: Option<GLRState>,
    /// Parsed metadata from `.parsetable` load.
    #[cfg(all(feature = "pure-rust", feature = "serialization"))]
    parsetable_metadata: Option<ParsetableMetadata>,
}

/// GLR parsing state (pure-Rust mode, bypasses TSLanguage)
#[cfg(feature = "pure-rust")]
#[derive(Debug)]
struct GLRState {
    /// Direct reference to ParseTable from glr-core
    parse_table: &'static adze_glr_core::ParseTable,
    /// Symbol metadata for tree construction
    symbol_metadata: Vec<crate::language::SymbolMetadata>,
    /// Token patterns for tokenizer (Phase 3.2)
    token_patterns: Option<Vec<crate::tokenizer::TokenPattern>>,
}

impl Parser {
    /// Create a new parser
    pub fn new() -> Self {
        Self {
            language: None,
            timeout: None,
            #[cfg(feature = "arenas")]
            arena: None,
            #[cfg(feature = "pure-rust")]
            glr_state: None,
            #[cfg(all(feature = "pure-rust", feature = "serialization"))]
            parsetable_metadata: None,
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
    ///
    /// # Mode Selection
    ///
    /// - If GLR mode is active (`set_glr_table()` was called), uses pure-Rust GLR engine
    /// - Otherwise, uses language-based parsing (`set_language()` was called)
    ///
    pub fn parse(
        &mut self,
        input: impl AsRef<[u8]>,
        old_tree: Option<&Tree>,
    ) -> Result<Tree, ParseError> {
        let input = input.as_ref();

        // Route to GLR engine if in pure-Rust GLR mode
        #[cfg(feature = "pure-rust")]
        if self.glr_state.is_some() {
            return self.parse_glr(input, old_tree);
        }

        // Otherwise, use language-based parsing
        let language_ptr =
            self.language.as_ref().ok_or(ParseError::no_language())? as *const Language;

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

    #[cfg(feature = "incremental_glr")]
    fn parse_incremental(
        &mut self,
        language: &Language,
        input: &[u8],
        old_tree: &Tree,
    ) -> Result<Tree, ParseError> {
        #[cfg(all(feature = "glr-core", feature = "incremental_glr"))]
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

    #[cfg(not(feature = "incremental_glr"))]
    fn parse_incremental(
        &mut self,
        language: &Language,
        input: &[u8],
        _old_tree: &Tree,
    ) -> Result<Tree, ParseError> {
        // Fall back to full parse when incremental is disabled
        self.parse_full(language, input)
    }

    /// Parse using pure-Rust GLR engine (Phase 3.1)
    ///
    /// This method is called when the parser is in GLR mode (via `set_glr_table()`).
    ///
    /// # Phase 3.2 Integration
    ///
    /// - Uses Tokenizer to scan input
    /// - Uses GLREngine to build ParseForest
    /// - Uses ForestConverter to convert forest to Tree
    ///
    #[cfg(feature = "pure-rust")]
    fn parse_glr(&mut self, input: &[u8], _old_tree: Option<&Tree>) -> Result<Tree, ParseError> {
        use crate::forest_converter::{DisambiguationStrategy, ForestConverter};
        use crate::glr_engine::{GLRConfig, GLREngine};
        use crate::tokenizer::{Tokenizer, WhitespaceMode};

        // Get GLR state
        let glr_state = self
            .glr_state
            .as_ref()
            .ok_or_else(|| ParseError::with_msg("No GLR state"))?;

        // Phase 3.2 Component 1: Tokenize input
        let tokens = if let Some(ref patterns) = glr_state.token_patterns {
            // Use real tokenizer with provided patterns
            let tokenizer = Tokenizer::new(patterns.clone(), WhitespaceMode::Skip);
            tokenizer
                .scan(input)
                .map_err(|e| ParseError::with_msg(&e.to_string()))?
        } else {
            // Fallback: stub tokenizer for backward compatibility
            vec![crate::Token {
                kind: 0, // EOF
                start: input.len() as u32,
                end: input.len() as u32,
            }]
        };

        // Phase 3.1: Parse with GLR engine
        let config = GLRConfig::default();
        let mut engine = GLREngine::new(glr_state.parse_table, config);
        let forest = engine.parse(&tokens)?;

        // Phase 3.2 Component 2: Convert forest to Tree
        let converter = ForestConverter::new(DisambiguationStrategy::PreferShift);
        let mut tree = converter
            .to_tree(&forest, input)
            .map_err(|e| ParseError::with_msg(&e.to_string()))?;

        // Phase 3.3: Build Language from ParseTable for symbol names
        let language = Self::build_language_from_parse_table(glr_state.parse_table);
        tree.set_language(language);

        // Set tree metadata
        tree.set_source(input.to_vec());

        Ok(tree)
    }

    /// Build a Language from ParseTable for symbol name resolution (Phase 3.3)
    ///
    /// Extracts symbol names from the ParseTable's grammar and creates a minimal
    /// Language struct for tree node name resolution.
    ///
    /// # Symbol Name Resolution
    ///
    /// - Terminals (tokens): Use `grammar.tokens[symbol_id].name`
    /// - Non-terminals: Use `grammar.rule_names[symbol_id]`
    /// - Unknown symbols: Use "unknown"
    ///
    #[cfg(feature = "pure-rust")]
    fn build_language_from_parse_table(
        parse_table: &'static adze_glr_core::ParseTable,
    ) -> Language {
        use std::collections::BTreeMap;

        // Find maximum symbol ID to size the symbol_names Vec correctly
        // (symbol_count may not match max symbol ID due to sparse symbol numbering)
        let max_terminal_id = parse_table
            .grammar
            .tokens
            .keys()
            .map(|id| id.0 as usize)
            .max()
            .unwrap_or(0);
        let max_nonterminal_id = parse_table
            .grammar
            .rule_names
            .keys()
            .map(|id| id.0 as usize)
            .max()
            .unwrap_or(0);
        let vec_size = (max_terminal_id.max(max_nonterminal_id) + 1).max(parse_table.symbol_count);

        // Build symbol_names Vec indexed by symbol ID
        let mut symbol_names = vec![String::from("unknown"); vec_size];

        // Add terminal (token) names
        for (symbol_id, token) in &parse_table.grammar.tokens {
            let idx = symbol_id.0 as usize;
            symbol_names[idx] = token.name.clone();
        }

        // Add non-terminal names
        for (symbol_id, name) in &parse_table.grammar.rule_names {
            let idx = symbol_id.0 as usize;
            symbol_names[idx] = name.clone();
        }

        // Create Language with symbol names
        Language {
            version: 1,
            symbol_count: parse_table.symbol_count as u32,
            field_count: 0,
            max_alias_sequence_length: 0,
            #[cfg(feature = "glr-core")]
            parse_table: Some(parse_table),
            #[cfg(not(feature = "glr-core"))]
            parse_table: crate::language::ParseTable::default(),
            #[cfg(feature = "glr-core")]
            tokenize: None,
            symbol_names,
            symbol_metadata: Vec::new(),
            field_names: Vec::new(),
            #[cfg(feature = "external_scanners")]
            external_scanner: None,
        }
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
    /// use adze_runtime::Parser;
    /// use adze_glr_core::build_lr1_automaton;
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
    #[cfg(feature = "pure-rust")]
    #[cfg_attr(docsrs, doc(cfg(feature = "pure-rust")))]
    pub fn set_glr_table(
        &mut self,
        table: &'static adze_glr_core::ParseTable,
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
            token_patterns: None,        // Will be set by set_token_patterns()
        });

        #[cfg(all(feature = "pure-rust", feature = "serialization"))]
        {
            self.parsetable_metadata = None;
        }

        // Clear LR mode state (mode switching)
        self.language = None;

        Ok(())
    }

    /// Load GLR parse table from .parsetable file bytes
    ///
    /// This is the primary method for loading pre-generated parse tables in production.
    /// The .parsetable file format is a binary format that includes:
    /// - **Magic number**: "RSPT" (Adze Parse Table)
    /// - **Format version**: u32 version number (currently 1)
    /// - **Grammar hash**: SHA-256 hash for verification
    /// - **Metadata**: JSON metadata with grammar info, statistics, and feature flags
    /// - **ParseTable**: Postcard-serialized parse table with GLR multi-action cells
    ///
    /// # File Format Layout
    ///
    /// ```text
    /// ┌────────────────────────────────┐
    /// │ "RSPT" (4 bytes)              │ Magic number
    /// ├────────────────────────────────┤
    /// │ Version: 1 (u32 LE)           │ Format version
    /// ├────────────────────────────────┤
    /// │ Grammar Hash (32 bytes)       │ SHA-256
    /// ├────────────────────────────────┤
    /// │ Metadata Length (u32 LE)      │
    /// ├────────────────────────────────┤
    /// │ Metadata JSON (variable)      │ Grammar metadata
    /// ├────────────────────────────────┤
    /// │ Table Length (u32 LE)         │
    /// ├────────────────────────────────┤
    /// │ ParseTable (postcard)         │ Serialized parse table
    /// └────────────────────────────────┘
    /// ```
    ///
    /// # Contract
    ///
    /// - Must validate magic number and format version
    /// - Must deserialize ParseTable without data loss
    /// - Must preserve GLR multi-action cells
    /// - Must leak ParseTable for 'static lifetime (safe, immutable)
    ///
    /// # Usage Flow
    ///
    /// 1. Load .parsetable file with this method
    /// 2. Set symbol metadata with `set_symbol_metadata()`
    /// 3. Set token patterns with `set_token_patterns()`
    /// 4. Parse input with `parse()`
    ///
    /// # Example
    ///
    /// ```ignore
    /// use adze_runtime::{Parser, language::SymbolMetadata, tokenizer::TokenPattern};
    ///
    /// // Step 1: Load .parsetable file
    /// let bytes = std::fs::read("grammar.parsetable")?;
    /// let mut parser = Parser::new();
    /// parser.load_glr_table_from_bytes(&bytes)?;
    ///
    /// // Step 2: Set symbol metadata
    /// let metadata = vec![
    ///     SymbolMetadata { is_terminal: true, is_visible: false, is_supertype: false },  // EOF
    ///     SymbolMetadata { is_terminal: true, is_visible: true, is_supertype: false },   // token
    ///     SymbolMetadata { is_terminal: false, is_visible: true, is_supertype: false },  // expr
    /// ];
    /// parser.set_symbol_metadata(metadata)?;
    ///
    /// // Step 3: Set token patterns
    /// let patterns = vec![
    ///     TokenPattern {
    ///         symbol_id: SymbolId(1),
    ///         matcher: Matcher::Regex(regex::Regex::new(r"[0-9]+").unwrap()),
    ///         is_keyword: false,
    ///     },
    /// ];
    /// parser.set_token_patterns(patterns)?;
    ///
    /// // Step 4: Parse
    /// let tree = parser.parse(b"42", None)?;
    /// assert_eq!(tree.root_node().kind(), "expr");
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// # Errors
    ///
    /// Returns `ParseError` if:
    /// - File is too short (< 44 bytes header)
    /// - Magic number is not "RSPT"
    /// - Format version is unsupported (not 1)
    /// - Metadata section is truncated
    /// - Table data section is truncated
    /// - ParseTable deserialization fails (corrupted postcard payload)
    ///
    /// # Performance
    ///
    /// - **Load time**: < 20ms for typical grammars (200-300 states)
    /// - **Memory overhead**: Parse table is leaked for 'static lifetime (~100-500 KB)
    /// - **Compact binary**: Uses postcard for efficient deserialization
    ///
    /// # Specification
    ///
    /// See [`docs/specs/PARSETABLE_FILE_FORMAT_SPEC.md`](https://github.com/EffortlessMetrics/adze/blob/main/docs/specs/PARSETABLE_FILE_FORMAT_SPEC.md) for complete file format specification.
    ///
    /// See [`docs/GLR_PARSETABLE_QUICKSTART.md`](https://github.com/EffortlessMetrics/adze/blob/main/docs/GLR_PARSETABLE_QUICKSTART.md) for usage guide.
    ///
    #[cfg(all(feature = "pure-rust", feature = "serialization"))]
    #[cfg_attr(
        docsrs,
        doc(cfg(all(feature = "pure-rust", feature = "serialization")))
    )]
    pub fn load_glr_table_from_bytes(&mut self, bytes: &[u8]) -> Result<(), ParseError> {
        // Parse .parsetable file format
        // Format: magic(4) + version(4) + hash(32) + metadata_len(4) + metadata(variable) + table_len(4) + table(variable)

        if bytes.len() < 44 {
            return Err(ParseError::with_msg(&format!(
                "Invalid .parsetable file: too short ({} bytes, need at least 44)",
                bytes.len()
            )));
        }

        // Verify magic number "RSPT"
        let magic = &bytes[0..4];
        if magic != MAGIC_NUMBER {
            return Err(ParseError::with_msg(&format!(
                "Invalid .parsetable file: bad magic number {:?} (expected 'RSPT')",
                magic
            )));
        }

        // Read format version
        let version = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
        if version != FORMAT_VERSION {
            return Err(ParseError::with_msg(&format!(
                "Unsupported .parsetable format version {} (expected {})",
                version, FORMAT_VERSION
            )));
        }

        // Skip grammar hash (bytes 8-40) for now
        // TODO Phase 3.3: Verify hash matches expected grammar

        // Read metadata length
        let metadata_len =
            u32::from_le_bytes([bytes[40], bytes[41], bytes[42], bytes[43]]) as usize;

        let metadata_start = 44;
        let metadata_end = metadata_start + metadata_len;

        if bytes.len() < metadata_end {
            return Err(ParseError::with_msg(&format!(
                "Invalid .parsetable file: truncated metadata (need {} bytes, have {})",
                metadata_end,
                bytes.len()
            )));
        }

        let metadata = if metadata_len == 0 {
            None
        } else {
            let metadata_bytes = &bytes[metadata_start..metadata_end];
            Some(ParsetableMetadata::from_bytes(metadata_bytes).map_err(|e| {
                ParseError::with_msg(&format!("Invalid .parsetable metadata: {}", e))
            })?)
        };

        // Read table data length
        if bytes.len() < metadata_end + 4 {
            return Err(ParseError::with_msg(
                "Invalid .parsetable file: missing table length",
            ));
        }

        let table_len = u32::from_le_bytes([
            bytes[metadata_end],
            bytes[metadata_end + 1],
            bytes[metadata_end + 2],
            bytes[metadata_end + 3],
        ]) as usize;

        let table_start = metadata_end + 4;
        let table_end = table_start + table_len;

        if bytes.len() < table_end {
            return Err(ParseError::with_msg(&format!(
                "Invalid .parsetable file: truncated table data (need {} bytes, have {})",
                table_end,
                bytes.len()
            )));
        }

        let table_bytes = &bytes[table_start..table_end];

        // Deserialize ParseTable using glr-core serialization
        let table = adze_glr_core::ParseTable::from_bytes(table_bytes).map_err(|e| {
            ParseError::with_msg(&format!("Failed to deserialize ParseTable: {}", e))
        })?;

        // Leak the table to get a 'static reference
        // This is safe because parse tables are immutable and live for the entire program
        let table_static: &'static adze_glr_core::ParseTable = Box::leak(Box::new(table));

        // Set the GLR table
        self.set_glr_table(table_static)?;
        #[cfg(all(feature = "pure-rust", feature = "serialization"))]
        {
            self.parsetable_metadata = metadata;
        }

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
    #[cfg(feature = "pure-rust")]
    #[cfg_attr(docsrs, doc(cfg(feature = "pure-rust")))]
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

    /// Set token patterns for GLR mode tokenizer (Phase 3.2)
    ///
    /// Token patterns define how to scan input into tokens for the GLR parser.
    ///
    /// # Contract
    ///
    /// - Should be called after `set_glr_table()`
    /// - Patterns define terminal symbols from the grammar
    ///
    /// # Errors
    ///
    /// - `ParseError::NoGLRState`: If `set_glr_table()` was not called first
    ///
    #[cfg(feature = "pure-rust")]
    #[cfg_attr(docsrs, doc(cfg(feature = "pure-rust")))]
    pub fn set_token_patterns(
        &mut self,
        patterns: Vec<crate::tokenizer::TokenPattern>,
    ) -> Result<(), ParseError> {
        let glr_state = self
            .glr_state
            .as_mut()
            .ok_or_else(|| ParseError::with_msg("No GLR state: call set_glr_table() first"))?;

        glr_state.token_patterns = Some(patterns);
        Ok(())
    }

    /// Check if parser is in GLR mode
    ///
    /// Returns `true` if `set_glr_table()` was called and GLR mode is active.
    #[cfg(feature = "pure-rust")]
    #[cfg_attr(docsrs, doc(cfg(feature = "pure-rust")))]
    pub fn is_glr_mode(&self) -> bool {
        self.glr_state.is_some()
    }

    /// Return metadata loaded from the last `.parsetable` file.
    ///
    /// Returns `None` when no `.parsetable` has been loaded in this parser
    /// instance or when the method is called without the required features.
    #[cfg(all(feature = "pure-rust", feature = "serialization"))]
    pub fn parsetable_metadata(&self) -> Option<&ParsetableMetadata> {
        self.parsetable_metadata.as_ref()
    }
}

impl Default for Parser {
    fn default() -> Self {
        Self::new()
    }
}
