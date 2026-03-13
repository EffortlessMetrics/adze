# Grammar Author's Guide

**Version**: v0.8.0  
**Status**: Production Ready  
**Target Audience**: Grammar developers and language implementers

## Table of Contents

1. [Introduction](#introduction)
2. [Quick Start](#quick-start)
3. [Defining Tokens](#defining-tokens)
4. [Defining Rules](#defining-rules)
5. [Handling Precedence](#handling-precedence)
6. [Repetition and Options](#repetition-and-options)
7. [External Scanners](#external-scanners)
8. [Common Patterns](#common-patterns)
9. [Troubleshooting](#troubleshooting)

---

## Introduction

### What is Adze?

Adze is an AST-first grammar toolchain for Rust. It generates parsers from Rust type annotations using a pure-Rust GLR (Generalized LR) implementation. Unlike traditional parser generators that require learning a separate grammar syntax, Adze lets you define grammars using familiar Rust types and attributes.

### Grammar Definition Approach

Adze uses **Rust types + attributes** to define grammars:

- **Types** (structs and enums) define the structure of your AST
- **Attributes** (`#[adze::...]`) annotate how types map to grammar rules
- **Build-time code generation** produces the parser

This approach provides:

- ✅ **Type safety**: Your AST types are validated at compile time
- ✅ **IDE support**: Full Rust tooling support for grammar definitions
- ✅ **Composability**: Reuse types across grammars naturally
- ✅ **GLR parsing**: Handle ambiguous grammars gracefully

---

## Quick Start

### Minimal Grammar Example

Create a new Rust project and add Adze as a dependency:

```toml
# Cargo.toml
[dependencies]
adze = { version = "0.8" }

[build-dependencies]
adze-tool = { version = "0.8" }
```

Define your grammar in `src/lib.rs`:

```rust
use adze::Extract;

#[adze::grammar("calc")]
pub mod grammar {
    #[adze::language]
    pub struct Program {
        #[adze::repeat(non_empty = true)]
        pub statements: Vec<Statement>,
    }

    #[adze::language]
    pub enum Statement {
        Expression(Expression),
    }

    #[adze::language]
    pub enum Expression {
        Number(#[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())] i32),
        
        #[adze::prec_left(1)]
        Add(Box<Expression>, #[adze::leaf(text = "+")] (), Box<Expression>),
        
        #[adze::prec_left(2)]
        Multiply(Box<Expression>, #[adze::leaf(text = "*")] (), Box<Expression>),
    }

    #[adze::extra]
    struct Whitespace {
        #[adze::leaf(pattern = r"\s+")]
        _ws: (),
    }
}

pub use grammar::*;
```

Add a `build.rs` to generate the parser:

```rust
// build.rs
fn main() {
    adze_tool::build();
}
```

### Building and Testing

```bash
# Build the grammar
cargo build

# Run tests
cargo test

# Parse input programmatically
use my_grammar::grammar::parse;
let tree = parse("1 + 2 * 3").unwrap();
```

### Debug Output

Enable artifact emission to inspect generated grammar files:

```bash
export ADZE_EMIT_ARTIFACTS=true
cargo build
# Check adze_debug_*.log in your temp directory
```

---

## Defining Tokens

Tokens are the terminal symbols of your grammar. Adze provides the `#[adze::leaf]` attribute to define them.

### The `#[adze::leaf]` Attribute

The `#[adze::leaf]` attribute marks a field or variant as a terminal token:

```rust
#[adze::language]
pub struct NumberLiteral {
    #[adze::leaf(pattern = r"\d+")]
    pub value: String,
}
```

### Text Literals

Use `text = "..."` for exact string matches:

```rust
#[adze::language]
pub struct PlusOperator {
    #[adze::leaf(text = "+")]
    _plus: (),
}
```

Text literals are ideal for:
- Keywords (`"if"`, `"while"`, `"fn"`)
- Operators (`"+"`, `"=="`, `"=>"`)
- Punctuation (`","`, `";"`, `"{"`)

### Regex Patterns

Use `pattern = r"..."` for regex-based matching:

```rust
#[adze::language]
pub struct Identifier {
    #[adze::leaf(pattern = r"[a-zA-Z_][a-zA-Z0-9_]*")]
    pub name: String,
}

#[adze::language]
pub struct StringLiteral {
    #[adze::leaf(pattern = r#""([^"\\]|\\.)*""#)]
    pub value: String,
}
```

Pattern examples:

| Token Type | Pattern |
|------------|---------|
| Integer | `r"\d+"` |
| Float | `r"(\d+\.?\d*\|\.\d+)([eE][+-]?\d+)?"` |
| Identifier | `r"[a-zA-Z_][a-zA-Z0-9_]*"` |
| Single-quoted string | `r"'([^'\\]\|\\.)*'"` |
| Double-quoted string | `r#""([^"\\]|\\.)*""#` |
| Line comment | `r"//[^\n]*"` |

### Transform Functions

Use `transform = ...` to convert matched text to Rust types:

```rust
#[adze::language]
pub struct IntegerLiteral {
    #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
    pub value: i32,
}

#[adze::language]
pub struct BooleanLiteral {
    #[adze::leaf(text = "true", transform = |_| true)]
    pub value: bool,
}
```

Transform functions receive `&str` and can return any type that implements `FromStr` or custom types:

```rust
#[adze::language]
pub struct HexNumber {
    #[adze::leaf(
        pattern = r"0x[0-9a-fA-F]+",
        transform = |v| i32::from_str_radix(&v[2..], 16).unwrap()
    )]
    pub value: i32,
}
```

### External Tokens

For tokens that cannot be expressed with regex, use `#[adze::external]`:

```rust
#[adze::grammar("python")]
pub mod grammar {
    #[adze::external]
    pub struct Newline;

    #[adze::external]
    pub struct Indent;

    #[adze::external]
    pub struct Dedent;
}
```

External tokens require implementing an [External Scanner](#external-scanners).

### Extra Tokens (Whitespace/Comments)

Use `#[adze::extra]` for tokens that should be skipped:

```rust
#[adze::extra]
struct Whitespace {
    #[adze::leaf(pattern = r"\s+")]
    _ws: (),
}

#[adze::extra]
struct LineComment {
    #[adze::leaf(pattern = r"//[^\n]*")]
    _comment: (),
}
```

Multiple extra tokens can be combined:

```rust
#[adze::extra]
struct Whitespace {
    #[adze::leaf(pattern = r"\s")]
    _ws: (),
}

#[adze::extra]
struct BlockComment {
    #[adze::leaf(pattern = r"/\*[^*]*\*/")]
    _comment: (),
}
```

---

## Defining Rules

Grammar rules define how non-terminal symbols combine. Adze uses Rust types to represent these structures.

### Struct Rules (Product Types)

Structs represent sequences of symbols:

```rust
#[adze::language]
pub struct Assignment {
    pub target: Identifier,
    #[adze::leaf(text = "=")]
    _equals: (),
    pub value: Expression,
}
```

This generates a rule: `Assignment → Identifier "=" Expression`

#### Named Fields

```rust
#[adze::language]
pub struct FunctionDefinition {
    #[adze::leaf(text = "fn")]
    _fn_keyword: (),
    pub name: Identifier,
    pub parameters: ParameterList,
    #[adze::leaf(text = "->")]
    _arrow: (),
    pub return_type: Type,
    pub body: Block,
}
```

#### Tuple Structs

For simple sequences without named fields:

```rust
#[adze::language]
pub struct ParenExpression(
    #[adze::leaf(text = "(")] (),
    pub Expression,
    #[adze::leaf(text = ")")] (),
);
```

### Enum Rules (Sum Types)

Enums represent alternatives:

```rust
#[adze::language]
pub enum Statement {
    Expression(ExpressionStatement),
    Variable(VariableDeclaration),
    Function(FunctionDeclaration),
    Return(ReturnStatement),
    If(IfStatement),
    Block(BlockStatement),
}
```

This generates rules:
```
Statement → ExpressionStatement
Statement → VariableDeclaration
Statement → FunctionDeclaration
...
```

#### Unit Variants with Tokens

Enums can have unit variants that act as tokens:

```rust
#[adze::language]
pub enum BinaryOperator {
    #[adze::leaf(text = "+")]
    Add,
    #[adze::leaf(text = "-")]
    Subtract,
    #[adze::leaf(text = "*")]
    Multiply,
    #[adze::leaf(text = "/")]
    Divide,
}
```

#### Inline Variant Fields

For concise definitions, use inline tuple variants:

```rust
#[adze::language]
pub enum Expression {
    Number(#[adze::leaf(pattern = r"\d+")] String),
    
    #[adze::prec_left(1)]
    Add(Box<Expression>, #[adze::leaf(text = "+")] (), Box<Expression>),
    
    #[adze::prec_left(2)]
    Multiply(Box<Expression>, #[adze::leaf(text = "*")] (), Box<Expression>),
}
```

#### Struct Variants

For variants with named fields:

```rust
#[adze::language]
pub enum Expression {
    Binary {
        left: Box<Expression>,
        operator: BinaryOperator,
        right: Box<Expression>,
    },
    Unary {
        operator: UnaryOperator,
        operand: Box<Expression>,
    },
    Primary(PrimaryExpression),
}
```

### Recursive Rules

Use `Box<T>` for recursive structures:

```rust
#[adze::language]
pub enum Expression {
    Number(NumberLiteral),
    Binary(Box<BinaryExpression>),
}

#[adze::language]
pub struct BinaryExpression {
    pub left: Expression,  // No Box needed here
    pub operator: BinaryOperator,
    pub right: Expression,
}
```

---

## Handling Precedence

Operator precedence and associativity are controlled with precedence attributes.

### Precedence Attributes

| Attribute | Description | Example |
|-----------|-------------|---------|
| `#[adze::prec(n)]` | Non-associative | Comparison operators |
| `#[adze::prec_left(n)]` | Left-associative | `a - b - c` = `(a - b) - c` |
| `#[adze::prec_right(n)]` | Right-associative | `x = y = z` = `x = (y = z)` |

Higher numbers bind tighter (higher precedence).

### Precedence Levels

```rust
#[adze::language]
pub enum Expression {
    // Primary expressions (highest precedence - no attribute needed)
    Primary(PrimaryExpression),
    
    // Unary operators (precedence 10)
    #[adze::prec(10)]
    Unary(Box<UnaryExpression>),
    
    // Multiplicative (precedence 5 - left-associative)
    #[adze::prec_left(5)]
    Multiply(Box<Expression>, #[adze::leaf(text = "*")] (), Box<Expression>),
    #[adze::prec_left(5)]
    Divide(Box<Expression>, #[adze::leaf(text = "/")] (), Box<Expression>),
    
    // Additive (precedence 4 - left-associative)
    #[adze::prec_left(4)]
    Add(Box<Expression>, #[adze::leaf(text = "+")] (), Box<Expression>),
    #[adze::prec_left(4)]
    Subtract(Box<Expression>, #[adze::leaf(text = "-")] (), Box<Expression>),
    
    // Comparison (precedence 3 - non-associative)
    #[adze::prec(3)]
    LessThan(Box<Expression>, #[adze::leaf(text = "<")] (), Box<Expression>),
    
    // Assignment (precedence 1 - right-associative)
    #[adze::prec_right(1)]
    Assign(Box<Expression>, #[adze::leaf(text = "=")] (), Box<Expression>),
}
```

### Common Precedence Table

| Level | Operators | Associativity |
|-------|-----------|---------------|
| 1 | `=` | Right |
| 2 | `||` | Left |
| 3 | `&&` | Left |
| 4 | `==`, `!=` | Left |
| 5 | `<`, `>`, `<=`, `>=` | Left |
| 6 | `+`, `-` | Left |
| 7 | `*`, `/`, `%` | Left |
| 8 | Unary `-`, `!` | N/A |
| 9 | `.` , `()` , `[]` | Left |

### Example: Expression Grammar

```rust
#[adze::language]
pub enum Expression {
    // Literals and identifiers
    Number(#[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())] i32),
    Identifier(#[adze::leaf(pattern = r"[a-zA-Z_]\w*")] String),
    
    // Parenthesized expression
    Paren(
        #[adze::leaf(text = "(")] (),
        Box<Expression>,
        #[adze::leaf(text = ")")] (),
    ),
    
    // Binary operators with precedence
    #[adze::prec_left(1)]
    Add(Box<Expression>, #[adze::leaf(text = "+")] (), Box<Expression>),
    
    #[adze::prec_left(1)]
    Subtract(Box<Expression>, #[adze::leaf(text = "-")] (), Box<Expression>),
    
    #[adze::prec_left(2)]
    Multiply(Box<Expression>, #[adze::leaf(text = "*")] (), Box<Expression>),
    
    #[adze::prec_left(2)]
    Divide(Box<Expression>, #[adze::leaf(text = "/")] (), Box<Expression>),
    
    // Right-associative exponentiation
    #[adze::prec_right(3)]
    Power(Box<Expression>, #[adze::leaf(text = "**")] (), Box<Expression>),
}
```

---

## Repetition and Options

### Optional Elements with `Option<T>`

Use `Option<T>` for optional elements:

```rust
#[adze::language]
pub struct ReturnStatement {
    #[adze::leaf(text = "return")]
    _return: (),
    pub value: Option<Expression>,
    #[adze::leaf(text = ";")]
    _semicolon: (),
}
```

This allows both `return;` and `return value;`.

### Repetition with `Vec<T>`

Use `Vec<T>` with `#[adze::repeat]` for sequences:

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

#### Non-empty Sequences

Use `non_empty = true` to require at least one element:

```rust
#[adze::language]
pub struct Program {
    #[adze::repeat(non_empty = true)]
    pub statements: Vec<Statement>,
}
```

### Delimited Lists with `#[adze::delimited]`

Use `#[adze::delimited]` for comma-separated lists:

```rust
#[adze::language]
pub struct ParameterList {
    #[adze::leaf(text = "(")]
    _open: (),
    #[adze::repeat]
    #[adze::delimited(#[adze::leaf(text = ",")] ())]
    pub params: Vec<Parameter>,
    #[adze::leaf(text = ")")]
    _close: (),
}
```

This parses: `()`, `(x)`, `(x, y)`, `(x, y, z)`, etc.

#### Combining Attributes

```rust
#[adze::language]
pub struct Arguments {
    #[adze::leaf(text = "(")]
    _open: (),
    #[adze::repeat]
    #[adze::delimited(#[adze::leaf(text = ",")] ())]
    pub args: Vec<Expression>,
    #[adze::leaf(text = ")")]
    _close: (),
}
```

### Whitespace Handling in Lists

Add optional whitespace patterns for better formatting support:

```rust
#[adze::language]
pub struct ListExpression {
    #[adze::leaf(text = "[")]
    _open: (),
    #[adze::leaf(pattern = r"\s*")]
    _ws1: (),
    #[adze::repeat]
    #[adze::delimited(#[adze::leaf(text = ",")] ())]
    pub elements: Vec<Expression>,
    #[adze::leaf(pattern = r"\s*")]
    _ws2: (),
    #[adze::leaf(text = "]")]
    _close: (),
}
```

---

## External Scanners

External scanners handle tokens that cannot be expressed with regular expressions.

### When to Use External Scanners

- **Indentation-sensitive languages** (Python, YAML)
- **Context-sensitive tokens** (JavaScript template literals)
- **Complex delimited strings** (heredocs, raw strings)
- **Nested comments** (`/* /* nested */ */`)
- **Custom whitespace rules**

### Implementation Pattern

1. **Define external tokens in the grammar:**

```rust
#[adze::grammar("python")]
pub mod grammar {
    #[adze::external]
    pub struct Newline;

    #[adze::external]
    pub struct Indent;

    #[adze::external]
    pub struct Dedent;
}
```

2. **Implement the `ExternalScanner` trait:**

```rust
use adze::external_scanner::{ExternalScanner, Lexer, ScanResult};

#[derive(Debug, Clone)]
pub struct PythonScanner {
    indent_stack: Vec<u16>,
}

impl PythonScanner {
    pub fn new() -> Self {
        PythonScanner {
            indent_stack: vec![0],
        }
    }
}

impl Default for PythonScanner {
    fn default() -> Self {
        Self::new()
    }
}

impl ExternalScanner for PythonScanner {
    fn scan(&mut self, lexer: &mut dyn Lexer, valid_symbols: &[bool]) -> Option<ScanResult> {
        // Check which tokens are valid in current state
        if valid_symbols[NEWLINE_INDEX] {
            // Handle newline detection
            if let Some(b'\n') = lexer.lookahead() {
                lexer.advance(1);
                lexer.mark_end();
                return Some(ScanResult {
                    symbol: TokenType::Newline as u16,
                    length: 1,
                });
            }
        }
        
        if valid_symbols[INDENT_INDEX] {
            // Handle indent detection
            // ...
        }
        
        None
    }
}
```

3. **Register the scanner:**

```rust
pub fn register_scanner() {
    adze::scanner_registry::register_rust_scanner::<PythonScanner>("python");
}
```

### Scanner Interface

```rust
pub trait ExternalScanner {
    fn scan(&mut self, lexer: &mut dyn Lexer, valid_symbols: &[bool]) -> Option<ScanResult>;
}

pub trait Lexer {
    fn lookahead(&self) -> Option<u8>;
    fn advance(&mut self, count: usize);
    fn mark_end(&mut self);
    fn column(&self) -> usize;
    fn is_eof(&self) -> bool;
}

pub struct ScanResult {
    pub symbol: u16,
    pub length: usize,
}
```

### Complete Example: Python Indentation

```rust
// scanner.rs
use adze::external_scanner::{ExternalScanner, Lexer, ScanResult};

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u16)]
pub enum TokenType {
    Newline = 203,
    Indent = 204,
    Dedent = 205,
}

const NEWLINE_INDEX: usize = 0;
const INDENT_INDEX: usize = 1;
const DEDENT_INDEX: usize = 2;

#[derive(Debug, Clone)]
pub struct PythonScanner {
    indent_stack: Vec<u16>,
}

impl PythonScanner {
    pub fn new() -> Self {
        PythonScanner {
            indent_stack: vec![0],
        }
    }
}

impl Default for PythonScanner {
    fn default() -> Self {
        Self::new()
    }
}

impl ExternalScanner for PythonScanner {
    fn scan(&mut self, lexer: &mut dyn Lexer, valid_symbols: &[bool]) -> Option<ScanResult> {
        // Skip leading whitespace and count indentation
        let mut indent = 0;
        while let Some(b' ') = lexer.lookahead() {
            lexer.advance(1);
            indent += 1;
        }
        while let Some(b'\t') = lexer.lookahead() {
            lexer.advance(1);
            indent += 4; // Treat tab as 4 spaces
        }
        
        // Check for newline
        if valid_symbols.len() > NEWLINE_INDEX && valid_symbols[NEWLINE_INDEX] {
            if let Some(b'\n') = lexer.lookahead() {
                lexer.advance(1);
                lexer.mark_end();
                return Some(ScanResult {
                    symbol: TokenType::Newline as u16,
                    length: 1,
                });
            }
        }
        
        // Check for indent
        if valid_symbols.len() > INDENT_INDEX && valid_symbols[INDENT_INDEX] {
            let current_indent = *self.indent_stack.last().unwrap();
            if indent > current_indent {
                self.indent_stack.push(indent);
                lexer.mark_end();
                return Some(ScanResult {
                    symbol: TokenType::Indent as u16,
                    length: 0,
                });
            }
        }
        
        // Check for dedent
        if valid_symbols.len() > DEDENT_INDEX && valid_symbols[DEDENT_INDEX] {
            let current_indent = *self.indent_stack.last().unwrap();
            if indent < current_indent && self.indent_stack.len() > 1 {
                self.indent_stack.pop();
                lexer.mark_end();
                return Some(ScanResult {
                    symbol: TokenType::Dedent as u16,
                    length: 0,
                });
            }
        }
        
        None
    }
}
```

---

## Common Patterns

### Expression Grammars

A complete expression grammar with all operator types:

```rust
#[adze::language]
pub enum Expression {
    // Literals
    Number(#[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())] i32),
    String(#[adze::leaf(pattern = r#""[^"]*""#)] String),
    Boolean(#[adze::leaf(pattern = r"true|false", transform = |v| v == "true")] bool),
    Identifier(#[adze::leaf(pattern = r"[a-zA-Z_]\w*")] String),
    
    // Grouping
    Paren(
        #[adze::leaf(text = "(")] (),
        Box<Expression>,
        #[adze::leaf(text = ")")] (),
    ),
    
    // Unary operators
    #[adze::prec(8)]
    Negate(#[adze::leaf(text = "-")] (), Box<Expression>),
    #[adze::prec(8)]
    Not(#[adze::leaf(text = "!")] (), Box<Expression>),
    
    // Multiplicative
    #[adze::prec_left(7)]
    Multiply(Box<Expression>, #[adze::leaf(text = "*")] (), Box<Expression>),
    #[adze::prec_left(7)]
    Divide(Box<Expression>, #[adze::leaf(text = "/")] (), Box<Expression>),
    #[adze::prec_left(7)]
    Modulo(Box<Expression>, #[adze::leaf(text = "%")] (), Box<Expression>),
    
    // Additive
    #[adze::prec_left(6)]
    Add(Box<Expression>, #[adze::leaf(text = "+")] (), Box<Expression>),
    #[adze::prec_left(6)]
    Subtract(Box<Expression>, #[adze::leaf(text = "-")] (), Box<Expression>),
    
    // Comparison
    #[adze::prec(5)]
    Equal(Box<Expression>, #[adze::leaf(text = "==")] (), Box<Expression>),
    #[adze::prec(5)]
    NotEqual(Box<Expression>, #[adze::leaf(text = "!=")] (), Box<Expression>),
    #[adze::prec(5)]
    Less(Box<Expression>, #[adze::leaf(text = "<")] (), Box<Expression>),
    #[adze::prec(5)]
    Greater(Box<Expression>, #[adze::leaf(text = ">")] (), Box<Expression>),
    
    // Logical
    #[adze::prec_left(3)]
    And(Box<Expression>, #[adze::leaf(text = "&&")] (), Box<Expression>),
    #[adze::prec_left(2)]
    Or(Box<Expression>, #[adze::leaf(text = "||")] (), Box<Expression>),
    
    // Assignment (right-associative)
    #[adze::prec_right(1)]
    Assign(Box<Expression>, #[adze::leaf(text = "=")] (), Box<Expression>),
}

#[adze::extra]
struct Whitespace {
    #[adze::leaf(pattern = r"\s+")]
    _ws: (),
}
```

### Statement Lists

```rust
#[adze::language]
pub struct Program {
    #[adze::repeat]
    pub statements: Vec<Statement>,
}

#[adze::language]
pub enum Statement {
    Expression(ExpressionStatement),
    Variable(VariableDeclaration),
    Function(FunctionDeclaration),
    Return(ReturnStatement),
    If(IfStatement),
    While(WhileStatement),
    Block(BlockStatement),
}

#[adze::language]
pub struct ExpressionStatement {
    pub expression: Expression,
    #[adze::leaf(text = ";")]
    _semicolon: (),
}

#[adze::language]
pub struct BlockStatement {
    #[adze::leaf(text = "{")]
    _open: (),
    #[adze::repeat]
    pub statements: Vec<Statement>,
    #[adze::leaf(text = "}")]
    _close: (),
}
```

### Block Structures

```rust
#[adze::language]
pub struct IfStatement {
    #[adze::leaf(text = "if")]
    _if: (),
    #[adze::leaf(text = "(")]
    _open: (),
    pub condition: Expression,
    #[adze::leaf(text = ")")]
    _close: (),
    pub then_block: BlockStatement,
    pub else_clause: Option<ElseClause>,
}

#[adze::language]
pub struct ElseClause {
    #[adze::leaf(text = "else")]
    _else: (),
    pub block: BlockStatement,
}

#[adze::language]
pub struct WhileStatement {
    #[adze::leaf(text = "while")]
    _while: (),
    #[adze::leaf(text = "(")]
    _open: (),
    pub condition: Expression,
    #[adze::leaf(text = ")")]
    _close: (),
    pub body: BlockStatement,
}
```

### Function Definitions

```rust
#[adze::language]
pub struct FunctionDeclaration {
    #[adze::leaf(text = "fn")]
    _fn: (),
    pub name: Identifier,
    pub parameters: ParameterList,
    #[adze::leaf(text = "->")]
    _arrow: (),
    pub return_type: Option<Type>,
    pub body: BlockStatement,
}

#[adze::language]
pub struct ParameterList {
    #[adze::leaf(text = "(")]
    _open: (),
    #[adze::repeat]
    #[adze::delimited(#[adze::leaf(text = ",")] ())]
    pub params: Vec<Parameter>,
    #[adze::leaf(text = ")")]
    _close: (),
}

#[adze::language]
pub struct Parameter {
    pub name: Identifier,
    #[adze::leaf(text = ":")]
    _colon: (),
    pub type_annotation: Type,
}
```

### Data Structures

```rust
#[adze::language]
pub struct ListExpression {
    #[adze::leaf(text = "[")]
    _open: (),
    #[adze::repeat]
    #[adze::delimited(#[adze::leaf(text = ",")] ())]
    pub elements: Vec<Expression>,
    #[adze::leaf(text = "]")]
    _close: (),
}

#[adze::language]
pub struct DictExpression {
    #[adze::leaf(text = "{")]
    _open: (),
    #[adze::repeat]
    #[adze::delimited(#[adze::leaf(text = ",")] ())]
    pub items: Vec<DictItem>,
    #[adze::leaf(text = "}")]
    _close: (),
}

#[adze::language]
pub struct DictItem {
    pub key: Expression,
    #[adze::leaf(text = ":")]
    _colon: (),
    pub value: Expression,
}
```

---

## Troubleshooting

### Common Errors

#### 1. "Ambiguous Parse" or GLR Forks

**Symptom**: Parser forks unexpectedly, multiple parse results.

**Cause**: Two rules match the same input with the same precedence.

**Fix**: Ensure all conflicting operators have distinct precedence levels:

```rust
// ❌ Bad: Same precedence causes ambiguity
#[adze::prec_left(1)]
Add(Box<Expression>, #[adze::leaf(text = "+")] (), Box<Expression>),
#[adze::prec_left(1)]
Subtract(Box<Expression>, #[adze::leaf(text = "-")] (), Box<Expression>),

// ✅ Good: Same precedence, same associativity is fine for same-level operators
#[adze::prec_left(1)]
Add(Box<Expression>, #[adze::leaf(text = "+")] (), Box<Expression>),
#[adze::prec_left(1)]
Subtract(Box<Expression>, #[adze::leaf(text = "-")] (), Box<Expression>),
```

#### 2. Infinite Recursion

**Symptom**: Parser generator fails or parser hangs.

**Cause**: Recursive rules without proper termination.

**Fix**: Ensure recursive calls are preceded by a terminal or have a base case:

```rust
// ❌ Bad: Direct left recursion without termination
#[adze::language]
pub enum Expression {
    Add(Expression, #[adze::leaf(text = "+")] (), Expression),
    Number(NumberLiteral),
}

// ✅ Good: Use Box and precedence attributes
#[adze::language]
pub enum Expression {
    #[adze::prec_left(1)]
    Add(Box<Expression>, #[adze::leaf(text = "+")] (), Box<Expression>),
    Number(NumberLiteral),
}
```

#### 3. Associativity Mismatch

**Symptom**: `1 - 2 - 3` parses as `1 - (2 - 3)` instead of `(1 - 2) - 3`.

**Cause**: Wrong associativity attribute.

**Fix**: Use `prec_left` for left-associative operators:

```rust
// ❌ Wrong: Right-associative subtraction
#[adze::prec_right(1)]
Subtract(Box<Expression>, #[adze::leaf(text = "-")] (), Box<Expression>),

// ✅ Correct: Left-associative subtraction
#[adze::prec_left(1)]
Subtract(Box<Expression>, #[adze::leaf(text = "-")] (), Box<Expression>),
```

#### 4. Empty Rule Issues

**Symptom**: Parse fails or produces unexpected results for empty input.

**Cause**: Improper handling of optional/repeated elements.

**Fix**: Use `non_empty = false` or handle empty cases explicitly:

```rust
// Allow empty modules
#[adze::language]
pub struct Module {
    #[adze::repeat(non_empty = false)]
    pub statements: Vec<Statement>,
}
```

#### 5. Token Not Matching

**Symptom**: Valid input fails to parse.

**Cause**: Regex pattern doesn't match the input.

**Fix**: Test patterns independently and escape special characters:

```rust
// ❌ Wrong: Unescaped special characters
#[adze::leaf(pattern = r"+")]
_plus: (),

// ✅ Correct: Escaped special characters
#[adze::leaf(pattern = r"\+")]
_plus: (),

// Or use text for exact matches
#[adze::leaf(text = "+")]
_plus: (),
```

### Debug Techniques

#### 1. Enable Artifact Emission

```bash
export ADZE_EMIT_ARTIFACTS=true
cargo build
```

Check the generated `adze_debug_*.log` files in your temp directory.

#### 2. Print Parse Trees

```rust
#[test]
fn debug_parse() {
    let input = "1 + 2 * 3";
    let result = parse(input);
    
    if let Ok(tree) = &result {
        print_tree(tree, input.as_bytes(), 0);
    } else {
        eprintln!("Parse failed: {:?}", result);
    }
}

fn print_tree(node: &adze::pure_parser::ParsedNode, source: &[u8], indent: usize) {
    let text = std::str::from_utf8(&source[node.start_byte..node.end_byte]).unwrap_or("<invalid>");
    eprintln!(
        "{:indent$}{}: '{}'",
        "",
        node.kind(),
        text,
        indent = indent
    );
    for child in &node.children {
        print_tree(child, source, indent + 2);
    }
}
```

#### 3. Check Symbol Names

```rust
#[test]
fn debug_symbols() {
    let lang = language();
    unsafe {
        let symbol_count = lang.symbol_count;
        eprintln!("Total symbols: {}", symbol_count);
        
        let symbol_names = std::slice::from_raw_parts(lang.symbol_names, symbol_count as usize);
        for (i, &name_ptr) in symbol_names.iter().enumerate() {
            if !name_ptr.is_null() {
                let name = std::ffi::CStr::from_ptr(name_ptr as *const i8);
                eprintln!("  Symbol {}: '{}'", i, name.to_string_lossy());
            }
        }
    }
}
```

#### 4. Use Snapshot Testing

```rust
use insta::assert_debug_snapshot;

#[test]
fn test_expression_parsing() {
    let tree = parse("1 + 2 * 3").unwrap();
    assert_debug_snapshot!(tree);
}
```

Review changes with:

```bash
cargo insta review
```

#### 5. Test Individual Tokens

```rust
#[test]
fn test_identifier_pattern() {
    let pattern = regex::Regex::new(r"[a-zA-Z_][a-zA-Z0-9_]*").unwrap();
    assert!(pattern.is_match("valid_name"));
    assert!(pattern.is_match("_private"));
    assert!(!pattern.is_match("123invalid"));
}
```

### Getting Help

1. Check the [FAQ](../../FAQ.md) for common questions
2. Review existing grammar implementations in `grammars/` directory
3. Enable debug logging with `ADZE_EMIT_ARTIFACTS=true`
4. Search existing issues on GitHub
5. Ask in the community chat or forums

---

## Appendix: Attribute Reference

| Attribute | Applies To | Description |
|-----------|------------|-------------|
| `#[adze::grammar("name")]` | Module | Defines a grammar with the given name |
| `#[adze::language]` | Struct, Enum, Variant | Marks a type as a grammar rule |
| `#[adze::leaf(text = "...")]` | Field, Variant | Defines a literal token |
| `#[adze::leaf(pattern = r"...")]` | Field, Variant | Defines a regex token |
| `#[adze::leaf(transform = \|v\| ...)]` | Field | Transforms matched text |
| `#[adze::external]` | Struct | Marks as external token |
| `#[adze::extra]` | Struct | Marks as skipped token |
| `#[adze::repeat]` | Field (Vec) | Marks as repeated |
| `#[adze::repeat(non_empty = true)]` | Field (Vec) | Requires at least one |
| `#[adze::delimited(...)]` | Field (Vec) | Specifies delimiter |
| `#[adze::prec(n)]` | Variant | Non-associative precedence |
| `#[adze::prec_left(n)]` | Variant | Left-associative precedence |
| `#[adze::prec_right(n)]` | Variant | Right-associative precedence |
