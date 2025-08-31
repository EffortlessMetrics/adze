# GLR Parsing

GLR (Generalized LR) parsing enables rust-sitter to handle ambiguous grammars and complex language constructs that traditional LR parsers cannot handle. This chapter explains when and how to use GLR parsing effectively.

## What is GLR Parsing?

GLR parsing extends LR parsing to handle **ambiguous grammars** by maintaining multiple parse stacks simultaneously. When the parser encounters a conflict (shift/reduce or reduce/reduce), instead of choosing one action, it **forks** to explore all valid paths.

### Key Capabilities

- **Ambiguity Support**: Parse inherently ambiguous language constructs
- **Complex Precedence**: Handle intricate operator precedence and associativity
- **Error Recovery**: Multiple parse paths improve error reporting and recovery
- **Research Applications**: Foundation for grammar inference and language analysis

## When to Use GLR Parsing

### Ideal Use Cases

1. **Ambiguous Language Constructs**
   ```rust
   // C-style function declarations vs expressions
   foo * bar;  // Declaration or multiplication?
   ```

2. **Complex Expression Grammars**
   ```rust
   // Mathematical expressions with multiple interpretations
   a + b * c - d / e
   ```

3. **Programming Languages with Context-Sensitive Features**
   ```rust
   // Rust-like syntax where context matters
   Vec<Vec<T>>  // Generic types with ambiguous parsing
   ```

4. **Research and Experimentation**
   - Grammar inference
   - Language design exploration
   - Parser development and testing

### When to Avoid GLR

- **Simple, unambiguous grammars** (standard LR is more efficient)
- **Performance-critical applications** (GLR has overhead)
- **Deterministic parsing requirements** (GLR can return multiple parse trees)

## Enabling GLR Parsing

### Feature Flag
```toml
[dependencies]
rust-sitter = { version = "0.6", features = ["pure-rust", "glr"] }
```

### Grammar Definition
GLR parsing works with the same grammar definition syntax:

```rust
#[rust_sitter::grammar("ambiguous")]
#[derive(Debug, Clone)]
pub enum AmbiguousGrammar {
    #[rust_sitter::rule("expression")]
    Expression(Expression),
}

#[derive(Debug, Clone)]
pub enum Expression {
    // Ambiguous: could be multiplication or declaration
    #[rust_sitter::rule("identifier '*' identifier")]
    MulOrDecl(String, String),
    
    #[rust_sitter::rule("identifier")]
    Identifier(String),
}
```

## Working with Grammar Decoding

One of GLR parsing's key features is the ability to **dynamically load** and **reconstruct** grammars from compiled Tree-sitter languages.

### Loading Existing Grammars

```rust
use rust_sitter::decoder;

// Load a Python grammar
let lang = rust_sitter_python::get_language();
let grammar = decoder::decode_grammar(lang);
let table = decoder::decode_parse_table(lang);

// The decoded grammar contains complete rule information
println!("Grammar has {} rules", grammar.rules.len());
println!("Grammar has {} fields", grammar.fields.len());
```

### Grammar Reconstruction Features

The decoder extracts comprehensive information:

- **Production Rules**: Complete RHS sequences with symbol resolution
- **Field Mappings**: All field names and their positions
- **Precedence Information**: Dynamic precedence from parse actions
- **Token Patterns**: Pattern extraction with intelligent fallbacks
- **External Scanner Integration**: Full external token support

### Example: Dynamic Python Parsing

```rust
use rust_sitter::{decoder, glr_parser::GLRParser};

// Decode Python grammar at runtime
let lang = rust_sitter_python::get_language();
let grammar = decoder::decode_grammar(lang);
let table = decoder::decode_parse_table(lang);

// Create GLR parser
let parser = GLRParser::new(grammar, table);

// Parse Python code with full ambiguity handling
let result = parser.parse(r#"
def hello():
    if True:
        pass
    else:
        return 42
"#)?;

// GLR parser handles all Python constructs correctly
assert!(result.is_success());
```

## GLR Parser API

### Creating a GLR Parser

```rust
use rust_sitter::glr_parser::GLRParser;

let parser = GLRParser::new(grammar, parse_table)
    .with_timeout(Duration::from_secs(30))
    .with_max_stack_size(10000)
    .with_ambiguity_limit(100);
```

### Parsing with Multiple Results

GLR parsing can return multiple parse trees for ambiguous input:

```rust
let results = parser.parse_all("ambiguous input")?;

match results {
    ParseResult::Unique(tree) => {
        println!("Unambiguous parse: {:?}", tree);
    }
    ParseResult::Ambiguous(trees) => {
        println!("Found {} possible interpretations", trees.len());
        for (i, tree) in trees.iter().enumerate() {
            println!("Interpretation {}: {:?}", i + 1, tree);
        }
    }
}
```

### Error Recovery

GLR parsers provide enhanced error recovery:

```rust
let result = parser.parse_with_recovery("malformed input")?;

match result {
    RecoveryResult::Success(tree) => println!("Parsed successfully"),
    RecoveryResult::Partial { tree, errors } => {
        println!("Partial parse with {} errors", errors.len());
        for error in errors {
            println!("Error at {}: {}", error.location, error.message);
        }
    }
}
```

## Performance Considerations

### GLR Parser Overhead

GLR parsing has performance implications:

1. **Memory Usage**: Multiple stacks require more memory
2. **Time Complexity**: Can be exponential for highly ambiguous grammars
3. **Fork Management**: Stack forking and merging adds overhead

### Optimization Strategies

1. **Precedence Rules**: Use precedence to reduce ambiguity where possible
2. **Grammar Design**: Design grammars to minimize conflicts
3. **Resource Limits**: Set appropriate timeouts and stack limits
4. **Profiling**: Use rust-sitter's profiling tools to identify bottlenecks

### Monitoring Performance

```rust
let parser = GLRParser::new(grammar, table)
    .with_profiling(true);

let result = parser.parse(input)?;
let stats = parser.get_statistics();

println!("Parse time: {:?}", stats.parse_time);
println!("Max stacks: {}", stats.max_concurrent_stacks);
println!("Forks created: {}", stats.fork_count);
```

## Testing GLR Parsers

### Stress Testing

rust-sitter includes comprehensive GLR stress tests:

```bash
# Run GLR stress tests
cargo test stress_deeply_nested_parentheses
cargo test test_extremely_ambiguous_parsing
cargo test test_long_ambiguous_chain
```

### Custom Stress Tests

```rust
#[test]
fn test_custom_ambiguous_grammar() {
    let parser = create_ambiguous_parser();
    
    // Test deeply nested structures
    let nested = "((((((a))))))".repeat(100);
    let result = parser.parse(&nested);
    assert!(result.is_ok());
    
    // Test highly ambiguous expressions
    let ambiguous = "a + b * c - d / e + f * g - h / i";
    let results = parser.parse_all(&ambiguous);
    assert!(results.unwrap().len() > 1);
}
```

### Roundtrip Testing

Verify that decoded grammars work correctly:

```rust
#[test]
fn test_decoded_grammar_roundtrip() {
    let lang = get_test_language();
    let grammar = decoder::decode_grammar(lang);
    let table = decoder::decode_parse_table(lang);
    
    let parser = GLRParser::new(grammar, table);
    
    // Test that original test cases still work
    for test_case in test_cases() {
        let result = parser.parse(test_case);
        assert!(result.is_ok(), "Failed to parse: {}", test_case);
    }
}
```

## Advanced Features

### External Scanner Integration

GLR parsing fully supports external scanners:

```rust
// External scanners work seamlessly with GLR
let grammar_with_scanner = decoder::decode_grammar(python_lang);
assert!(!grammar_with_scanner.externals.is_empty());

let parser = GLRParser::new(grammar_with_scanner, table);
let result = parser.parse("def foo():\n    pass")?; // Handles indentation
```

### Incremental GLR Parsing

```rust
let mut parser = GLRParser::new(grammar, table)
    .with_incremental(true);

// Parse initial input
let tree = parser.parse("initial code")?;

// Update with changes
let updated_tree = parser.reparse(
    &tree,
    InputEdit {
        start_byte: 10,
        old_end_byte: 15,
        new_end_byte: 20,
        start_point: Point { row: 0, column: 10 },
        old_end_point: Point { row: 0, column: 15 },
        new_end_point: Point { row: 0, column: 20 },
    },
    "updated code"
)?;
```

### Visualization

GLR parsers can visualize their internal state:

```rust
use rust_sitter::visualization::GLRVisualizer;

let visualizer = GLRVisualizer::new();
let dot_graph = visualizer.visualize_parse_forest(&parse_result);
std::fs::write("parse_forest.dot", dot_graph)?;
```

## Troubleshooting

### Common Issues

1. **Exponential Parse Times**
   - Add precedence rules to reduce ambiguity
   - Set appropriate timeouts
   - Profile to identify problematic rules

2. **Memory Exhaustion**
   - Lower max_stack_size limits
   - Implement ambiguity limits
   - Redesign highly ambiguous rules

3. **Unexpected Parse Results**
   - Use parse_all() to see all interpretations
   - Check precedence and associativity rules
   - Verify grammar expectations with small examples

### Debugging Tips

```rust
// Enable detailed logging
let parser = GLRParser::new(grammar, table)
    .with_debug(true);

// Use development builds for better error messages
#[cfg(debug_assertions)]
let parser = parser.with_stack_trace(true);
```

## Migration from LR Parsing

### Incremental Migration

1. **Start with existing grammar**: GLR works with any LR grammar
2. **Enable GLR features gradually**: Add ambiguous constructs as needed
3. **Performance test**: Ensure GLR overhead is acceptable
4. **Update test suites**: Account for potential multiple parse results

### API Changes

The GLR API is largely compatible with the standard parser API:

```rust
// LR parser
let result: ParseResult = lr_parser.parse(input)?;

// GLR parser (compatible)
let result: ParseResult = glr_parser.parse(input)?;

// GLR-specific features
let all_results: Vec<ParseResult> = glr_parser.parse_all(input)?;
```

GLR parsing transforms rust-sitter into a powerful tool for handling complex, ambiguous grammars while maintaining compatibility with simpler use cases.