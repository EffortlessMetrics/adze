# ADR-006: Tree-sitter Compatibility Layer

## Status

Accepted

## Context

Tree-sitter has become the de facto standard for incremental parsing in editors:
- Integrated into Neovim, Emacs, Atom, and other editors
- Large ecosystem of existing grammars (50+ languages)
- Well-understood API patterns and conventions

While Adze provides a pure-Rust alternative, complete compatibility with Tree-sitter's ecosystem provides significant value:

1. **Grammar Reuse**: Import existing Tree-sitter grammars rather than rewriting
2. **Editor Integration**: Work with existing Tree-sitter integrations
3. **Migration Path**: Users can adopt Adze incrementally
4. **Validation**: Compare output against Tree-sitter for correctness

### Alternatives Considered

1. **No Compatibility**: Pure Rust API with no Tree-sitter resemblance
2. **Full Reimplementation**: Clone Tree-sitter's API exactly
3. **Bridge Layer**: Provide conversion between Tree-sitter and Adze types
4. **ABI Compatibility**: Generate binary-compatible output

## Decision

We maintain **API compatibility** with Tree-sitter while providing **ABI compatibility** for grammar import:

### API Compatibility

The `runtime2/` crate provides Tree-sitter-compatible types:

```rust
// Tree-sitter API                    // Adze runtime2 API
tree_sitter::Parser          →  adze_runtime::Parser
tree_sitter::Tree            →  adze_runtime::Tree
tree_sitter::Node            →  adze_runtime::Node
tree_sitter::Language        →  adze_runtime::Language
tree_sitter::Query           →  adze_runtime::Query (planned)
```

Example usage (identical to Tree-sitter):

```rust
use adze_runtime::{Parser, Language};

let mut parser = Parser::new();
parser.set_language(language)?;

let tree = parser.parse_utf8("def hello(): pass", None)?;
let root = tree.root_node();

// Tree-sitter-compatible traversal
for i in 0..root.child_count() {
    if let Some(child) = root.child(i) {
        println!("Child {}: {}", i, child.kind());
    }
}
```

### ABI Compatibility

The `tools/ts-bridge` tool imports existing Tree-sitter grammars:

```
┌─────────────────────────────────────────────────────────────┐
│                 Tree-sitter Grammar                          │
│                   grammar.js                                 │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                    ts-bridge                                 │
│  - Parse grammar.js                                          │
│  - Extract parse tables from compiled TSLanguage             │
│  - Convert to Adze IR format                                 │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                    Adze IR                                   │
│  - Language-agnostic grammar representation                  │
│  - Works with both backends                                  │
└─────────────────────────────────────────────────────────────┘
```

### Compatibility Matrix

| Feature | Tree-sitter | Adze runtime2 | Notes |
|---------|-------------|---------------|-------|
| Basic parsing | ✅ | ✅ | Full compatibility |
| Incremental parsing | ✅ | 🧪 | Different algorithm, same API |
| GLR/ambiguous grammars | ❌ | ✅ | Adze advantage |
| Query system | ✅ | 📋 | Planned |
| External scanners | ✅ | 🧪 | Experimental |
| WASM support | ✅ | ✅ | Pure Rust easier |
| Node types JSON | ✅ | ✅ | Compatible format |

### Table Generation

The `adze-tablegen` crate generates Tree-sitter-compatible parse tables:

```rust
// Generates ABI-compatible TSLanguage structure
// Can be used with Tree-sitter runtime if needed
pub struct TSLanguage {
    // Compatible with tree_sitter::Language ABI
}
```

## Consequences

### Positive

- **Ecosystem Access**: Can use existing Tree-sitter grammars via ts-bridge
- **Familiar API**: Users already familiar with Tree-sitter can adopt easily
- **Editor Integration**: Can integrate with Tree-sitter-powered editors
- **Validation**: Can compare outputs for correctness testing
- **Migration Path**: Gradual adoption without wholesale rewrites

### Negative

- **API Constraints**: Must maintain compatibility limits innovation
- **Legacy Patterns**: Some Tree-sitter design decisions don't fit Rust idioms
- **Testing Burden**: Must test against Tree-sitter for compatibility
- **Feature Lag**: New Tree-sitter features require tracking

### Neutral

- **Dual Implementation**: Some features exist in both codebases
- **Documentation**: Must document where we differ from Tree-sitter
- **Version Tracking**: Must track Tree-sitter version for compatibility

## Implementation Details

### Node API

```rust
pub struct Node<'tree> {
    // Internal representation differs from Tree-sitter
    // but external API is compatible
}

impl<'tree> Node<'tree> {
    pub fn kind(&self) -> &str;
    pub fn child_count(&self) -> usize;
    pub fn child(&self, i: usize) -> Option<Node<'tree>>;
    pub fn named_child(&self, i: usize) -> Option<Node<'tree>>;
    pub fn start_position(&self) -> Point;
    pub fn end_position(&self) -> Point;
    pub fn byte_range(&self) -> Range<usize>;
    // ... Tree-sitter-compatible methods
}
```

### Tree Cursor

```rust
pub struct TreeCursor<'tree> {
    // Efficient tree traversal matching Tree-sitter's cursor API
}

impl<'tree> TreeCursor<'tree> {
    pub fn goto_first_child(&mut self) -> bool;
    pub fn goto_next_sibling(&mut self) -> bool;
    pub fn goto_parent(&mut self) -> bool;
    pub fn node(&self) -> Node<'tree>;
}
```

## Related

- Related ADRs: [ADR-001](001-pure-rust-glr-implementation.md), [ADR-003](003-dual-runtime-strategy.md)
- Reference: [runtime2/README.md](../../runtime2/README.md) - API documentation
- Reference: [tools/ts-bridge](../../tools/ts-bridge/) - Grammar import tool
- Tests: [golden-tests/](../../golden-tests/) - Tree-sitter parity validation
