# Rust Sitter LSP Generator

Automatically generate Language Server Protocol (LSP) implementations from rust-sitter grammars.

## Features

- **Automatic LSP Generation**: Generate complete LSP servers from your grammar
- **Multiple Features**: Support for completion, hover, diagnostics, and more
- **Type-Safe**: Leverages Rust's type system for safety
- **Incremental Updates**: Built-in support for incremental parsing
- **Easy to Use**: Simple CLI tool and builder API

## Installation

```bash
cargo install rust-sitter-lsp-generator
```

## Usage

### CLI Tool

Generate an LSP server with all features:

```bash
rust-sitter-lsp-gen generate \
  --name my-language-lsp \
  --grammar ./my-grammar/src/lib.rs \
  --output ./my-lsp-server \
  --all-features
```

Generate with specific features:

```bash
rust-sitter-lsp-gen generate \
  --name my-language-lsp \
  --grammar ./my-grammar/src/lib.rs \
  --completion \
  --hover \
  --diagnostics
```

### Builder API

```rust
use rust_sitter_lsp_generator::LspBuilder;

fn main() -> Result<()> {
    LspBuilder::new("my-language-lsp")
        .version("1.0.0")
        .grammar_path("path/to/grammar.rs")
        .output_dir("./output")
        .feature("completion")
        .feature("hover")
        .feature("diagnostics")
        .build()?;
    
    Ok(())
}
```

## Features

### Completion

Provides intelligent code completion based on your grammar:
- Keywords from terminal symbols
- Symbol names from non-terminals
- Context-aware suggestions

### Hover

Shows documentation on hover with UTF-8 safe word extraction:
- **Grammar rule information**: Detailed descriptions of grammar rules
- **Keyword documentation**: Built-in documentation for common language keywords
- **Multi-language support**: Covers Rust, JavaScript/TypeScript, Python, and generic programming concepts
- **UTF-8 safe text processing**: Properly handles multi-byte characters and Unicode
- **Smart word boundaries**: Accurate word extraction at cursor position
- **Error recovery**: Graceful handling of invalid positions and file access errors

### Diagnostics

Real-time syntax error detection:
- Parse errors with exact locations
- Error recovery suggestions
- Incremental updates

### Coming Soon

- **Semantic Tokens**: Syntax highlighting
- **Goto Definition**: Navigate to symbol definitions
- **Find References**: Find all usages of symbols
- **Rename**: Safe symbol renaming
- **Code Actions**: Quick fixes and refactoring

## Generated Server Structure

```
my-lsp-server/
├── Cargo.toml          # Dependencies and build config
├── main.rs             # Entry point
├── server.rs           # LSP server implementation
└── handlers.rs         # Feature handlers
```

## Running the Generated Server

1. Build the server:
   ```bash
   cd my-lsp-server
   cargo build --release
   ```

2. Run the server:
   ```bash
   ./target/release/my-language-lsp
   ```

3. Configure your editor to use the server

### VS Code Configuration

Create `.vscode/settings.json`:

```json
{
  "my-language.server.path": "./my-lsp-server/target/release/my-language-lsp"
}
```

## Configuration

Create an LSP config file:

```json
{
  "name": "my-language-lsp",
  "version": "1.0.0",
  "language_id": "my-language",
  "file_extensions": [".ml", ".mli"],
  "capabilities": {
    "incremental_sync": true,
    "semantic_tokens": false,
    "code_actions": false,
    "formatting": false,
    "goto_definition": false,
    "find_references": false,
    "rename": false
  },
  "logging": {
    "level": "info",
    "stderr": true
  }
}
```

Then generate from config:

```bash
rust-sitter-lsp-gen from-config --config lsp-config.json
```

## Examples

### Basic Hover Example

Generate an LSP server with enhanced hover support:

```rust
use rust_sitter_lsp_generator::{LspGenerator, LspConfig};
use rust_sitter_ir::Grammar;

fn main() -> anyhow::Result<()> {
    // Load your grammar
    let grammar = Grammar::load_from_file("my_grammar.json")?;
    
    // Configure the LSP server
    let config = LspConfig {
        name: "my-language-lsp".to_string(),
        version: "1.0.0".to_string(),
        language_id: "my-lang".to_string(),
        file_extensions: vec![".mylang".to_string()],
        ..Default::default()
    };
    
    // Generate LSP server with hover support
    LspGenerator::new(grammar)
        .with_config(config)
        .with_hover()
        .generate("./generated-lsp")?;
    
    println!("✅ LSP server with hover support generated!");
    Ok(())
}
```

### Hover Features in Action

The generated hover handler provides documentation for:

- **Rust keywords**: `fn`, `let`, `mut`, `if`, `match`, `struct`, `enum`, etc.
- **Common types**: `String`, `Vec`, `Option`, `Result`, `bool`, `i32`, etc. 
- **JavaScript/TypeScript**: `function`, `const`, `class`, `interface`, `type`, etc.
- **Python constructs**: `def`, `class`, `import`, `async`, `await`, etc.
- **Generic programming**: `return`, `break`, `continue`, `while`, `for`, `try`, etc.

When a user hovers over any of these keywords, they'll see formatted documentation like:

```
**fn**: Declares a function
```

### Complete Examples

See the `examples/` directory for complete examples:
- JavaScript LSP server
- Python LSP server with indentation
- Go LSP server

## Architecture

The LSP generator works by:
1. Analyzing your rust-sitter grammar
2. Extracting keywords, symbols, and structure
3. Generating handler implementations
4. Creating a tower-lsp based server
5. Configuring capabilities based on features

## Contributing

Contributions are welcome! Areas for improvement:
- Additional LSP features
- Performance optimizations
- More language examples
- Editor integration guides

## License

Same as rust-sitter project.