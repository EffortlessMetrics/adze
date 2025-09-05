# How to Use Incremental Parsing in rust-sitter

This guide demonstrates how to use rust-sitter's production-ready incremental parsing features (implemented in PR #62) to achieve significant performance improvements when handling text edits.

## Prerequisites

Add rust-sitter with incremental parsing support to your `Cargo.toml`:

```toml
[dependencies]
rust-sitter = { version = "0.6", features = ["incremental_glr"] }
```

**Feature Requirements**:
- **`incremental_glr`**: Enables production `reparse()` method (recommended)
- **`incremental`**: Legacy incremental support (older API)
- **`all-features`**: Includes all incremental capabilities

## Quick Start

### 1. Basic Incremental Parsing

```rust
use rust_sitter::parser_v4::{Parser, Tree};
use rust_sitter::pure_incremental::Edit;
use rust_sitter::pure_parser::Point;
use rust_sitter::glr_incremental::{get_reuse_count, reset_reuse_counter};

// Create your parser (requires grammar, table, and language name)
let mut parser = Parser::new(grammar, parse_table, "my_language".to_string());

// Initial parse
let tree1 = parser.parse("fn main() { println!(\"Hello\"); }")?;
println!("Initial parse - errors: {}", tree1.error_count);

// Create an edit operation: change "Hello" to "World"
let edit = Edit {
    start_byte: 20,      // Position of "Hello"  
    old_end_byte: 25,    // End of "Hello"
    new_end_byte: 25,    // Same length for "World"
    start_point: Point { row: 0, column: 20 },
    old_end_point: Point { row: 0, column: 25 },
    new_end_point: Point { row: 0, column: 25 },
};

// Reset counter to track reuse
reset_reuse_counter();

// Incremental reparse (automatic GLR routing)
let tree2 = parser.reparse("fn main() { println!(\"World\"); }", &tree1, &edit)?;
println!("Incremental parse - errors: {}", tree2.error_count);

// Check performance
let reused = get_reuse_count();
println!("Reused {} subtrees", reused);
```

### 2. Monitoring Performance

Track incremental parsing effectiveness:

```rust
use std::time::Instant;
use rust_sitter::glr_incremental::{get_reuse_count, reset_reuse_counter};

fn demonstrate_performance() -> Result<(), Box<dyn std::error::Error>> {
    let mut parser = Parser::new(grammar, table, "demo".to_string());
    
    // Parse a larger file
    let initial_content = "fn main() {\n    let x = 42;\n    println!(\"{}\", x);\n}";
    let tree = parser.parse(initial_content)?;
    
    // Small edit: change 42 to 43
    let edit = Edit {
        start_byte: 21,
        old_end_byte: 23, 
        new_end_byte: 23,
        start_point: Point { row: 1, column: 12 },
        old_end_point: Point { row: 1, column: 14 },
        new_end_point: Point { row: 1, column: 14 },
    };
    
    // Measure incremental parse time
    reset_reuse_counter();
    let start = Instant::now();
    
    let new_content = "fn main() {\n    let x = 43;\n    println!(\"{}\", x);\n}";
    let incremental_tree = parser.reparse(new_content, &tree, &edit)?;
    
    let incremental_time = start.elapsed();
    let reused = get_reuse_count();
    
    println!("Incremental parse: {:?}", incremental_time);
    println!("Subtrees reused: {}", reused);
    
    // Compare with full reparse
    let start = Instant::now();
    let full_tree = parser.parse(new_content)?;
    let full_time = start.elapsed();
    
    println!("Full parse: {:?}", full_time);
    
    if incremental_time < full_time {
        let speedup = full_time.as_nanos() as f64 / incremental_time.as_nanos() as f64;
        println!("Speedup: {:.1}x", speedup);
    }
    
    Ok(())
}
```

### 3. Working with Multiple Edits

Handle sequences of edits efficiently:

```rust
fn handle_multiple_edits() -> Result<(), Box<dyn std::error::Error>> {
    let mut parser = Parser::new(grammar, table, "multi_edit".to_string());
    
    // Initial content
    let mut content = "let a = 1;\nlet b = 2;\nlet c = 3;".to_string();
    let mut tree = parser.parse(&content)?;
    
    // Series of edits (e.g., user typing)
    let edits = vec![
        ("let a = 10;", Edit {
            start_byte: 8, old_end_byte: 9, new_end_byte: 10,
            start_point: Point { row: 0, column: 8 },
            old_end_point: Point { row: 0, column: 9 },
            new_end_point: Point { row: 0, column: 10 },
        }),
        ("let b = 20;", Edit {
            start_byte: 19, old_end_byte: 20, new_end_byte: 21,
            start_point: Point { row: 1, column: 8 },
            old_end_point: Point { row: 1, column: 9 },
            new_end_point: Point { row: 1, column: 10 },
        }),
    ];
    
    for (i, (new_content, edit)) in edits.iter().enumerate() {
        reset_reuse_counter();
        
        tree = parser.reparse(new_content, &tree, edit)?;
        
        let reused = get_reuse_count();
        println!("Edit {}: reused {} subtrees", i + 1, reused);
    }
    
    Ok(())
}
```

## Advanced Usage

### Feature Flag Detection

Handle cases where incremental parsing might be disabled:

```rust
fn parse_with_fallback(
    parser: &mut Parser,
    content: &str, 
    old_tree: Option<&Tree>,
    edit: Option<&Edit>
) -> Result<Tree, Box<dyn std::error::Error>> {
    #[cfg(feature = "incremental_glr")]
    {
        if let (Some(old), Some(edit)) = (old_tree, edit) {
            // Try incremental parsing
            match parser.reparse(content, old, edit) {
                Ok(tree) => {
                    println!("✅ Incremental parse succeeded");
                    return Ok(tree);
                }
                Err(e) => {
                    println!("⚠️  Incremental parse failed: {}, falling back", e);
                }
            }
        }
    }
    
    #[cfg(not(feature = "incremental_glr"))]
    println!("ℹ️  Incremental parsing not enabled, using full parse");
    
    // Fallback to full parse
    Ok(parser.parse(content)?)
}
```

### Performance Analysis

Analyze incremental parsing effectiveness:

```rust
use rust_sitter::glr_incremental::{get_reuse_count, reset_reuse_counter};

struct PerformanceAnalyzer {
    total_edits: usize,
    total_reuse: usize,
    successful_incremental: usize,
}

impl PerformanceAnalyzer {
    fn new() -> Self {
        Self {
            total_edits: 0,
            total_reuse: 0,
            successful_incremental: 0,
        }
    }
    
    fn analyze_edit(&mut self, parser: &mut Parser, content: &str, old_tree: &Tree, edit: &Edit) -> Result<Tree, Box<dyn std::error::Error>> {
        self.total_edits += 1;
        reset_reuse_counter();
        
        let tree = parser.reparse(content, old_tree, edit)?;
        
        let reused = get_reuse_count();
        self.total_reuse += reused;
        
        if reused > 0 {
            self.successful_incremental += 1;
        }
        
        Ok(tree)
    }
    
    fn report(&self) {
        println!("Performance Analysis:");
        println!("  Total edits: {}", self.total_edits);
        println!("  Successful incremental: {}", self.successful_incremental);
        println!("  Success rate: {:.1}%", 
                (self.successful_incremental as f64 / self.total_edits as f64) * 100.0);
        println!("  Average subtrees reused: {:.1}", 
                self.total_reuse as f64 / self.total_edits as f64);
    }
}
```

## Troubleshooting

### Common Issues

#### 1. Low Reuse Counts

**Problem**: `get_reuse_count()` returns 0 or very low numbers.

**Solutions**:
```rust
// Check if feature is enabled
#[cfg(not(feature = "incremental_glr"))]
compile_error!("incremental_glr feature is required");

// Check edit size - very large edits may trigger full reparse
fn is_edit_size_reasonable(edit: &Edit) -> bool {
    let edit_size = edit.new_end_byte.saturating_sub(edit.start_byte);
    edit_size < 1000 // Reasonable threshold
}

// Check for ambiguous grammar scenarios
fn analyze_grammar_complexity(tree: &Tree) -> bool {
    // Heuristic: very deep or wide trees may reduce reuse
    tree.root_node_count < 1000
}
```

#### 2. Performance Issues

**Problem**: Incremental parsing is slower than expected.

**Solutions**:
```rust
// Use release builds for performance testing
#[cfg(debug_assertions)]
compile_error!("Use --release for production performance");

// Monitor system resources
use std::time::Instant;

fn benchmark_parsing() {
    let start = Instant::now();
    // ... parsing code ...
    let duration = start.elapsed();
    
    if duration.as_millis() > 100 {
        println!("⚠️  Slow parse: {:?}", duration);
    } else {
        println!("✅ Fast parse: {:?}", duration);
    }
}
```

#### 3. Memory Usage

**Problem**: High memory usage during incremental parsing.

**Solutions**:
```rust
// Periodically reset parser to free memory
fn periodic_full_reparse(parser: &mut Parser, content: &str, edit_count: usize) -> Result<Tree, Box<dyn std::error::Error>> {
    if edit_count % 100 == 0 {
        println!("Performing full reparse to free memory");
        parser.parse(content)
    } else {
        // Use incremental parsing
        Err("Use incremental path".into())
    }
}
```

## Best Practices

### 1. Edit Size Considerations

- **Small Edits** (1-10 tokens): Excellent reuse, 10x+ speedup expected
- **Medium Edits** (10-100 tokens): Good reuse, 3-5x speedup typical  
- **Large Edits** (100+ tokens): May trigger full reparse for correctness

### 2. Grammar Complexity

- **Simple Grammars**: Higher reuse rates
- **Ambiguous Grammars**: Conservative reuse for correctness
- **Complex Nesting**: May reduce reuse effectiveness

### 3. Memory Management

- Reset reuse counters regularly: `reset_reuse_counter()`
- Consider periodic full reparses for long-running applications
- Monitor memory usage in production

### 4. Error Handling

```rust
fn robust_incremental_parse(
    parser: &mut Parser,
    content: &str,
    old_tree: &Tree, 
    edit: &Edit
) -> Result<Tree, Box<dyn std::error::Error>> {
    match parser.reparse(content, old_tree, edit) {
        Ok(tree) => {
            if tree.error_count == 0 {
                Ok(tree)
            } else {
                println!("⚠️  Parse has errors, considering full reparse");
                Ok(tree) // Or fallback to full parse
            }
        }
        Err(e) => {
            println!("❌ Incremental parse failed: {}", e);
            println!("🔄 Falling back to full parse");
            Ok(parser.parse(content)?)
        }
    }
}
```

## Integration Examples

### IDE Language Server

```rust
use rust_sitter::parser_v4::Parser;

struct LanguageServer {
    parser: Parser,
    current_tree: Option<Tree>,
}

impl LanguageServer {
    fn handle_text_change(&mut self, params: TextDocumentChangeEvent) -> Result<(), ServerError> {
        let edit = Edit {
            start_byte: params.range.start_byte,
            old_end_byte: params.range.end_byte,
            new_end_byte: params.range.start_byte + params.text.len(),
            // ... position calculations
        };
        
        reset_reuse_counter();
        
        let new_tree = if let Some(ref old_tree) = self.current_tree {
            self.parser.reparse(&params.full_text, old_tree, &edit)?
        } else {
            self.parser.parse(&params.full_text)?
        };
        
        let reused = get_reuse_count();
        self.log_performance(reused);
        
        self.current_tree = Some(new_tree);
        Ok(())
    }
    
    fn log_performance(&self, reused: usize) {
        if reused > 0 {
            println!("📈 Incremental parse: {} subtrees reused", reused);
        } else {
            println!("📊 Full parse performed");  
        }
    }
}
```

### Real-time Code Analysis

```rust
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

struct CodeAnalyzer {
    parser: Parser,
    edit_count: AtomicUsize,
    reuse_total: AtomicUsize,
}

impl CodeAnalyzer {
    fn analyze_incremental(&self, content: &str, old_tree: &Tree, edit: &Edit) -> AnalysisResult {
        reset_reuse_counter();
        
        let tree = self.parser.reparse(content, old_tree, edit).unwrap();
        let reused = get_reuse_count();
        
        // Update metrics
        self.edit_count.fetch_add(1, Ordering::Relaxed);
        self.reuse_total.fetch_add(reused, Ordering::Relaxed);
        
        // Perform analysis on tree
        AnalysisResult::new(tree, reused > 0)
    }
    
    fn get_metrics(&self) -> (usize, f64) {
        let edits = self.edit_count.load(Ordering::Relaxed);
        let total_reuse = self.reuse_total.load(Ordering::Relaxed);
        let avg_reuse = if edits > 0 { total_reuse as f64 / edits as f64 } else { 0.0 };
        (edits, avg_reuse)
    }
}
```

## Conclusion

rust-sitter's incremental parsing provides significant performance improvements for text editing scenarios. The Direct Forest Splicing algorithm achieves 16x speedup for typical edits while maintaining correctness and GLR compatibility.

**Key Takeaways**:
- ✅ Use `incremental_glr` feature for production
- ✅ Monitor reuse with global counters
- ✅ Handle fallback gracefully
- ✅ Consider edit size and grammar complexity
- ✅ Test performance in release builds

For more detailed technical information, see:
- [Incremental Parsing Technical Guide](../incremental-parsing.md)
- [GLR Incremental Design](../implementation/GLR_INCREMENTAL_DESIGN.md)
- [API Documentation](../../API_DOCUMENTATION.md)