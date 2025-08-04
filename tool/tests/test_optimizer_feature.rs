// Test that the optimizer feature flag works correctly
#[cfg(test)]
mod tests {
    use anyhow::Result;
    use rust_sitter_ir::Grammar;
    use rust_sitter_ir::optimizer::GrammarOptimizer;

    #[test]
    #[cfg(feature = "optimize")]
    fn test_optimizer_feature_enabled() -> Result<()> {
        // When the optimize feature is enabled, the optimizer module should be available
        let mut grammar = Grammar::new("test".to_string());
        let mut optimizer = GrammarOptimizer::new();
        let stats = optimizer.optimize(&mut grammar);
        
        // The optimizer should run without error
        assert_eq!(stats.total(), 0); // Empty grammar, no optimizations
        
        Ok(())
    }

    #[test]
    fn test_basic_compilation() -> Result<()> {
        // This test ensures the crate compiles correctly regardless of features
        let grammar = Grammar::new("test".to_string());
        assert_eq!(grammar.name, "test");
        Ok(())
    }
}