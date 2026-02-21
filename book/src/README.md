# Adze Documentation

[![Crates.io](https://img.shields.io/crates/v/adze)](https://crates.io/crates/adze)

Welcome to the official documentation for **Adze** - a Rust framework that makes it easy to create efficient parsers by leveraging the [Tree-sitter](https://tree-sitter.github.io/tree-sitter/) parser generator.

With Adze, you can define your entire grammar with annotations on idiomatic Rust code, and let macros generate the parser and type-safe bindings for you!

## Key Features

### 🚀 v0.5.0-beta Highlights

- **GLR Parsing**: Full support for ambiguous grammars with efficient fork/merge handling
- **Pure-Rust Option**: Generate static parsers at compile-time without C dependencies  
- **Enhanced Error Recovery**: Sophisticated error recovery strategies for robust parsing
- **Two-Phase Parser**: Proper reduction-shift separation for correct GLR semantics
- **Comprehensive Testing**: Golden tests, benchmarks, and validation infrastructure
- **WASM Support**: Full WebAssembly compatibility with the pure-Rust backend
- **Performance Optimizations**: SIMD lexing, parallel parsing, and memory pooling

## Quick Example

Here's a simple arithmetic expression parser:

```rust
#[adze::grammar("arithmetic")]
mod grammar {
    #[adze::language]
    pub enum Expr {
        Number(
            #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
            u32,
        ),
        #[adze::prec_left(1)]
        Add(
            Box<Expr>,
            #[adze::leaf(text = "+")] (),
            Box<Expr>,
        )
    }
}

// Usage
let result = grammar::parse("1+2+3");
```

## When to Use Adze

Adze is ideal for:

- **Language Server Protocol (LSP) implementations** - Fast incremental parsing for IDE support
- **Code analysis tools** - Syntax highlighting, linting, formatting
- **Transpilers and interpreters** - Type-safe AST generation
- **Documentation generators** - Parsing code for documentation extraction
- **Any application requiring robust parsing** - With error recovery and ambiguity handling

## How This Book is Organized

- **Getting Started** - Installation, quick start guide, and migration from Tree-sitter
- **User Guide** - Core concepts like grammar definition, parser generation, and queries
- **Advanced Topics** - GLR parsing, optimization, external scanners, and more
- **Reference** - API documentation, examples, and known limitations
- **Development** - Contributing guidelines, architecture overview, and testing

## Getting Help

- **GitHub Issues**: Report bugs or request features at [adze/issues](https://github.com/EffortlessMetrics/adze/issues)
- **Discussions**: Ask questions and share experiences in [GitHub Discussions](https://github.com/EffortlessMetrics/adze/discussions)
- **Examples**: Check out the [example grammars](reference/grammar-examples.md) for inspiration

## License

Adze is licensed under the MIT license. See the [LICENSE](https://github.com/EffortlessMetrics/adze/blob/main/LICENSE) file for details.