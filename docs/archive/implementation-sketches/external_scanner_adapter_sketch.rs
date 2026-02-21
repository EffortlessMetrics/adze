//! External Scanner Adapter Implementation Sketch
//!
//! Implementation plan for wiring external scanners into the parse loop.

use std::sync::Arc;

// === Step 1: TSLexerAdapter Implementation ===

/// Adapter that implements TSLexer trait for external scanners
pub struct TSLexerAdapter<'a> {
    /// Source text being parsed
    source: &'a [u8],

    /// Current byte position
    cursor: usize,

    /// End of current token (set by mark_end)
    mark_end_pos: usize,

    /// Current position as row/column
    point: Point,

    /// Precomputed line starts for efficient column calculation
    line_starts: &'a [usize],

    /// Included ranges (for multi-language parsing)
    ranges: Option<&'a [Range<usize>]>,

    /// Current range index
    current_range: usize,
}

#[derive(Debug, Clone, Copy)]
pub struct Point {
    pub row: u32,
    pub column: u32,
}

impl<'a> TSLexerAdapter<'a> {
    pub fn new(
        source: &'a [u8],
        cursor: usize,
        line_starts: &'a [usize],
        ranges: Option<&'a [Range<usize>]>,
    ) -> Self {
        let point = Self::byte_to_point(cursor, line_starts);
        Self {
            source,
            cursor,
            mark_end_pos: cursor,
            point,
            line_starts,
            ranges,
            current_range: 0,
        }
    }

    fn byte_to_point(byte_idx: usize, line_starts: &[usize]) -> Point {
        // Binary search for line
        let row = line_starts
            .partition_point(|&start| start <= byte_idx)
            .saturating_sub(1);
        let line_start = line_starts[row];
        let column = byte_idx - line_start;
        Point {
            row: row as u32,
            column: column as u32,
        }
    }

    pub fn consumed_bytes(&self) -> usize {
        self.mark_end_pos - self.cursor
    }
}

impl TSLexer for TSLexerAdapter<'_> {
    fn advance(&mut self, skip: bool) -> bool {
        // Check if we're at range boundary
        if let Some(ranges) = self.ranges {
            if let Some(range) = ranges.get(self.current_range) {
                if self.cursor >= range.end {
                    return false;
                }
            }
        }

        if self.cursor >= self.source.len() {
            return false;
        }

        let ch = self.source[self.cursor];
        self.cursor += 1;

        // Update row/column
        if ch == b'\n' {
            self.point.row += 1;
            self.point.column = 0;
        } else if ch == b'\r' {
            // Handle CRLF
            if self.cursor < self.source.len() && self.source[self.cursor] == b'\n' {
                self.cursor += 1;
            }
            self.point.row += 1;
            self.point.column = 0;
        } else {
            self.point.column += 1;
        }

        if !skip {
            self.mark_end_pos = self.cursor;
        }

        true
    }

    fn mark_end(&mut self) {
        self.mark_end_pos = self.cursor;
    }

    fn get_column(&self) -> u32 {
        self.point.column
    }

    fn is_at_included_range_start(&self) -> bool {
        if let Some(ranges) = self.ranges {
            ranges
                .get(self.current_range)
                .map(|r| r.start == self.cursor)
                .unwrap_or(false)
        } else {
            false
        }
    }

    fn lookahead(&self) -> char {
        if self.cursor < self.source.len() {
            self.source[self.cursor] as char
        } else {
            '\0'
        }
    }
}

// === Step 2: Parse Loop Integration ===

/// In parser_v4.rs or equivalent
pub struct ParserV4 {
    // ... existing fields ...
    /// External scanner for custom lexing
    external_scanner: Option<Arc<dyn ExternalScanner + Send + Sync>>,
}

impl ParserV4 {
    /// Main lexing step with external scanner support
    fn lex_step(&mut self, state: TSStateId, source: &[u8], cursor: usize) -> Token {
        // Get valid external symbols for current state
        let valid_external = self.get_valid_external_symbols(state);

        // Try external scanner first if we have one and valid symbols
        if !valid_external.is_empty() {
            if let Some(scanner) = &self.external_scanner {
                let mut adapter = TSLexerAdapter::new(
                    source,
                    cursor,
                    &self.line_starts,
                    self.included_ranges.as_deref(),
                );

                // Clone scanner for thread safety (or use mutex)
                let mut scanner = scanner.clone();

                if scanner.scan(&mut adapter, &valid_external) {
                    let token_len = adapter.consumed_bytes();
                    let token_type = self.find_external_token_type(&valid_external);

                    return Token {
                        kind: TokenKind::External(token_type),
                        start: cursor,
                        end: cursor + token_len,
                    };
                }
            }
        }

        // Fall back to normal lexer
        self.lex_internal(state, source, cursor)
    }

    fn get_valid_external_symbols(&self, state: TSStateId) -> Vec<bool> {
        // Look up in language data which external symbols are valid
        // for the current parse state
        self.language.valid_external_symbols(state)
    }

    fn find_external_token_type(&self, valid_symbols: &[bool]) -> TSSymbol {
        // Map from valid symbol index to actual symbol ID
        for (idx, &valid) in valid_symbols.iter().enumerate() {
            if valid {
                return self.language.external_symbol_map[idx];
            }
        }
        0 // Should not happen
    }
}

// === Step 3: Scanner Registration ===

impl Parser {
    /// Set external scanner for custom lexing
    pub fn set_external_scanner(&mut self, scanner: Arc<dyn ExternalScanner + Send + Sync>) {
        self.v4_parser.external_scanner = Some(scanner);
    }
}

// === Step 4: C Scanner FFI ===

/// FFI wrapper for C external scanners
#[repr(C)]
pub struct CExternalScanner {
    create: unsafe extern "C" fn() -> *mut c_void,
    destroy: unsafe extern "C" fn(*mut c_void),
    scan: unsafe extern "C" fn(*mut c_void, *mut TSLexer, *const bool) -> bool,
    serialize: unsafe extern "C" fn(*mut c_void, *mut u8) -> u32,
    deserialize: unsafe extern "C" fn(*mut c_void, *const u8, u32),
}

/// Wrapper to make C scanner implement Rust trait
struct CExternalScannerWrapper {
    scanner: *mut c_void,
    vtable: &'static CExternalScanner,
}

impl ExternalScanner for CExternalScannerWrapper {
    fn scan(&mut self, lexer: &mut dyn TSLexer, valid_symbols: &[bool]) -> bool {
        unsafe {
            // Convert Rust lexer to C lexer
            let c_lexer = create_c_lexer_wrapper(lexer);
            (self.vtable.scan)(self.scanner, c_lexer, valid_symbols.as_ptr())
        }
    }

    fn serialize(&self, buffer: &mut Vec<u8>) -> usize {
        let start_len = buffer.len();
        buffer.resize(start_len + 1024, 0); // Reserve space

        unsafe {
            let size = (self.vtable.serialize)(self.scanner, buffer.as_mut_ptr().add(start_len));
            buffer.truncate(start_len + size as usize);
            size as usize
        }
    }

    fn deserialize(&mut self, buffer: &[u8]) -> usize {
        unsafe {
            (self.vtable.deserialize)(self.scanner, buffer.as_ptr(), buffer.len() as u32);
            buffer.len()
        }
    }
}

impl Drop for CExternalScannerWrapper {
    fn drop(&mut self) {
        unsafe {
            (self.vtable.destroy)(self.scanner);
        }
    }
}

// === Step 5: Fix FFI stub ===

/// In ffi.rs:186 - replace stub
extern "C" fn ts_lexer_is_at_included_range_start(lexer: *const TSLexer) -> bool {
    let adapter = unsafe { &*(lexer as *const TSLexerAdapter) };
    adapter.is_at_included_range_start()
}
