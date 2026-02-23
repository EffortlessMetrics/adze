use adze_concurrency_init_classifier_core::is_already_initialized_error;
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
}
