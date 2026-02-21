# LSP Server Generation

This guide covers how to generate Language Server Protocol (LSP) servers from your adze grammars, including the enhanced hover functionality introduced in v0.6.1.

## Overview

The adze LSP generator automatically creates fully-featured language servers with:

- **Hover Information**: Contextual documentation and help
- **Syntax Highlighting**: Token-based and semantic highlighting
- **Diagnostics**: Real-time syntax error reporting
- **Code Completion**: Context-aware suggestions
- **Code Navigation**: Go to definition, find references

## Quick Start

### Basic LSP Generation

```rust
use adze_lsp_generator::{LspConfig, generate_lsp};
use adze_lsp_generator::features::{HoverProvider, CompletionProvider, DiagnosticsProvider};

// Configure LSP features
let config = LspConfig::builder()
    .name("my-language-lsp")
    .language("my_language")
    .with_hover(true)           // Enable hover information
    .with_completions(true)     // Enable code completion
    .with_diagnostics(true)     // Enable syntax diagnostics
    .build();

// Generate the LSP server
generate_lsp(&grammar, &config, "target/my-language-lsp")?;
```

### Running the Generated LSP

```bash
cd target/my-language-lsp
cargo build --release
./target/release/my-language-lsp --stdio
```

## Hover Information (NEW in v0.6.1)

The enhanced hover functionality provides intelligent contextual help as users navigate their code.

### Automatic Word Recognition

The hover system intelligently extracts words under the cursor, supporting:

- Alphanumeric characters and underscores
- Multi-byte UTF-8 characters
- Accurate LSP position handling
- File system integration via URI resolution

### Comprehensive Documentation Database

The hover provider includes documentation for 45+ language constructs:

**Rust Keywords and Types**:
```
fn, let, mut, if, else, match, struct, enum, trait, impl, pub, use, mod
String, str, i32, u32, bool, Vec, Option, Result
```

**JavaScript/TypeScript**:
```
function, const, var, class, interface, type, import, export
```

**Python**:
```
def, async, await, yield, return
```

**Universal Control Flow**:
```
while, for, try, catch, finally, break, continue
```

### Custom Documentation

You can extend the documentation system with grammar-specific help:

```rust
impl HoverProvider {
    pub fn with_custom_docs(mut self, docs: HashMap<String, String>) -> Self {
        self.documentation.extend(docs);
        self
    }
}

// Add custom documentation
let mut custom_docs = HashMap::new();
custom_docs.insert("my_keyword".to_string(), "Custom keyword description".to_string());

let hover_provider = HoverProvider::new(&grammar)
    .with_custom_docs(custom_docs);
```

## Feature Configuration

### HoverProvider

```rust
use adze_lsp_generator::features::HoverProvider;

let hover_provider = HoverProvider::new(&grammar);

// The provider automatically generates:
// - LSP hover capabilities
// - Word extraction logic
// - Documentation lookup
// - Formatted responses
```

### CompletionProvider

```rust
use adze_lsp_generator::features::CompletionProvider;

let completion_provider = CompletionProvider::new(&grammar);

// Provides:
// - Keywords from grammar tokens
// - Symbols from rule names  
// - Context-aware suggestions
```

### DiagnosticsProvider

```rust
use adze_lsp_generator::features::DiagnosticsProvider;

let diagnostics_provider = DiagnosticsProvider::new(&grammar);

// Provides:
// - Syntax error reporting
// - Parse error diagnostics
// - Real-time validation
```

## Generated LSP Structure

The LSP generator creates a complete Rust project:

```
my-language-lsp/
├── Cargo.toml
├── src/
│   ├── main.rs           # LSP server entry point
│   ├── handlers/
│   │   ├── mod.rs
│   │   ├── hover.rs      # Hover implementation
│   │   ├── completion.rs # Completion logic
│   │   └── diagnostics.rs # Error reporting
│   ├── features.rs       # Feature implementations
│   └── lib.rs           # Library exports
└── README.md            # Usage documentation
```

## Integration Examples

### VS Code Integration

```json
{
  "contributes": {
    "languages": [
      {
        "id": "my-language",
        "extensions": [".ml"],
        "configuration": "./language-configuration.json"
      }
    ],
    "grammars": [
      {
        "language": "my-language", 
        "scopeName": "source.my-language",
        "path": "./syntaxes/my-language.tmGrammar.json"
      }
    ]
  }
}
```

### Neovim Integration

```lua
local lspconfig = require('lspconfig')
local configs = require('lspconfig.configs')

configs.my_language = {
  default_config = {
    cmd = {'my-language-lsp', '--stdio'},
    filetypes = {'my-language'},
    root_dir = lspconfig.util.root_pattern('.git'),
  }
}

lspconfig.my_language.setup{}
```

### Emacs Integration

```elisp
(use-package lsp-mode
  :hook ((my-language-mode . lsp))
  :config
  (lsp-register-client
   (make-lsp-client
    :new-connection (lsp-stdio-connection '("my-language-lsp" "--stdio"))
    :major-modes '(my-language-mode)
    :server-id 'my-language-lsp)))
```

## Testing Your LSP

### Unit Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use lsp_types::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_hover_word_extraction() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "let sample_word = 1;").unwrap();

        let uri = Url::from_file_path(file.path()).unwrap();
        let params = HoverParams {
            text_document_position_params: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier { uri },
                position: Position { line: 0, character: 6 },
            },
            work_done_progress_params: Default::default(),
        };

        let word = get_word_at_position(&params).unwrap();
        assert_eq!(word, "sample_word");
    }

    #[test]  
    fn test_documentation_lookup() {
        let result = lookup_documentation("fn");
        assert!(result.is_some());
        assert!(result.unwrap().contains("Declares a function"));
    }
}
```

### Integration Testing

```bash
# Test LSP server manually
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}' | my-language-lsp --stdio

# Test hover functionality
echo '{"jsonrpc":"2.0","id":2,"method":"textDocument/hover","params":{"textDocument":{"uri":"file://test.ml"},"position":{"line":0,"character":5}}}' | my-language-lsp --stdio
```

## Advanced Configuration

### Custom Feature Development

```rust
use adze_lsp_generator::LspFeature;

pub struct CustomFeature {
    // Custom implementation
}

impl LspFeature for CustomFeature {
    fn name(&self) -> &str { "custom" }
    
    fn generate_handler(&self) -> String {
        // Generate custom handler code
    }
    
    fn required_imports(&self) -> Vec<String> {
        vec!["use custom_crate::CustomType;".to_string()]
    }
    
    fn capabilities(&self) -> serde_json::Value {
        serde_json::json!({"customCapability": true})
    }
}
```

### Performance Optimization

```rust
let config = LspConfig::builder()
    .with_caching(true)          // Enable response caching
    .with_parallel_analysis(true) // Parallel processing
    .thread_pool_size(4)         // Limit thread usage
    .build();
```

## Troubleshooting

### Common Issues

**LSP server not starting**:
- Check that the binary path is correct
- Verify the `--stdio` flag is supported
- Check file permissions

**Hover not working**:
- Ensure hover capabilities are enabled
- Verify file URI format is correct
- Check that the word is in the documentation database

**Performance issues**:
- Enable caching in LSP configuration
- Limit thread pool size on resource-constrained systems
- Use incremental parsing for large files

### Debug Mode

```bash
# Run with debug logging
RUST_LOG=debug my-language-lsp --stdio

# Enable performance monitoring  
ADZE_LOG_PERFORMANCE=true my-language-lsp --stdio
```

## Next Steps

- Explore the [LSP Generator documentation](../LSP_GENERATOR.md) for complete API reference
- Check out [API Documentation](../API_DOCUMENTATION.md) for detailed function signatures
- Review [examples](../../../example/) for real-world grammar implementations
- Learn about [advanced LSP features](../advanced/lsp-advanced.md) like semantic tokens and code actions