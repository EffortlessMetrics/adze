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
}
