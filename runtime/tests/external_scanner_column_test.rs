use rust_sitter::external_scanner_ffi::{RustLexerAdapter, TSLexer};

#[test]
fn test_column_tracking_basic() {
    let input = b"hello world";
    let mut adapter = RustLexerAdapter::new(input, 0);
    
    // Initial position
    assert_eq!(adapter.get_column(), 0);
    
    // Create TSLexer with adapter context
    let mut ts_lexer = adapter.as_ts_lexer();
    
    // Advance through "hello"
    for i in 0..5 {
        unsafe { (ts_lexer.advance)(&mut ts_lexer as *mut TSLexer, false); }
    }
    
    // After "hello", column should be 5
    let column = unsafe { (ts_lexer.get_column)(&mut ts_lexer as *mut TSLexer) };
    assert_eq!(column, 5);
}

#[test]
fn test_column_tracking_with_newlines() {
    let input = b"hello\nworld\ntest";
    let mut adapter = RustLexerAdapter::new(input, 0);
    
    // Initial position
    assert_eq!(adapter.get_column(), 0);
    
    let mut ts_lexer = adapter.as_ts_lexer();
    
    // Advance to newline
    for _ in 0..5 {
        unsafe { (ts_lexer.advance)(&mut ts_lexer as *mut TSLexer, false); }
    }
    assert_eq!(unsafe { (ts_lexer.get_column)(&mut ts_lexer as *mut TSLexer) }, 5);
    
    // Advance past newline
    unsafe { (ts_lexer.advance)(&mut ts_lexer as *mut TSLexer, false); }
    
    // Column should reset to 0 after newline
    assert_eq!(unsafe { (ts_lexer.get_column)(&mut ts_lexer as *mut TSLexer) }, 0);
    
    // Advance through "world"
    for _ in 0..5 {
        unsafe { (ts_lexer.advance)(&mut ts_lexer as *mut TSLexer, false); }
    }
    assert_eq!(unsafe { (ts_lexer.get_column)(&mut ts_lexer as *mut TSLexer) }, 5);
}

#[test]
fn test_column_tracking_with_crlf() {
    let input = b"hello\r\nworld";
    let mut adapter = RustLexerAdapter::new(input, 0);
    let mut ts_lexer = adapter.as_ts_lexer();
    
    // Advance to CR
    for _ in 0..5 {
        unsafe { (ts_lexer.advance)(&mut ts_lexer as *mut TSLexer, false); }
    }
    assert_eq!(unsafe { (ts_lexer.get_column)(&mut ts_lexer as *mut TSLexer) }, 5);
    
    // Advance past CRLF (should handle both characters)
    unsafe { (ts_lexer.advance)(&mut ts_lexer as *mut TSLexer, false); }
    
    // Column should reset to 0 after CRLF
    assert_eq!(unsafe { (ts_lexer.get_column)(&mut ts_lexer as *mut TSLexer) }, 0);
}

#[test]
fn test_column_tracking_with_skip() {
    let input = b"  hello";
    let mut adapter = RustLexerAdapter::new(input, 0);
    let mut ts_lexer = adapter.as_ts_lexer();
    
    // Skip whitespace (with skip=true)
    unsafe { (ts_lexer.advance)(&mut ts_lexer as *mut TSLexer, true); }
    unsafe { (ts_lexer.advance)(&mut ts_lexer as *mut TSLexer, true); }
    
    // Column should still advance even when skipping
    assert_eq!(unsafe { (ts_lexer.get_column)(&mut ts_lexer as *mut TSLexer) }, 2);
}

#[test]
fn test_initial_position_calculation() {
    let input = b"line1\nline2\nline3";
    
    // Start at position 6 (beginning of "line2")
    let adapter = RustLexerAdapter::new(input, 6);
    assert_eq!(adapter.get_column(), 0); // Should be column 0 of line 1
    
    // Start at position 8 (middle of "line2")
    let adapter = RustLexerAdapter::new(input, 8);
    assert_eq!(adapter.get_column(), 2); // Should be column 2 of line 1
}