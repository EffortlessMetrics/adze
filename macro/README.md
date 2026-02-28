# adze-macro

Procedural macros for the [Adze](https://github.com/EffortlessMetrics/adze) parser toolchain.

## Overview

`adze-macro` provides the attribute macros that let you define grammars using Rust types. It is the user-facing entry point for grammar definitions.

## Attributes

| Attribute | Description |
|-----------|-------------|
| `#[adze::grammar]` | Mark a module as containing a grammar definition |
| `#[adze::language]` | Define the top-level language entry point |
| `#[adze::leaf]` | Mark a type as a terminal/leaf node |
| `#[adze::prec_left]` | Left-associative precedence |
| `#[adze::prec_right]` | Right-associative precedence |
| `#[adze::extra]` | Mark tokens as extras (whitespace, comments) |
| `#[adze::word]` | Define the word token for keyword extraction |

## Usage

```rust
#[adze::grammar]
mod arithmetic {
    #[adze::language]
    pub enum Expression {
        Number(Number),
        #[adze::prec_left(1)]
        Add(Box<Expression>, #[adze::leaf(text = "+")] (), Box<Expression>),
    }

    #[adze::leaf(pattern = r"\d+")]
    pub struct Number(String);
}
```

## Features

| Feature | Description |
|---------|-------------|
| `pure-rust` | Use pure-Rust parsing backend |
| `strict_docs` | Enforce documentation requirements |

## License

Licensed under either of [Apache License, Version 2.0](../LICENSE-APACHE) or [MIT License](../LICENSE-MIT) at your option.
