#![no_main]

use adze_linecol_core::LineCol;
use libfuzzer_sys::fuzz_target;

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

fuzz_target!(|data: &[u8]| {
    if data.len() < 8 {
        return;
    }

    let mut pos_bytes = [0u8; 8];
    pos_bytes.copy_from_slice(&data[..8]);
    let raw_position = u64::from_le_bytes(pos_bytes) as usize;

    let input = &data[8..];
    let bounded_position = if input.is_empty() {
        0
    } else {
        raw_position % (input.len() + 1)
    };

    let tracker = LineCol::at_position(input, bounded_position);
    let (expected_line, expected_line_start) = model_linecol(input, bounded_position);
    assert_eq!(tracker.line, expected_line);
    assert_eq!(tracker.line_start, expected_line_start);

    let mut stream_tracker = LineCol::new();
    for i in 0..bounded_position {
        stream_tracker.process_byte(input[i], input.get(i + 1).copied(), i);
    }
    assert_eq!(stream_tracker, tracker);

    let end = bounded_position.min(input.len());
    assert_eq!(tracker.column(end), end.saturating_sub(tracker.line_start));
});
