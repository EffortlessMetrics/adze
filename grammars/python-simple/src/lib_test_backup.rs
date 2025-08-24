// This is a backup of the test that was removed
// Temporary test to prove the symbol ID mapping fix would work
#[test]
#[cfg(feature = "pure-rust")]
#[ignore] // Temporarily ignore while fixing API
fn test_symbol_id_mapping() {
    use rust_sitter::pure_parser::Parser;

    // Parse "42" and check the parse tree directly
    eprintln!("\n=== Testing symbol ID mapping for '42' ===");
    let input = "42";
    let language = grammar::language();

    let mut parser = Parser::new(language);
    match parser.parse_bytes_with_tree(input.as_bytes()) {
        Ok((_, tree)) => {
            eprintln!("Successfully parsed '42'");
            eprintln!("Root symbol: {}", tree.root.symbol);

            // Navigate to the actual number node
            if let Some(expr) = tree.root.children.get(0) {
                eprintln!("Expression symbol: {}", expr.symbol);
                if let Some(primary) = expr.children.get(0) {
                    eprintln!("Primary symbol: {}", primary.symbol);

                    // The primary expression should have symbol ID that corresponds to NumberLiteral
                    // Based on debug output, we expect symbol 17 for NumberLiteral
                    assert_eq!(
                        primary.symbol, 17,
                        "Expected symbol 17 (NumberLiteral) but got {}",
                        primary.symbol
                    );
                    eprintln!("✓ Correct symbol ID for NumberLiteral");
                }
            }
        }
        Err(e) => panic!("Failed to parse '42': {:?}", e),
    }

    // Parse "a" and check the parse tree
    eprintln!("\n=== Testing symbol ID mapping for 'a' ===");
    let input = "a";

    let mut parser2 = Parser::new(language);
    match parser2.parse_bytes_with_tree(input.as_bytes()) {
        Ok((_, tree)) => {
            eprintln!("Successfully parsed 'a'");
            eprintln!("Root symbol: {}", tree.root.symbol);

            // Navigate to the actual identifier node
            if let Some(expr) = tree.root.children.get(0) {
                eprintln!("Expression symbol: {}", expr.symbol);
                if let Some(primary) = expr.children.get(0) {
                    eprintln!("Primary symbol: {}", primary.symbol);

                    // The primary expression should have symbol ID that corresponds to Identifier
                    // Based on debug output, we expect symbol 60 for Identifier
                    assert_eq!(
                        primary.symbol, 60,
                        "Expected symbol 60 (Identifier) but got {}",
                        primary.symbol
                    );
                    eprintln!("✓ Correct symbol ID for Identifier");
                }
            }
        }
        Err(e) => panic!("Failed to parse 'a': {:?}", e),
    }
}
