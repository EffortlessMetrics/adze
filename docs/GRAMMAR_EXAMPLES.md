# Rust-Sitter Grammar Examples

This document provides comprehensive examples of how to define grammars using rust-sitter v0.5.0-beta.

## Table of Contents

1. [Basic Grammar Structure](#basic-grammar-structure)
2. [Leaf Patterns](#leaf-patterns)
3. [Repetition and Optionals](#repetition-and-optionals)
4. [Enums and Variants](#enums-and-variants)
5. [Complex Grammars](#complex-grammars)

## Basic Grammar Structure

Every rust-sitter grammar starts with the `#[rust_sitter::grammar]` attribute:

```rust
#[rust_sitter::grammar("my_language")]
pub mod grammar {
    #[rust_sitter::language]
    pub struct Program {
        pub statement: Statement,
    }
    
    #[rust_sitter::language]
    pub struct Statement {
        pub value: Expression,
        #[rust_sitter::leaf(text = ";")]
        _semicolon: (),
    }
    
    #[rust_sitter::language]
    pub struct Expression {
        #[rust_sitter::leaf(pattern = r"\d+")]
        pub number: String,
    }
}
```

## Leaf Patterns

Leaf nodes represent terminal symbols in your grammar:

### Exact Text Match

```rust
#[rust_sitter::language]
pub struct Keywords {
    #[rust_sitter::leaf(text = "if")]
    _if: (),
    
    #[rust_sitter::leaf(text = "else")]
    _else: (),
    
    #[rust_sitter::leaf(text = "return")]
    _return: (),
}
```

### Pattern Matching

```rust
#[rust_sitter::language]
pub struct Tokens {
    // Identifier pattern
    #[rust_sitter::leaf(pattern = r"[a-zA-Z_][a-zA-Z0-9_]*")]
    pub identifier: String,
    
    // Number patterns
    #[rust_sitter::leaf(pattern = r"\d+")]
    pub integer: String,
    
    #[rust_sitter::leaf(pattern = r"\d+\.\d*")]
    pub float: String,
    
    // String patterns
    #[rust_sitter::leaf(pattern = r#""([^"\\]|\\.)*""#)]
    pub string: String,
}
```

### Transformation

```rust
#[rust_sitter::language]
pub struct Numbers {
    // Parse integer values
    #[rust_sitter::leaf(pattern = r"\d+", transform = |s| s.parse().unwrap())]
    pub int_value: i32,
    
    // Parse float values
    #[rust_sitter::leaf(pattern = r"\d+\.\d*", transform = |s| s.parse().unwrap())]
    pub float_value: f64,
}
```

## Repetition and Optionals

### Optional Fields

```rust
#[rust_sitter::language]
pub struct Function {
    #[rust_sitter::leaf(text = "fn")]
    _fn: (),
    pub name: Identifier,
    pub params: Parameters,
    pub return_type: Option<ReturnType>,
    pub body: Block,
}

#[rust_sitter::language]
pub struct ReturnType {
    #[rust_sitter::leaf(text = "->")]
    _arrow: (),
    pub type_name: Type,
}
```

### Repetition (Zero or More)

```rust
#[rust_sitter::language]
pub struct Block {
    #[rust_sitter::leaf(text = "{")]
    _open: (),
    #[rust_sitter::repeat]
    pub statements: Vec<Statement>,
    #[rust_sitter::leaf(text = "}")]
    _close: (),
}
```

### Non-Empty Repetition (One or More)

```rust
#[rust_sitter::language]
pub struct ParameterList {
    #[rust_sitter::leaf(text = "(")]
    _open: (),
    #[rust_sitter::repeat(non_empty = true)]
    pub params: Vec<Parameter>,
    #[rust_sitter::leaf(text = ")")]
    _close: (),
}
```

## Enums and Variants

Enums represent choice points in your grammar:

```rust
#[rust_sitter::language]
pub enum Expression {
    Binary(BinaryExpr),
    Unary(UnaryExpr),
    Literal(Literal),
    Identifier(Identifier),
    Call(CallExpr),
}

#[rust_sitter::language]
pub struct BinaryExpr {
    pub left: Box<Expression>,
    pub op: BinaryOp,
    pub right: Box<Expression>,
}

#[rust_sitter::language]
pub enum BinaryOp {
    Add(AddOp),
    Sub(SubOp),
    Mul(MulOp),
    Div(DivOp),
}

#[rust_sitter::language]
pub struct AddOp {
    #[rust_sitter::leaf(text = "+")]
    _op: (),
}
```

## Complex Grammars

### JSON Grammar Example

```rust
#[rust_sitter::grammar("json")]
pub mod json_grammar {
    #[rust_sitter::language]
    pub struct Document {
        pub value: Value,
    }
    
    #[rust_sitter::language]
    pub enum Value {
        Object(Object),
        Array(Array),
        String(StringLit),
        Number(Number),
        Boolean(Boolean),
        Null(Null),
    }
    
    #[rust_sitter::language]
    pub struct Object {
        #[rust_sitter::leaf(text = "{")]
        _open: (),
        #[rust_sitter::repeat]
        pub members: Vec<Member>,
        #[rust_sitter::leaf(text = "}")]
        _close: (),
    }
    
    #[rust_sitter::language]
    pub struct Member {
        pub key: StringLit,
        #[rust_sitter::leaf(text = ":")]
        _colon: (),
        pub value: Value,
        pub comma: Option<Comma>,
    }
    
    #[rust_sitter::language]
    pub struct Comma {
        #[rust_sitter::leaf(text = ",")]
        _comma: (),
    }
    
    #[rust_sitter::language]
    pub struct Array {
        #[rust_sitter::leaf(text = "[")]
        _open: (),
        #[rust_sitter::repeat]
        pub elements: Vec<Element>,
        #[rust_sitter::leaf(text = "]")]
        _close: (),
    }
    
    #[rust_sitter::language]
    pub struct Element {
        pub value: Value,
        pub comma: Option<Comma>,
    }
    
    #[rust_sitter::language]
    pub struct StringLit {
        #[rust_sitter::leaf(pattern = r#""([^"\\]|\\.)*""#)]
        pub value: String,
    }
    
    #[rust_sitter::language]
    pub struct Number {
        #[rust_sitter::leaf(pattern = r"-?(?:0|[1-9]\d*)(?:\.\d+)?(?:[eE][+-]?\d+)?")]
        pub value: String,
    }
    
    #[rust_sitter::language]
    pub enum Boolean {
        True(True),
        False(False),
    }
    
    #[rust_sitter::language]
    pub struct True {
        #[rust_sitter::leaf(text = "true")]
        _true: (),
    }
    
    #[rust_sitter::language]
    pub struct False {
        #[rust_sitter::leaf(text = "false")]
        _false: (),
    }
    
    #[rust_sitter::language]
    pub struct Null {
        #[rust_sitter::leaf(text = "null")]
        _null: (),
    }
}
```

### Expression Grammar with Operators

```rust
#[rust_sitter::grammar("calc")]
pub mod calc_grammar {
    #[rust_sitter::language]
    pub struct Program {
        pub expression: Expression,
    }
    
    #[rust_sitter::language]
    pub enum Expression {
        Binary(Box<BinaryExpression>),
        Unary(Box<UnaryExpression>),
        Primary(PrimaryExpression),
    }
    
    #[rust_sitter::language]
    pub struct BinaryExpression {
        pub left: Expression,
        pub operator: BinaryOperator,
        pub right: Expression,
    }
    
    #[rust_sitter::language]
    pub enum BinaryOperator {
        Add(AddOp),
        Subtract(SubOp),
        Multiply(MulOp),
        Divide(DivOp),
        Power(PowerOp),
    }
    
    #[rust_sitter::language]
    pub struct AddOp {
        #[rust_sitter::leaf(text = "+")]
        _op: (),
    }
    
    #[rust_sitter::language]
    pub struct SubOp {
        #[rust_sitter::leaf(text = "-")]
        _op: (),
    }
    
    #[rust_sitter::language]
    pub struct MulOp {
        #[rust_sitter::leaf(text = "*")]
        _op: (),
    }
    
    #[rust_sitter::language]
    pub struct DivOp {
        #[rust_sitter::leaf(text = "/")]
        _op: (),
    }
    
    #[rust_sitter::language]
    pub struct PowerOp {
        #[rust_sitter::leaf(text = "^")]
        _op: (),
    }
    
    #[rust_sitter::language]
    pub struct UnaryExpression {
        pub operator: UnaryOperator,
        pub operand: Expression,
    }
    
    #[rust_sitter::language]
    pub enum UnaryOperator {
        Plus(UnaryPlusOp),
        Minus(UnaryMinusOp),
    }
    
    #[rust_sitter::language]
    pub struct UnaryPlusOp {
        #[rust_sitter::leaf(text = "+")]
        _op: (),
    }
    
    #[rust_sitter::language]
    pub struct UnaryMinusOp {
        #[rust_sitter::leaf(text = "-")]
        _op: (),
    }
    
    #[rust_sitter::language]
    pub enum PrimaryExpression {
        Number(Number),
        Identifier(Identifier),
        Parenthesized(Box<ParenthesizedExpression>),
    }
    
    #[rust_sitter::language]
    pub struct Number {
        #[rust_sitter::leaf(pattern = r"\d+(?:\.\d+)?", transform = |s| s.parse::<f64>().unwrap())]
        pub value: f64,
    }
    
    #[rust_sitter::language]
    pub struct Identifier {
        #[rust_sitter::leaf(pattern = r"[a-zA-Z_][a-zA-Z0-9_]*")]
        pub name: String,
    }
    
    #[rust_sitter::language]
    pub struct ParenthesizedExpression {
        #[rust_sitter::leaf(text = "(")]
        _open: (),
        pub expression: Expression,
        #[rust_sitter::leaf(text = ")")]
        _close: (),
    }
}
```

## Best Practices

1. **Use underscores for syntax-only fields**: Fields that represent punctuation or keywords should start with `_` to indicate they're not semantically important.

2. **Box recursive types**: When you have recursive structures (like expressions), use `Box<T>` to avoid infinite-size types.

3. **Prefer enums for alternatives**: Use enums to represent different variants of a language construct.

4. **Use Option for optional syntax**: When a language feature is optional, use `Option<T>`.

5. **Use Vec for repetitions**: The `#[rust_sitter::repeat]` attribute works with `Vec<T>`.

## Current Limitations

The v0.5.0-beta release has some limitations:

- No support for precedence annotations (`#[rust_sitter::prec]`)
- No support for associativity (`#[rust_sitter::prec_left]`, `#[rust_sitter::prec_right]`)
- No support for external scanners (`#[rust_sitter::external]`)
- No support for word tokens (`#[rust_sitter::word]`)
- No support for delimited lists (`#[rust_sitter::delimited]`)

These features are planned for future releases.

## Using Your Grammar

Once you've defined your grammar, add it to your `build.rs`:

```rust
use rust_sitter_tool::build_parsers;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=src/grammar.rs");
    build_parsers(&PathBuf::from("src/grammar.rs"));
}
```

And use it in your code:

```rust
use my_language::grammar::*;

fn main() {
    // Your parsing code here
}
```