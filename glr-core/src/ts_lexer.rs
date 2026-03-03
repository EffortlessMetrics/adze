//! Tree-sitter FFI lexer wrapper for calling grammar's lex_fn
#![cfg_attr(feature = "strict_docs", allow(missing_docs))]

use crate::LexMode;
use std::ffi::c_void;
use std::os::raw::c_char;

/// Tree-sitter lexer struct passed to lex function
#[repr(C)]
pub struct TSLexer {
    pub lookahead: i32,
    pub result_symbol: u16,
    pub eof: extern "C" fn(payload: *mut c_void) -> bool,
    pub advance: extern "C" fn(payload: *mut c_void, is_skipped: bool),
    pub mark_end: extern "C" fn(payload: *mut c_void),
    pub get_column: extern "C" fn(payload: *mut c_void) -> u32,
    pub is_included: extern "C" fn(payload: *mut c_void) -> bool,
    pub payload: *mut c_void,
}

/// Function pointer type for lexer functions
pub type LexFn = unsafe extern "C" fn(lexer: *mut TSLexer, state: u16) -> bool;

/// Tree-sitter lex mode struct
#[repr(C)]
pub struct TSLexMode {
    pub lex_state: u16,
    pub external_lex_state: u16,
}

/// Tree-sitter language struct (partial - only fields we need)
#[repr(C)]
pub struct TSLanguage {
    pub version: u32,
    pub symbol_count: u32,
    pub alias_count: u32,
    pub token_count: u32,
    pub external_token_count: u32,
    pub state_count: u32,
    pub large_state_count: u32,
    pub production_id_count: u32,
    pub field_count: u32,
    pub max_alias_sequence_length: u16,
    // Skip parse table pointers we don't need
    _parse_table: *const u16,
    _small_parse_table: *const u16,
    _small_parse_table_map: *const u32,
    _parse_actions: *const c_void,
    _symbol_names: *const *const c_char,
    _field_names: *const *const c_char,
    _field_map_slices: *const c_void,
    _field_map_entries: *const c_void,
    _symbol_metadata: *const c_void,
    _public_symbol_map: *const u16,
    _alias_map: *const u16,
    _alias_sequences: *const u16,
    /// Lex modes for each state
    pub lex_modes: *const TSLexMode,
    /// Lex function
    pub lex_fn: Option<LexFn>,
    /// Keyword lex function
    pub keyword_lex_fn: Option<LexFn>,
    /// Capture function for keywords
    pub keyword_capture_token: u16,
    /// External scanner functions
    pub external_scanner: ExternalScanner,
}

/// External scanner functions
#[repr(C)]
pub struct ExternalScanner {
    pub states: *const bool,
    pub symbol_map: *const u16,
    pub create: Option<extern "C" fn() -> *mut c_void>,
    pub destroy: Option<extern "C" fn(scanner: *mut c_void)>,
    pub scan: Option<
        extern "C" fn(
            scanner: *mut c_void,
            lexer: *mut TSLexer,
            valid_symbols: *const bool,
        ) -> bool,
    >,
    pub serialize: Option<extern "C" fn(scanner: *mut c_void, buffer: *mut c_char) -> u32>,
    pub deserialize:
        Option<extern "C" fn(scanner: *mut c_void, buffer: *const c_char, length: u32)>,
}

/// Token produced by the lexer
#[derive(Debug, Clone, Copy)]
pub struct NextToken {
    pub kind: u32,
    pub start: u32,
    pub end: u32,
}

/// Host struct for callbacks from the C lexer
pub struct TsLexerHost<'a> {
    input: &'a [u8],
    pos: usize,
    end_mark: usize,
}

impl<'a> TsLexerHost<'a> {
    // C callbacks — invoked by the Tree-sitter lex_fn during `GrammarLexer::next()`.
    // SAFETY (shared across eof/advance/mark_end): `payload` was set to a valid
    // `&mut TsLexerHost` pointer in `GrammarLexer::next()` and these callbacks are
    // only called synchronously by the C lex_fn during that call, so the pointer is
    // valid and exclusively borrowed for the duration.
    extern "C" fn eof(payload: *mut c_void) -> bool {
        // SAFETY: see shared invariant above.
        let host = unsafe { &mut *(payload as *mut Self) };
        host.pos >= host.input.len()
    }

    extern "C" fn advance(payload: *mut c_void, skip: bool) {
        // SAFETY: see shared invariant above.
        let host = unsafe { &mut *(payload as *mut Self) };
        if host.pos < host.input.len() {
            host.pos += 1;
            if !skip {
                host.end_mark = host.pos;
            }
        }
    }

    extern "C" fn mark_end(payload: *mut c_void) {
        // SAFETY: see shared invariant above.
        let host = unsafe { &mut *(payload as *mut Self) };
        host.end_mark = host.pos;
    }

    extern "C" fn get_column(_payload: *mut c_void) -> u32 {
        0 // TODO: Track column for proper error reporting
    }

    extern "C" fn is_included(_payload: *mut c_void) -> bool {
        false // TODO: Support included ranges for injections
    }
}

/// Grammar lexer that calls the compiled Tree-sitter lex function
pub struct GrammarLexer {
    lang: *const TSLanguage,
}

impl GrammarLexer {
    /// Create a lexer for a specific Tree-sitter language
    ///
    /// # Safety
    ///
    /// `lang` must be a valid, non-null pointer to a live [`TSLanguage`]
    /// from Tree-sitter. It must remain valid for the lifetime of the
    /// returned wrapper. Passing an invalid pointer or one that outlives
    /// the wrapper is undefined behavior.
    pub unsafe fn new(lang: *const TSLanguage) -> Self {
        Self { lang }
    }

    /// Get the next token from the input
    pub fn next(
        &self,
        input: &str,
        pos: usize,
        mode: LexMode,
        _valid_symbols: &[bool], // TODO: Use for external scanner
    ) -> Option<NextToken> {
        let mut host = TsLexerHost {
            input: input.as_bytes(),
            pos,
            end_mark: pos,
        };

        // Update lookahead
        let lookahead = if pos < host.input.len() {
            host.input[pos] as i32
        } else {
            0 // EOF
        };

        let mut c_lexer = TSLexer {
            lookahead,
            result_symbol: 0,
            eof: TsLexerHost::eof,
            advance: TsLexerHost::advance,
            mark_end: TsLexerHost::mark_end,
            get_column: TsLexerHost::get_column,
            is_included: TsLexerHost::is_included,
            payload: &mut host as *mut _ as *mut _,
        };

        // SAFETY: `self.lang` was required to be a valid, non-null pointer to a live
        // TSLanguage by the safety contract of `GrammarLexer::new()`.
        let lex_fn = unsafe { (*self.lang).lex_fn }?;
        // SAFETY: `lex_fn` is a Tree-sitter-generated C function that expects a valid
        // TSLexer pointer; `c_lexer` is a stack-local struct with valid callbacks.
        let ok = unsafe { lex_fn(&mut c_lexer as *mut TSLexer, mode.lex_state) };

        if !ok || c_lexer.result_symbol == 0 {
            return None;
        }

        Some(NextToken {
            kind: c_lexer.result_symbol as u32,
            start: pos as u32,
            end: host.end_mark as u32,
        })
    }
}

// Example of how to get a Tree-sitter language function
// This would be linked from the compiled grammar library
#[allow(dead_code)]
unsafe extern "C" {
    // Example: Link to tree-sitter-json
    // fn tree_sitter_json() -> *const TSLanguage;
}

#[cfg(test)]
mod tests {

    #[test]
    #[ignore = "requires actual Tree-sitter library to be linked"]
    fn test_json_lexer() {
        // This test would require linking to a real Tree-sitter grammar
        // unsafe {
        //     let lang = tree_sitter_json();
        //     let lexer = GrammarLexer::new(lang);
        //     let mode = LexMode { lex_state: 0, external_lex_state: 0 };
        //     let valid = vec![true; 100];
        //     let token = lexer.next("{", 0, mode, &valid);
        //     assert!(token.is_some());
        //     assert_eq!(token.unwrap().kind, 1); // { token
        // }
    }
}
