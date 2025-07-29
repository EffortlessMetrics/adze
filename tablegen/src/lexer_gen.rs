// Lexer generation for pure-Rust parser
use proc_macro2::TokenStream;
use quote::quote;
use rust_sitter_ir::{Grammar, TokenPattern, SymbolId};
use std::collections::HashMap;

/// Generate a simple lexer function for the grammar
pub fn generate_lexer(grammar: &Grammar, symbol_to_index: &HashMap<SymbolId, usize>) -> TokenStream {
    // Collect all tokens and their patterns
    let mut token_matches = Vec::new();
    
    // The lexer needs to return symbol indices that match the parse table.
    // We use the symbol_to_index mapping from the parse table to ensure consistency.
    
    // Sort tokens by their parse table index for deterministic ordering
    let mut tokens: Vec<_> = grammar.tokens.iter()
        .filter_map(|(id, token)| {
            symbol_to_index.get(id).map(|&idx| (idx, id, token))
        })
        .collect();
    tokens.sort_by_key(|(idx, _, _)| *idx);
    
    // Generate token matches for each token
    for (idx, token_id, token) in &tokens {
        let symbol_index = *idx as u16;
        match &token.pattern {
            TokenPattern::String(lit) => {
                // Single character or string literal
                if lit.len() == 1 {
                    let ch = lit.chars().next().unwrap();
                    token_matches.push(quote! {
                        if input[position] == #ch as u8 {
                            state.result_symbol = #symbol_index;
                            state.result_length = 1;
                            return true;
                        }
                    });
                } else {
                    let bytes = lit.as_bytes();
                    let len = bytes.len();
                    let byte_values = bytes.iter().map(|&b| b).collect::<Vec<_>>();
                    token_matches.push(quote! {
                        if position + #len <= input.len() && &input[position..position + #len] == &[#(#byte_values),*] {
                            state.result_symbol = #symbol_index;
                            state.result_length = #len;
                            return true;
                        }
                    });
                }
            }
            TokenPattern::Regex(pattern) => {
                // Handle common patterns
                if pattern == r"\d+" {
                    token_matches.push(quote! {
                        if input[position].is_ascii_digit() {
                            let mut len = 1;
                            while position + len < input.len() && input[position + len].is_ascii_digit() {
                                len += 1;
                            }
                            state.result_symbol = #symbol_index;
                            state.result_length = len;
                            return true;
                        }
                    });
                } else if pattern == r"\s" || pattern == r"\s+" {
                    // Whitespace is typically an extra token
                    token_matches.push(quote! {
                        if input[position].is_ascii_whitespace() {
                            let mut len = 1;
                            while position + len < input.len() && input[position + len].is_ascii_whitespace() {
                                len += 1;
                            }
                            state.result_symbol = #symbol_index;
                            state.result_length = len;
                            return true;
                        }
                    });
                }
                // TODO: Add more pattern support
            }
        }
    }
    
    quote! {
        unsafe extern "C" fn lexer_fn(state_ptr: *mut ::std::ffi::c_void, _lex_mode: TSLexState) -> bool {
            let state = &mut *(state_ptr as *mut LexerState);
            let input = std::slice::from_raw_parts(state.input, state.input_len);
            let position = state.position;
            
            if position >= input.len() {
                return false;
            }
            
            #(#token_matches)*
            
            // No match found
            false
        }
        
        #[repr(C)]
        struct LexerState {
            input: *const u8,
            input_len: usize,
            position: usize,
            point_row: u32,
            point_column: u32,
            result_symbol: u16,
            result_length: usize,
        }
    }
}