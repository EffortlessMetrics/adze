# Known Limitations

> **Doc status:** Up to date for Adze 0.8.0-dev.

Adze is a high-performance GLR parser generator. While it achieves high compatibility with the Tree-sitter ecosystem, there are some known limitations and experimental areas.

## ✅ Supported Features

- **Core Grammar**: Sequences, choices, repeats, optionals, and recursive types.
- **Pure-Rust LR(1)**: Fast, zero-dependency parsing for deterministic grammars.
- **GLR (Generalized LR)**: Handles ambiguous grammars by forking and merging stacks.
- **Precedence & Associativity**: Full support for `#[adze::prec_left]`, `#[adze::prec_right]`, etc.
- **Extra Tokens**: Support for whitespace and comments via `#[adze::extra]`.
- **WASM Support**: Native compatibility via the pure-Rust runtime.

## ⚠️ Experimental / Limited Features

### 1. External Scanners
Support for custom Rust-based external scanners is available but the API is still stabilizing. This is required for indentation-sensitive languages like Python.
- **Status**: Implemented for Python in `grammars/python`.

### 2. Query Language
Tree-sitter compatible query support (`.scm` files) is under active development.
- **Status**: Basic pattern matching works; advanced predicates are in progress.

### 3. Incremental Parsing
Reparsing only changed parts of a file is supported in the core engine but may fall back to full parses in complex GLR scenarios.
- **Status**: Conservative fallback enabled; forest-splicing is experimental.

### 4. `transform` Closures
There is a known bug (FR-005) where `transform` closures on leaf nodes are captured but not executed.
- **Workaround**: Use `String` fields and parse the text in your application logic.

## 📊 Language Compatibility

| Language | Status | Notes |
|----------|--------|-------|
| JSON | ✅ Stable | Standard reference grammar. |
| Arithmetic | ✅ Stable | Demonstrates precedence handling. |
| Go | ✅ Stable | High-speed deterministic parsing. |
| JavaScript | 🟡 Stabilizing | Large grammar, uses GLR for conflicts. |
| Python | 🟡 Stabilizing | Requires external indentation scanner. |
| Rust | 🚧 Planned | Complex grammar with many edge cases. |

## 🤝 Roadmap

For upcoming features and milestones, see [ROADMAP.md](../../ROADMAP.md).
