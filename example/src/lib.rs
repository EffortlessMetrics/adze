// Ensure only one backend is enabled
#[cfg(all(feature = "pure-rust", feature = "c-backend"))]
compile_error!("Enable exactly one backend: 'pure-rust' OR 'c-backend'.");

// Re-export modules that contain grammars
pub mod ambiguous;
pub mod arithmetic;
pub mod external_word_example;
pub mod optionals;
pub mod performance_test;
pub mod repetitions;
pub mod test_precedence;
pub mod test_whitespace;
pub mod words;

// Tree-sitter compatibility language helpers
#[cfg(all(feature = "ts-compat", feature = "pure-rust"))]
pub mod ts_langs {
    use rust_sitter::ts_compat::Language;
    use std::sync::Arc;
    use std::collections::BTreeMap;
    
    /// Get the arithmetic language for ts_compat API
    pub fn arithmetic() -> Arc<Language> {
        // For now, return a minimal valid language
        // The actual implementation will need access to the generated tables
        // which requires refactoring how the grammar is exposed
        
        // Create minimal valid Grammar
        let grammar = Default::default();
        
        // Create minimal valid ParseTable
        // We need to import the types from glr-core through rust-sitter's re-exports
        // For now, create a simple placeholder structure
        let table = rust_sitter::__private::create_empty_parse_table();
        
        Arc::new(Language::new("arithmetic", grammar, table))
    }
}
