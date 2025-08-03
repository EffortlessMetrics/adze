// Lexer generation for pure-Rust parser
use proc_macro2::TokenStream;
use quote::quote;
use rust_sitter_ir::{Grammar, SymbolId, TokenPattern};
use std::collections::BTreeMap;

/// Generate a simple lexer function for the grammar
pub fn generate_lexer(
    grammar: &Grammar,
    symbol_to_index: &BTreeMap<SymbolId, usize>,
) -> TokenStream {
    // The lexer needs to return symbol indices that match the parse table.
    // We use the symbol_to_index mapping from the parse table to ensure consistency.

    // Collect all tokens and categorize them
    let mut keywords = Vec::new();
    let mut other_strings = Vec::new();
    let mut regex_patterns = Vec::new();
    let mut identifier_pattern = None;
    
    for (id, token) in &grammar.tokens {
        if let Some(&idx) = symbol_to_index.get(id) {
            let symbol_index = idx as u16;
            match &token.pattern {
                TokenPattern::String(s) => {
                    // Check if it's a keyword (all alphabetic characters)
                    if s.chars().all(|c| c.is_ascii_alphabetic() || c == '_') && s.len() > 1 {
                        keywords.push((symbol_index, s.clone()));
                    } else {
                        other_strings.push((symbol_index, s.clone()));
                    }
                }
                TokenPattern::Regex(pattern) => {
                    if pattern == r"[a-zA-Z_][a-zA-Z0-9_]*" {
                        identifier_pattern = Some(symbol_index);
                    } else {
                        regex_patterns.push((symbol_index, pattern.clone()));
                    }
                }
            }
        }
    }
    
    // Sort keywords by length (longest first) to match longer keywords before shorter ones
    keywords.sort_by(|a, b| b.1.len().cmp(&a.1.len()));
    
    let mut token_matches = Vec::new();
    
    // First: Add keyword matching (before identifier pattern)
    for (symbol_index, keyword) in keywords {
        let bytes = keyword.as_bytes();
        let len = bytes.len();
        let byte_values = bytes.iter().copied().collect::<Vec<_>>();
        token_matches.push(quote! {
            if position + #len <= input.len() && 
               &input[position..position + #len] == &[#(#byte_values),*] &&
               (position + #len >= input.len() || 
                (!input[position + #len].is_ascii_alphanumeric() && input[position + #len] != b'_')) {
                state.result_symbol = #symbol_index;
                state.result_length = #len;
                return true;
            }
        });
    }
    
    // Second: Add other string patterns (operators, punctuation)
    for (symbol_index, s) in other_strings {
        if s.len() == 1 {
            let ch = s.chars().next().unwrap();
            token_matches.push(quote! {
                if input[position] == #ch as u8 {
                    state.result_symbol = #symbol_index;
                    state.result_length = 1;
                    return true;
                }
            });
        } else {
            let bytes = s.as_bytes();
            let len = bytes.len();
            let byte_values = bytes.iter().copied().collect::<Vec<_>>();
            token_matches.push(quote! {
                if position + #len <= input.len() && &input[position..position + #len] == &[#(#byte_values),*] {
                    state.result_symbol = #symbol_index;
                    state.result_length = #len;
                    return true;
                }
            });
        }
    }
    
    // Third: Add regex patterns (except identifier)
    for (symbol_index, pattern) in regex_patterns {
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
        } else if pattern == r"\s" || pattern == r"\s+" || pattern == r"\s*" {
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
        } else if pattern == r"\d+(\.\d+)?" {
            // Number with optional decimal
            token_matches.push(quote! {
                if input[position].is_ascii_digit() {
                    let mut len = 1;
                    // Match initial digits
                    while position + len < input.len() && input[position + len].is_ascii_digit() {
                        len += 1;
                    }
                    // Check for optional decimal part
                    if position + len + 1 < input.len() && input[position + len] == b'.' && input[position + len + 1].is_ascii_digit() {
                        len += 2; // Skip '.' and first decimal digit
                        while position + len < input.len() && input[position + len].is_ascii_digit() {
                            len += 1;
                        }
                    }
                    state.result_symbol = #symbol_index;
                    state.result_length = len;
                    return true;
                }
            });
        } else if pattern == r#""[^"]*"|'[^']*'"# {
            // String literal pattern (double or single quotes)
            token_matches.push(quote! {
                if input[position] == b'"' || input[position] == b'\'' {
                    let quote_char = input[position];
                    let mut len = 1;
                    while position + len < input.len() && input[position + len] != quote_char {
                        len += 1;
                    }
                    if position + len < input.len() && input[position + len] == quote_char {
                        len += 1; // Include closing quote
                        state.result_symbol = #symbol_index;
                        state.result_length = len;
                        return true;
                    }
                }
            });
        }
        // TODO: Add more pattern support
    }
    
    // Fourth: Add identifier pattern last (after all keywords have been checked)
    if let Some(symbol_index) = identifier_pattern {
        token_matches.push(quote! {
            if input[position].is_ascii_alphabetic() || input[position] == b'_' {
                let mut len = 1;
                while position + len < input.len() && 
                      (input[position + len].is_ascii_alphanumeric() || input[position + len] == b'_') {
                    len += 1;
                }
                state.result_symbol = #symbol_index;
                state.result_length = len;
                return true;
            }
        });
    }

    quote! {
        unsafe extern "C" fn lexer_fn(state_ptr: *mut ::std::ffi::c_void, _lex_mode: TSLexState) -> bool {
            // SAFETY: state_ptr is guaranteed to be a valid pointer to LexerState by the Tree-sitter runtime
            let state = unsafe { &mut *(state_ptr as *mut LexerState) };
            // SAFETY: input pointer and length are provided by Tree-sitter runtime and guaranteed to be valid
            let input = unsafe { std::slice::from_raw_parts(state.input, state.input_len) };
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
