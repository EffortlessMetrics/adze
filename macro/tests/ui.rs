//! Compile-fail tests for rust-sitter macros
//! 
//! These tests ensure that invalid grammar definitions produce
//! helpful error messages at compile time.

#[test]
fn ui() {
    let t = trybuild::TestCases::new();
    
    // Tests for invalid grammar definitions
    t.compile_fail("tests/ui/invalid_grammar_*.rs");
    
    // Tests for valid grammar definitions that should compile
    t.pass("tests/ui/valid_grammar_*.rs");
}