# ADR-010: External Scanner Architecture

## Status

Accepted

## Context

Many programming languages have context-sensitive lexical constructs that cannot be handled by regular expressions alone:

1. **Python indentation**: Significant whitespace that affects parsing
2. **Ruby heredocs**: Multi-line strings with custom delimiters
3. **String interpolation**: Nested expressions within strings
4. **Block comments**: Nested comment structures
5. **Custom literals**: Language-specific literal formats

Tree-sitter addresses this through **external scanners** - custom C code that extends the lexer. For Adze's pure-Rust implementation, we needed a strategy that:

- Maintains Tree-sitter compatibility for existing grammars
- Supports native Rust scanners for pure-Rust GLR mode
- Handles FFI safely for C scanner integration
- Provides a clean API for scanner authors

### Alternatives Considered

1. **FFI-only**: Only support C scanners via FFI
   - Pros: Full Tree-sitter compatibility
   - Cons: Requires C toolchain, unsafe code, no WASM support

2. **Rust-only**: Only support native Rust scanners
   - Pros: Safe code, WASM compatible, good DX
   - Cons: Breaks Tree-sitter ecosystem compatibility

3. **Dual-mode scanner**: Support both FFI and native Rust
   - Pros: Best of both worlds
   - Cons: More complex implementation

## Decision

We implemented a **dual-mode external scanner architecture** that supports both FFI-based Tree-sitter compatibility and native Rust scanners for GLR mode.

### Scanner Trait Abstraction

```rust
/// Core external scanner trait
pub trait ExternalScanner {
    /// Scan for a token in the current context
    fn scan(&mut self, lexer: &mut Lexer) -> ScanResult;
    
    /// Serialize scanner state for incremental parsing
    fn serialize(&self) -> Vec<u8>;
    
    /// Deserialize scanner state
    fn deserialize(&mut self, data: &[u8]);
}

/// Result of scanning operation
pub enum ScanResult {
    /// Token found with symbol ID
    Found(SymbolId),
    /// No token found, continue with main lexer
    NotFound,
    /// Error during scanning
    Error(ScannerError),
}
```

### FFI Mode (Tree-sitter Compatibility)

For grammars with existing C scanners:

```rust
/// FFI wrapper for Tree-sitter external scanners
pub struct TSExtScanner {
    /// Function pointers to C scanner
    scanner: *mut c_void,
    vtable: TSScannerVTable,
}

impl ExternalScanner for TSExtScanner {
    fn scan(&mut self, lexer: &mut Lexer) -> ScanResult {
        // Call C scanner via FFI
        unsafe {
            let result = (self.vtable.scan)(self.scanner, lexer.as_ptr());
            ScanResult::from_c_result(result)
        }
    }
    
    fn serialize(&self) -> Vec<u8> {
        // Call C serialize function
        unsafe {
            let mut buf = vec![0u8; 1024];
            let len = (self.vtable.serialize)(self.scanner, buf.as_mut_ptr());
            buf.truncate(len);
            buf
        }
    }
}
```

### Native Rust Mode (GLR)

For pure-Rust grammars:

```rust
/// Native Rust scanner for Python indentation
pub struct PythonIndentScanner {
    indent_stack: Vec<usize>,
    pending_dedent: usize,
}

impl ExternalScanner for PythonIndentScanner {
    fn scan(&mut self, lexer: &mut Lexer) -> ScanResult {
        // Handle indentation at line start
        if lexer.at_line_start() {
            let current_indent = lexer.count_leading_spaces();
            
            match current_indent.cmp(&self.indent_stack.last()) {
                Ordering::Greater => {
                    self.indent_stack.push(current_indent);
                    return ScanResult::Found(INDENT_TOKEN);
                }
                Ordering::Less => {
                    self.pending_dedent = self.indent_stack.len() - 1;
                    return self.emit_dedent();
                }
                _ => {}
            }
        }
        ScanResult::NotFound
    }
}
```

### Scanner Registry

Global registry maps grammar names to scanner factories:

```rust
/// Global scanner registry
pub static SCANNER_REGISTRY: Lazy<ScannerRegistry> = Lazy::new(|| {
    let mut registry = ScannerRegistry::new();
    
    // Register built-in scanners
    registry.register("python", PythonIndentScanner::new);
    registry.register("ruby", RubyHeredocScanner::new);
    
    registry
});

/// Look up scanner by grammar name
pub fn get_scanner(grammar_name: &str) -> Option<Box<dyn ExternalScanner>> {
    SCANNER_REGISTRY.get(grammar_name)
}
```

### Integration Points

1. **Grammar definition**: `#[adze::grammar("python")]` sets grammar name
2. **Extract trait**: Generated code includes `GRAMMAR_NAME` constant
3. **GLR engine**: Looks up scanner by grammar name at parse time
4. **FFI bridge**: C scanners wrapped in safe Rust interface

## Consequences

### Positive

- **Tree-sitter compatibility**: Existing C scanners work via FFI
- **Pure-Rust option**: Native scanners for WASM and safe code
- **Clean API**: Scanner authors implement a simple trait
- **Incremental parsing**: Serialize/deserialize supports state preservation
- **Grammar isolation**: Each grammar can have its own scanner
- **Testability**: Native scanners are easy to unit test

### Negative

- **Complexity**: Two code paths (FFI and native) to maintain
- **FFI overhead**: C scanner calls have performance cost
- **Safety concerns**: FFI code requires unsafe blocks
- **State management**: Serialize/deserialize must be correct for incremental parsing
- **Tooling dependency**: FFI mode requires C compiler

### Neutral

- **Feature flag**: `external-scanners` feature gates FFI support
- **Build complexity**: Native-only builds are simpler
- **Documentation split**: Need docs for both scanner types

## Related

- Related ADRs: [ADR-001](001-pure-rust-glr-implementation.md), [ADR-006](006-tree-sitter-compatibility-layer.md)
- Reference: [docs/archive/specs/GLR_ENGINE_CONTRACT.md](../archive/specs/GLR_ENGINE_CONTRACT.md)
- Reference: [runtime/src/external_scanner.rs](../../runtime/src/external_scanner.rs)
- Reference: [grammars/python/src/scanner.rs](../../grammars/python/src/scanner.rs) - Python indentation scanner example
