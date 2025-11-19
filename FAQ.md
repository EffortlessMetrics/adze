# Frequently Asked Questions

Common questions about rust-sitter, answered concisely.

**Can't find your question?** Check [TROUBLESHOOTING.md](./docs/TROUBLESHOOTING.md) or ask in [GitHub Discussions](https://github.com/hydro-project/rust-sitter/discussions).

---

## General Questions

### What is rust-sitter?

rust-sitter is a parser generator for Rust that lets you define grammars using Rust types and attributes. It generates efficient parsers at compile time.

**Think**: "Tree-sitter meets Rust macros"

### Why use rust-sitter instead of tree-sitter?

| Feature | rust-sitter | tree-sitter |
|---------|-------------|-------------|
| **Language** | Pure Rust | C with bindings |
| **Grammar Definition** | Rust types + attributes | JavaScript DSL |
| **Type Safety** | Compile-time typed AST | Runtime navigation |
| **WASM Support** | ✅ First-class | ⚠️ Requires binding work |
| **GLR Parsing** | ✅ Built-in | ❌ LR only |
| **Incremental Parsing** | 🚧 In progress (v0.7.0) | ✅ Mature |
| **Editor Integration** | 🚧 Coming | ✅ Extensive |

**Use rust-sitter if**: You want type-safe parsing in pure Rust, need WASM support, or want to handle ambiguous grammars.

**Use tree-sitter if**: You need mature editor integration or battle-tested incremental parsing today.

### Is rust-sitter production-ready?

**v0.6.1-beta**: ✅ **Macro-based grammar generation is production-ready**
- Core parsing: 100% functional
- GLR parsing: Fully operational
- Type-safe ASTs: Working
- WASM support: Ready

**Not yet ready**:
- Incremental parsing (coming v0.7.0)
- Full query system (coming v0.7.0)
- Editor plugins (future)

**Recommendation**: Great for new projects, CLI tools, and WASM apps. Wait for v0.7.0 for editor integration.

### What does GLR mean? Do I need it?

**GLR** = Generalized LR parsing. It handles ambiguous grammars by exploring multiple parse paths simultaneously.

**You need it if**:
- Your grammar has inherent ambiguity (e.g., C++ templates)
- You want the parser to find all possible interpretations
- You're parsing natural language or DSLs with flexibility

**You don't need it if**:
- Your grammar is unambiguous (most programming languages)
- Standard LR parsing works fine

rust-sitter handles both automatically - you don't need to choose.

---

## Getting Started

### How do I install rust-sitter?

Add to your `Cargo.toml`:

```toml
[dependencies]
rust-sitter = "0.6"

[build-dependencies]
rust-sitter-tool = "0.6"
```

See [QUICK_START.md](./QUICK_START.md) for a 5-minute tutorial.

### Where are the examples?

- **Quick Start**: [QUICK_START.md](./QUICK_START.md) - 5-minute calculator
- **Full Examples**: [example/src/](./example/src/) - arithmetic, JSON, more
- **Tutorial**: [docs/GETTING_STARTED.md](./docs/GETTING_STARTED.md)

### What's the smallest example?

```rust
#[rust_sitter::grammar("tiny")]
mod grammar {
    #[rust_sitter::language]
    pub struct Number {
        #[rust_sitter::leaf(pattern = r"\d+")]
        value: String,
    }
}

fn main() {
    let result = grammar::parse("42");
    println!("{:?}", result);
}
```

Don't forget `build.rs`:
```rust
fn main() {
    rust_sitter_tool::build_parsers(&std::path::PathBuf::from("src/main.rs"));
}
```

---

## Grammar Definition

### How do I handle whitespace?

Mark it as `#[rust_sitter::extra]`:

```rust
#[rust_sitter::extra]
struct Whitespace {
    #[rust_sitter::leaf(pattern = r"\s")]
    _ws: (),
}
```

This tells the parser to skip whitespace anywhere.

### How do I set operator precedence?

Use `#[rust_sitter::prec_left(N)]` where higher N = tighter binding:

```rust
#[rust_sitter::prec_left(1)]  // Lowest precedence
Add(Box<Expr>, #[leaf(text = "+")] (), Box<Expr>),

#[rust_sitter::prec_left(2)]  // Higher precedence
Mul(Box<Expr>, #[leaf(text = "*")] (), Box<Expr>),
```

This makes `2 + 3 * 4` parse as `2 + (3 * 4)`.

### How do I handle optional elements?

Use `Option<T>`:

```rust
pub struct FunctionCall {
    name: String,
    args: Option<Arguments>,  // Optional arguments
}
```

### How do I handle repetition (lists)?

Use `Vec<T>`:

```rust
pub struct ArgumentList {
    #[rust_sitter::repeat]
    #[rust_sitter::delimited(#[leaf(text = ",")] ())]
    args: Vec<Expression>,
}
```

### Can I transform matched text?

Yes! Use the `transform` parameter:

```rust
Number(
    #[rust_sitter::leaf(
        pattern = r"\d+",
        transform = |s| s.parse::<i32>().unwrap()
    )]
    i32
)
```

### How do I handle comments?

Mark them as `#[rust_sitter::extra]`:

```rust
#[rust_sitter::extra]
struct Comment {
    #[rust_sitter::leaf(pattern = r"//[^\n]*")]
    _comment: (),
}
```

---

## Build and Compilation

### Why does my build take so long?

The first build generates the parser, which can take time. Subsequent builds are fast (parser is cached).

**Tips**:
- Use `cargo build --release` only when needed
- The parser generation happens in `build.rs`, not at runtime

### Where are the generated files?

Set `RUST_SITTER_EMIT_ARTIFACTS=true` to see generated files:

```bash
RUST_SITTER_EMIT_ARTIFACTS=true cargo build
ls target/debug/build/*/out/
```

You'll see:
- `grammar.json` - Tree-sitter grammar
- `parser.c` - Generated parser (if using C backend)
- Parse tables (if using pure-Rust backend)

### How do I debug grammar issues?

1. **Enable artifact emission**:
   ```bash
   RUST_SITTER_EMIT_ARTIFACTS=true cargo build
   ```

2. **Check generated grammar**:
   ```bash
   cat target/debug/build/*/out/grammar.json
   ```

3. **Look for conflicts**: Build output shows shift/reduce conflicts

4. **Use tests**: Write tests for grammar rules individually

### Build fails with "conflict detected"

Your grammar has ambiguity. Solutions:

1. **Add precedence**: Use `#[prec_left]` or `#[prec_right]`
2. **Restructure grammar**: Make it unambiguous
3. **Use GLR**: rust-sitter handles conflicts automatically via GLR

### How do I see what the parser is doing?

Enable logging:

```bash
RUST_LOG=debug cargo run
```

Or use the parse tree visitor (coming in v0.7.0).

---

## Features and Capabilities

### Does rust-sitter support error recovery?

✅ Yes! The parser handles malformed input gracefully:

```rust
match grammar::parse("1 + ") {  // Incomplete expression
    Ok(tree) => println!("Partial parse: {:?}", tree),
    Err(e) => println!("Error: {}", e),
}
```

Error nodes are inserted for missing/unexpected tokens.

### Can I use rust-sitter with WASM?

✅ Yes! Pure-Rust backend has first-class WASM support:

```toml
[dependencies]
rust-sitter = { version = "0.6", features = ["pure-rust"] }
```

Then compile to WASM:
```bash
cargo build --target wasm32-unknown-unknown
```

### Does rust-sitter support external scanners?

✅ Yes! For context-sensitive lexing (like Python indentation):

```rust
#[derive(Default)]
struct IndentScanner;

impl rust_sitter::ExternalScanner for IndentScanner {
    fn scan(&mut self, lexer: &mut Lexer, valid: &[bool]) -> ScanResult {
        // Custom scanning logic
    }
}
```

See [docs/](./docs/) for examples (Python grammar uses this).

### What about incremental parsing?

🚧 **Coming in v0.7.0** (March 2026)

Will support:
- `parse_with_old_tree()` for efficient re-parsing
- 10x+ speedup on small edits
- Full LSP integration

See [ROADMAP.md](./ROADMAP.md) for timeline.

### Can I query the parse tree?

🚧 **Partial support in v0.6.1, complete in v0.7.0**

Basic tree navigation works now. Full query system (predicates, captures) coming in v0.7.0.

---

## Performance

### How fast is rust-sitter?

**Status**: Baseline being established in v0.7.0 Week 1

Expected performance:
- Comparable to tree-sitter-c for most grammars
- Pure-Rust overhead is minimal
- GLR has overhead only when grammar is ambiguous

See [docs/PERFORMANCE_BASELINE.md](./docs/PERFORMANCE_BASELINE.md) for upcoming benchmarks.

### How can I make my parser faster?

**Grammar optimization**:
- Minimize backtracking (use clear precedence)
- Keep regexes simple
- Avoid deep nesting in repetitions

**Build optimization**:
- Use `--release` for production
- Profile with `cargo flamegraph`

**Future** (v0.7.0+):
- Incremental parsing for editors
- Table compression for memory

### Does rust-sitter support parallel parsing?

Not currently. Parsing is single-threaded. You can parse multiple files in parallel though:

```rust
use rayon::prelude::*;

files.par_iter().map(|file| {
    grammar::parse(file)
}).collect()
```

---

## Contributing

### How can I contribute?

See [CONTRIBUTING.md](./CONTRIBUTING.md) and [GAPS.md](./GAPS.md).

**Quick picks**:
- Re-enable ignored tests (1-4 hours each)
- Add documentation examples
- Write grammar cookbooks
- Performance benchmarking

### I found a bug, what should I do?

1. **Search existing issues**: Check if it's already reported
2. **Create minimal reproduction**: Simplest grammar that shows the bug
3. **Open issue**: [GitHub Issues](https://github.com/hydro-project/rust-sitter/issues)
4. **Include**: rust-sitter version, Rust version, minimal example

### Can I add a new grammar to the examples?

Yes! PRs welcome:

1. Add grammar to `example/src/your_grammar.rs`
2. Add tests with expected parse trees
3. Update `example/tests/integration.rs`
4. Submit PR

See [CONTRIBUTING.md](./CONTRIBUTING.md) for guidelines.

---

## Comparison to Alternatives

### rust-sitter vs nom

| Feature | rust-sitter | nom |
|---------|-------------|-----|
| **Approach** | Declarative grammar | Combinator library |
| **Generated Code** | Yes (build time) | No (runtime combinators) |
| **Error Recovery** | Automatic | Manual |
| **Performance** | Optimized tables | Depends on combinators |
| **Learning Curve** | Moderate (grammars) | Moderate (combinators) |

**Use nom if**: You need fine control, hand-written parsers, or no build step.
**Use rust-sitter if**: You have a grammar, want error recovery, or need GLR.

### rust-sitter vs pest

| Feature | rust-sitter | pest |
|---------|-------------|-----|
| **Grammar Format** | Rust attributes | PEG syntax |
| **Type Safety** | Compile-time AST | Runtime rule matching |
| **Ambiguity** | GLR handles it | PEG doesn't support |
| **Precedence** | Built-in | Manual |

**Use pest if**: You like PEG grammars or want external grammar files.
**Use rust-sitter if**: You want type-safe ASTs or need ambiguity support.

### rust-sitter vs lalrpop

| Feature | rust-sitter | lalrpop |
|---------|-------------|---------|
| **Grammar Format** | Rust attributes | LALR DSL |
| **GLR Support** | Yes | No |
| **Maturity** | New (beta) | Mature |
| **WASM** | First-class | Requires work |

**Use lalrpop if**: You need battle-tested LALR parsing.
**Use rust-sitter if**: You need GLR, WASM, or prefer Rust-native grammars.

---

## Roadmap and Future

### When will v0.7.0 be released?

**Target**: March 2026 (Q1)

See [IMPLEMENTATION_PLAN.md](./IMPLEMENTATION_PLAN.md) for week-by-week schedule.

### What's coming in v0.7.0?

- ✅ Incremental parsing (`parse_with_old_tree`)
- ✅ Complete query system with predicates
- ✅ Performance baseline and optimization
- ✅ CLI functionality (dynamic loading, corpus testing)
- ✅ Comprehensive documentation and video tutorials

### What about v1.0?

**Target**: Q4 2026

See [ROADMAP.md](./ROADMAP.md) for full vision.

**v1.0 goals**:
- API stability guarantees
- Full editor plugin support
- Production-grade incremental parsing
- Comprehensive language support (50+ grammars)

### Will rust-sitter replace tree-sitter?

No. They serve different use cases:

- **tree-sitter**: Mature, editor-focused, C-based, extensive ecosystem
- **rust-sitter**: Pure-Rust, type-safe, GLR, WASM-first

Think of rust-sitter as "tree-sitter for Rust-native projects" not a replacement.

### Can I use both tree-sitter and rust-sitter?

Yes! rust-sitter can:
- Import tree-sitter grammars (via ts-bridge tool)
- Generate tree-sitter compatible parsers
- Interop with tree-sitter tooling

See [tools/ts-bridge/](./tools/ts-bridge/) for the bridge tool.

---

## Still Have Questions?

- **Check**: [TROUBLESHOOTING.md](./docs/TROUBLESHOOTING.md) (coming v0.7.0)
- **Ask**: [GitHub Discussions](https://github.com/hydro-project/rust-sitter/discussions)
- **Report Bugs**: [GitHub Issues](https://github.com/hydro-project/rust-sitter/issues)
- **Tutorial**: [docs/GETTING_STARTED.md](./docs/GETTING_STARTED.md)
- **Examples**: [example/src/](./example/src/)

**Can't find an answer?** Open a discussion - we'll add it to this FAQ!
