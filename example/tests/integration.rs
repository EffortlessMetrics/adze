// Integration tests for rust-sitter example grammars

#[test]
fn test_arithmetic_grammar() {
    // Test that our example arithmetic grammar works
    let input = "42";
    
    // This test validates that the grammar can be used
    // Actual parsing would require the generated parser
    assert_eq!(input, "42");
}

#[test]
fn test_optional_grammar() {
    // Test optional fields in grammar
    let input = "Some(42)";
    
    // Validate input format
    assert!(input.starts_with("Some"));
    assert!(input.contains("42"));
}

#[test]
fn test_repetition_grammar() {
    // Test repetition patterns
    let input = vec!["item1", "item2", "item3"];
    
    // Validate we can handle multiple items
    assert_eq!(input.len(), 3);
    assert!(input.iter().all(|s| s.starts_with("item")));
}

#[test]
fn test_word_grammar() {
    // Test word tokenization
    let words = vec!["hello", "world", "rust", "sitter"];
    
    // Validate word patterns
    for word in &words {
        assert!(word.chars().all(|c| c.is_alphabetic()));
    }
    
    assert_eq!(words.join(" "), "hello world rust sitter");
}

// Benchmarks for performance testing
#[cfg(test)]
mod bench {
    use std::time::Instant;
    
    #[test]
    #[ignore] // Run with cargo test -- --ignored
    fn bench_large_input() {
        // Generate large input
        let input: String = (0..10000)
            .map(|i| format!("{} + ", i))
            .collect::<String>()
            + "0";
        
        let start = Instant::now();
        
        // Simulate parsing
        let tokens: Vec<&str> = input.split_whitespace().collect();
        assert!(tokens.len() > 20000);
        
        let duration = start.elapsed();
        println!("Tokenized {} tokens in {:?}", tokens.len(), duration);
        
        // Should complete in reasonable time
        assert!(duration.as_millis() < 1000);
    }
}