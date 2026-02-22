# Adze Grammar Examples

This document provides comprehensive examples of how to define grammars using Adze 0.8.0-dev.

## Table of Contents

1. [Basic Grammar Structure](#basic-grammar-structure)
2. [Leaf Patterns](#leaf-patterns)
3. [Repetition and Optionals](#repetition-and-optionals)
4. [Enums and Variants](#enums-and-variants)
5. [Complex Grammars](#complex-grammars)

## Basic Grammar Structure

Every Adze grammar starts with the `#[adze::grammar]` attribute:

```rust
#[adze::grammar("my_language")]
pub mod grammar {
    #[adze::language]
    pub struct Program {
        pub statement: Statement,
    }
    
    #[adze::language]
    pub struct Statement {
        pub value: Expression,
        #[adze::leaf(text = ";")]
        _semicolon: (),
    }
    
    #[adze::language]
    pub struct Expression {
        // Automatically extracts text into the String
        #[adze::leaf(pattern = r"\d+")]
        pub number: String,
    }
}
```

## Leaf Patterns

Leaf nodes represent terminal symbols in your grammar:

### Exact Text Match

```rust
#[adze::language]
pub struct Keywords {
    #[adze::leaf(text = "if")]
    _if: (),
    
    #[adze::leaf(text = "else")]
    _else: (),
    
    #[adze::leaf(text = "return")]
    _return: (),
}
```

### Pattern Matching

```rust
#[adze::language]
pub struct Tokens {
    // Identifier pattern
    #[adze::leaf(pattern = r"[a-zA-Z_][a-zA-Z0-9_]*")]
    pub identifier: String,
    
    // Number patterns
    #[adze::leaf(pattern = r"\d+")]
    pub integer: String,
    
    #[adze::leaf(pattern = r"\d+\.\d*")]
    pub float: String,
    
    // String patterns
    #[adze::leaf(pattern = r#""([^"\\]|\\.)*""#)]
    pub string: String,
}
```

### Transformation (Manual)

*Note: Built-in `transform` closures are currently disabled (see Friction Log).*
To transform values (e.g. string to integer), use the `String` type in your grammar and parse it in your application logic:

```rust
#[adze::language]
pub struct Numbers {
    // Parse integer values
    #[adze::leaf(pattern = r"\d+")]
    pub int_text: String,
}

impl Numbers {
    pub fn value(&self) -> i32 {
        self.int_text.parse().unwrap()
    }
}
```

## Repetition and Optionals

### Optional Fields

```rust
#[adze::language]
pub struct Function {
    #[adze::leaf(text = "fn")]
    _fn: (),
    pub name: Identifier,
    pub params: Parameters,
    pub return_type: Option<ReturnType>,
    pub body: Block,
}

#[adze::language]
pub struct ReturnType {
    #[adze::leaf(text = "->")]
    _arrow: (),
    pub type_name: Type,
}
```

### Repetition (Zero or More)

```rust
#[adze::language]
pub struct Block {
    #[adze::leaf(text = "{")]
    _open: (),
    #[adze::repeat]
    pub statements: Vec<Statement>,
    #[adze::leaf(text = "}")]
    _close: (),
}
```

### Non-Empty Repetition (One or More)

```rust
#[adze::language]
pub struct ParameterList {
    #[adze::leaf(text = "(")]
    _open: (),
    #[adze::repeat(non_empty = true)]
    pub params: Vec<Parameter>,
    #[adze::leaf(text = ")")]
    _close: (),
}
```

## Enums and Variants

Enums represent choice points in your grammar:

```rust
#[adze::language]
pub enum Expression {
    Binary(Box<BinaryExpr>), // Box recursive types!
    Unary(Box<UnaryExpr>),
    Literal(Literal),
    Identifier(Identifier),
    Call(Box<CallExpr>),
}

#[adze::language]
pub struct BinaryExpr {
    pub left: Expression,
    pub op: BinaryOp,
    pub right: Expression,
}

#[adze::language]
pub enum BinaryOp {
    Add(AddOp),
    Sub(SubOp),
    Mul(MulOp),
    Div(DivOp),
}

#[adze::language]
pub struct AddOp {
    #[adze::leaf(text = "+")]
    _op: (),
}
```

## Complex Grammars

### JSON Grammar Example

```rust
#[adze::grammar("json")]
pub mod json_grammar {
    #[adze::language]
    pub struct Document {
        pub value: Value,
    }
    
    #[adze::language]
    pub enum Value {
        Object(Object),
        Array(Array),
        String(StringLit),
        Number(Number),
        Boolean(Boolean),
        Null(Null),
    }
    
    #[adze::language]
    pub struct Object {
        #[adze::leaf(text = "{")]
        _open: (),
        #[adze::repeat]
        pub members: Vec<Member>,
        #[adze::leaf(text = "}")]
        _close: (),
    }
    
    #[adze::language]
    pub struct Member {
        pub key: StringLit,
        #[adze::leaf(text = ":")]
        _colon: (),
        pub value: Value,
        pub comma: Option<Comma>,
    }
    
    #[adze::language]
    pub struct Comma {
        #[adze::leaf(text = ",")]
        _comma: (),
    }
    
    #[adze::language]
    pub struct Array {
        #[adze::leaf(text = "[")]
        _open: (),
        #[adze::repeat]
        pub elements: Vec<Element>,
        #[adze::leaf(text = "]")]
        _close: (),
    }
    
    #[adze::language]
    pub struct Element {
        pub value: Value,
        pub comma: Option<Comma>,
    }
    
    #[adze::language]
    pub struct StringLit {
        #[adze::leaf(pattern = r#""([^"\\]|\\.)*""#)]
        pub value: String,
    }
    
    #[adze::language]
    pub struct Number {
        #[adze::leaf(pattern = r"-?(?:0|[1-9]\d*)(?:\.\d+)?(?:[eE][+-]?\d+)?")]
        pub value: String,
    }
    
    #[adze::language]
    pub enum Boolean {
        True(True),
        False(False),
    }
    
    #[adze::language]
    pub struct True {
        #[adze::leaf(text = "true")]
        _true: (),
    }
    
    #[adze::language]
    pub struct False {
        #[adze::leaf(text = "false")]
        _false: (),
    }
    
    #[adze::language]
    pub struct Null {
        #[adze::leaf(text = "null")]
        _null: (),
    }
}
```

## Best Practices

1. **Use underscores for syntax-only fields**: Fields that represent punctuation or keywords should start with `_` to indicate they're not semantically important.

2. **Box recursive types**: When you have recursive structures (like expressions), use `Box<T>` to avoid infinite-size types.

3. **Prefer enums for alternatives**: Use enums to represent different variants of a language construct.

4. **Use Option for optional syntax**: When a language feature is optional, use `Option<T>`.

5. **Use Vec for repetitions**: The `#[adze::repeat]` attribute works with `Vec<T>`.

## Using Your Grammar

Once you've defined your grammar, add it to your `build.rs`:

```rust
use adze_tool::build_parsers;
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
    let ast = parse("some input").unwrap();
}
```
