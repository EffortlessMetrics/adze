#![cfg_attr(feature = "strict_docs", allow(missing_docs))]
//! External scanner code generation for Tree-sitter.

use adze_ir::{ExternalToken, Grammar, SymbolId};
use quote::quote;
use std::collections::HashMap;

/// Generates external scanner data and interface for Tree-sitter
pub struct ExternalScannerGenerator {
    #[allow(dead_code)]
    grammar: Grammar,
    external_tokens: Vec<ExternalToken>,
    /// Maps symbol IDs to their indices in the external scanner
    #[allow(dead_code)]
    symbol_map: HashMap<SymbolId, usize>,
}

impl ExternalScannerGenerator {
    pub fn new(grammar: Grammar) -> Self {
        let external_tokens = grammar.externals.clone();
        let mut symbol_map = HashMap::new();

        for (index, token) in external_tokens.iter().enumerate() {
            symbol_map.insert(token.symbol_id, index);
        }

        Self {
            grammar,
            external_tokens,
            symbol_map,
        }
    }

    /// Generates the external scanner state bitmap
    /// Each state has a boolean array indicating which external tokens are valid
    pub fn generate_state_bitmap(&self, state_count: usize) -> Vec<Vec<bool>> {
        // For now, return a simple bitmap where all external tokens are valid in all states
        // TODO: This needs to be computed from the parse table
        let external_count = self.external_tokens.len();
        vec![vec![true; external_count]; state_count]
    }

    /// Generates the symbol map array that maps external scanner indices to symbol IDs
    pub fn generate_symbol_map(&self) -> Vec<u16> {
        let mut map = vec![0u16; self.external_tokens.len()];

        for (token_index, token) in self.external_tokens.iter().enumerate() {
            map[token_index] = token.symbol_id.0;
        }

        map
    }

    /// Generates the external scanner FFI interface code
    pub fn generate_scanner_interface(&self) -> proc_macro2::TokenStream {
        if self.external_tokens.is_empty() {
            return quote! {};
        }

        // Generate external scanner state data
        let state_bitmap = self.generate_state_bitmap(100); // TODO: Get actual state count
        let mut state_data = Vec::new();

        for state in &state_bitmap {
            for &valid in state {
                state_data.push(valid);
            }
        }

        // Generate symbol map
        let symbol_map = self.generate_symbol_map();

        quote! {
            // External scanner state bitmap
            static EXTERNAL_SCANNER_STATES: &[bool] = &[#(#state_data),*];

            // External scanner symbol map
            static EXTERNAL_SCANNER_SYMBOL_MAP: &[u16] = &[#(#symbol_map),*];

            // External scanner data
            #[allow(dead_code)]
            static EXTERNAL_SCANNER_DATA: adze::ffi::TSExternalScannerData = adze::ffi::TSExternalScannerData {
                states: EXTERNAL_SCANNER_STATES.as_ptr(),
                symbol_map: EXTERNAL_SCANNER_SYMBOL_MAP.as_ptr(),
                create: None, // TODO: Link to user scanner
                destroy: None,
                scan: None,
                serialize: None,
                deserialize: None,
            };
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_external_scanner_empty() {
        let grammar = Grammar::new("test".to_string());
        let generator = ExternalScannerGenerator::new(grammar);

        assert!(!generator.has_external_tokens());
        assert_eq!(generator.external_token_count(), 0);
        let interface = generator.generate_scanner_interface();
        assert_eq!(interface.to_string(), "");
    }

    #[test]
    fn test_external_scanner_with_tokens() {
        let mut grammar = Grammar::new("test".to_string());

        // Add some external tokens
        grammar.externals.push(ExternalToken {
            name: "HEREDOC".to_string(),
            symbol_id: SymbolId(100),
        });

        grammar.externals.push(ExternalToken {
            name: "TEMPLATE_STRING".to_string(),
            symbol_id: SymbolId(101),
        });

        let generator = ExternalScannerGenerator::new(grammar);

        assert!(generator.has_external_tokens());
        assert_eq!(generator.external_token_count(), 2);

        let symbol_map = generator.generate_symbol_map();
        assert_eq!(symbol_map, vec![100, 101]);

        let interface = generator.generate_scanner_interface();
        let interface_str = interface.to_string();
        assert!(interface_str.contains("EXTERNAL_SCANNER_STATES"));
        assert!(interface_str.contains("EXTERNAL_SCANNER_SYMBOL_MAP"));
        assert!(interface_str.contains("TSExternalScannerData"));
    }

    #[test]
    fn test_state_bitmap_generation() {
        let mut grammar = Grammar::new("test".to_string());

        grammar.externals.push(ExternalToken {
            name: "TOKEN1".to_string(),
            symbol_id: SymbolId(200),
        });

        grammar.externals.push(ExternalToken {
            name: "TOKEN2".to_string(),
            symbol_id: SymbolId(201),
        });

        let generator = ExternalScannerGenerator::new(grammar);
        let bitmap = generator.generate_state_bitmap(3); // 3 states

        assert_eq!(bitmap.len(), 3); // 3 states
        assert_eq!(bitmap[0].len(), 2); // 2 external tokens

        // Currently all tokens are valid in all states
        assert!(bitmap[0][0] && bitmap[0][1]);
        assert!(bitmap[1][0] && bitmap[1][1]);
        assert!(bitmap[2][0] && bitmap[2][1]);
    }
}
