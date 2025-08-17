//! Compile-time API contract tests using trybuild
//!
//! These tests ensure that ParseTable and other core types enforce their required fields
//! at compile time, preventing regression of critical ABI contracts.

#[test]
fn parse_table_requires_reverse_map() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/fixtures/missing_index_to_symbol.rs");
}

#[test]
fn parse_table_requires_all_fields() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/fixtures/missing_parse_table_fields.rs");
}
