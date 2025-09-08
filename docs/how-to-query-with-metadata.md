# How-To: Query Matching with Node Metadata

This guide shows you how to use rust-sitter's enhanced query engine with symbol metadata validation for accurate pattern matching on parsed trees (PR #54).

## Problem: Inaccurate Query Matching

Without proper metadata validation, query patterns may:
- Match anonymous tokens when expecting named nodes
- Include unwanted "extra" nodes (comments, whitespace)
- Crash on malformed or missing symbol metadata
- Produce inconsistent results across different grammar types

## Solution: QueryMatcher with Symbol Metadata

The enhanced `QueryMatcher` API uses `SymbolMetadata` to provide accurate, memory-safe query matching.

### Step 1: Basic Setup with Metadata

```rust
use rust_sitter_runtime::{Parser, query::{QueryMatcher, compile_query}};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize parser with your language
    let language = my_language::language();
    let metadata = language.symbol_metadata(); // Get metadata array
    let mut parser = Parser::new();
    parser.set_language(language)?;
    
    // Parse source code
    let source = r#"
    function calculateSum(a, b) {
        // Add two numbers
        return a + b;
    }
    "#;
    let tree = parser.parse_utf8(source, None)?;
    
    // Create query with proper metadata validation
    let query = compile_query(r#"
        (function_declaration
          name: (identifier) @func_name
          parameters: (parameter_list) @params
          body: (block) @body)
    "#)?;
    
    // Create matcher with metadata - this is the key difference
    let matcher = QueryMatcher::new(&query, source, &metadata);
    let matches = matcher.matches(&tree.root());
    
    for m in matches {
        println!("Found {} captures", m.captures.len());
        for capture in m.captures {
            println!("  Capture {}: {:?}", capture.index, capture.node);
        }
    }
    
    Ok(())
}
```

### Step 2: Understanding Named vs Anonymous Nodes

The metadata-aware matcher automatically handles node type filtering:

```rust
// Example metadata-aware query patterns
let query = compile_query(r#"
  (function_declaration           ; Named node - only matches actual function declarations
    "function"                    ; Anonymous token - matches literal "function" keyword
    name: (identifier) @name      ; Named node - only matches identifier nodes
    "(" @lparen                   ; Anonymous token - matches literal "(" character
    parameters: (parameter_list)  ; Named node - only matches parameter list structures
    ")" @rparen                   ; Anonymous token - matches literal ")" character
    body: (block) @body)          ; Named node - only matches block structures
"#)?;

let matcher = QueryMatcher::new(&query, source, &metadata);

// The engine automatically uses metadata to:
// 1. Check metadata.named to filter named vs anonymous nodes
// 2. Skip nodes where metadata.is_extra == true (comments/whitespace)
// 3. Use null-safe access to prevent crashes on missing metadata
```

### Step 3: Memory-Safe Metadata Access

The internal implementation uses safe patterns that you can apply in your code:

```rust
// Safe metadata access pattern (internal to QueryMatcher)
fn node_is_named(&self, node: &ParseNode) -> bool {
    self.symbol_metadata
        .get(node.symbol.0 as usize)  // Bounds-checked access
        .map(|m| m.named)             // Safe field access
        .unwrap_or(true)              // Conservative fallback
}

fn node_is_extra(&self, node: &ParseNode) -> bool {
    self.symbol_metadata
        .get(node.symbol.0 as usize)  // Bounds-checked access
        .map(|m| m.is_extra)          // Safe field access
        .unwrap_or(false)             // Safe fallback
}

// Apply this pattern in your own code:
fn check_node_properties(node: &ParseNode, metadata: &[SymbolMetadata]) -> (bool, bool) {
    let symbol_meta = metadata
        .get(node.symbol.0 as usize)
        .unwrap_or(&SymbolMetadata::default());
        
    (symbol_meta.named, symbol_meta.is_extra)
}
```

### Step 4: Advanced Query Patterns

#### Pattern 1: Context-Sensitive Matching
```rust
// Match function calls only in specific contexts
let query = compile_query(r#"
  (block
    (expression_statement
      (call_expression
        function: (identifier) @func_name
        arguments: (argument_list) @args))
    
    ; Only match functions called inside if statements  
    (if_statement
      condition: (call_expression
        function: (identifier) @conditional_func)))
"#)?;

let matcher = QueryMatcher::new(&query, source, &metadata);
let matches = matcher.matches(&tree.root());

// Separate matches by capture type
for m in matches {
    for capture in m.captures {
        match capture.index {
            0 => println!("Function call: {:?}", capture.node),
            1 => println!("Arguments: {:?}", capture.node),
            2 => println!("Conditional function: {:?}", capture.node),
            _ => {}
        }
    }
}
```

#### Pattern 2: Multi-Language Pattern Matching
```rust
// Use different patterns based on language type
fn create_language_specific_query(language_name: &str) -> Result<Query, QueryError> {
    let query_text = match language_name {
        "javascript" | "typescript" => r#"
            (function_declaration
              name: (identifier) @func_name
              parameters: (parameter_list) @params)
              
            (arrow_function
              parameters: (parameter_list) @params)
        "#,
        "python" => r#"
            (function_definition
              name: (identifier) @func_name
              parameters: (parameters) @params)
        "#,
        "rust" => r#"
            (function_item
              name: (identifier) @func_name
              parameters: (parameters) @params)
        "#,
        _ => r#"
            (identifier) @any_identifier
        "#,
    };
    
    compile_query(query_text)
}

// Use with proper metadata
let language = detect_language(&source);
let metadata = language.symbol_metadata();
let query = create_language_specific_query(&language.name())?;
let matcher = QueryMatcher::new(&query, source, &metadata);
```

#### Pattern 3: Error-Tolerant Matching
```rust
// Handle parsing errors gracefully
let query = compile_query(r#"
  [
    (function_declaration) @func
    (ERROR) @error  ; Match error nodes
  ]
"#)?;

let matcher = QueryMatcher::new(&query, source, &metadata);
let matches = matcher.matches(&tree.root());

for m in matches {
    for capture in m.captures {
        match capture.index {
            0 => {
                println!("Found valid function");
                // Process successful parse
            }
            1 => {
                println!("Found parse error at: {:?}", capture.node);
                // Handle error gracefully
            }
            _ => {}
        }
    }
}
```

### Step 5: Iterator-Based Processing

Use the `QueryMatches` iterator for efficient large-file processing:

```rust
use rust_sitter_runtime::query::QueryMatches;

fn process_large_file(
    query: &Query, 
    root: &ParseNode, 
    source: &str, 
    metadata: &[SymbolMetadata]
) -> Result<Vec<String>, ProcessingError> {
    let mut results = Vec::new();
    
    // Use iterator for memory-efficient processing
    let matches = QueryMatches::new(query, root, source, metadata);
    
    for m in matches {
        for capture in m.captures {
            // Extract text for each capture
            let start = capture.node.start_byte;
            let end = capture.node.end_byte;
            let text = &source[start..end];
            
            results.push(format!("Capture {}: {}", capture.index, text));
        }
        
        // Limit results to prevent memory exhaustion
        if results.len() > 1000 {
            break;
        }
    }
    
    Ok(results)
}
```

### Step 6: Performance Optimization

#### Pattern 1: Metadata Caching
```rust
use std::sync::Arc;

struct CachedQueryMatcher<'a> {
    query: &'a Query,
    source: &'a str,
    metadata: Arc<[SymbolMetadata]>,  // Shared metadata
}

impl<'a> CachedQueryMatcher<'a> {
    fn new(
        query: &'a Query, 
        source: &'a str, 
        metadata: Arc<[SymbolMetadata]>
    ) -> Self {
        Self { query, source, metadata }
    }
    
    fn matches(&self, root: &ParseNode) -> Vec<QueryMatch> {
        let matcher = QueryMatcher::new(self.query, self.source, &self.metadata);
        matcher.matches(root)
    }
}
```

#### Pattern 2: Batch Query Processing
```rust
fn batch_process_queries(
    queries: &[Query],
    trees: &[ParseNode],
    sources: &[&str],
    metadata: &[SymbolMetadata],
) -> Vec<Vec<QueryMatch>> {
    queries
        .iter()
        .zip(trees.iter().zip(sources.iter()))
        .map(|(query, (tree, source))| {
            let matcher = QueryMatcher::new(query, source, metadata);
            matcher.matches(tree)
        })
        .collect()
}
```

## Migration from Old API

### Before (v0.5.x)
```rust
// Old API without metadata validation
let matcher = QueryMatcher::new(&query, source);
let matches = matcher.matches(&tree.root());
```

### After (v0.6.x)
```rust
// New API with metadata validation
let metadata = language.symbol_metadata();
let matcher = QueryMatcher::new(&query, source, &metadata);
let matches = matcher.matches(&tree.root());
```

### Migration Helper Function
```rust
fn migrate_query_usage(
    query: &Query,
    source: &str,
    tree: &ParseNode,
    language: &Language,
) -> Vec<QueryMatch> {
    // Get metadata from language
    let metadata = language.symbol_metadata();
    
    // Use new API
    let matcher = QueryMatcher::new(query, source, &metadata);
    matcher.matches(tree)
}
```

## Testing Your Queries

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_metadata_query_matching() {
        let language = test_language::language();
        let metadata = language.symbol_metadata();
        let mut parser = Parser::new();
        parser.set_language(language).unwrap();
        
        let source = "function test() { return 42; }";
        let tree = parser.parse_utf8(source, None).unwrap();
        
        let query = compile_query(r#"
            (function_declaration
              name: (identifier) @name)
        "#).unwrap();
        
        let matcher = QueryMatcher::new(&query, source, &metadata);
        let matches = matcher.matches(&tree.root());
        
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].captures.len(), 1);
        assert_eq!(matches[0].captures[0].index, 0);
    }
    
    #[test]  
    fn test_named_vs_anonymous_filtering() {
        // Test that named patterns only match named nodes
        // and anonymous patterns only match anonymous tokens
        let metadata = create_test_metadata();
        let query = compile_query(r#"
            (identifier) @named      ; Should only match named identifier nodes
            "(" @anonymous          ; Should only match anonymous "(" tokens
        "#).unwrap();
        
        let tree = create_test_tree();
        let matcher = QueryMatcher::new(&query, "test()", &metadata);
        let matches = matcher.matches(&tree);
        
        // Verify correct filtering based on metadata.named
        for m in matches {
            for capture in m.captures {
                match capture.index {
                    0 => assert!(is_named_node(&capture.node, &metadata)),
                    1 => assert!(!is_named_node(&capture.node, &metadata)),
                    _ => {}
                }
            }
        }
    }
}
```

## Troubleshooting Common Issues

### Issue 1: No Matches Found
```rust
// Check metadata availability
if metadata.is_empty() {
    eprintln!("Warning: No symbol metadata available");
    // Consider using legacy API or providing default metadata
}

// Check query syntax
match compile_query(query_text) {
    Ok(query) => { /* proceed */ }
    Err(e) => eprintln!("Query compilation failed: {}", e),
}
```

### Issue 2: Unexpected Match Behavior  
```rust
// Debug metadata properties
for (i, meta) in metadata.iter().enumerate() {
    println!("Symbol {}: name={}, named={}, is_extra={}", 
             i, meta.name, meta.named, meta.is_extra);
}

// Check node types in your tree
fn debug_tree_structure(node: &ParseNode, depth: usize) {
    let indent = "  ".repeat(depth);
    println!("{}Node: symbol={}, children={}", 
             indent, node.symbol.0, node.children.len());
    
    for child in &node.children {
        debug_tree_structure(child, depth + 1);
    }
}
```

### Issue 3: Performance Problems
```rust
// Monitor query performance
use std::time::Instant;

let start = Instant::now();
let matches = matcher.matches(&root);
let duration = start.elapsed();

if duration.as_millis() > 100 {
    println!("Slow query detected: {}ms", duration.as_millis());
    // Consider optimizing query or using caching
}
```

## Best Practices

1. **Always Use Metadata**: Prefer the new API with symbol metadata validation
2. **Cache Metadata**: Reuse metadata arrays when processing multiple queries
3. **Handle Missing Metadata**: Provide fallback behavior when metadata is unavailable
4. **Test Both Paths**: Test queries with both named and anonymous node patterns
5. **Monitor Performance**: Profile query performance on large files
6. **Use Safe Patterns**: Follow bounds-checking patterns for custom metadata access

This metadata-aware query system provides accurate, memory-safe pattern matching while maintaining high performance and Tree-sitter compatibility.