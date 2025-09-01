# Rust Sitter API Documentation

Complete API reference for rust-sitter v0.6.0 - the production-ready pure-Rust parser generator with GLR support.

> **Note**: This document covers the stable API. Some advanced features (queries, incremental parsing, serialization) are available under feature flags and their APIs may change before v1.0.
> 
> **v0.5+ Breaking Changes**: The `SymbolMetadata` struct has been updated for GLR compatibility. See [Migration Guide](./MIGRATION_GUIDE.md#symbolmetadata-struct-changes) for upgrade instructions.

## Table of Contents

1. [Core Types](#core-types)
2. [Grammar Definition](#grammar-definition)
3. [Parser API](#parser-api)
4. [External Scanners](#external-scanners)
5. [Query Language](#query-language)
6. [Error Recovery](#error-recovery)
7. [Incremental Parsing](#incremental-parsing)
8. [Visitor API](#visitor-api)
9. [Table Generation](#table-generation)
10. [Testing Framework](#testing-framework)
11. [Performance Analysis](#performance-analysis)
12. [LSP Generation](#lsp-generation)
13. [Playground API](#playground-api)
14. [Thread Safety & Concurrency](#thread-safety)

## Core Types

### `Grammar`
```rust
pub struct Grammar {
    pub name: String,
    pub rules: IndexMap<String, Rule>,
    pub extras: Vec<RuleId>,
    pub conflicts: Vec<Vec<RuleId>>,
    pub externals: Vec<ExternalToken>,
    pub inline: Vec<RuleId>,
    pub supertypes: Vec<RuleId>,
    pub word: Option<RuleId>,
    pub precedences: Vec<PrecedenceLevel>,
}
```

The main grammar structure containing all rules and metadata.

### `Rule`
```rust
pub enum Rule {
    Symbol(String),
    Blank,
    String(String),
    Pattern(String, Option<String>),
    Repeat(Box<Rule>),
    Repeat1(Box<Rule>),
    Choice(Vec<Rule>),
    Seq(Vec<Rule>),
    Prec(Precedence, Box<Rule>),
    PrecLeft(Precedence, Box<Rule>),
    PrecRight(Precedence, Box<Rule>),
    PrecDynamic(Precedence, Box<Rule>),
    Optional(Box<Rule>),
    Field(String, Box<Rule>),
    Alias(Box<Rule>, AliasValue),
    Token(TokenModifier, Box<Rule>),
}
```

Represents different types of grammar rules.

### `ParseNode`
```rust
pub struct ParseNode {
    pub symbol: SymbolId,
    pub children: Vec<ParseNode>,
    pub start_byte: usize,
    pub end_byte: usize,
    pub field_name: Option<String>,
}
```

A node in the parse tree.

### `SymbolMetadata`
```rust
pub struct SymbolMetadata {
    pub name: String,
    pub visible: bool,     // Renamed from is_visible (v0.5+)
    pub named: bool,       // New field (v0.5+)
    pub hidden: bool,      // New field for extras (v0.5+)
    pub terminal: bool,    // Renamed from is_terminal (v0.5+)
    // GLR-specific extensions (v0.5+)
    pub is_terminal: bool, // GLR core compatibility
    pub is_extra: bool,    // Extra symbol marker
    pub is_fragile: bool,  // Fragile token marker
    pub symbol_id: SymbolId, // Symbol identifier
}
```

Metadata for symbols in the grammar. **Breaking Change in v0.5**: Field names have been standardized (`is_visible` → `visible`, `is_terminal` → `terminal`) and new fields added for GLR compatibility. See [Migration Guide](./MIGRATION_GUIDE.md#symbolmetadata-struct-changes) for upgrade instructions.

## Grammar Definition

### Procedural Macros

#### `#[rust_sitter::grammar("name")]`
Defines a grammar module:
```rust
#[rust_sitter::grammar("my_language")]
mod grammar {
    // Grammar definitions
}
```

#### `#[rust_sitter::language]`
Marks the root AST type:
```rust
#[rust_sitter::language]
pub struct Program {
    statements: Vec<Statement>,
}
```

#### `#[rust_sitter::leaf(...)]`
Defines leaf nodes:
```rust
// Pattern matching
#[rust_sitter::leaf(pattern = r"\d+", transform = |s| s.parse().unwrap())]
field: u32,

// Exact text
#[rust_sitter::leaf(text = "+")]
plus: (),
```

#### `#[rust_sitter::prec_left(n)]` / `#[rust_sitter::prec_right(n)]`
Sets precedence and associativity:
```rust
#[rust_sitter::prec_left(1)]
Add(Box<Expr>, #[rust_sitter::leaf(text = "+")] (), Box<Expr>),
```

#### `#[rust_sitter::extra]`
Marks nodes that can be skipped:
```rust
#[rust_sitter::extra]
struct Whitespace {
    #[rust_sitter::leaf(pattern = r"\s+")]
    _ws: (),
}
```

## Parser API

### `Parser` (GLR Runtime - `runtime2/`)
The main parser API with Tree-sitter compatibility and production-ready GLR engine integration:

```rust
impl Parser {
    /// Create a new parser
    pub fn new() -> Self;
    
    /// Set the language for parsing
    /// Validates GLR-specific requirements (parse table, tokenizer)
    /// Returns error if language lacks parse table or tokenizer in GLR mode
    pub fn set_language(&mut self, language: Language) -> Result<(), ParseError>;
    
    /// Get the current language
    pub fn language(&self) -> Option<&Language>;
    
    /// Set a timeout for parsing operations
    pub fn set_timeout(&mut self, timeout: Duration);
    
    /// Get the current timeout
    pub fn timeout(&self) -> Option<Duration>;
    
    /// Parse input bytes with optional incremental parsing
    /// Automatically routes to GLR engine when glr-core feature is enabled
    /// Falls back to full parse when incremental features are disabled
    pub fn parse(&mut self, input: impl AsRef<[u8]>, old_tree: Option<&Tree>) -> Result<Tree, ParseError>;
    
    /// Parse UTF-8 string input with automatic validation
    pub fn parse_utf8(&mut self, input: &str, old_tree: Option<&Tree>) -> Result<Tree, ParseError>;
    
    /// Reset the parser state (clears arenas if enabled)
    pub fn reset(&mut self);
}
```

**GLR Integration Status**: **Production Ready** ✅
- Complete GLR engine routing with Tree-sitter API compatibility
- Feature-gated compilation for different GLR capabilities
- Memory-safe GLR forest management with performance monitoring
- Incremental parsing optimization with subtree reuse

**Feature Gates:**
- **`glr-core`**: Enables GLR parsing engine and forest-to-tree conversion (default)
- **`incremental`**: Enables true incremental parsing with subtree reuse and edit operations
- **`arenas`**: Enables arena allocators for improved memory performance
- **`external-scanners`**: Support for custom external scanners (indentation, heredocs)
- **`queries`**: Tree-sitter style query language support (future)

### `Language` Structure (GLR-Compatible)
```rust
pub struct Language {
    pub version: u32,
    pub symbol_count: usize,
    pub field_count: usize,
    pub max_alias_sequence_length: usize,
    
    // GLR-specific fields (production ready)
    pub parse_table: Option<&'static ParseTable>,  // Required for GLR parsing
    pub tokenize: Option<Box<dyn for<'a> Fn(&'a [u8]) -> Box<dyn Iterator<Item = Token> + 'a>>>,  // Required for GLR
    
    // Symbol and field metadata
    pub symbol_names: Vec<String>,
    pub symbol_metadata: Vec<SymbolMetadata>,
    pub field_names: Vec<String>,
    
    #[cfg(feature = "external-scanners")]
    pub external_scanner: Option<Box<dyn ExternalScanner>>,
}

impl Language {
    /// Create a new language with GLR support
    pub fn new_glr(
        parse_table: &'static ParseTable,
        tokenizer: Box<dyn for<'a> Fn(&'a [u8]) -> Box<dyn Iterator<Item = Token> + 'a>>,
        symbol_metadata: Vec<SymbolMetadata>,
    ) -> Self;
    
    /// Validate GLR requirements (parse table and tokenizer present)
    pub fn validate_glr(&self) -> Result<(), String>;
}
```

### `GLRParser`
```rust
impl GLRParser {
    /// Create a new GLR parser
    pub fn new(grammar: Grammar, parse_table: ParseTable) -> Self;
    
    /// Parse potentially ambiguous input
    pub fn parse_ambiguous(&mut self, input: &str) -> Result<ParseResult>;
    
    /// Set maximum number of parallel stacks
    pub fn set_max_stacks(&mut self, max: usize);
}

pub enum ParseResult {
    Single(ParseNode),
    Ambiguous(ParseForest),
}
```

## External Scanners

> **Safety Note**: External scanner FFI interface includes compile-time ABI validation and proper resource cleanup via `destroy_lexer()`. All FFI structs use `#[repr(C)]` with size assertions.

### `ExternalScanner` Trait
```rust
pub trait ExternalScanner: Send + Sync {
    /// Initialize scanner state
    fn create() -> Self where Self: Sized + Default {
        Self::default()
    }
    
    /// Scan for tokens
    fn scan(&mut self, lexer: &mut Lexer, valid_symbols: &[bool]) -> ScanResult;
    
    /// Serialize scanner state
    fn serialize(&self, buffer: &mut Vec<u8>);
    
    /// Deserialize scanner state
    fn deserialize(buffer: &[u8]) -> Self where Self: Sized;
}
```

### `Lexer` Interface
```rust
impl Lexer {
    /// Advance to next character
    pub fn advance(&mut self, skip: bool);
    
    /// Current character
    pub fn lookahead(&self) -> char;
    
    /// Check if at end of input
    pub fn eof(&self) -> bool;
    
    /// Current column position
    pub fn get_column(&self) -> usize;
    
    /// Mark end of current token
    pub fn mark_end(&mut self);
    
    /// Result token type
    pub fn result_symbol(&mut self, symbol: usize);
}
```

### Built-in Scanners

#### `IndentationScanner`
```rust
let scanner = IndentationScanner::new()
    .with_newline_token(NEWLINE)
    .with_indent_token(INDENT)
    .with_dedent_token(DEDENT);
```

#### `HeredocScanner`
```rust
let scanner = HeredocScanner::new()
    .with_delimiters(vec!["<<<", "<<-"])
    .with_content_token(HEREDOC_CONTENT)
    .with_end_token(HEREDOC_END);
```

## Query Language

### Query Compilation
```rust
/// Compile a query string
pub fn compile_query(source: &str) -> Result<Query>;

/// Example query
let query = compile_query(r#"
(function_definition
  name: (identifier) @function.name
  parameters: (parameters) @function.params
  body: (block) @function.body)

(#match? @function.name "^test_")
"#)?;
```

### `QueryCursor`
```rust
impl QueryCursor {
    /// Create new cursor
    pub fn new() -> Self;
    
    /// Execute query on tree
    pub fn matches<'a>(
        &'a mut self,
        query: &'a Query,
        node: Node,
        source: &'a [u8],
    ) -> impl Iterator<Item = QueryMatch<'a>>;
    
    /// Set match limit
    pub fn set_match_limit(&mut self, limit: u32);
}
```

### Predicates
- `#eq?` - Equality check
- `#match?` - Regex matching
- `#any-of?` - Value in set
- `#not-eq?` - Inequality
- `#not-match?` - Negative regex


## Error Recovery

### `ErrorRecoveryConfig`
```rust
impl ErrorRecoveryConfig {
    /// Create builder
    pub fn builder() -> ErrorRecoveryConfigBuilder;
    
    /// Default configuration
    pub fn default() -> Self;
}

impl ErrorRecoveryConfigBuilder {
    /// Set synchronization tokens
    pub fn sync_tokens(mut self, tokens: Vec<u16>) -> Self;
    
    /// Set scope delimiters
    pub fn scope_delimiters(mut self, pairs: Vec<(u16, u16)>) -> Self;
    
    /// Set panic mode threshold
    pub fn panic_threshold(mut self, threshold: usize) -> Self;
    
    /// Enable scope tracking
    pub fn enable_scope_tracking(mut self) -> Self;
    
    /// Build configuration
    pub fn build(self) -> ErrorRecoveryConfig;
}
```

### Recovery Actions
```rust
pub enum RecoveryAction {
    /// Insert a token
    InsertToken(SymbolId),
    
    /// Delete current token
    DeleteToken,
    
    /// Replace current token
    ReplaceToken(SymbolId),
    
    /// Create error node
    CreateErrorNode(Vec<SymbolId>),
}
```

## Incremental Parsing

> **Feature Flags**: Incremental parsing capabilities require feature flags:
> ```toml
> [dependencies] 
> rust-sitter = { version = "0.6", features = ["incremental"] }           # Basic incremental
> rust-sitter = { version = "0.6", features = ["incremental_glr"] }       # GLR + incremental  
> ```

### GLR-Compatible Incremental Parsing (Production Ready)
The GLR runtime2 provides seamless incremental parsing through the standard Parser API:

```rust
use rust_sitter_runtime::{Parser, Tree, InputEdit, Point};

// Create parser with GLR language
let mut parser = Parser::new();
parser.set_language(glr_language)?;

// Initial parse
let tree = parser.parse_utf8("def main(): pass", None)?;

// Create edit operation
let edit = InputEdit {
    start_byte: 4,
    old_end_byte: 8,    // Replace "main"
    new_end_byte: 12,   // With "hello_world"
    start_position: Point { row: 0, column: 4 },
    old_end_position: Point { row: 0, column: 8 },
    new_end_position: Point { row: 0, column: 12 },
};

// Apply edit and reparse incrementally
let mut new_tree = tree.clone();
new_tree.edit(&edit)?;  // Mark dirty regions
let incremental_tree = parser.parse_utf8("def hello_world(): pass", Some(&new_tree))?;
```

**GLR Incremental Features (Integrated into runtime2)**:
- **Automatic Routing**: Parser automatically selects incremental vs full parse based on edit scope
- **Conservative Reuse**: Only reuses subtrees completely outside edit ranges to maintain GLR correctness
- **Performance Optimization**: Input comparison short-circuit for unchanged text
- **Error Safety**: Comprehensive EditError handling prevents overflow/underflow
- **Feature Gating**: Falls back gracefully when incremental features are disabled

### `Tree` - Enhanced with Incremental Support
```rust
impl Tree {
    /// Apply an edit to the tree for incremental parsing
    /// Returns EditError if the edit operation would cause overflow/underflow
    #[cfg(feature = "incremental")]
    pub fn edit(&mut self, edit: &InputEdit) -> Result<(), EditError>;
    
    /// Deep clone a tree for non-destructive analysis
    pub fn clone(&self) -> Self;
    
    /// Get the root node of the tree
    pub fn root_node(&self) -> Node;
    
    /// Get the language used to parse this tree
    pub fn language(&self) -> Option<&Language>;
}
```

### `EditError` - Comprehensive Error Handling
```rust
#[cfg(feature = "incremental")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EditError {
    /// Invalid byte range in edit operation
    InvalidRange {
        start: usize,
        old_end: usize,
    },
    /// Arithmetic overflow during position calculation
    ArithmeticOverflow,
    /// Arithmetic underflow during position calculation  
    ArithmeticUnderflow,
}

impl std::fmt::Display for EditError { /* ... */ }
impl std::error::Error for EditError {}
```

**Error Conditions**:
- `InvalidRange`: Occurs when `old_end_byte < start_byte` or `new_end_byte < start_byte`
- `ArithmeticOverflow`: Prevents integer overflow during node position adjustments
- `ArithmeticUnderflow`: Prevents integer underflow during large deletions

### `InputEdit` - Tree Edit Operations
```rust
pub struct InputEdit {
    pub start_byte: usize,
    pub old_end_byte: usize,
    pub new_end_byte: usize,
    pub start_position: Point,
    pub old_end_position: Point,
    pub new_end_position: Point,
}

pub struct Point {
    pub row: usize,
    pub column: usize,
}
```

### Incremental Parsing Workflow
```rust
use rust_sitter_runtime::{Tree, InputEdit, Point, EditError};

// 1. Parse initial content
let mut tree = parser.parse_utf8("fn main() {}", None)?;

// 2. Create an edit operation
let edit = InputEdit {
    start_byte: 10,
    old_end_byte: 11,  // Replace one character
    new_end_byte: 15,  // With 4 characters
    start_position: Point::new(0, 10),
    old_end_position: Point::new(0, 11),
    new_end_position: Point::new(0, 15),
};

// 3. Apply edit safely with error handling
match tree.edit(&edit) {
    Ok(()) => {
        // Tree updated successfully - nodes marked dirty as needed
        // Now reparse with the new source content
        let new_tree = parser.parse_utf8("fn main() { println!(\"Hello\"); }", Some(&tree))?;
    }
    Err(EditError::InvalidRange { start, old_end }) => {
        eprintln!("Invalid edit range: start={}, end={}", start, old_end);
    }
    Err(EditError::ArithmeticOverflow) => {
        eprintln!("Edit would cause position overflow");
    }
    Err(EditError::ArithmeticUnderflow) => {
        eprintln!("Edit would cause position underflow");
    }
}
```

### `IncrementalParser`
```rust
impl IncrementalParser {
    /// Create new incremental parser
    pub fn new(grammar: Grammar, table: ParseTable) -> Self;
    
    /// Initial parse
    pub fn parse(&mut self, source: &str) -> Result<Tree>;
    
    /// Reparse with edits
    pub fn reparse(
        &mut self,
        old_tree: &Tree,
        edit: &InputEdit,
        new_source: &str,
    ) -> Result<Tree>;
    
    /// Reset parser state
    pub fn reset(&mut self);
}
```

## Visitor API

### `TreeVisitor` Trait
```rust
pub trait TreeVisitor {
    /// Visit a node
    fn visit_node(&mut self, node: &Node) -> VisitAction;
    
    /// Called after visiting children
    fn leave_node(&mut self, node: &Node);
}

pub enum VisitAction {
    /// Continue traversal
    Continue,
    
    /// Skip children
    SkipChildren,
    
    /// Stop traversal
    Stop,
}
```

### Built-in Visitors

#### `StatsVisitor`
```rust
let mut visitor = StatsVisitor::default();
walker.walk(tree.root_node(), &mut visitor);
println!("Total nodes: {}", visitor.node_count);
println!("Max depth: {}", visitor.max_depth);
```

#### `PrettyPrintVisitor`
```rust
let mut visitor = PrettyPrintVisitor::new();
walker.walk(tree.root_node(), &mut visitor);
println!("{}", visitor.output());
```

## Table Generation

### `generate_language`
```rust
/// Generate Tree-sitter compatible language
pub fn generate_language(
    grammar: &Grammar,
    parse_table: &ParseTable,
    lex_table: &LexTable,
    node_types: &NodeTypes,
    abi_version: u32,
) -> Result<TSLanguage>;
```

### `CompressedTable`
```rust
impl CompressedTable {
    /// Compress parse table
    pub fn compress(table: &ParseTable) -> Result<Self>;
    
    /// Get compressed size
    pub fn size_bytes(&self) -> usize;
    
    /// Decompress for verification
    pub fn decompress(&self) -> ParseTable;
}
```

### Language Metadata
```rust
pub struct NodeTypes {
    pub types: Vec<NodeType>,
    pub fields: IndexMap<String, FieldId>,
}

pub struct NodeType {
    pub kind: String,
    pub named: bool,
    pub fields: IndexMap<String, FieldInfo>,
    pub children: ChildrenInfo,
    pub supertypes: Vec<String>,
}
```

## Advanced Usage

### Custom Token Types
```rust
#[derive(Clone, Debug)]
pub struct ComplexToken {
    pub kind: TokenKind,
    pub value: String,
    pub metadata: HashMap<String, Value>,
}

impl From<ComplexToken> for ParseNode {
    // Custom conversion logic
}
```

### Grammar Composition
```rust
let base_grammar = Grammar::new("base");
let extension = Grammar::new("extension");

let composed = GrammarComposer::new()
    .base(base_grammar)
    .extend(extension)
    .compose()?;
```

### Performance Tuning
```rust
let parser = Parser::new(grammar, table)
    .with_stack_size(1024 * 1024)  // 1MB stack
    .with_node_pool_size(10000)     // Pre-allocate nodes
    .with_lookahead_cache(true);    // Enable caching
```

## Error Handling

All parser operations return `Result<T, ParseError>`:

```rust
pub enum ParseError {
    /// Unexpected token
    UnexpectedToken {
        expected: Vec<String>,
        found: String,
        location: Location,
    },
    
    /// Ambiguous parse
    AmbiguousParse {
        alternatives: Vec<ParseNode>,
    },
    
    /// External scanner error
    ScannerError(String),
    
    /// Grammar error
    GrammarError(String),
}
```

## Testing Framework

### `GrammarTester`
```rust
impl GrammarTester {
    /// Create a new tester
    pub fn new(grammar: Grammar) -> Self;
    
    /// Add test corpus
    pub fn add_corpus(&mut self, pattern: &str) -> Result<()>;
    
    /// Run all tests
    pub fn run_all(&self) -> Result<TestResults>;
    
    /// Run property-based tests
    pub fn property_test(&mut self, config: PropertyConfig) -> Result<()>;
    
    /// Fuzz test the grammar
    pub fn fuzz(&mut self, config: FuzzConfig) -> Result<FuzzResults>;
}
```

### `PropertyConfig`
```rust
pub struct PropertyConfig {
    pub max_depth: usize,
    pub iterations: usize,
    pub seed: Option<u64>,
    pub shrink_attempts: usize,
}
```

### `FuzzConfig`
```rust
pub struct FuzzConfig {
    pub timeout: Duration,
    pub max_input_size: usize,
    pub corpus_dir: Option<PathBuf>,
    pub coverage_guided: bool,
}
```

## Performance Analysis

### GLR Performance Monitoring (Production Ready)
The runtime2 includes comprehensive performance monitoring and optimization:

```rust
// Enable performance logging via environment variable
std::env::set_var("RUST_SITTER_LOG_PERFORMANCE", "true");

// Parse with automatic performance monitoring
let mut parser = Parser::new();
parser.set_language(glr_language)?;
let tree = parser.parse_utf8(large_input, old_tree)?;
// Console output: "🚀 Forest->Tree conversion: 1247 nodes, depth 23, took 2.1ms"
```

**Environment Variables:**
- `RUST_SITTER_LOG_PERFORMANCE=true`: Enables detailed forest-to-tree conversion metrics
- `RUST_TEST_THREADS=N`: Controls test concurrency for stable benchmarking
- `RAYON_NUM_THREADS=N`: Limits parallel processing for predictable performance

**GLR Performance Metrics (Integrated):**
- **Node Count**: Total nodes processed during forest-to-tree conversion
- **Tree Depth**: Maximum depth of the parse tree for stack usage estimation
- **Conversion Time**: Time spent converting GLR forest to Tree-sitter tree format
- **Memory Usage**: Arena allocation tracking when arenas feature is enabled
- **Parse Route**: Whether incremental or full parsing was selected

**Performance Features:**
- **Zero-Cost Abstractions**: Performance monitoring has no overhead when disabled
- **Smart Caching**: Input comparison optimization prevents unnecessary reparsing
- **Memory Efficiency**: Arena allocators reduce allocation overhead
- **Bounded Concurrency**: Thread pool management prevents resource exhaustion

### `Profiler`
```rust
impl Profiler {
    /// Create new profiler
    pub fn new() -> Self;
    
    /// Profile grammar on corpus
    pub fn analyze(
        &mut self,
        grammar: &Grammar,
        corpus: &[String],
    ) -> Result<ProfileStats>;
    
    /// Generate flame graph
    pub fn flame_graph(&self, output: &Path) -> Result<()>;
    
    /// Memory usage analysis
    pub fn memory_profile(&mut self) -> MemoryStats;
    
    /// GLR-specific performance analysis
    pub fn glr_stats(&mut self) -> GLRProfileStats;
}
```

### `ProfileStats`
```rust
pub struct ProfileStats {
    pub parse_time: Duration,
    pub tokens_per_second: f64,
    pub memory_usage: usize,
    pub cache_hit_rate: f64,
    pub hotspots: Vec<Hotspot>,
}
```

### Grammar Optimization
```rust
/// Optimize grammar for performance
pub fn optimize_grammar(grammar: &Grammar) -> GrammarOptimizer;

impl GrammarOptimizer {
    /// Inline small rules
    pub fn inline_rules(mut self, enabled: bool) -> Self;
    
    /// Compress parse tables
    pub fn compress_tables(mut self, enabled: bool) -> Self;
    
    /// Optimize for size or speed
    pub fn optimization_level(mut self, level: OptLevel) -> Self;
    
    /// Build optimized grammar
    pub fn build(self) -> Result<Grammar>;
}
```

## LSP Generation

### `LspConfig`
```rust
impl LspConfig {
    /// Create builder
    pub fn builder() -> LspConfigBuilder;
}

impl LspConfigBuilder {
    /// Enable semantic tokens
    pub fn with_semantic_tokens(mut self, enabled: bool) -> Self;
    
    /// Enable goto definition
    pub fn with_goto_definition(mut self, enabled: bool) -> Self;
    
    /// Enable completions
    pub fn with_completions(mut self, enabled: bool) -> Self;
    
    /// Enable diagnostics
    pub fn with_diagnostics(mut self, enabled: bool) -> Self;
    
    /// Custom handlers
    pub fn with_custom_handler(
        mut self,
        method: &str,
        handler: Box<dyn LspHandler>,
    ) -> Self;
    
    /// Build configuration
    pub fn build(self) -> LspConfig;
}
```

### `generate_lsp`
```rust
/// Generate LSP server from grammar
pub fn generate_lsp(
    grammar: &Grammar,
    config: &LspConfig,
    output_dir: &Path,
) -> Result<()>;

/// Generate VS Code extension
pub fn generate_vscode_extension(
    grammar: &Grammar,
    lsp_config: &LspConfig,
    extension_config: &ExtensionConfig,
    output_dir: &Path,
) -> Result<()>;
```

## Playground API

### `PlaygroundServer`
```rust
impl PlaygroundServer {
    /// Create new playground server
    pub fn new(port: u16) -> Self;
    
    /// Add grammar to playground
    pub fn add_grammar(
        &mut self,
        name: &str,
        grammar: Grammar,
    ) -> Result<()>;
    
    /// Start server
    pub async fn start(self) -> Result<()>;
}
```

### `PlaygroundConfig`
```rust
pub struct PlaygroundConfig {
    pub theme: Theme,
    pub examples: Vec<Example>,
    pub features: PlaygroundFeatures,
}

pub struct PlaygroundFeatures {
    pub syntax_highlighting: bool,
    pub parse_tree_view: bool,
    pub query_editor: bool,
    pub performance_metrics: bool,
    pub export_options: ExportOptions,
}
```

## Feature Flags

Rust-sitter uses feature flags to enable optional functionality. Configure features in your `Cargo.toml`:

```toml
[dependencies]
rust-sitter = { version = "0.6", features = ["incremental", "external-scanners", "queries"] }
```

### Available Features

#### Core Features (runtime2) - Production Ready
- **`default`** = `["glr-core"]` - Enables GLR parser core integration
- **`glr-core`** - GLR (Generalized LR) parser support for ambiguous grammars with multi-action cells
- **`incremental`** - Tree editing and incremental parsing with conservative subtree reuse
- **`external-scanners`** - Support for custom external scanners (indentation, heredocs, etc.)
- **`arenas`** - Arena allocators for improved memory performance during parsing
- **`queries`** - Tree-sitter style query language support (future expansion)

#### Combined Features (runtime2)
- **`incremental_glr`** - Combines GLR and incremental parsing for maximum capabilities
- **`all-features`** - Enables all available features for comprehensive functionality

#### Backend Features (runtime) - Legacy
- **`tree-sitter-c2rust`** (default) - Pure Rust Tree-sitter implementation, WASM-compatible
- **`tree-sitter-standard`** - Standard C Tree-sitter runtime

#### Development Features
- **`stub-ts`** (ts-bridge) - Development mode with stubbed Tree-sitter libraries
- **`with-grammars`** (ts-bridge) - Production mode with actual Tree-sitter grammars
- **`test-api`** (glr-core) - Internal debug helpers for integration tests

### Feature Compatibility

**Incremental Parsing** (requires `incremental` feature):
```rust
#[cfg(feature = "incremental")]
use rust_sitter_runtime::{Tree, InputEdit, EditError};

#[cfg(feature = "incremental")]
fn edit_tree(tree: &mut Tree, edit: InputEdit) -> Result<(), EditError> {
    tree.edit(&edit)
}

#[cfg(not(feature = "incremental"))]
fn edit_tree(_tree: &mut Tree, _edit: InputEdit) -> Result<(), EditError> {
    Err("Incremental parsing not enabled".into())
}
```

**WASM Compatibility**:
- Use `tree-sitter-c2rust` feature for browser environments
- Incremental parsing works in WASM with checked arithmetic safety
- External scanners require WASM-compatible implementations

**Performance Tuning**:
- Enable `arenas` for reduced allocation overhead
- Use `glr-core` for complex grammars with conflicts
- Consider `external-scanners` for languages with significant whitespace semantics

## Thread Safety

- `Grammar`: `Send + Sync`
- `Parser`: `Send` (not `Sync`)
- `ExternalScanner`: `Send + Sync`
- `Query`: `Send + Sync`
- `ParseNode`: `Send + Sync`
- `Tree`: `Send + Sync` (with incremental feature)
- `GrammarTester`: `Send`
- `Profiler`: `Send`
- `PlaygroundServer`: `Send`

Use `Arc<Grammar>` to share grammars across threads.

### Concurrency Management (v0.5+)
```rust
use rust_sitter::concurrency_caps;

/// Initialize bounded thread pools for stable performance
pub fn init_concurrency_caps();

/// Bounded parallel iteration with configurable concurrency
pub fn bounded_parallel_map<T, R, F>(
    items: Vec<T>, 
    concurrency: usize, 
    f: F
) -> Vec<R>
where
    T: Send,
    R: Send,
    F: Fn(T) -> R + Send + Sync;
```

**Environment Variables** (configurable caps):
- `RUST_TEST_THREADS`: Test parallelism (default: 2)
- `RAYON_NUM_THREADS`: Rayon thread pool size (default: 4) 
- `TOKIO_WORKER_THREADS`: Tokio async workers (default: 2)
- `TOKIO_BLOCKING_THREADS`: Tokio blocking pool (default: 8)
- `CARGO_BUILD_JOBS`: Parallel compilation (default: 4)

**Usage**: Call `concurrency_caps::init_concurrency_caps()` once at startup for stable resource usage across machines.

## Version Compatibility

- Tree-sitter ABI: v14-v15
- Minimum Rust: 1.89.0 (Rust 2024 Edition)
- WASM targets: wasm32-unknown-unknown, wasm32-wasi
- Supported platforms: Linux, macOS, Windows, WebAssembly

**Recent Changes (August 2025)**:
- Updated SymbolMetadata API for GLR compatibility (breaking change)
- Added concurrency caps system for stable testing
- Implemented grammar loading and parse table generation
- Enhanced GLR parser infrastructure