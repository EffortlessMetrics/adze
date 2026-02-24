#![no_main]

use adze_concurrency_init_classifier_core::is_already_initialized_error;
use libfuzzer_sys::fuzz_target;

fn model_is_already_initialized_error(message: &str) -> bool {
    let message = message.to_ascii_lowercase();
    message.contains("global") && message.contains("already")
}

fuzz_target!(|data: &[u8]| {
    let message = String::from_utf8_lossy(data);
    let got = is_already_initialized_error(&message);
    let expected = model_is_already_initialized_error(&message);

    assert_eq!(got, expected);
});
