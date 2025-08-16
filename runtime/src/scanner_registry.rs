// External scanner registry for rust-sitter
// This module provides a registry for external scanners that can be used by parsers

use crate::external_scanner::{ExternalScanner, ScanResult};
use crate::external_scanner_ffi::{CExternalScanner, TSExternalScannerData};
use once_cell::sync::Lazy;
use rust_sitter_ir::SymbolId;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Type-erased external scanner
pub trait DynExternalScanner: Send + Sync {
    /// Scan for external tokens
    fn scan(
        &mut self,
        _lexer: &mut dyn crate::external_scanner::Lexer,
        valid_symbols: &[bool],
    ) -> Option<ScanResult>;

    /// Serialize scanner state
    fn serialize(&self, buffer: &mut Vec<u8>);

    /// Deserialize scanner state
    fn deserialize(&mut self, buffer: &[u8]);
}

/// Wrapper for Rust external scanners
struct RustScannerWrapper<S: ExternalScanner> {
    scanner: S,
}

impl<S: ExternalScanner> DynExternalScanner for RustScannerWrapper<S> {
    fn scan(
        &mut self,
        lexer: &mut dyn crate::external_scanner::Lexer,
        valid_symbols: &[bool],
    ) -> Option<ScanResult> {
        self.scanner.scan(lexer, valid_symbols)
    }

    fn serialize(&self, buffer: &mut Vec<u8>) {
        self.scanner.serialize(buffer)
    }

    fn deserialize(&mut self, buffer: &[u8]) {
        self.scanner.deserialize(buffer)
    }
}

/// Wrapper for C external scanners
struct CScannerWrapper {
    scanner: CExternalScanner,
    external_tokens: Vec<SymbolId>,
}

impl DynExternalScanner for CScannerWrapper {
    fn scan(
        &mut self,
        _lexer: &mut dyn crate::external_scanner::Lexer,
        valid_symbols: &[bool],
    ) -> Option<ScanResult> {
        // For C scanners, we need to adapt the Rust lexer to C API
        // Extract input and position from lexer - this needs better API
        let input = &[]; // TODO: Get from lexer
        let position = 0; // TODO: Get from lexer
        use crate::external_scanner_ffi::RustLexerAdapter;

        // Create a lexer adapter
        let mut adapter = RustLexerAdapter::new(input, position);
        let mut ts_lexer = adapter.as_ts_lexer();

        // Call the C scanner
        let scan_result = unsafe { self.scanner.scan(&mut ts_lexer, valid_symbols) };
        if scan_result {
            let symbol_index = ts_lexer.result_symbol as usize;
            if symbol_index < self.external_tokens.len() {
                Some(ScanResult {
                    symbol: self.external_tokens[symbol_index].0,
                    length: adapter.token_length(),
                })
            } else {
                None
            }
        } else {
            None
        }
    }

    fn serialize(&self, buffer: &mut Vec<u8>) {
        unsafe {
            self.scanner.serialize(buffer);
        }
    }

    fn deserialize(&mut self, buffer: &[u8]) {
        unsafe { self.scanner.deserialize(buffer) }
    }
}

/// Scanner factory function
pub type ScannerFactory = Box<dyn Fn() -> Box<dyn DynExternalScanner> + Send + Sync>;

/// Global scanner registry
static SCANNER_REGISTRY: Lazy<Arc<Mutex<ScannerRegistry>>> =
    Lazy::new(|| Arc::new(Mutex::new(ScannerRegistry::new())));

/// Registry for external scanners
pub struct ScannerRegistry {
    scanners: HashMap<String, ScannerFactory>,
}

impl Default for ScannerRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ScannerRegistry {
    /// Create a new scanner registry
    pub fn new() -> Self {
        ScannerRegistry {
            scanners: HashMap::new(),
        }
    }

    /// Register a Rust external scanner
    pub fn register_rust_scanner<S>(&mut self, language: &str)
    where
        S: ExternalScanner + Default + Send + Sync + 'static,
    {
        eprintln!("Registering scanner for language: {}", language);
        let factory: ScannerFactory = Box::new(|| {
            Box::new(RustScannerWrapper {
                scanner: S::default(),
            })
        });
        self.scanners.insert(language.to_string(), factory);
        eprintln!(
            "Scanner registered. Total scanners: {}",
            self.scanners.len()
        );
    }

    /// Register a C external scanner
    pub fn register_c_scanner(
        &mut self,
        language: &str,
        data: TSExternalScannerData,
        external_tokens: Vec<SymbolId>,
    ) {
        let _language_owned = language.to_string();
        let factory: ScannerFactory = Box::new(move || {
            let scanner = unsafe { CExternalScanner::new(&data) };
            if let Some(scanner) = scanner {
                Box::new(CScannerWrapper {
                    scanner,
                    external_tokens: external_tokens.clone(),
                })
            } else {
                panic!("Failed to create C external scanner")
            }
        });
        self.scanners.insert(language.to_string(), factory);
    }

    /// Get a scanner factory for a language
    pub fn get_factory(&self, language: &str) -> Option<&ScannerFactory> {
        self.scanners.get(language)
    }

    /// Create a scanner instance for a language
    pub fn create_scanner(&self, language: &str) -> Option<Box<dyn DynExternalScanner>> {
        eprintln!(
            "Looking for scanner for language: '{}'. Available: {:?}",
            language,
            self.scanners.keys().collect::<Vec<_>>()
        );
        self.scanners.get(language).map(|factory| factory())
    }
}

/// Get the global scanner registry  
pub fn get_global_registry() -> Arc<Mutex<ScannerRegistry>> {
    SCANNER_REGISTRY.clone()
}

/// Register a Rust scanner with the global registry
pub fn register_rust_scanner<S>(language: &str)
where
    S: ExternalScanner + Default + Send + Sync + 'static,
{
    let registry = get_global_registry();
    let mut registry = registry.lock().unwrap();
    registry.register_rust_scanner::<S>(language);
}

/// Register a C scanner with the global registry
pub fn register_c_scanner(
    language: &str,
    data: TSExternalScannerData,
    external_tokens: Vec<SymbolId>,
) {
    let registry = get_global_registry();
    let mut registry = registry.lock().unwrap();
    registry.register_c_scanner(language, data, external_tokens);
}

/// Builder for configuring external scanners
pub struct ExternalScannerBuilder {
    language: String,
    external_tokens: Vec<SymbolId>,
}

impl ExternalScannerBuilder {
    /// Create a new builder for a language
    pub fn new(language: impl Into<String>) -> Self {
        ExternalScannerBuilder {
            language: language.into(),
            external_tokens: Vec::new(),
        }
    }

    /// Set the external tokens for this scanner
    pub fn with_external_tokens(mut self, tokens: Vec<SymbolId>) -> Self {
        self.external_tokens = tokens;
        self
    }

    /// Register a Rust scanner
    pub fn register_rust<S>(self) -> Self
    where
        S: ExternalScanner + Default + Send + Sync + 'static,
    {
        register_rust_scanner::<S>(&self.language);
        self
    }

    /// Register a C scanner
    pub fn register_c(self, data: TSExternalScannerData) -> Self {
        register_c_scanner(&self.language, data, self.external_tokens.clone());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::external_scanner::StringScanner;

    #[test]
    fn test_scanner_registry() {
        let mut registry = ScannerRegistry::new();

        // Register a Rust scanner
        registry.register_rust_scanner::<StringScanner>("test_lang");

        // Create scanner instance
        let scanner = registry.create_scanner("test_lang");
        assert!(scanner.is_some());

        // Test non-existent language
        let scanner = registry.create_scanner("unknown_lang");
        assert!(scanner.is_none());
    }

    #[test]
    fn test_scanner_builder() {
        // Test the builder pattern
        let builder = ExternalScannerBuilder::new("python")
            .with_external_tokens(vec![SymbolId(100), SymbolId(101)]);

        // Verify builder fields are set correctly
        assert_eq!(builder.language, "python");
        assert_eq!(builder.external_tokens.len(), 2);

        // The actual registration would happen through the builder methods
        // but we can't test global state reliably in unit tests
    }
}
