// Build system integration for external scanners
// This module provides functionality to discover and compile user-provided scanner implementations

use anyhow::{bail, Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

/// Scanner source file information
#[derive(Debug, Clone)]
pub struct ScannerSource {
    /// Path to the scanner source file
    pub path: PathBuf,
    /// Language of the scanner (C, C++, or Rust)
    pub language: ScannerLanguage,
    /// Name of the grammar this scanner belongs to
    pub grammar_name: String,
}

/// Supported scanner implementation languages
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ScannerLanguage {
    C,
    Cpp,
    Rust,
}

impl ScannerLanguage {
    /// Get the file extension for this language
    pub fn extension(&self) -> &'static str {
        match self {
            ScannerLanguage::C => "c",
            ScannerLanguage::Cpp => "cc",
            ScannerLanguage::Rust => "rs",
        }
    }
}

/// Scanner builder configuration
pub struct ScannerBuilder {
    /// Grammar name
    grammar_name: String,
    /// Source directory to search for scanner files
    src_dir: PathBuf,
    /// Output directory for compiled scanner
    out_dir: PathBuf,
}

impl ScannerBuilder {
    /// Create a new scanner builder
    pub fn new(grammar_name: impl Into<String>, src_dir: PathBuf, out_dir: PathBuf) -> Self {
        ScannerBuilder {
            grammar_name: grammar_name.into(),
            src_dir,
            out_dir,
        }
    }

    /// Find scanner source file in the source directory
    pub fn find_scanner(&self) -> Result<Option<ScannerSource>> {
        // Look for scanner files in standard locations
        let scanner_names = vec![
            "scanner.c".to_string(),
            "scanner.cc".to_string(),
            "scanner.cpp".to_string(),
            "scanner.rs".to_string(),
            format!("{}_scanner.c", self.grammar_name),
            format!("{}_scanner.cc", self.grammar_name),
            format!("{}_scanner.rs", self.grammar_name),
        ];

        for name in &scanner_names {
            let path = self.src_dir.join(name);
            if path.exists() {
                let language = match path.extension().and_then(|s| s.to_str()) {
                    Some("c") => ScannerLanguage::C,
                    Some("cc") | Some("cpp") => ScannerLanguage::Cpp,
                    Some("rs") => ScannerLanguage::Rust,
                    _ => continue,
                };

                return Ok(Some(ScannerSource {
                    path,
                    language,
                    grammar_name: self.grammar_name.clone(),
                }));
            }
        }

        Ok(None)
    }

    /// Build the scanner and generate integration code
    pub fn build(&self) -> Result<()> {
        let scanner = match self.find_scanner()? {
            Some(scanner) => scanner,
            None => {
                // No scanner found - that's OK, not all grammars need external scanners
                return Ok(());
            }
        };

        println!("cargo:rerun-if-changed={}", scanner.path.display());

        match scanner.language {
            ScannerLanguage::C | ScannerLanguage::Cpp => {
                self.build_c_scanner(&scanner)?;
            }
            ScannerLanguage::Rust => {
                self.build_rust_scanner(&scanner)?;
            }
        }

        Ok(())
    }

    /// Build a C/C++ scanner
    fn build_c_scanner(&self, scanner: &ScannerSource) -> Result<()> {
        // Use cc crate to compile the scanner
        let mut build = cc::Build::new();

        build
            .file(&scanner.path)
            .include(&self.src_dir)
            .warnings(false);

        if scanner.language == ScannerLanguage::Cpp {
            build.cpp(true);
        }

        // Set output name based on grammar
        let lib_name = format!("{}_scanner", self.grammar_name);
        build.compile(&lib_name);

        // Generate Rust bindings
        self.generate_c_bindings(scanner)?;

        Ok(())
    }

    /// Generate Rust bindings for C scanner
    fn generate_c_bindings(&self, _scanner: &ScannerSource) -> Result<()> {
        let bindings_path = self
            .out_dir
            .join(format!("{}_scanner_bindings.rs", self.grammar_name));

        let bindings = format!(
            r#"
// Auto-generated bindings for {} scanner
use rust_sitter::external_scanner_ffi::{{TSExternalScannerData, CreateFn, DestroyFn, ScanFn, SerializeFn, DeserializeFn}};

extern "C" {{
    fn tree_sitter_{}_external_scanner_create() -> *mut std::ffi::c_void;
    fn tree_sitter_{}_external_scanner_destroy(payload: *mut std::ffi::c_void);
    fn tree_sitter_{}_external_scanner_scan(
        payload: *mut std::ffi::c_void,
        lexer: *mut rust_sitter::external_scanner_ffi::TSLexer,
        valid_symbols: *const bool,
    ) -> bool;
    fn tree_sitter_{}_external_scanner_serialize(
        payload: *mut std::ffi::c_void,
        buffer: *mut std::os::raw::c_char,
    ) -> std::os::raw::c_uint;
    fn tree_sitter_{}_external_scanner_deserialize(
        payload: *mut std::ffi::c_void,
        buffer: *const std::os::raw::c_char,
        length: std::os::raw::c_uint,
    );
}}

/// Get the external scanner data for this grammar
pub fn get_external_scanner_data() -> TSExternalScannerData {{
    TSExternalScannerData {{
        states: std::ptr::null(),
        symbol_map: std::ptr::null(),
        create: Some(tree_sitter_{}_external_scanner_create as CreateFn),
        destroy: Some(tree_sitter_{}_external_scanner_destroy as DestroyFn),
        scan: Some(tree_sitter_{}_external_scanner_scan as ScanFn),
        serialize: Some(tree_sitter_{}_external_scanner_serialize as SerializeFn),
        deserialize: Some(tree_sitter_{}_external_scanner_deserialize as DeserializeFn),
    }}
}}

/// Register this scanner with the global registry
pub fn register_scanner(external_tokens: Vec<rust_sitter::SymbolId>) {{
    let data = get_external_scanner_data();
    rust_sitter::scanner_registry::register_c_scanner(
        "{}",
        data,
        external_tokens,
    );
}}
"#,
            self.grammar_name,
            self.grammar_name,
            self.grammar_name,
            self.grammar_name,
            self.grammar_name,
            self.grammar_name,
            self.grammar_name,
            self.grammar_name,
            self.grammar_name,
            self.grammar_name,
            self.grammar_name,
            self.grammar_name
        );

        fs::write(&bindings_path, bindings)
            .with_context(|| format!("Failed to write scanner bindings to {:?}", bindings_path))?;

        println!(
            "cargo:rustc-env=RUST_SITTER_SCANNER_BINDINGS_{}",
            self.grammar_name.to_uppercase()
        );

        Ok(())
    }

    /// Build a Rust scanner
    fn build_rust_scanner(&self, scanner: &ScannerSource) -> Result<()> {
        // For Rust scanners, generate code to register them
        let registration_path = self
            .out_dir
            .join(format!("{}_scanner_registration.rs", self.grammar_name));

        // Read the scanner file to extract the scanner struct name
        let scanner_content = fs::read_to_string(&scanner.path)
            .with_context(|| format!("Failed to read scanner file {:?}", scanner.path))?;

        // Simple heuristic to find the scanner struct name
        let scanner_struct = self.find_scanner_struct(&scanner_content)?;

        let registration = format!(
            r#"
// Auto-generated registration for {} Rust scanner
use rust_sitter::scanner_registry::ExternalScannerBuilder;

include!({:?});

/// Register this scanner with the global registry
pub fn register_scanner() {{
    ExternalScannerBuilder::new("{}")
        .register_rust::<{}>();
}}
"#,
            self.grammar_name,
            scanner.path.display(),
            self.grammar_name,
            scanner_struct
        );

        fs::write(&registration_path, registration).with_context(|| {
            format!(
                "Failed to write scanner registration to {:?}",
                registration_path
            )
        })?;

        Ok(())
    }

    /// Find the scanner struct name in Rust code
    fn find_scanner_struct(&self, content: &str) -> Result<String> {
        // Look for "impl ExternalScanner for StructName"
        for line in content.lines() {
            if line.contains("impl ExternalScanner for") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 4 {
                    return Ok(parts[3].trim_end_matches('{').to_string());
                }
            }
        }

        // Fallback: look for struct definitions with "Scanner" in the name
        for line in content.lines() {
            if line.trim().starts_with("pub struct") && line.contains("Scanner") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 3 {
                    return Ok(parts[2].trim_end_matches('{').to_string());
                }
            }
        }

        bail!("Could not find scanner struct in {:?}", self.src_dir)
    }
}

/// Helper function to build scanners in build.rs
pub fn build_scanner(grammar_name: &str) -> Result<()> {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").context("CARGO_MANIFEST_DIR not set")?;
    let out_dir = std::env::var("OUT_DIR").context("OUT_DIR not set")?;

    let src_dir = Path::new(&manifest_dir).join("src");
    let out_dir = PathBuf::from(out_dir);

    let builder = ScannerBuilder::new(grammar_name, src_dir, out_dir);
    builder.build()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_find_scanner() {
        let temp_dir = TempDir::new().unwrap();
        let src_dir = temp_dir.path().to_path_buf();

        // Create a test scanner file
        fs::write(src_dir.join("scanner.c"), "// test scanner").unwrap();

        let builder = ScannerBuilder::new("test", src_dir, PathBuf::new());
        let scanner = builder.find_scanner().unwrap().unwrap();

        assert_eq!(scanner.language, ScannerLanguage::C);
        assert_eq!(scanner.grammar_name, "test");
    }

    #[test]
    fn test_find_scanner_struct() {
        let builder = ScannerBuilder::new("test", PathBuf::new(), PathBuf::new());

        let content = r#"
pub struct MyScanner {
    state: u32,
}

impl ExternalScanner for MyScanner {
    // implementation
}
"#;

        let struct_name = builder.find_scanner_struct(content).unwrap();
        assert_eq!(struct_name, "MyScanner");
    }
}
