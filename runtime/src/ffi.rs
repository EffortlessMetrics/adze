//! FFI types and functions for bridging between C and Rust interfaces.
#![cfg_attr(feature = "strict_docs", allow(missing_docs))]

use crate::external_scanner_ffi::TSLexer;

// Re-export types from external_scanner_ffi
pub use crate::external_scanner_ffi::TSExternalScannerData;

// Type alias for TSSymbol
pub type TSSymbol = u16;

/// Tree-sitter symbol metadata
#[repr(C)]
pub struct TSSymbolMetadata {
    pub visible: bool,
    pub named: bool,
    pub supertype: bool,
}

/// Tree-sitter parse action types
#[repr(C)]
pub enum TSParseActionType {
    Shift = 0,
    Reduce = 1,
    Accept = 2,
    Error = 3,
}

/// Tree-sitter parse action entry
#[repr(C)]
pub struct TSParseActionEntry {
    pub type_: TSParseActionType,
    pub state: u16,
    pub symbol: u16,
    pub child_count: u8,
    pub production_id: u8,
}

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
    /// Current lookahead character
    pub lookahead: u32,
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
    let mut initial_lookahead = 0u32;
    if position < length {
        unsafe {
            initial_lookahead = *input.add(position) as u32;
        }
    }

    let state = Box::new(LexerAdapterState {
        input,
        position,
        length,
        token_end: position,
        lookahead: initial_lookahead,
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
        context: state_ptr.cast(), // Store the lexer state as context
        result_symbol: 0,
    });
    let lexer_ptr = Box::into_raw(lexer);

    (lexer_ptr, state_ptr)
}

/// Clean up the lexer adapter
pub unsafe fn destroy_lexer_adapter(lexer: *mut TSLexer, state: *mut LexerAdapterState) {
    let state_ptr = if lexer.is_null() {
        state
    } else {
        unsafe { (*lexer).context as *mut LexerAdapterState }
    };

    if !lexer.is_null() {
        let _ = unsafe { Box::from_raw(lexer) };
    }
    if !state_ptr.is_null() {
        let _ = unsafe { Box::from_raw(state_ptr) };
    }
    if !state.is_null() && state != state_ptr {
        let _ = unsafe { Box::from_raw(state) };
    }
}

#[inline]
unsafe fn lexer_state(lexer: *mut TSLexer) -> *mut LexerAdapterState {
    unsafe { (*lexer).context as *mut LexerAdapterState }
}

#[inline]
unsafe fn lexer_state_const(lexer: *const TSLexer) -> *const LexerAdapterState {
    unsafe { (*lexer).context as *const LexerAdapterState }
}

// Callback functions for TSLexer

extern "C" fn ts_lexer_lookahead(lexer: *mut TSLexer) -> u32 {
    unsafe {
        let state_ptr = lexer_state(lexer);
        if state_ptr.is_null() {
            return 0;
        }
        let state = &*state_ptr;
        state.lookahead
    }
}

extern "C" fn ts_lexer_advance(lexer: *mut TSLexer, _skip: bool) {
    unsafe {
        let state_ptr = lexer_state(lexer);
        if state_ptr.is_null() {
            return;
        }
        let state = &mut *state_ptr;

        if state.position < state.length {
            state.position += 1;
            // Update the lookahead character in state
            if state.position < state.length {
                let byte = *state.input.add(state.position);
                state.lookahead = byte as u32;
            } else {
                state.lookahead = 0; // EOF
            }
        }
    }
}

extern "C" fn ts_lexer_mark_end(lexer: *mut TSLexer) {
    unsafe {
        let state_ptr = lexer_state(lexer);
        if state_ptr.is_null() {
            return;
        }
        let state = &mut *state_ptr;
        state.token_end = state.position;
    }
}

extern "C" fn ts_lexer_get_column(lexer: *mut TSLexer) -> u32 {
    unsafe {
        let state_ptr = lexer_state(lexer);
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
        let state_ptr = lexer_state_const(lexer);
        if state_ptr.is_null() {
            return true;
        }
        let state = &*state_ptr;
        state.position >= state.length
    }
}
