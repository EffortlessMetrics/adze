// Comprehensive tests for TableGenError and error handling in adze-tablegen.

use adze_tablegen::error::TableGenError;
use std::error::Error;

// ===== Variant construction =====

#[test]
fn invalid_input() {
    let e = TableGenError::InvalidInput("bad data");
    let s = format!("{}", e);
    assert!(s.contains("invalid input"));
    assert!(s.contains("bad data"));
}

#[test]
fn automaton_error() {
    let e = TableGenError::Automaton("build failed".to_string());
    let s = format!("{}", e);
    assert!(s.contains("automaton build failed"));
}

#[test]
fn compression_error() {
    let e = TableGenError::Compression("overflow".to_string());
    let s = format!("{}", e);
    assert!(s.contains("compression failed"));
}

#[test]
fn table_generation_error() {
    let e = TableGenError::TableGeneration("general failure".to_string());
    let s = format!("{}", e);
    assert!(s.contains("table generation failed"));
}

#[test]
fn invalid_table_error() {
    let e = TableGenError::InvalidTable("corrupted".to_string());
    let s = format!("{}", e);
    assert!(s.contains("invalid table structure"));
}

#[test]
fn invalid_symbol_index() {
    let e = TableGenError::InvalidSymbolIndex(42);
    let s = format!("{}", e);
    assert!(s.contains("symbol index out of bounds"));
    assert!(s.contains("42"));
}

#[test]
fn invalid_state_index() {
    let e = TableGenError::InvalidStateIndex(99);
    let s = format!("{}", e);
    assert!(s.contains("state index out of bounds"));
    assert!(s.contains("99"));
}

#[test]
fn empty_grammar_error() {
    let e = TableGenError::EmptyGrammar;
    let s = format!("{}", e);
    assert!(s.contains("empty grammar"));
}

#[test]
fn validation_error() {
    let e = TableGenError::ValidationError("missing start".to_string());
    let s = format!("{}", e);
    assert!(s.contains("grammar validation failed"));
}

// ===== Debug output =====

#[test]
fn debug_invalid_input() {
    let e = TableGenError::InvalidInput("test");
    let s = format!("{:?}", e);
    assert!(s.contains("InvalidInput"));
}

#[test]
fn debug_automaton() {
    let e = TableGenError::Automaton("msg".to_string());
    let s = format!("{:?}", e);
    assert!(s.contains("Automaton"));
}

#[test]
fn debug_compression() {
    let e = TableGenError::Compression("msg".to_string());
    let s = format!("{:?}", e);
    assert!(s.contains("Compression"));
}

#[test]
fn debug_empty_grammar() {
    let e = TableGenError::EmptyGrammar;
    let s = format!("{:?}", e);
    assert!(s.contains("EmptyGrammar"));
}

#[test]
fn debug_invalid_symbol_index() {
    let e = TableGenError::InvalidSymbolIndex(0);
    let s = format!("{:?}", e);
    assert!(s.contains("InvalidSymbolIndex"));
}

#[test]
fn debug_invalid_state_index() {
    let e = TableGenError::InvalidStateIndex(0);
    let s = format!("{:?}", e);
    assert!(s.contains("InvalidStateIndex"));
}

// ===== From conversions =====

#[test]
fn from_string() {
    let e: TableGenError = "some error".to_string().into();
    let s = format!("{}", e);
    assert!(s.contains("some error"));
}

#[test]
fn from_str() {
    let e: TableGenError = "str error".into();
    let s = format!("{}", e);
    assert!(s.contains("str error"));
}

#[test]
fn from_io_error() {
    let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
    let e: TableGenError = io_err.into();
    let s = format!("{}", e);
    assert!(s.contains("file not found"));
}

#[test]
fn from_io_error_permission() {
    let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "access denied");
    let e: TableGenError = io_err.into();
    let s = format!("{}", e);
    assert!(s.contains("access denied"));
}

// ===== Error trait =====

#[test]
fn implements_error_trait() {
    let e = TableGenError::EmptyGrammar;
    let _: &dyn Error = &e;
}

#[test]
fn error_source_none_for_simple() {
    let e = TableGenError::InvalidInput("test");
    assert!(e.source().is_none());
}

#[test]
fn error_source_for_io() {
    let io_err = std::io::Error::new(std::io::ErrorKind::Other, "io");
    let e: TableGenError = io_err.into();
    // #[error(transparent)] delegates Display and source; the inner io::Error
    // may or may not have its own source depending on construction
    let _ = e.source();
}

// ===== Edge cases =====

#[test]
fn empty_string_automaton() {
    let e = TableGenError::Automaton(String::new());
    let s = format!("{}", e);
    assert!(s.contains("automaton build failed"));
}

#[test]
fn empty_string_compression() {
    let e = TableGenError::Compression(String::new());
    let s = format!("{}", e);
    assert!(s.contains("compression failed"));
}

#[test]
fn empty_string_table_generation() {
    let e = TableGenError::TableGeneration(String::new());
    let s = format!("{}", e);
    assert!(s.contains("table generation failed"));
}

#[test]
fn empty_string_validation() {
    let e = TableGenError::ValidationError(String::new());
    let s = format!("{}", e);
    assert!(s.contains("grammar validation failed"));
}

#[test]
fn long_message() {
    let msg = "x".repeat(10_000);
    let e = TableGenError::TableGeneration(msg.clone());
    let s = format!("{}", e);
    assert!(s.len() > 10_000);
}

#[test]
fn special_chars_in_message() {
    let e = TableGenError::Automaton("error: 'unexpected' \"token\" <here>".to_string());
    let s = format!("{}", e);
    assert!(s.contains("unexpected"));
}

#[test]
fn unicode_in_message() {
    let e = TableGenError::Compression("圧縮エラー".to_string());
    let s = format!("{}", e);
    assert!(s.contains("圧縮エラー"));
}

#[test]
fn invalid_symbol_index_zero() {
    let e = TableGenError::InvalidSymbolIndex(0);
    let s = format!("{}", e);
    assert!(s.contains("0"));
}

#[test]
fn invalid_symbol_index_max() {
    let e = TableGenError::InvalidSymbolIndex(usize::MAX);
    let s = format!("{}", e);
    assert!(s.contains(&format!("{}", usize::MAX)));
}

#[test]
fn invalid_state_index_zero() {
    let e = TableGenError::InvalidStateIndex(0);
    let s = format!("{}", e);
    assert!(s.contains("0"));
}

#[test]
fn invalid_state_index_max() {
    let e = TableGenError::InvalidStateIndex(usize::MAX);
    let s = format!("{}", e);
    assert!(s.contains(&format!("{}", usize::MAX)));
}

// ===== Newlines and whitespace in messages =====

#[test]
fn message_with_newlines() {
    let e = TableGenError::TableGeneration("line1\nline2\nline3".to_string());
    let s = format!("{}", e);
    assert!(s.contains("line1"));
    assert!(s.contains("line3"));
}

#[test]
fn message_with_tabs() {
    let e = TableGenError::InvalidTable("\ttabbed\t".to_string());
    let s = format!("{}", e);
    assert!(s.contains("tabbed"));
}

// ===== Result type alias =====

#[test]
fn result_ok() {
    let r: adze_tablegen::error::Result<i32> = Ok(42);
    assert_eq!(r.unwrap(), 42);
}

#[test]
fn result_err() {
    let r: adze_tablegen::error::Result<i32> = Err(TableGenError::EmptyGrammar);
    assert!(r.is_err());
}

#[test]
fn result_from_string() {
    fn might_fail() -> adze_tablegen::error::Result<()> {
        Err("failed".into())
    }
    assert!(might_fail().is_err());
}

// ===== All variants display contains relevant text =====

#[test]
fn all_variants_have_nonempty_display() {
    let variants: Vec<TableGenError> = vec![
        TableGenError::InvalidInput("a"),
        TableGenError::Automaton("b".to_string()),
        TableGenError::Compression("c".to_string()),
        TableGenError::TableGeneration("d".to_string()),
        TableGenError::InvalidTable("e".to_string()),
        TableGenError::InvalidSymbolIndex(1),
        TableGenError::InvalidStateIndex(2),
        TableGenError::EmptyGrammar,
        TableGenError::ValidationError("f".to_string()),
    ];
    for e in &variants {
        let s = format!("{}", e);
        assert!(!s.is_empty(), "Display should not be empty for {:?}", e);
    }
}

#[test]
fn all_variants_have_nonempty_debug() {
    let variants: Vec<TableGenError> = vec![
        TableGenError::InvalidInput("a"),
        TableGenError::Automaton("b".to_string()),
        TableGenError::Compression("c".to_string()),
        TableGenError::TableGeneration("d".to_string()),
        TableGenError::InvalidTable("e".to_string()),
        TableGenError::InvalidSymbolIndex(1),
        TableGenError::InvalidStateIndex(2),
        TableGenError::EmptyGrammar,
        TableGenError::ValidationError("f".to_string()),
    ];
    for e in &variants {
        let s = format!("{:?}", e);
        assert!(!s.is_empty(), "Debug should not be empty");
    }
}
