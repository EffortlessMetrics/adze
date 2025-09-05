# API Reference

Complete API reference for rust-sitter library components.

For the most comprehensive API documentation, see [API_DOCUMENTATION.md](../../API_DOCUMENTATION.md) in the repository root.

## Core Modules

### `rust_sitter`

Main runtime library providing parsing functionality.

```rust
use rust_sitter::*;
```

**Key Components:**
- `Extract` trait for AST conversion
- `Parser` API for GLR parsing  
- Tree and node manipulation
- Error recovery and incremental parsing

### `rust_sitter_tool`

Build-time code generation and grammar processing.

```rust
use rust_sitter_tool::build_parsers;

// In build.rs
fn main() {
    build_parsers(&PathBuf::from("src/grammar.rs"));
}
```

### `rust_sitter_macro`

Procedural macros for grammar definition.

```rust
#[rust_sitter::grammar("mylang")]
mod grammar {
    #[rust_sitter::language]
    pub struct Program { /* ... */ }
}
```

**Available Macros:**
- `#[rust_sitter::grammar]` - Grammar module definition
- `#[rust_sitter::language]` - Root language type
- `#[rust_sitter::leaf]` - Terminal symbol
- `#[rust_sitter::repeat]` - Repeated elements
- `#[rust_sitter::extra]` - Extra/whitespace tokens

## Runtime APIs

### GLR Parser (runtime2)

Production-ready GLR parser with Tree-sitter compatibility.

```rust
use rust_sitter_runtime2::{Parser, Language};

let mut parser = Parser::new();
parser.set_language(my_grammar::language())?;

let tree = parser.parse_utf8(input, None)?;
let ast = my_grammar::extract_ast(&tree)?;
```

### Tree-sitter Compatibility Layer

```rust
use rust_sitter::tree_sitter::{Parser, Language, Tree, Node};

let mut parser = Parser::new();
parser.set_language(&language)?;

let tree = parser.parse(input, None)?;
let root = tree.root_node();
```

## Serialization APIs

### TreeSerializer

Main serialization interface with multiple output formats.

```rust
use rust_sitter::serialization::TreeSerializer;

let serializer = TreeSerializer::new(source)
    .with_unnamed_nodes()
    .with_max_text_length(Some(100));

// JSON serialization  
let json = serializer.serialize_tree(&tree)?;

// Node-level serialization
let node_json = serializer.serialize_node(node);
```

### Output Formats

**Standard JSON:**
```rust
let json = TreeSerializer::new(source).serialize_tree(&tree)?;
```

**Compact JSON:**  
```rust
let compact = CompactSerializer::new(source).serialize_tree(&tree)?;
```

**S-Expressions:**
```rust
let sexp = SExpressionSerializer::new(source)
    .with_positions()
    .serialize_tree(&tree);
```

**Binary Format:**
```rust
let mut serializer = BinarySerializer::new();
let binary = serializer.serialize_tree(&tree);
```

## Dynamic Loading APIs

### libloading Integration

Safe dynamic library loading with FFI safety.

```rust
use libloading::Library;
use rust_sitter::tree_sitter::{Language, Parser};

// Load grammar library
let lib = Library::new("path/to/grammar.so")?;

// Get language function
let get_language: libloading::Symbol<unsafe extern "C" fn() -> Language> = 
    unsafe { lib.get(b"tree_sitter_json\0")? };

// Create parser
let language = unsafe { get_language() };
let mut parser = Parser::new();
parser.set_language(&language)?;
```

### CLI Integration

The CLI provides a safe wrapper around dynamic loading:

```rust
// Internal CLI function - reference implementation
fn parse_file_dynamic(
    grammar: &Path,
    input: &Path, 
    format: OutputFormat,
    symbol: &str,
) -> Result<()> {
    // Input validation
    validate_library_path(grammar)?;
    validate_input_file(input)?;
    validate_symbol_name(symbol)?;
    
    // Safe library loading
    let lib = Library::new(grammar)?;
    let get_language = unsafe { 
        lib.get::<unsafe extern "C" fn() -> Language>(symbol.as_bytes())?
    };
    
    // Parse with safety checks
    let language = unsafe { get_language() };
    validate_language(&language)?;
    
    let mut parser = Parser::new();
    parser.set_language(&language)?;
    
    let tree = parser.parse(input_content, None)?;
    format_output(&tree, format)?;
    
    Ok(())
}
```

## Grammar Definition APIs

### Basic Types

```rust
#[rust_sitter::language]
pub struct Program {
    #[rust_sitter::repeat] 
    pub statements: Vec<Statement>,
}

#[rust_sitter::language]
pub enum Statement {
    Expression(Expression),
    Declaration(Declaration),
}

#[rust_sitter::language]
pub struct Identifier {
    #[rust_sitter::leaf(pattern = r"[a-zA-Z_]\w*")]
    pub name: String,
}
```

### Advanced Features

**Precedence:**
```rust
#[rust_sitter::prec_left(1)]
Add(Box<Expr>, #[rust_sitter::leaf(text = "+")] (), Box<Expr>),

#[rust_sitter::prec_left(2)]  
Multiply(Box<Expr>, #[rust_sitter::leaf(text = "*")] (), Box<Expr>),
```

**Optional Fields:**
```rust
pub struct Function {
    pub name: Identifier,
    pub params: Option<ParamList>,  // Automatically optional
    pub body: Block,
}
```

**Delimited Lists:**
```rust
pub struct ArgList {
    #[rust_sitter::repeat(separator = ",")]
    pub args: Vec<Expression>,
}
```

**External Scanners:**
```rust
#[rust_sitter::external_scanner]
pub struct IndentationScanner {
    // External scanner implementation
}

#[rust_sitter::external_token]  
pub struct Indent {
    #[rust_sitter::scanner_ref(IndentationScanner)]
    scanner: (),
}
```

## Error Handling APIs

### Parse Errors

```rust
use rust_sitter::{ParseError, ParseResult};

match parser.parse_utf8(input, None) {
    Ok(tree) => {
        // Successful parse
        process_tree(tree)?;
    },
    Err(ParseError::SyntaxError { position, expected, .. }) => {
        eprintln!("Syntax error at {}: expected {}", position, expected);
        
        // GLR parsers may provide partial trees
        if let Some(partial) = error.partial_tree() {
            recover_from_partial(partial)?;
        }
    },
    Err(ParseError::InternalError(msg)) => {
        eprintln!("Parser internal error: {}", msg);
    }
}
```

### AST Extraction Errors

```rust
use rust_sitter::{AstError, AstResult};

match my_grammar::extract_ast(&tree) {
    Ok(ast) => process_ast(ast),
    Err(AstError::MissingField { node_kind, field_name }) => {
        eprintln!("Missing required field '{}' in {}", field_name, node_kind);
    },
    Err(AstError::TypeError { expected, actual, .. }) => {
        eprintln!("Type error: expected {}, got {}", expected, actual);
    },
    Err(AstError::ValidationError(msg)) => {
        eprintln!("AST validation failed: {}", msg);
    }
}
```

## Testing APIs

### Snapshot Testing

```rust
use insta::assert_snapshot;

#[test]
fn test_expression_parsing() {
    let input = "1 + 2 * 3";
    let tree = parse_expression(input).unwrap();
    let formatted = format_tree(&tree);
    
    assert_snapshot!(formatted);
}
```

### Property Testing

```rust
use proptest::prelude::*;
use rust_sitter::testing::roundtrip_test;

proptest! {
    #[test]
    fn test_serialization_roundtrip(
        tree in arbitrary_parse_tree(),
        source in arbitrary_source_text()
    ) {
        roundtrip_test(&tree, &source)?;
    }
}
```

### Performance Testing

```rust
use rust_sitter::testing::{BenchmarkResult, benchmark_parsing};

#[test]
fn test_parsing_performance() {
    let inputs = load_large_test_files();
    
    let results = benchmark_parsing(inputs, 100)?; // 100 iterations
    
    assert!(results.avg_time_ms < 10.0); // Max 10ms average
    assert!(results.memory_mb < 50.0);   // Max 50MB memory
}
```

## Incremental Parsing APIs

### Basic Incremental Parsing

```rust
use rust_sitter::{Parser, Tree, InputEdit};

let mut parser = Parser::new();
parser.set_language(language)?;

// Initial parse
let mut tree = parser.parse_utf8("let x = 1", None)?;

// Make an edit
let edit = InputEdit {
    start_byte: 8,
    old_end_byte: 9,
    new_end_byte: 10,
    start_position: Point::new(0, 8),
    old_end_position: Point::new(0, 9), 
    new_end_position: Point::new(0, 10),
};

tree.edit(&edit);

// Incremental reparse (reuses unchanged nodes)
let new_tree = parser.parse_utf8("let x = 42", Some(&tree))?;
```

### Tree Editing

```rust
use rust_sitter::{Tree, EditError, Point};

// Safe tree editing with error handling
match tree.edit(&edit) {
    Ok(()) => {
        println!("Tree edited successfully");
    },
    Err(EditError::InvalidRange { start, end }) => {
        eprintln!("Invalid edit range: {}..{}", start, end);
    },
    Err(EditError::Overflow) => {
        eprintln!("Edit would cause integer overflow");
    },
    Err(EditError::InvalidPosition(pos)) => {
        eprintln!("Invalid position: {:?}", pos);  
    }
}
```

## Feature Flags

Control functionality with Cargo features:

```toml
[dependencies]
rust-sitter = { version = "0.6", features = [
    "glr-core",      # GLR parsing engine
    "incremental",   # Incremental parsing
    "serialization", # Tree serialization
    "external_scanners", # External scanner support  
    "pure-rust",     # Pure Rust implementation
    "tree-sitter-standard", # Standard Tree-sitter runtime
    "tree-sitter-c2rust",   # Pure Rust Tree-sitter runtime
    "all-features"   # Enable everything
]}
```

**Feature Combinations:**
- `default` = `["tree-sitter-c2rust", "incremental"]`
- `pure-rust` = `["glr-core", "pure-rust", "serialization"]`
- `tree-sitter-compat` = `["tree-sitter-standard", "incremental"]`

## Platform Support

### Rust Versions

- **MSRV**: Rust 1.89.0
- **Edition**: 2024 (required)
- **Components**: rustfmt, clippy

### Target Platforms

**Tier 1 Support:**
- `x86_64-unknown-linux-gnu`
- `x86_64-pc-windows-msvc`  
- `x86_64-apple-darwin`
- `aarch64-apple-darwin` (Apple Silicon)

**WebAssembly:**
- `wasm32-unknown-unknown` (pure-Rust features only)
- `wasm32-wasi` (with filesystem access)

**Embedded:**
- `thumbv7em-none-eabihf` (ARM Cortex-M4, no-std)
- Limited feature set for embedded targets

### System Dependencies

**Optional (for specific features):**
- `libtree-sitter-dev` - Required for ts-bridge tool
- `libclang` - Required for some binding generation
- Dynamic libraries (`.so/.dylib/.dll`) for dynamic loading

## Migration Guide

### From v0.5 to v0.6

**Breaking Changes:**
1. `SymbolMetadata` struct field renames
2. New GLR runtime API in runtime2  
3. Enhanced serialization format

**Migration Steps:**
```rust
// Old (v0.5)
if symbol.is_visible { /* ... */ }
if symbol.is_terminal { /* ... */ }

// New (v0.6)  
if symbol.visible { /* ... */ }
if symbol.terminal { /* ... */ }
```

**New Features:**
- Dynamic loading with `--dynamic` flag
- Enhanced serialization formats
- Improved FFI safety
- GLR runtime integration

See [MIGRATION_GUIDE.md](../../MIGRATION_GUIDE.md) for complete migration instructions.