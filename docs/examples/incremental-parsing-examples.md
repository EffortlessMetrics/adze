# Incremental Parsing Examples

> **⚠️ Status**: The incremental parsing path is currently **disabled** and falls back to fresh parsing for consistency. The examples below document the intended API, but the implementation currently falls back to fresh parsing.

This document provides examples demonstrating the incremental parsing infrastructure in adze (PR #62).

## Example 1: Basic Single Edit

```rust
use adze::parser_v4::{Parser, Tree};
use adze::pure_incremental::Edit;
use adze::pure_parser::Point;
use adze::glr_incremental::{get_reuse_count, reset_reuse_counter};

fn basic_incremental_example() -> Result<(), Box<dyn std::error::Error>> {
    // Create parser with your grammar
    let mut parser = Parser::new(grammar, table, "example_lang".to_string());
    
    // Parse initial content
    let initial_content = "fn hello() { println!(\"Hello\"); }";
    let tree1 = parser.parse(initial_content)?;
    
    println!("Initial parse - errors: {}", tree1.error_count);
    assert_eq!(tree1.error_count, 0);
    
    // Create edit: change "Hello" to "World"
    let edit = Edit {
        start_byte: 23,    // Position of "Hello"
        old_end_byte: 28,  // End of "Hello"
        new_end_byte: 28,  // Same length for "World"
        start_point: Point { row: 0, column: 23 },
        old_end_point: Point { row: 0, column: 28 },
        new_end_point: Point { row: 0, column: 28 },
    };
    
    // Reset counter to track reuse
    reset_reuse_counter();
    
    // Perform incremental reparse
    let new_content = "fn hello() { println!(\"World\"); }";
    let tree2 = parser.reparse(new_content, &tree1, &edit)?;
    
    println!("Incremental parse - errors: {}", tree2.error_count);
    assert_eq!(tree2.error_count, 0);
    
    // Check performance
    let reused = get_reuse_count();
    println!("✅ Reused {} subtrees", reused);
    
    if reused > 0 {
        println!("🚀 Incremental parsing is working!");
    }
    
    Ok(())
}
```

## Example 2: Performance Measurement

```rust
use std::time::Instant;
use adze::glr_incremental::{get_reuse_count, reset_reuse_counter};

fn measure_performance() -> Result<(), Box<dyn std::error::Error>> {
    let mut parser = Parser::new(grammar, table, "perf_test".to_string());
    
    // Create a larger file to test performance
    let large_content = r#"
        fn main() {
            let x = 42;
            let y = x + 1;
            println!("Result: {}", y);
            
            for i in 0..10 {
                println!("Iteration: {}", i);
            }
        }
    "#;
    
    // Initial parse
    let start = Instant::now();
    let tree = parser.parse(large_content)?;
    let initial_time = start.elapsed();
    
    println!("Initial parse: {:?}", initial_time);
    
    // Small edit: change 42 to 43  
    let edit = Edit {
        start_byte: 35,
        old_end_byte: 37,
        new_end_byte: 37,
        start_point: Point { row: 2, column: 20 },
        old_end_point: Point { row: 2, column: 22 },
        new_end_point: Point { row: 2, column: 22 },
    };
    
    let new_content = large_content.replace("42", "43");
    
    // Measure incremental parse
    reset_reuse_counter();
    let start = Instant::now();
    let incremental_tree = parser.reparse(&new_content, &tree, &edit)?;
    let incremental_time = start.elapsed();
    
    let reused = get_reuse_count();
    println!("Incremental parse: {:?} (reused {} subtrees)", incremental_time, reused);
    
    // Compare with full reparse
    let start = Instant::now();
    let full_tree = parser.parse(&new_content)?;
    let full_time = start.elapsed();
    
    println!("Full parse: {:?}", full_time);
    
    // Calculate speedup
    if incremental_time < full_time && incremental_time.as_nanos() > 0 {
        let speedup = full_time.as_nanos() as f64 / incremental_time.as_nanos() as f64;
        println!("🚀 Speedup: {:.1}x faster", speedup);
        
        if speedup > 2.0 {
            println!("✅ Excellent performance improvement!");
        }
    }
    
    Ok(())
}
```

## Example 3: Multiple Sequential Edits

```rust
fn sequential_edits() -> Result<(), Box<dyn std::error::Error>> {
    let mut parser = Parser::new(grammar, table, "sequential".to_string());
    
    // Initial content
    let mut content = "let a = 1;\nlet b = 2;\nlet c = 3;".to_string();
    let mut tree = parser.parse(&content)?;
    
    println!("Initial content:\n{}", content);
    
    // Define a series of edits
    let edits = vec![
        // Edit 1: Change 1 to 10
        (
            "let a = 10;\nlet b = 2;\nlet c = 3;".to_string(),
            Edit {
                start_byte: 8,
                old_end_byte: 9,
                new_end_byte: 10,
                start_point: Point { row: 0, column: 8 },
                old_end_point: Point { row: 0, column: 9 },
                new_end_point: Point { row: 0, column: 10 },
            }
        ),
        // Edit 2: Change 2 to 20 
        (
            "let a = 10;\nlet b = 20;\nlet c = 3;".to_string(),
            Edit {
                start_byte: 20,
                old_end_byte: 21,
                new_end_byte: 22,
                start_point: Point { row: 1, column: 8 },
                old_end_point: Point { row: 1, column: 9 },
                new_end_point: Point { row: 1, column: 10 },
            }
        ),
        // Edit 3: Change 3 to 30
        (
            "let a = 10;\nlet b = 20;\nlet c = 30;".to_string(),
            Edit {
                start_byte: 33,
                old_end_byte: 34,
                new_end_byte: 35,
                start_point: Point { row: 2, column: 8 },
                old_end_point: Point { row: 2, column: 9 },
                new_end_point: Point { row: 2, column: 10 },
            }
        ),
    ];
    
    let mut total_reuse = 0;
    
    // Apply each edit sequentially
    for (i, (new_content, edit)) in edits.iter().enumerate() {
        println!("\n--- Edit {} ---", i + 1);
        
        reset_reuse_counter();
        let start = Instant::now();
        
        tree = parser.reparse(new_content, &tree, edit)?;
        
        let time = start.elapsed();
        let reused = get_reuse_count();
        total_reuse += reused;
        
        println!("Content: {}", new_content);
        println!("Time: {:?}, Reused: {} subtrees", time, reused);
        assert_eq!(tree.error_count, 0);
    }
    
    println!("\n📊 Summary:");
    println!("Total edits: {}", edits.len());
    println!("Total subtrees reused: {}", total_reuse);
    println!("Average reuse per edit: {:.1}", total_reuse as f64 / edits.len() as f64);
    
    Ok(())
}
```

## Example 4: Feature Flag Handling

```rust
fn feature_aware_parsing() -> Result<(), Box<dyn std::error::Error>> {
    let mut parser = Parser::new(grammar, table, "feature_test".to_string());
    
    let content = "fn test() { return 42; }";
    let tree = parser.parse(content)?;
    
    let edit = Edit {
        start_byte: 15,
        old_end_byte: 21, // "return"
        new_end_byte: 21,
        start_point: Point { row: 0, column: 15 },
        old_end_point: Point { row: 0, column: 21 },
        new_end_point: Point { row: 0, column: 21 },
    };
    
    let new_content = "fn test() { return 43; }";
    
    // Try incremental parsing with feature detection
    #[cfg(feature = "incremental_glr")]
    {
        println!("✅ Incremental GLR feature enabled");
        reset_reuse_counter();
        
        match parser.reparse(new_content, &tree, &edit) {
            Ok(new_tree) => {
                let reused = get_reuse_count();
                println!("🚀 Incremental parse succeeded: {} subtrees reused", reused);
                assert_eq!(new_tree.error_count, 0);
            }
            Err(e) => {
                println!("⚠️  Incremental parse failed: {}", e);
                println!("🔄 Falling back to full parse");
                let fallback_tree = parser.parse(new_content)?;
                assert_eq!(fallback_tree.error_count, 0);
            }
        }
    }
    
    #[cfg(not(feature = "incremental_glr"))]
    {
        println!("ℹ️  Incremental GLR feature not enabled");
        println!("📝 Note: Add 'incremental_glr' feature for better performance");
        
        // The reparse method still works, but falls back to full parse
        let result_tree = parser.reparse(new_content, &tree, &edit)?;
        assert_eq!(result_tree.error_count, 0);
        println!("✅ Full parse completed successfully");
    }
    
    Ok(())
}
```

## Example 5: Error Handling and Recovery

```rust
fn robust_error_handling() -> Result<(), Box<dyn std::error::Error>> {
    let mut parser = Parser::new(grammar, table, "error_test".to_string());
    
    let content = "fn valid() { }";
    let tree = parser.parse(content)?;
    
    // Create potentially problematic edit
    let problematic_edit = Edit {
        start_byte: 0,
        old_end_byte: 14,  // Replace entire function
        new_end_byte: 20,  // With something much larger
        start_point: Point { row: 0, column: 0 },
        old_end_point: Point { row: 0, column: 14 },
        new_end_point: Point { row: 0, column: 20 },
    };
    
    let new_content = "fn complex() { /* large change */ }";
    
    // Robust parsing with error handling
    reset_reuse_counter();
    
    match parser.reparse(new_content, &tree, &problematic_edit) {
        Ok(new_tree) => {
            let reused = get_reuse_count();
            println!("✅ Parse succeeded");
            println!("   Errors: {}", new_tree.error_count);
            println!("   Subtrees reused: {}", reused);
            
            if reused == 0 {
                println!("ℹ️  No reuse - likely triggered full reparse for correctness");
            }
        }
        Err(e) => {
            println!("❌ Parse failed: {}", e);
            println!("🔄 Attempting recovery with full parse");
            
            match parser.parse(new_content) {
                Ok(recovery_tree) => {
                    println!("✅ Recovery successful");
                    println!("   Errors: {}", recovery_tree.error_count);
                }
                Err(recovery_err) => {
                    println!("❌ Recovery also failed: {}", recovery_err);
                    println!("🔍 Check grammar and input validity");
                }
            }
        }
    }
    
    Ok(())
}
```

## Example 6: IDE Integration Pattern

```rust
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

struct DocumentManager {
    parser: Parser,
    documents: HashMap<String, Document>,
}

struct Document {
    content: String,
    tree: Tree,
    version: u32,
}

impl DocumentManager {
    fn new(grammar: Grammar, table: ParseTable) -> Self {
        Self {
            parser: Parser::new(grammar, table, "ide_lang".to_string()),
            documents: HashMap::new(),
        }
    }
    
    fn open_document(&mut self, uri: String, content: String) -> Result<(), Box<dyn std::error::Error>> {
        let tree = self.parser.parse(&content)?;
        
        let document = Document {
            content: content.clone(),
            tree,
            version: 1,
        };
        
        self.documents.insert(uri.clone(), document);
        println!("📂 Opened document: {}", uri);
        
        Ok(())
    }
    
    fn update_document(
        &mut self, 
        uri: String, 
        new_content: String,
        edit: Edit
    ) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(document) = self.documents.get_mut(&uri) {
            reset_reuse_counter();
            let start = Instant::now();
            
            // Try incremental parse
            match self.parser.reparse(&new_content, &document.tree, &edit) {
                Ok(new_tree) => {
                    let time = start.elapsed();
                    let reused = get_reuse_count();
                    
                    // Update document
                    document.content = new_content;
                    document.tree = new_tree;
                    document.version += 1;
                    
                    println!("📝 Updated {}: v{} ({:?}, {} reused)", 
                           uri, document.version, time, reused);
                    
                    if reused > 0 {
                        println!("⚡ Incremental parsing active");
                    }
                }
                Err(e) => {
                    println!("⚠️  Incremental update failed: {}", e);
                    
                    // Fallback to full parse
                    let new_tree = self.parser.parse(&new_content)?;
                    document.content = new_content;
                    document.tree = new_tree;
                    document.version += 1;
                    
                    println!("🔄 Full reparse completed for {}: v{}", uri, document.version);
                }
            }
        }
        
        Ok(())
    }
    
    fn get_diagnostics(&self, uri: &str) -> Vec<Diagnostic> {
        if let Some(document) = self.documents.get(uri) {
            // Analyze the parse tree for errors
            if document.tree.error_count > 0 {
                vec![Diagnostic {
                    message: format!("Parse errors found: {}", document.tree.error_count),
                    severity: DiagnosticSeverity::Error,
                }]
            } else {
                vec![]
            }
        } else {
            vec![]
        }
    }
}

#[derive(Debug)]
struct Diagnostic {
    message: String,
    severity: DiagnosticSeverity,
}

#[derive(Debug)]
enum DiagnosticSeverity {
    Error,
    Warning,
    Info,
}

// Usage example
fn ide_integration_example() -> Result<(), Box<dyn std::error::Error>> {
    let mut manager = DocumentManager::new(grammar, table);
    
    // Open a document
    let uri = "file:///example.rs".to_string();
    let initial_content = "fn main() {\n    println!(\"Hello\");\n}";
    manager.open_document(uri.clone(), initial_content.to_string())?;
    
    // Simulate user editing
    let edit = Edit {
        start_byte: 26,
        old_end_byte: 31, // "Hello"
        new_end_byte: 31,
        start_point: Point { row: 1, column: 14 },
        old_end_point: Point { row: 1, column: 19 },
        new_end_point: Point { row: 1, column: 19 },
    };
    
    let new_content = "fn main() {\n    println!(\"World\");\n}";
    manager.update_document(uri.clone(), new_content.to_string(), edit)?;
    
    // Check for errors
    let diagnostics = manager.get_diagnostics(&uri);
    println!("📋 Diagnostics: {:?}", diagnostics);
    
    Ok(())
}
```

## Example 7: Batch Operations and Analysis

```rust
fn batch_analysis() -> Result<(), Box<dyn std::error::Error>> {
    let mut parser = Parser::new(grammar, table, "batch_test".to_string());
    
    // Test data: various edit sizes
    let test_cases = vec![
        ("Small edit", "let x = 1;", "let x = 2;", 8, 9, 9),
        ("Medium edit", "fn test() { return 42; }", "fn test() { return 1234; }", 15, 21, 23),
        ("Large edit", "struct Point { x: i32, y: i32 }", "struct Point3D { x: i32, y: i32, z: i32 }", 0, 32, 43),
    ];
    
    println!("📊 Batch Performance Analysis");
    println!("{:<12} {:<10} {:<10} {:<10} {:<8}", "Edit Type", "Reuse", "Inc Time", "Full Time", "Speedup");
    println!("{:-<60}", "");
    
    for (name, old_content, new_content, start, old_end, new_end) in test_cases {
        // Parse original
        let old_tree = parser.parse(old_content)?;
        
        let edit = Edit {
            start_byte: start,
            old_end_byte: old_end,
            new_end_byte: new_end,
            start_point: Point { row: 0, column: start },
            old_end_point: Point { row: 0, column: old_end },
            new_end_point: Point { row: 0, column: new_end },
        };
        
        // Measure incremental parse
        reset_reuse_counter();
        let inc_start = Instant::now();
        let _inc_tree = parser.reparse(new_content, &old_tree, &edit)?;
        let inc_time = inc_start.elapsed();
        let reused = get_reuse_count();
        
        // Measure full parse
        let full_start = Instant::now();
        let _full_tree = parser.parse(new_content)?;
        let full_time = full_start.elapsed();
        
        // Calculate speedup
        let speedup = if inc_time.as_nanos() > 0 {
            full_time.as_nanos() as f64 / inc_time.as_nanos() as f64
        } else {
            0.0
        };
        
        println!("{:<12} {:<10} {:<10?} {:<10?} {:<8.1}x", 
               name, reused, inc_time, full_time, speedup);
    }
    
    Ok(())
}
```

## Running the Examples

To run these examples in your own project:

1. **Add the feature flag**:
   ```toml
   [dependencies]
   adze = { version = "0.6", features = ["incremental_glr"] }
   ```

2. **Create a test binary**:
   ```bash
   cargo new --bin incremental_example
   cd incremental_example
   ```

3. **Copy the examples** into `src/main.rs` and add your grammar setup.

4. **Run with timing**:
   ```bash
   cargo run --release  # Use release mode for accurate performance measurements
   ```

## Expected Results

When running these examples with the `incremental_glr` feature enabled, you should see:

- **Single edits**: 5-20x speedup with significant subtree reuse
- **Sequential edits**: Consistent reuse across multiple operations  
- **Large edits**: Automatic fallback to full parsing when needed
- **Error cases**: Graceful recovery with full parsing as backup

Without the feature flag, all operations fall back to full parsing but remain functionally correct.

## Troubleshooting

1. **Zero reuse**: Check that `incremental_glr` feature is enabled
2. **Slow performance**: Use `--release` builds for benchmarking
3. **Parse errors**: Verify grammar compatibility and edit ranges
4. **Memory issues**: Consider periodic full reparses for long-running applications

For more detailed guidance, see the [Incremental Parsing How-To Guide](../how-to/incremental-parsing-guide.md).