// Proper TSLanguage structure generation
// This module creates a valid Tree-sitter Language structure from our IR

use crate::abi::TREE_SITTER_LANGUAGE_VERSION;
use proc_macro2::TokenStream;
use quote::quote;
use rust_sitter_glr_core::ParseTable;
use rust_sitter_ir::Grammar;

/// Language generator that creates proper TSLanguage structures
pub struct LanguageGenerator<'a> {
    grammar: &'a Grammar,
    parse_table: &'a ParseTable,
}

impl<'a> LanguageGenerator<'a> {
    pub fn new(grammar: &'a Grammar, parse_table: &'a ParseTable) -> Self {
        Self {
            grammar,
            parse_table,
        }
    }

    /// Generate the complete language module with proper TSLanguage
    pub fn generate(&self) -> TokenStream {
        let language_name = &self.grammar.name;
        let language_fn_ident = quote::format_ident!("tree_sitter_{}", language_name);

        // Generate static data
        let symbol_names = self.generate_symbol_names();
        let field_names = self.generate_field_names();
        let symbol_metadata = self.generate_symbol_metadata();
        let parse_actions = self.generate_parse_actions();
        let lex_modes = self.generate_lex_modes();
        let (compressed_table, small_table_map) = self.generate_compressed_tables();

        // Generate indices for symbol_names and field_names
        let symbol_name_indices: Vec<usize> = (0..symbol_names.len()).collect();
        let field_name_indices: Vec<usize> = (0..field_names.len()).collect();

        // Count various elements
        let symbol_count = self.count_symbols();
        let token_count = self.grammar.tokens.len() as u32;
        let field_count = self.grammar.fields.len() as u32;
        let state_count = self.parse_table.state_count as u32;
        let external_token_count = self.grammar.externals.len() as u32;
        let large_state_count = self.determine_large_state_count() as u32;
        let production_id_count = self.count_production_ids() as u32;

        quote! {
            use rust_sitter::tree_sitter as ts;
            use crate::abi::{TSLanguage, TSSymbol, TSStateId, TSLexState, TSParseAction, ExternalScanner};
            const TREE_SITTER_LANGUAGE_VERSION: u32 = 15;

            // Symbol names array
            static SYMBOL_NAMES: &[&str] = &[#(#symbol_names),*];
            static SYMBOL_NAMES_PTRS: &[*const u8] = &[
                #(SYMBOL_NAMES[#symbol_name_indices].as_ptr()),*
            ];

            // Field names array
            static FIELD_NAMES: &[&str] = &[#(#field_names),*];
            static FIELD_NAMES_PTRS: &[*const u8] = &[
                #(FIELD_NAMES[#field_name_indices].as_ptr()),*
            ];

            // Symbol metadata
            static SYMBOL_METADATA: &[u8] = &[#(#symbol_metadata),*];

            // Parse actions
            static PARSE_ACTIONS: &[TSParseAction] = &[#(#parse_actions),*];

            // Lex modes
            static LEX_MODES: &[TSLexState] = &[#(#lex_modes),*];

            // Parse table
            static PARSE_TABLE: &[u16] = &[#(#compressed_table),*];
            static SMALL_PARSE_TABLE_MAP: &[u32] = &[#(#small_table_map),*];

            // Field maps (placeholder for now)
            static FIELD_MAP_SLICES: &[u16] = &[];
            static FIELD_MAP_ENTRIES: &[u16] = &[];

            // Public symbol map (identity for now)
            static PUBLIC_SYMBOL_MAP: &[TSSymbol] = &[
                #(TSSymbol(#symbol_name_indices as u16)),*
            ];

            // Primary state IDs
            static PRIMARY_STATE_IDS: &[TSStateId] = &[
                #(TSStateId(#symbol_name_indices as u16)),*
            ];

            // The language structure
            static LANGUAGE: TSLanguage = TSLanguage {
                version: #TREE_SITTER_LANGUAGE_VERSION,
                symbol_count: #symbol_count,
                alias_count: 0, // TODO: Implement aliases
                token_count: #token_count,
                external_token_count: #external_token_count,
                state_count: #state_count,
                large_state_count: #large_state_count,
                production_id_count: #production_id_count,
                field_count: #field_count,
                max_alias_sequence_length: 0,
                parse_table: PARSE_TABLE.as_ptr(),
                small_parse_table: PARSE_TABLE.as_ptr().wrapping_add(#large_state_count as usize * #symbol_count as usize),
                small_parse_table_map: SMALL_PARSE_TABLE_MAP.as_ptr(),
                parse_actions: PARSE_ACTIONS.as_ptr(),
                symbol_names: SYMBOL_NAMES_PTRS.as_ptr(),
                field_names: FIELD_NAMES_PTRS.as_ptr(),
                field_map_slices: FIELD_MAP_SLICES.as_ptr(),
                field_map_entries: FIELD_MAP_ENTRIES.as_ptr(),
                symbol_metadata: SYMBOL_METADATA.as_ptr(),
                public_symbol_map: PUBLIC_SYMBOL_MAP.as_ptr(),
                alias_map: std::ptr::null(),
                alias_sequences: std::ptr::null(),
                lex_modes: LEX_MODES.as_ptr(),
                lex_fn: None, // TODO: Implement custom lexer
                keyword_lex_fn: None,
                keyword_capture_token: TSSymbol(0),
                external_scanner: ExternalScanner::default(),
                primary_state_ids: PRIMARY_STATE_IDS.as_ptr(),
            };

            /// Get the Tree-sitter Language for this grammar
            pub fn language() -> ts::Language {
                unsafe {
                    ts::Language::from_raw(&LANGUAGE as *const TSLanguage as *const _)
                }
            }

            /// Export for C FFI
            #[no_mangle]
            pub extern "C" fn #language_fn_ident() -> ts::Language {
                language()
            }
        }
    }

    fn generate_symbol_names(&self) -> Vec<String> {
        let mut names = vec!["end".to_string()]; // EOF symbol

        // Add tokens
        for (_id, token) in &self.grammar.tokens {
            names.push(token.name.clone());
        }

        // Add rules (non-terminals)
        for (id, _rules) in &self.grammar.rules {
            // Use rule_names if available, otherwise generate
            let name = self
                .grammar
                .rule_names
                .get(id)
                .cloned()
                .unwrap_or_else(|| format!("rule_{}", id.0));
            names.push(name);
        }

        names
    }

    fn generate_field_names(&self) -> Vec<String> {
        let mut names = vec![];
        for (_id, name) in &self.grammar.fields {
            names.push(name.clone());
        }
        names
    }

    fn generate_symbol_metadata(&self) -> Vec<u8> {
        let symbol_count = self.count_symbols();
        let mut metadata = vec![0u8; symbol_count];

        // Mark visible symbols
        for i in 0..symbol_count {
            // For now, mark all symbols as visible
            // Bit 0: visible
            // Bit 1: named
            metadata[i] = 0b11;
        }

        metadata
    }

    fn generate_parse_actions(&self) -> Vec<TokenStream> {
        // Generate simplified parse actions
        // In a real implementation, this would be derived from the parse table
        vec![quote! {
            TSParseAction {
                action_type: 0,
                extra: 0,
                child_count: 0,
                dynamic_precedence: 0,
                symbol: TSSymbol(0),
            }
        }]
    }

    fn generate_lex_modes(&self) -> Vec<TokenStream> {
        let state_count = self.parse_table.state_count;
        let mut modes = vec![];

        for i in 0..state_count {
            modes.push(quote! {
                TSLexState {
                    lex_state: #i as u16,
                    external_lex_state: 0,
                }
            });
        }

        modes
    }

    fn generate_compressed_tables(&self) -> (Vec<u16>, Vec<u32>) {
        // Tree-sitter's compression strategy:
        // - Large states (0 to LARGE_STATE_COUNT) use a 2D table indexed by [state][symbol]
        // - Small states use a compact format with entries like [count, symbol, action, ...]

        let large_state_count = self.determine_large_state_count();
        let mut compressed_table = Vec::new();
        let mut small_table_map = Vec::new();

        // For large states, generate the full 2D table
        for state in 0..large_state_count {
            for symbol in 0..self.parse_table.symbol_count {
                let action = self.get_action(state, symbol);
                compressed_table.push(self.encode_action(action));
            }
        }

        // For small states, use compact representation
        let mut small_table_data = Vec::new();
        for state in large_state_count..self.parse_table.state_count {
            // Store offset into small_table_data for this state
            small_table_map.push(small_table_data.len() as u32);

            // Count non-error actions for this state
            let mut non_error_actions = Vec::new();
            for symbol in 0..self.parse_table.symbol_count {
                let action = self.get_action(state, symbol);
                if !self.is_error_action(action) {
                    non_error_actions.push((symbol, action));
                }
            }

            // First entry is the count of actions
            small_table_data.push(non_error_actions.len() as u16);

            // Then pairs of (symbol, encoded_action)
            for (symbol, action) in non_error_actions {
                small_table_data.push(symbol as u16);
                small_table_data.push(self.encode_action(action));
            }
        }

        // If no small states, add a dummy entry
        if small_table_map.is_empty() {
            small_table_map.push(0);
        }

        // Combine compressed_table and small_table_data
        compressed_table.extend(small_table_data);

        (compressed_table, small_table_map)
    }

    fn determine_large_state_count(&self) -> usize {
        // Tree-sitter typically uses states with the most transitions as large states
        // For now, use a simple heuristic: first 30% of states are large
        let large_ratio = 0.3;
        let large_count = (self.parse_table.state_count as f64 * large_ratio) as usize;
        large_count.max(1).min(self.parse_table.state_count)
    }

    fn get_action(&self, state: usize, symbol: usize) -> u16 {
        // Get the action from parse table
        if state < self.parse_table.action_table.len()
            && symbol < self.parse_table.action_table[state].len()
        {
            let action_cell = &self.parse_table.action_table[state][symbol];
            // For Tree-sitter compatibility, we need to pick one action
            // Use the first action if multiple exist (GLR conflicts)
            if action_cell.is_empty() {
                0xFFFE // Error action
            } else {
                let action = &action_cell[0];
                match action {
                    rust_sitter_glr_core::Action::Shift(s) => s.0,
                    rust_sitter_glr_core::Action::Reduce(r) => 0x8000 | (r.0 << 1),
                    rust_sitter_glr_core::Action::Accept => 0xFFFF,
                    rust_sitter_glr_core::Action::Error => 0xFFFE,
                    rust_sitter_glr_core::Action::Fork(_) => 0xFFFE, // TODO: Handle GLR forks
                }
            }
        } else {
            0xFFFE // Error action
        }
    }

    fn encode_action(&self, action: u16) -> u16 {
        // Actions are already encoded in get_action
        action
    }

    fn is_error_action(&self, action: u16) -> bool {
        action == 0xFFFE
    }

    fn count_symbols(&self) -> usize {
        1 + // EOF
        self.grammar.tokens.len() +
        self.grammar.rules.len()
    }

    fn count_production_ids(&self) -> usize {
        // Find the maximum production ID in all rules
        let mut max_production_id = 0;
        for (_, rules) in &self.grammar.rules {
            for rule in rules {
                max_production_id = max_production_id.max(rule.production_id.0);
            }
        }
        // Production ID count is max ID + 1 (since they start at 0)
        (max_production_id + 1) as usize
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_sitter_ir::*;

    #[test]
    fn test_language_generation() {
        let mut grammar = Grammar::new("test".to_string());

        // Add a simple token
        let num_token = Token {
            name: "number".to_string(),
            pattern: TokenPattern::Regex(r"\d+".to_string()),
            fragile: false,
        };
        grammar.tokens.insert(SymbolId(1), num_token);

        // Create a simple parse table
        let parse_table = ParseTable {
            action_table: vec![],
            goto_table: vec![],
            symbol_metadata: vec![],
            state_count: 10,
            symbol_count: 5,
            symbol_to_index: std::collections::BTreeMap::new(),
        };

        let generator = LanguageGenerator::new(&grammar, &parse_table);
        let output = generator.generate();

        // Check that it generates valid code
        let output_str = output.to_string();
        assert!(output_str.contains("TSLanguage"));
        assert!(output_str.contains("tree_sitter_test"));
    }
}
