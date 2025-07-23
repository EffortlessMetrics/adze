// Enhanced external scanner generator with state-based validity computation
use rust_sitter_ir::{ExternalToken, Grammar, SymbolId};
use rust_sitter_glr_core::ParseTable;
use std::collections::{HashMap, HashSet};
use quote::quote;

/// Enhanced external scanner generator that computes state-based validity
pub struct ExternalScannerGenerator {
    grammar: Grammar,
    external_tokens: Vec<ExternalToken>,
    /// Maps symbol IDs to their indices in the external scanner
    symbol_map: HashMap<SymbolId, usize>,
    /// Parse table for computing valid external tokens
    parse_table: ParseTable,
}

impl ExternalScannerGenerator {
    pub fn new(grammar: Grammar, parse_table: ParseTable) -> Self {
        let external_tokens = grammar.externals.clone();
        let mut symbol_map = HashMap::new();
        
        for (index, token) in external_tokens.iter().enumerate() {
            symbol_map.insert(token.symbol_id, index);
        }
        
        Self {
            grammar,
            external_tokens,
            symbol_map,
            parse_table,
        }
    }
    
    /// Computes which external tokens are valid in each state
    pub fn compute_state_validity(&self) -> Vec<Vec<bool>> {
        let state_count = self.parse_table.state_count;
        let external_count = self.external_tokens.len();
        let mut state_bitmap = vec![vec![false; external_count]; state_count];
        
        // Build a set of external symbol IDs for quick lookup
        let external_symbols: HashSet<SymbolId> = self.external_tokens
            .iter()
            .map(|token| token.symbol_id)
            .collect();
        
        // For each state, check which external tokens can be shifted
        for state_index in 0..state_count {
            // Check each symbol's action in this state
            for (&symbol_id, &symbol_index) in &self.parse_table.symbol_to_index {
                // If this is an external symbol
                if external_symbols.contains(&symbol_id) {
                    if let Some(&external_index) = self.symbol_map.get(&symbol_id) {
                        // Check if there's a valid action for this symbol in this state
                        if symbol_index < self.parse_table.symbol_count &&
                           state_index < self.parse_table.action_table.len() &&
                           symbol_index < self.parse_table.action_table[state_index].len() {
                            let action = &self.parse_table.action_table[state_index][symbol_index];
                            // Any non-error action means the external token is valid
                            if !matches!(action, rust_sitter_glr_core::Action::Error) {
                                state_bitmap[state_index][external_index] = true;
                            }
                        }
                    }
                }
            }
        }
        
        state_bitmap
    }
    
    /// Generates the external scanner state bitmap with computed validity
    pub fn generate_state_bitmap(&self) -> Vec<Vec<bool>> {
        self.compute_state_validity()
    }
    
    /// Generates the symbol map array that maps external scanner indices to symbol IDs
    pub fn generate_symbol_map(&self) -> Vec<u16> {
        let mut map = vec![0u16; self.external_tokens.len()];
        
        for (token_index, token) in self.external_tokens.iter().enumerate() {
            map[token_index] = token.symbol_id.0 as u16;
        }
        
        map
    }
    
    /// Generates the external scanner FFI interface code
    pub fn generate_scanner_interface(&self) -> proc_macro2::TokenStream {
        if self.external_tokens.is_empty() {
            return quote! {};
        }
        
        // Generate external scanner state data with computed validity
        let state_bitmap = self.generate_state_bitmap();
        let mut state_data = Vec::new();
        
        for state in &state_bitmap {
            for &valid in state {
                state_data.push(valid);
            }
        }
        
        // Generate symbol map
        let symbol_map = self.generate_symbol_map();
        
        // Generate external token count and state count constants
        let external_count = self.external_tokens.len();
        let state_count = state_bitmap.len();
        
        quote! {
            // External scanner constants
            const EXTERNAL_TOKEN_COUNT: usize = #external_count;
            const STATE_COUNT: usize = #state_count;
            
            // External scanner state bitmap (computed from parse table)
            static EXTERNAL_SCANNER_STATES: &[bool] = &[#(#state_data),*];
            
            // External scanner symbol map
            static EXTERNAL_SCANNER_SYMBOL_MAP: &[u16] = &[#(#symbol_map),*];
            
            // External scanner data
            static EXTERNAL_SCANNER_DATA: ts::ffi::TSExternalScannerData = ts::ffi::TSExternalScannerData {
                states: EXTERNAL_SCANNER_STATES.as_ptr(),
                symbol_map: EXTERNAL_SCANNER_SYMBOL_MAP.as_ptr(),
                create: None, // TODO: Link to user scanner
                destroy: None,
                scan: None,
                serialize: None,
                deserialize: None,
            };
            
            // Helper function to get valid external tokens for a state
            #[allow(dead_code)]
            fn get_valid_external_tokens(state: usize) -> Vec<bool> {
                if state >= STATE_COUNT {
                    return vec![false; EXTERNAL_TOKEN_COUNT];
                }
                
                let start = state * EXTERNAL_TOKEN_COUNT;
                let end = start + EXTERNAL_TOKEN_COUNT;
                EXTERNAL_SCANNER_STATES[start..end].to_vec()
            }
        }
    }
    
    /// Returns whether the grammar has external tokens
    pub fn has_external_tokens(&self) -> bool {
        !self.external_tokens.is_empty()
    }
    
    /// Returns the number of external tokens
    pub fn external_token_count(&self) -> usize {
        self.external_tokens.len()
    }
    
    /// Debug helper: print validity matrix
    pub fn debug_print_validity(&self) {
        let state_bitmap = self.compute_state_validity();
        
        println!("External Token Validity Matrix:");
        println!("States x External Tokens");
        
        // Print header with external token names
        print!("State |");
        for token in &self.external_tokens {
            print!(" {} |", token.name);
        }
        println!();
        
        // Print validity for each state
        for (state_idx, state_validity) in state_bitmap.iter().enumerate() {
            print!("{:5} |", state_idx);
            for &valid in state_validity {
                print!(" {:5} |", if valid { "✓" } else { " " });
            }
            println!();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_sitter_glr_core::Action;
    
    #[test]
    fn test_state_validity_computation() {
        let mut grammar = Grammar::new("test".to_string());
        
        // Add external tokens
        grammar.externals.push(ExternalToken {
            name: "INDENT".to_string(),
            symbol_id: SymbolId(100),
        });
        grammar.externals.push(ExternalToken {
            name: "DEDENT".to_string(),
            symbol_id: SymbolId(101),
        });
        
        // Create a simple parse table
        let mut parse_table = ParseTable {
            action_table: vec![vec![Action::Error; 2]; 2], // 2 states, 2 symbols
            goto_table: vec![vec![rust_sitter_ir::StateId(0); 2]; 2],
            symbol_metadata: vec![],
            state_count: 2,
            symbol_count: 2,
            symbol_to_index: HashMap::new(),
        };
        
        // Map external symbols to indices
        parse_table.symbol_to_index.insert(SymbolId(100), 0); // INDENT
        parse_table.symbol_to_index.insert(SymbolId(101), 1); // DEDENT
        
        // State 0: INDENT is valid (shift to state 1)
        parse_table.action_table[0][0] = Action::Shift(rust_sitter_ir::StateId(1));
        
        // State 1: DEDENT is valid (shift to state 2)
        parse_table.action_table[1][1] = Action::Shift(rust_sitter_ir::StateId(2));
        
        let generator = ExternalScannerGenerator::new(grammar, parse_table);
        let validity = generator.compute_state_validity();
        
        // Check state 0: only INDENT should be valid
        assert_eq!(validity[0], vec![true, false]);
        
        // Check state 1: only DEDENT should be valid
        assert_eq!(validity[1], vec![false, true]);
    }
    
    #[test]
    fn test_symbol_map_generation() {
        let mut grammar = Grammar::new("test".to_string());
        
        grammar.externals.push(ExternalToken {
            name: "TOKEN1".to_string(),
            symbol_id: SymbolId(200),
        });
        grammar.externals.push(ExternalToken {
            name: "TOKEN2".to_string(),
            symbol_id: SymbolId(201),
        });
        
        let parse_table = ParseTable::new();
        let generator = ExternalScannerGenerator::new(grammar, parse_table);
        
        let symbol_map = generator.generate_symbol_map();
        assert_eq!(symbol_map, vec![200, 201]);
    }
}