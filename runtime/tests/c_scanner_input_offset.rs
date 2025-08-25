use rust_sitter::external_scanner::Lexer;
use rust_sitter::external_scanner_ffi::{TSExternalScannerData, TSLexer};
use rust_sitter::scanner_registry::ScannerRegistry;
use rust_sitter_ir::SymbolId;
use std::ffi::{c_char, c_void};

// Store the last observed byte and column from the C scanner
static mut LAST_BYTE: u8 = 0;
static mut LAST_COLUMN: u32 = 0;

extern "C" fn test_create() -> *mut c_void {
    // Allocate a dummy payload so pointer is non-null
    Box::into_raw(Box::new(())) as *mut c_void
}

extern "C" fn test_destroy(payload: *mut c_void) {
    if !payload.is_null() {
        unsafe {
            drop(Box::from_raw(payload as *mut ()));
        }
    }
}

extern "C" fn test_scan(
    _payload: *mut c_void,
    lexer: *mut TSLexer,
    _valid_symbols: *const bool,
) -> bool {
    unsafe {
        let lookahead = ((*lexer).lookahead)(lexer) as u8;
        let column = ((*lexer).get_column)(lexer);
        LAST_BYTE = lookahead;
        LAST_COLUMN = column;
    }
    false
}

extern "C" fn test_serialize(_payload: *mut c_void, _buffer: *mut c_char) -> u32 {
    0
}
extern "C" fn test_deserialize(_payload: *mut c_void, _buffer: *const c_char, _length: u32) {}

struct TestLexer {
    input: Vec<u8>,
    position: usize,
}

impl Lexer for TestLexer {
    fn lookahead(&self) -> Option<u8> {
        self.input.get(self.position).copied()
    }
    fn advance(&mut self, n: usize) {
        self.position = (self.position + n).min(self.input.len());
    }
    fn mark_end(&mut self) {}
    fn column(&self) -> usize {
        let line_start = self.input[..self.position]
            .iter()
            .rposition(|&b| b == b'\n')
            .map(|i| i + 1)
            .unwrap_or(0);
        self.position - line_start
    }
    fn is_eof(&self) -> bool {
        self.position >= self.input.len()
    }
    fn input(&self) -> &[u8] {
        &self.input
    }
    fn byte_position(&self) -> usize {
        self.position
    }
}

#[test]
fn c_scanner_receives_correct_text_and_offset() {
    unsafe {
        LAST_BYTE = 0;
        LAST_COLUMN = 0;
    }

    let data = TSExternalScannerData {
        states: std::ptr::null(),
        symbol_map: std::ptr::null(),
        create: Some(test_create),
        destroy: Some(test_destroy),
        scan: Some(test_scan),
        serialize: Some(test_serialize),
        deserialize: Some(test_deserialize),
    };

    let mut registry = ScannerRegistry::new();
    registry.register_c_scanner("test_lang", data, vec![SymbolId(1)]);
    let mut scanner = registry.create_scanner("test_lang").expect("scanner");

    // Input with a newline to test column calculation
    let mut lexer = TestLexer {
        input: b"ab\ncd".to_vec(),
        position: 4,
    }; // points at 'd'
    let valid = [false];
    scanner.scan(&mut lexer, &valid);

    unsafe {
        assert_eq!(core::ptr::addr_of!(LAST_BYTE).read(), b'd');
        assert_eq!(core::ptr::addr_of!(LAST_COLUMN).read(), 1); // after newline, 'd' is column 1
    }
}
