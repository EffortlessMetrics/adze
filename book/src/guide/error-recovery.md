# Error Recovery

Error recovery in rust-sitter enables robust parsing of malformed or partially complete code. This guide covers the error recovery systems available in rust-sitter, from basic span validation to advanced parser recovery strategies.

## Overview

Rust-sitter provides multiple layers of error recovery:

1. **Span Error Recovery** (PR #55) - Safe span operations with comprehensive validation
2. **Parser Error Recovery** - Graceful handling of syntax errors during parsing
3. **Incremental Error Recovery** - Smart recovery during incremental parsing operations
4. **GLR Error Recovery** - Advanced error handling for ambiguous grammars

## Span Error Recovery

### The SpanError System

The `SpanError` system provides comprehensive error handling for span-based operations, eliminating panic-prone indexing that can crash parsers when working with malformed input.

```rust
use rust_sitter::{Spanned, SpanError};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SpanError {
    /// The span start index is greater than the span end index
    InvalidRange { start: usize, end: usize },
    /// The span extends beyond the bounds of the target string or buffer  
    OutOfBounds { span: (usize, usize), length: usize },
}
```

### Safe Span Operations

All span operations now provide safe alternatives that return `Result` instead of panicking:

```rust
// Validate spans before use
let span = Spanned::new("identifier", (10, 20));
match span.validate_for_str(source_code) {
    Ok(()) => {
        // Safe to proceed with span operations
        let text = span.try_slice_str(source_code)?;
        println!("Extracted: {}", text);
    },
    Err(SpanError::InvalidRange { start, end }) => {
        eprintln!("Invalid span: start {} > end {}", start, end);
        // Handle malformed span gracefully
    },
    Err(SpanError::OutOfBounds { span, length }) => {
        eprintln!("Span {:?} exceeds source length {}", span, length);
        // Handle truncated input gracefully  
    }
}
```

### Error Recovery Patterns

#### Pattern 1: Defensive Span Extraction
```rust
fn safe_extract_token(source: &str, span: &Spanned<()>) -> Option<String> {
    match span.try_slice_str(source) {
        Ok(text) => Some(text.to_string()),
        Err(SpanError::OutOfBounds { .. }) => {
            // Input was truncated, extract what we can
            if span.span.0 < source.len() {
                Some(source[span.span.0..].to_string())
            } else {
                None
            }
        },
        Err(SpanError::InvalidRange { .. }) => {
            // Malformed span, skip this token
            None
        }
    }
}
```

#### Pattern 2: Graceful Mutable Operations
```rust
fn safe_rename_identifier(
    source: &mut String, 
    span: &Spanned<()>, 
    new_name: &str
) -> Result<(), String> {
    match span.try_slice_str_mut(source) {
        Ok(identifier) => {
            // Safely replace the identifier
            let start = span.span.0;
            let end = span.span.1;
            source.replace_range(start..end, new_name);
            Ok(())
        },
        Err(e) => Err(format!("Cannot rename identifier: {}", e))
    }
}
```

#### Pattern 3: Error Recovery in Batch Operations
```rust
fn process_spans_with_recovery(
    source: &str, 
    spans: &[Spanned<String>]
) -> (Vec<String>, Vec<SpanError>) {
    let mut results = Vec::new();
    let mut errors = Vec::new();
    
    for span in spans {
        match span.try_slice_str(source) {
            Ok(text) => results.push(text.to_string()),
            Err(e) => {
                errors.push(e);
                // Continue processing other spans
            }
        }
    }
    
    (results, errors)
}
```

## Parser Error Recovery

### Basic Error Handling

Rust-sitter parsers return detailed error information when parsing fails:

```rust
use rust_sitter_runtime::{Parser, ParseError};

let mut parser = Parser::new();
parser.set_language(my_language())?;

match parser.parse_utf8("fn main( { // missing closing paren", None) {
    Ok(tree) => println!("Parsed successfully"),
    Err(ParseError::UnexpectedToken { expected, found, location }) => {
        eprintln!("Expected {:?}, found '{}' at {}:{}", 
            expected, found, location.line, location.column);
        // Implement recovery strategy
    },
    Err(ParseError::AmbiguousParse { alternatives }) => {
        eprintln!("Ambiguous parse with {} alternatives", alternatives.len());
        // Choose best alternative or request disambiguation
    },
    Err(e) => eprintln!("Parse error: {}", e),
}
```

### Error Recovery Strategies

#### Strategy 1: Partial Parse Recovery
```rust
fn parse_with_recovery(source: &str) -> Result<PartialParseResult, ParseError> {
    let mut parser = Parser::new();
    parser.set_language(my_language())?;
    
    // Try full parse first
    match parser.parse_utf8(source, None) {
        Ok(tree) => Ok(PartialParseResult::Complete(tree)),
        Err(ParseError::UnexpectedToken { location, .. }) => {
            // Try parsing up to the error location
            let truncated = &source[..location.byte_offset.min(source.len())];
            match parser.parse_utf8(truncated, None) {
                Ok(partial_tree) => Ok(PartialParseResult::Partial {
                    tree: partial_tree,
                    error_location: location,
                }),
                Err(e) => Err(e),
            }
        },
        Err(e) => Err(e),
    }
}

enum PartialParseResult {
    Complete(Tree),
    Partial { tree: Tree, error_location: Location },
}
```

#### Strategy 2: Error Node Insertion
```rust
fn parse_with_error_nodes(source: &str) -> Result<Tree, ParseError> {
    let mut parser = Parser::new();
    parser.set_language(my_language())?;
    
    // Configure parser to create error nodes for invalid syntax
    let config = ErrorRecoveryConfig::builder()
        .enable_error_nodes(true)
        .max_error_recovery_attempts(5)
        .build();
        
    parser.set_error_recovery_config(config);
    parser.parse_utf8(source, None)
}
```

## Incremental Error Recovery

When using incremental parsing, error recovery becomes more complex because edits can invalidate previously valid spans.

### Edit Validation

```rust
use rust_sitter_runtime::{Tree, InputEdit, EditError, Point};

fn apply_edit_safely(
    tree: &mut Tree, 
    edit: InputEdit
) -> Result<(), EditError> {
    // Validate edit bounds
    match tree.edit(&edit) {
        Ok(()) => {
            println!("Edit applied successfully");
            Ok(())
        },
        Err(EditError::InvalidRange { start, old_end }) => {
            eprintln!("Invalid edit range: {} -> {}", start, old_end);
            // Could try to fix the edit range
            let corrected_edit = InputEdit {
                start_byte: start,
                old_end_byte: start, // Make it a pure insertion
                new_end_byte: edit.new_end_byte,
                start_position: edit.start_position,
                old_end_position: edit.start_position,
                new_end_position: edit.new_end_position,
            };
            tree.edit(&corrected_edit)
        },
        Err(EditError::ArithmeticOverflow) => {
            eprintln!("Edit would cause position overflow");
            Err(EditError::ArithmeticOverflow)
        },
        Err(EditError::ArithmeticUnderflow) => {
            eprintln!("Edit would cause position underflow");  
            Err(EditError::ArithmeticUnderflow)
        }
    }
}
```

### Incremental Recovery Patterns

#### Pattern 1: Conservative Recovery
```rust
fn incremental_parse_with_fallback(
    parser: &mut Parser,
    source: &str,
    old_tree: Option<&Tree>,
    edit: Option<InputEdit>
) -> Result<Tree, ParseError> {
    if let (Some(tree), Some(edit)) = (old_tree, edit) {
        // Try incremental parsing
        let mut tree_copy = tree.clone();
        match tree_copy.edit(&edit) {
            Ok(()) => {
                // Incremental parse succeeded
                parser.parse_utf8(source, Some(&tree_copy))
            },
            Err(_) => {
                // Edit failed, fall back to full parse
                eprintln!("Edit failed, falling back to full parse");
                parser.parse_utf8(source, None)
            }
        }
    } else {
        // No incremental context, do full parse
        parser.parse_utf8(source, None)
    }
}
```

#### Pattern 2: Edit Repair
```rust
fn repair_edit(edit: InputEdit, source_len: usize) -> InputEdit {
    let InputEdit { mut start_byte, mut old_end_byte, mut new_end_byte, .. } = edit;
    
    // Ensure bounds are within source
    start_byte = start_byte.min(source_len);
    old_end_byte = old_end_byte.min(source_len).max(start_byte);
    
    // Ensure new_end is reasonable (could be larger for insertions)
    new_end_byte = new_end_byte.max(start_byte);
    
    InputEdit {
        start_byte,
        old_end_byte, 
        new_end_byte,
        ..edit
    }
}
```

## GLR Error Recovery

GLR parsers handle ambiguous grammars and can provide more sophisticated error recovery.

### Ambiguity Resolution

```rust
fn handle_glr_ambiguity(result: ParseResult) -> Tree {
    match result {
        ParseResult::Single(tree) => tree,
        ParseResult::Ambiguous(forest) => {
            // Choose the most likely parse based on heuristics
            let best_parse = forest.alternatives()
                .max_by_key(|alt| alt.confidence_score())
                .unwrap_or_else(|| forest.alternatives().next().unwrap());
            
            println!("Resolved ambiguity: chose parse with {} nodes", 
                best_parse.node_count());
            best_parse.to_tree()
        }
    }
}
```

### Error Forest Analysis

```rust
fn analyze_parse_errors(forest: &ParseForest) -> Vec<RecoveryHint> {
    let mut hints = Vec::new();
    
    for error_node in forest.error_nodes() {
        let hint = match error_node.error_type() {
            ErrorType::MissingToken(expected) => {
                RecoveryHint::InsertToken {
                    position: error_node.start_position(),
                    token: expected,
                }
            },
            ErrorType::UnexpectedToken(found) => {
                RecoveryHint::DeleteToken {
                    span: error_node.span(),
                    token: found,
                }
            },
            ErrorType::StructuralError => {
                RecoveryHint::Restructure {
                    span: error_node.span(),
                    suggestion: "Consider adding parentheses or braces",
                }
            }
        };
        hints.push(hint);
    }
    
    hints
}

enum RecoveryHint {
    InsertToken { position: Point, token: String },
    DeleteToken { span: (usize, usize), token: String },
    Restructure { span: (usize, usize), suggestion: &'static str },
}
```

## Testing Error Recovery

### Unit Testing Error Conditions

```rust
#[cfg(test)]
mod error_recovery_tests {
    use super::*;

    #[test]
    fn test_span_out_of_bounds_recovery() {
        let source = "hello";
        let span = Spanned::new((), (0, 10)); // Extends beyond source
        
        match span.try_slice_str(source) {
            Err(SpanError::OutOfBounds { span, length }) => {
                assert_eq!(span, (0, 10));
                assert_eq!(length, 5);
            },
            _ => panic!("Expected OutOfBounds error"),
        }
    }
    
    #[test]
    fn test_invalid_range_recovery() {
        let source = "hello world";
        let span = Spanned::new((), (5, 3)); // start > end
        
        match span.validate_for_str(source) {
            Err(SpanError::InvalidRange { start, end }) => {
                assert_eq!(start, 5);
                assert_eq!(end, 3);
            },
            _ => panic!("Expected InvalidRange error"),
        }
    }
    
    #[test]  
    fn test_edit_error_recovery() {
        let mut tree = create_test_tree();
        let invalid_edit = InputEdit {
            start_byte: 10,
            old_end_byte: 5, // Invalid: old_end < start
            new_end_byte: 15,
            start_position: Point { row: 0, column: 10 },
            old_end_position: Point { row: 0, column: 5 },
            new_end_position: Point { row: 0, column: 15 },
        };
        
        match tree.edit(&invalid_edit) {
            Err(EditError::InvalidRange { start, old_end }) => {
                assert_eq!(start, 10);
                assert_eq!(old_end, 5);
            },
            _ => panic!("Expected InvalidRange error"),
        }
    }
}
```

### Integration Testing

```rust
#[test]
fn test_malformed_input_recovery() {
    let malformed_inputs = vec![
        "fn main(",           // Missing closing paren
        "let x = ;",          // Missing expression  
        "if true { else }",   // Missing if body
        "",                   // Empty input
        "fn main() { let x = 42",  // Missing closing brace
    ];
    
    let mut parser = Parser::new();
    parser.set_language(rust_language())?;
    
    for input in malformed_inputs {
        match parser.parse_utf8(input, None) {
            Ok(tree) => {
                // Parser successfully recovered
                assert!(tree.root_node().has_error(), 
                    "Expected error nodes in tree for input: {}", input);
            },
            Err(e) => {
                // Parser failed gracefully with useful error
                println!("Parse error for '{}': {}", input, e);
                assert!(!e.to_string().contains("panic"));
            }
        }
    }
}
```

## Best Practices

### 1. Always Use Safe Span Operations

```rust
// ❌ Panic-prone 
let text = &source[span.0..span.1];

// ✅ Safe with error handling
let text = match span.try_slice_str(source) {
    Ok(text) => text,
    Err(e) => {
        eprintln!("Failed to extract span: {}", e);
        return Err("Invalid span".into());
    }
};
```

### 2. Validate Edits Before Applying

```rust
// ✅ Validate edit before use
fn apply_edit_safely(tree: &mut Tree, edit: InputEdit) -> Result<(), EditError> {
    // Check bounds manually if needed
    if edit.start_byte > edit.old_end_byte {
        return Err(EditError::InvalidRange { 
            start: edit.start_byte, 
            old_end: edit.old_end_byte 
        });
    }
    
    tree.edit(&edit)
}
```

### 3. Implement Progressive Recovery

```rust
fn parse_with_progressive_recovery(source: &str) -> ParseResult {
    // Try full parse first
    if let Ok(tree) = try_full_parse(source) {
        return ParseResult::Success(tree);
    }
    
    // Try partial parse
    if let Ok(partial) = try_partial_parse(source) {
        return ParseResult::Partial(partial);  
    }
    
    // Create minimal error tree
    ParseResult::Error(create_error_tree(source))
}
```

### 4. Provide Rich Error Information

```rust
#[derive(Debug)]
struct DetailedParseError {
    message: String,
    location: Point,
    suggestions: Vec<String>,
    recovery_hint: Option<RecoveryHint>,
}

impl DetailedParseError {
    fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.suggestions.push(suggestion.into());
        self
    }
    
    fn with_recovery_hint(mut self, hint: RecoveryHint) -> Self {
        self.recovery_hint = Some(hint);
        self
    }
}
```

## Performance Considerations

Error recovery adds some overhead, but rust-sitter's implementation is designed to be efficient:

- **Lazy Validation**: Spans are only validated when accessed
- **Zero-Cost Abstractions**: No overhead when not using error recovery features
- **Incremental Recovery**: Only affected regions are revalidated during incremental parsing
- **GLR Efficiency**: Ambiguity resolution uses efficient forest algorithms

Monitor performance using the built-in instrumentation:

```rust
// Enable performance logging
std::env::set_var("RUST_SITTER_LOG_PERFORMANCE", "true");

// Monitor error recovery overhead
let start = std::time::Instant::now();
let result = parse_with_recovery(large_malformed_input);
let duration = start.elapsed();

println!("Recovery parse took {:?}", duration);
```

## Conclusion

Rust-sitter's error recovery system provides multiple layers of protection against malformed input:

1. **SpanError system** prevents panics and provides detailed error information
2. **Safe span operations** allow graceful handling of invalid ranges
3. **Incremental error recovery** maintains consistency during edits
4. **GLR error recovery** handles ambiguous and malformed syntax elegantly

By following the patterns and best practices in this guide, you can build robust parsers that handle real-world malformed input gracefully while providing useful error messages for debugging and user feedback.