//! FFI types and functions for bridging between C and Rust interfaces

use core::ffi::c_void;
use crate::external_scanner_ffi::TSLexer;

/// Runtime state for the lexer adapter
pub struct LexerAdapterState {
    /// Input buffer
    pub input: *const u8,
    /// Current position in the input
    pub position: usize,
    /// Length of the input
    pub length: usize,
    /// End position of the current token
    pub token_end: usize,
}

/// Create a lexer adapter for use in scan functions
/// 
/// This function creates a TSLexer struct that the external scanner can use
/// to read input and mark token boundaries.
pub unsafe fn create_lexer_adapter(
    input: *const u8,
    position: usize,
    length: usize,
) -> (*mut TSLexer, *mut LexerAdapterState) {
    // Create the adapter state
    let state = Box::new(LexerAdapterState {
        input,
        position,
        length,
        token_end: position,
    });
    let state_ptr = Box::into_raw(state);
    
    // Create the TSLexer struct with function pointers
    let lexer = Box::new(TSLexer {
        lookahead: ts_lexer_lookahead,
        advance: ts_lexer_advance,
        mark_end: ts_lexer_mark_end,
        get_column: ts_lexer_get_column,
        is_at_included_range_start: ts_lexer_is_at_included_range_start,
        eof: ts_lexer_eof,
        result_symbol: 0,
    });
    let lexer_ptr = Box::into_raw(lexer);
    
    // Store the state pointer in a way that the callback functions can access it
    // For simplicity, we'll use the lexer pointer + 1 to store the state pointer
    let state_storage = lexer_ptr.add(1) as *mut *mut LexerAdapterState;
    *state_storage = state_ptr;
    
    (lexer_ptr, state_ptr)
}

/// Clean up the lexer adapter
pub unsafe fn destroy_lexer_adapter(
    lexer: *mut TSLexer,
    state: *mut LexerAdapterState,
) {
    if !lexer.is_null() {
        let _ = Box::from_raw(lexer);
    }
    if !state.is_null() {
        let _ = Box::from_raw(state);
    }
}

// Callback functions for TSLexer

extern "C" fn ts_lexer_lookahead(lexer: *mut TSLexer) -> u32 {
    unsafe {
        let state_ptr = *(lexer.add(1) as *mut *mut LexerAdapterState);
        if state_ptr.is_null() {
            return 0;
        }
        let state = &*state_ptr;
        
        if state.position >= state.length {
            return 0; // EOF
        }
        
        // Get the current byte and convert to UTF-32
        let byte = *state.input.add(state.position);
        // For simplicity, just return the byte as-is (assumes ASCII/UTF-8)
        byte as u32
    }
}

extern "C" fn ts_lexer_advance(lexer: *mut TSLexer, _skip: bool) {
    unsafe {
        let state_ptr = *(lexer.add(1) as *mut *mut LexerAdapterState);
        if state_ptr.is_null() {
            return;
        }
        let state = &mut *state_ptr;
        
        if state.position < state.length {
            state.position += 1;
            // Update the lookahead character
            if state.position < state.length {
                let byte = *state.input.add(state.position);
                (*lexer).lookahead = byte as u32;
            } else {
                (*lexer).lookahead = 0; // EOF
            }
        }
    }
}

extern "C" fn ts_lexer_mark_end(lexer: *mut TSLexer) {
    unsafe {
        let state_ptr = *(lexer.add(1) as *mut *mut LexerAdapterState);
        if state_ptr.is_null() {
            return;
        }
        let state = &mut *state_ptr;
        state.token_end = state.position;
    }
}

extern "C" fn ts_lexer_get_column(lexer: *mut TSLexer) -> u32 {
    unsafe {
        let state_ptr = *(lexer.add(1) as *mut *mut LexerAdapterState);
        if state_ptr.is_null() {
            return 0;
        }
        let state = &*state_ptr;
        
        // Count columns from the beginning of the current line
        let mut column = 0;
        let mut pos = state.position;
        
        // Go back to find the start of the line
        while pos > 0 {
            pos -= 1;
            let byte = *state.input.add(pos);
            if byte == b'\n' {
                break;
            }
            column += 1;
        }
        
        column
    }
}

extern "C" fn ts_lexer_is_at_included_range_start(_lexer: *const TSLexer) -> bool {
    // We don't support included ranges yet
    false
}

extern "C" fn ts_lexer_eof(lexer: *const TSLexer) -> bool {
    unsafe {
        let state_ptr = *(lexer.add(1) as *const *const LexerAdapterState);
        if state_ptr.is_null() {
            return true;
        }
        let state = &*state_ptr;
        state.position >= state.length
    }
}