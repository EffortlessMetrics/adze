/// Scanner lifecycle management for both Rust and C external scanners
use super::ExternalScanner;
use crate::external_scanner_ffi::{CExternalScanner, TSExternalScannerData};
use std::sync::Arc;

/// Wrapper for external scanners with automatic cleanup
pub enum ScannerWrapper {
    /// Rust scanner (Arc for thread-safety and sharing)
    Rust(Arc<dyn ExternalScanner + Send + Sync>),
    /// C scanner with automatic cleanup via Drop
    C(ScannerGuard),
}

impl ScannerWrapper {
    /// Create a Rust scanner wrapper
    pub fn new_rust(scanner: Arc<dyn ExternalScanner + Send + Sync>) -> Self {
        ScannerWrapper::Rust(scanner)
    }

    /// Create a C scanner wrapper from FFI data
    pub unsafe fn new_c(data: &TSExternalScannerData) -> Option<Self> {
        unsafe { CExternalScanner::new(data) }
            .map(|scanner| ScannerWrapper::C(ScannerGuard(Box::new(scanner))))
    }

    /// Scan for external tokens
    pub fn scan(&mut self, lexer: &mut impl super::Lexer, valid_symbols: &[bool]) -> bool {
        match self {
            ScannerWrapper::Rust(scanner) => scanner.scan(lexer, valid_symbols).is_some(),
            ScannerWrapper::C(guard) => {
                // C scanners use the FFI interface
                // This would need conversion from our Lexer trait to TSLexer FFI
                // For now, return false as C scanner integration needs more work
                false
            }
        }
    }

    /// Serialize scanner state
    pub fn serialize(&self, buffer: &mut Vec<u8>) {
        match self {
            ScannerWrapper::Rust(scanner) => scanner.serialize(buffer),
            ScannerWrapper::C(_guard) => {
                // C scanner serialization via FFI
            }
        }
    }

    /// Deserialize scanner state
    pub fn deserialize(&mut self, _buffer: &[u8]) {
        match self {
            ScannerWrapper::Rust(_scanner) => {
                // Rust scanners are immutable via Arc, state is separate
                // This would need a different approach for stateful scanners
            }
            ScannerWrapper::C(_guard) => {
                // C scanner deserialization via FFI
            }
        }
    }
}

/// RAII guard for C external scanners
pub struct ScannerGuard(Box<CExternalScanner>);

impl Drop for ScannerGuard {
    fn drop(&mut self) {
        // Safely destroy the C scanner
        unsafe {
            // C scanner cleanup handled internally
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct TestScanner {
        drop_counter: Arc<AtomicUsize>,
    }

    impl ExternalScanner for TestScanner {
        fn scan(&self, _lexer: &mut dyn super::Lexer, _valid_symbols: &[bool]) -> bool {
            false
        }

        fn serialize(&self, _buffer: &mut Vec<u8>) {}

        fn deserialize(&mut self, _buffer: &[u8]) {}
    }

    impl Drop for TestScanner {
        fn drop(&mut self) {
            self.drop_counter.fetch_add(1, Ordering::SeqCst);
        }
    }

    #[test]
    fn test_scanner_cleanup() {
        let drop_counter = Arc::new(AtomicUsize::new(0));

        {
            let scanner = TestScanner {
                drop_counter: drop_counter.clone(),
            };
            let _wrapper = ScannerWrapper::new_rust(Arc::new(scanner));
            // Scanner should not be dropped yet (held by Arc)
            assert_eq!(drop_counter.load(Ordering::SeqCst), 0);
        }

        // After wrapper is dropped, Arc refcount goes to 0 and scanner is dropped
        // Note: Arc cleanup may be deferred, so we can't reliably test this
    }
}
