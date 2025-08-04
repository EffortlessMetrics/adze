# Query Predicate Evaluation in rust-sitter

## Overview

rust-sitter now supports Tree-sitter's query predicate system, allowing you to filter pattern matches based on conditions. This is essential for many language-specific queries where syntax alone isn't sufficient.

## Supported Predicates

### 1. `#eq?` - Equality Predicate

Tests if a captured node's text equals a value or another capture.

```scm
; Match only 'if' keywords
(keyword) @kw (#eq? @kw "if")

; Match identifiers that are equal
(identifier) @first . (identifier) @second (#eq? @first @second)
```

### 2. `#not-eq?` - Inequality Predicate

Tests if a captured node's text is NOT equal to a value or another capture.

```scm
; Match keywords that are not 'if'
(keyword) @kw (#not-eq? @kw "if")
```

### 3. `#match?` - Regular Expression Predicate

Tests if a captured node's text matches a regular expression.

```scm
; Match identifiers starting with underscore
(identifier) @private (#match? @private "^_")

; Match camelCase identifiers
(identifier) @camel (#match? @camel "^[a-z][a-zA-Z0-9]*$")
```

### 4. `#not-match?` - Negative Regular Expression

Tests if a captured node's text does NOT match a regular expression.

```scm
; Match identifiers that don't start with underscore
(identifier) @public (#not-match? @public "^_")
```

### 5. `#any-of?` - Set Membership Predicate

Tests if a captured node's text is in a set of values.

```scm
; Match control flow keywords
(keyword) @control (#any-of? @control "if" "while" "for" "switch")

; Match visibility modifiers
(modifier) @vis (#any-of? @vis "public" "private" "protected")
```

## Implementation Details

### Architecture

The predicate evaluation system consists of:

1. **PredicateContext** (`query/predicate_eval.rs`): Handles predicate evaluation with source text
2. **Enhanced Matcher** (`query/matcher_v2.rs`): Integrates predicate checking into pattern matching
3. **Regex Caching**: Compiled regexes are cached for performance

### Usage Example

```rust
use rust_sitter::{
    parser::ParseNode,
    query::{Query, matcher_v2::QueryMatcher},
};

// Parse tree and source code
let tree = parse_code(source);
let source = "if (condition) { return true; }";

// Query with predicates
let query = compile_query(r#"
    (keyword) @kw
    (#eq? @kw "if")
"#);

// Match with predicate evaluation
let matcher = QueryMatcher::new(&query, source);
let matches = matcher.matches(&tree);

// Only 'if' keywords are matched, not 'return'
```

### Performance Considerations

1. **Text Extraction**: Node text is extracted on-demand using byte offsets
2. **Regex Caching**: Regular expressions are compiled once and cached
3. **Early Termination**: Predicates are evaluated after structural matching

## Integration with Tree-sitter

The predicate system is designed to be compatible with Tree-sitter's query language:

- All standard predicates are supported
- Custom predicates can be added via the `Custom` variant
- Property predicates (`#set!`, `#is?`) are parsed but need external handling

## Future Work

1. **Query Parser Integration**: Full S-expression query parser
2. **Property Predicates**: Support for `#set!` and `#is?` predicates
3. **Custom Predicates**: API for registering custom predicate handlers
4. **Streaming Evaluation**: Evaluate predicates during matching for better performance

## Testing

The predicate system includes comprehensive tests:

- Unit tests for each predicate type (`predicate_eval.rs`)
- Integration tests with mock parse trees (`test_query_predicates.rs`)
- Example demonstrating all predicates (`examples/predicate_demo.rs`)

Run the example:
```bash
cargo run -p rust-sitter --example predicate_demo
```

## Compatibility Note

This implementation aims for full compatibility with Tree-sitter's predicate system. Any differences in behavior should be reported as bugs.