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

    pub enum Statement {
        Assignment(Box<Assignment>),
        Expression(Box<Expression>),
    }

    pub struct Assignment {
        pub name: Identifier,
        #[rust_sitter::leaf(text = "=")]
        _eq: (),
        pub value: Expression,
    }

    pub enum Expression {
        Number(Number),
        Identifier(Identifier),
    }

    pub struct Number {
        #[rust_sitter::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
        pub value: u32,
    }

    pub struct Identifier {
        #[rust_sitter::leaf(pattern = r"[a-zA-Z_]\w*")]
        pub name: String,
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

To use the parser, you can now call the generated `parse` function:

```rust
fn main() {
    let input = "x = 42\ny = 100";
    match my_language::grammar::parse(input) {
        Ok(tree) => println!("Parsed successfully: {:#?}", tree),
        Err(e) => println!("Parse error: {}", e),
    }
}
```

## Next Steps

- Explore the examples in the `/examples` directory in the repository root.
- Read the [Usage Examples](./USAGE_EXAMPLES.md) for more patterns.
- Consult the [API Documentation](./API_DOCUMENTATION.md) for a complete reference.
- Join the discussion on GitHub!

Happy parsing! 🦀🌳