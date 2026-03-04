//! Comprehensive tests for scanner_build module.
//!
//! Tests scanner discovery, language detection, builder config, and generated code.

use adze_tool::scanner_build::*;
use std::path::PathBuf;
use tempfile::TempDir;

// ── 1. ScannerLanguage enum ─────────────────────────────────────

#[test]
fn test_scanner_language_c_extension() {
    assert_eq!(ScannerLanguage::C.extension(), "c");
}

#[test]
fn test_scanner_language_cpp_extension() {
    assert_eq!(ScannerLanguage::Cpp.extension(), "cc");
}

#[test]
fn test_scanner_language_rust_extension() {
    assert_eq!(ScannerLanguage::Rust.extension(), "rs");
}

#[test]
fn test_scanner_language_equality() {
    assert_eq!(ScannerLanguage::C, ScannerLanguage::C);
    assert_eq!(ScannerLanguage::Cpp, ScannerLanguage::Cpp);
    assert_eq!(ScannerLanguage::Rust, ScannerLanguage::Rust);
    assert_ne!(ScannerLanguage::C, ScannerLanguage::Rust);
}

#[test]
fn test_scanner_language_debug() {
    assert!(format!("{:?}", ScannerLanguage::C).contains("C"));
    assert!(format!("{:?}", ScannerLanguage::Cpp).contains("Cpp"));
    assert!(format!("{:?}", ScannerLanguage::Rust).contains("Rust"));
}

#[test]
fn test_scanner_language_clone() {
    let lang = ScannerLanguage::Rust;
    let cloned = lang;
    assert_eq!(lang, cloned);
}

#[test]
fn test_scanner_language_copy() {
    let lang = ScannerLanguage::C;
    let copied = lang;
    assert_eq!(lang, copied);
}

// ── 2. ScannerSource struct ─────────────────────────────────────

#[test]
fn test_scanner_source_construction() {
    let src = ScannerSource {
        path: PathBuf::from("scanner.rs"),
        language: ScannerLanguage::Rust,
        grammar_name: "test".to_string(),
    };
    assert_eq!(src.path, PathBuf::from("scanner.rs"));
    assert_eq!(src.language, ScannerLanguage::Rust);
    assert_eq!(src.grammar_name, "test");
}

#[test]
fn test_scanner_source_clone() {
    let src = ScannerSource {
        path: PathBuf::from("scanner.c"),
        language: ScannerLanguage::C,
        grammar_name: "json".to_string(),
    };
    let cloned = src.clone();
    assert_eq!(cloned.path, src.path);
    assert_eq!(cloned.language, src.language);
    assert_eq!(cloned.grammar_name, src.grammar_name);
}

#[test]
fn test_scanner_source_debug() {
    let src = ScannerSource {
        path: PathBuf::from("scanner.cc"),
        language: ScannerLanguage::Cpp,
        grammar_name: "cpp".to_string(),
    };
    let debug = format!("{:?}", src);
    assert!(debug.contains("ScannerSource"));
    assert!(debug.contains("scanner.cc"));
}

// ── 3. ScannerBuilder construction ──────────────────────────────

#[test]
fn test_scanner_builder_new() {
    let builder = ScannerBuilder::new("test", PathBuf::from("/src"), PathBuf::from("/out"));
    let _ = builder;
}

#[test]
fn test_scanner_builder_new_with_string() {
    let builder = ScannerBuilder::new(
        String::from("my_grammar"),
        PathBuf::from("/src"),
        PathBuf::from("/out"),
    );
    let _ = builder;
}

// ── 4. Scanner discovery ─────────────────────────────────────────

#[test]
fn test_find_scanner_empty_dir() {
    let dir = TempDir::new().unwrap();
    let builder = ScannerBuilder::new("test", dir.path().to_path_buf(), dir.path().to_path_buf());
    let result = builder.find_scanner().unwrap();
    assert!(result.is_none(), "empty dir should have no scanner");
}

#[test]
fn test_find_scanner_c() {
    let dir = TempDir::new().unwrap();
    std::fs::write(dir.path().join("scanner.c"), "// C scanner").unwrap();
    let builder = ScannerBuilder::new("test", dir.path().to_path_buf(), dir.path().to_path_buf());
    let result = builder.find_scanner().unwrap();
    assert!(result.is_some());
    let src = result.unwrap();
    assert_eq!(src.language, ScannerLanguage::C);
    assert!(src.path.ends_with("scanner.c"));
}

#[test]
fn test_find_scanner_cpp() {
    let dir = TempDir::new().unwrap();
    std::fs::write(dir.path().join("scanner.cc"), "// C++ scanner").unwrap();
    let builder = ScannerBuilder::new("test", dir.path().to_path_buf(), dir.path().to_path_buf());
    let result = builder.find_scanner().unwrap();
    assert!(result.is_some());
    assert_eq!(result.unwrap().language, ScannerLanguage::Cpp);
}

#[test]
fn test_find_scanner_rust() {
    let dir = TempDir::new().unwrap();
    std::fs::write(dir.path().join("scanner.rs"), "// Rust scanner").unwrap();
    let builder = ScannerBuilder::new("test", dir.path().to_path_buf(), dir.path().to_path_buf());
    let result = builder.find_scanner().unwrap();
    assert!(result.is_some());
    assert_eq!(result.unwrap().language, ScannerLanguage::Rust);
}

#[test]
fn test_find_scanner_named_grammar() {
    let dir = TempDir::new().unwrap();
    std::fs::write(dir.path().join("json_scanner.c"), "// scanner").unwrap();
    let builder = ScannerBuilder::new("json", dir.path().to_path_buf(), dir.path().to_path_buf());
    let result = builder.find_scanner().unwrap();
    assert!(result.is_some());
    let src = result.unwrap();
    assert_eq!(src.grammar_name, "json");
    assert_eq!(src.language, ScannerLanguage::C);
}

#[test]
fn test_find_scanner_priority_c_first() {
    let dir = TempDir::new().unwrap();
    // Create both scanner.c and scanner.rs — C should be found first
    std::fs::write(dir.path().join("scanner.c"), "// C").unwrap();
    std::fs::write(dir.path().join("scanner.rs"), "// Rust").unwrap();
    let builder = ScannerBuilder::new("test", dir.path().to_path_buf(), dir.path().to_path_buf());
    let result = builder.find_scanner().unwrap();
    assert!(result.is_some());
    // scanner.c should be found first due to iteration order
    assert_eq!(result.unwrap().language, ScannerLanguage::C);
}

#[test]
fn test_find_scanner_nonexistent_dir() {
    let builder = ScannerBuilder::new(
        "test",
        PathBuf::from("/nonexistent/path/12345"),
        PathBuf::from("/tmp"),
    );
    let result = builder.find_scanner().unwrap();
    assert!(result.is_none());
}

// ── 5. Build with no scanner ────────────────────────────────────

#[test]
fn test_build_no_scanner_succeeds() {
    let dir = TempDir::new().unwrap();
    let builder = ScannerBuilder::new("test", dir.path().to_path_buf(), dir.path().to_path_buf());
    // Build with no scanner should succeed (no-op)
    let result = builder.build();
    assert!(result.is_ok());
}

// ── 6. Scanner source with various paths ────────────────────────

#[test]
fn test_scanner_source_with_nested_path() {
    let src = ScannerSource {
        path: PathBuf::from("/a/b/c/scanner.rs"),
        language: ScannerLanguage::Rust,
        grammar_name: "deep".to_string(),
    };
    assert!(src.path.ends_with("scanner.rs"));
}

#[test]
fn test_scanner_source_grammar_name_preserved() {
    let names = vec!["json", "python", "javascript", "c", "rust_grammar"];
    for name in names {
        let src = ScannerSource {
            path: PathBuf::from("scanner.c"),
            language: ScannerLanguage::C,
            grammar_name: name.to_string(),
        };
        assert_eq!(src.grammar_name, name);
    }
}

// ── 7. Multiple find_scanner calls ──────────────────────────────

#[test]
fn test_find_scanner_idempotent() {
    let dir = TempDir::new().unwrap();
    std::fs::write(dir.path().join("scanner.rs"), "// Rust").unwrap();
    let builder = ScannerBuilder::new("test", dir.path().to_path_buf(), dir.path().to_path_buf());
    let r1 = builder.find_scanner().unwrap();
    let r2 = builder.find_scanner().unwrap();
    assert!(r1.is_some() && r2.is_some());
    assert_eq!(r1.unwrap().language, r2.unwrap().language);
}
