# GLR Parser User Guide

**Version**: 1.0.0
**Date**: 2025-11-20
**Type**: How-To Guide (Diataxis)
**Audience**: rust-sitter users implementing GLR parsers

---

## Table of Contents

1. [Introduction](#introduction)
2. [When to Use GLR](#when-to-use-glr)
3. [Enabling GLR Mode](#enabling-glr-mode)
4. [Writing GLR Grammars](#writing-glr-grammars)
5. [Debugging Conflicts](#debugging-conflicts)
6. [Performance Tuning](#performance-tuning)
7. [Migration from LR](#migration-from-lr)
8. [Troubleshooting](#troubleshooting)

---

## Introduction

This guide helps you use rust-sitter's GLR (Generalized LR) parser to handle ambiguous grammars and parsing conflicts. You'll learn how to:

- ✅ Decide when GLR is appropriate for your language
- ✅ Enable and configure GLR mode
- ✅ Write effective GLR grammars
- ✅ Debug and resolve parsing conflicts
- ✅ Optimize GLR parser performance

**Prerequisites:**
- Familiarity with rust-sitter basics
- Understanding of grammar syntax
- Basic knowledge of LR parsing concepts

**Time to Complete:** 30-45 minutes

---

## When to Use GLR

### Decision Flowchart

```
┌─────────────────────────────────────────┐
│ Does your grammar have ambiguities?     │
└─────────────┬───────────────────────────┘
              │
              ├─ NO ──▶ Use LR Parser (runtime/)
              │         ✓ Faster, simpler
              │
              └─ YES ─▶ Continue...
                        │
                        ▼
┌─────────────────────────────────────────┐
│ Can ambiguities be resolved with        │
│ precedence/associativity annotations?   │
└─────────────┬───────────────────────────┘
              │
              ├─ YES ──▶ Try LR first
              │          (May work with annotations)
              │
              └─ NO ──▶ Use GLR Parser (runtime2/)
                        ✓ Handles all ambiguities
```

### Use GLR When...

✅ **Your language has inherent ambiguities**
- Dangling-else problem
- Context-dependent syntax (C++, Rust)
- Multiple valid interpretations

✅ **Grammar readability is important**
- Don't want complex rewrites
- Prefer natural grammar structure

✅ **You need robust error recovery**
- Multiple parse paths improve recovery
- Better error messages

✅ **You're experimenting with language design**
- GLR enables rapid prototyping
- Easy to test ambiguous constructs

### Avoid GLR When...

❌ **Performance is critical**
- Hard real-time systems
- Extremely high-throughput parsers
- Limited memory environments

❌ **Grammar is provably unambiguous**
- Standard LR is sufficient and faster
- No need for GLR overhead

❌ **Debugging resources are limited**
- GLR conflicts can be complex to debug
- May need visualization tools

---

## Enabling GLR Mode

### Step 1: Add Dependencies

**Cargo.toml:**
```toml
[dependencies]
rust-sitter-runtime = { version = "0.1", features = ["pure-rust-glr", "serialization"] }
rust-sitter-ir = "0.8"
rust-sitter-glr-core = "0.8"
rust-sitter-tablegen = "0.8"

[build-dependencies]
rust-sitter-tool = "0.8"
```

### Step 2: Configure Build Script

**build.rs:**
```rust
use rust_sitter_tool::build_parsers;

fn main() {
    // Enable GLR mode
    std::env::set_var("RUST_SITTER_GLR_MODE", "true");

    // Build parsers with GLR support
    build_parsers(&["src/grammar.rs"]).expect("Parser generation failed");

    println!("cargo:rerun-if-changed=src/grammar.rs");
}
```

### Step 3: Use GLR Parser

**src/main.rs:**
```rust
use rust_sitter_runtime::{Parser, Language};
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load grammar with parse tables
    let language = Language::from_parsetable_file("grammar.parsetable")?;

    // Create GLR parser
    let mut parser = Parser::new();
    parser.set_language(language)?;

    // Parse input
    let source = fs::read("input.txt")?;
    let tree = parser.parse(&source, None)?;

    // Use tree (100% Tree-sitter API compatible)
    let root = tree.root_node();
    println!("Parsed: {}", root.kind());

    Ok(())
}
```

### Verification

Run this to verify GLR mode is active:

```bash
# Build your parser
cargo build

# Check for .parsetable files (GLR artifact)
find target -name "*.parsetable"

# Should output: target/debug/build/.../grammar.parsetable
```

---

## Writing GLR Grammars

### Example: Dangling-Else Grammar

**src/grammar.rs:**
```rust
use rust_sitter::grammar;

#[grammar("dangling_else")]
mod grammar {
    #[derive(Debug, Clone, PartialEq)]
    pub enum Stmt {
        /// if expr then stmt
        IfThen {
            #[rust_sitter(keyword = "if")]
            _if: (),
            condition: Box<Expr>,
            #[rust_sitter(keyword = "then")]
            _then: (),
            body: Box<Stmt>,
        },

        /// if expr then stmt else stmt
        IfThenElse {
            #[rust_sitter(keyword = "if")]
            _if: (),
            condition: Box<Expr>,
            #[rust_sitter(keyword = "then")]
            _then: (),
            then_body: Box<Stmt>,
            #[rust_sitter(keyword = "else")]
            _else: (),
            else_body: Box<Stmt>,
        },

        /// Simple statement
        #[rust_sitter(leaf)]
        Simple(String),
    }

    #[derive(Debug, Clone, PartialEq)]
    #[rust_sitter(leaf)]
    pub struct Expr(String);
}
```

### Best Practices

#### 1. Document Ambiguities

```rust
/// Dangling-else ambiguity:
///
/// Input: "if a then if b then x else y"
///
/// Interpretation 1 (else binds to inner if):
///   if a then (if b then x else y)
///
/// Interpretation 2 (else binds to outer if):
///   if a then (if b then x) else y
///
/// GLR explores both, selects first (matches most languages).
#[derive(Debug)]
pub enum Stmt { ... }
```

#### 2. Use Precedence Wisely

```rust
#[grammar("expr")]
mod grammar {
    #[derive(Debug)]
    pub enum Expr {
        /// Addition (lower precedence)
        #[rust_sitter(prec_left = 1)]
        Add {
            left: Box<Expr>,
            #[rust_sitter(token = "+")]
            _op: (),
            right: Box<Expr>,
        },

        /// Multiplication (higher precedence)
        #[rust_sitter(prec_left = 2)]
        Mul {
            left: Box<Expr>,
            #[rust_sitter(token = "*")]
            _op: (),
            right: Box<Expr>,
        },

        /// Number literal
        #[rust_sitter(leaf, regex = r"\d+")]
        Number(String),
    }
}
```

**Key Points:**
- Higher `prec_left` value = higher precedence
- Use `prec_left` for left-associative operators (+, -, *, /)
- Use `prec_right` for right-associative operators (^, =)
- GLR will **order** actions by precedence, not eliminate them

#### 3. Keep Grammars Simple

❌ **Don't:**
```rust
// Overly complex, hard to debug
pub enum Expr {
    #[rust_sitter(prec_left = 1, assoc = "left", dynamic = true)]
    ComplexOp { ... }
}
```

✅ **Do:**
```rust
// Clear, simple, debuggable
pub enum Expr {
    #[rust_sitter(prec_left = 1)]
    Add { left: Box<Expr>, right: Box<Expr> },
}
```

---

## Debugging Conflicts

### Step 1: Enable Debug Output

**Build with debug info:**
```bash
export RUST_LOG=rust_sitter=debug
cargo build 2>&1 | tee build.log
```

**Look for conflict reports:**
```
DEBUG: State 4 has shift/reduce conflict:
  Shift: 'else' -> State 6
  Reduce: Production 0 (IfThen)
  -> Created multi-action cell: [Shift(6), Reduce(0)]
```

### Step 2: Visualize Parse Table

**Generate parse table dump:**
```bash
export RUST_SITTER_EMIT_ARTIFACTS=true
cargo build

# Find generated files
ls target/debug/build/*/out/*.parsetable
ls target/debug/build/*/out/parse_table.txt  # Human-readable dump
```

**Examine parse_table.txt:**
```
State 4:
  'if'   -> Shift(1)
  'else' -> [Shift(6), Reduce(0)]  ← Conflict!
  'stmt' -> Shift(2)
  EOF    -> Reduce(0)
```

### Step 3: Understand Conflict Type

**Shift/Reduce Conflict:**
- **Shift**: Consume token and move to new state
- **Reduce**: Complete a production and go back

**Example:**
```
Input: "if a then if b then x else y"
                             ^
At 'else': Should we:
  1. Shift 'else' (continue outer if-then-else)
  2. Reduce to IfThen (complete inner if-then)
```

**Reduce/Reduce Conflict:**
- Multiple productions can be reduced
- Rarer than shift/reduce

### Step 4: Test Parsing Behavior

**Test file:**
```rust
#[test]
fn test_dangling_else_fork() {
    let input = b"if a then if b then x else y";
    let tree = parse_with_glr(input);

    // Examine resulting tree structure
    let root = tree.root_node();
    assert_eq!(root.kind(), "Stmt");

    // GLR should resolve to: if a then (if b then x else y)
    let if_stmt = root.child(0).unwrap();
    assert!(if_stmt.utf8_text(input).contains("else"));
}
```

### Step 5: Add Performance Logging

```bash
export RUST_SITTER_LOG_PERFORMANCE=true
cargo test test_dangling_else_fork

# Output:
# [PERF] GLR parse:
#   Input: 29 bytes
#   Forks: 2
#   Merges: 1
#   Nodes: 15
#   Time: 120 µs
```

---

## Performance Tuning

### Baseline Performance

From our benchmarks (see `docs/PERFORMANCE_BASELINE.md`):

| Grammar | Input Size | LR Time | GLR Time | Overhead |
|---------|-----------|---------|----------|----------|
| Arithmetic | 100 tokens | 80 µs | 120 µs | 1.5x |
| Dangling-Else | 10 stmts | N/A | 150 µs | - |
| Python | 1000 lines | 5 ms | 8 ms | 1.6x |

**Rule of Thumb**: GLR adds 1.5-2x overhead for ambiguous grammars, approaches LR speed for unambiguous parts.

### Optimization Strategies

#### 1. Reduce Conflicts

**Before:**
```rust
// Many conflicts due to ambiguous operators
pub enum Expr {
    BinOp { left: Box<Expr>, op: Op, right: Box<Expr> },
}
```

**After:**
```rust
// Separate variants with precedence
pub enum Expr {
    #[rust_sitter(prec_left = 1)]
    Add { left: Box<Expr>, right: Box<Expr> },

    #[rust_sitter(prec_left = 2)]
    Mul { left: Box<Expr>, right: Box<Expr> },
}
```

**Result**: Fewer forks → better performance

#### 2. Use Efficient Tokenization

**Tokenizer config:**
```rust
// Skip whitespace efficiently
#[rust_sitter(whitespace = r"\s+", skip = true)]
pub struct Grammar { ... }
```

#### 3. Monitor Fork Count

**Test harness:**
```rust
#[test]
fn benchmark_fork_count() {
    let inputs = load_test_corpus();

    for input in inputs {
        let stats = parse_with_stats(input);

        // Alert if forks are excessive
        assert!(
            stats.forks < 100,
            "Input {:?} caused {} forks (limit: 100)",
            input, stats.forks
        );
    }
}
```

#### 4. Profile Hot Paths

```bash
# Use perf for profiling
cargo build --release
perf record --call-graph=dwarf ./target/release/my_parser input.txt
perf report

# Look for:
# - Engine::fork_and_execute
# - Builder::build_node
# - Tokenizer::next_token
```

#### 5. Enable Release Optimizations

**Cargo.toml:**
```toml
[profile.release]
opt-level = 3
lto = true
codegen-units = 1
```

---

## Migration from LR

### Step-by-Step Migration

#### 1. Backup Current Parser

```bash
git checkout -b migrate-to-glr
cp -r src/grammar.rs src/grammar.rs.lr-backup
```

#### 2. Update Dependencies

**Cargo.toml changes:**
```diff
 [dependencies]
-rust-sitter = "0.6"
+rust-sitter-runtime = { version = "0.1", features = ["pure-rust-glr", "serialization"] }
+rust-sitter-ir = "0.8"
+rust-sitter-glr-core = "0.8"
+rust-sitter-tablegen = "0.8"
```

#### 3. Update Grammar (if needed)

Most grammars work as-is! GLR is a superset of LR.

**Optional enhancements:**
```rust
// Add precedence annotations for clarity
#[rust_sitter(prec_left = 2)]
Mul { ... }
```

#### 4. Update Parser Code

**Before (LR):**
```rust
use rust_sitter::{Parser, Language};

let mut parser = Parser::new();
parser.set_language(my_language()).unwrap();
let tree = parser.parse(source, None).unwrap();
```

**After (GLR):**
```rust
use rust_sitter_runtime::{Parser, Language};

let language = Language::from_parsetable_file("my_grammar.parsetable")?;
let mut parser = Parser::new();
parser.set_language(language)?;
let tree = parser.parse(source, None)?;
```

**Key Differences:**
- `Language::from_parsetable_file()` instead of generated function
- Error handling may differ slightly
- Tree API is 100% compatible ✅

#### 5. Run Tests

```bash
# Run existing test suite
cargo test

# Should see:
# test result: ok. X passed; 0 failed
```

If tests fail:
- Check for parse table file path issues
- Verify feature flags are set
- Review error messages (usually helpful)

#### 6. Validate Performance

```bash
# Benchmark before and after
cargo bench

# Expect:
# LR:  X ns/iter
# GLR: Y ns/iter  (Y ≈ 1.5X for ambiguous grammars)
```

---

## Troubleshooting

### Problem: "Parse table file not found"

**Error:**
```
Error: Parse table file 'grammar.parsetable' not found
```

**Solution:**
```bash
# Ensure build script runs
cargo clean
cargo build

# Check artifacts
find target -name "*.parsetable"

# If missing, verify build.rs is correct
cat build.rs
```

### Problem: "Too many forks, performance degraded"

**Symptoms:**
- Parsing takes >>2x LR time
- High memory usage

**Diagnosis:**
```bash
export RUST_SITTER_LOG_PERFORMANCE=true
cargo test -- --nocapture

# Look for:
# Forks: 500+  ← Too many!
```

**Solutions:**
1. Add precedence annotations to reduce conflicts
2. Simplify grammar structure
3. Use profiling to find hot spots

### Problem: "Incorrect parse tree"

**Symptoms:**
- Tree structure doesn't match expected
- Syntax errors on valid input

**Diagnosis:**
```rust
#[test]
fn debug_parse_tree() {
    let input = b"your input here";
    let tree = parse_with_glr(input);

    // Print tree structure
    println!("{:#?}", tree.root_node());

    // Or use visualization
    println!("{}", tree.root_node().to_sexp());
}
```

**Solutions:**
1. Check grammar precedence rules
2. Verify tokenizer patterns
3. Inspect parse table conflicts (see "Debugging Conflicts")

### Problem: "Out of memory during parsing"

**Symptoms:**
- Parser crashes with OOM
- Happens on large inputs

**Solutions:**
```rust
// Limit stack depth
const MAX_STACK_DEPTH: usize = 1000;

impl Parser {
    fn check_stack_depth(&self) -> Result<(), ParseError> {
        if self.stack.len() > MAX_STACK_DEPTH {
            return Err(ParseError::StackOverflow);
        }
        Ok(())
    }
}
```

Or use streaming parsing for large files:
```rust
// Parse in chunks
for chunk in input.chunks(10_000) {
    let tree = parser.parse(chunk, prev_tree.as_ref())?;
    process_tree(tree);
}
```

---

## Next Steps

**After completing this guide, you can:**

✅ Enable GLR mode in your rust-sitter project
✅ Write effective GLR grammars
✅ Debug and resolve parsing conflicts
✅ Optimize GLR parser performance

**Further Reading:**

- [GLR_ARCHITECTURE.md](../architecture/GLR_ARCHITECTURE.md) - Deep dive into GLR internals
- [PRECEDENCE_ASSOCIATIVITY.md](./PRECEDENCE_ASSOCIATIVITY.md) - Advanced conflict resolution
- [TREE_API_COMPATIBILITY_CONTRACT.md](../specs/TREE_API_COMPATIBILITY_CONTRACT.md) - API reference
- [PERFORMANCE_BASELINE.md](../PERFORMANCE_BASELINE.md) - Benchmark results

**Get Help:**

- 📖 Documentation: [docs.rs/rust-sitter](https://docs.rs/rust-sitter)
- 💬 Discussions: [GitHub Discussions](https://github.com/hydro-project/rust-sitter/discussions)
- 🐛 Issues: [GitHub Issues](https://github.com/hydro-project/rust-sitter/issues)

---

**Document Version**: 1.0.0
**Last Updated**: 2025-11-20
**Feedback**: Please report issues or suggest improvements via GitHub

---

END OF USER GUIDE
