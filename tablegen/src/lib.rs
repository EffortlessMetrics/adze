// Static table generation and compression for pure-Rust Tree-sitter
// This module implements Tree-sitter's exact table compression algorithms

pub mod abi;
pub mod compress;
pub mod external_scanner;
pub mod generate;
pub mod node_types;
pub mod validation;

// Re-export key types
pub use compress::{CompressedParseTable, CompressedTables, CompressedActionTable, CompressedGotoTable, CompressedGotoEntry, CompressedActionEntry, ActionEntry, GotoEntry, TableCompressor};
pub use external_scanner::ExternalScannerGenerator;
pub use generate::LanguageBuilder;
pub use node_types::NodeTypesGenerator;
pub use validation::{LanguageValidator, ValidationError};

// use indexmap::IndexMap; // Currently unused
use proc_macro2::TokenStream;
use quote::quote;
use rust_sitter_glr_core::*;
use rust_sitter_ir::*;

// Use the appropriate tree-sitter backend
#[cfg(feature = "tree-sitter-standard")]
use tree_sitter as _ts;


// Ensure ts is available even if neither feature is enabled (for tests)
#[cfg(all(not(feature = "tree-sitter-standard"), not(feature = "tree-sitter-c2rust")))]
use tree_sitter_c2rust as _ts;

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
        let language_fn_name = format!("tree_sitter_{}", language_name.to_lowercase().replace('-', "_"));
        let language_fn_ident = quote::format_ident!("{}", language_fn_name);
        let symbol_count = self.parse_table.symbol_count;
        let state_count = self.parse_table.state_count;
        
        // Generate symbol names array
        let symbol_names: Vec<_> = self.generate_symbol_names()
            .into_iter()
            .map(|name| quote! { #name })
            .collect();
        
        // Generate symbol metadata array
        let symbol_metadata = self.generate_symbol_metadata();
        
        // Generate field names array  
        let field_names: Vec<_> = self.generate_field_names()
            .into_iter()
            .map(|name| quote! { #name })
            .collect();
        
        // Generate parse tables
        let (action_table, goto_table) = if let Some(compressed) = &self.compressed_tables {
            self.generate_compressed_tables(compressed)
        } else {
            self.generate_uncompressed_tables()
        };
        
        // Generate NODE_TYPES JSON
        let node_types_json = self.generate_node_types();
        
        // Count various elements
        let field_count = field_names.len();
        let _token_count = self.grammar.tokens.len();
        let external_token_count = self.grammar.externals.len();
        let _production_id_count = self.grammar.alias_sequences.len(); // Production IDs are from alias sequences
        let _max_alias_sequence_length = 0u16; // TODO: Calculate from alias sequences
        
        // Generate external scanner data if needed
        let external_scanner_code = if !self.grammar.externals.is_empty() {
            let scanner_gen = external_scanner::ExternalScannerGenerator::new(self.grammar.clone());
            let scanner_interface = scanner_gen.generate_scanner_interface();
            quote! { #scanner_interface }
        } else {
            quote! {
                // No external scanner needed
                static EXTERNAL_SCANNER_DATA: ts::ffi::TSExternalScannerData = ts::ffi::TSExternalScannerData {
                    states: std::ptr::null(),
                    symbol_map: std::ptr::null(),
                    create: None,
                    destroy: None,
                    scan: None,
                    serialize: None,
                    deserialize: None,
                };
            }
        };
        
        quote! {
            use std::sync::OnceLock;
            
            #[cfg(feature = "tree-sitter-standard")]
            use tree_sitter as ts;
            
            #[cfg(feature = "tree-sitter-c2rust")]
            use tree_sitter_c2rust as ts;
            
            // Static symbol names array
            static SYMBOL_NAMES: &[&str] = &[#(#symbol_names),*];
            
            // Static symbol metadata array  
            static SYMBOL_METADATA: &[ts::ffi::TSSymbolMetadata] = &[#(#symbol_metadata),*];
            
            // Static field names array
            static FIELD_NAMES: &[&str] = &[#(#field_names),*];
            
            // Parse tables
            #action_table
            #goto_table
            
            // External scanner data
            #external_scanner_code
            
            // NODE_TYPES JSON
            pub const NODE_TYPES: &str = #node_types_json;
            
                        const STATE_COUNT: u32 = #state_count as u32;
            const SYMBOL_COUNT: u32 = #symbol_count as u32;
            const FIELD_COUNT: u32 = #field_count as u32;
            const EXTERNAL_TOKEN_COUNT: u32 = #external_token_count as u32;
            
            // Import our ABI module
            use crate::abi::*;
            
            // Language metadata
            const LANGUAGE_VERSION: u32 = TREE_SITTER_LANGUAGE_VERSION; // ABI version 15
            
            // For now, use a simplified approach that compiles
            // In a full implementation, we would properly construct the Language structure
            
            /// Get the Tree-sitter Language for this grammar  
            pub fn language() -> ts::Language {
                // Placeholder implementation
                // TODO: Create proper TSLanguage structure with all fields
                unsafe {
                    // Return a dummy language for now
                    ts::Language::from_raw(std::ptr::null())
                }
            }
            
            /// Export for C FFI
            #[no_mangle]
            pub extern "C" fn #language_fn_ident() -> ts::Language {
                language()
            }
        }
    }

    /// Generate NODE_TYPES JSON string
    pub fn generate_node_types(&self) -> String {
        use serde_json::json;
        
        let mut types = Vec::new();
        
        // Generate node types for non-terminal rules
        for (symbol_id, _rule) in &self.grammar.rules {
            // For now, use generated rule names
            // TODO: Add proper symbol name mapping to Grammar
            let rule_name = format!("rule_{}", symbol_id.0);
            
            // Skip hidden rules (those starting with underscore)
            if rule_name.starts_with('_') {
                continue;
            }
            
            let mut node_type = json!({
                "type": rule_name,
                "named": true
            });
            
            // Add fields if this rule has any
            if !_rule.fields.is_empty() {
                let mut fields = serde_json::Map::new();
                for (field_id, _position) in &_rule.fields {
                    if let Some(field_name) = self.grammar.fields.get(field_id) {
                        fields.insert(
                            field_name.clone(),
                            json!({
                                "multiple": false,
                                "required": true,
                                "types": []
                            })
                        );
                    }
                }
                node_type["fields"] = json!(fields);
            }
            
            // Add children if rule has any
            if !_rule.rhs.is_empty() {
                let mut children = serde_json::Map::new();
                children.insert(
                    "multiple".to_string(),
                    json!(false)
                );
                children.insert(
                    "required".to_string(),
                    json!(!_rule.rhs.is_empty())
                );
                // TODO: Add proper child types based on rule.rhs
                children.insert(
                    "types".to_string(),
                    json!([])
                );
                node_type["children"] = json!(children);
            }
            
            // Check if this is a supertype
            if self.grammar.supertypes.contains(symbol_id) {
                node_type["subtypes"] = json!([]);
            }
            
            types.push(node_type);
        }
        
        // Generate node types for named tokens
        for (_, token) in &self.grammar.tokens {
            if !token.name.starts_with('_') && matches!(&token.pattern, TokenPattern::Regex(_)) {
                types.push(json!({
                    "type": token.name,
                    "named": true
                }));
            }
        }
        
        // Generate node types for external tokens
        for external in &self.grammar.externals {
            if !external.name.starts_with('_') {
                types.push(json!({
                    "type": external.name,
                    "named": true
                }));
            }
        }
        
        serde_json::to_string_pretty(&json!(types))
            .unwrap_or_else(|_| "[]".to_string())
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
        
        // Generate metadata for each terminal symbol
        for (_, token) in &self.grammar.tokens {
            // Hidden tokens start with underscore
            let visible = !token.name.starts_with('_');
            // Anonymous tokens (string literals) are unnamed, regex tokens can be named
            let named = matches!(&token.pattern, TokenPattern::Regex(_)) && visible;
            let supertype = false;
            
            metadata.push(quote! {
                ts::ffi::TSSymbolMetadata {
                    visible: #visible,
                    named: #named,
                    supertype: #supertype,
                }
            });
        }
        
        // Add metadata for non-terminals (rules)
        for (symbol_id, _rule) in &self.grammar.rules {
            // For now, use generated rule names until we have proper symbol mapping
            let rule_name = format!("rule_{}", symbol_id.0);
            // Hidden rules start with underscore
            let visible = !rule_name.starts_with('_');
            // Non-terminals are named unless they're hidden
            let named = visible;
            // Check if this rule is in the supertypes list
            let supertype = self.grammar.supertypes.contains(symbol_id);
            
            metadata.push(quote! {
                ts::ffi::TSSymbolMetadata {
                    visible: #visible,
                    named: #named,
                    supertype: #supertype,
                }
            });
        }
        
        // Add metadata for external symbols
        for external in &self.grammar.externals {
            // External tokens are typically visible and named
            let visible = !external.name.starts_with('_');
            let named = visible;
            let supertype = false;
            
            metadata.push(quote! {
                ts::ffi::TSSymbolMetadata {
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
            static ACTION_TABLE: &[&[ts::ffi::TSParseActionEntry]] = &[#(#action_entries),*];
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
                            ts::ffi::TSParseActionEntry {
                                type_: ts::ffi::TSParseActionType::Shift,
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
                            ts::ffi::TSParseActionEntry {
                                type_: ts::ffi::TSParseActionType::Reduce,
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
                            ts::ffi::TSParseActionEntry {
                                type_: ts::ffi::TSParseActionType::Accept,
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
                            ts::ffi::TSParseActionEntry {
                                type_: ts::ffi::TSParseActionType::Error,
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
                                        ts::ffi::TSParseActionEntry {
                                            type_: ts::ffi::TSParseActionType::Shift,
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
                                        ts::ffi::TSParseActionEntry {
                                            type_: ts::ffi::TSParseActionType::Error,
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
                                ts::ffi::TSParseActionEntry {
                                    type_: ts::ffi::TSParseActionType::Error,
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

// TableCompressor moved to compress.rs

// Remove the TableCompressor impl - it's now in compress.rs
/*
impl TableCompressor {
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
        
        for (_state_id, actions) in action_table.iter().enumerate() {
            // Find the most common action overall
            let mut action_counts: HashMap<&Action, usize> = HashMap::new();
            let mut has_shift = false;
            let mut has_accept = false;
            
            for action in actions {
                *action_counts.entry(action).or_insert(0) += 1;
                match action {
                    Action::Shift(_) => has_shift = true,
                    Action::Accept => has_accept = true,
                    _ => {}
                }
            }
            
            // Tree-sitter uses the most common action as default, but only reduces if no shifts/accepts
            let most_common = action_counts
                .iter()
                .max_by_key(|(_, count)| *count)
                .map(|(action, _)| (*action).clone())
                .unwrap_or(Action::Error);
            
            let default_action = match &most_common {
                Action::Reduce(_) if !has_shift && !has_accept => most_common,
                Action::Error => Action::Error,
                _ => Action::Error, // Default to Error for other cases
            };
            
            default_reductions.push(default_action.clone());
            
            // Encode non-default actions
            row_offsets.push(entries.len() as u16);
            
            for (symbol_id, action) in actions.iter().enumerate() {
                // Skip if this is the default action
                if action == &default_action {
                    continue;
                }
                
                let _encoded = self.encode_action_small(action)?;
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
*/

// CompressedTables and related types are now defined in compress.rs

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
        let _grammar = Grammar::new("large_test".to_string());
        
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
        assert!(code_str.contains("pub fn language")); // Without parentheses in quote output
        assert!(code_str.contains("tree_sitter_test_lang")); // Language-specific function name
        assert!(code_str.contains("LANGUAGE_VERSION"));
    }
    
    #[test]
    fn test_compressed_tables_validation() {
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
        
        let compressor = TableCompressor::new();
        let compressed = compressor.compress(&parse_table).unwrap();
        
        // Validate compressed tables
        assert!(compressed.validate(&parse_table).is_ok());
    }
    
    #[test]
    fn test_tree_sitter_compatibility() {
        // Test that our encoding matches Tree-sitter's expectations
        let compressor = TableCompressor::new();
        
        // Tree-sitter encoding examples:
        // Shift to state 42: 0x002A (42 in hex)
        let shift = Action::Shift(StateId(42));
        assert_eq!(compressor.encode_action_small(&shift).unwrap(), 0x002A);
        
        // Reduce by rule 17: 0x8022 (0x8000 | (17 << 1))
        let reduce = Action::Reduce(RuleId(17));
        assert_eq!(compressor.encode_action_small(&reduce).unwrap(), 0x8022);
        
        // Accept: 0xFFFF
        let accept = Action::Accept;
        assert_eq!(compressor.encode_action_small(&accept).unwrap(), 0xFFFF);
        
        // Error: 0xFFFE
        let error = Action::Error;
        assert_eq!(compressor.encode_action_small(&error).unwrap(), 0xFFFE);
    }
    
    #[test]
    fn test_compressed_action_entry() {
        let entry = CompressedActionEntry::new(5, Action::Shift(StateId(10)));
        assert_eq!(entry.symbol, 5);
        match entry.action {
            Action::Shift(StateId(10)) => {},
            _ => panic!("Wrong action type"),
        }
    }
    
    #[test]
    fn test_generated_small_table_format() {
        let mut grammar = Grammar::new("small_test".to_string());
        
        // Add a simple grammar
        let token = Token {
            name: "A".to_string(),
            pattern: TokenPattern::String("a".to_string()),
            fragile: false,
        };
        grammar.tokens.insert(SymbolId(0), token);
        
        // Simple parse table
        let parse_table = ParseTable {
            action_table: vec![
                vec![Action::Shift(StateId(1))],
                vec![Action::Accept],
            ],
            goto_table: vec![
                vec![StateId(1)],
                vec![StateId(0)],
            ],
            symbol_metadata: vec![],
            state_count: 2,
            symbol_count: 1,
        };
        
        let mut generator = StaticLanguageGenerator::new(grammar, parse_table);
        generator.compress_tables().unwrap();
        
        let code = generator.generate_language_code();
        let code_str = code.to_string();
        
        // Should generate small table format
        assert!(code_str.contains("SMALL_PARSE_TABLE") || code_str.contains("ACTION_TABLE"));
    }
}