use adze_linecol_core::LineCol;

#[test]
fn given_unix_newlines_when_tracking_position_then_column_resets_on_new_line() {
    // Given
    let input = b"alpha\nbeta\ngamma";

    // When
    let tracker = LineCol::at_position(input, 6);

    // Then
    assert_eq!(tracker.line, 1);
    assert_eq!(tracker.line_start, 6);
    assert_eq!(tracker.column(9), 3);
}

#[test]
fn given_mixed_line_endings_when_tracking_position_then_line_and_offset_are_correct() {
    // Given
    let input = b"line1\r\nline2\rline3\nline4";

    // When
    let after_crlf = LineCol::at_position(input, 7);
    let after_cr = LineCol::at_position(input, 13);
    let after_lf = LineCol::at_position(input, 19);

    // Then
    assert_eq!(after_crlf.line, 1);
    assert_eq!(after_crlf.line_start, 7);
    assert_eq!(after_cr.line, 2);
    assert_eq!(after_cr.line_start, 13);
    assert_eq!(after_lf.line, 3);
    assert_eq!(after_lf.line_start, 19);
}

#[test]
fn given_stream_scanning_when_processing_bytes_then_result_matches_direct_lookup() {
    // Given
    let input = b"a\r\nb\nc\rd";

    for position in 0..=input.len() {
        // When
        let mut stream_tracker = LineCol::new();
        for i in 0..position {
            let next = input.get(i + 1).copied();
            stream_tracker.process_byte(input[i], next, i);
        }
        let direct_tracker = LineCol::at_position(input, position);

        // Then
        assert_eq!(stream_tracker, direct_tracker, "position={position}");
    }
}
