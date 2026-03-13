# Adze Integration Guide

This guide covers integrating Adze grammars with editors, build systems, CI/CD pipelines, WASM targets, and LSP servers.

## Table of Contents

1. [Editor Integrations](#editor-integrations)
2. [Build System Integration](#build-system-integration)
3. [CI/CD Integration](#cicd-integration)
4. [WASM Integration](#wasm-integration)
5. [LSP Server Generation](#lsp-server-generation)

---

## Editor Integrations

Adze generates Tree-sitter-compatible parsers that work with any editor supporting tree-sitter.

### VS Code Extension Setup

#### Recommended Extensions

The project recommends these extensions in [`.vscode/extensions.json`](../../.vscode/extensions.json):

```json
{
  "recommendations": [
    "rust-lang.rust-analyzer",
    "tamasfe.even-better-toml",
    "serayuzgur.crates"
  ]
}
```

#### Workspace Settings

Recommended settings in [`.vscode/settings.json`](../../.vscode/settings.json):

```json
{
  "editor.formatOnSave": true,
  "editor.defaultFormatter": null,
  "rust-analyzer.check.command": "clippy",
  "rust-analyzer.cargo.features": "all",
  "files.trimTrailingWhitespace": true,
  "files.insertFinalNewline": true,
  "[rust]": {
    "editor.defaultFormatter": "rust-lang.rust-analyzer"
  }
}
```

#### Using Adze Grammars in VS Code

1. **Install tree-sitter CLI**:
   ```bash
   cargo install tree-sitter-cli
   ```

2. **Generate Tree-sitter grammar** from your Adze grammar:
   ```bash
   adze-tool generate --grammar ./src/grammar.rs --output ./tree-sitter-mylang
   ```

3. **Create VS Code extension**:
   ```bash
   npx --yes yo tree-sitter
   ```

4. **Link the grammar** in your extension's `package.json`:
   ```json
   {
     "contributes": {
       "languages": [{
         "id": "mylang",
         "extensions": [".mylang"],
         "grammar": "grammar/tree-sitter-mylang.wasm"
       }],
       "grammars": [{
         "language": "mylang",
         "scopeName": "source.mylang",
         "path": "grammar/tree-sitter-mylang.wasm"
       }]
     }
   }
   ```

### Neovim/nvim-treesitter Configuration

#### Installation

1. **Ensure nvim-treesitter is installed**:
   ```lua
   use {
     'nvim-treesitter/nvim-treesitter',
     run = ':TSUpdate'
   }
   ```

2. **Configure parser installation** in `init.lua`:
   ```lua
   require'nvim-treesitter.configs'.setup {
     ensure_installed = { "mylang" },
     highlight = {
       enable = true,
     },
     incremental_selection = {
       enable = true,
     },
     indent = {
       enable = true,
     },
   }
   ```

#### Installing Custom Adze Grammars

1. **Generate Tree-sitter output** from your Adze grammar:
   ```bash
   adze-tool generate --grammar ./src/grammar.rs --output ./tree-sitter-mylang
   ```

2. **Install the parser manually**:
   ```lua
   local parser_config = require "nvim-treesitter.parsers".get_parser_configs()
   parser_config.mylang = {
     install_info = {
       url = "https://github.com/yourorg/tree-sitter-mylang",
       files = { "src/parser.c", "src/scanner.c" },
       branch = "main",
     },
     filetype = "mylang",
   }
   ```

3. **Run installation**:
   ```vim
   :TSInstall mylang
   ```

#### Filetype Detection

Add to `ftdetect/mylang.vim`:
```vim
autocmd BufRead,BufNewFile *.mylang set filetype=mylang
```

### Emacs tree-sitter-mode

#### Prerequisites

- Emacs 29+ with native tree-sitter support
- Or `tree-sitter` package from ELPA/MELPA

#### Installation

1. **Install tree-sitter package**:
   ```elisp
   (use-package tree-sitter
     :ensure t
     :config
     (global-tree-sitter-mode)
     (add-hook 'tree-sitter-after-on-hook #'tree-sitter-hl-mode))
   
   (use-package tree-sitter-langs
     :ensure t)
   ```

2. **Add custom grammar**:
   ```elisp
   (add-to-list 'tree-sitter-load-path "/path/to/tree-sitter-mylang/dist")
   (add-to-list 'auto-mode-alist '("\\.mylang\\'" . mylang-mode))
   
   (define-derived-mode mylang-mode prog-mode "MyLang"
     (tree-sitter-hl-mode))
   
   (tree-sitter-require 'mylang)
   ```

#### Queries for Highlighting

Create `queries/mylang/highlights.scm`:
```scheme
(keyword) @keyword
(string) @string
(number) @number
(comment) @comment
(function_name) @function
```

### Helix Support

Helix has built-in tree-sitter support and auto-detects grammars.

#### Configuration

1. **Add language to** `~/.config/helix/languages.toml`:
   ```toml
   [[language]]
   name = "mylang"
   scope = "source.mylang"
   file-types = ["mylang"]
   roots = []
   comment-token = "#"
   
   [language.indent]
   tab-width = 2
   unit = "  "
   ```

2. **Install grammar**:
   ```bash
   cp tree-sitter-mylang.wasm ~/.config/helix/runtime/grammars/
   ```

3. **Add queries** in `~/.config/helix/runtime/queries/mylang/`:
   - `highlights.scm` - Syntax highlighting
   - `indents.scm` - Indentation rules
   - `textobjects.scm` - Text objects for navigation

---

## Build System Integration

### Cargo Workspace Configuration

Adze uses a 75-crate workspace defined in [`Cargo.toml`](../../Cargo.toml):

```toml
[workspace]
resolver = "2"
members = [
  "macro",
  "runtime",
  "runtime2",
  "common",
  "ir",
  "glr-core",
  "tablegen",
  "tool",
  # ... additional members
]

# Excluded from default workspace commands
exclude = [
  "runtime/fuzz",
  "tools/ts-bridge",
  "crates/ts-c-harness",
  "example",
]
```

#### Workspace Metadata

```toml
[workspace.package]
edition = "2024"
rust-version = "1.92.0"
license = "Apache-2.0 OR MIT"

[workspace.lints.rust]
unsafe_op_in_unsafe_fn = "deny"
unused_must_use = "deny"
missing_docs = "warn"
```

### Feature Flags Reference

#### Core Features

| Feature | Crates | Description |
|---------|--------|-------------|
| `pure-rust` | `adze`, grammars | Pure Rust GLR implementation without C backend |
| `ts-compat` | `adze`, grammars | Tree-sitter compatibility layer |
| `serialization` | `adze-glr-core` | Serialize/deserialize parse tables |
| `perf-counters` | `adze` | Performance counters for profiling |
| `test-api` | Various | Additional testing APIs |

#### Grammar Features

| Feature | Description |
|---------|-------------|
| `pure-rust` | Use pure Rust parser implementation |
| `ts-compat` | Enable Tree-sitter compatibility helpers |

#### Usage in Cargo.toml

```toml
[dependencies]
adze = { version = "0.1", features = ["pure-rust", "ts-compat"] }
adze-python = { version = "0.1", features = ["pure-rust"] }
```

### Just Command Runner Recipes

The project uses `just` as a command runner. Key recipes from [`justfile`](../../justfile):

#### Development Commands

```bash
# Format all code
just fmt

# Run clippy on core crates
just clippy

# Run tests on core crates
just test

# Build everything
just build

# Build with release optimizations
just release
```

#### PR Gate Commands

```bash
# Required PR gate - MUST PASS before submitting
just ci-supported
```

This runs:
1. `cargo fmt --all -- --check`
2. `cargo clippy` on 7 core crates
3. `cargo test` on core crates
4. Doc tests with serialization feature

#### Testing Commands

```bash
# Run test matrix
just matrix

# Update insta snapshots
just snap

# Run mutation testing
just mutate

# Run mutation testing on all crates
just mutate-all
```

#### Utility Commands

```bash
# Verify MSRV consistency
just check-msrv

# Show crates.io publish order
just publish-order

# Clean build artifacts
just clean
```

#### Environment Variables

| Variable | Default | Purpose |
|----------|---------|---------|
| `RUST_TEST_THREADS` | 2 | Test thread concurrency |
| `RAYON_NUM_THREADS` | 4 | Rayon thread pool size |
| `CARGO_BUILD_JOBS` | 4 (CI: 2) | Parallel build jobs |
| `TOKIO_WORKER_THREADS` | 2 | Tokio worker threads |
| `ADZE_EMIT_ARTIFACTS` | unset | Set `true` to output generated files |
| `ADZE_LOG_PERFORMANCE` | unset | Set `true` for GLR performance logging |

---

## CI/CD Integration

### GitHub Actions Workflows

The project uses 16 workflows in [`.github/workflows/`](../../.github/workflows/):

| Workflow | Purpose |
|----------|---------|
| `ci.yml` | Main CI with PR gate |
| `pure-rust-ci.yml` | Pure Rust implementation tests |
| `core-tests.yml` | Core crate testing |
| `golden-tests.yml` | Tree-sitter parity validation |
| `microcrate-ci.yml` | Governance micro-crates |
| `fuzz.yml` | Fuzz testing |
| `benchmarks.yml` | Performance benchmarks |
| `release.yml` | Release automation |

### PR Gate Requirements

The PR gate is defined in `ci.yml` and executed via `just ci-supported`:

```yaml
jobs:
  ci-supported:
    name: CI Supported (PR Gate)
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      
      - name: CI Supported
        run: just ci-supported
```

#### Supported Crates (7 crates)

The PR gate covers these core crates:
- `adze` - Main runtime library
- `adze-macro` - Proc-macro attributes
- `adze-tool` - Build-time code generation
- `adze-common` - Shared grammar expansion
- `adze-ir` - Grammar IR with GLR support
- `adze-glr-core` - GLR parser generation
- `adze-tablegen` - Table compression, FFI generation

### CI Configuration

Standard CI environment variables:

```yaml
env:
  RUST_BACKTRACE: 1
  CARGO_TERM_COLOR: always
  RUST_TEST_THREADS: 2
  RAYON_NUM_THREADS: 4
  TOKIO_WORKER_THREADS: 2
  CARGO_BUILD_JOBS: 4
```

### Example Workflow

```yaml
name: CI

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      
      - name: Run PR gate
        run: just ci-supported
```

### Release Process

Releases are triggered via `workflow_dispatch` in [`release.yml`](../../.github/workflows/release.yml):

```yaml
on:
  workflow_dispatch:
    inputs:
      version:
        description: 'Version to release (e.g., 1.2.3)'
        required: true
      release_surface_mode:
        description: 'Release-surface mode'
        default: fixed
        type: choice
        options: [fixed, auto]
      dry_run:
        description: 'Dry run (no actual publish)'
        default: true
        type: boolean
```

#### Release Steps

1. **Validate release surface**:
   ```bash
   ./scripts/validate-release-surface.sh
   ```

2. **Get publish order**:
   ```bash
   ./scripts/release-surface.sh
   ```

3. **Publish to crates.io** (after validation):
   ```bash
   cargo publish -p <crate> --dry-run
   ```

---

## WASM Integration

### Build Process with wasm-pack

The WASM demo is located in [`wasm-demo/`](../../wasm-demo/).

#### Prerequisites

```bash
# Add WASM target
rustup target add wasm32-unknown-unknown

# Install wasm-pack
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
```

#### Building

Use the provided build script [`wasm-demo/build.sh`](../../wasm-demo/build.sh):

```bash
cd wasm-demo
./build.sh
```

Or manually:

```bash
wasm-pack build --target web --out-dir pkg
```

#### Cargo Configuration

[`wasm-demo/Cargo.toml`](../../wasm-demo/Cargo.toml):

```toml
[package]
name = "adze-wasm-demo"
version = "0.1.0"
edition.workspace = true

[lib]
crate-type = ["cdylib"]

[dependencies]
wasm-bindgen = "0.2"
console_error_panic_hook = { version = "0.1", optional = true }
adze-python = { path = "../grammars/python", features = ["pure-rust"] }
web-sys = { version = "0.3", features = ["console"] }
```

### Browser Demo Setup

#### HTML Entry Point

[`wasm-demo/index.html`](../../wasm-demo/index.html):

```html
<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <title>Adze WASM Demo</title>
</head>
<body>
    <script type="module">
        import init, { parse_arithmetic, get_parser_stats } from './pkg/adze_wasm_demo.js';
        
        await init();
        
        const result = parse_arithmetic("1 + 2 * 3");
        console.log(result);
    </script>
</body>
</html>
```

#### Running Locally

```bash
cd wasm-demo
./build.sh

# Start a local server
python3 -m http.server 8000

# Open http://localhost:8000
```

### JavaScript/TypeScript API

#### Exported Functions

From [`wasm-demo/src/lib.rs`](../../wasm-demo/src/lib.rs):

```rust
#[wasm_bindgen]
pub fn parse_arithmetic(source: &str) -> String {
    match adze_example::arithmetic::grammar::parse(source) {
        Ok(ast) => format!("Parse successful! {:?}", ast),
        Err(_) => "Parse failed".to_string(),
    }
}

#[wasm_bindgen]
pub fn get_parser_stats() -> String {
    "Stats: To be implemented".to_string()
}
```

#### Usage in JavaScript

```javascript
import init, { parse_arithmetic } from './pkg/adze_wasm_demo.js';

await init();

const result = parse_arithmetic("1 + 2 * 3");
// "Parse successful! [...]"
```

### Node.js Bindings

For Node.js, build with `nodejs` target:

```bash
wasm-pack build --target nodejs --out-dir pkg-node
```

Usage in Node.js:

```javascript
const { parse_arithmetic } = require('./pkg-node/adze_wasm_demo.js');

const result = parse_arithmetic("1 + 2 * 3");
console.log(result);
```

---

## LSP Server Generation

Adze can automatically generate LSP servers from grammars using the [`lsp-generator/`](../../lsp-generator/) crate.

### Quick Start

#### Installation

```bash
cargo install adze-lsp-generator
```

#### Generate LSP Server

Generate with all features:

```bash
adze-lsp-gen generate \
  --name my-language-lsp \
  --grammar ./my-grammar/src/lib.rs \
  --output ./my-lsp-server \
  --all-features
```

Generate with specific features:

```bash
adze-lsp-gen generate \
  --name my-language-lsp \
  --grammar ./my-grammar/src/lib.rs \
  --completion \
  --hover \
  --diagnostics
```

### Builder API

Programmatic generation:

```rust
use adze_lsp_generator::LspBuilder;

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

### Feature Configuration

#### Completion

Provides intelligent code completion:
- Keywords from terminal symbols
- Symbol names from non-terminals
- Context-aware suggestions

#### Hover

Shows documentation on hover:
- Grammar rule information
- Keyword documentation
- Multi-language support (Rust, JavaScript/TypeScript, Python)
- UTF-8 safe text processing

#### Diagnostics

Real-time syntax error detection:
- Parse errors with exact locations
- Error recovery suggestions
- Incremental updates

#### Coming Soon

- Semantic Tokens - Syntax highlighting
- Goto Definition - Navigate to symbol definitions
- Find References - Find all usages
- Rename - Safe symbol renaming
- Code Actions - Quick fixes and refactoring

### Generated Server Structure

```
my-lsp-server/
├── Cargo.toml
├── src/
│   ├── main.rs           # Entry point
│   ├── handlers/
│   │   ├── completion.rs # Completion handler
│   │   ├── hover.rs      # Hover handler
│   │   └── diagnostics.rs # Diagnostics handler
│   └── capabilities.rs   # LSP capabilities
└── README.md
```

### Editor-Specific Setup

#### VS Code

1. **Generate LSP server**:
   ```bash
   adze-lsp-gen generate --name mylang-lsp --grammar ./grammar.rs --output ./mylang-lsp
   ```

2. **Create VS Code extension**:
   ```bash
   npx yo code
   ```

3. **Configure extension** in `package.json`:
   ```json
   {
     "contributes": {
       "languages": [{
         "id": "mylang",
         "extensions": [".mylang"]
       }],
       "commands": [{
         "command": "mylang.restartServer",
         "title": "MyLang: Restart Language Server"
       }]
     },
     "activationEvents": [
       "onLanguage:mylang"
     ],
     "main": "./out/extension",
     "dependencies": {
       "vscode-languageclient": "^9.0.0"
     }
   }
   ```

4. **Implement client** in `extension.ts`:
   ```typescript
   import { LanguageClient, ServerOptions, LanguageClientOptions } from 'vscode-languageclient/node';
   
   let client: LanguageClient;
   
   export function activate(context: vscode.ExtensionContext) {
       const serverOptions: ServerOptions = {
           command: context.asAbsolutePath('./mylang-lsp'),
           args: []
       };
       
       const clientOptions: LanguageClientOptions = {
           documentSelector: [{ scheme: 'file', language: 'mylang' }]
       };
       
       client = new LanguageClient('mylang', 'MyLang Language Server', serverOptions, clientOptions);
       client.start();
   }
   ```

#### Neovim

Configure with `nvim-lspconfig`:

```lua
local lspconfig = require('lspconfig')

lspconfig.mylang.setup {
    cmd = { "/path/to/mylang-lsp" },
    filetypes = { "mylang" },
    root_dir = lspconfig.util.root_pattern(".git", "."),
    settings = {},
}
```

#### Emacs

Using `lsp-mode`:

```elisp
(use-package lsp-mode
  :config
  (lsp-register-client
   (make-lsp-client
    :new-connection (lsp-stdio-connection "/path/to/mylang-lsp")
    :major-modes '(mylang-mode)
    :server-id 'mylang-lsp))
  
  (add-hook 'mylang-mode-hook #'lsp))
```

#### Helix

Add to `~/.config/helix/languages.toml`:

```toml
[[language]]
name = "mylang"
language-servers = ["mylang-lsp"]

[language-server.mylang-lsp]
command = "/path/to/mylang-lsp"
```

---

## Troubleshooting

### Common Issues

#### WASM Build Fails

Ensure you have the WASM target:
```bash
rustup target add wasm32-unknown-unknown
```

#### Tree-sitter Grammar Not Loading

1. Verify the WASM file exists
2. Check file paths are correct
3. Ensure tree-sitter version compatibility

#### LSP Server Not Starting

1. Check server logs
2. Verify grammar path is correct
3. Ensure all features are enabled

### Getting Help

- **Documentation**: See [`docs/`](../) directory
- **Issues**: GitHub Issues
- **Known Issues**: [`docs/status/KNOWN_RED.md`](../status/KNOWN_RED.md)

---

## See Also

- [Getting Started Tutorial](../tutorials/getting-started.md)
- [GLR Quickstart](../tutorials/glr-quickstart.md)
- [JSON Tutorial](../tutorials/json-tutorial.md)
- [Testing Guide](../testing/TESTING_GUIDE.md)
- [API Stability](../status/API_STABILITY.md)
