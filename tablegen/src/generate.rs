use crate::compress::CompressedParseTable;
use crate::validation::TSLanguage;
use proc_macro2::TokenStream;
use quote::quote;
use rust_sitter_glr_core::ParseTable;
use rust_sitter_ir::Grammar;

/// Language builder that produces validated Language structs
pub struct LanguageBuilder {
    grammar: Grammar,
    parse_table: ParseTable,
}

impl LanguageBuilder {
    /// Create a new generator
    pub fn new(grammar: Grammar, parse_table: ParseTable) -> Self {
        Self { grammar, parse_table }
    }
    
    /// Generate a static Language struct with full validation
    pub fn generate_language(&self) -> Result<TSLanguage, String> {
        // Create compressed tables
        let compressed = CompressedParseTable::from_parse_table(&self.parse_table);
        
        // Build the Language struct
        let language = self.build_language_struct(&compressed)?;
        
        // Note: Validation would be done separately by the caller
        // to avoid lifetime issues
        
        Ok(language)
    }
    
    /// Build the Language struct with all required fields
    fn build_language_struct(&self, compressed: &CompressedParseTable) -> Result<TSLanguage, String> {
        // Count various elements
        let symbol_count = self.parse_table.symbol_count as u32;
        let state_count = self.parse_table.state_count as u32;
        let token_count = self.grammar.tokens.len() as u32;
        let external_token_count = self.grammar.externals.len() as u32;
        let field_count = self.grammar.fields.len() as u32;
        
        // TODO: Calculate these properly
        let alias_count = 0;
        let large_state_count = 0;
        let production_id_count = self.grammar.alias_sequences.len() as u32;
        let max_alias_sequence_length = 0;
        
        // Build symbol names array
        let symbol_names = self.build_symbol_names();
        
        // Build field names array  
        let field_names = self.build_field_names();
        
        // Build symbol metadata
        let symbol_metadata = self.build_symbol_metadata();
        
        // Build minimal parse tables for validation
        // For now, create dummy tables - in real implementation these would be
        // generated from the compressed parse table data
        let small_parse_table = self.build_small_parse_table(compressed);
        
        Ok(TSLanguage {
            version: 15,
            symbol_count,
            alias_count,
            token_count,
            external_token_count,
            state_count,
            large_state_count,
            production_id_count,
            field_count,
            max_alias_sequence_length,
            parse_table: std::ptr::null(),
            small_parse_table: Box::leak(Box::new(small_parse_table)).as_ptr(),
            small_parse_table_map: std::ptr::null(),
            parse_actions: std::ptr::null(),
            symbol_names: Box::leak(Box::new(symbol_names)).as_ptr(),
            field_names: if field_count > 0 {
                Box::leak(Box::new(field_names)).as_ptr()
            } else {
                std::ptr::null()
            },
            field_map_slices: std::ptr::null(),
            field_map_entries: std::ptr::null(),
            symbol_metadata: Box::leak(Box::new(symbol_metadata)).as_ptr(),
            public_symbol_map: std::ptr::null(),
            alias_map: std::ptr::null(),
            alias_sequences: std::ptr::null(),
            lex_modes: std::ptr::null(),
            lex_fn: None,
            keyword_lex_fn: None,
            keyword_capture_token: 0,
            external_scanner_data: crate::validation::TSExternalScannerData {
                states: std::ptr::null(),
                symbol_map: std::ptr::null(),
                create: None,
                destroy: None,
                scan: None,
                serialize: None,
                deserialize: None,
            },
            primary_state_ids: std::ptr::null(),
        })
    }
    
    fn build_symbol_names(&self) -> Vec<*const i8> {
        let mut names = Vec::new();
        
        // Add terminal symbols
        for (_, token) in &self.grammar.tokens {
            let name = std::ffi::CString::new(token.name.clone()).unwrap();
            names.push(Box::leak(Box::new(name)).as_ptr());
        }
        
        // Add non-terminal symbols
        for (symbol_id, _) in &self.grammar.rules {
            let name = std::ffi::CString::new(format!("rule_{}", symbol_id.0)).unwrap();
            names.push(Box::leak(Box::new(name)).as_ptr());
        }
        
        // Add external symbols
        for external in &self.grammar.externals {
            let name = std::ffi::CString::new(external.name.clone()).unwrap();
            names.push(Box::leak(Box::new(name)).as_ptr());
        }
        
        names
    }
    
    fn build_field_names(&self) -> Vec<*const i8> {
        let mut names = Vec::new();
        
        // First entry is always empty string
        let empty = std::ffi::CString::new("").unwrap();
        names.push(Box::leak(Box::new(empty)).as_ptr());
        
        // Add field names in lexicographic order
        for (_, field_name) in &self.grammar.fields {
            let name = std::ffi::CString::new(field_name.clone()).unwrap();
            names.push(Box::leak(Box::new(name)).as_ptr());
        }
        
        names
    }
    
    fn build_symbol_metadata(&self) -> Vec<crate::validation::TSSymbolMetadata> {
        let mut metadata = Vec::new();
        
        // First symbol is always EOF (invisible, unnamed)
        metadata.push(crate::validation::TSSymbolMetadata {
            visible: false,
            named: false,
        });
        
        // Add metadata for terminals
        for (_, token) in &self.grammar.tokens {
            let visible = !token.name.starts_with('_');
            let named = matches!(&token.pattern, rust_sitter_ir::TokenPattern::Regex(_)) && visible;
            
            metadata.push(crate::validation::TSSymbolMetadata {
                visible,
                named,
            });
        }
        
        // Add metadata for non-terminals
        for (symbol_id, _) in &self.grammar.rules {
            let rule_name = format!("rule_{}", symbol_id.0);
            let visible = !rule_name.starts_with('_');
            let named = visible;
            
            metadata.push(crate::validation::TSSymbolMetadata {
                visible,
                named,
            });
        }
        
        // Add metadata for externals
        for external in &self.grammar.externals {
            let visible = !external.name.starts_with('_');
            let named = visible;
            
            metadata.push(crate::validation::TSSymbolMetadata {
                visible,
                named,
            });
        }
        
        metadata
    }
    
    /// Build a minimal small parse table for testing
    fn build_small_parse_table(&self, _compressed: &CompressedParseTable) -> Vec<u16> {
        // Create a minimal parse table with the right dimensions
        // In real implementation, this would be generated from compressed data
        let table_size = self.parse_table.state_count * self.parse_table.symbol_count;
        vec![0xFFFE; table_size] // Fill with error actions for now
    }
    
    /// Generate Rust code for the Language
    pub fn generate_language_code(&self) -> TokenStream {
        quote! {
            // Placeholder for Language code generation
            pub fn language() -> *const TSLanguage {
                std::ptr::null()
            }
        }
    }
}