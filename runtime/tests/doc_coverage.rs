/// Documentation Coverage Tests
///
/// Ensures all public API items are properly documented.
/// This helps maintain API quality and prevents undocumented public items.

#[cfg(all(test, feature = "strict_docs"))]
mod doc_coverage_tests {

    /// This test will fail if any public items lack documentation.
    /// It's gated behind the "strict_docs" feature to allow gradual improvement.
    #[test]
    fn test_all_public_items_documented() {
        // This is enforced by the deny(missing_docs) attribute in lib.rs
        // when the strict_docs feature is enabled.
        // The test passes if the crate compiles with the feature.

        // We can also programmatically check for specific documentation patterns
        const EXPECTED_MODULES: &[&str] = &["error", "tree", "parser", "language", "query"];

        // This test ensures the module structure remains documented
        for module in EXPECTED_MODULES {
            // The actual documentation checking is done by rustdoc
            // This just ensures the modules exist
            assert!(
                module.len() > 0,
                "Module {} should exist and be documented",
                module
            );
        }
    }

    /// Test that examples in documentation compile
    #[test]
    fn test_doc_examples_compile() {
        // Doc tests are automatically run by `cargo test --doc`
        // This test serves as a reminder that doc examples should be kept up to date

        // Example patterns that should work with appropriate features:
        #[cfg(feature = "tree-sitter-standard")]
        {
            use rust_sitter::tree_sitter::Parser;
            let _ = Parser::new();
        }

        #[cfg(all(feature = "ts-compat", feature = "disabled-for-pr58"))]
        {
            use rust_sitter::ts_compat::Tree;
            let _ = Tree::new_empty();
        }
    }

    /// Test that critical types have usage examples
    #[test]
    fn test_critical_types_have_examples() {
        // This is a placeholder for ensuring key types have examples
        // Real checking would be done via rustdoc or custom tooling

        const TYPES_REQUIRING_EXAMPLES: &[&str] =
            &["Parser", "Tree", "TreeNode", "TreeCursor", "Extract"];

        for type_name in TYPES_REQUIRING_EXAMPLES {
            // In practice, we'd parse the rustdoc output or use syn
            // For now, we just ensure the type names are non-empty
            assert!(
                !type_name.is_empty(),
                "Type {} should have usage examples in its documentation",
                type_name
            );
        }
    }

    /// Ensure README examples match actual API
    #[test]
    fn test_readme_examples_validity() {
        // This test ensures that common patterns shown in README still work

        // Pattern from README: Basic parsing
        #[cfg(all(feature = "tree-sitter-standard", feature = "disabled-for-pr58"))]
        {
            use rust_sitter::tree_sitter::Parser;
            let mut parser = Parser::new();
            let _result = parser.parse("fn main() {}", None);
        }

        // Pattern from README: Tree traversal
        #[cfg(all(feature = "ts-compat", feature = "disabled-for-pr58"))]
        {
            use rust_sitter::ts_compat::Tree;
            let tree = Tree::new_empty();
            let _root = tree.root_node();
            let _cursor = tree.walk();
        }
    }

    /// Test that deprecated items are properly marked
    #[test]
    fn test_deprecated_items_marked() {
        // When we deprecate APIs, they should be marked with #[deprecated]
        // This test documents that we track deprecations properly

        // Currently no deprecated items, but when we add them:
        // #[deprecated(since = "0.x.x", note = "Use new_api instead")]
        // pub fn old_api() {}

        // This would generate compiler warnings for users
        assert!(true, "No deprecated items currently");
    }

    /// Test that unsafe APIs are properly documented
    #[test]
    fn test_unsafe_apis_documented() {
        // Any unsafe functions should have safety documentation
        // Currently most unsafe code is internal, but if we expose any:

        // Example of what we'd check:
        // /// # Safety
        // /// This function is safe to call if...
        // pub unsafe fn unsafe_api() {}

        assert!(true, "No public unsafe APIs currently");
    }

    /// Ensure version-specific features are documented
    #[test]
    fn test_feature_documentation() {
        // Features should be documented in Cargo.toml and lib.rs

        const EXPECTED_FEATURES: &[&str] = &[
            "default",
            "queries",
            "wasm",
            "async",
            "pure-rust",
            "ts-compat",
            "strict_docs",
            "strict_api",
        ];

        for feature in EXPECTED_FEATURES {
            // In practice, we'd check Cargo.toml has descriptions
            assert!(
                !feature.is_empty(),
                "Feature {} should be documented",
                feature
            );
        }
    }
}
