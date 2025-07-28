// Lexer generation for pure-Rust parser
use proc_macro2::TokenStream;
use quote::quote;
use rust_sitter_ir::{Grammar, TokenPattern};

/// Generate a simple lexer function for the grammar
pub fn generate_lexer(grammar: &Grammar) -> TokenStream {
    // Collect all tokens and their patterns
    let mut token_matches = Vec::new();
    
    // The lexer needs to return symbols that match what the parser expects.
    // Symbol 0 is always EOF (end), handled by the parser itself.
    // Tokens start at symbol 1.
    
    // Sort tokens by ID for deterministic ordering
    let mut tokens: Vec<_> = grammar.tokens.iter().collect();
    tokens.sort_by_key(|(id, _)| id.0);
    
    for (symbol_id, token) in &tokens {
        match &token.pattern {
            TokenPattern::String(lit) => {
                // Single character or string literal
                if lit.len() == 1 {
                    let ch = lit.chars().next().unwrap();
                    let symbol = symbol_id.0 as u16;
                    token_matches.push(quote! {
                        if input[position] == #ch as u8 {
                            state.result_symbol = #symbol;
                            state.result_length = 1;
                            return true;
                        }
                    });
                } else {
                    let bytes = lit.as_bytes();
                    let len = bytes.len();
                    let symbol = symbol_id.0 as u16;
                    let byte_values = bytes.iter().map(|&b| b).collect::<Vec<_>>();
                    token_matches.push(quote! {
                        if position + #len <= input.len() && &input[position..position + #len] == &[#(#byte_values),*] {
                            state.result_symbol = #symbol;
                            state.result_length = #len;
                            return true;
                        }
                    });
                }
            }
            TokenPattern::Regex(pattern) => {
                // Handle common patterns
                if pattern == r"\d+" {
                    // For the arithmetic grammar, number token should be symbol 2
                    let symbol = symbol_id.0 as u16;
                    token_matches.push(quote! {
                        if input[position].is_ascii_digit() {
                            let mut len = 1;
                            while position + len < input.len() && input[position + len].is_ascii_digit() {
                                len += 1;
                            }
                            state.result_symbol = #symbol;
                            state.result_length = len;
                            return true;
                        }
                    });
                } else if pattern == r"\s" || pattern == r"\s+" {
                    // Whitespace is typically an extra token
                    let symbol = symbol_id.0 as u16;
                    token_matches.push(quote! {
                        if input[position].is_ascii_whitespace() {
                            let mut len = 1;
                            while position + len < input.len() && input[position + len].is_ascii_whitespace() {
                                len += 1;
                            }
                            state.result_symbol = #symbol;
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