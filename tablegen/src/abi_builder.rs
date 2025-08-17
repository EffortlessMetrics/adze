// ABI-compatible language builder for Tree-sitter
// This module generates static Language structures that match Tree-sitter's C ABI exactly

use crate::abi::*;
use crate::compress::CompressedTables;
use proc_macro2::TokenStream;
use quote::quote;
use rust_sitter_glr_core::{Action, ParseTable};
use rust_sitter_ir::{Grammar, Rule, Symbol, SymbolId, TokenPattern};
use std::collections::HashSet;

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

    /// Get the name of a symbol for debugging
    fn get_symbol_name(&self, symbol_id: SymbolId) -> String {
        if symbol_id.0 == 0 {
            "end".to_string()
        } else if let Some(name) = self.grammar.rule_names.get(&symbol_id) {
            name.clone()
        } else if let Some(token) = self.grammar.tokens.get(&symbol_id) {
            token.name.clone()
        } else {
            format!("???{}", symbol_id.0)
        }
    }

    /// Generate the complete language module
    pub fn generate(&self) -> TokenStream {
        let language_name = &self.grammar.name;
        let language_fn_ident = quote::format_ident!("tree_sitter_{}", language_name);

        eprintln!(
            "DEBUG AbiLanguageBuilder: Generating language for '{}'",
            language_name
        );
        eprintln!("DEBUG AbiLanguageBuilder: symbol_to_index mapping:");
        for (symbol_id, &index) in &self.parse_table.symbol_to_index {
            let symbol_name = self.get_symbol_name(*symbol_id);
            eprintln!(
                "  SymbolId({}) -> index {} ('{}')",
                symbol_id.0, index, symbol_name
            );
        }

        // Check what the initial state expects
        if !self.parse_table.action_table.is_empty() {
            eprintln!("DEBUG AbiLanguageBuilder: State 0 actions:");
            for (symbol_idx, action_cell) in self.parse_table.action_table[0].iter().enumerate() {
                if !action_cell.is_empty() {
                    // Find the symbol ID for this index
                    let symbol_id = self
                        .parse_table
                        .symbol_to_index
                        .iter()
                        .find(|(_, idx)| **idx == symbol_idx)
                        .map(|(id, _)| *id);
                    eprintln!(
                        "  Index {} (SymbolId {:?}): {:?}",
                        symbol_idx, symbol_id, action_cell
                    );
                }
            }
        }

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
        let production_lhs_index = self.generate_production_lhs_index();
        let variant_symbol_map = self.generate_variant_symbol_map();

        // Generate external scanner data if needed
        let (external_scanner_code, external_scanner_struct) = if !self.grammar.externals.is_empty()
        {
            use crate::external_scanner_v2::ExternalScannerGenerator;

            let scanner_gen =
                ExternalScannerGenerator::new(self.grammar.clone(), self.parse_table.clone());
            let scanner_interface = scanner_gen.generate_scanner_interface();

            // Skip generating scanner FFI functions - let grammars provide their own
            // Grammars with external scanners should implement their own FFI functions
            let scanner_functions = quote! {};

            let scanner_struct = quote! {
                ExternalScanner {
                    states: EXTERNAL_SCANNER_STATES.as_ptr() as *const u8,
                    symbol_map: EXTERNAL_SCANNER_SYMBOL_MAP.as_ptr() as *const TSSymbol,
                    create: None,
                    destroy: None,
                    scan: None,
                    serialize: None,
                    deserialize: None,
                }
            };

            (
                quote! {
                    #scanner_interface
                    #scanner_functions
                },
                scanner_struct,
            )
        } else {
            (
                quote! {},
                quote! {
                    ExternalScanner {
                        states: std::ptr::null(),
                        symbol_map: std::ptr::null(),
                        create: None,
                        destroy: None,
                        scan: None,
                        serialize: None,
                        deserialize: None,
                    }
                },
            )
        };

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

        // Debug: Print symbol_to_index mapping for tokens
        eprintln!("DEBUG: Symbol to index mapping for lexer generation:");
        for (sym_id, idx) in &self.parse_table.symbol_to_index {
            if self.grammar.tokens.contains_key(sym_id) {
                let token = &self.grammar.tokens[sym_id];
                eprintln!(
                    "  Token '{}' (SymbolId {:?}) -> index {}",
                    token.name, sym_id, idx
                );
            }
        }
        eprintln!("DEBUG: token_count = {}", self.parse_table.token_count);

        // Generate lexer function with symbol mapping
        let lexer_code =
            crate::lexer_gen::generate_lexer(self.grammar, &self.parse_table.symbol_to_index);

        // Only import TSSymbol if we have external scanners
        let ts_symbol_import = if !self.grammar.externals.is_empty() {
            quote! { use rust_sitter::TSSymbol; }
        } else {
            quote! {}
        };

        quote! {
            use rust_sitter::pure_parser::*;
            #ts_symbol_import

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

            // Parse table (for large states - empty if all states are compressed)
            static PARSE_TABLE: &[u16] = &[];

            // Small parse table (compressed states data)
            pub static SMALL_PARSE_TABLE: &[u16] = &[#(#parse_table_data),*];

            // Small parse table map
            pub static SMALL_PARSE_TABLE_MAP: &[u32] = &[#(#small_parse_table_map),*];

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

            // Production LHS index (maps production IDs to LHS symbols in table index space)
            static PRODUCTION_LHS_INDEX: &[u16] = &[#(#production_lhs_index),*];

            // Variant symbol map (for Extract trait to use)
            #variant_symbol_map

            // External scanner support (if needed)
            #external_scanner_code

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
                small_parse_table: SMALL_PARSE_TABLE.as_ptr(),
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
                external_scanner: #external_scanner_struct,
                primary_state_ids: PRIMARY_STATE_IDS.as_ptr(),
                production_lhs_index: PRODUCTION_LHS_INDEX.as_ptr(),
                production_count: #production_id_count as u16,
            };

            // Export the language function for FFI
            // Edition-aware attribute toggle (2021 vs 2024)
            #[cfg(rust_sitter_unsafe_attrs)]
            #[unsafe(no_mangle)]
            #[cfg(not(rust_sitter_unsafe_attrs))]
            #[no_mangle]
            pub unsafe extern "C" fn #language_fn_ident() -> *const TSLanguage {
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

        let ptrs = name_idents
            .iter()
            .map(|ident| {
                quote! { SyncPtr::new(#ident.as_ptr()) }
            })
            .collect();

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

        let ptrs = name_idents
            .iter()
            .map(|ident| {
                quote! { SyncPtr::new(#ident.as_ptr()) }
            })
            .collect();

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
            eprintln!(
                "    Token {:?}: name='{}', pattern={:?}",
                id, token.name, token.pattern
            );
        }

        // First, find all terminal tokens that should be marked as extras
        let extra_tokens = self.find_extra_tokens();
        eprintln!("  extra_tokens found = {:?}", extra_tokens);

        // Debug: Print which symbol corresponds to whitespace
        eprintln!("  Looking for whitespace token (should be symbol 4):");
        for (id, token) in &self.grammar.tokens {
            if token.name.contains("whitespace")
                || token.pattern == TokenPattern::Regex(r"\s".to_string())
            {
                eprintln!(
                    "    Found whitespace-like token: {:?} -> {}",
                    id, token.name
                );
            }
        }

        // Generate metadata in parse table order using symbol_to_index mapping
        let mut index_to_symbol: Vec<Option<SymbolId>> = vec![None; self.parse_table.symbol_count];
        for (symbol_id, &index) in &self.parse_table.symbol_to_index {
            if index < self.parse_table.symbol_count {
                index_to_symbol[index] = Some(*symbol_id);
            }
        }

        eprintln!("  Generating metadata in parse table order:");
        eprintln!(
            "  symbol_to_index mapping: {:?}",
            self.parse_table.symbol_to_index
        );
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
                    let _original_hidden = extra_tokens.contains(symbol_id);

                    // Special handling for whitespace tokens
                    // If this is a whitespace token (by pattern), it should be hidden
                    let is_whitespace_token = matches!(&token.pattern, TokenPattern::Regex(p) if p == r"\s")
                        || token.name.to_lowercase().contains("whitespace");

                    if is_whitespace_token {
                        eprintln!(
                            "    WHITESPACE TOKEN FOUND: {} (id={:?})",
                            token.name, symbol_id
                        );
                        eprintln!("      Pattern: {:?}", token.pattern);
                        eprintln!(
                            "      Was in extra_tokens: {}",
                            extra_tokens.contains(symbol_id)
                        );
                    }

                    // Force whitespace tokens to be hidden
                    let hidden = extra_tokens.contains(symbol_id) || is_whitespace_token;

                    let meta_byte = create_symbol_metadata(visible, named, hidden, false, false);
                    eprintln!(
                        "    Index {}: Token {} (id={:?}): visible={}, named={}, hidden={}, metadata={:#x}",
                        idx, token.name, symbol_id, visible, named, hidden, meta_byte
                    );
                    metadata.push(quote! { #meta_byte });
                } else if self.grammar.rules.contains_key(symbol_id) {
                    // Non-terminal
                    let name = self
                        .grammar
                        .rule_names
                        .get(symbol_id)
                        .cloned()
                        .unwrap_or_else(|| format!("rule_{}", symbol_id.0));
                    let visible = !name.starts_with('_');
                    let named = visible;
                    let hidden = false; // Non-terminals are never hidden
                    let supertype = self.grammar.supertypes.contains(symbol_id);
                    let meta_byte =
                        create_symbol_metadata(visible, named, hidden, false, supertype);
                    eprintln!(
                        "    Index {}: Non-terminal {} (id={:?}): visible={}, named={}, supertype={}, metadata={:#x}",
                        idx, name, symbol_id, visible, named, supertype, meta_byte
                    );
                    metadata.push(quote! { #meta_byte });
                } else if let Some(external) = self
                    .grammar
                    .externals
                    .iter()
                    .find(|e| e.symbol_id == *symbol_id)
                {
                    // External token
                    let visible = !external.name.starts_with('_');
                    let named = visible;
                    let meta_byte = create_symbol_metadata(visible, named, false, false, false);
                    eprintln!(
                        "    Index {}: External {} (id={:?}): visible={}, named={}, metadata={:#x}",
                        idx, external.name, symbol_id, visible, named, meta_byte
                    );
                    metadata.push(quote! { #meta_byte });
                } else {
                    // Unknown symbol - shouldn't happen
                    eprintln!(
                        "    Index {}: WARNING: Unknown symbol id={:?}",
                        idx, symbol_id
                    );
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

            // TODO: Also encode goto table entries
            // Tree-sitter combines both action and goto entries in the parse table
            // The goto entries should be added here as well

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
                eprintln!(
                    "DEBUG: State {} starts at offset {} (u16 index)",
                    state_idx, current_offset
                );
                map_data.push(quote! { #current_offset });

                eprintln!("DEBUG: Processing state {}", state_idx);

                // Check if this state has a default reduce action
                // (all non-error actions are the same reduce action)
                let mut default_reduce = None;
                let mut has_non_reduce = false;
                let mut non_error_actions = Vec::new();

                eprintln!(
                    "DEBUG: State {} iterating through {} symbols",
                    state_idx, self.parse_table.symbol_count
                );
                for symbol_idx in 0..self.parse_table.symbol_count {
                    // Get the symbol ID for this index
                    let symbol_id = self
                        .parse_table
                        .symbol_to_index
                        .iter()
                        .find(|&(_, &idx)| idx == symbol_idx)
                        .map(|(id, _)| *id);

                    if symbol_id.is_none() {
                        eprintln!("DEBUG: No symbol ID found for index {}", symbol_idx);
                        continue;
                    }

                    let symbol_id = symbol_id.unwrap();

                    // Debug: Print symbol mapping
                    eprintln!(
                        "DEBUG: State {} checking symbol_idx={} -> symbol_id={}",
                        state_idx, symbol_idx, symbol_id.0
                    );

                    // Check if this symbol is a terminal or non-terminal
                    // Terminals include tokens and externals
                    let is_terminal = self.grammar.tokens.contains_key(&symbol_id)
                        || self
                            .grammar
                            .externals
                            .iter()
                            .any(|e| e.symbol_id == symbol_id)
                        || symbol_id.0 == 0; // EOF is also a terminal

                    // Create owned action to avoid borrowing issues
                    let action_owned = if is_terminal {
                        // Terminal symbol - use action table
                        if state_idx < self.parse_table.action_table.len()
                            && symbol_idx < self.parse_table.action_table[state_idx].len()
                        {
                            let actions = &self.parse_table.action_table[state_idx][symbol_idx];
                            if actions.is_empty() {
                                Action::Error
                            } else if actions.len() == 1 {
                                actions[0].clone()
                            } else {
                                // Multiple actions - for now use Fork, though Tree-sitter
                                // typically resolves these during table generation
                                Action::Fork(actions.clone())
                            }
                        } else {
                            Action::Error
                        }
                    } else {
                        // Non-terminal symbol - use goto table indexed by symbol_idx
                        if state_idx < self.parse_table.goto_table.len()
                            && symbol_idx < self.parse_table.goto_table[state_idx].len()
                        {
                            let goto_state = self.parse_table.goto_table[state_idx][symbol_idx];
                            eprintln!(
                                "DEBUG: Non-terminal {} -> goto_table[{}][{}] = state {}",
                                symbol_id.0, state_idx, symbol_idx, goto_state.0
                            );
                            if goto_state.0 > 0 {
                                // Convert goto to a shift action for Tree-sitter compatibility
                                Action::Shift(goto_state)
                            } else {
                                Action::Error
                            }
                        } else {
                            eprintln!(
                                "DEBUG: Non-terminal {} -> goto_table bounds check failed: state_idx={}, symbol_idx={}, goto_table.len={}, goto_table[{}].len={}",
                                symbol_id.0,
                                state_idx,
                                symbol_idx,
                                self.parse_table.goto_table.len(),
                                state_idx,
                                if state_idx < self.parse_table.goto_table.len() {
                                    self.parse_table.goto_table[state_idx].len()
                                } else {
                                    0
                                }
                            );
                            Action::Error
                        }
                    };
                    let action = &action_owned;
                    eprintln!(
                        "DEBUG: State {} symbol_idx={} is_terminal={} action={:?}",
                        state_idx, symbol_idx, is_terminal, action
                    );

                    match action {
                        Action::Error => continue,
                        Action::Reduce(prod_id) => {
                            non_error_actions.push((symbol_idx, action_owned.clone()));
                            if let Some(default_prod) = &default_reduce {
                                if default_prod != prod_id {
                                    // Different reduce actions, no default
                                    eprintln!(
                                        "DEBUG: State {} has different reduce actions: {:?} vs {:?}",
                                        state_idx, default_prod, prod_id
                                    );
                                    has_non_reduce = true;
                                }
                            } else {
                                eprintln!(
                                    "DEBUG: State {} setting default_reduce to {:?}",
                                    state_idx, prod_id
                                );
                                default_reduce = Some(*prod_id);
                            }
                        }
                        _ => {
                            // Shift, Accept, or Fork - no default reduce
                            eprintln!(
                                "DEBUG: State {} has non-reduce action: {:?}",
                                state_idx, action
                            );
                            has_non_reduce = true;
                            non_error_actions.push((symbol_idx, action_owned.clone()));
                        }
                    }
                }

                // If all actions are the same reduce, emit a default reduce entry
                if let Some(prod_id) = default_reduce {
                    if !has_non_reduce && !non_error_actions.is_empty() {
                        eprintln!(
                            "DEBUG: State {} has default reduce to production {}",
                            state_idx, prod_id.0
                        );
                        // Emit default reduce entry with high bit set in symbol
                        // The symbol field is 0x8000 to indicate default reduce
                        // The action value contains the 1-based production ID with high bit set
                        table_data.push(quote! { 0x8000u16 });
                        let reduce_action = 0x8000u16 | (prod_id.0 + 1);
                        table_data.push(quote! { #reduce_action });
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
                                common_reduce = Some(*prod_id);
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
                        eprintln!(
                            "DEBUG: State {} has default reduce to production {} (fallback check)",
                            state_idx, prod_id.0
                        );
                        // Emit default reduce entry with high bit set in symbol
                        // The symbol field is 0x8000 to indicate default reduce
                        // The action value contains the 1-based production ID with high bit set
                        table_data.push(quote! { 0x8000u16 });
                        let reduce_action = 0x8000u16 | (prod_id.0 + 1);
                        table_data.push(quote! { #reduce_action });
                        current_offset += 2;
                    }
                } else {
                    eprintln!(
                        "DEBUG: State {} NOT using default reduce, all_same_reduce={}",
                        state_idx, all_same_reduce
                    );
                    // Add entries for this state (only non-error actions)
                    eprintln!(
                        "DEBUG: State {} has {} non-error actions",
                        state_idx,
                        non_error_actions.len()
                    );

                    // The LR construction in glr-core now handles EOF reduce actions properly

                    for (symbol_idx, action) in non_error_actions {
                        // Convert table index back to symbol ID for runtime lookup
                        // The runtime compares stored values with symbol IDs, not indices
                        let symbol_id = self.parse_table.index_to_symbol[symbol_idx].0;
                        table_data.push(quote! { #symbol_id });

                        if let Ok(encoded) = self.encode_action(&action) {
                            eprintln!(
                                "DEBUG: State {} entry: symbol_id={} (from idx={}), action={:?}, encoded={}",
                                state_idx, symbol_id, symbol_idx, action, encoded
                            );
                            table_data.push(quote! { #encoded });
                        } else {
                            table_data.push(quote! { 0u16 });
                        }
                        current_offset += 2;
                    }
                }
                eprintln!(
                    "DEBUG: State {} ends at offset {}",
                    state_idx, current_offset
                );
            }

            // Add final offset for end of table
            eprintln!("DEBUG: Final offset: {}", current_offset);
            map_data.push(quote! { #current_offset });

            (table_data, map_data)
        }
    }

    /// Encode an action as u16
    #[allow(clippy::only_used_in_recursion)]
    fn encode_action(&self, action: &Action) -> Result<u16, String> {
        match action {
            Action::Shift(state) => Ok(state.0),
            Action::Reduce(rule) => {
                // Tree-sitter uses 1-based production IDs in reduce actions
                // The runtime will map through PRODUCTION_ID_MAP to get the actual index
                Ok(0x8000 | (rule.0 + 1))
            }
            Action::Accept => Ok(0x7FFF), // Use 0x7FFF for accept to match parser
            Action::Error => Ok(0),       // Use 0 for error to match parser expectation
            Action::Recover => Ok(0xFFFD), // Use distinct value for Recover
            Action::Fork(actions) => {
                // For Fork actions, we need to choose one action from the fork
                // For now, let's prefer reduce actions over shift actions
                // This is a simplified conflict resolution strategy

                // First, try to find a reduce action
                for action in actions {
                    if let Action::Reduce(_) = action {
                        return self.encode_action(action);
                    }
                }

                // If no reduce action, take the first non-error action
                for action in actions {
                    if !matches!(action, Action::Error) {
                        return self.encode_action(action);
                    }
                }

                // If all actions are errors (shouldn't happen), return error
                Ok(0)
            }
            _ => {
                // Unknown action type - treat as error
                crate::util::unexpected_action(action, "encode_action");
                Ok(0)
            }
        }
    }

    /// Generate parse actions
    fn generate_parse_actions(&self) -> Vec<TokenStream> {
        // Generate production information for reduce actions
        // The array must be indexed by production ID, not sequential

        // First, find the maximum production ID to size the array
        let max_production_id = self
            .grammar
            .all_rules()
            .map(|rule| rule.production_id.0)
            .max()
            .unwrap_or(0);

        // Create array with dummy entries
        let mut actions = vec![
            quote! {
                TSParseAction {
                    action_type: 0,
                    extra: 0,
                    child_count: 0,
                    dynamic_precedence: 0,
                    symbol: 0,
                }
            };
            (max_production_id + 1) as usize
        ];

        // Fill in the actual productions at their correct indices
        eprintln!(
            "DEBUG: Building PARSE_ACTIONS for {} productions",
            self.grammar.all_rules().count()
        );
        eprintln!(
            "DEBUG: symbol_to_index mapping: {:?}",
            self.parse_table.symbol_to_index
        );

        for rule in self.grammar.all_rules() {
            let index = rule.production_id.0 as usize;
            let child_count = rule.rhs.len() as u8;
            // Convert symbol ID to symbol index for the parse table
            let symbol_id = rule.lhs;
            let symbol = self
                .parse_table
                .symbol_to_index
                .get(&symbol_id)
                .copied()
                .unwrap_or_else(|| {
                    eprintln!(
                        "WARNING: No symbol index found for symbol ID {} in production {}",
                        symbol_id.0, rule.production_id.0
                    );
                    symbol_id.0 as usize // Fallback to symbol ID
                }) as u16;

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
        let mut field_map_slices = Vec::new();
        let mut field_map_entries = Vec::new();

        // Group rules by production ID
        let mut rules_by_production: std::collections::BTreeMap<u16, Vec<&Rule>> =
            std::collections::BTreeMap::new();
        for (_, rules) in &self.grammar.rules {
            for rule in rules {
                rules_by_production
                    .entry(rule.production_id.0)
                    .or_default()
                    .push(rule);
            }
        }

        // Build field map entries for each production
        for (production_id, rules) in rules_by_production {
            if production_id == 0 {
                continue; // Skip production ID 0
            }

            let start_index = field_map_entries.len() as u16;
            let mut entry_count = 0u16;

            // Process each rule with this production ID
            for rule in rules {
                // Add entries for each field in this rule
                for (field_id, position) in &rule.fields {
                    let field_id_val = field_id.0;
                    let child_index = *position as u8;
                    let inherited = 0u8; // false - TODO: implement inheritance detection

                    // Pack TSFieldMapEntry: field_id (16 bits) | child_index (8 bits) | inherited (8 bits)
                    let packed_entry = (field_id_val as u32)
                        | ((child_index as u32) << 16)
                        | ((inherited as u32) << 24);
                    field_map_entries.push(quote! { #packed_entry as u16 });
                    field_map_entries.push(quote! { (#packed_entry >> 16) as u16 });
                    entry_count += 1;
                }
            }

            // Add slice for this production ID if it has fields
            if entry_count > 0 {
                // Ensure we have slices for all production IDs up to this one
                while field_map_slices.len() < production_id as usize {
                    field_map_slices.push(quote! { 0u16 }); // start
                    field_map_slices.push(quote! { 0u16 }); // length
                }

                // Add this production's slice
                field_map_slices.push(quote! { #start_index });
                field_map_slices.push(quote! { #entry_count });
            }
        }

        // If no field maps were generated, provide minimal data
        if field_map_slices.is_empty() {
            field_map_slices.push(quote! { 0u16 });
        }
        if field_map_entries.is_empty() {
            field_map_entries.push(quote! { 0u16 });
        }

        (field_map_slices, field_map_entries)
    }

    /// Generate public symbol map
    fn generate_public_symbol_map(&self) -> Vec<TokenStream> {
        let symbol_count = self.calculate_symbol_count();
        (0..symbol_count)
            .map(|i| {
                quote! { #i as u16 }
            })
            .collect()
    }

    /// Generate primary state IDs
    fn generate_primary_state_ids(&self) -> Vec<TokenStream> {
        (0..self.parse_table.state_count)
            .map(|i| {
                quote! { #i as u16 }
            })
            .collect()
    }

    /// Generate variant to symbol ID mapping for Extract trait
    fn generate_variant_symbol_map(&self) -> TokenStream {
        // For now, just generate the complete symbol-to-index mapping
        // that the macro can use to fix enum variant extraction
        let mut symbol_entries = Vec::new();

        // Sort symbols by their index to ensure deterministic output
        let mut index_to_symbol: Vec<(usize, SymbolId)> = Vec::new();
        for (symbol_id, &index) in &self.parse_table.symbol_to_index {
            index_to_symbol.push((index, *symbol_id));
        }
        index_to_symbol.sort_by_key(|(idx, _)| *idx);

        // Generate entries for the mapping
        for (index, symbol_id) in index_to_symbol {
            let symbol_id_val = symbol_id.0 as u32;
            let index_val = index as u16;

            // Also include the symbol name for debugging
            let _symbol_name = if symbol_id.0 == 0 {
                "EOF".to_string()
            } else if let Some(token) = self.grammar.tokens.get(&symbol_id) {
                token.name.clone()
            } else if let Some(rule_name) = self.grammar.rule_names.get(&symbol_id) {
                rule_name.clone()
            } else {
                format!("symbol_{}", symbol_id.0)
            };

            symbol_entries.push(quote! {
                // #symbol_name
                (#symbol_id_val, #index_val)
            });
        }

        // Generate the inverse mapping array (index to symbol ID)
        let symbol_count = self.parse_table.symbol_to_index.len();
        let mut index_to_id_entries = vec![quote! { 0 }; symbol_count];

        for (symbol_id, &index) in &self.parse_table.symbol_to_index {
            let symbol_id_val = symbol_id.0;
            index_to_id_entries[index] = quote! { #symbol_id_val };
        }

        quote! {
            // Complete symbol ID to parse table index mapping
            // This is used by the Extract trait to correctly identify symbols
            pub const SYMBOL_ID_TO_INDEX: &[(u32, u16)] = &[
                #(#symbol_entries),*
            ];

            // Inverse mapping: index to symbol ID
            // This is used by the pure parser to convert indices back to symbol IDs
            pub const SYMBOL_INDEX_TO_ID: &[u16] = &[
                #(#index_to_id_entries),*
            ];

            // Helper function to get symbol index from symbol ID
            #[allow(dead_code)]
            pub fn get_symbol_index(symbol_id: u32) -> Option<u16> {
                SYMBOL_ID_TO_INDEX.iter()
                    .find(|(id, _)| *id == symbol_id)
                    .map(|(_, index)| *index)
            }

            // Helper function to get symbol ID from symbol index
            #[allow(dead_code)]
            pub fn get_symbol_id(symbol_index: u16) -> u16 {
                SYMBOL_INDEX_TO_ID[symbol_index as usize]
            }
        }
    }

    /// Generate production ID map
    fn generate_production_id_map(&self) -> Vec<TokenStream> {
        // Tree-sitter uses 1-based production IDs in the parse table
        // This map converts from 1-based IDs to 0-based indices into PARSE_ACTIONS
        let mut production_map = Vec::new();

        // Get all rules sorted by production ID
        let mut rules: Vec<_> = self
            .grammar
            .rules
            .iter()
            .flat_map(|(_, rules)| rules.iter())
            .collect();
        rules.sort_by_key(|rule| rule.production_id.0);

        // Create a mapping from production_id to index
        let mut id_to_index = std::collections::HashMap::new();
        for (index, rule) in rules.iter().enumerate() {
            id_to_index.insert(rule.production_id.0, index);
        }

        // The production_id_map maps from 1-based parse table IDs to 0-based production indices
        // Since we have N productions (0..N-1), the parse table will use IDs 1..N
        let num_productions = rules.len();

        // Build the map: production_id_map[parse_table_id - 1] = production_index
        // For each production index 0..N-1, the parse table uses ID index+1
        for i in 0..num_productions {
            let index_u16 = i as u16;
            production_map.push(quote! { #index_u16 });
        }

        production_map
    }

    fn generate_production_lhs_index(&self) -> Vec<TokenStream> {
        // Generate array of LHS symbols in table index space, indexed by production ID
        let mut lhs_indices = Vec::new();

        // Get all rules sorted by production ID
        let mut rules: Vec<_> = self
            .grammar
            .rules
            .iter()
            .flat_map(|(_, rules)| rules.iter())
            .collect();
        rules.sort_by_key(|rule| rule.production_id.0);

        // For each production, get its LHS symbol in table index space
        for rule in &rules {
            let lhs_idx = self
                .parse_table
                .symbol_to_index
                .get(&rule.lhs)
                .copied()
                .unwrap_or_else(|| {
                    panic!(
                        "LHS symbol {} not found in symbol_to_index for production {}",
                        rule.lhs.0, rule.production_id.0
                    );
                });

            // Guard rail: production LHS must be a non-terminal column
            debug_assert!(
                (lhs_idx as u32) >= self.parse_table.token_count as u32,
                "production LHS must be a non-terminal column (lhs_idx={}, token_count={})",
                lhs_idx,
                self.parse_table.token_count
            );

            let lhs_index = lhs_idx as u16;
            lhs_indices.push(quote! { #lhs_index });
        }

        lhs_indices
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
        self.grammar
            .rules
            .values()
            .flat_map(|rules| rules.iter())
            .count()
    }

    /// Find all terminal tokens that should be marked as extras
    fn find_extra_tokens(&self) -> HashSet<SymbolId> {
        let mut extra_tokens = HashSet::new();
        let mut visited = HashSet::new();

        eprintln!(
            "DEBUG find_extra_tokens: grammar.extras = {:?}",
            self.grammar.extras
        );

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
        visited: &mut HashSet<SymbolId>,
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
            eprintln!(
                "    Exploring non-terminal {:?} with {} rules",
                symbol,
                rules.len()
            );
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
                        Symbol::External(_)
                        | Symbol::Optional(_)
                        | Symbol::Repeat(_)
                        | Symbol::RepeatOne(_)
                        | Symbol::Choice(_)
                        | Symbol::Sequence(_)
                        | Symbol::Epsilon => {
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

    #[test]
    fn test_deterministic_symbol_ordering() {
        let mut grammar = Grammar::new("test".to_string());

        // Add tokens in non-sorted order
        grammar.tokens.insert(
            SymbolId(5),
            Token {
                name: "token5".to_string(),
                pattern: TokenPattern::String("5".to_string()),
                fragile: false,
            },
        );
        grammar.tokens.insert(
            SymbolId(1),
            Token {
                name: "token1".to_string(),
                pattern: TokenPattern::String("1".to_string()),
                fragile: false,
            },
        );

        let mut symbol_to_index = std::collections::BTreeMap::new();
        symbol_to_index.insert(SymbolId(0), 0); // EOF
        symbol_to_index.insert(SymbolId(1), 1); // token1
        symbol_to_index.insert(SymbolId(5), 2); // token5

        // Create a minimal parse table for testing
        let mut parse_table = crate::empty_table!(states: 1, terms: 2, nonterms: 0);

        // Override the symbol mapping for the test
        parse_table.symbol_to_index = symbol_to_index;
        parse_table.symbol_count = 3;

        let builder = AbiLanguageBuilder::new(&grammar, &parse_table);
        let (names, _) = builder.generate_symbol_names();

        // Should have EOF + 2 tokens
        assert_eq!(names.len(), 3);

        // Check that tokens are sorted by ID
        let code = quote! { #(#names)* }.to_string();

        // The token names are encoded as u8 byte arrays
        // "token1" = [116u8, 111u8, 107u8, 101u8, 110u8, 49u8, 0u8]
        // "token5" = [116u8, 111u8, 107u8, 101u8, 110u8, 53u8, 0u8]
        // We check for the distinguishing bytes: 49u8 for '1' and 53u8 for '5'
        assert!(code.contains("49u8")); // '1' in token1
        assert!(code.contains("53u8")); // '5' in token5
        let token1_pos = code.find("49u8").unwrap();
        let token5_pos = code.find("53u8").unwrap();
        assert!(token1_pos < token5_pos);
    }
}
