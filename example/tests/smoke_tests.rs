//! Smoke tests to ensure both pure-rust and c-backend work correctly

// Pure-Rust backend tests
#[cfg(feature = "pure-rust")]
#[test]
fn test_basic_parsing() {
    // Basic smoke test - parse a simple arithmetic expression
    use adze_example::arithmetic::grammar;

    let input = "1 - 2 * 3";
    let result = grammar::parse(input);

    assert!(
        result.is_ok(),
        "Failed to parse basic arithmetic expression"
    );

    // Verify we got some result
    let expr = result.unwrap();
    // Just check it's not completely broken - actual parsing tests are elsewhere
    assert!(matches!(
        expr,
        grammar::Expression::Sub(_, _, _)
            | grammar::Expression::Mul(_, _, _)
            | grammar::Expression::Number(_)
    ));
}

// C backend tests
#[cfg(feature = "c-backend")]
#[test]
fn test_c_backend_language_function() {
    // Ensure the C backend's tree_sitter_* function is available and non-null
    unsafe {
        let lang_ptr = adze_example::arithmetic::tree_sitter_arithmetic();
        assert!(
            !lang_ptr.is_null(),
            "tree_sitter_arithmetic() returned null"
        );

        // Basic sanity check on the language struct
        let version = (*lang_ptr).version;
        assert!(version > 0, "Language version should be positive");
    }
}

#[cfg(feature = "pure-rust")]
#[test]
fn test_pure_rust_no_c_symbols() {
    // This test just needs to compile and run - it verifies we're not
    // accidentally pulling in C symbols in pure-rust mode
    use adze_example::arithmetic::grammar;

    let result = grammar::parse("42");
    assert!(result.is_ok());

    // Verify we got the expected number
    if let Ok(grammar::Expression::Number(n)) = result {
        assert_eq!(n, 42);
    } else {
        panic!("Expected Number(42)");
    }
}

#[cfg(feature = "pure-rust")]
#[test]
fn test_multiple_grammars_available() {
    // Verify that multiple grammars are available

    // Test arithmetic grammar
    {
        use adze_example::arithmetic::grammar;
        assert!(grammar::parse("1 - 1").is_ok());
    }

    // Test repetitions grammar
    {
        use adze_example::repetitions::grammar;
        // This grammar expects specific input format
        let result = grammar::parse("1,2,3");
        // Just verify it doesn't panic
        let _ = result;
    }

    // Test words grammar
    {
        use adze_example::words::grammar;
        let result = grammar::parse("hello world");
        // Just verify it doesn't panic
        let _ = result;
    }
}

#[cfg(feature = "pure-rust")]
#[test]
fn test_error_recovery() {
    // Basic test that parsing continues even with errors
    use adze_example::arithmetic::grammar;

    // Invalid input with missing operand
    let result = grammar::parse("1 -");
    // Should still parse what it can rather than completely failing
    // The exact behavior depends on error recovery implementation
    let _ = result; // Just ensure it doesn't panic

    // Invalid tokens
    let result = grammar::parse("1 @ 2");
    let _ = result; // Just ensure it doesn't panic
}

// C backend multi-grammar test
#[cfg(feature = "c-backend")]
#[test]
fn test_c_backend_multiple_grammars() {
    // Test that multiple grammar language functions are available
    unsafe {
        let arith_lang = adze_example::arithmetic::tree_sitter_arithmetic();
        assert!(!arith_lang.is_null());

        let words_lang = adze_example::words::tree_sitter_words();
        assert!(!words_lang.is_null());

        let reps_lang = adze_example::repetitions::tree_sitter_repetitions();
        assert!(!reps_lang.is_null());
    }
}
