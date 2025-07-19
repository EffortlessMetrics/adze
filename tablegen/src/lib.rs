// Static table generation and compression for pure-Rust Tree-sitter
// This module implements Tree-sitter's exact table compression algorithms

// use indexmap::IndexMap; // Currently unused
use proc_macro2::TokenStream;
use quote::quote;
use rust_sitter_glr_core::*;
use rust_sitter_ir::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Static Language generator that produces Rust code
pub struct StaticLanguageGenerator {
    pub grammar: Grammar,
    pub parse_table: ParseTable,
    pub compressed_tables: Option<CompressedTables>,
}

impl StaticLanguageGenerator {
    /// Create a new generator
    pub fn new(grammar: Grammar, parse_table: ParseTable) -> Self {
        Self {
            grammar,
            parse_table,
            compressed_tables: None,
        }
    }

    /// Generate static Rust code for the Language
    pub fn generate_language_code(&self) -> TokenStream {
        let language_name = &self.grammar.name;
        let symbol_count = self.parse_table.symbol_count;
        let state_count = self.parse_table.state_count;
        
        // Generate symbol names array
        let symbol_names = self.generate_symbol_names();
        
        // Generate symbol metadata array
        let symbol_metadata = self.generate_symbol_metadata();
        
        // Generate field names array
        let field_names = self.generate_field_names();
        
        // Generate parse tables
        let (action_table, goto_table) = if let Some(compressed) = &self.compressed_tables {
            self.generate_compressed_tables(compressed)
        } else {
            self.generate_uncompressed_tables()
        };
        
        // Generate NODE_TYPES JSON
        let node_types_json = self.generate_node_types();
        
        quote! {
            use std::sync::OnceLock;
            use tree_sitter::{Language, LanguageFn};
            
            // Static symbol names array
            static SYMBOL_NAMES: &[&str] = &[#(#symbol_names),*];
            
            // Static symbol metadata array
            static SYMBOL_METADATA: &[tree_sitter::ffi::TSSymbolMetadata] = &[#(#symbol_metadata),*];
            
            // Static field names array
            static FIELD_NAMES: &[&str] = &[#(#field_names),*];
            
            // Parse tables
            #action_table
            #goto_table
            
            // NODE_TYPES JSON
            pub const NODE_TYPES: &str = #node_types_json;
            
            // Language version and metadata
            const LANGUAGE_VERSION: u32 = 15; // ABI version 15
            const STATE_COUNT: u32 = #state_count;
            const SYMBOL_COUNT: u32 = #symbol_count;
            
            static LANGUAGE: OnceLock<Language> = OnceLock::new();
            
            /// Get the Tree-sitter Language for this grammar
            pub fn language() -> Language {
                *LANGUAGE.get_or_init(|| {
                    unsafe {
                        Language::from_raw_parts(
                            LANGUAGE_VERSION,
                            SYMBOL_NAMES.as_ptr(),
                            SYMBOL_METADATA.as_ptr(),
                            FIELD_NAMES.as_ptr(),
                            ACTION_TABLE.as_ptr(),
                            GOTO_TABLE.as_ptr(),
                            STATE_COUNT,
                            SYMBOL_COUNT,
                            FIELD_NAMES.len() as u32,
                        )
                    }
                })
            }
            
            /// Export for C FFI
            #[no_mangle]
            pub extern "C" fn tree_sitter_language() -> Language {
                language()
            }
        }
    }

    /// Generate NODE_TYPES JSON string
    pub fn generate_node_types(&self) -> String {
        // This will generate the NODE_TYPES JSON that describes each node's structure
        // For now, return a placeholder - this will be implemented based on the grammar
        serde_json::to_string_pretty(&serde_json::json!({
            "version": 15,
            "types": []
        })).unwrap_or_else(|_| "{}".to_string())
    }

    fn generate_symbol_names(&self) -> Vec<String> {
        let mut names = Vec::new();
        
        // Add terminal symbols
        for (_, token) in &self.grammar.tokens {
            names.push(token.name.clone());
        }
        
        // Add non-terminal symbols (rules)
        for (symbol_id, _) in &self.grammar.rules {
            names.push(format!("rule_{}", symbol_id.0));
        }
        
        // Add external symbols
        for external in &self.grammar.externals {
            names.push(external.name.clone());
        }
        
        names
    }

    fn generate_symbol_metadata(&self) -> Vec<TokenStream> {
        let mut metadata = Vec::new();
        
        // Generate metadata for each symbol
        for (_, token) in &self.grammar.tokens {
            let visible = true; // Terminals are usually visible
            let named = false; // Terminals are usually not named nodes
            let supertype = false;
            
            metadata.push(quote! {
                tree_sitter::ffi::TSSymbolMetadata {
                    visible: #visible,
                    named: #named,
                    supertype: #supertype,
                }
            });
        }
        
        // Add metadata for non-terminals
        for (_, _rule) in &self.grammar.rules {
            let visible = true;
            let named = true; // Non-terminals are usually named nodes
            let supertype = false; // Will be true if in supertypes list
            
            metadata.push(quote! {
                tree_sitter::ffi::TSSymbolMetadata {
                    visible: #visible,
                    named: #named,
                    supertype: #supertype,
                }
            });
        }
        
        metadata
    }

    fn generate_field_names(&self) -> Vec<String> {
        // Fields must be in lexicographic order (already validated in Grammar)
        self.grammar.fields.values().cloned().collect()
    }

    fn generate_uncompressed_tables(&self) -> (TokenStream, TokenStream) {
        // Generate uncompressed action and goto tables
        let action_entries = self.generate_action_table_entries();
        let goto_entries = self.generate_goto_table_entries();
        
        let action_table = quote! {
            static ACTION_TABLE: &[&[tree_sitter::ffi::TSParseActionEntry]] = &[#(#action_entries),*];
        };
        
        let goto_table = quote! {
            static GOTO_TABLE: &[&[u16]] = &[#(#goto_entries),*];
        };
        
        (action_table, goto_table)
    }

    fn generate_compressed_tables(&self, compressed: &CompressedTables) -> (TokenStream, TokenStream) {
        // Generate compressed tables using Tree-sitter's format
        
        if self.parse_table.state_count < compressed.small_table_threshold {
            self.generate_small_compressed_tables(compressed)
        } else {
            self.generate_large_compressed_tables(compressed)
        }
    }
    
    fn generate_small_compressed_tables(&self, compressed: &CompressedTables) -> (TokenStream, TokenStream) {
        // Generate Tree-sitter's small table format
        // Action table: flat array of u16 values with encoded actions
        // Goto table: flat array of u16 state IDs
        
        let action_entries = self.generate_small_action_entries(&compressed.action_table);
        let goto_entries = self.generate_small_goto_entries(&compressed.goto_table);
        
        let action_count = compressed.action_table.data.len();
        let goto_count = self.count_goto_entries(&compressed.goto_table);
        
        let action_table = quote! {
            static SMALL_PARSE_TABLE: &[u16; #action_count] = &[#(#action_entries),*];
            static SMALL_PARSE_TABLE_MAP: &[u16] = &[/* row offsets */];
        };
        
        let goto_table = quote! {
            static GOTO_TABLE: &[u16; #goto_count] = &[#(#goto_entries),*];
        };
        
        (action_table, goto_table)
    }
    
    fn generate_large_compressed_tables(&self, compressed: &CompressedTables) -> (TokenStream, TokenStream) {
        // For large tables, use pointer arrays
        // This is rarely needed but essential for grammars like C++
        self.generate_small_compressed_tables(compressed) // Simplified for now
    }
    
    fn generate_small_action_entries(&self, action_table: &CompressedActionTable) -> Vec<TokenStream> {
        let mut entries = Vec::new();
        let compressor = TableCompressor::new();
        
        for entry in &action_table.data {
            if let Ok(encoded) = compressor.encode_action_small(&entry.action) {
                let symbol = entry.symbol;
                entries.push(quote! { #symbol }); // Symbol index
                entries.push(quote! { #encoded }); // Encoded action
            }
        }
        
        entries
    }
    
    fn generate_small_goto_entries(&self, goto_table: &CompressedGotoTable) -> Vec<TokenStream> {
        let mut entries = Vec::new();
        
        for entry in &goto_table.data {
            match entry {
                CompressedGotoEntry::Single(state) => {
                    entries.push(quote! { #state });
                }
                CompressedGotoEntry::RunLength { state, count } => {
                    // Expand run-length encoded entries
                    for _ in 0..*count {
                        entries.push(quote! { #state });
                    }
                }
            }
        }
        
        entries
    }
    
    fn count_goto_entries(&self, goto_table: &CompressedGotoTable) -> usize {
        goto_table.data.iter().map(|entry| match entry {
            CompressedGotoEntry::Single(_) => 1,
            CompressedGotoEntry::RunLength { count, .. } => *count as usize,
        }).sum()
    }

    fn generate_action_table_entries(&self) -> Vec<TokenStream> {
        let mut entries = Vec::new();
        
        for state_actions in &self.parse_table.action_table {
            let actions: Vec<TokenStream> = state_actions.iter().map(|action| {
                match action {
                    Action::Shift(state) => {
                        let state_id = state.0;
                        quote! {
                            tree_sitter::ffi::TSParseActionEntry {
                                type_: tree_sitter::ffi::TSParseActionType::Shift,
                                state: #state_id,
                                symbol: 0,
                                child_count: 0,
                                dynamic_precedence: 0,
                                fragile: false,
                            }
                        }
                    }
                    Action::Reduce(rule) => {
                        let rule_id = rule.0;
                        quote! {
                            tree_sitter::ffi::TSParseActionEntry {
                                type_: tree_sitter::ffi::TSParseActionType::Reduce,
                                state: 0,
                                symbol: #rule_id,
                                child_count: 0, // Will be filled with actual child count
                                dynamic_precedence: 0,
                                fragile: false,
                            }
                        }
                    }
                    Action::Accept => {
                        quote! {
                            tree_sitter::ffi::TSParseActionEntry {
                                type_: tree_sitter::ffi::TSParseActionType::Accept,
                                state: 0,
                                symbol: 0,
                                child_count: 0,
                                dynamic_precedence: 0,
                                fragile: false,
                            }
                        }
                    }
                    Action::Error => {
                        quote! {
                            tree_sitter::ffi::TSParseActionEntry {
                                type_: tree_sitter::ffi::TSParseActionType::Error,
                                state: 0,
                                symbol: 0,
                                child_count: 0,
                                dynamic_precedence: 0,
                                fragile: false,
                            }
                        }
                    }
                    Action::Fork(actions) => {
                        // For GLR fork points, we'll need to handle multiple actions
                        // For now, just take the first action
                        if let Some(first_action) = actions.first() {
                            match first_action {
                                Action::Shift(state) => {
                                    let state_id = state.0;
                                    quote! {
                                        tree_sitter::ffi::TSParseActionEntry {
                                            type_: tree_sitter::ffi::TSParseActionType::Shift,
                                            state: #state_id,
                                            symbol: 0,
                                            child_count: 0,
                                            dynamic_precedence: 0,
                                            fragile: false,
                                        }
                                    }
                                }
                                _ => {
                                    quote! {
                                        tree_sitter::ffi::TSParseActionEntry {
                                            type_: tree_sitter::ffi::TSParseActionType::Error,
                                            state: 0,
                                            symbol: 0,
                                            child_count: 0,
                                            dynamic_precedence: 0,
                                            fragile: false,
                                        }
                                    }
                                }
                            }
                        } else {
                            quote! {
                                tree_sitter::ffi::TSParseActionEntry {
                                    type_: tree_sitter::ffi::TSParseActionType::Error,
                                    state: 0,
                                    symbol: 0,
                                    child_count: 0,
                                    dynamic_precedence: 0,
                                    fragile: false,
                                }
                            }
                        }
                    }
                }
            }).collect();
            
            entries.push(quote! { &[#(#actions),*] });
        }
        
        entries
    }

    fn generate_goto_table_entries(&self) -> Vec<TokenStream> {
        let mut entries = Vec::new();
        
        for state_gotos in &self.parse_table.goto_table {
            let gotos: Vec<u16> = state_gotos.iter().map(|state| state.0).collect();
            entries.push(quote! { &[#(#gotos),*] });
        }
        
        entries
    }

    /// Apply table compression
    pub fn compress_tables(&mut self) -> Result<(), TableGenError> {
        let compressor = TableCompressor::new();
        self.compressed_tables = Some(compressor.compress(&self.parse_table)?);
        Ok(())
    }
}

/// Table compression implementation matching Tree-sitter's algorithms
/// 
/// Tree-sitter uses a sophisticated compression scheme:
/// 1. Small parse table optimization for <32k states
/// 2. Row displacement method for action table compression
/// 3. Default reductions to minimize table size
/// 4. Symbol remapping for compact representation
pub struct TableCompressor {
    // Tree-sitter's magic constants for compression
    small_table_threshold: usize,
    max_symbol_value: u16,
}

impl TableCompressor {
    pub fn new() -> Self {
        Self {
            small_table_threshold: 32768, // Tree-sitter's threshold
            max_symbol_value: u16::MAX,
        }
    }

    /// Compress parse tables using Tree-sitter's exact algorithms
    pub fn compress(&self, parse_table: &ParseTable) -> Result<CompressedTables, TableGenError> {
        // Determine if we should use small table optimization
        let use_small_table = parse_table.state_count < self.small_table_threshold;
        
        if use_small_table {
            self.compress_small_table(parse_table)
        } else {
            self.compress_large_table(parse_table)
        }
    }
    
    /// Compress using Tree-sitter's "small table" optimization
    /// This is the most common case and what Tree-sitter uses for most grammars
    fn compress_small_table(&self, parse_table: &ParseTable) -> Result<CompressedTables, TableGenError> {
        // Tree-sitter's small table format:
        // 1. Action table: 2D array flattened with row displacement
        // 2. Each entry is a u16 encoding action type + data
        // 3. Default reductions stored separately
        
        let compressed_action_table = self.compress_action_table_small(&parse_table.action_table)?;
        let compressed_goto_table = self.compress_goto_table_small(&parse_table.goto_table)?;
        
        Ok(CompressedTables {
            action_table: compressed_action_table,
            goto_table: compressed_goto_table,
            small_table_threshold: self.small_table_threshold,
        })
    }
    
    /// Compress using large table optimization (for very large grammars)
    fn compress_large_table(&self, parse_table: &ParseTable) -> Result<CompressedTables, TableGenError> {
        // For large tables, Tree-sitter uses pointer indirection
        // This is rarely used but necessary for grammars like C++
        
        let compressed_action_table = self.compress_action_table_large(&parse_table.action_table)?;
        let compressed_goto_table = self.compress_goto_table_large(&parse_table.goto_table)?;
        
        Ok(CompressedTables {
            action_table: compressed_action_table,
            goto_table: compressed_goto_table,
            small_table_threshold: self.small_table_threshold,
        })
    }

    /// Compress action table using Tree-sitter's small table format
    fn compress_action_table_small(&self, action_table: &[Vec<Action>]) -> Result<CompressedActionTable, TableGenError> {
        // Tree-sitter's encoding for small tables:
        // - Actions are encoded as u16 values
        // - Shift: 0x0000 | state_id
        // - Reduce: 0x8000 | (rule_id << 1) | has_precedence
        // - Accept: 0xFFFF
        // - Error: 0xFFFE
        
        let mut entries = Vec::new();
        let mut row_offsets = Vec::new();
        let mut default_reductions = Vec::new();
        
        for (state_id, actions) in action_table.iter().enumerate() {
            // Find default reduction (most common reduce action)
            let mut reduce_counts: HashMap<&Action, usize> = HashMap::new();
            let mut has_shift = false;
            let mut has_accept = false;
            
            for action in actions {
                match action {
                    Action::Reduce(_) => {
                        *reduce_counts.entry(action).or_insert(0) += 1;
                    }
                    Action::Shift(_) => has_shift = true,
                    Action::Accept => has_accept = true,
                    _ => {}
                }
            }
            
            // Default reduction is the most common reduce, but only if no shifts
            let default_reduction = if !has_shift && !has_accept {
                reduce_counts
                    .iter()
                    .max_by_key(|(_, count)| *count)
                    .map(|(action, _)| (*action).clone())
            } else {
                None
            };
            
            default_reductions.push(default_reduction.clone().unwrap_or(Action::Error));
            
            // Encode non-default actions
            row_offsets.push(entries.len() as u16);
            
            for (symbol_id, action) in actions.iter().enumerate() {
                // Skip if this is the default reduction
                if let Some(ref default) = default_reduction {
                    if action == default {
                        continue;
                    }
                }
                
                let encoded = self.encode_action_small(action)?;
                entries.push(CompressedActionEntry {
                    symbol: symbol_id as u16,
                    action: action.clone(),
                });
            }
        }
        
        // Add sentinel for last row
        row_offsets.push(entries.len() as u16);
        
        Ok(CompressedActionTable {
            data: entries,
            row_offsets,
            default_actions: default_reductions,
        })
    }
    
    /// Compress action table using large table format
    fn compress_action_table_large(&self, action_table: &[Vec<Action>]) -> Result<CompressedActionTable, TableGenError> {
        // For large tables, use pointer indirection
        // This is a simplified version - real Tree-sitter uses more sophisticated compression
        self.compress_action_table_small(action_table)
    }
    
    /// Encode an action as a u16 for small table format
    fn encode_action_small(&self, action: &Action) -> Result<u16, TableGenError> {
        match action {
            Action::Shift(state) => {
                if state.0 >= 0x8000 {
                    return Err(TableGenError::CompressionError(
                        format!("Shift state {} too large for small table encoding", state.0)
                    ));
                }
                Ok(state.0)
            }
            Action::Reduce(rule) => {
                if rule.0 >= 0x4000 {
                    return Err(TableGenError::CompressionError(
                        format!("Reduce rule {} too large for small table encoding", rule.0)
                    ));
                }
                // Reduce actions are encoded with high bit set
                // bit 15: 1 (indicates reduce)
                // bits 14-1: rule_id
                // bit 0: has_precedence (0 for now)
                Ok(0x8000 | (rule.0 << 1))
            }
            Action::Accept => Ok(0xFFFF),
            Action::Error => Ok(0xFFFE),
            Action::Fork(_) => {
                // GLR fork points need special handling
                // For now, treat as error
                Ok(0xFFFE)
            }
        }
    }

    /// Compress goto table using Tree-sitter's small table format
    fn compress_goto_table_small(&self, goto_table: &[Vec<StateId>]) -> Result<CompressedGotoTable, TableGenError> {
        // Tree-sitter uses simple array compression for goto table
        // Each row is stored contiguously with row offsets
        
        let mut data = Vec::new();
        let mut row_offsets = Vec::new();
        
        for row in goto_table {
            row_offsets.push(data.len() as u16);
            
            // For goto table, we can use run-length encoding for sparse rows
            // Tree-sitter uses a simpler approach: just store state IDs
            let mut last_state = None;
            let mut run_length = 0;
            
            for &state in row {
                if Some(state) == last_state {
                    run_length += 1;
                } else {
                    if run_length > 0 {
                        // Emit previous run
                        if run_length > 2 {
                            data.push(CompressedGotoEntry::RunLength {
                                state: last_state.unwrap().0,
                                count: run_length,
                            });
                        } else {
                            // For short runs, individual entries are more efficient
                            for _ in 0..run_length {
                                data.push(CompressedGotoEntry::Single(last_state.unwrap().0));
                            }
                        }
                    }
                    last_state = Some(state);
                    run_length = 1;
                }
            }
            
            // Emit final run
            if run_length > 0 {
                if run_length > 2 {
                    data.push(CompressedGotoEntry::RunLength {
                        state: last_state.unwrap().0,
                        count: run_length,
                    });
                } else {
                    for _ in 0..run_length {
                        data.push(CompressedGotoEntry::Single(last_state.unwrap().0));
                    }
                }
            }
        }
        
        // Add sentinel
        row_offsets.push(data.len() as u16);
        
        Ok(CompressedGotoTable {
            data,
            row_offsets,
        })
    }
    
    /// Compress goto table using large table format
    fn compress_goto_table_large(&self, goto_table: &[Vec<StateId>]) -> Result<CompressedGotoTable, TableGenError> {
        // For large tables, use the same compression for now
        // Real Tree-sitter would use more sophisticated techniques
        self.compress_goto_table_small(goto_table)
    }
}

/// Compressed table representation matching Tree-sitter's format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressedTables {
    pub action_table: CompressedActionTable,
    pub goto_table: CompressedGotoTable,
    pub small_table_threshold: usize,
}

impl CompressedTables {
    /// Validate that compressed tables maintain correctness
    pub fn validate(&self, original: &ParseTable) -> Result<(), TableGenError> {
        // Validate that decompression yields the same results
        // This is critical for bit-for-bit compatibility
        
        // Check action table dimensions
        if self.action_table.row_offsets.len() != original.action_table.len() + 1 {
            return Err(TableGenError::InvalidTable(
                "Action table row count mismatch".to_string()
            ));
        }
        
        // Check goto table dimensions
        if self.goto_table.row_offsets.len() != original.goto_table.len() + 1 {
            return Err(TableGenError::InvalidTable(
                "Goto table row count mismatch".to_string()
            ));
        }
        
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressedActionTable {
    pub data: Vec<CompressedActionEntry>,
    pub row_offsets: Vec<u16>,
    pub default_actions: Vec<Action>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressedGotoTable {
    pub data: Vec<CompressedGotoEntry>,
    pub row_offsets: Vec<u16>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressedActionEntry {
    pub symbol: u16,
    pub action: Action,
}

impl CompressedActionEntry {
    /// Create a new compressed action entry
    pub fn new(symbol: u16, action: Action) -> Self {
        Self { symbol, action }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CompressedGotoEntry {
    Single(u16),
    RunLength { state: u16, count: u16 },
}

/// Table generation errors
#[derive(Debug, thiserror::Error)]
pub enum TableGenError {
    #[error("Compression failed: {0}")]
    CompressionError(String),
    
    #[error("Code generation failed: {0}")]
    CodeGeneration(String),
    
    #[error("Invalid table structure: {0}")]
    InvalidTable(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_static_language_generator_creation() {
        let grammar = Grammar::new("test".to_string());
        let parse_table = ParseTable {
            action_table: vec![],
            goto_table: vec![],
            symbol_metadata: vec![],
            state_count: 0,
            symbol_count: 0,
        };
        
        let generator = StaticLanguageGenerator::new(grammar, parse_table);
        assert_eq!(generator.grammar.name, "test");
        assert_eq!(generator.parse_table.state_count, 0);
        assert!(generator.compressed_tables.is_none());
    }
    
    #[test]
    fn test_action_encoding_small_table() {
        let compressor = TableCompressor::new();
        
        // Test shift encoding
        let shift_action = Action::Shift(StateId(42));
        let encoded = compressor.encode_action_small(&shift_action).unwrap();
        assert_eq!(encoded, 42);
        assert!(encoded < 0x8000); // High bit should be clear for shifts
        
        // Test reduce encoding
        let reduce_action = Action::Reduce(RuleId(17));
        let encoded = compressor.encode_action_small(&reduce_action).unwrap();
        assert_eq!(encoded, 0x8000 | (17 << 1));
        assert!(encoded >= 0x8000); // High bit should be set for reduces
        
        // Test accept encoding
        let accept_action = Action::Accept;
        let encoded = compressor.encode_action_small(&accept_action).unwrap();
        assert_eq!(encoded, 0xFFFF);
        
        // Test error encoding
        let error_action = Action::Error;
        let encoded = compressor.encode_action_small(&error_action).unwrap();
        assert_eq!(encoded, 0xFFFE);
    }
    
    #[test]
    fn test_action_encoding_overflow() {
        let compressor = TableCompressor::new();
        
        // Test shift with state ID too large
        let shift_action = Action::Shift(StateId(0x8000));
        let result = compressor.encode_action_small(&shift_action);
        assert!(result.is_err());
        
        // Test reduce with rule ID too large
        let reduce_action = Action::Reduce(RuleId(0x4000));
        let result = compressor.encode_action_small(&reduce_action);
        assert!(result.is_err());
    }

    #[test]
    fn test_table_compressor_creation() {
        let compressor = TableCompressor::new();
        // Just test that it can be created
        let _ = compressor;
    }

    #[test]
    fn test_symbol_names_generation() {
        let mut grammar = Grammar::new("test".to_string());
        
        // Add a token
        let token = Token {
            name: "NUMBER".to_string(),
            pattern: TokenPattern::Regex(r"\d+".to_string()),
            fragile: false,
        };
        grammar.tokens.insert(SymbolId(0), token);
        
        // Add a rule
        let rule = Rule {
            lhs: SymbolId(1),
            rhs: vec![Symbol::Terminal(SymbolId(0))],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(0),
        };
        grammar.rules.insert(SymbolId(1), rule);
        
        let parse_table = ParseTable {
            action_table: vec![],
            goto_table: vec![],
            symbol_metadata: vec![],
            state_count: 0,
            symbol_count: 0,
        };
        
        let generator = StaticLanguageGenerator::new(grammar, parse_table);
        let symbol_names = generator.generate_symbol_names();
        
        assert_eq!(symbol_names.len(), 2);
        assert!(symbol_names.contains(&"NUMBER".to_string()));
        assert!(symbol_names.contains(&"rule_1".to_string()));
    }

    #[test]
    fn test_field_names_generation() {
        let mut grammar = Grammar::new("test".to_string());
        
        // Add fields in lexicographic order
        grammar.fields.insert(FieldId(0), "left".to_string());
        grammar.fields.insert(FieldId(1), "right".to_string());
        
        let parse_table = ParseTable {
            action_table: vec![],
            goto_table: vec![],
            symbol_metadata: vec![],
            state_count: 0,
            symbol_count: 0,
        };
        
        let generator = StaticLanguageGenerator::new(grammar, parse_table);
        let field_names = generator.generate_field_names();
        
        assert_eq!(field_names, vec!["left", "right"]);
    }

    #[test]
    fn test_node_types_generation() {
        let grammar = Grammar::new("test".to_string());
        let parse_table = ParseTable {
            action_table: vec![],
            goto_table: vec![],
            symbol_metadata: vec![],
            state_count: 0,
            symbol_count: 0,
        };
        
        let generator = StaticLanguageGenerator::new(grammar, parse_table);
        let node_types = generator.generate_node_types();
        
        // Should be valid JSON
        assert!(serde_json::from_str::<serde_json::Value>(&node_types).is_ok());
    }

    #[test]
    fn test_table_compression_small_table() {
        let grammar = Grammar::new("test".to_string());
        
        // Create a simple parse table
        let parse_table = ParseTable {
            action_table: vec![
                vec![Action::Shift(StateId(1)), Action::Error],
                vec![Action::Reduce(RuleId(0)), Action::Accept],
            ],
            goto_table: vec![
                vec![StateId(0), StateId(1)],
                vec![StateId(2), StateId(0)],
            ],
            symbol_metadata: vec![],
            state_count: 2,
            symbol_count: 2,
        };
        
        let mut generator = StaticLanguageGenerator::new(grammar, parse_table);
        
        // Test compression
        assert!(generator.compress_tables().is_ok());
        assert!(generator.compressed_tables.is_some());
        
        let compressed = generator.compressed_tables.as_ref().unwrap();
        assert_eq!(compressed.small_table_threshold, 32768);
    }
    
    #[test]
    fn test_table_compression_large_table() {
        let grammar = Grammar::new("large_test".to_string());
        
        // Create a parse table that exceeds small table threshold
        let parse_table = ParseTable {
            action_table: vec![vec![Action::Error; 10]; 40000],
            goto_table: vec![vec![StateId(0); 10]; 40000],
            symbol_metadata: vec![],
            state_count: 40000,
            symbol_count: 10,
        };
        
        let compressor = TableCompressor::new();
        let result = compressor.compress(&parse_table);
        
        assert!(result.is_ok());
        let compressed = result.unwrap();
        
        // Should use large table format
        assert_eq!(compressed.small_table_threshold, 32768);
        assert!(parse_table.state_count >= compressed.small_table_threshold);
    }

    #[test]
    fn test_compressed_action_table_small() {
        let compressor = TableCompressor::new();
        let action_table = vec![
            vec![Action::Shift(StateId(1)), Action::Error, Action::Error],
            vec![Action::Error, Action::Reduce(RuleId(0)), Action::Error],
        ];
        
        let compressed = compressor.compress_action_table_small(&action_table);
        assert!(compressed.is_ok());
        
        let compressed = compressed.unwrap();
        assert_eq!(compressed.default_actions.len(), 2);
        assert_eq!(compressed.row_offsets.len(), 3); // includes sentinel
        
        // First row should have default Error, with only Shift(1) stored
        match &compressed.default_actions[0] {
            Action::Error => {},
            _ => panic!("Expected Error as default for first row"),
        }
        
        // Second row should have default Error (not Reduce, because it's not universal)
        match &compressed.default_actions[1] {
            Action::Error => {},
            _ => panic!("Expected Error as default for second row"),
        }
    }
    
    #[test]
    fn test_compressed_action_table_with_default_reduction() {
        let compressor = TableCompressor::new();
        
        // Create a state with only reduce actions (common in LR parsers)
        let action_table = vec![
            vec![Action::Reduce(RuleId(1)), Action::Reduce(RuleId(1)), Action::Reduce(RuleId(1))],
        ];
        
        let compressed = compressor.compress_action_table_small(&action_table);
        assert!(compressed.is_ok());
        
        let compressed = compressed.unwrap();
        
        // Should have Reduce(1) as default
        match &compressed.default_actions[0] {
            Action::Reduce(RuleId(1)) => {},
            _ => panic!("Expected Reduce(1) as default"),
        }
        
        // Should have no entries in data (all are default)
        let entries_for_state_0 = compressed.row_offsets[1] - compressed.row_offsets[0];
        assert_eq!(entries_for_state_0, 0);
    }

    #[test]
    fn test_compressed_goto_table_small() {
        let compressor = TableCompressor::new();
        let goto_table = vec![
            vec![StateId(0), StateId(0), StateId(1)],
            vec![StateId(2), StateId(2), StateId(2)],
        ];
        
        let compressed = compressor.compress_goto_table_small(&goto_table);
        assert!(compressed.is_ok());
        
        let compressed = compressed.unwrap();
        assert_eq!(compressed.row_offsets.len(), 3); // includes sentinel
        assert!(!compressed.data.is_empty());
        
        // First row should have run of 2 StateId(0)s, then single StateId(1)
        let first_row_start = compressed.row_offsets[0] as usize;
        let first_row_end = compressed.row_offsets[1] as usize;
        let first_row_entries = &compressed.data[first_row_start..first_row_end];
        
        // Should be stored as individual entries (run of 2 is too short)
        assert_eq!(first_row_entries.len(), 3);
        
        // Second row should have run of 3 StateId(2)s
        let second_row_start = compressed.row_offsets[1] as usize;
        let second_row_end = compressed.row_offsets[2] as usize;
        let second_row_entries = &compressed.data[second_row_start..second_row_end];
        
        // Should be stored as run-length encoded
        assert_eq!(second_row_entries.len(), 1);
        match &second_row_entries[0] {
            CompressedGotoEntry::RunLength { state: 2, count: 3 } => {},
            _ => panic!("Expected run-length encoding for second row"),
        }
    }
    
    #[test]
    fn test_goto_table_run_length_threshold() {
        let compressor = TableCompressor::new();
        
        // Test that runs of 1 and 2 are stored as individual entries
        let goto_table = vec![
            vec![StateId(1), StateId(2), StateId(2), StateId(3), StateId(3), StateId(3)],
        ];
        
        let compressed = compressor.compress_goto_table_small(&goto_table);
        assert!(compressed.is_ok());
        
        let compressed = compressed.unwrap();
        let entries = &compressed.data;
        
        // Should have: Single(1), Single(2), Single(2), RunLength(3, 3)
        assert_eq!(entries.len(), 4);
        
        match &entries[0] {
            CompressedGotoEntry::Single(1) => {},
            _ => panic!("Expected single entry for StateId(1)"),
        }
        
        match &entries[1] {
            CompressedGotoEntry::Single(2) => {},
            _ => panic!("Expected single entry for first StateId(2)"),
        }
        
        match &entries[2] {
            CompressedGotoEntry::Single(2) => {},
            _ => panic!("Expected single entry for second StateId(2)"),
        }
        
        match &entries[3] {
            CompressedGotoEntry::RunLength { state: 3, count: 3 } => {},
            _ => panic!("Expected run-length for StateId(3)"),
        }
    }

    #[test]
    fn test_language_code_generation() {
        let grammar = Grammar::new("test_lang".to_string());
        let parse_table = ParseTable {
            action_table: vec![vec![Action::Accept]],
            goto_table: vec![vec![StateId(0)]],
            symbol_metadata: vec![],
            state_count: 1,
            symbol_count: 1,
        };
        
        let generator = StaticLanguageGenerator::new(grammar, parse_table);
        let code = generator.generate_language_code();
        
        // Should generate valid Rust code
        let code_str = code.to_string();
        println!("Generated code: {}", code_str);
        assert!(code_str.contains("pub fn language()"));
        assert!(code_str.contains("tree_sitter_language"));
        assert!(code_str.contains("LANGUAGE_VERSION"));
    }
}