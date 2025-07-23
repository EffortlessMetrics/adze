// External scanner registry for rust-sitter
// This module provides a registry for external scanners that can be used by parsers

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use crate::external_scanner::{ExternalScanner, ScanResult};
use crate::external_scanner_ffi::{CExternalScanner, TSExternalScannerData};
use rust_sitter_ir::SymbolId;

/// Type-erased external scanner
pub trait DynExternalScanner: Send + Sync {
    /// Scan for external tokens
    fn scan(
        &mut self,
        valid_symbols: &[bool],
        input: &[u8],
        position: usize,
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
        valid_symbols: &[bool],
        input: &[u8],
        position: usize,
    ) -> Option<ScanResult> {
        self.scanner.scan(valid_symbols, input, position)
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
        valid_symbols: &[bool],
        input: &[u8],
        position: usize,
    ) -> Option<ScanResult> {
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
                    symbol: self.external_tokens[symbol_index],
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
        unsafe { self.scanner.serialize(buffer); }
    }
    
    fn deserialize(&mut self, buffer: &[u8]) {
        unsafe { self.scanner.deserialize(buffer) }
    }
}

/// Scanner factory function
pub type ScannerFactory = Box<dyn Fn() -> Box<dyn DynExternalScanner> + Send + Sync>;

/// Global scanner registry
static SCANNER_REGISTRY: Mutex<Option<ScannerRegistry>> = Mutex::new(None);

/// Registry for external scanners
pub struct ScannerRegistry {
    scanners: HashMap<String, ScannerFactory>,
}

impl ScannerRegistry {
    /// Create a new scanner registry
    pub fn new() -> Self {
        ScannerRegistry {
            scanners: HashMap::new(),
        }
    }
    
    /// Register a Rust external scanner
    pub fn register_rust_scanner<S: ExternalScanner + 'static>(
        &mut self,
        language: &str,
    ) where
        S: Send + Sync,
    {
        let factory: ScannerFactory = Box::new(|| {
            Box::new(RustScannerWrapper {
                scanner: S::new(),
            })
        });
        self.scanners.insert(language.to_string(), factory);
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
        self.scanners.get(language).map(|factory| factory())
    }
}

/// Get the global scanner registry
pub fn get_global_registry() -> Arc<Mutex<ScannerRegistry>> {
    let mut guard = SCANNER_REGISTRY.lock().unwrap();
    if guard.is_none() {
        *guard = Some(ScannerRegistry::new());
    }
    Arc::new(Mutex::new(guard.take().unwrap()))
}

/// Register a Rust scanner with the global registry
pub fn register_rust_scanner<S: ExternalScanner + 'static>(language: &str)
where
    S: Send + Sync,
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
    pub fn register_rust<S: ExternalScanner + 'static>(self) -> Self
    where
        S: Send + Sync,
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
        let _builder = ExternalScannerBuilder::new("python")
            .with_external_tokens(vec![SymbolId(100), SymbolId(101)])
            .register_rust::<StringScanner>();
        
        // Verify it was registered
        let registry = get_global_registry();
        let registry = registry.lock().unwrap();
        assert!(registry.get_factory("python").is_some());
    }
}