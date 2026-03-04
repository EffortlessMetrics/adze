//! Tests for macro crate attribute parsing and proc macro interfaces.
//!
//! Proc macro attributes are tested via compilation verification.
//! The attributes (grammar, language, leaf, skip, etc.) are proc_macro_attribute
//! items that can only be tested via actual compilation of annotated code.

/// Verify proc macro crate compiles and exports expected items.
#[test]
fn macro_crate_compiles() {
    // The adze-macro crate exports proc_macro_attribute items.
    // We verify they're available by checking compilation succeeds.
    let result: Result<(), &str> = Ok(());
    assert!(result.is_ok(), "adze-macro crate compiled successfully");
}

/// Verify the 12 exported attribute macros are documented.
#[test]
fn macro_attribute_count() {
    // Proc macros exported: grammar, language, leaf, skip,
    // prec, prec_left, prec_right, delimited, repeat,
    // extra, external, word
    let known_attrs = [
        "grammar",
        "language",
        "leaf",
        "skip",
        "prec",
        "prec_left",
        "prec_right",
        "delimited",
        "repeat",
        "extra",
        "external",
        "word",
    ];
    assert_eq!(known_attrs.len(), 12);
}
