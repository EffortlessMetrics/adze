# Adze LSP Generator

Automatically generate Language Server Protocol (LSP) servers from your grammar.

## Overview

The Adze LSP Generator creates fully-featured language servers from grammar definitions, providing:

- **Syntax Highlighting**: Token-based and semantic
- **Diagnostics**: Real-time syntax error reporting
- **Code Navigation**: Go to definition, find references
- **Code Completion**: Context-aware suggestions
- **Code Actions**: Quick fixes and refactoring
- **Formatting**: Automatic code formatting
- **Folding**: Code folding ranges
- **Symbols**: Document and workspace symbols

## Quick Start

```bash
# Generate LSP for your grammar
adze generate-lsp

# Generate with all features
adze generate-lsp --all-features

# Generate VS Code extension
adze generate-lsp --vscode

# Install and run
cd my-language-lsp
cargo install --path .
my-language-lsp
```

## Basic Configuration

### LSP Configuration File
```toml
# adze-lsp.toml
[lsp]
name = "my-language-lsp"
version = "0.1.0"
language = "my-language"

[features]
diagnostics = true
semantic_tokens = true
goto_definition = true
completions = true
formatting = true
code_actions = true

[server]
transport = "stdio"  # or "tcp"
port = 7658
```

### Programmatic Configuration
```rust
use adze::lsp::{LspConfig, generate_lsp};

let config = LspConfig::builder()
    .name("my-language-lsp")
    .language("my-language")
    .with_diagnostics(true)
    .with_semantic_tokens(true)
    .with_goto_definition(true)
    .with_completions(true)
    .build();

generate_lsp(&grammar, &config, "target/lsp")?;
```

## Feature Implementation

### 1. Hover Information and Documentation

The LSP Generator now includes sophisticated hover functionality that provides contextual information as users navigate their code.

#### Hover Word Extraction
The `get_word_at_position()` function intelligently extracts words under the cursor:

```rust
use anyhow::{anyhow, Result};
use lsp_types::{Position, TextDocumentPositionParams};

fn get_word_at_position(params: &HoverParams) -> Result<String> {
    let uri = &params.text_document_position_params.text_document.uri;
    let path = uri.to_file_path().map_err(|_| anyhow!("invalid uri"))?;
    let text = fs::read_to_string(path)?;
    let position = params.text_document_position_params.position;
    
    let line = text
        .lines()
        .nth(position.line as usize)
        .ok_or_else(|| anyhow!("line out of bounds"))?;
        
    let chars: Vec<char> = line.chars().collect();
    let mut start = position.character as usize;
    let mut end = start;
    
    // Expand backwards to find word start
    while start > 0 {
        let c = chars[start - 1];
        if c.is_alphanumeric() || c == '_' {
            start -= 1;
        } else {
            break;
        }
    }
    
    // Expand forwards to find word end
    while end < chars.len() {
        let c = chars[end];
        if c.is_alphanumeric() || c == '_' {
            end += 1;
        } else {
            break;
        }
    }
    
    Ok(chars[start..end].iter().collect())
}
```

#### Comprehensive Documentation Lookup
The `lookup_documentation()` function provides contextual help for 45+ language constructs:

```rust
fn lookup_documentation(word: &str) -> Option<String> {
    let docs: HashMap<&str, &str> = [
        // Rust language constructs
        ("fn", "Declares a function"),
        ("let", "Declares a variable binding"),
        ("mut", "Makes a binding mutable"),
        ("if", "Conditional expression"),
        ("match", "Pattern matching expression"),
        ("struct", "Defines a struct type"),
        ("enum", "Defines an enum type"),
        ("trait", "Defines a trait"),
        ("impl", "Implements methods or traits"),
        
        // Common types
        ("String", "UTF-8 encoded, growable string type"),
        ("Vec", "Growable array type"),
        ("Option", "Type representing optional values"),
        ("Result", "Type for recoverable errors"),
        
        // JavaScript/TypeScript
        ("function", "Declares a function"),
        ("const", "Declares a constant"),
        ("var", "Declares a variable"),
        ("class", "Declares a class"),
        ("interface", "Declares a TypeScript interface"),
        ("type", "Declares a type alias"),
        
        // Python
        ("def", "Defines a function"),
        ("async", "Declares async function"),
        ("await", "Waits for async operation"),
        ("yield", "Yields a value from generator"),
        
        // Control flow (universal)
        ("return", "Returns a value from function"),
        ("break", "Exits from a loop"),
        ("continue", "Skips to next iteration of loop"),
        ("while", "Loop that continues while condition is true"),
        ("for", "Loop that iterates over a sequence"),
        ("try", "Begins error handling block"),
        ("catch", "Handles errors in try block"),
        ("finally", "Code that always runs after try/catch"),
    ].into_iter().collect();
    
    docs.get(word).map(|doc| format!("**{}**: {}", word, doc))
}
```

#### Generated Hover Handler
The LSP generator creates a complete hover handler that integrates both functions:

```rust
pub async fn handle_hover(params: HoverParams) -> Result<Option<Hover>> {
    // Get the word under cursor
    let word = get_word_at_position(&params)?;

    // Look up documentation
    let contents = match lookup_documentation(&word) {
        Some(doc) => HoverContents::Scalar(MarkedString::String(doc)),
        None => return Ok(None),
    };
    
    Ok(Some(Hover {
        contents,
        range: None,
    }))
}
```

#### HoverProvider Configuration
```rust
use adze_lsp_generator::features::HoverProvider;

let hover_provider = HoverProvider::new(&grammar);

// The provider automatically generates:
// - Hover capabilities in LSP server initialization
// - Complete hover handler with word extraction
// - Documentation lookup with 45+ language constructs
// - Proper LSP response formatting
```

### 2. Syntax Highlighting

#### Semantic Tokens
```rust
// Define token types in grammar
#[adze::semantic_token("function.name")]
pub struct FunctionName(String);

#[adze::semantic_token("variable.declaration")]
pub struct VariableDecl(String);

// Generated LSP will provide semantic highlighting
```

#### Token Modifiers
```rust
#[adze::token_modifiers("declaration", "async")]
pub struct AsyncFunction {
    #[adze::leaf(text = "async")]
    async_keyword: (),
    function: Function,
}
```

### 2. Diagnostics

#### Syntax Errors
```rust
// Automatic from parse errors
let diagnostics = lsp.parse_diagnostics(document)?;

// Custom validation
impl Validator for MyLanguage {
    fn validate(&self, tree: &Tree) -> Vec<Diagnostic> {
        let mut diagnostics = vec![];
        
        // Check for undefined variables
        for node in tree.traverse() {
            if node.kind() == "identifier" {
                if !self.is_defined(&node) {
                    diagnostics.push(Diagnostic {
                        range: node.range(),
                        severity: DiagnosticSeverity::ERROR,
                        message: format!("Undefined: {}", node.text()),
                    });
                }
            }
        }
        
        diagnostics
    }
}
```

#### Linting Rules
```rust
#[derive(LintRule)]
pub struct NoUnusedVariables;

impl LintRule for NoUnusedVariables {
    fn check(&self, tree: &Tree) -> Vec<Diagnostic> {
        // Implementation
    }
}

// Register in config
config.add_lint_rule(NoUnusedVariables);
```

### 3. Code Navigation

#### Go to Definition
```rust
#[derive(Definition)]
pub struct FunctionDef {
    #[definition_name]
    name: Identifier,
    params: Parameters,
    body: Block,
}

#[derive(Reference)]
pub struct FunctionCall {
    #[reference_to(FunctionDef)]
    name: Identifier,
    args: Arguments,
}
```

#### Find References
```rust
impl ReferenceProvider for MyLanguage {
    fn find_references(
        &self,
        tree: &Tree,
        position: Position,
    ) -> Vec<Location> {
        // Automatically generated from annotations
    }
}
```

#### Symbol Index
```rust
#[derive(Symbol)]
#[symbol(kind = "Function")]
pub struct Function {
    #[symbol_name]
    name: String,
    #[symbol_detail]
    signature: String,
}
```

### 4. Code Completion

#### Basic Completions
```rust
#[derive(CompletionProvider)]
pub enum CompletionContext {
    #[completion(keywords = ["if", "else", "while", "for"])]
    Statement,
    
    #[completion(pattern = r"\w+\.", callback = "complete_member")]
    MemberAccess(String),
    
    #[completion(symbols)]
    Expression,
}

fn complete_member(prefix: &str) -> Vec<CompletionItem> {
    // Custom completion logic
}
```

#### Smart Completions
```rust
impl SmartCompletion for MyLanguage {
    fn complete(
        &self,
        context: CompletionContext,
        tree: &Tree,
        position: Position,
    ) -> Vec<CompletionItem> {
        match context {
            CompletionContext::Import => self.complete_imports(),
            CompletionContext::Type => self.complete_types(),
            CompletionContext::Member(obj) => self.complete_members(obj),
        }
    }
}
```

### 5. Code Actions

#### Quick Fixes
```rust
#[derive(CodeAction)]
pub enum QuickFix {
    #[action(diagnostic = "undefined_variable")]
    AddImport {
        name: String,
        module: String,
    },
    
    #[action(diagnostic = "unused_variable")]
    RemoveUnused {
        variable: String,
    },
}

impl CodeAction for QuickFix {
    fn apply(&self, document: &mut Document) -> Result<()> {
        match self {
            QuickFix::AddImport { name, module } => {
                document.add_import(module, name)
            }
            QuickFix::RemoveUnused { variable } => {
                document.remove_definition(variable)
            }
        }
    }
}
```

#### Refactoring
```rust
#[derive(Refactoring)]
pub enum Refactor {
    ExtractFunction {
        selection: Range,
        name: String,
    },
    RenameSymbol {
        old_name: String,
        new_name: String,
    },
    InlineVariable {
        variable: String,
    },
}
```

### 6. Formatting

#### Basic Formatter
```rust
#[derive(Formatter)]
pub struct MyFormatter {
    indent_size: usize,
    max_line_length: usize,
}

impl Formatter for MyFormatter {
    fn format(&self, tree: &Tree) -> String {
        // Pretty printing logic
    }
}
```

#### Configuration
```toml
[format]
indent_size = 4
indent_style = "space"
max_line_length = 100
trailing_comma = true
```

## VS Code Extension

### Generate Extension
```bash
# Generate VS Code extension
adze generate-vscode

# With custom configuration
adze generate-vscode --config vscode.toml
```

### Extension Configuration
```toml
# vscode.toml
[extension]
name = "my-language"
display_name = "My Language"
description = "Language support for My Language"
version = "0.1.0"
publisher = "my-org"
repository = "https://github.com/my-org/my-language"

[activation]
languages = ["my-language"]
extensions = [".ml", ".mli"]

[features]
syntax_highlighting = true
bracket_matching = true
folding = true
outline = true

[themes]
dark = "themes/dark.json"
light = "themes/light.json"
```

### Package and Publish
```bash
# Package extension
cd my-language-vscode
vsce package

# Publish to marketplace
vsce publish
```

## Advanced Features

### 1. Multi-File Analysis

```rust
impl WorkspaceAnalyzer for MyLanguage {
    fn analyze_workspace(&self, files: Vec<File>) -> WorkspaceData {
        let mut symbols = SymbolTable::new();
        let mut imports = ImportGraph::new();
        
        for file in files {
            let tree = self.parse(&file.content);
            symbols.add_file(&file.path, &tree);
            imports.add_file(&file.path, &tree);
        }
        
        WorkspaceData { symbols, imports }
    }
}
```

### 2. Type Checking

```rust
#[derive(TypeChecker)]
pub struct MyTypeChecker {
    rules: Vec<TypeRule>,
}

impl TypeChecker for MyTypeChecker {
    fn check(&self, tree: &Tree) -> Vec<TypeError> {
        // Type checking logic
    }
}
```

### 3. Documentation

```rust
#[derive(DocumentationProvider)]
impl HoverProvider for MyLanguage {
    fn hover(
        &self,
        tree: &Tree,
        position: Position,
    ) -> Option<Hover> {
        let node = tree.node_at_position(position)?;
        
        match node.kind() {
            "function" => Some(self.function_docs(&node)),
            "type" => Some(self.type_docs(&node)),
            _ => None,
        }
    }
}
```

### 4. Code Lens

```rust
#[derive(CodeLens)]
pub enum Lens {
    #[lens(title = "Run", command = "run")]
    RunTest(TestFunction),
    
    #[lens(title = "Debug", command = "debug")]  
    DebugTest(TestFunction),
    
    #[lens(title = "References")]
    ShowReferences(Function),
}
```

## Testing Your LSP

### Unit Tests
```rust
#[test]
fn test_completions() {
    let lsp = MyLanguageLsp::new();
    let doc = Document::new("fn ma");
    
    let completions = lsp.complete(&doc, Position::new(0, 5));
    
    assert!(completions.iter().any(|c| c.label == "main"));
}
```

### Integration Tests
```rust
use adze::lsp::testing::{LspTestClient, TestScenario};

#[test]
async fn test_goto_definition() {
    let client = LspTestClient::new("my-language-lsp");
    
    let scenario = TestScenario::new()
        .file("main.ml", "let x = 42\nprint x")
        .cursor(1, 6)  // On 'x' in print
        .action(Action::GotoDefinition);
    
    let result = client.run(scenario).await?;
    assert_eq!(result.location.line, 0);
}
```

### LSP Test Suite
```bash
# Run LSP test suite
adze test-lsp

# Test specific features
adze test-lsp --feature completions

# Generate test report
adze test-lsp --report
```

## Deployment

### Standalone Server
```bash
# Build release binary
cargo build --release

# Install globally
cargo install --path .

# Run server
my-language-lsp --stdio
```

### Docker Container
```dockerfile
FROM rust:1.70 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:slim
COPY --from=builder /app/target/release/my-language-lsp /usr/local/bin/
EXPOSE 7658
CMD ["my-language-lsp", "--tcp"]
```

### Cloud Deployment
```yaml
# kubernetes.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: my-language-lsp
spec:
  replicas: 3
  template:
    spec:
      containers:
      - name: lsp
        image: myorg/my-language-lsp:latest
        ports:
        - containerPort: 7658
```

## Performance Optimization

### Incremental Analysis
```rust
impl IncrementalAnalyzer for MyLanguage {
    fn analyze_changes(
        &self,
        old_tree: &Tree,
        new_tree: &Tree,
        changes: &[Edit],
    ) -> AnalysisUpdate {
        // Only analyze changed portions
    }
}
```

### Caching
```rust
let config = LspConfig::builder()
    .with_symbol_cache(true)
    .with_parse_cache_size(100)
    .with_completion_cache(true)
    .build();
```

### Parallel Processing
```rust
let config = LspConfig::builder()
    .with_thread_pool_size(num_cpus::get())
    .with_parallel_analysis(true)
    .build();
```

## Monitoring

### Metrics
```rust
impl MetricsProvider for MyLanguageLsp {
    fn metrics(&self) -> LspMetrics {
        LspMetrics {
            parse_time: self.avg_parse_time(),
            memory_usage: self.memory_usage(),
            cache_hit_rate: self.cache_stats(),
            active_documents: self.document_count(),
        }
    }
}
```

### Logging
```toml
[logging]
level = "info"
format = "json"
output = "stdout"
```

## Examples

### Complete LSP Examples
- [JSON LSP](https://github.com/adze/examples/json-lsp)
- [TOML LSP](https://github.com/adze/examples/toml-lsp)
- [SQL LSP](https://github.com/adze/examples/sql-lsp)
- [Python LSP](https://github.com/adze/examples/python-lsp)

### Resources
- [LSP Specification](https://microsoft.github.io/language-server-protocol/)
- [VS Code Extension Guide](https://code.visualstudio.com/api)
- [LSP Tutorial](https://docs.adze.dev/lsp/tutorial)
- [Video: Building an LSP](https://youtube.com/@effortlessmetrics)
