// FFI bridge for Tree-sitter C external scanners
// This module provides the C ABI-compatible interface for external scanners

use std::ffi::c_void;
use std::os::raw::{c_char, c_uint};

/// Tree-sitter external scanner function signatures
/// These match the C API defined in tree-sitter/parser.h

/// Create a new scanner instance
pub type CreateFn = extern "C" fn() -> *mut c_void;

/// Destroy a scanner instance
pub type DestroyFn = extern "C" fn(payload: *mut c_void);

/// Scan for external tokens
pub type ScanFn =
    extern "C" fn(payload: *mut c_void, lexer: *mut TSLexer, valid_symbols: *const bool) -> bool;

/// Serialize scanner state
pub type SerializeFn = extern "C" fn(payload: *mut c_void, buffer: *mut c_char) -> c_uint;

/// Deserialize scanner state
pub type DeserializeFn = extern "C" fn(payload: *mut c_void, buffer: *const c_char, length: c_uint);

/// Tree-sitter lexer interface (matches C struct)
#[repr(C)]
pub struct TSLexer {
    /// Current lookahead character (0 if at end)
    pub lookahead: extern "C" fn(*mut TSLexer) -> u32,
    /// Advance to next character
    pub advance: extern "C" fn(*mut TSLexer, skip: bool),
    /// Mark the end of the current token
    pub mark_end: extern "C" fn(*mut TSLexer),
    /// Get the current column
    pub get_column: extern "C" fn(*mut TSLexer) -> u32,
    /// Check if at EOF
    pub is_at_included_range_start: extern "C" fn(*const TSLexer) -> bool,
    /// EOF flag
    pub eof: extern "C" fn(*const TSLexer) -> bool,
    /// Result symbol to set
    pub result_symbol: u16,
}

/// External scanner data structure for FFI
#[repr(C)]
#[derive(Copy, Clone)]
pub struct TSExternalScannerData {
    pub states: *const bool,
    pub symbol_map: *const u16,
    pub create: Option<CreateFn>,
    pub destroy: Option<DestroyFn>,
    pub scan: Option<ScanFn>,
    pub serialize: Option<SerializeFn>,
    pub deserialize: Option<DeserializeFn>,
}

// Safety: TSExternalScannerData contains pointers to static data and function pointers.
// The static data is expected to be immutable and the functions are expected to be thread-safe.
unsafe impl Send for TSExternalScannerData {}
unsafe impl Sync for TSExternalScannerData {}

/// Rust wrapper for C external scanners
pub struct CExternalScanner {
    /// Scanner instance payload
    payload: *mut c_void,
    /// Function pointers
    destroy: Option<DestroyFn>,
    scan: Option<ScanFn>,
    serialize: Option<SerializeFn>,
    deserialize: Option<DeserializeFn>,
}

// Safety: CExternalScanner manages a C scanner instance via FFI.
// The C scanner is expected to be thread-safe or used with proper synchronization.
unsafe impl Send for CExternalScanner {}
unsafe impl Sync for CExternalScanner {}

impl CExternalScanner {
    /// Create a new C external scanner wrapper
    pub unsafe fn new(data: &TSExternalScannerData) -> Option<Self> {
        let create = data.create?;
        let payload = create();

        if payload.is_null() {
            return None;
        }

        Some(CExternalScanner {
            payload,
            destroy: data.destroy,
            scan: data.scan,
            serialize: data.serialize,
            deserialize: data.deserialize,
        })
    }

    /// Scan for external tokens
    pub unsafe fn scan(&mut self, lexer: &mut TSLexer, valid_symbols: &[bool]) -> bool {
        if let Some(scan_fn) = self.scan {
            scan_fn(self.payload, lexer as *mut TSLexer, valid_symbols.as_ptr())
        } else {
            false
        }
    }

    /// Serialize scanner state
    pub unsafe fn serialize(&self, buffer: &mut Vec<u8>) -> usize {
        if let Some(serialize_fn) = self.serialize {
            // Tree-sitter uses a fixed buffer size of 1024
            const BUFFER_SIZE: usize = 1024;
            let mut temp_buffer = vec![0u8; BUFFER_SIZE];

            let bytes_written = serialize_fn(self.payload, temp_buffer.as_mut_ptr() as *mut c_char);

            let bytes_written = bytes_written as usize;
            if bytes_written > 0 && bytes_written <= BUFFER_SIZE {
                buffer.extend_from_slice(&temp_buffer[..bytes_written]);
                bytes_written
            } else {
                0
            }
        } else {
            0
        }
    }

    /// Deserialize scanner state
    pub unsafe fn deserialize(&mut self, buffer: &[u8]) {
        if let Some(deserialize_fn) = self.deserialize {
            deserialize_fn(
                self.payload,
                buffer.as_ptr() as *const c_char,
                buffer.len() as c_uint,
            )
        }
    }
}

impl Drop for CExternalScanner {
    fn drop(&mut self) {
        if let Some(destroy_fn) = self.destroy {
            #[allow(unused_unsafe)]
            unsafe {
                destroy_fn(self.payload);
            }
        }
    }
}

/// Rust lexer adapter that implements the TSLexer interface
pub struct RustLexerAdapter<'a> {
    input: &'a [u8],
    position: usize,
    token_end: usize,
    result_symbol: u16,
    line: usize,
    column: usize,
}

impl<'a> RustLexerAdapter<'a> {
    pub fn new(input: &'a [u8], position: usize) -> Self {
        // Calculate initial line and column from position
        let (line, column) = Self::calculate_line_column(input, position);
        RustLexerAdapter {
            input,
            position,
            token_end: position,
            result_symbol: 0,
            line,
            column,
        }
    }
    
    /// Calculate line and column from byte position
    fn calculate_line_column(input: &[u8], position: usize) -> (usize, usize) {
        let mut line = 0;
        let mut column = 0;
        
        for i in 0..position.min(input.len()) {
            if input[i] == b'\n' {
                line += 1;
                column = 0;
            } else {
                column += 1;
            }
        }
        
        (line, column)
    }

    /// Create a C-compatible TSLexer
    pub fn as_ts_lexer(&mut self) -> TSLexer {
        TSLexer {
            lookahead: rust_lexer_lookahead,
            advance: rust_lexer_advance,
            mark_end: rust_lexer_mark_end,
            get_column: rust_lexer_get_column,
            is_at_included_range_start: rust_lexer_is_at_included_range_start,
            eof: rust_lexer_eof,
            result_symbol: self.result_symbol,
        }
    }

    /// Get the consumed token length
    pub fn token_length(&self) -> usize {
        self.token_end - self.position
    }
}

// C-compatible callback functions
extern "C" fn rust_lexer_lookahead(lexer: *mut TSLexer) -> u32 {
    unsafe {
        let adapter = lexer as *mut RustLexerAdapter;
        let adapter = &*adapter;

        if adapter.position < adapter.input.len() {
            adapter.input[adapter.position] as u32
        } else {
            0
        }
    }
}

extern "C" fn rust_lexer_advance(lexer: *mut TSLexer, _skip: bool) {
    unsafe {
        let adapter = lexer as *mut RustLexerAdapter;
        let adapter = &mut *adapter;

        if adapter.position < adapter.input.len() {
            // Check if we're advancing past a newline
            if adapter.input[adapter.position] == b'\n' {
                adapter.line += 1;
                adapter.column = 0;
            } else {
                adapter.column += 1;
            }
            adapter.position += 1;
            if adapter.token_end < adapter.position {
                adapter.token_end = adapter.position;
            }
        }
    }
}

extern "C" fn rust_lexer_mark_end(lexer: *mut TSLexer) {
    unsafe {
        let adapter = lexer as *mut RustLexerAdapter;
        let adapter = &mut *adapter;
        adapter.token_end = adapter.position;
    }
}

extern "C" fn rust_lexer_get_column(lexer: *mut TSLexer) -> u32 {
    unsafe {
        let adapter = lexer as *const RustLexerAdapter;
        let adapter = &*adapter;
        adapter.column as u32
    }
}

extern "C" fn rust_lexer_is_at_included_range_start(_lexer: *const TSLexer) -> bool {
    false
}

extern "C" fn rust_lexer_eof(lexer: *const TSLexer) -> bool {
    unsafe {
        let adapter = lexer as *const RustLexerAdapter;
        let adapter = &*adapter;
        adapter.position >= adapter.input.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rust_lexer_adapter() {
        let input = b"hello world";
        let mut adapter = RustLexerAdapter::new(input, 0);

        // Test lookahead directly on adapter
        assert_eq!(adapter.position, 0);
        assert_eq!(adapter.input[adapter.position], b'h');

        // Test advance
        adapter.position += 1;
        adapter.token_end = adapter.position;
        assert_eq!(adapter.input[adapter.position], b'e');

        // Test EOF
        assert!(adapter.position < adapter.input.len());

        // The actual FFI interface would need proper pointer handling
        // which is complex to test in a unit test
    }
}
