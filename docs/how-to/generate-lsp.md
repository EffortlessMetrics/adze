# Adze LSP Generator

> **Status**: Experimental prototype in Adze 0.8.0-dev.

The Adze LSP Generator (`lsp-generator` crate) is an experimental tool that generates Language Server Protocol (LSP) server projects from Adze grammar definitions.

## Features

The generator currently supports:

- **Diagnostics**: Real-time syntax error reporting powered by the Adze runtime.
- **Hover Information**: Basic documentation lookup for common language keywords.
- **Code Completion**: Context-aware suggestions for grammar rules and keywords.
- **Syntax Highlighting**: Structural information for editor-side highlighting.

## Quick Start

The LSP generator is intended to be used as a CLI tool or library.

```bash
# Build the generator
cargo build -p adze-lsp-generator

# Generate an LSP for your grammar
# (Requires a compiled grammar.json)
cargo run -p adze-lsp-generator -- \
  --name my-lang-server \
  --grammar path/to/grammar.json \
  --output ./my-lsp
```

## How It Works

The generator analyzes your grammar's Intermediate Representation (IR) and produces a complete Rust project using `tower-lsp`.

1. **Codegen**: It generates `server.rs` (the LSP driver) and `handlers.rs` (the logic for completion/hover).
2. **Handlers**: The generated handlers use the Adze `Parser` and `Extract` logic to understand the code structure and provide relevant LSP responses.
3. **Workspace**: It produces a `Cargo.toml` so the generated server can be built and run immediately.

## Limitations

- **State Management**: Multi-file analysis is not yet supported.
- **Advanced Predicates**: Custom LSP logic usually requires manual editing of the generated `handlers.rs`.
- **Formatting**: Automatic code formatting must be implemented manually in the generated project.

## Future Plans

- Full integration with the Adze CLI (`adze generate-lsp`).
- Better support for Tree-sitter query files (`.scm`) for highlighting.
- Automatic VS Code extension packaging.
