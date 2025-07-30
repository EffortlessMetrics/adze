// ABI-compatible language builder for Tree-sitter
// This module generates static Language structures that match Tree-sitter's C ABI exactly

use crate::abi::*;
use crate::compress::CompressedTables;
use rust_sitter_ir::{Grammar, TokenPattern, Symbol, SymbolId};
use rust_sitter_glr_core::{ParseTable, Action};
use std::collections::HashSet;
use proc_macro2::TokenStream;
use quote::quote;

/// Builder for generating ABI-compatible language structures
pub struct AbiLanguageBuilder<'a> {
    grammar: &'a Grammar,
    parse_table: &'a ParseTable,
    compressed_tables: Option<&'a CompressedTables>,
}

impl<'a> AbiLanguageBuilder<'a> {
    pub fn new(grammar: &'a Grammar, parse_table: &'a ParseTable) -> Self {
        Self {
            grammar,
            parse_table,
            compressed_tables: None,
        }
    }
    
    pub fn with_compressed_tables(mut self, tables: &'a CompressedTables) -> Self {
        self.compressed_tables = Some(tables);
        self
    }
    
    /// Generate the complete language module
    pub fn generate(&self) -> TokenStream {
        let language_name = &self.grammar.name;
        let language_fn_ident = quote::format_ident!("tree_sitter_{}", language_name);
        
        // Generate all static data with deterministic ordering
        let (symbol_names, symbol_name_ptrs) = self.generate_symbol_names();
        let (field_names, field_name_ptrs) = self.generate_field_names();
        let symbol_metadata = self.generate_symbol_metadata();
        let (parse_table_data, small_parse_table_map) = self.generate_parse_tables();
        let parse_actions = self.generate_parse_actions();
        let lex_modes = self.generate_lex_modes();
        let (field_map_slices, field_map_entries) = self.generate_field_maps();
        let public_symbol_map = self.generate_public_symbol_map();
        let primary_state_ids = self.generate_primary_state_ids();
        let production_id_map = self.generate_production_id_map();
        
        // Count elements
        let counts = self.calculate_counts();
        let symbol_count = counts.symbol_count;
        let alias_count = counts.alias_count;
        let token_count = counts.token_count;
        let external_token_count = counts.external_token_count;
        let state_count = counts.state_count;
        let large_state_count = counts.large_state_count;
        let production_id_count = counts.production_id_count;
        let field_count = counts.field_count;
        let max_alias_sequence_length = counts.max_alias_sequence_length;
        
        // Generate field names array
        let field_names_array = if field_count == 0 {
            quote! {
                static FIELD_NAME_PTRS: [SyncPtr; 0] = [];
            }
        } else {
            quote! {
                const FIELD_NAME_PTRS_LEN: usize = #field_count as usize;
                static FIELD_NAME_PTRS: [SyncPtr; FIELD_NAME_PTRS_LEN] = [
                    #(#field_name_ptrs),*
                ];
            }
        };
        
        // Generate lexer function with symbol mapping
        let lexer_code = crate::lexer_gen::generate_lexer(self.grammar, &self.parse_table.symbol_to_index);
        
        quote! {
            use ::rust_sitter::pure_parser::*;
            
            // Lexer implementation
            #lexer_code
            
            // Symbol names (null-terminated strings)
            #(#symbol_names)*
            
            // Symbol name pointers array
            const SYMBOL_NAME_PTRS_LEN: usize = #symbol_count as usize;
            static SYMBOL_NAME_PTRS: [SyncPtr; SYMBOL_NAME_PTRS_LEN] = [
                #(#symbol_name_ptrs),*
            ];
            
            // Field names (null-terminated strings)
            #(#field_names)*
            
            // Field name pointers array - handle empty case specially
            #field_names_array
            
            // Symbol metadata (visibility, named, etc.)
            static SYMBOL_METADATA: &[u8] = &[#(#symbol_metadata),*];
            
            // Parse table (compressed)
            static PARSE_TABLE: &[u16] = &[#(#parse_table_data),*];
            
            // Small parse table map
            static SMALL_PARSE_TABLE_MAP: &[u32] = &[#(#small_parse_table_map),*];
            
            // Parse actions
            static PARSE_ACTIONS: &[TSParseAction] = &[#(#parse_actions),*];
            
            // Lex modes
            static LEX_MODES: &[TSLexState] = &[#(#lex_modes),*];
            
            // Field map slices
            static FIELD_MAP_SLICES: &[u16] = &[#(#field_map_slices),*];
            
            // Field map entries
            static FIELD_MAP_ENTRIES: &[u16] = &[#(#field_map_entries),*];
            
            // Public symbol map
            static PUBLIC_SYMBOL_MAP: &[u16] = &[#(#public_symbol_map),*];
            
            // Primary state IDs
            static PRIMARY_STATE_IDS: &[u16] = &[#(#primary_state_ids),*];
            
            // Production ID map (maps production IDs to rule IDs)
            static PRODUCTION_ID_MAP: &[u16] = &[#(#production_id_map),*];
            
            // The language structure
            pub static LANGUAGE: TSLanguage = TSLanguage {
                version: TREE_SITTER_LANGUAGE_VERSION,
                symbol_count: #symbol_count,
                alias_count: #alias_count,
                token_count: #token_count,
                external_token_count: #external_token_count,
                state_count: #state_count,
                large_state_count: #large_state_count,
                production_id_count: #production_id_count,
                field_count: #field_count,
                max_alias_sequence_length: #max_alias_sequence_length,
                production_id_map: PRODUCTION_ID_MAP.as_ptr(),
                parse_table: PARSE_TABLE.as_ptr(),
                small_parse_table: std::ptr::null(),
                small_parse_table_map: SMALL_PARSE_TABLE_MAP.as_ptr(),
                parse_actions: PARSE_ACTIONS.as_ptr(),
                symbol_names: SYMBOL_NAME_PTRS.as_ptr() as *const SyncPtr as *const *const u8,
                field_names: FIELD_NAME_PTRS.as_ptr() as *const SyncPtr as *const *const u8,
                field_map_slices: FIELD_MAP_SLICES.as_ptr(),
                field_map_entries: FIELD_MAP_ENTRIES.as_ptr(),
                symbol_metadata: SYMBOL_METADATA.as_ptr(),
                public_symbol_map: PUBLIC_SYMBOL_MAP.as_ptr(),
                alias_map: std::ptr::null(),
                alias_sequences: std::ptr::null::<u16>(),
                lex_modes: LEX_MODES.as_ptr(),
                lex_fn: Some(lexer_fn),
                keyword_lex_fn: None,
                keyword_capture_token: 0,
                external_scanner: ExternalScanner { 
                    states: std::ptr::null(),
                    symbol_map: std::ptr::null(),
                    create: None,
                    destroy: None,
                    scan: None,
                    serialize: None,
                    deserialize: None,
                },
                primary_state_ids: PRIMARY_STATE_IDS.as_ptr(),
            };
            
            /// Get the Tree-sitter Language for this grammar
            #[unsafe(no_mangle)]
            pub extern "C" fn #language_fn_ident() -> *const TSLanguage {
                &LANGUAGE as *const TSLanguage
            }
        }
    }
    
    /// Generate symbol names with deterministic ordering
    fn generate_symbol_names(&self) -> (Vec<TokenStream>, Vec<TokenStream>) {
        let mut names = Vec::new();
        let mut name_idents = Vec::new();
        
        // Use the parse table's symbol ordering
        // Create reverse mapping from index to symbol ID
        let mut index_to_symbol: Vec<Option<SymbolId>> = vec![None; self.parse_table.symbol_count];
        for (symbol_id, &index) in &self.parse_table.symbol_to_index {
            if index < self.parse_table.symbol_count {
                index_to_symbol[index] = Some(*symbol_id);
            }
        }
        
        // Generate names in parse table order
        for (idx, symbol_id_opt) in index_to_symbol.iter().enumerate() {
            let ident = quote::format_ident!("SYMBOL_NAME_{}", idx);
            
            let name_str = if let Some(symbol_id) = symbol_id_opt {
                if symbol_id.0 == 0 {
                    // EOF symbol
                    "end".to_string()
                } else if let Some(token) = self.grammar.tokens.get(symbol_id) {
                    // Terminal symbol
                    token.name.clone()
                } else if let Some(rule_name) = self.grammar.rule_names.get(symbol_id) {
                    // Non-terminal with explicit name
                    rule_name.clone()
                } else {
                    // Non-terminal without name - generate one
                    format!("rule_{}", symbol_id.0)
                }
            } else {
                // Should not happen
                format!("unknown_{}", idx)
            };
            
            let name_bytes = format!("{}\0", name_str).into_bytes();
            names.push(quote! {
                static #ident: &[u8] = &[#(#name_bytes),*];
            });
            name_idents.push(ident);
        }
        
        let ptrs = name_idents.iter().map(|ident| {
            quote! { SyncPtr::new(#ident.as_ptr()) }
        }).collect();
        
        (names, ptrs)
    }
    
    /// Generate field names with lexicographic ordering
    fn generate_field_names(&self) -> (Vec<TokenStream>, Vec<TokenStream>) {
        let mut names = Vec::new();
        let mut name_idents = Vec::new();
        
        // Fields must be in lexicographic order
        let mut fields: Vec<_> = self.grammar.fields.iter().collect();
        fields.sort_by_key(|(_, name)| name.as_str());
        
        for (i, (_id, name)) in fields.iter().enumerate() {
            let ident = quote::format_ident!("FIELD_NAME_{}", i);
            let name_bytes = format!("{}\0", name).into_bytes();
            names.push(quote! {
                static #ident: &[u8] = &[#(#name_bytes),*];
            });
            name_idents.push(ident);
        }
        
        let ptrs = name_idents.iter().map(|ident| {
            quote! { SyncPtr::new(#ident.as_ptr()) }
        }).collect();
        
        (names, ptrs)
    }
    
    /// Generate symbol metadata
    fn generate_symbol_metadata(&self) -> Vec<TokenStream> {
        let mut metadata = Vec::new();
        
        eprintln!("\nDEBUG generate_symbol_metadata: Starting metadata generation");
        eprintln!("  grammar.extras = {:?}", self.grammar.extras);
        
        // Debug: Check all tokens in the grammar
        eprintln!("  All tokens in grammar:");
        for (id, token) in &self.grammar.tokens {
            eprintln!("    Token {:?}: name='{}', pattern={:?}", id, token.name, token.pattern);
        }
        
        // First, find all terminal tokens that should be marked as extras
        let extra_tokens = self.find_extra_tokens();
        eprintln!("  extra_tokens found = {:?}", extra_tokens);
        
        // Generate metadata in parse table order using symbol_to_index mapping
        let mut index_to_symbol: Vec<Option<SymbolId>> = vec![None; self.parse_table.symbol_count];
        for (symbol_id, &index) in &self.parse_table.symbol_to_index {
            if index < self.parse_table.symbol_count {
                index_to_symbol[index] = Some(*symbol_id);
            }
        }
        
        eprintln!("  Generating metadata in parse table order:");
        eprintln!("  symbol_to_index mapping: {:?}", self.parse_table.symbol_to_index);
        for (idx, symbol_id_opt) in index_to_symbol.iter().enumerate() {
            if let Some(symbol_id) = symbol_id_opt {
                if symbol_id.0 == 0 {
                    // EOF symbol
                    let meta_byte = create_symbol_metadata(true, false, false, false, false);
                    eprintln!("    Index {}: EOF, metadata={:#x}", idx, meta_byte);
                    metadata.push(quote! { #meta_byte });
                } else if let Some(token) = self.grammar.tokens.get(symbol_id) {
                    // Terminal token
                    let visible = !token.name.starts_with('_');
                    let named = visible && matches!(&token.pattern, TokenPattern::Regex(_));
                    let hidden = extra_tokens.contains(symbol_id);
                    let meta_byte = create_symbol_metadata(visible, named, hidden, false, false);
                    eprintln!("    Index {}: Token {} (id={:?}): visible={}, named={}, hidden={}, metadata={:#x}", 
                             idx, token.name, symbol_id, visible, named, hidden, meta_byte);
                    metadata.push(quote! { #meta_byte });
                } else if self.grammar.rules.contains_key(symbol_id) {
                    // Non-terminal
                    let name = self.grammar.rule_names.get(symbol_id)
                        .cloned()
                        .unwrap_or_else(|| format!("rule_{}", symbol_id.0));
                    let visible = !name.starts_with('_');
                    let named = visible;
                    let hidden = false; // Non-terminals are never hidden
                    let supertype = self.grammar.supertypes.contains(symbol_id);
                    let meta_byte = create_symbol_metadata(visible, named, hidden, false, supertype);
                    eprintln!("    Index {}: Non-terminal {} (id={:?}): visible={}, named={}, supertype={}, metadata={:#x}", 
                             idx, name, symbol_id, visible, named, supertype, meta_byte);
                    metadata.push(quote! { #meta_byte });
                } else if let Some(external) = self.grammar.externals.iter().find(|e| e.symbol_id == *symbol_id) {
                    // External token
                    let visible = !external.name.starts_with('_');
                    let named = visible;
                    let meta_byte = create_symbol_metadata(visible, named, false, false, false);
                    eprintln!("    Index {}: External {} (id={:?}): visible={}, named={}, metadata={:#x}", 
                             idx, external.name, symbol_id, visible, named, meta_byte);
                    metadata.push(quote! { #meta_byte });
                } else {
                    // Unknown symbol - shouldn't happen
                    eprintln!("    Index {}: WARNING: Unknown symbol id={:?}", idx, symbol_id);
                    metadata.push(quote! { 0u8 });
                }
            } else {
                // No symbol for this index - shouldn't happen
                eprintln!("    Index {}: WARNING: No symbol mapped", idx);
                metadata.push(quote! { 0u8 });
            }
        }
        
        metadata
    }
    
    /// Generate compressed parse tables
    fn generate_parse_tables(&self) -> (Vec<TokenStream>, Vec<TokenStream>) {
        if let Some(compressed) = self.compressed_tables {
            // Generate compressed table data
            let mut table_data = Vec::new();
            let mut map_data = Vec::new();
            
            // Encode action table
            for entry in &compressed.action_table.data {
                let symbol = entry.symbol;
                table_data.push(quote! { #symbol });
                if let Ok(encoded) = self.encode_action(&entry.action) {
                    table_data.push(quote! { #encoded });
                }
            }
            
            // Add row offsets to map
            // Note: row_offsets are in terms of entries, but the parse table
            // uses u16 indices, so we need to multiply by 2
            for &offset in &compressed.action_table.row_offsets {
                let u16_offset = offset * 2;
                map_data.push(quote! { #u16_offset as u32 });
            }
            
            (table_data, map_data)
        } else {
            // Fallback: generate compressed table format without proper compression
            // This stores only non-error entries as (symbol, action) pairs
            let mut table_data = Vec::new();
            let mut map_data = Vec::new();
            let mut current_offset = 0u32;
            
            for state_idx in 0..self.parse_table.state_count {
                // Record the starting offset for this state
                eprintln!("DEBUG: State {} starts at offset {} (u16 index)", state_idx, current_offset);
                map_data.push(quote! { #current_offset });
                
                eprintln!("DEBUG: Processing state {}", state_idx);
                
                // Check if this state has a default reduce action
                // (all non-error actions are the same reduce action)
                let mut default_reduce = None;
                let mut has_non_reduce = false;
                let mut non_error_actions = Vec::new();
                
                eprintln!("DEBUG: State {} iterating through {} symbols", state_idx, self.parse_table.symbol_count);
                for symbol_idx in 0..self.parse_table.symbol_count {
                    let action = if state_idx < self.parse_table.action_table.len() 
                        && symbol_idx < self.parse_table.action_table[state_idx].len() {
                        &self.parse_table.action_table[state_idx][symbol_idx]
                    } else {
                        &Action::Error
                    };
                    eprintln!("DEBUG: State {} symbol_idx={} action={:?}", state_idx, symbol_idx, action);
                    
                    match action {
                        Action::Error => continue,
                        Action::Reduce(prod_id) => {
                            non_error_actions.push((symbol_idx, action));
                            if let Some(default_prod) = &default_reduce {
                                if default_prod != prod_id {
                                    // Different reduce actions, no default
                                    eprintln!("DEBUG: State {} has different reduce actions: {:?} vs {:?}", state_idx, default_prod, prod_id);
                                    has_non_reduce = true;
                                }
                            } else {
                                eprintln!("DEBUG: State {} setting default_reduce to {:?}", state_idx, prod_id);
                                default_reduce = Some(prod_id.clone());
                            }
                        }
                        _ => {
                            // Shift, Accept, or Fork - no default reduce
                            eprintln!("DEBUG: State {} has non-reduce action: {:?}", state_idx, action);
                            has_non_reduce = true;
                            non_error_actions.push((symbol_idx, action));
                        }
                    }
                }
                
                // If all actions are the same reduce, emit a default reduce entry
                if let Some(prod_id) = default_reduce {
                    if !has_non_reduce && !non_error_actions.is_empty() {
                        eprintln!("DEBUG: State {} has default reduce to production {}", state_idx, prod_id.0);
                        // Emit default reduce entry with high bit set in symbol
                        // The symbol field contains the production ID with high bit set
                        let symbol_with_high_bit = 0x8000u16 | prod_id.0;
                        table_data.push(quote! { #symbol_with_high_bit });
                        table_data.push(quote! { 0u16 }); // action value (unused for default reduce)
                        current_offset += 2;
                        continue; // Skip to next state
                    }
                }
                
                // Check if all non-error actions are the same reduce
                // This is a more comprehensive check for default reduce
                let mut all_same_reduce = true;
                let mut common_reduce = None;
                for (_, action) in &non_error_actions {
                    match action {
                        Action::Reduce(prod_id) => {
                            if let Some(common) = &common_reduce {
                                if common != prod_id {
                                    all_same_reduce = false;
                                    break;
                                }
                            } else {
                                common_reduce = Some(prod_id.clone());
                            }
                        }
                        _ => {
                            all_same_reduce = false;
                            break;
                        }
                    }
                }
                
                // If all non-error actions are the same reduce, emit a default reduce
                if all_same_reduce {
                    if let Some(prod_id) = common_reduce {
                        eprintln!("DEBUG: State {} has default reduce to production {} (fallback check)", state_idx, prod_id.0);
                        // Emit default reduce entry with high bit set in symbol
                        let symbol_with_high_bit = 0x8000u16 | prod_id.0;
                        table_data.push(quote! { #symbol_with_high_bit });
                        table_data.push(quote! { 0u16 }); // action value (unused for default reduce)
                        current_offset += 2;
                    }
                } else {
                    // Add entries for this state (only non-error actions)
                    eprintln!("DEBUG: State {} has {} non-error actions", state_idx, non_error_actions.len());
                    for (symbol_idx, action) in non_error_actions {
                        // The parse table already uses indices, not symbol IDs
                        let symbol_index = symbol_idx as u16;
                        table_data.push(quote! { #symbol_index });
                        
                        if let Ok(encoded) = self.encode_action(action) {
                            eprintln!("DEBUG: State {} entry: symbol={}, action={:?}, encoded={}", state_idx, symbol_index, action, encoded);
                            table_data.push(quote! { #encoded });
                        } else {
                            table_data.push(quote! { 0u16 });
                        }
                        current_offset += 2;
                    }
                }
                eprintln!("DEBUG: State {} ends at offset {}", state_idx, current_offset);
            }
            
            // Add final offset for end of table
            eprintln!("DEBUG: Final offset: {}", current_offset);
            map_data.push(quote! { #current_offset });
            
            (table_data, map_data)
        }
    }
    
    /// Encode an action as u16
    fn encode_action(&self, action: &Action) -> Result<u16, String> {
        match action {
            Action::Shift(state) => Ok(state.0),
            Action::Reduce(rule) => Ok(0x8000 | rule.0), // Don't shift rule ID
            Action::Accept => Ok(0x7FFF),  // Use 0x7FFF for accept to match parser
            Action::Error => Ok(0),         // Use 0 for error to match parser expectation
            Action::Fork(_) => Ok(0),       // Treat fork as error for now
        }
    }
    
    /// Generate parse actions
    fn generate_parse_actions(&self) -> Vec<TokenStream> {
        // Generate production information for reduce actions
        // The array must be indexed by production ID, not sequential
        
        // First, find the maximum production ID to size the array
        let max_production_id = self.grammar.all_rules()
            .map(|rule| rule.production_id.0)
            .max()
            .unwrap_or(0);
        
        // Create array with dummy entries
        let mut actions = vec![quote! {
            TSParseAction {
                action_type: 0,
                extra: 0,
                child_count: 0,
                dynamic_precedence: 0,
                symbol: 0,
            }
        }; (max_production_id + 1) as usize];
        
        // Fill in the actual productions at their correct indices
        for rule in self.grammar.all_rules() {
            let index = rule.production_id.0 as usize;
            let child_count = rule.rhs.len() as u8;
            let symbol = rule.lhs.0;
            
            actions[index] = quote! {
                TSParseAction {
                    action_type: 1, // Reduce
                    extra: 0,
                    child_count: #child_count,
                    dynamic_precedence: 0,
                    symbol: #symbol,
                }
            };
        }
        
        actions
    }
    
    /// Generate lex modes
    fn generate_lex_modes(&self) -> Vec<TokenStream> {
        let mut modes = Vec::new();
        
        for i in 0..self.parse_table.state_count {
            modes.push(quote! {
                TSLexState {
                    lex_state: #i as u16,
                    external_lex_state: 0,
                }
            });
        }
        
        modes
    }
    
    /// Generate field maps
    fn generate_field_maps(&self) -> (Vec<TokenStream>, Vec<TokenStream>) {
        // TODO: Implement proper field mapping
        (vec![quote! { 0u16 }], vec![quote! { 0u16 }])
    }
    
    /// Generate public symbol map
    fn generate_public_symbol_map(&self) -> Vec<TokenStream> {
        let symbol_count = self.calculate_symbol_count();
        (0..symbol_count).map(|i| {
            quote! { #i as u16 }
        }).collect()
    }
    
    /// Generate primary state IDs
    fn generate_primary_state_ids(&self) -> Vec<TokenStream> {
        (0..self.parse_table.state_count).map(|i| {
            quote! { #i as u16 }
        }).collect()
    }
    
    /// Generate production ID map
    fn generate_production_id_map(&self) -> Vec<TokenStream> {
        // Map production IDs to rule symbols
        let mut production_map = Vec::new();
        
        // Sort rules by production ID for deterministic output
        let mut rules: Vec<_> = self.grammar.rules.iter()
            .flat_map(|(_, rules)| rules.iter())
            .collect();
        rules.sort_by_key(|rule| rule.production_id.0);
        
        for rule in rules {
            let rule_symbol = rule.lhs.0 as u16;
            production_map.push(quote! { #rule_symbol });
        }
        
        production_map
    }
    
    /// Calculate counts for the language structure
    fn calculate_counts(&self) -> LanguageCounts {
        LanguageCounts {
            symbol_count: self.calculate_symbol_count() as u32,
            alias_count: 0, // TODO: Implement aliases
            token_count: self.grammar.tokens.len() as u32,
            external_token_count: self.grammar.externals.len() as u32,
            state_count: self.parse_table.state_count as u32,
            large_state_count: 0, // TODO: Calculate large states
            production_id_count: self.calculate_production_count() as u32,
            field_count: self.grammar.fields.len() as u32,
            max_alias_sequence_length: 0,
        }
    }
    
    fn calculate_symbol_count(&self) -> usize {
        // Use the parse table's symbol count which is the correct count after processing
        self.parse_table.symbol_count
    }
    
    fn calculate_production_count(&self) -> usize {
        self.grammar.rules.values()
            .flat_map(|rules| rules.iter())
            .count()
    }
    
    /// Find all terminal tokens that should be marked as extras
    fn find_extra_tokens(&self) -> HashSet<SymbolId> {
        let mut extra_tokens = HashSet::new();
        let mut visited = HashSet::new();
        
        eprintln!("DEBUG find_extra_tokens: grammar.extras = {:?}", self.grammar.extras);
        
        // Check if any extras directly refer to tokens
        for &extra_symbol in &self.grammar.extras {
            if self.grammar.tokens.contains_key(&extra_symbol) {
                eprintln!("  Extra symbol {:?} is directly a token!", extra_symbol);
                extra_tokens.insert(extra_symbol);
            }
        }
        
        // For each extra symbol, find all terminal tokens it can produce (recursively)
        for &extra_symbol in &self.grammar.extras {
            eprintln!("  Processing extra symbol: {:?}", extra_symbol);
            self.find_terminals_recursive(extra_symbol, &mut extra_tokens, &mut visited);
        }
        
        eprintln!("DEBUG find_extra_tokens: result = {:?}", extra_tokens);
        extra_tokens
    }
    
    /// Recursively find all terminal tokens reachable from a symbol
    fn find_terminals_recursive(
        &self,
        symbol: SymbolId,
        terminals: &mut HashSet<SymbolId>,
        visited: &mut HashSet<SymbolId>
    ) {
        // Avoid infinite recursion
        if !visited.insert(symbol) {
            return;
        }
        
        // If it's a terminal token, add it
        if self.grammar.tokens.contains_key(&symbol) {
            eprintln!("    Found terminal: {:?}", symbol);
            terminals.insert(symbol);
            return;
        }
        
        // If it's a non-terminal, explore all its rules
        if let Some(rules) = self.grammar.rules.get(&symbol) {
            eprintln!("    Exploring non-terminal {:?} with {} rules", symbol, rules.len());
            for rule in rules {
                eprintln!("      Rule: {:?} -> {:?}", rule.lhs, rule.rhs);
                for sym in &rule.rhs {
                    match sym {
                        Symbol::Terminal(token_id) => {
                            eprintln!("        Found terminal in rule: {:?}", token_id);
                            terminals.insert(*token_id);
                        }
                        Symbol::NonTerminal(nt_id) => {
                            eprintln!("        Recursing into non-terminal: {:?}", nt_id);
                            self.find_terminals_recursive(*nt_id, terminals, visited);
                        }
                        Symbol::External(_) | Symbol::Optional(_) | Symbol::Repeat(_) | 
                        Symbol::RepeatOne(_) | Symbol::Choice(_) | Symbol::Sequence(_) |
                        Symbol::Epsilon => {
                            // These symbol types are not expected in the IR at this stage
                            eprintln!("        WARNING: Unexpected symbol type in rule: {:?}", sym);
                        }
                    }
                }
            }
        }
    }
}

struct LanguageCounts {
    symbol_count: u32,
    alias_count: u32,
    token_count: u32,
    external_token_count: u32,
    state_count: u32,
    large_state_count: u32,
    production_id_count: u32,
    field_count: u32,
    max_alias_sequence_length: u16,
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_sitter_ir::*;
    use std::collections::HashMap;
    
    #[test]
    fn test_deterministic_symbol_ordering() {
        let mut grammar = Grammar::new("test".to_string());
        
        // Add tokens in non-sorted order
        grammar.tokens.insert(SymbolId(5), Token {
            name: "token5".to_string(),
            pattern: TokenPattern::String("5".to_string()),
            fragile: false,
        });
        grammar.tokens.insert(SymbolId(1), Token {
            name: "token1".to_string(),
            pattern: TokenPattern::String("1".to_string()),
            fragile: false,
        });
        
        let parse_table = ParseTable {
            action_table: vec![],
            goto_table: vec![],
            symbol_metadata: vec![],
            state_count: 1,
            symbol_count: 3,
            symbol_to_index: HashMap::new(),
        };
        
        let builder = AbiLanguageBuilder::new(&grammar, &parse_table);
        let (names, _) = builder.generate_symbol_names();
        
        // Should have EOF + 2 tokens
        assert_eq!(names.len(), 3);
        
        // Check that tokens are sorted by ID
        let code = quote! { #(#names)* }.to_string();
        assert!(code.contains("token1"));
        assert!(code.contains("token5"));
        let token1_pos = code.find("token1").unwrap();
        let token5_pos = code.find("token5").unwrap();
        assert!(token1_pos < token5_pos);
    }
}