# Rust-Sitter Usage Examples

> **v0.5.0-beta:** This is a beta release. The examples below are intended to demonstrate the core features of Rust Sitter.

This document provides comprehensive examples of using the pure-Rust Tree-sitter implementation.

## Table of Contents
1. [Basic Grammar Definition](#basic-grammar-definition)
2. [JSON Parser](#json-parser)
3. [Error Handling](#error-handling)
4. [Tree Traversal](#tree-traversal)
5. [Custom Transformations](#custom-transformations)

## Basic Grammar Definition

### Simple Calculator

```rust
#[rust_sitter::grammar("calculator")]
pub mod grammar {
    #[rust_sitter::language]
    #[derive(Debug)]
    pub enum Expression {
        Number(
            #[rust_sitter::leaf(pattern = r"-?\d+(\.\d+)?", transform = |v| v.parse().unwrap())]
            f64
        ),
        #[rust_sitter::prec_left(1)]
        Add(Box<Expression>, #[rust_sitter::leaf(text = "+")] (), Box<Expression>),
        #[rust_sitter::prec_left(1)]
        Subtract(Box<Expression>, #[rust_sitter::leaf(text = "-")] (), Box<Expression>),
        #[rust_sitter::prec_left(2)]
        Multiply(Box<Expression>, #[rust_sitter::leaf(text = "*")] (), Box<Expression>),
        #[rust_sitter::prec_left(2)]
        Divide(Box<Expression>, #[rust_sitter::leaf(text = "/")] (), Box<Expression>),
        #[rust_sitter::prec(3)]
        Parenthesized(
            #[rust_sitter::leaf(text = "(")] (),
            Box<Expression>,
            #[rust_sitter::leaf(text = ")")] ()
        ),
    }

    #[rust_sitter::extra]
    struct Whitespace {
        #[rust_sitter::leaf(pattern = r"\s+")]
        _ws: (),
    }
}

// Usage
fn main() {
    let input = "2 + 3 * (4 - 1)";
    match grammar::parse(input) {
        Ok(expr) => println!("Parsed: {:#?}", expr),
        Err(e) => eprintln!("Parse error: {:?}", e),
    }
}
```

## JSON Parser

### Complete JSON Grammar

```rust
#[rust_sitter::grammar("json")]
pub mod json_grammar {
    use std::collections::HashMap;

    #[rust_sitter::language]
    #[derive(Debug)]
    pub enum Value {
        Null(#[rust_sitter::leaf(text = "null")] ()),
        Bool(bool),
        Number(f64),
        String(String),
        Array(Vec<Value>),
        Object(HashMap<String, Value>),
    }

    #[rust_sitter::extra]
    struct Whitespace {
        #[rust_sitter::leaf(pattern = r"\s+")]
        _ws: (),
    }
}
```

## Error Handling

### Comprehensive Error Handling

```rust
use rust_sitter::errors::{ParseError, ParseErrorReason};

fn parse_with_diagnostics(input: &str) {
    match grammar::parse(input) {
        Ok(ast) => {
            println!("Successfully parsed!");
        }
        Err(errors) => {
            for error in errors {
                println!("Error: {:?}", error);
            }
        }
    }
}
```

## Tree Traversal

### Using the Visitor API

```rust
use rust_sitter::visitor::{Visitor, VisitorAction};
use rust_sitter::tree_sitter::Node;

// Count specific node types
struct NodeCounter {
    node_type: String,
    count: usize,
}

impl Visitor for NodeCounter {
    fn enter_node(&mut self, node: &Node) -> VisitorAction {
        if node.kind() == self.node_type {
            self.count += 1;
        }
        VisitorAction::Continue
    }
}
```

## Custom Transformations

### Building an Interpreter

```rust
// Calculator interpreter
impl calculator::Expression {
    fn evaluate(&self) -> f64 {
        use calculator::Expression::*;
        match self {
            Number(n) => *n,
            Add(left, _, right) => left.evaluate() + right.evaluate(),
            Subtract(left, _, right) => left.evaluate() - right.evaluate(),
            Multiply(left, _, right) => left.evaluate() * right.evaluate(),
            Divide(left, _, right) => left.evaluate() / right.evaluate(),
            Parenthesized(_, expr, _) => expr.evaluate(),
        }
    }
}
```
