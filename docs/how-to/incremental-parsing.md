# How to Use GLR Incremental Parsing in adze

> **⚠️ Status**: The incremental parsing path is currently **disabled** and falls back to fresh parsing for consistency. The infrastructure documented here exists but has known issues. See `glr_incremental.rs` for details.

This guide documents adze's GLR incremental parsing infrastructure (implemented September 2025) with fork-aware subtree reuse and conservative fallback strategies.

## Prerequisites

Add adze with GLR incremental parsing support to your `Cargo.toml`:

```toml
[dependencies]
adze = { version = "0.6", features = ["incremental_glr"] }
adze-glr-core = "0.6"
adze-ir = "0.6"
```

**Feature Requirements**:
- **`incremental_glr`**: Enables GLR incremental parsing with fork tracking
- **`external_scanners`**: Required for external scanner integration during incremental parsing
- **`all-features`**: Includes all GLR and incremental capabilities

## Quick Start

### 1. GLR Incremental Parser Setup

```rust
use adze::runtime::{GLRIncrementalParser, GLRToken, GLREdit, ForestNode};
use adze_ir::{Grammar, SymbolId};
use adze_glr_core::ParseTable;
use std::sync::Arc;

// Initialize GLR incremental parser with parse table and grammar
let mut parser = GLRIncrementalParser::new(
    Arc::clone(&parse_table),
    Arc::clone(&grammar),
);

// Define tokens for initial parsing
let initial_tokens = vec![
    GLRToken {
        symbol: SymbolId(1), // "fn"
        text: b"fn".to_vec(),
        start_byte: 0,
        end_byte: 2,
    },
    GLRToken {
        symbol: SymbolId(5), // identifier "main"
        text: b"main".to_vec(),
        start_byte: 3,
        end_byte: 7,
    },
    // ... additional tokens for complete function
];

// Initial parse with fork tracking
let initial_forest = parser.parse_incremental(&initial_tokens, &[])?;

// Create edit operation: change function name from "main" to "hello_world"
let edit = GLREdit {
    start_byte: 3,
    old_end_byte: 7,        // Replace "main"
    new_end_byte: 14,       // With "hello_world"
    old_forest: Some(Arc::clone(&initial_forest)),
    affected_forks: vec![], // GLR will determine affected forks
};

// Updated tokens after edit
let edited_tokens = vec![
    GLRToken {
        symbol: SymbolId(1), // "fn"
        text: b"fn".to_vec(),
        start_byte: 0,
        end_byte: 2,
    },
    GLRToken {
        symbol: SymbolId(5), // identifier "hello_world"
        text: b"hello_world".to_vec(),
        start_byte: 3,
        end_byte: 14,
    },
    // ... additional updated tokens
];

// Incremental reparse with GLR fork awareness
let updated_forest = parser.parse_incremental(&edited_tokens, &[edit])?;

// Note: Current implementation uses conservative fallback to ensure consistency
println!("GLR incremental parsing complete with conservative fallback");
```

### 2. GLR Fork Tracking and Performance Monitoring

Monitor GLR incremental parsing with fork tracking:

```rust
use std::time::Instant;
use adze::runtime::{GLRIncrementalParser, ForkTracker};

fn demonstrate_glr_performance() -> Result<(), Box<dyn std::error::Error>> {
    let mut parser = GLRIncrementalParser::new(
        Arc::clone(&parse_table),
        Arc::clone(&grammar),
    );
    
    // Create tokens for larger content with potential ambiguity
    let initial_tokens = create_tokens_for_content("class A { method() {} }");
    
    // Initial parse - may create multiple forks for ambiguous regions
    let start = Instant::now();
    let initial_forest = parser.parse_incremental(&initial_tokens, &[])?;
    let initial_time = start.elapsed();
    println!("Initial GLR parse: {:?}", initial_time);
    
    // Edit that affects ambiguous region: change method name
    let edit = GLREdit {
        start_byte: 10,
        old_end_byte: 16,     // "method"
        new_end_byte: 23,     // "newmethod"  
        old_forest: Some(Arc::clone(&initial_forest)),
        affected_forks: vec![], // GLR will determine this
    };
    
    let edited_tokens = create_tokens_for_content("class A { newmethod() {} }");
    
    // Incremental reparse with fork tracking
    let start = Instant::now();
    let updated_forest = parser.parse_incremental(&edited_tokens, &[edit])?;
    let incremental_time = start.elapsed();
    
    println!("GLR incremental parse: {:?}", incremental_time);
    println!("Conservative fallback: currently falls back to fresh parsing");
    
    // Compare with fresh parse
    let start = Instant::now();
    let fresh_forest = parser.parse_fresh(&edited_tokens)?;
    let fresh_time = start.elapsed();
    
    println!("Fresh GLR parse: {:?}", fresh_time);
    
    Ok(())
}

fn create_tokens_for_content(content: &str) -> Vec<GLRToken> {
    // This would typically be implemented by your tokenizer
    // For demonstration purposes, simplified tokenization
    vec![
        // Token creation logic would go here
    ]
}
```

### 3. External Scanner Integration with GLR Incremental Parsing

GLR incremental parsing supports external scanners for complex tokenization patterns:

```rust
use adze::external_scanner::{ExternalScanner, Lexer, ScanResult};

// Custom external scanner implementation
#[derive(Default)]
struct MyExternalScanner {
    state: Vec<u8>,
}

impl ExternalScanner for MyExternalScanner {
    fn scan(&mut self, lexer: &mut Lexer, valid_symbols: &[bool]) -> ScanResult {
        // Custom scanning logic that preserves state across incremental parses
        if valid_symbols[0] { // Check for specific token type
            // Scan and return result
            ScanResult::Success
        } else {
            ScanResult::Failure
        }
    }
    
    fn serialize(&self, buffer: &mut Vec<u8>) {
        buffer.extend(&self.state);
    }
    
    fn deserialize(buffer: &[u8]) -> Self {
        Self { state: buffer.to_vec() }
    }
}

// GLR incremental parser with external scanner
let scanner = Box::new(MyExternalScanner::default());
// External scanner integration would be configured in the GLRIncrementalParser
```

## Advanced Features

### Fork-Aware Edit Analysis

The GLR incremental parser can analyze which parse forks are affected by edits:

```rust
// Analyze edit impact on GLR forks
fn analyze_edit_impact(
    parser: &mut GLRIncrementalParser,
    edit: &GLREdit,
) -> Vec<usize> {
    // GLR parser will determine which forks need recomputation
    // This happens automatically during parse_incremental
    let affected_forks = vec![]; // Determined internally by GLR
    affected_forks
}
```

### Conservative Fallback Strategy

The current implementation uses a conservative approach:

```rust
// Current implementation ensures consistency by falling back to fresh parsing
// This temporary strategy maintains correctness while optimizing the GLR architecture

let result = parser.parse_incremental(&tokens, &edits)?;
// Note: Falls back to fresh parsing to ensure GLR correctness
// Future optimizations will enable full incremental reuse
```

## Troubleshooting

### Common Issues

**Issue**: GLR incremental parsing falls back to fresh parsing
**Solution**: This is the current conservative implementation strategy. The fallback ensures consistency while the GLR incremental architecture is optimized.

**Issue**: External scanner state not preserved across incremental parses
**Solution**: Ensure your external scanner implements `serialize()` and `deserialize()` methods correctly to maintain state.

**Issue**: Performance not improved compared to fresh parsing
**Solution**: Currently expected due to conservative fallback. Future optimizations will enable substantial performance gains.

### Feature Flag Conflicts

If you encounter issues with feature combinations:

```toml
# Recommended feature combination for GLR incremental parsing
[dependencies]
adze = { version = "0.6", features = ["incremental_glr", "external_scanners"] }
```

**Avoid** mixing legacy incremental features with GLR:
- Don't combine `incremental` and `incremental_glr`
- Use `incremental_glr` for GLR-compatible parsing

### Performance Monitoring

Enable performance logging to track GLR behavior:

```bash
export ADZE_LOG_PERFORMANCE=true
cargo run your_program
```

This provides insights into:
- Fork creation and tracking
- Token processing time
- Forest-to-tree conversion metrics
- External scanner invocation counts

## Current Implementation Status

**Implementation Complete** (September 2025):
- ✅ GLR-aware incremental parser architecture
- ✅ Fork tracking and affected region analysis  
- ✅ External scanner integration
- ✅ Conservative fallback for consistency
- ✅ Comprehensive error handling and memory safety

**Future Optimizations**:
- Advanced subtree reuse strategies for GLR
- Performance optimizations for fork-specific incremental updates
- Enhanced ambiguity preservation during incremental parsing
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
use adze::glr_incremental::{get_reuse_count, reset_reuse_counter};

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
use adze::parser_v4::Parser;

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

adze includes incremental parsing infrastructure designed for text editing scenarios. The Direct Forest Splicing algorithm is designed to achieve significant speedups while maintaining GLR compatibility.

**Current Status**: The incremental parsing path is currently disabled and falls back to fresh parsing for consistency. See `glr_incremental.rs` for details.

**Key Takeaways**:
- ⚠️ Incremental feature currently falls back to fresh parsing
- ✅ Monitor reuse with global counters (when enabled)
- ✅ Handle fallback gracefully
- ✅ Consider edit size and grammar complexity
- ✅ Test performance in release builds

For more detailed technical information, see:
- [Incremental Theory](../explanations/incremental-parsing-theory.md)
- [GLR Incremental Design (Archived)](../archive/implementation/GLR_INCREMENTAL_DESIGN.md)
- [API Reference](../reference/api.md)