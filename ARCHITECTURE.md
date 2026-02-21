# adze Architecture Overview

A visual guide to how adze components fit together.

---

## System Overview

```
┌─────────────────────────────────────────────────────────────┐
│                    Your Rust Project                         │
│                                                              │
│  src/main.rs                                                 │
│  ┌────────────────────────────────────────┐                 │
│  │ #[adze::grammar("mylang")]      │                 │
│  │ mod grammar {                           │                 │
│  │     #[adze::language]           │                 │
│  │     pub enum Expr { ... }              │                 │
│  │ }                                       │                 │
│  │                                         │                 │
│  │ fn main() {                             │                 │
│  │     let ast = grammar::parse("...");   │                 │
│  │ }                                       │                 │
│  └────────────────────────────────────────┘                 │
│           │                                                  │
│           │ compile time                                    │
│           ▼                                                  │
│  ┌────────────────────────────────────────┐                 │
│  │        build.rs (build script)         │                 │
│  │  adze_tool::build_parsers()    │                 │
│  └────────────────────────────────────────┘                 │
└───────────────│──────────────────────────────────────────────┘
                │
                │ calls
                ▼
┌─────────────────────────────────────────────────────────────┐
│              adze Workspace                           │
│                                                              │
│  ┌──────────────────┐        ┌──────────────────┐           │
│  │ adze-macro│───────▶│ adze-common│           │
│  │  (proc macros)   │        │  (shared utils)   │           │
│  └──────────────────┘        └──────────────────┘           │
│           │                           │                      │
│           │                           │                      │
│           ▼                           ▼                      │
│  ┌──────────────────┐        ┌──────────────────┐           │
│  │ adze-tool │───────▶│  adze-ir  │           │
│  │ (build-time gen) │        │ (IR representation)│          │
│  └──────────────────┘        └──────────────────┘           │
│           │                           │                      │
│           │                           ▼                      │
│           │                  ┌──────────────────┐            │
│           │                  │adze-glr   │            │
│           │                  │      -core       │            │
│           │                  │ (GLR algorithm)  │            │
│           │                  └──────────────────┘            │
│           │                           │                      │
│           │                           ▼                      │
│           │                  ┌──────────────────┐            │
│           │                  │adze-      │            │
│           │                  │   tablegen       │            │
│           │                  │(table compression)│           │
│           │                  └──────────────────┘            │
│           │                           │                      │
│           └───────────────────────────┘                      │
│                           │                                  │
│                           ▼                                  │
│                  ┌──────────────────┐                        │
│                  │  adze     │                        │
│                  │   (runtime)      │                        │
│                  │ - Parser API     │                        │
│                  │ - Tree API       │                        │
│                  │ - Query API      │                        │
│                  └──────────────────┘                        │
│                           │                                  │
└───────────────────────────│──────────────────────────────────┘
                            │
                            │ generates
                            ▼
                    ┌──────────────────┐
                    │  Compiled Parser │
                    │  + Typed AST     │
                    │  in Your App     │
                    └──────────────────┘
```

---

## Grammar Processing Pipeline

```
1. Source Code (Rust with attributes)
   ↓
   #[adze::grammar("name")]
   mod grammar { ... }

2. Macro Expansion (compile time)
   ↓
   adze-macro processes attributes
   → Generates marker traits
   → Validation happens here

3. Build Script Execution (build time)
   ↓
   build.rs calls adze_tool::build_parsers()
   → Extracts grammar from annotated types
   → Converts to Intermediate Representation (IR)

4. IR Processing
   ↓
   adze-ir
   → Grammar optimization
   → Validation
   → Symbol resolution

5. Parser Generation
   ↓
   adze-glr-core
   → Build LR(1) automaton
   → Detect and handle conflicts
   → Generate action/goto tables

6. Table Compression
   ↓
   adze-tablegen
   → Compress parse tables (tree-sitter format)
   → Generate static Language struct
   → FFI compatibility layer

7. Runtime Linking
   ↓
   adze (runtime)
   → Links compressed tables
   → Provides Parser API
   → Returns typed AST

8. Usage in Your Code
   ↓
   let ast = grammar::parse(source);
   → Typed Rust value returned
```

---

## Crate Dependency Graph

```
                        ┌─────────────────┐
                        │   Your Project  │
                        └────────┬────────┘
                                 │
                     ┌───────────┴───────────┐
                     │                       │
          (compile time)              (build time)
                     │                       │
             ┌───────▼───────┐      ┌───────▼──────┐
             │ adze-  │      │ adze- │
             │    macro      │      │     tool     │
             └───────┬───────┘      └───────┬──────┘
                     │                      │
                     └──────┬───────────────┘
                            │
                    ┌───────▼──────┐
                    │ adze- │
                    │   common     │
                    └───────┬──────┘
                            │
                    ┌───────▼──────┐
                    │ adze- │
                    │      ir      │
                    └───────┬──────┘
                            │
                    ┌───────▼──────┐
                    │ adze- │
                    │   glr-core   │
                    └───────┬──────┘
                            │
                    ┌───────▼──────┐
                    │ adze- │
                    │   tablegen   │
                    └───────┬──────┘
                            │
                    ┌───────▼──────┐
                    │ adze  │◀───── (runtime dependency)
                    │  (runtime)   │
                    └──────────────┘
```

---

## Core Concepts

### Two-Phase Processing

**Phase 1: Compile Time (Macros)**
- `#[adze::grammar]` → Marks grammar module
- `#[adze::language]` → Marks root type
- `#[adze::leaf]` → Defines token patterns
- Macros generate marker traits, no parser code yet

**Phase 2: Build Time (build.rs)**
- `build_parsers()` extracts grammar from markers
- Generates actual parser tables
- Compiles into binary

### Pure-Rust vs C Backend

```
Pure-Rust Backend (default, recommended):
┌─────────────┐
│ Your Grammar│
└──────┬──────┘
       │
       ▼
┌──────────────┐     ┌──────────────┐
│ adze- │────▶│ adze- │
│   glr-core   │     │   tablegen   │
└──────────────┘     └──────┬───────┘
                            │
                            ▼
                    ┌──────────────┐
                    │ Compressed   │
                    │ Parse Tables │
                    │ (Pure Rust)  │
                    └──────────────┘
                    → WASM compatible
                    → No C dependencies

C Backend (legacy, tree-sitter compatible):
┌─────────────┐
│ Your Grammar│
└──────┬──────┘
       │
       ▼
┌──────────────┐     ┌──────────────┐
│ adze- │────▶│ grammar.json │
│     tool     │     │(tree-sitter) │
└──────────────┘     └──────┬───────┘
                            │
                            ▼
                    ┌──────────────┐
                    │ tree-sitter  │
                    │     CLI      │
                    └──────┬───────┘
                            │
                            ▼
                    ┌──────────────┐
                    │  parser.c    │
                    │  (compiled)  │
                    └──────────────┘
                    → Requires Node.js
                    → C compiler needed
```

### Parser Runtime Modes

adze supports multiple parser runtime implementations:

| Mode | Runtime File | GLR Support | Status | Implementation |
|------|-------------|-------------|---------|----------------|
| **tree-sitter** | Tree-sitter C runtime | ✅ LR(1) | Stable | Default, uses Tree-sitter's proven C parser |
| **pure-rust** | `runtime/src/pure_parser.rs` | ⚠️ LR only | Stable | Simple LR parser, first-action-only |
| **pure-rust+GLR** | `runtime/src/parser_v4.rs` | ✅ Full GLR | Experimental | True GLR with fork/merge, not default yet |

**Key Architectural Issue** (v0.6.1):
- GLR table generation (`glr-core`, `tablegen`) is **correct** ✅
- Macro-generated grammars call `__private::parse()` which uses `pure_parser.rs` ⚠️
- `pure_parser.rs` only takes the **first action** per state/symbol, ignoring GLR capabilities
- `parser_v4.rs` is a **complete GLR implementation** but not wired as default

**Impact**:
- ❌ Operator associativity may not work correctly in pure-Rust mode
- ❌ Ambiguous grammars requiring GLR fail with pure-Rust
- ✅ Tree-sitter C backend works correctly (recommended for production)

**Resolution Plan**:
- v0.7.0: Wire `parser_v4.rs` as default runtime for macro grammars

---

## GLR Parser Architecture

```
Input Tokens
     │
     ▼
┌────────────────────┐
│  GLR Driver        │
│  - State stacks    │
│  - Fork on conflict│
│  - Merge on join   │
└────────┬───────────┘
         │
         ▼
┌────────────────────┐
│  Action Table      │
│  [state][symbol]   │
│  → Vec<Action>     │  ← Multiple actions per cell (GLR!)
└────────┬───────────┘
         │
    ┌────┴─────┐
    │          │
    ▼          ▼
┌────────┐ ┌────────┐
│ Shift  │ │ Reduce │
└────┬───┘ └───┬────┘
     │         │
     └────┬────┘
          │
          ▼
    ┌──────────┐
    │  GOTO    │
    │  Table   │
    └────┬─────┘
         │
         ▼
    ┌────────────────┐
    │  Parse Forest  │  ← All valid parse trees
    │  - Shared nodes│
    │  - Packed SPPFs│
    └────────────────┘
```

---

## Data Flow Example

Let's trace `grammar::parse("2 + 3")`:

```
1. Build Time (happens once):
   ┌──────────────┐
   │ #[grammar]   │
   │ enum Expr {  │
   │   Number(..) │
   │   Add(..)    │
   │ }            │
   └──────┬───────┘
          │
          ▼
   ┌──────────────┐
   │ build.rs     │
   │ extracts     │
   │ grammar      │
   └──────┬───────┘
          │
          ▼
   ┌──────────────┐
   │ IR Grammar   │
   │ - 2 rules    │
   │ - 3 symbols  │
   └──────┬───────┘
          │
          ▼
   ┌──────────────┐
   │ LR(1) States │
   │ Action Table │
   │ GOTO Table   │
   └──────┬───────┘
          │
          ▼
   ┌──────────────┐
   │ Compressed   │
   │ Static Data  │
   │ in Binary    │
   └──────────────┘

2. Runtime (parse call):
   Input: "2 + 3"
          │
          ▼
   ┌──────────────┐
   │ Tokenize     │
   │ → [2, +, 3]  │
   └──────┬───────┘
          │
          ▼
   ┌──────────────┐
   │ GLR Driver   │
   │ State: [0]   │
   └──────┬───────┘
          │
          ▼
   Token: 2 (Number)
   Action: Shift
          │
          ▼
   ┌──────────────┐
   │ State: [0,3] │
   └──────┬───────┘
          │
          ▼
   Reduce: Number(2)
          │
          ▼
   Token: + (Plus)
   Action: Shift
          │
          ▼
   ┌──────────────┐
   │ State: [0,5] │
   └──────┬───────┘
          │
          ▼
   Token: 3 (Number)
   Action: Shift & Reduce
          │
          ▼
   ┌──────────────┐
   │ AST Built    │
   │ Add(         │
   │   Number(2), │
   │   Number(3)  │
   │ )            │
   └──────────────┘
```

---

## File Organization

```
adze/
├── runtime/              # Runtime library (what you depend on)
│   ├── src/
│   │   ├── lib.rs       # Main API
│   │   ├── parser.rs    # Parser implementation
│   │   ├── tree.rs      # Parse tree API
│   │   └── query.rs     # Query system
│   └── tests/           # Runtime tests
│
├── macro/               # Procedural macros
│   └── src/
│       └── lib.rs       # #[grammar], #[language], etc.
│
├── tool/                # Build-time code generation
│   ├── src/
│   │   ├── lib.rs       # build_parsers() entry point
│   │   └── extract.rs   # Grammar extraction
│   └── tests/           # Tool tests
│
├── common/              # Shared utilities
│   └── src/
│       └── lib.rs       # Common types
│
├── ir/                  # Intermediate Representation
│   └── src/
│       ├── grammar.rs   # Grammar IR
│       └── optimizer.rs # Grammar optimization
│
├── glr-core/            # GLR parser generation
│   ├── src/
│   │   ├── lib.rs       # LR(1) automaton
│   │   └── conflicts.rs # Conflict resolution
│   └── tests/           # GLR tests
│
├── tablegen/            # Table compression
│   ├── src/
│   │   └── compress.rs  # Tree-sitter table format
│   └── tests/           # Compression tests
│
├── example/             # Example grammars
│   ├── src/
│   │   ├── arithmetic.rs
│   │   ├── json.rs
│   │   └── ...
│   └── tests/           # Integration tests
│
├── tools/
│   └── ts-bridge/       # Tree-sitter grammar importer
│
└── docs/                # Documentation
    ├── GETTING_STARTED.md
    └── ...
```

---

## Key Interfaces

### User-Facing API

```rust
// In your code:
use adze::Parser;

// Parse text
let ast = grammar::parse("source code")?;

// Or use Parser directly:
let mut parser = Parser::new();
parser.set_language(grammar::language());
let tree = parser.parse("source", None)?;
```

### Build-Time API

```rust
// In build.rs:
use adze_tool::build_parsers;
use std::path::PathBuf;

fn main() {
    build_parsers(&PathBuf::from("src/main.rs"));
}
```

### Grammar Definition API

```rust
#[adze::grammar("name")]
mod grammar {
    #[adze::language]
    pub enum MyType {
        Variant1(
            #[adze::leaf(pattern = r"...")]
            FieldType
        ),
    }
}
```

---

## Extension Points

### Custom External Scanners

```rust
impl adze::ExternalScanner for MyScanner {
    fn scan(&mut self, lexer: &mut Lexer, valid: &[bool]) -> ScanResult {
        // Custom lexing logic
    }
}
```

### Tree Visitors (coming v0.7.0)

```rust
impl adze::Visitor for MyVisitor {
    fn visit_node(&mut self, node: &Node) {
        // Custom tree traversal
    }
}
```

### Query Predicates (coming v0.7.0)

```rust
let query = compile_query(r#"
    (function_definition
      name: (identifier) @name
      (#eq? @name "main"))
"#)?;
```

---

## Performance Characteristics

### Time Complexity

**Parse Time**: O(n³) worst case (GLR), O(n) typical case (LR)
- Unambiguous grammars: Linear in input size
- Ambiguous grammars: Polynomial (but rare in practice)

**Build Time**: O(states²) for automaton construction
- Happens once at build time
- Cached for subsequent builds

### Space Complexity

**Parse Tables**: O(states × symbols)
- Compressed using tree-sitter algorithm
- Typical compression: 10:1 ratio

**Parse Trees**: O(n) for AST nodes
- Shared subtrees in GLR forest
- Compact representation

---

## Comparison to tree-sitter Architecture

| Component | tree-sitter | adze |
|-----------|-------------|-------------|
| Grammar Language | JavaScript DSL | Rust types |
| Parser Generator | Node.js CLI | Rust build.rs |
| Parser Runtime | C library | Pure Rust |
| Parse Table Format | Custom binary | Compatible + Rust |
| GLR Support | No (LR only) | Yes (full GLR) |
| Incremental Parsing | Mature | In progress |
| Language Bindings | Many languages | Rust-first |

**Compatibility**: adze can import tree-sitter grammars and generate compatible parsers via ts-bridge.

---

## Debug Tips

### View Generated Grammar

```bash
ADZE_EMIT_ARTIFACTS=true cargo build
cat target/debug/build/*/out/grammar.json
```

### Enable Logging

```bash
RUST_LOG=adze=debug cargo run
```

### Profile Performance

```bash
cargo install flamegraph
cargo flamegraph --bin your-app
# Open flamegraph.svg in browser
```

### Inspect Parse Tables

```bash
# With emit_ir! macro in your grammar:
cargo build 2>&1 | grep "IR:"
```

---

## Next Steps

- **Learn the basics**: [QUICK_START.md](./QUICK_START.md)
- **Deep dive**: [docs/GETTING_STARTED.md](./docs/GETTING_STARTED.md)
- **See examples**: [example/src/](./example/src/)
- **Contribute**: [CONTRIBUTING.md](./CONTRIBUTING.md)

---

**Questions?** See [FAQ.md](./FAQ.md) or ask in [GitHub Discussions](https://github.com/EffortlessMetrics/adze/discussions)
