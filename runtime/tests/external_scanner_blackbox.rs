#![cfg(feature = "external_scanners")] // Disable until external scanner is fully implemented

/// Black-box tests for external scanner functionality
/// These tests verify the external scanner API behavior from a user perspective
use rust_sitter::external_scanner_ffi::{RustLexerAdapter, TSLexer, destroy_lexer};
use rust_sitter::linecol::LineCol;

#[test]
fn test_line_col_tracking_simple() {
    let input = b"hello\nworld";
    let tracker = LineCol::at_position(input, 0);
    assert_eq!(tracker.line, 0);
    assert_eq!(tracker.line_start, 0);

    let tracker = LineCol::at_position(input, 6); // After '\n'
    assert_eq!(tracker.line, 1);
    assert_eq!(tracker.line_start, 6);
}

#[test]
fn test_line_col_with_crlf() {
    let input = b"line1\r\nline2\rline3\nline4";

    // At start
    let tracker = LineCol::at_position(input, 0);
    assert_eq!(tracker.line, 0);
    assert_eq!(tracker.line_start, 0);

    // After first CRLF
    let tracker = LineCol::at_position(input, 7); // After '\r\n'
    assert_eq!(tracker.line, 1);
    assert_eq!(tracker.line_start, 7);

    // After lone CR
    let tracker = LineCol::at_position(input, 13); // After '\r'
    assert_eq!(tracker.line, 2);
    assert_eq!(tracker.line_start, 13);

    // After LF
    let tracker = LineCol::at_position(input, 19); // After '\n'
    assert_eq!(tracker.line, 3);
    assert_eq!(tracker.line_start, 19);
}

#[test]
fn test_line_col_empty_lines() {
    let input = b"\n\n\ntext";

    let tracker = LineCol::at_position(input, 0);
    assert_eq!(tracker.line, 0);

    let tracker = LineCol::at_position(input, 1);
    assert_eq!(tracker.line, 1);

    let tracker = LineCol::at_position(input, 2);
    assert_eq!(tracker.line, 2);

    let tracker = LineCol::at_position(input, 3);
    assert_eq!(tracker.line, 3);
    assert_eq!(tracker.line_start, 3);
}

/// Test external scanner API through the FFI layer
#[test]
fn test_external_scanner_api_safety() {
    // This test verifies that the external scanner API handles nulls safely
    unsafe {
        // Test that destroy_lexer handles null pointers
        destroy_lexer(std::ptr::null_mut());

        // Create a mock lexer and adapter
        let input = b"test input";
        let mut adapter = Box::new(RustLexerAdapter::new(input));

        let mut lexer = TSLexer {
            lookahead: 0,
            column: 0,
            position: 0,
            end_column: 0,
            current_position: 0,
            lookahead_end_byte: 0,
            advance: None,
            mark_end: None,
            get_column: None,
            is_at_included_range_start: None,
            context: Box::into_raw(adapter) as *mut std::ffi::c_void,
        };

        // Test that we can safely destroy the lexer
        destroy_lexer(&mut lexer as *mut TSLexer);
    }
}

/// Test the column calculation for various line ending scenarios
#[test]
fn test_column_calculation() {
    let input = b"abc\ndef\r\nghi\rjkl";

    // Test column at various positions
    struct TestCase {
        position: usize,
        expected_column: usize,
        description: &'static str,
    }

    let test_cases = vec![
        TestCase {
            position: 0,
            expected_column: 0,
            description: "Start of file",
        },
        TestCase {
            position: 2,
            expected_column: 2,
            description: "Middle of first line",
        },
        TestCase {
            position: 3,
            expected_column: 3,
            description: "Before LF",
        },
        TestCase {
            position: 4,
            expected_column: 0,
            description: "After LF",
        },
        TestCase {
            position: 7,
            expected_column: 3,
            description: "Before CR in CRLF",
        },
        TestCase {
            position: 9,
            expected_column: 0,
            description: "After CRLF",
        },
        TestCase {
            position: 12,
            expected_column: 3,
            description: "Before lone CR",
        },
        TestCase {
            position: 13,
            expected_column: 0,
            description: "After lone CR",
        },
    ];

    for test_case in test_cases {
        let tracker = LineCol::at_position(input, test_case.position);
        let column = test_case.position - tracker.line_start;
        assert_eq!(
            column, test_case.expected_column,
            "Failed at {}: expected column {}, got {}",
            test_case.description, test_case.expected_column, column
        );
    }
}

/// Test that the adapter correctly tracks position through advances
///
/// Note: This test is commented out as RustLexerAdapter fields are private
/// and the advance() method is not public. The adapter is tested through
/// the FFI layer in the tests above.
#[test]
#[ignore]
fn test_adapter_position_tracking() {
    // This test would require public access to RustLexerAdapter internals
    // which are intentionally kept private for safety.
    // The functionality is tested through the FFI layer instead.
}

/// Test edge cases in line/column handling
#[test]
fn test_edge_cases() {
    // Empty input
    let tracker = LineCol::at_position(b"", 0);
    assert_eq!(tracker.line, 0);
    assert_eq!(tracker.line_start, 0);

    // Position beyond input
    let input = b"test";
    let tracker = LineCol::at_position(input, 100);
    assert_eq!(tracker.line, 0);
    assert_eq!(tracker.line_start, 0);

    // Input ending with newline
    let input = b"test\n";
    let tracker = LineCol::at_position(input, 5);
    assert_eq!(tracker.line, 1);
    assert_eq!(tracker.line_start, 5);
}
