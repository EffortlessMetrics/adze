//! Message classification helpers for Rayon global thread-pool initialization errors.

#![forbid(unsafe_op_in_unsafe_fn)]
#![deny(missing_docs)]
#![cfg_attr(feature = "strict_api", deny(unreachable_pub))]
#![cfg_attr(not(feature = "strict_api"), warn(unreachable_pub))]
#![cfg_attr(feature = "strict_docs", deny(missing_docs))]
#![cfg_attr(not(feature = "strict_docs"), allow(missing_docs))]

/// Return whether the provided message represents a Rayon global-pool
/// already-initialized error.
#[must_use]
pub fn is_already_initialized_error(message: &str) -> bool {
    let message = message.to_ascii_lowercase();
    message.contains("global") && message.contains("already")
}

#[cfg(test)]
mod tests {
    use super::is_already_initialized_error;

    #[test]
    fn already_initialized_error_classifier_is_case_insensitive() {
        assert!(is_already_initialized_error(
            "The GlObAl thread pool has AlReAdY been initialized"
        ));
    }

    #[test]
    fn already_initialized_error_classifier_requires_both_tokens() {
        assert!(!is_already_initialized_error(
            "thread pool already initialized"
        ));
        assert!(!is_already_initialized_error(
            "global thread pool initialized"
        ));
    }

    #[test]
    fn empty_string_is_not_already_initialized_error() {
        assert!(!is_already_initialized_error(""));
    }

    #[test]
    fn unrelated_message_is_not_detected() {
        assert!(!is_already_initialized_error("something went wrong"));
    }

    #[test]
    fn reversed_token_order_is_still_detected() {
        assert!(is_already_initialized_error(
            "already set on the global pool"
        ));
    }

    #[test]
    fn adjacent_tokens_are_detected() {
        assert!(is_already_initialized_error("globalalready"));
    }

    #[test]
    fn only_global_token_is_not_detected() {
        assert!(!is_already_initialized_error("global"));
    }

    #[test]
    fn only_already_token_is_not_detected() {
        assert!(!is_already_initialized_error("already"));
    }
}
