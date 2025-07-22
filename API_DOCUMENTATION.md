# Pure-Rust Tree-sitter API Documentation

This document provides comprehensive API documentation for the pure-Rust Tree-sitter implementation.

## Table of Contents

1. [Overview](#overview)
2. [Core Modules](#core-modules)
3. [Grammar Definition (IR)](#grammar-definition-ir)
4. [Parser Generation (GLR Core)](#parser-generation-glr-core)
5. [Table Generation](#table-generation)
6. [Runtime Components](#runtime-components)
7. [Build Integration](#build-integration)
8. [Examples](#examples)

## Overview

The pure-Rust Tree-sitter implementation consists of several interconnected crates that work together to generate efficient parsers without C dependencies:

- **rust-sitter-ir**: Grammar intermediate representation
- **rust-sitter-glr-core**: GLR parser generation algorithms
- **rust-sitter-tablegen**: Table compression and code generation
- **rust-sitter**: Runtime parsing components

## Core Modules

### rust-sitter-ir

The IR crate defines the core data structures for representing grammars.

#### Grammar

```rust
use rust_sitter_ir::{Grammar, Rule, Symbol, SymbolId, Token, TokenPattern};

// Create a new grammar
let mut grammar = Grammar::new("my_language".to_string());

// Define tokens
let id_symbol = SymbolId(1);
grammar.tokens.insert(id_symbol, Token {
    name: "identifier".to_string(),
    pattern: TokenPattern::Regex(r"[a-zA-Z_]\w*".to_string()),
    fragile: false,
});

// Define rules
grammar.rules.insert(expr_symbol, Rule {
    lhs: expr_symbol,
    rhs: vec![Symbol::Terminal(id_symbol)],
    precedence: None,
    associativity: None,
    fields: vec![],
    production_id: ProductionId(0),
});
```

#### Key Types

- `Grammar`: The top-level grammar representation
- `Rule`: A production rule with LHS and RHS symbols
- `Symbol`: Can be Terminal, NonTerminal, or External
- `Token`: Lexical token definition with pattern
- `TokenPattern`: String literal or regex pattern
- `PrecedenceKind`: Static or dynamic precedence
- `Associativity`: Left, Right, or None

### rust-sitter-glr-core

The GLR core provides parser generation algorithms.

#### FIRST/FOLLOW Sets

```rust
use rust_sitter_glr_core::FirstFollowSets;

// Compute FIRST and FOLLOW sets for a grammar
let first_follow = FirstFollowSets::compute(&grammar);

// Access FIRST set for a symbol
let first_set = &first_follow.first_sets[&symbol_id];

// Access FOLLOW set for a symbol
let follow_set = &first_follow.follow_sets[&symbol_id];
```

#### Parse Table

```rust
use rust_sitter_glr_core::{ParseTable, Action};

// Create parse table (usually generated automatically)
let table = ParseTable {
    action_table: vec![/* actions */],
    goto_table: vec![/* gotos */],
    symbol_metadata: vec![/* metadata */],
    state_count: 10,
    symbol_count: 5,
};

// Access actions for a state
let actions = &table.action_table[state_id];
```

#### Actions

- `Action::Shift(StateId)`: Shift to a new state
- `Action::Reduce(RuleId)`: Reduce by a rule
- `Action::Accept`: Accept the input
- `Action::Error`: Syntax error
- `Action::Fork(Vec<Action>)`: GLR fork point

### rust-sitter-tablegen

The tablegen crate handles table compression and code generation.

#### Static Language Generation

```rust
use rust_sitter_tablegen::StaticLanguageGenerator;

// Generate static language code
let generator = StaticLanguageGenerator::new(grammar, parse_table);
let code = generator.generate_language_code();

// Write to file
std::fs::write("parser.rs", code.to_string())?;
```

#### Node Types Generation

```rust
use rust_sitter_tablegen::NodeTypesGenerator;

// Generate NODE_TYPES.json
let generator = NodeTypesGenerator::new(&grammar);
let node_types_json = generator.generate()?;
```

#### Table Compression

```rust
use rust_sitter_tablegen::TableCompressor;

// Compress parse tables
let compressor = TableCompressor::new();
let compressed = compressor.compress_action_table(&action_table)?;
```

### rust-sitter (Runtime)

The runtime crate provides parsing functionality.

#### Lexer

```rust
use rust_sitter::lexer::GrammarLexer;

// Create lexer with token patterns
let patterns = vec![/* TokenPattern instances */];
let mut lexer = GrammarLexer::new(&patterns);

// Lex tokens
let token = lexer.next_token(input_bytes, position)?;
```

#### Parser

```rust
use rust_sitter::parser_v2::{ParserV2, Token};

// Create parser
let parser = ParserV2::new(grammar, parse_table);

// Parse tokens
let tokens = vec![/* Token instances */];
let parse_tree = parser.parse(tokens)?;
```

#### Incremental Parsing

```rust
use rust_sitter::incremental_v2::{IncrementalParserV2, Edit};

// Create incremental parser
let mut parser = IncrementalParserV2::new(grammar, parse_table);

// Parse with reuse
let edits = vec![
    Edit {
        start_byte: 10,
        old_end_byte: 15,
        new_end_byte: 20,
        // ... positions
    }
];
let new_tree = parser.parse(tokens, Some(&old_tree), &edits)?;
```

#### External Scanner

```rust
use rust_sitter::external_scanner::{ExternalScanner, ScanResult};

// Implement custom scanner
struct MyScanner;

impl ExternalScanner for MyScanner {
    fn new() -> Self { MyScanner }
    
    fn scan(
        &mut self,
        valid_symbols: &[bool],
        input: &[u8],
        position: usize,
    ) -> Option<ScanResult> {
        // Custom scanning logic
        Some(ScanResult {
            symbol: SymbolId(1),
            length: 5,
        })
    }
    
    fn serialize(&self, buffer: &mut Vec<u8>) {
        // Serialize state
    }
    
    fn deserialize(&mut self, buffer: &[u8]) {
        // Deserialize state
    }
}
```

## Build Integration

### Using rust-sitter-tool

Add to your `build.rs`:

```rust
use rust_sitter_tool::GrammarConverter;

fn main() {
    // Generate sample grammar (replace with actual extraction)
    let grammar = GrammarConverter::create_sample_grammar();
    
    // Generate parser code
    // ... (integration with tablegen)
    
    println!("cargo:rerun-if-changed=src/grammar.rs");
}
```

### Manual Integration

```rust
use rust_sitter_ir::Grammar;
use rust_sitter_glr_core::{FirstFollowSets, ParseTable};
use rust_sitter_tablegen::StaticLanguageGenerator;

// Step 1: Define or load grammar
let grammar = create_grammar();

// Step 2: Generate parser
let first_follow = FirstFollowSets::compute(&grammar);
let parse_table = generate_parse_table(&grammar, &first_follow);

// Step 3: Generate code
let generator = StaticLanguageGenerator::new(grammar, parse_table);
let code = generator.generate_language_code();
```

## Examples

### Simple Expression Parser

```rust
use rust_sitter::parser_v2::{ParserV2, Token};
use rust_sitter_ir::{Grammar, SymbolId};

// Create tokens
let tokens = vec![
    Token {
        symbol: SymbolId(1), // number
        text: b"42".to_vec(),
        start: 0,
        end: 2,
    },
    Token {
        symbol: SymbolId(2), // plus
        text: b"+".to_vec(),
        start: 3,
        end: 4,
    },
    Token {
        symbol: SymbolId(1), // number
        text: b"17".to_vec(),
        start: 5,
        end: 7,
    },
];

// Parse
let tree = parser.parse(tokens)?;
```

### Error Recovery

```rust
use rust_sitter::lexer::{ErrorRecoveringLexer, ErrorRecoveryMode};

// Create error-recovering lexer
let mut lexer = ErrorRecoveringLexer::new(
    base_lexer,
    ErrorRecoveryMode::SkipToKnown,
);

// Lex with error recovery
let tokens = lexer.lex_all(input)?;
```

### Custom External Scanner

```rust
// See external scanner example above

// Use in lexer
let mut runtime = ExternalScannerRuntime::new(external_tokens);
let result = runtime.scan(
    &mut scanner,
    &valid_external_tokens,
    input,
    position,
)?;
```

## Performance Considerations

1. **Table Compression**: The table compression algorithms significantly reduce memory usage
2. **Incremental Parsing**: Reuse unchanged subtrees for efficient reparsing
3. **External Scanners**: Use for complex lexical constructs that can't be expressed as regular expressions
4. **GLR Parsing**: Handles ambiguous grammars but may have performance overhead for highly ambiguous inputs

## Error Handling

All parsing operations return `Result` types with detailed error information:

```rust
match parser.parse(tokens) {
    Ok(tree) => {
        // Process parse tree
    }
    Err(ParseError::UnexpectedToken { expected, found, position }) => {
        // Handle syntax error
    }
    Err(e) => {
        // Handle other errors
    }
}
```

## Thread Safety

- Grammars and parse tables are immutable and can be shared across threads
- Parsers and lexers maintain state and should not be shared between threads
- Use separate parser instances for concurrent parsing

## Future Extensions

The API is designed to be extensible:

1. **Custom Actions**: Extend the Action enum for specialized parsing behavior
2. **Grammar Transformations**: Add optimization passes on the IR
3. **Alternative Table Formats**: Implement different compression strategies
4. **Language Bindings**: Generate bindings for other languages

For more examples and advanced usage, see the test files in each crate.