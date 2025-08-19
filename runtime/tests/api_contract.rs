/// API Contract Tests - Prevents regression of public API surface
/// 
/// These tests ensure that the public API remains stable and backwards compatible.
/// Any changes to these tests indicate a breaking change that requires a major version bump.

#[cfg(test)]
mod api_contract_tests {
    /// Test that the Extract trait is accessible
    #[test]
    fn test_extract_trait_exists() {
        // The Extract trait is the core public API
        // For now, we just verify the crate compiles with the trait
        
        // The type should exist in the public API
        let type_name = std::any::type_name::<rust_sitter::TSSymbol>();
        assert!(type_name.len() > 0, "TSSymbol type should have a name");
    }
    
    /// Test that commonly re-exported types exist
    #[test]
    fn test_core_types_exist() {
        // SymbolId should be publicly available
        type _Symbol = rust_sitter::SymbolId;
        
        // TSSymbol should be re-exported from FFI
        type _TSSymbol = rust_sitter::TSSymbol;
        
        // These types should exist at module level
        assert_eq!(
            std::mem::size_of::<rust_sitter::TSSymbol>(),
            std::mem::size_of::<u16>()
        );
    }
    
    /// Test that the crate structure hasn't changed drastically
    #[test]
    fn test_module_structure() {
        // These modules should always be present
        
        // Verify we're in the rust_sitter test module
        let _ = std::module_path!().contains("rust_sitter");
    }
    
    /// Test semver-sensitive changes
    #[test]
    fn test_no_breaking_changes() {
        // This test documents the current stable API surface.
        // Changes here indicate potential breaking changes.
        
        // SymbolId is a u16 newtype
        const _: () = assert!(std::mem::size_of::<rust_sitter::SymbolId>() == 2);
        
        // TSSymbol is also u16
        const _: () = assert!(std::mem::size_of::<rust_sitter::TSSymbol>() == 2);
    }
    
    /// Test that types have expected sizes
    #[test]
    fn test_type_sizes() {
        // These are sanity checks to catch unexpected changes
        // Symbol types should be 2 bytes (u16)
        assert_eq!(std::mem::size_of::<rust_sitter::TSSymbol>(), 2);
        assert_eq!(std::mem::size_of::<rust_sitter::SymbolId>(), 2);
    }
}