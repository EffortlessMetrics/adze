use adze_concurrency_init_core::{
    init_concurrency_caps, init_rayon_global_once, is_already_initialized_error,
};
use proptest::prelude::*;

fn model_is_already_initialized_error(message: &str) -> bool {
    let message = message.to_ascii_lowercase();
    message.contains("global") && message.contains("already")
}

fn masked_case(input: &str, mask: u64) -> String {
    let mut output = String::with_capacity(input.len());
    for (index, byte) in input.bytes().enumerate() {
        let upper = ((mask >> (index % 63)) & 1) == 1;
        let ch = if upper {
            byte.to_ascii_uppercase()
        } else {
            byte.to_ascii_lowercase()
        };
        output.push(char::from(ch));
    }
    output
}

proptest! {
    #[test]
    fn classifier_matches_model_for_arbitrary_messages(
        bytes in prop::collection::vec(any::<u8>(), 0..512),
    ) {
        let message = String::from_utf8_lossy(&bytes);
        let expected = model_is_already_initialized_error(&message);
        prop_assert_eq!(is_already_initialized_error(&message), expected);
    }

    #[test]
    fn classifier_is_case_insensitive_for_required_keywords(
        prefix in "[ -~]{0,64}",
        middle in "[ -~]{0,32}",
        suffix in "[ -~]{0,64}",
        global_mask in any::<u64>(),
        already_mask in any::<u64>(),
    ) {
        let global = masked_case("global", global_mask);
        let already = masked_case("already", already_mask);
        let message = format!("{prefix}{global}{middle}{already}{suffix}");

        prop_assert!(is_already_initialized_error(&message));
    }

    #[test]
    fn repeated_init_calls_never_panic(call_count in 0usize..1024) {
        for _ in 0..call_count {
            init_concurrency_caps();
        }
    }

    #[test]
    fn repeated_low_level_init_calls_never_return_error(
        thread_counts in prop::collection::vec(0usize..256, 0..256),
    ) {
        for thread_count in thread_counts {
            prop_assert!(init_rayon_global_once(thread_count).is_ok());
        }
    }
}
