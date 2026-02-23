#![no_main]

use adze_concurrency_parse_core::parse_positive_usize_or_default;
use libfuzzer_sys::fuzz_target;

fn model_parse(value: Option<&str>, default: usize) -> usize {
    value
        .and_then(|raw| raw.trim().parse::<usize>().ok())
        .filter(|parsed| *parsed > 0)
        .unwrap_or(default)
}

fuzz_target!(|data: &[u8]| {
    let default = usize::from(data.first().copied().unwrap_or(0));
    let raw = String::from_utf8_lossy(if data.len() > 1 { &data[1..] } else { &[] }).to_string();

    let got = parse_positive_usize_or_default(Some(&raw), default);
    let expected = model_parse(Some(&raw), default);

    assert_eq!(got, expected);
    assert_eq!(parse_positive_usize_or_default(None, default), default);
});
