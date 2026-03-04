// Comprehensive tests for linecol-core
use adze_linecol_core::LineCol;

// ---------------------------------------------------------------------------
// Construction
// ---------------------------------------------------------------------------

#[test]
fn new_at_origin() {
    let lc = LineCol::new();
    assert_eq!(lc.line, 0);
    assert_eq!(lc.line_start, 0);
}

#[test]
fn at_position_start() {
    let lc = LineCol::at_position(b"hello", 0);
    assert_eq!(lc.line, 0);
    assert_eq!(lc.column(0), 0);
}

#[test]
fn at_position_middle_of_first_line() {
    let lc = LineCol::at_position(b"hello", 3);
    assert_eq!(lc.line, 0);
    assert_eq!(lc.column(3), 3);
}

#[test]
fn at_position_on_second_line() {
    let lc = LineCol::at_position(b"hello\nworld", 8);
    assert_eq!(lc.line, 1);
    assert_eq!(lc.column(8), 2); // "wo" = 2 bytes into "world"
}

#[test]
fn at_position_at_newline() {
    let lc = LineCol::at_position(b"hello\nworld", 5);
    // Position 5 is the \n character itself — still line 0
    assert_eq!(lc.line, 0);
}

#[test]
fn at_position_right_after_newline() {
    let lc = LineCol::at_position(b"hello\nworld", 6);
    assert_eq!(lc.line, 1);
    assert_eq!(lc.column(6), 0);
}

// ---------------------------------------------------------------------------
// Multiple lines
// ---------------------------------------------------------------------------

#[test]
fn three_lines() {
    let input = b"a\nb\nc";
    let lc = LineCol::at_position(input, 4);
    assert_eq!(lc.line, 2);
    assert_eq!(lc.column(4), 0);
}

#[test]
fn empty_lines() {
    let input = b"\n\n\n";
    let lc = LineCol::at_position(input, 2);
    assert_eq!(lc.line, 2);
    assert_eq!(lc.column(2), 0);
}

// ---------------------------------------------------------------------------
// Windows line endings
// ---------------------------------------------------------------------------

#[test]
fn crlf_line_ending() {
    let input = b"hello\r\nworld";
    let lc = LineCol::at_position(input, 7);
    assert_eq!(lc.line, 1);
    assert_eq!(lc.column(7), 0);
}

#[test]
fn cr_only() {
    let input = b"hello\rworld";
    let lc = LineCol::at_position(input, 6);
    assert_eq!(lc.line, 1);
    assert_eq!(lc.column(6), 0);
}

// ---------------------------------------------------------------------------
// advance_line
// ---------------------------------------------------------------------------

#[test]
fn advance_line_increments() {
    let mut lc = LineCol::new();
    lc.advance_line(5);
    assert_eq!(lc.line, 1);
    assert_eq!(lc.line_start, 5);
}

#[test]
fn advance_line_twice() {
    let mut lc = LineCol::new();
    lc.advance_line(5);
    lc.advance_line(10);
    assert_eq!(lc.line, 2);
    assert_eq!(lc.line_start, 10);
}

// ---------------------------------------------------------------------------
// column
// ---------------------------------------------------------------------------

#[test]
fn column_at_line_start() {
    let lc = LineCol {
        line: 1,
        line_start: 6,
    };
    assert_eq!(lc.column(6), 0);
}

#[test]
fn column_offset() {
    let lc = LineCol {
        line: 1,
        line_start: 6,
    };
    assert_eq!(lc.column(10), 4);
}

// ---------------------------------------------------------------------------
// process_byte
// ---------------------------------------------------------------------------

#[test]
fn process_byte_newline() {
    let mut lc = LineCol::new();
    let result = lc.process_byte(b'\n', Some(b'x'), 5);
    assert!(result);
    assert_eq!(lc.line, 1);
    assert_eq!(lc.line_start, 6);
}

#[test]
fn process_byte_regular() {
    let mut lc = LineCol::new();
    let result = lc.process_byte(b'a', Some(b'b'), 0);
    assert!(!result);
    assert_eq!(lc.line, 0);
}

#[test]
fn process_byte_cr_then_lf() {
    let mut lc = LineCol::new();
    // \r followed by \n returns false - \n will be processed separately
    let result = lc.process_byte(b'\r', Some(b'\n'), 5);
    assert!(!result);
    assert_eq!(lc.line, 0);
}

#[test]
fn process_byte_cr_no_lf() {
    let mut lc = LineCol::new();
    let result = lc.process_byte(b'\r', Some(b'x'), 5);
    assert!(result);
    assert_eq!(lc.line, 1);
}

// ---------------------------------------------------------------------------
// Clone / Copy / Debug / Eq / Hash
// ---------------------------------------------------------------------------

#[test]
fn linecol_clone() {
    let lc = LineCol {
        line: 5,
        line_start: 100,
    };
    let lc2 = lc;
    assert_eq!(lc, lc2);
}

#[test]
fn linecol_debug() {
    let lc = LineCol::new();
    let d = format!("{:?}", lc);
    assert!(d.contains("LineCol"));
}

#[test]
fn linecol_eq() {
    let a = LineCol {
        line: 1,
        line_start: 5,
    };
    let b = LineCol {
        line: 1,
        line_start: 5,
    };
    assert_eq!(a, b);
}

#[test]
fn linecol_ne() {
    let a = LineCol {
        line: 1,
        line_start: 5,
    };
    let b = LineCol {
        line: 2,
        line_start: 5,
    };
    assert_ne!(a, b);
}

// ---------------------------------------------------------------------------
// Edge cases
// ---------------------------------------------------------------------------

#[test]
fn empty_input() {
    let lc = LineCol::at_position(b"", 0);
    assert_eq!(lc.line, 0);
    assert_eq!(lc.column(0), 0);
}

#[test]
fn single_newline() {
    let lc = LineCol::at_position(b"\n", 1);
    assert_eq!(lc.line, 1);
}

#[test]
fn only_newlines() {
    let input = b"\n\n\n\n\n";
    let lc = LineCol::at_position(input, 5);
    assert_eq!(lc.line, 5);
}

#[test]
fn long_line() {
    let input: Vec<u8> = (0..1000).map(|_| b'a').collect();
    let lc = LineCol::at_position(&input, 500);
    assert_eq!(lc.line, 0);
    assert_eq!(lc.column(500), 500);
}
