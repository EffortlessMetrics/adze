//! Comprehensive tests for the `adze_tablegen::generate` module.
//!
//! Tests cover `LanguageBuilder` creation, `generate_language()`, symbol names,
//! field names, external tokens, symbol metadata, code generation, and
//! the `set_start_can_be_empty` flag.

use adze_glr_core::ParseTable;
use adze_ir::builder::GrammarBuilder;
use adze_ir::{ExternalToken, FieldId, Grammar, SymbolId, Token, TokenPattern};
use adze_tablegen::generate::LanguageBuilder;
use std::ffi::CStr;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Build a minimal grammar + parse table pair suitable for LanguageBuilder.
fn minimal_grammar_and_table() -> (Grammar, ParseTable) {
    let grammar = GrammarBuilder::new("minimal")
        .token("number", r"\d+")
        .rule("expr", vec!["number"])
        .start("expr")
        .build();

    let table = ParseTable::default();
    (grammar, table)
}

/// Read a C-string pointer into a Rust &str.
///
/// # Safety
/// The caller must ensure `ptr` is valid and null-terminated.
unsafe fn cstr_from_ptr(ptr: *const i8) -> &'static str {
    // SAFETY: upheld by caller; pointers originate from leaked CStrings.
    unsafe { CStr::from_ptr(ptr).to_str().expect("valid UTF-8") }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[test]
fn test_builder_creation_default_state() {
    let (grammar, table) = minimal_grammar_and_table();
    let builder = LanguageBuilder::new(grammar, table);

    // Should successfully generate a language without panicking.
    let lang = builder
        .generate_language()
        .expect("generate_language failed");
    assert_eq!(lang.version, 15, "ABI version must be 15");
}

#[test]
fn test_generate_language_counts() {
    let grammar = GrammarBuilder::new("counts")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a", "b"])
        .start("start")
        .build();

    let table = ParseTable::default();
    let builder = LanguageBuilder::new(grammar, table);
    let lang = builder
        .generate_language()
        .expect("generate_language failed");

    // token_count = grammar.tokens.len() + 1 (for EOF)
    assert_eq!(lang.token_count, 3, "2 tokens + EOF");
    assert_eq!(lang.external_token_count, 0);
    assert_eq!(lang.field_count, 0, "no fields declared");
}

#[test]
fn test_symbol_names_include_tokens() {
    let grammar = GrammarBuilder::new("sym")
        .token("identifier", r"[a-z]+")
        .rule("root", vec!["identifier"])
        .start("root")
        .build();

    let table = ParseTable::default();
    let builder = LanguageBuilder::new(grammar, table);
    let lang = builder
        .generate_language()
        .expect("generate_language failed");

    // symbol_names should be non-null and contain "identifier".
    // The names array length = tokens + rules + externals (built by build_symbol_names).
    assert!(
        !lang.symbol_names.is_null(),
        "symbol_names must not be null"
    );
    let num_tokens = 1; // "identifier"
    let num_rules = 1; // "root"
    let total_names = num_tokens + num_rules;
    let names: Vec<&str> = (0..total_names)
        .filter_map(|i| {
            // SAFETY: pointers come from build_symbol_names, which leaks CStrings.
            let ptr = unsafe { *lang.symbol_names.add(i) };
            if ptr.is_null() {
                None
            } else {
                Some(unsafe { cstr_from_ptr(ptr) })
            }
        })
        .collect();

    assert!(
        names.contains(&"identifier"),
        "expected 'identifier' in symbol names, got: {names:?}"
    );
}

#[test]
fn test_field_names_with_fields() {
    let mut grammar = GrammarBuilder::new("fields")
        .token("num", r"\d+")
        .rule("pair", vec!["num", "num"])
        .start("pair")
        .build();

    // Manually add fields (GrammarBuilder doesn't expose field API).
    grammar.fields.insert(FieldId(0), "left".to_string());
    grammar.fields.insert(FieldId(1), "right".to_string());

    let table = ParseTable::default();
    let builder = LanguageBuilder::new(grammar, table);
    let lang = builder
        .generate_language()
        .expect("generate_language failed");

    assert_eq!(lang.field_count, 2);
    assert!(
        !lang.field_names.is_null(),
        "field_names must not be null when fields exist"
    );

    // First entry is always empty string; then the user fields.
    let first = unsafe { cstr_from_ptr(*lang.field_names.add(0)) };
    assert_eq!(first, "", "first field name entry must be empty string");

    // Remaining entries should include "left" and "right".
    let field1 = unsafe { cstr_from_ptr(*lang.field_names.add(1)) };
    let field2 = unsafe { cstr_from_ptr(*lang.field_names.add(2)) };
    let field_set: Vec<&str> = vec![field1, field2];
    assert!(
        field_set.contains(&"left"),
        "expected 'left' in fields, got: {field_set:?}"
    );
    assert!(
        field_set.contains(&"right"),
        "expected 'right' in fields, got: {field_set:?}"
    );
}

#[test]
fn test_field_names_null_when_no_fields() {
    let (grammar, table) = minimal_grammar_and_table();
    let builder = LanguageBuilder::new(grammar, table);
    let lang = builder
        .generate_language()
        .expect("generate_language failed");

    assert_eq!(lang.field_count, 0);
    assert!(
        lang.field_names.is_null(),
        "field_names should be null when there are no fields"
    );
}

#[test]
fn test_externals_counted() {
    let mut grammar = GrammarBuilder::new("ext")
        .token("id", r"[a-z]+")
        .rule("root", vec!["id"])
        .start("root")
        .build();

    grammar.externals.push(ExternalToken {
        name: "comment".to_string(),
        symbol_id: SymbolId(100),
    });
    grammar.externals.push(ExternalToken {
        name: "string".to_string(),
        symbol_id: SymbolId(101),
    });

    let table = ParseTable::default();
    let builder = LanguageBuilder::new(grammar, table);
    let lang = builder
        .generate_language()
        .expect("generate_language failed");

    assert_eq!(
        lang.external_token_count, 2,
        "should count two external tokens"
    );
}

#[test]
fn test_symbol_metadata_starts_with_eof() {
    let (grammar, table) = minimal_grammar_and_table();
    let builder = LanguageBuilder::new(grammar, table);
    let lang = builder
        .generate_language()
        .expect("generate_language failed");

    // symbol_metadata must not be null and the first entry is EOF (invisible, unnamed).
    assert!(
        !lang.symbol_metadata.is_null(),
        "symbol_metadata must not be null"
    );
    let eof_meta = unsafe { &*lang.symbol_metadata };
    assert!(!eof_meta.visible, "EOF symbol must not be visible");
    assert!(!eof_meta.named, "EOF symbol must not be named");
}

#[test]
fn test_set_start_can_be_empty() {
    let (grammar, table) = minimal_grammar_and_table();
    let mut builder = LanguageBuilder::new(grammar, table);

    // Default should work.
    builder.set_start_can_be_empty(true);
    let lang = builder
        .generate_language()
        .expect("generate_language failed");
    assert_eq!(lang.version, 15);
}

#[test]
fn test_generate_language_code_produces_tokens() {
    let (grammar, table) = minimal_grammar_and_table();
    let builder = LanguageBuilder::new(grammar, table);
    let code = builder.generate_language_code();
    let code_str = code.to_string();

    assert!(
        code_str.contains("language"),
        "generated code must contain 'language'"
    );
    assert!(
        code_str.contains("TSLanguage"),
        "generated code must reference TSLanguage"
    );
}

#[test]
fn test_multiple_tokens_all_named() {
    let grammar = GrammarBuilder::new("multi")
        .token("alpha", r"[a-z]+")
        .token("digit", r"\d+")
        .token("underscore", "_")
        .rule("root", vec!["alpha"])
        .start("root")
        .build();

    let table = ParseTable::default();
    let builder = LanguageBuilder::new(grammar, table);
    let lang = builder
        .generate_language()
        .expect("generate_language failed");

    // token_count = 3 user tokens + 1 EOF
    assert_eq!(lang.token_count, 4);
}

#[test]
fn test_hidden_token_metadata() {
    // Tokens whose name starts with '_' should be invisible.
    let mut grammar = GrammarBuilder::new("hidden")
        .token("visible_tok", "x")
        .rule("root", vec!["visible_tok"])
        .start("root")
        .build();

    // Manually insert a hidden token.
    grammar.tokens.insert(
        SymbolId(50),
        Token {
            name: "_hidden".to_string(),
            pattern: TokenPattern::String("h".to_string()),
            fragile: false,
        },
    );

    let table = ParseTable::default();
    let builder = LanguageBuilder::new(grammar, table);
    let lang = builder
        .generate_language()
        .expect("generate_language failed");

    // We just verify generation succeeds with hidden tokens.
    assert!(lang.token_count >= 2);
}
