//! Language representation compatible with Tree-sitter

#[cfg(feature = "glr-core")]
type TokenizerFn = dyn for<'a> Fn(&'a [u8]) -> Box<dyn Iterator<Item = crate::Token> + 'a>;

/// A language definition containing parse tables and metadata
pub struct Language {
    /// Language version for compatibility checking
    pub version: u32,
    /// Number of symbols in the grammar
    pub symbol_count: u32,
    /// Number of fields in the grammar
    pub field_count: u32,
    /// Maximum alias sequence length
    pub max_alias_sequence_length: u32,
    /// Parse table (action/goto combined for GLR)
    #[cfg(feature = "glr-core")]
    pub parse_table: Option<&'static rust_sitter_glr_core::ParseTable>,
    #[cfg(not(feature = "glr-core"))]
    pub parse_table: ParseTable,
    /// Optional tokenizer. If absent, parsing will fail with a clear error.
    #[cfg(feature = "glr-core")]
    pub tokenize: Option<Box<TokenizerFn>>,
    /// Symbol names
    pub symbol_names: Vec<String>,
    /// Symbol metadata
    pub symbol_metadata: Vec<SymbolMetadata>,
    /// Field names
    pub field_names: Vec<String>,
    /// External scanner if present
    #[cfg(feature = "external-scanners")]
    pub external_scanner: Option<Box<dyn crate::external_scanner::ExternalScanner>>,
}

/// Parse tables for GLR parsing
#[derive(Debug, Clone)]
pub struct ParseTable {
    /// State count
    pub state_count: usize,
    /// Action table: state x symbol -> Vec<Action> (multiple for conflicts)
    pub action_table: Vec<Vec<Vec<Action>>>,
    /// Small parse table (compressed representation)
    pub small_parse_table: Option<Vec<u16>>,
    /// Small parse table map
    pub small_parse_table_map: Option<Vec<u32>>,
}

/// Parser action
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    /// Shift to state
    Shift(u16),
    /// Reduce by production
    Reduce {
        /// Symbol produced by the reduction
        symbol: u16,
        /// Number of children consumed by the reduction
        child_count: u8,
    },
    /// Accept the input
    Accept,
    /// Error/invalid
    Error,
}

/// Symbol metadata
#[derive(Debug, Clone, Copy)]
pub struct SymbolMetadata {
    /// Is this a terminal symbol?
    pub is_terminal: bool,
    /// Is this symbol visible in the syntax tree?
    pub is_visible: bool,
    /// Is this a supertype?
    pub is_supertype: bool,
}

impl std::fmt::Debug for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut ds = f.debug_struct("Language");
        ds.field("version", &self.version)
            .field("symbol_count", &self.symbol_count)
            .field("field_count", &self.field_count)
            .field("max_alias_sequence_length", &self.max_alias_sequence_length);

        #[cfg(feature = "glr-core")]
        ds.field("parse_table", &self.parse_table.is_some());
        #[cfg(not(feature = "glr-core"))]
        ds.field("parse_table", &self.parse_table);

        ds.field("symbol_names", &self.symbol_names)
            .field("symbol_metadata", &self.symbol_metadata)
            .field("field_names", &self.field_names);

        #[cfg(feature = "glr-core")]
        ds.field("tokenize", &self.tokenize.is_some());

        #[cfg(feature = "external-scanners")]
        ds.field("external_scanner", &self.external_scanner.is_some());

        ds.finish()
    }
}

impl Clone for Language {
    fn clone(&self) -> Self {
        Self {
            version: self.version,
            symbol_count: self.symbol_count,
            field_count: self.field_count,
            max_alias_sequence_length: self.max_alias_sequence_length,
            #[cfg(feature = "glr-core")]
            parse_table: self.parse_table,
            #[cfg(not(feature = "glr-core"))]
            parse_table: self.parse_table.clone(),
            #[cfg(feature = "glr-core")]
            tokenize: None, // Can't clone closures, so reset
            symbol_names: self.symbol_names.clone(),
            symbol_metadata: self.symbol_metadata.clone(),
            field_names: self.field_names.clone(),
            #[cfg(feature = "external-scanners")]
            external_scanner: None,
        }
    }
}

impl Language {
    /// Convenience: turn a `Vec<Token>` into a source for tests/examples.
    #[cfg(feature = "glr-core")]
    pub fn with_static_tokens(mut self, toks: Vec<crate::Token>) -> Self {
        self.tokenize = Some(Box::new(move |_: &[u8]| {
            // This allocates a boxed iterator; fine for examples/tests.
            Box::new(toks.clone().into_iter()) as Box<dyn Iterator<Item = crate::Token>>
        }));
        self
    }

    /// Get symbol name by ID
    pub fn symbol_name(&self, id: u16) -> Option<&str> {
        self.symbol_names.get(id as usize).map(|s| s.as_str())
    }

    /// Get field name by ID
    pub fn field_name(&self, id: u16) -> Option<&str> {
        self.field_names.get(id as usize).map(|s| s.as_str())
    }

    /// Check if a symbol is terminal
    pub fn is_terminal(&self, symbol: u16) -> bool {
        self.symbol_metadata
            .get(symbol as usize)
            .is_some_and(|m| m.is_terminal)
    }

    /// Check if a symbol is visible
    pub fn is_visible(&self, symbol: u16) -> bool {
        self.symbol_metadata
            .get(symbol as usize)
            .is_some_and(|m| m.is_visible)
    }
}

/// Builder for constructing `Language` instances.
#[derive(Default)]
pub struct LanguageBuilder {
    version: u32,
    max_alias_sequence_length: u32,
    #[cfg(feature = "glr-core")]
    parse_table: Option<&'static rust_sitter_glr_core::ParseTable>,
    #[cfg(not(feature = "glr-core"))]
    parse_table: Option<ParseTable>,
    #[cfg(feature = "glr-core")]
    tokenize: Option<Box<dyn Fn(&[u8]) -> Box<dyn Iterator<Item = crate::Token> + '_>>>,
    symbol_names: Option<Vec<String>>,
    symbol_metadata: Option<Vec<SymbolMetadata>>,
    field_names: Option<Vec<String>>,
    #[cfg(feature = "external-scanners")]
    external_scanner: Option<Box<dyn crate::external_scanner::ExternalScanner>>,
}

impl Language {
    /// Start building a `Language`.
    pub fn builder() -> LanguageBuilder {
        LanguageBuilder::default()
    }
}

impl LanguageBuilder {
    /// Set the language version.
    pub fn version(mut self, version: u32) -> Self {
        self.version = version;
        self
    }

    /// Set the maximum alias sequence length.
    pub fn max_alias_sequence_length(mut self, len: u32) -> Self {
        self.max_alias_sequence_length = len;
        self
    }

    /// Provide the parse table.
    #[cfg(feature = "glr-core")]
    pub fn parse_table(mut self, table: &'static rust_sitter_glr_core::ParseTable) -> Self {
        self.parse_table = Some(table);
        self
    }

    /// Provide the parse table.
    #[cfg(not(feature = "glr-core"))]
    pub fn parse_table(mut self, table: ParseTable) -> Self {
        self.parse_table = Some(table);
        self
    }

    /// Provide symbol names.
    pub fn symbol_names(mut self, names: Vec<String>) -> Self {
        self.symbol_names = Some(names);
        self
    }

    /// Provide symbol metadata.
    pub fn symbol_metadata(mut self, meta: Vec<SymbolMetadata>) -> Self {
        self.symbol_metadata = Some(meta);
        self
    }

    /// Provide field names.
    pub fn field_names(mut self, names: Vec<String>) -> Self {
        self.field_names = Some(names);
        self
    }

    /// Provide a tokenizer.
    #[cfg(feature = "glr-core")]
    pub fn tokenizer<F>(mut self, f: F) -> Self
    where
        F: Fn(&[u8]) -> Box<dyn Iterator<Item = crate::Token> + '_> + 'static,
    {
        self.tokenize = Some(Box::new(f));
        self
    }

    /// Build the language, failing if required components are missing.
    pub fn build(self) -> Result<Language, &'static str> {
        let parse_table = self.parse_table.ok_or("missing parse table")?;
        let symbol_metadata = self.symbol_metadata.ok_or("missing symbol metadata")?;
        let symbol_names = self
            .symbol_names
            .unwrap_or_else(|| vec![String::new(); symbol_metadata.len()]);
        let field_names = self.field_names.unwrap_or_default();
        let symbol_count = symbol_names.len() as u32;
        let field_count = field_names.len() as u32;

        Ok(Language {
            version: self.version,
            symbol_count,
            field_count,
            max_alias_sequence_length: self.max_alias_sequence_length,
            #[cfg(feature = "glr-core")]
            parse_table: Some(parse_table),
            #[cfg(not(feature = "glr-core"))]
            parse_table,
            #[cfg(feature = "glr-core")]
            tokenize: self.tokenize,
            symbol_names,
            symbol_metadata,
            field_names,
            #[cfg(feature = "external-scanners")]
            external_scanner: self.external_scanner,
        })
    }
}

/// FFI-compatible language struct for C interop (future)
#[repr(C)]
pub struct TSLanguage {
    version: u32,
    symbol_count: u32,
    alias_count: u32,
    token_count: u32,
    external_token_count: u32,
    state_count: u32,
    large_state_count: u32,
    production_id_count: u32,
    field_count: u32,
    max_alias_sequence_length: u16,
    parse_table: *const u16,
    small_parse_table: *const u16,
    small_parse_table_map: *const u32,
    parse_actions: *const u32,
    symbol_names: *const *const std::os::raw::c_char,
    field_names: *const *const std::os::raw::c_char,
    // ... other fields omitted for brevity
}
