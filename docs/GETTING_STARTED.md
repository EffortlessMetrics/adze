# Getting Started with Rust-Sitter

A comprehensive guide to building parsers with Rust-Sitter's macro-based grammar generation.

## Table of Contents

1. [Quick Start](#quick-start)
2. [Basic Concepts](#basic-concepts)
3. [Creating Your First Grammar](#creating-your-first-grammar)
4. [Working Examples](#working-examples)
5. [Common Patterns](#common-patterns)
6. [Troubleshooting](#troubleshooting)

## Quick Start

### Installation

Add rust-sitter to your `Cargo.toml`:

```toml
[dependencies]
rust-sitter = "0.6.1"

[build-dependencies]
rust-sitter-tool = "0.6.1"
```

### Create a Simple Grammar

Create `src/lib.rs`:

```rust
#[rust_sitter::grammar("my_lang")]
pub mod grammar {
    #[rust_sitter::language]
    pub struct Program {
        #[rust_sitter::leaf(pattern = r"\d+", text = true)]
        pub number: String,
    }
}

#[cfg(test)]
mod tests {
    use super::grammar;

    #[test]
    fn test_parse() {
        let result = grammar::parse("42");
        assert!(result.is_ok());
        let program = result.unwrap();
        assert_eq!(program.number, "42");
    }
}
```

### Build and Run

```bash
cargo build
cargo test
```

That's it! You now have a working parser for numbers.

## Basic Concepts

### Grammar Attributes

- **`#[rust_sitter::grammar("name")]`**: Marks a module as a grammar definition
- **`#[rust_sitter::language]`**: Marks a type as part of the grammar
- **`#[rust_sitter::leaf]`**: Defines a terminal symbol (token)
- **`#[rust_sitter::extra]`**: Defines symbols to ignore (like whitespace)
- **`#[rust_sitter::repeat]`**: Allows repetition of elements
- **`#[rust_sitter::prec]`**: Sets precedence levels for disambiguation

### Grammar Types

1. **Structs**: Sequences of required fields
2. **Enums**: Alternatives (choice between variants)
3. **Vec<T>**: Repetition of elements
4. **Option<T>**: Optional elements
5. **Box<T>**: Recursive structures

## Creating Your First Grammar

### Example 1: Simple Calculator

```rust
#[rust_sitter::grammar("calc")]
pub mod grammar {
    #[rust_sitter::language]
    pub struct Program {
        pub expression: Expression,
    }

    #[rust_sitter::language]
    pub enum Expression {
        Number(NumberLiteral),
        #[rust_sitter::prec_left(1)]
        Add(Box<Expression>, #[rust_sitter::leaf(text = "+")] (), Box<Expression>),
        #[rust_sitter::prec_left(2)]
        Multiply(Box<Expression>, #[rust_sitter::leaf(text = "*")] (), Box<Expression>),
    }

    #[rust_sitter::language]
    pub struct NumberLiteral {
        #[rust_sitter::leaf(pattern = r"\d+", text = true)]
        pub value: String,
    }

    #[rust_sitter::extra]
    #[allow(dead_code)]
    struct Whitespace {
        #[rust_sitter::leaf(pattern = r"\s+")]
        _ws: (),
    }
}
```

### Example 2: Lists and Repetition

```rust
#[rust_sitter::grammar("list")]
pub mod grammar {
    #[rust_sitter::language]
    pub struct ItemList {
        #[rust_sitter::repeat(non_empty = false)]
        pub items: Vec<Item>,
    }

    #[rust_sitter::language]
    pub struct Item {
        #[rust_sitter::leaf(pattern = r"\w+", text = true)]
        pub name: String,
    }

    #[rust_sitter::extra]
    #[allow(dead_code)]
    struct Whitespace {
        #[rust_sitter::leaf(pattern = r"\s+")]
        _ws: (),
    }
}
```

### Example 3: JSON-like Structure

```rust
#[rust_sitter::grammar("json_simple")]
pub mod grammar {
    #[rust_sitter::language]
    pub enum Value {
        Number(NumberLiteral),
        String(StringLiteral),
        Object(Object),
        Array(Array),
    }

    #[rust_sitter::language]
    pub struct Object {
        #[rust_sitter::leaf(text = "{")]
        _open: (),
        #[rust_sitter::repeat(non_empty = false)]
        pub pairs: Vec<Pair>,
        #[rust_sitter::leaf(text = "}")]
        _close: (),
    }

    #[rust_sitter::language]
    pub struct Pair {
        pub key: StringLiteral,
        #[rust_sitter::leaf(text = ":")]
        _colon: (),
        pub value: Value,
    }

    #[rust_sitter::language]
    pub struct Array {
        #[rust_sitter::leaf(text = "[")]
        _open: (),
        #[rust_sitter::repeat(non_empty = false)]
        pub items: Vec<Value>,
        #[rust_sitter::leaf(text = "]")]
        _close: (),
    }

    #[rust_sitter::language]
    pub struct NumberLiteral {
        #[rust_sitter::leaf(pattern = r"-?\d+", text = true)]
        pub value: String,
    }

    #[rust_sitter::language]
    pub struct StringLiteral {
        #[rust_sitter::leaf(pattern = r#""[^"]*""#, text = true)]
        pub value: String,
    }

    #[rust_sitter::extra]
    #[allow(dead_code)]
    struct Whitespace {
        #[rust_sitter::leaf(pattern = r"\s+")]
        _ws: (),
    }
}
```

## Working Examples

The following examples are included in the repository and are fully working:

### test-mini

The simplest possible grammar - parses a single number.

**Location**: `/grammars/test-mini/`

**Usage**:
```bash
cd grammars/test-mini
cargo test
```

### test-vec-wrapper

Demonstrates Vec repetition with multiple numbers.

**Location**: `/grammars/test-vec-wrapper/`

**Features**:
- Repetition with `Vec<>`
- Multiple statements
- Whitespace handling

### arithmetic (example crate)

Full expression parser with precedence.

**Location**: `/example/src/arithmetic.rs`

**Features**:
- Operator precedence
- Left associativity
- Recursive expressions

## Common Patterns

### Text Extraction

To extract the actual text from a token, use `text = true`:

```rust
#[rust_sitter::leaf(pattern = r"\d+", text = true)]
pub value: String,
```

Without `text = true`, you get unit type `()`.

### Ignoring Whitespace

Use `#[rust_sitter::extra]` for whitespace:

```rust
#[rust_sitter::extra]
#[allow(dead_code)]
struct Whitespace {
    #[rust_sitter::leaf(pattern = r"\s+")]
    _ws: (),
}
```

### Precedence

Use precedence numbers for operators (higher = tighter binding):

```rust
#[rust_sitter::prec_left(1)]  // Lower precedence
Add(Box<Expression>, #[rust_sitter::leaf(text = "+")] (), Box<Expression>),

#[rust_sitter::prec_left(2)]  // Higher precedence
Multiply(Box<Expression>, #[rust_sitter::leaf(text = "*")] (), Box<Expression>),
```

This ensures `1 + 2 * 3` parses as `1 + (2 * 3)`.

### Repetition

For zero or more items:
```rust
#[rust_sitter::repeat(non_empty = false)]
pub items: Vec<Item>,
```

For one or more items:
```rust
#[rust_sitter::repeat(non_empty = true)]
pub items: Vec<Item>,
```

### Optional Elements

Use `Option<T>` for optional elements:

```rust
pub optional_field: Option<Identifier>,
```

## Troubleshooting

### Common Issues

#### "Could not determine enum variant from tree structure"

This error occurs when the Extract trait cannot determine which variant to use. Make sure your enum variants have distinct structures.

**Solution**: Ensure each variant has unique structure or use named structs instead of inline tuples.

#### Parse errors at position 0

Usually means a token_count issue. This was fixed in v0.6.1.

**Solution**: Upgrade to rust-sitter 0.6.1 or later.

#### Empty string extraction

Forgot `text = true` on leaf pattern.

**Solution**: Add `text = true` to the `#[rust_sitter::leaf]` attribute:

```rust
#[rust_sitter::leaf(pattern = r"\d+", text = true)]
pub value: String,
```

### Debug Tips

1. **Add Debug derives** to your types:
   ```rust
   #[derive(Debug)]
   #[rust_sitter::language]
   pub struct MyType { ... }
   ```

2. **Check parse result**:
   ```rust
   match grammar::parse(input) {
       Ok(result) => println!("Success: {:?}", result),
       Err(errors) => println!("Errors: {:?}", errors),
   }
   ```

3. **Use `--nocapture`** with tests:
   ```bash
   cargo test -- --nocapture
   ```

## Next Steps

- Read the [API Documentation](../API_DOCUMENTATION.md) for advanced features
- Check out [Grammar Examples](./GRAMMAR_EXAMPLES.md) for more patterns
- Explore the [Performance Guide](./PERFORMANCE_GUIDE.md) for optimization tips
- Review the [Project Status](../PROJECT_STATUS.md) for latest features

## Support

- **Issues**: [GitHub Issues](https://github.com/EffortlessMetrics/rust-sitter/issues)
- **Documentation**: [Comprehensive Docs](https://hydro-project.github.io/rust-sitter/)
- **Examples**: See `/example/` and `/grammars/` directories

## Success Stories

### What's Working (v0.6.1)

✅ **Macro-Based Grammar Generation**: 100% working (9/9 tests passing)
✅ **Text Extraction**: Leaf nodes properly extract source text
✅ **Repetition**: Vec<> fields fully functional
✅ **Precedence**: Operator precedence working correctly
✅ **Whitespace**: Extra symbols properly ignored
✅ **Parse Pipeline**: Token → Shift → Reduce → GOTO → Accept all working

### Test Results

- **test-mini**: 6/6 passing
- **test-vec-wrapper**: 3/3 passing
- **Total workspace**: 381/381 tests passing

Happy parsing! 🎉
