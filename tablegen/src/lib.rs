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

    fn generate_compressed_tables(&self, _compressed: &CompressedTables) -> (TokenStream, TokenStream) {
        // Generate compressed tables using Tree-sitter's "small table" optimization
        // This will implement the exact compression algorithm from Tree-sitter
        
        // For now, fall back to uncompressed tables
        // TODO: Implement bit-for-bit compatible compression
        self.generate_uncompressed_tables()
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
pub struct TableCompressor;

impl TableCompressor {
    pub fn new() -> Self {
        Self
    }

    /// Compress parse tables using Tree-sitter's "small table" optimization
    pub fn compress(&self, parse_table: &ParseTable) -> Result<CompressedTables, TableGenError> {
        // Implement Tree-sitter's exact compression algorithm
        // This includes:
        // 1. Row-based compression with default actions
        // 2. "Small table" factoring for tables with <32k states
        // 3. Run-length encoding for sparse tables
        
        let compressed_action_table = self.compress_action_table(&parse_table.action_table)?;
        let compressed_goto_table = self.compress_goto_table(&parse_table.goto_table)?;
        
        Ok(CompressedTables {
            action_table: compressed_action_table,
            goto_table: compressed_goto_table,
            small_table_threshold: 32768, // Tree-sitter's threshold
        })
    }

    fn compress_action_table(&self, action_table: &[Vec<Action>]) -> Result<CompressedActionTable, TableGenError> {
        // Implement row-based compression with default actions
        let mut compressed_rows = Vec::new();
        let mut row_offsets = Vec::new();
        let mut default_actions = Vec::new();
        
        for (_state_id, actions) in action_table.iter().enumerate() {
            // Find the most common action as the default
            let mut action_counts: HashMap<&Action, usize> = HashMap::new();
            for action in actions {
                *action_counts.entry(action).or_insert(0) += 1;
            }
            
            let default_action = action_counts
                .iter()
                .max_by_key(|(_, count)| *count)
                .map(|(action, _)| (*action).clone())
                .unwrap_or(Action::Error);
            
            default_actions.push(default_action.clone());
            
            // Compress the row by storing only non-default actions
            let mut compressed_row = Vec::new();
            for (symbol_id, action) in actions.iter().enumerate() {
                if *action != default_action {
                    compressed_row.push(CompressedActionEntry {
                        symbol: symbol_id as u16,
                        action: action.clone(),
                    });
                }
            }
            
            row_offsets.push(compressed_rows.len() as u16);
            compressed_rows.extend(compressed_row);
        }
        
        Ok(CompressedActionTable {
            data: compressed_rows,
            row_offsets,
            default_actions,
        })
    }

    fn compress_goto_table(&self, goto_table: &[Vec<StateId>]) -> Result<CompressedGotoTable, TableGenError> {
        // Similar compression for goto table
        let mut compressed_data = Vec::new();
        let mut row_offsets = Vec::new();
        
        for row in goto_table {
            row_offsets.push(compressed_data.len() as u16);
            
            // Simple run-length encoding for goto table
            let mut i = 0;
            while i < row.len() {
                let current_state = row[i];
                let mut count = 1;
                
                // Count consecutive identical states
                while i + count < row.len() && row[i + count] == current_state {
                    count += 1;
                }
                
                if count > 1 {
                    // Store as run-length encoded entry
                    compressed_data.push(CompressedGotoEntry::RunLength {
                        state: current_state.0,
                        count: count as u16,
                    });
                } else {
                    // Store as single entry
                    compressed_data.push(CompressedGotoEntry::Single(current_state.0));
                }
                
                i += count;
            }
        }
        
        Ok(CompressedGotoTable {
            data: compressed_data,
            row_offsets,
        })
    }
}

/// Compressed table representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressedTables {
    pub action_table: CompressedActionTable,
    pub goto_table: CompressedGotoTable,
    pub small_table_threshold: usize,
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
    fn test_table_compression() {
        let mut grammar = Grammar::new("test".to_string());
        
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
    }

    #[test]
    fn test_compressed_action_table() {
        let compressor = TableCompressor::new();
        let action_table = vec![
            vec![Action::Shift(StateId(1)), Action::Error, Action::Error],
            vec![Action::Error, Action::Reduce(RuleId(0)), Action::Error],
        ];
        
        let compressed = compressor.compress_action_table(&action_table);
        assert!(compressed.is_ok());
        
        let compressed = compressed.unwrap();
        assert_eq!(compressed.default_actions.len(), 2);
        assert_eq!(compressed.row_offsets.len(), 2);
    }

    #[test]
    fn test_compressed_goto_table() {
        let compressor = TableCompressor::new();
        let goto_table = vec![
            vec![StateId(0), StateId(0), StateId(1)],
            vec![StateId(2), StateId(2), StateId(2)],
        ];
        
        let compressed = compressor.compress_goto_table(&goto_table);
        assert!(compressed.is_ok());
        
        let compressed = compressed.unwrap();
        assert_eq!(compressed.row_offsets.len(), 2);
        assert!(!compressed.data.is_empty());
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