# Rust-Sitter v0.5.0-beta Quick Start Guide

## Installation

Add rust-sitter to your `Cargo.toml`:

```toml
[dependencies]
rust-sitter = "0.5.0-beta"

[build-dependencies]
rust-sitter-tool = "0.5.0-beta"
```

## Creating Your First Grammar

### 1. Define Your Grammar (src/lib.rs)

```rust
#[rust_sitter::grammar("my_language")]
pub mod grammar {
    #[rust_sitter::language]
    pub struct Program {
        #[rust_sitter::repeat]
        pub statements: Vec<Statement>,
    }
    
    #[rust_sitter::language]
    pub enum Statement {
        Assignment(Assignment),
        Expression(Expression),
    }
    
    #[rust_sitter::language]
    pub struct Assignment {
        pub name: Identifier,
        #[rust_sitter::leaf(text = "=")]
        _eq: (),
        pub value: Expression,
    }
    
    #[rust_sitter::language]
    pub enum Expression {
        Number(Number),
        Identifier(Identifier),
    }
    
    #[rust_sitter::language]
    pub struct Number {
        #[rust_sitter::leaf(pattern = r"\d+")]
        pub value: (),
    }
    
    #[rust_sitter::language]
    pub struct Identifier {
        #[rust_sitter::leaf(pattern = r"[a-zA-Z_]\w*")]
        pub name: (),
    }
}
```

### 2. Create Build Script (build.rs)

```rust
use rust_sitter_tool::build_parsers;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=src/lib.rs");
    build_parsers(&PathBuf::from("src/lib.rs"));
}
```

### 3. Use Your Parser

```rust
use my_language::grammar::Program;

fn main() {
    let input = "x = 42\ny = 100";
    match Program::parse(input) {
        Ok(tree) => println!("Parsed successfully: {:#?}", tree),
        Err(e) => println!("Parse error: {}", e),
    }
}
```

## Beta Limitations

### ❌ Not Yet Supported
- Precedence declarations (`#[rust_sitter::prec_left(1)]`)
- External scanners (full API)
- Complex conflict resolution
- Some Tree-sitter features (`word`, `extras`, etc.)

### ✅ What Works
- Basic grammar definitions
- Enums and structs
- Repetitions and optionals
- Pattern matching for tokens
- Simple parsing
- **GLR parsing** (ambiguous grammar support) ✨
- **True incremental parsing** with subtree reuse ✨
- **Performance monitoring** and optimization ✨

## Tips for Beta Users

1. **Keep Grammars Simple** - Avoid complex precedence rules
2. **Test Incrementally** - Build up your grammar piece by piece
3. **Check Examples** - Look at the JavaScript, Python, and Go examples
4. **Report Issues** - This is a beta, your feedback is valuable!

## GLR Features & Performance

### Using GLR Parsing
Enable GLR parsing for ambiguous grammars:

```toml
[dependencies]
rust-sitter = { version = "0.5.0-beta", features = ["glr-core"] }
# Note: GLR runtime is currently in runtime2/ directory (not yet published)
rust-sitter-runtime = { path = "../rust-sitter/runtime2", features = ["glr-core"] }
```

```rust
use rust_sitter_runtime::Parser;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let language = my_language::language();
    let mut parser = Parser::new();
    parser.set_language(language)?;
    
    let tree = parser.parse_utf8("ambiguous input", None)?;
    println!("Parsed with GLR: root kind = {}", tree.root_kind());
    Ok(())
}
```

### Incremental Parsing
Enable true incremental parsing for large files:

```toml
[dependencies]
rust-sitter = { version = "0.5.0-beta", features = ["incremental_glr"] }
```

```rust
use rust_sitter_runtime::{Parser, glr_incremental};

// Monitor reuse effectiveness
glr_incremental::reset_reuse_counter();

let old_tree = parser.parse_utf8("original content", None)?;
let new_tree = parser.parse_utf8("modified content", Some(&old_tree))?;

let reused = glr_incremental::get_reuse_count();
println!("Reused {} subtrees during incremental parse", reused);
```

### Performance Optimization
Enable performance monitoring to optimize your parser:

```rust
use std::env;

// Enable detailed logging
env::set_var("RUST_SITTER_LOG_PERFORMANCE", "true");

// Parse with metrics
let tree = parser.parse_utf8(large_input, None)?;
// Output: "🚀 Forest->Tree conversion: 1247 nodes, depth 23, took 2.1ms"
```

**Optimization Tips:**
- Use incremental parsing for large files or frequent edits
- Monitor subtree reuse with `SUBTREE_REUSE_COUNT` 
- Set `RUST_TEST_THREADS=2` for consistent benchmarking
- Enable `RUST_SITTER_LOG_PERFORMANCE` during development

## Query Matching with Node Metadata (v0.6+)

### Basic Query Usage
Use queries to pattern match against parsed trees with proper node metadata validation:

```rust
use rust_sitter_runtime::{Parser, query::QueryMatcher};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let language = my_language::language();
    let metadata = language.symbol_metadata(); // Get symbol metadata for validation
    let mut parser = Parser::new();
    parser.set_language(language)?;
    
    let source = "function test() { return 42; }";
    let tree = parser.parse_utf8(source, None)?;
    
    // Create a query pattern
    let query_text = r#"
    (function_definition
      name: (identifier) @func_name
      body: (block) @func_body)
    "#;
    
    let query = rust_sitter_runtime::query::compile_query(query_text)?;
    
    // Create matcher with metadata for proper node validation
    let matcher = QueryMatcher::new(&query, source, &metadata);
    let matches = matcher.matches(&tree.root());
    
    for m in matches {
        for capture in m.captures {
            println!("Captured {}: {:?}", capture.index, capture.node);
        }
    }
    
    Ok(())
}
```

### Node Metadata Validation
The query engine automatically uses symbol metadata to:

```rust
// Example: Only match named nodes vs anonymous tokens
let query = compile_query(r#"
  (function_definition          ; Only matches named "function_definition" nodes  
    "function"                  ; Matches anonymous "function" token
    name: (identifier) @name    ; Only matches named "identifier" nodes
    "(" @lparen                 ; Matches anonymous "(" token  
    ")" @rparen                 ; Matches anonymous ")" token
    body: (block) @body)        ; Only matches named "block" nodes
"#)?;

let matcher = QueryMatcher::new(&query, source, &metadata);
// Engine automatically filters based on node metadata:
// - metadata.named determines if patterns match named vs anonymous nodes
// - metadata.is_extra causes extra nodes (whitespace/comments) to be skipped
// - Null-safe access prevents crashes on malformed metadata
```

### Advanced Query Patterns

```rust
// Pattern with predicates and node type filtering
let query = compile_query(r#"
  (function_definition 
    name: (identifier) @func_name
    parameters: (parameter_list) @params)
    
  ; Only match functions starting with "test_"
  (#match? @func_name "^test_")
"#)?;

// Query execution with performance monitoring
env::set_var("RUST_SITTER_LOG_PERFORMANCE", "true");
let matches = matcher.matches(&root);
println!("Found {} test functions", matches.len());
```

## Common Patterns

### Optional Fields
```rust
pub struct Function {
    pub name: Identifier,
    pub params: Option<Parameters>,
}
```

### Repeated Elements
```rust
pub struct Block {
    #[rust_sitter::repeat]
    pub statements: Vec<Statement>,
}
```

### Token Patterns
```rust
pub struct StringLiteral {
    #[rust_sitter::leaf(pattern = r#""[^"]*""#)]
    pub value: (),
}
```

## Troubleshooting

### Grammar Conflicts
If you see "conflict" errors during build:
1. Simplify your grammar
2. Make optional elements explicit
3. Avoid ambiguous patterns

### Missing Features
If a Tree-sitter feature isn't working:
1. Check the known limitations
2. Find a workaround in the examples
3. Wait for the next release 😊

## Next Steps

- Explore the examples in `/examples`
- Read `GRAMMAR_EXAMPLES.md` for more patterns
- Check `KNOWN_LIMITATIONS.md` for current restrictions
- Join the discussion on GitHub

Happy parsing! 🦀🌳