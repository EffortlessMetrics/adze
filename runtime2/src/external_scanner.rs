//! External scanner support for custom lexing logic

/// Result of scanning for an external token
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScanResult {
    /// The token type that was found (index into external_tokens array)
    pub token_type: u32,
    /// Number of bytes consumed
    pub bytes_consumed: usize,
}

/// Trait for external scanners (pure Rust version)
pub trait ExternalScanner: Send + Sync {
    /// Initialize the scanner
    fn init(&mut self);
    
    /// Scan for a token
    ///
    /// # Arguments
    /// * `valid_symbols` - Bitset of valid external tokens at this position
    /// * `input` - Input bytes available for scanning
    ///
    /// # Returns
    /// * `Some(ScanResult)` if a token was found
    /// * `None` if no external token matches
    fn scan(&mut self, valid_symbols: &[bool], input: &[u8]) -> Option<ScanResult>;
    
    /// Serialize scanner state for incremental parsing
    fn serialize(&self) -> Vec<u8>;
    
    /// Deserialize scanner state for incremental parsing
    fn deserialize(&mut self, data: &[u8]);
}

/// FFI-compatible external scanner interface
#[cfg(feature = "external-scanners")]
#[repr(C)]
pub struct TSExternalScanner {
    /// Private data pointer
    pub data: *mut std::os::raw::c_void,
    /// Function pointers for scanner operations
    pub vtable: TSExternalScannerVTable,
}

#[cfg(feature = "external-scanners")]
#[repr(C)]
pub struct TSExternalScannerVTable {
    /// Create a new scanner instance
    pub create: unsafe extern "C" fn() -> *mut std::os::raw::c_void,
    /// Destroy a scanner instance
    pub destroy: unsafe extern "C" fn(*mut std::os::raw::c_void),
    /// Scan for a token
    pub scan: unsafe extern "C" fn(
        *mut std::os::raw::c_void,
        *const u32,  // lexer
        *const bool, // valid_symbols
    ) -> bool,
    /// Serialize scanner state
    pub serialize: unsafe extern "C" fn(
        *const std::os::raw::c_void,
        *mut u8, // buffer
    ) -> u32,    // bytes written
    /// Deserialize scanner state
    pub deserialize: unsafe extern "C" fn(
        *mut std::os::raw::c_void,
        *const u8, // buffer
        u32,       // length
    ),
}

/// Example external scanner for indentation-based languages
#[cfg(test)]
pub struct IndentationScanner {
    indent_stack: Vec<u32>,
}

#[cfg(test)]
impl ExternalScanner for IndentationScanner {
    fn init(&mut self) {
        self.indent_stack.clear();
        self.indent_stack.push(0);
    }
    
    fn scan(&mut self, _valid_symbols: &[bool], input: &[u8]) -> Option<ScanResult> {
        // Simple example: count leading spaces
        let indent = input.iter().take_while(|&&b| b == b' ').count() as u32;
        
        if indent > *self.indent_stack.last()? {
            // INDENT token
            self.indent_stack.push(indent);
            Some(ScanResult {
                token_type: 0, // INDENT
                bytes_consumed: 0,
            })
        } else if indent < *self.indent_stack.last()? {
            // DEDENT token(s)
            while self.indent_stack.len() > 1 && indent < *self.indent_stack.last()? {
                self.indent_stack.pop();
            }
            Some(ScanResult {
                token_type: 1, // DEDENT
                bytes_consumed: 0,
            })
        } else {
            None
        }
    }
    
    fn serialize(&self) -> Vec<u8> {
        // Serialize indent stack
        let mut data = Vec::new();
        data.extend_from_slice(&(self.indent_stack.len() as u32).to_le_bytes());
        for &indent in &self.indent_stack {
            data.extend_from_slice(&indent.to_le_bytes());
        }
        data
    }
    
    fn deserialize(&mut self, data: &[u8]) {
        // Deserialize indent stack
        if data.len() >= 4 {
            let len = u32::from_le_bytes([data[0], data[1], data[2], data[3]]) as usize;
            self.indent_stack.clear();
            for i in 0..len {
                let offset = 4 + i * 4;
                if offset + 4 <= data.len() {
                    let indent = u32::from_le_bytes([
                        data[offset],
                        data[offset + 1],
                        data[offset + 2],
                        data[offset + 3],
                    ]);
                    self.indent_stack.push(indent);
                }
            }
        }
    }
}