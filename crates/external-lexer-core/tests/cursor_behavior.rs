use adze_external_lexer_core::LexerCursor;

#[test]
fn initializes_line_metadata() {
    let input = b"a\r\nb\nc";
    let cursor = LexerCursor::new(input, 3);
    assert_eq!(cursor.line, 1);
    assert_eq!(cursor.line_start, 3);
    assert_eq!(cursor.column(), 0);
}

#[test]
fn advances_across_crlf_and_tracks_token_end() {
    let input = b"x\r\ny";
    let mut cursor = LexerCursor::new(input, 0);

    cursor.advance(input, false);
    assert_eq!(cursor.position, 1);
    assert_eq!(cursor.line, 0);

    cursor.advance(input, false);
    assert_eq!(cursor.position, 3);
    assert_eq!(cursor.line, 1);
    assert_eq!(cursor.line_start, 3);
    assert_eq!(cursor.token_end, 3);
}
