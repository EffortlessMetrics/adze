#[inline]
pub(crate) fn unexpected_action(action: &rust_sitter_glr_core::Action, where_: &str) {
    debug_assert!(
        matches!(
            action,
            rust_sitter_glr_core::Action::Shift(_)
                | rust_sitter_glr_core::Action::Reduce(_)
                | rust_sitter_glr_core::Action::Accept
                | rust_sitter_glr_core::Action::Error
                | rust_sitter_glr_core::Action::Recover
                | rust_sitter_glr_core::Action::Fork(_)
        ),
        "Unexpected action variant in {where_}: {action:?}"
    );
    // TODO: Add log-warns feature to Cargo.toml if needed
    // #[cfg(feature = "log-warns")]
    // {
    //     use std::sync::Once;
    //     static WARN: Once = Once::new();
    //     WARN.call_once(|| log::warn!("Unknown Action variant seen in {where_}; treating as error"));
    // }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_sitter_glr_core::Action;
    use rust_sitter_ir::{RuleId, StateId};

    #[test]
    fn unexpected_action_known_variant_no_panic() {
        // Should not panic or log in debug; it only warns on unknown variants.
        unexpected_action(&Action::Error, "util-test");
        unexpected_action(&Action::Shift(StateId(0)), "util-test");
        unexpected_action(&Action::Reduce(RuleId(0)), "util-test");
        unexpected_action(&Action::Accept, "util-test");
    }
}
