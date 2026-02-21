//! Compile-fail tests for adze macros
//!
//! These tests ensure that invalid grammar definitions produce
//! helpful error messages at compile time.

#[test]
#[ignore = "UI test infrastructure needs adze crate dependency configuration in trybuild environment"]
fn ui() {
    let t = trybuild::TestCases::new();

    // Tests for invalid grammar definitions
    t.compile_fail("tests/ui/invalid_grammar_*.rs");
    // TODO: Re-enable valid grammar tests once macros stabilize
    // t.pass("tests/ui/valid_grammar_*.rs");
}
