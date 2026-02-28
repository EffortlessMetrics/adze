use adze_linecol_core::LineCol;
use proptest::prelude::*;

fn model_linecol(input: &[u8], position: usize) -> (usize, usize) {
    let end = position.min(input.len());
    let mut line = 0usize;
    let mut line_start = 0usize;
    let mut i = 0usize;

    while i < end {
        match input[i] {
            b'\n' => {
                line += 1;
                line_start = i + 1;
                i += 1;
            }
            b'\r' => {
                if i + 1 < input.len() && input[i + 1] == b'\n' {
                    if i + 1 < end {
                        line += 1;
                        line_start = i + 2;
                        i += 2;
                    } else {
                        i += 1;
                    }
                } else {
                    line += 1;
                    line_start = i + 1;
                    i += 1;
                }
            }
            _ => i += 1,
        }
    }

    (line, line_start)
}

proptest! {
    #[test]
    fn at_position_matches_model(
        input in prop::collection::vec(any::<u8>(), 0..512),
        position in 0usize..768,
    ) {
        let tracker = LineCol::at_position(&input, position);
        let (line, line_start) = model_linecol(&input, position);
        prop_assert_eq!(tracker.line, line);
        prop_assert_eq!(tracker.line_start, line_start);
    }

    #[test]
    fn streaming_and_lookup_are_equivalent(
        input in prop::collection::vec(any::<u8>(), 0..512),
        position in 0usize..768,
    ) {
        let end = position.min(input.len());

        let mut stream_tracker = LineCol::new();
        for i in 0..end {
            let next = input.get(i + 1).copied();
            stream_tracker.process_byte(input[i], next, i);
        }

        let direct_tracker = LineCol::at_position(&input, position);
        prop_assert_eq!(stream_tracker, direct_tracker);
    }

    #[test]
    fn column_is_saturating_and_consistent(
        input in prop::collection::vec(any::<u8>(), 0..512),
        position in 0usize..768,
    ) {
        let end = position.min(input.len());
        let tracker = LineCol::at_position(&input, position);
        let col = tracker.column(end);

        prop_assert_eq!(col, end.saturating_sub(tracker.line_start));
        prop_assert!(col <= end);
    }

    #[test]
    fn empty_input_always_line_zero(position in 0usize..128) {
        let tracker = LineCol::at_position(b"", position);
        prop_assert_eq!(tracker.line, 0);
        prop_assert_eq!(tracker.line_start, 0);
        prop_assert_eq!(tracker.column(0), 0);
    }

    #[test]
    fn line_count_monotonically_nondecreasing(
        input in prop::collection::vec(any::<u8>(), 1..256),
    ) {
        let mut prev_line = 0usize;
        for pos in 0..=input.len() {
            let tracker = LineCol::at_position(&input, pos);
            prop_assert!(tracker.line >= prev_line,
                "line decreased from {} to {} at position {}", prev_line, tracker.line, pos);
            prev_line = tracker.line;
        }
    }

    #[test]
    fn line_start_always_leq_position(
        input in prop::collection::vec(any::<u8>(), 0..512),
        position in 0usize..768,
    ) {
        let end = position.min(input.len());
        let tracker = LineCol::at_position(&input, position);
        prop_assert!(tracker.line_start <= end,
            "line_start {} > clamped position {}", tracker.line_start, end);
    }

    #[test]
    fn position_beyond_input_clamps(
        input in prop::collection::vec(any::<u8>(), 0..256),
        extra in 0usize..512,
    ) {
        let beyond = input.len() + extra;
        let at_end = LineCol::at_position(&input, input.len());
        let at_beyond = LineCol::at_position(&input, beyond);
        prop_assert_eq!(at_end, at_beyond);
    }

    #[test]
    fn multibyte_utf8_does_not_produce_false_newlines(s in "\\PC{0,128}") {
        let input = s.as_bytes();
        let newline_count = input.iter().filter(|&&b| b == b'\n').count()
            + input.iter().enumerate().filter(|(i, b)| {
                **b == b'\r' && input.get(i + 1).copied() != Some(b'\n')
            }).count();
        // CRLF pairs count as one line ending on the \n byte
        let crlf_count = input.windows(2).filter(|w| w == &[b'\r', b'\n']).count();
        let expected_lines = newline_count + crlf_count;

        let tracker = LineCol::at_position(input, input.len());
        prop_assert_eq!(tracker.line, expected_lines,
            "line count mismatch for string with {} bytes", input.len());
    }

    #[test]
    fn at_position_idempotent(
        input in prop::collection::vec(any::<u8>(), 0..256),
        position in 0usize..512,
    ) {
        let a = LineCol::at_position(&input, position);
        let b = LineCol::at_position(&input, position);
        prop_assert_eq!(a, b);
    }
}
