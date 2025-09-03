# Rust Sitter API Documentation

Complete API reference for rust-sitter v0.6.0 - the production-ready pure-Rust parser generator with GLR support.

> **Note**: This document covers the stable API. Some advanced features (queries, incremental parsing, serialization) are available under feature flags and their APIs may change before v1.0.

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

### `Parser`
```rust
impl Parser {
    /// Create a new parser
    pub fn new(grammar: Grammar, parse_table: ParseTable) -> Self;
    
    /// Set error recovery configuration
    pub fn with_error_recovery(self, config: ErrorRecoveryConfig) -> Self;
    
    /// Parse input string
    pub fn parse(&mut self, input: &str) -> Result<ParseNode>;
    
    /// Get parsing statistics
    pub fn stats(&self) -> ParserStats;
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
        edit: &Edit,
        new_source: &str,
    ) -> Result<Tree>;
    
    /// Reset parser state
    pub fn reset(&mut self);
}
```

### `Edit`
```rust
pub struct Edit {
    pub start_byte: usize,
    pub old_end_byte: usize,
    pub new_end_byte: usize,
    pub start_position: Position,
    pub old_end_position: Position,
    pub new_end_position: Position,
}

pub struct Position {
    pub row: usize,
    pub column: usize,
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

## Thread Safety

- `Grammar`: `Send + Sync`
- `Parser`: `Send` (not `Sync`)
- `ExternalScanner`: `Send + Sync`
- `Query`: `Send + Sync`
- `ParseNode`: `Send + Sync`
- `GrammarTester`: `Send`
- `Profiler`: `Send`
- `PlaygroundServer`: `Send`

Use `Arc<Grammar>` to share grammars across threads.

## Version Compatibility

- Tree-sitter ABI: v15 (production requirement)
- Minimum Rust: 1.89.0
- WASM targets: wasm32-unknown-unknown, wasm32-wasi
- Supported platforms: Linux, macOS, Windows, WebAssembly