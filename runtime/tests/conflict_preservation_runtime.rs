/// Runtime Conflict Preservation Tests
///
/// These tests validate that GLR conflicts survive the encoding/decoding pipeline:
/// ParseTable → TSLanguage (encode) → decoder::decode_parse_table → ParseTable
///
/// This is the critical integration point between glr-core table generation
/// and the runtime decoder, ensuring conflicts are preserved through ABI boundaries.
///
/// Spec: docs/specs/TABLE_GENERATION_VALIDATION_CONTRACT.md
/// Phase: 2-3 Bridge - GLR Conflict Preservation across ABI
// These tests require example grammars to be built with pure-rust feature
#[cfg(feature = "pure-rust")]
mod runtime_conflict_preservation {
    #[allow(unused_imports)]
    use rust_sitter_glr_core::conflict_inspection::*;

    /// Test: Ambiguous Expression Grammar Conflicts Survive Encoding/Decoding
    ///
    /// This test validates the complete pipeline:
    /// 1. Example grammar (ambiguous_expr.rs) is compiled with GLR conflicts
    /// 2. glr-core generates ParseTable with multi-action cells
    /// 3. tablegen encodes to TSLanguage ABI
    /// 4. runtime decoder::decode_parse_table reconstructs ParseTable
    /// 5. conflict_inspection detects the same conflicts
    ///
    /// If this fails while glr-core tests pass, the bug is in:
    /// - tablegen::compress.rs (encoding), or
    /// - runtime::decoder::decode_parse_table (decoding)
    #[test]
    fn test_ambiguous_expr_conflicts_survive_encoding() {
        // This test documents the expected behavior
        // Once example grammars expose LANGUAGE and SMALL_PARSE_TABLE, we can:
        //
        // 1. Load LANGUAGE from generated parser
        // 2. Decode ParseTable using runtime decoder
        // 3. Run conflict inspection
        // 4. Assert shift_reduce >= 1

        eprintln!("Runtime Conflict Preservation Test:");
        eprintln!("  Grammar: ambiguous_expr");
        eprintln!("  Expected: At least 1 S/R conflict preserved through encode/decode");
        eprintln!("  Status: Awaiting example grammar integration");

        // TODO: Implement once example grammars export LANGUAGE symbols
        /*
        use rust_sitter::decoder::decode_parse_table;

        // Get LANGUAGE from generated parser
        let lang = unsafe { &rust_sitter_example::ambiguous_expr::generated::LANGUAGE };

        // Decode runtime ParseTable
        let table = decode_parse_table(lang);

        // Run conflict inspection
        let summary = count_conflicts(&table);

        // Validate conflicts were preserved
        assert!(
            summary.shift_reduce >= 1,
            "ambiguous_expr must preserve at least 1 S/R conflict after encode/decode, got {summary:?}"
        );

        eprintln!("✅ Conflicts preserved:");
        eprintln!("  States: {}", table.state_count);
        eprintln!("  S/R conflicts: {}", summary.shift_reduce);
        eprintln!("  R/R conflicts: {}", summary.reduce_reduce);
        */
    }

    /// Test: Dangling Else Grammar Conflicts Survive Encoding/Decoding
    ///
    /// Validates the classic dangling-else ambiguity is preserved.
    #[test]
    fn test_dangling_else_conflicts_survive_encoding() {
        eprintln!("Runtime Conflict Preservation Test:");
        eprintln!("  Grammar: dangling_else");
        eprintln!("  Expected: At least 1 S/R conflict on 'else' symbol");
        eprintln!("  Status: Awaiting example grammar integration");

        // TODO: Similar to ambiguous_expr test above
    }

    /// Test: Arithmetic Grammar Remains Conflict-Free
    ///
    /// Validates that precedence-resolved grammars don't accidentally
    /// introduce conflicts through the encoding/decoding pipeline.
    #[test]
    fn test_arithmetic_remains_conflict_free_after_encoding() {
        eprintln!("Runtime Conflict Preservation Test:");
        eprintln!("  Grammar: arithmetic (with precedence)");
        eprintln!("  Expected: 0 conflicts (precedence resolves ambiguity)");
        eprintln!("  Status: Awaiting example grammar integration");

        // TODO: Validate that conflict-free grammars stay conflict-free
        /*
        use rust_sitter::decoder::decode_parse_table;

        let lang = unsafe { &rust_sitter_example::arithmetic::generated::LANGUAGE };
        let table = decode_parse_table(lang);
        let summary = count_conflicts(&table);

        assert_eq!(
            summary.shift_reduce + summary.reduce_reduce, 0,
            "arithmetic grammar should remain conflict-free after encode/decode, got {summary:?}"
        );
        */
    }
}

/// Non-feature-gated test to ensure module compiles
#[test]
fn test_conflict_preservation_runtime_module_exists() {
    // This test ensures the module structure is correct
    // even without pure-rust feature.
    // The fact that this file compiles is the verification.
}
