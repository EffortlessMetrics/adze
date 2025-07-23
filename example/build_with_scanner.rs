// Example build.rs file showing how to build a grammar with external scanner

use anyhow::Result;

fn main() -> Result<()> {
    // Build the parser using rust-sitter
    rust_sitter_tool::build_parsers()?;
    
    // Build external scanner if present
    // This will look for scanner.c, scanner.cc, or scanner.rs in the src directory
    rust_sitter_tool::scanner_build::build_scanner("my_language")?;
    
    Ok(())
}

// In your grammar file, you would register the scanner like this:
/*
// src/lib.rs or src/grammar.rs

// If using a C scanner, include the generated bindings
#[cfg(feature = "external-scanner")]
include!(concat!(env!("OUT_DIR"), "/my_language_scanner_bindings.rs"));

// If using a Rust scanner, include the registration
#[cfg(feature = "external-scanner")]
include!(concat!(env!("OUT_DIR"), "/my_language_scanner_registration.rs"));

// In your grammar initialization
#[cfg(feature = "external-scanner")]
fn init_scanner() {
    // For C scanner
    register_scanner(vec![
        SymbolId(100), // INDENT
        SymbolId(101), // DEDENT
        SymbolId(102), // NEWLINE
    ]);
    
    // For Rust scanner - registration is automatic
    register_scanner();
}
*/