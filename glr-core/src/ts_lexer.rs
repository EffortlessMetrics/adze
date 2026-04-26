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
    included_ranges_supported: bool,
    unsupported_is_included_called: bool,
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

    extern "C" fn get_column(payload: *mut c_void) -> u32 {
        // SAFETY: see shared invariant above.
        let host = unsafe { &mut *(payload as *mut Self) };

        let capped = host.pos.min(host.input.len());
        let mut line_start = 0usize;
        let mut i = capped;

        while i > 0 {
            i -= 1;
            let byte = host.input[i];
            if byte == b'\n' {
                line_start = i + 1;
                break;
            }
            if byte == b'\r' {
                // Treat CR and CRLF as line breaks.
                line_start = i + 1;
                break;
            }
        }

        let mut col = 0u32;
        for byte in &host.input[line_start..capped] {
            // Count UTF-8 leading bytes as codepoint columns.
            if (byte & 0b1100_0000) != 0b1000_0000 {
                col = col.saturating_add(1);
            }
        }

        col
    }

    extern "C" fn is_included(payload: *mut c_void) -> bool {
        // SAFETY: see shared invariant above.
        let host = unsafe { &mut *(payload as *mut Self) };
        if host.included_ranges_supported {
            true
        } else {
            host.unsupported_is_included_called = true;
            false
        }
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
            included_ranges_supported: false,
            unsupported_is_included_called: false,
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

        if host.unsupported_is_included_called {
            return None;
        }

        if !ok || c_lexer.result_symbol == 0 {
            return None;
        }

        if host.end_mark <= pos {
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
    use super::*;

    extern "C" fn lex_reads_column(lexer: *mut TSLexer, _state: u16) -> bool {
        // SAFETY: called with a valid pointer from the test.
        let lexer = unsafe { &mut *lexer };
        let col = (lexer.get_column)(lexer.payload);
        if col == 1 {
            lexer.result_symbol = 1;
            (lexer.advance)(lexer.payload, false);
            (lexer.mark_end)(lexer.payload);
            true
        } else {
            false
        }
    }

    extern "C" fn lex_calls_is_included(lexer: *mut TSLexer, _state: u16) -> bool {
        // SAFETY: called with a valid pointer from the test.
        let lexer = unsafe { &mut *lexer };
        let _ = (lexer.is_included)(lexer.payload);
        lexer.result_symbol = 1;
        (lexer.mark_end)(lexer.payload);
        true
    }

    extern "C" fn lex_zero_width(lexer: *mut TSLexer, _state: u16) -> bool {
        // SAFETY: called with a valid pointer from the test.
        let lexer = unsafe { &mut *lexer };
        lexer.result_symbol = 1;
        true
    }

    fn test_language(lex_fn: LexFn) -> TSLanguage {
        TSLanguage {
            version: 0,
            symbol_count: 0,
            alias_count: 0,
            token_count: 0,
            external_token_count: 0,
            state_count: 0,
            large_state_count: 0,
            production_id_count: 0,
            field_count: 0,
            max_alias_sequence_length: 0,
            _parse_table: std::ptr::null(),
            _small_parse_table: std::ptr::null(),
            _small_parse_table_map: std::ptr::null(),
            _parse_actions: std::ptr::null(),
            _symbol_names: std::ptr::null(),
            _field_names: std::ptr::null(),
            _field_map_slices: std::ptr::null(),
            _field_map_entries: std::ptr::null(),
            _symbol_metadata: std::ptr::null(),
            _public_symbol_map: std::ptr::null(),
            _alias_map: std::ptr::null(),
            _alias_sequences: std::ptr::null(),
            lex_modes: std::ptr::null(),
            lex_fn: Some(lex_fn),
            keyword_lex_fn: None,
            keyword_capture_token: 0,
            external_scanner: ExternalScanner {
                states: std::ptr::null(),
                symbol_map: std::ptr::null(),
                create: None,
                destroy: None,
                scan: None,
                serialize: None,
                deserialize: None,
            },
        }
    }

    #[test]
    fn reports_column_from_input_position() {
        let lang = test_language(lex_reads_column);
        // SAFETY: language object is stack-local and valid for the call.
        let lexer = unsafe { GrammarLexer::new(&lang as *const TSLanguage) };
        let mode = LexMode {
            lex_state: 0,
            external_lex_state: 0,
        };
        let token = lexer.next("a\nbc", 3, mode, &[]);
        assert!(token.is_some());
    }

    #[test]
    fn rejects_is_included_callback_until_ranges_are_supported() {
        let lang = test_language(lex_calls_is_included);
        // SAFETY: language object is stack-local and valid for the call.
        let lexer = unsafe { GrammarLexer::new(&lang as *const TSLanguage) };
        let mode = LexMode {
            lex_state: 0,
            external_lex_state: 0,
        };
        let token = lexer.next("abc", 0, mode, &[]);
        assert!(token.is_none());
    }

    #[test]
    fn rejects_zero_width_tokens_from_lex_fn() {
        let lang = test_language(lex_zero_width);
        // SAFETY: language object is stack-local and valid for the call.
        let lexer = unsafe { GrammarLexer::new(&lang as *const TSLanguage) };
        let mode = LexMode {
            lex_state: 0,
            external_lex_state: 0,
        };
        let token = lexer.next("abc", 0, mode, &[]);
        assert!(token.is_none());
    }
}
