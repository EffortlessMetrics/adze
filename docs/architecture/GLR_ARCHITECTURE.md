# GLR Parser Architecture

**Version**: 1.0.0
**Date**: 2025-11-20
**Status**: Production Ready
**Type**: Explanation (Diataxis)

---

## Table of Contents

1. [Overview](#overview)
2. [Why GLR Parsing?](#why-glr-parsing)
3. [Architecture Layers](#architecture-layers)
4. [Multi-Action Cells](#multi-action-cells)
5. [Runtime Forking Mechanism](#runtime-forking-mechanism)
6. [Forest-to-Tree Conversion](#forest-to-tree-conversion)
7. [Performance Characteristics](#performance-characteristics)
8. [Comparison with LR Parsing](#comparison-with-lr-parsing)
9. [Implementation Details](#implementation-details)
10. [References](#references)

---

## Overview

The rust-sitter GLR (Generalized LR) parser is a production-ready parsing system that extends traditional LR parsing to handle **ambiguous grammars** and **parsing conflicts** through runtime forking and parse forest generation.

### Key Features

- **Conflict Preservation**: Shift/reduce and reduce/reduce conflicts are preserved in parse tables rather than eliminated
- **Runtime Forking**: Parser dynamically forks when encountering conflicts, exploring all valid parse paths
- **Parse Forests**: Multiple parse trees can coexist in a compact graph structure
- **Deterministic Selection**: A single tree is deterministically extracted from the forest
- **Tree-sitter Compatibility**: GLR-produced trees are 100% compatible with standard Tree-sitter API

### Design Philosophy

The GLR implementation follows rust-sitter's core principles:

1. **Pure Rust**: No C dependencies, enabling WASM compilation and memory safety
2. **Contract-First**: BDD/TDD methodology with comprehensive acceptance criteria
3. **Performance-Conscious**: Production-ready performance with established baselines
4. **API Compatibility**: Seamless integration with existing Tree-sitter ecosystem

---

## Why GLR Parsing?

### The Problem: Parsing Conflicts

Traditional LR parsers require grammars to be **unambiguous** - each input has exactly one valid parse. However, many real-world languages have inherent ambiguities:

**Dangling Else Problem** (Classic Example):
```
if expr then if expr then stmt else stmt
                                    ^
                         Which 'if' does 'else' belong to?
```

**Expression Grammars** (Without Precedence):
```
1 - 2 * 3
    ^
Is this (1 - 2) * 3 = -3  or  1 - (2 * 3) = -5 ?
```

### Traditional Solutions and Limitations

**1. Grammar Rewriting** ❌
- Rewrites grammar to eliminate ambiguity
- Makes grammar complex and unreadable
- Doesn't work for inherently ambiguous languages

**2. Precedence Declarations** ⚠️
- Annotates operators with precedence/associativity
- **Works for many cases but limited to shift/reduce conflicts**
- Doesn't handle all ambiguities (e.g., C++ template parsing)

**3. GLR Parsing** ✅
- Handles **all types of ambiguity** gracefully
- Preserves grammar readability
- Explores all valid interpretations
- **This is what rust-sitter implements**

### Benefits of GLR in rust-sitter

1. **Grammar Simplicity**: Write grammars naturally without complex rewrites
2. **Language Support**: Parse languages like C++, Rust, and others with context-dependent syntax
3. **Research Applications**: Foundation for grammar inference and language analysis
4. **Error Recovery**: Multiple parse paths improve error recovery strategies

---

## Architecture Layers

The GLR implementation is organized into distinct layers, each with clear responsibilities:

```
┌─────────────────────────────────────────────────────────┐
│                    User Application                      │
│               (uses rust-sitter-runtime)                 │
└─────────────────────────────────────────────────────────┘
                          ▲
                          │ Tree API
                          │ (100% compatible)
┌─────────────────────────────────────────────────────────┐
│                  runtime2/src/parser.rs                  │
│            GLR Parser API & Feature Routing              │
│  (Parser::new, parse, Tree API, Language validation)    │
└─────────────────────────────────────────────────────────┘
                          ▲
                          │
         ┌────────────────┴────────────────┐
         │                                  │
         ▼                                  ▼
┌──────────────────┐              ┌──────────────────────┐
│  engine.rs       │              │    builder.rs        │
│  GLR Engine      │─────────────▶│  Forest→Tree         │
│  Fork & Merge    │   Forest     │   Conversion         │
└──────────────────┘              └──────────────────────┘
         ▲                                  │
         │ Parse Table                      │ Tree
         │                                  ▼
┌─────────────────────────────────────────────────────────┐
│                    tree.rs / node.rs                     │
│          Tree & Node API (cursor, traversal)             │
└─────────────────────────────────────────────────────────┘
         ▲
         │ Parse Tables (.parsetable)
         │
┌─────────────────────────────────────────────────────────┐
│              rust-sitter-tablegen                        │
│   Parse Table Generation & Compression                   │
│  (ActionCell encoding, multi-action compression)         │
└─────────────────────────────────────────────────────────┘
         ▲
         │ LR(1) Automaton
         │
┌─────────────────────────────────────────────────────────┐
│              rust-sitter-glr-core                        │
│    LR(1) Automaton Construction & Conflict Detection    │
│  (FIRST/FOLLOW, item sets, conflict preservation)       │
└─────────────────────────────────────────────────────────┘
         ▲
         │ Grammar IR
         │
┌─────────────────────────────────────────────────────────┐
│                rust-sitter-ir                            │
│         Grammar Intermediate Representation              │
│  (Productions, symbols, precedence, associativity)       │
└─────────────────────────────────────────────────────────┘
```

### Layer Responsibilities

#### 1. **rust-sitter-ir** (Grammar IR)
- **Purpose**: Represent grammars with GLR-specific metadata
- **Key Components**:
  - `Grammar`: Productions, symbols, tokens
  - `Precedence`: Left, right, non-associative annotations
  - `Associativity`: Ordering rules for conflict resolution
  - `SymbolId`: Unique identifier for each grammar symbol

#### 2. **rust-sitter-glr-core** (Parser Generation)
- **Purpose**: Build LR(1) automaton and detect conflicts
- **Key Components**:
  - `FirstFollowSets`: FIRST/FOLLOW set computation
  - `build_lr1_automaton()`: Canonical LR(1) collection construction
  - `detect_conflicts()`: Identify shift/reduce and reduce/reduce conflicts
  - **Critical Innovation**: Preserves conflicts instead of eliminating them

#### 3. **rust-sitter-tablegen** (Table Generation)
- **Purpose**: Compress LR(1) automaton into efficient runtime tables
- **Key Components**:
  - `ActionCell`: Vec<Vec<Vec<Action>>> - multi-action cells
  - `compress_action_table()`: Tree-sitter compatible compression
  - `.parsetable` format: Bincode serialization of compressed tables
  - **Performance**: Bit-for-bit compatible with Tree-sitter C implementation

#### 4. **runtime2** (GLR Runtime)
- **Purpose**: Execute GLR parsing and produce trees
- **Key Components**:
  - `Parser`: User-facing API
  - `Engine`: GLR fork/merge logic
  - `Builder`: Forest-to-tree conversion
  - `Tree`/`Node`: Tree-sitter compatible tree API

---

## Multi-Action Cells

### The Core Innovation

Traditional LR parsers use **single-action cells**:
```rust
// LR Parser: ONE action per state/symbol
Vec<Vec<Action>>  // [state][symbol] → ONE Action
```

GLR parsers use **multi-action cells**:
```rust
// GLR Parser: MULTIPLE actions per state/symbol
Vec<Vec<Vec<Action>>>  // [state][symbol] → Vec<Action>
```

### Action Types

```rust
pub enum Action {
    /// Move to state N and consume token
    Shift(StateId),

    /// Reduce using production P
    Reduce(ProductionId),

    /// Accept the input (parsing complete)
    Accept,

    /// Parsing error
    Error,
}
```

### Example: Dangling Else

**Grammar:**
```
S → if expr then S
S → if expr then S else S
S → stmt
```

**State 0 Action Table** (simplified):
```
State 0, Symbol 'if':   [Shift(1)]        ✓ Unambiguous
State 0, Symbol 'stmt': [Shift(2)]        ✓ Unambiguous

State 4, Symbol 'else': [Shift(6), Reduce(Prod0)]  ← CONFLICT!
                         ^^^^^^^^  ^^^^^^^^^^^^^^
                         Shift the   OR   Reduce to S
                        'else' token      (if-then)
```

**LR Parser Behavior**: ❌ Error or forced choice (precedence declaration required)

**GLR Parser Behavior**: ✅ Forks into two parallel parse states, explores both paths

### Conflict Preservation Strategy

When building the LR(1) automaton, conflicts are **preserved** rather than eliminated:

1. **Detect Conflict**: Identify when multiple actions are valid
2. **Order Actions**: Use precedence/associativity to order (not eliminate)
3. **Create Multi-Action Cell**: Store all valid actions in the cell
4. **Runtime Decision**: Parser decides at runtime based on input

**Key Insight**: Precedence doesn't eliminate actions; it **orders** them for deterministic selection.

---

## Runtime Forking Mechanism

### Graph-Structured Stack (GSS)

The GLR parser uses a **Graph-Structured Stack** instead of a simple stack:

```
Traditional LR Stack (Single Path):
┌──────┐
│ S5   │  ← Top
├──────┤
│ S3   │
├──────┤
│ S1   │
├──────┤
│ S0   │  ← Bottom
└──────┘

GLR GSS (Multiple Paths):
        ┌──────┐
    ┌──▶│ S5   │
    │   └──────┘
┌───┴──┐        ┌──────┐
│ S3   │───────▶│ S6   │  ← Two parallel stacks
└───┬──┘        └──────┘
    │   ┌──────┐
    └──▶│ S4   │
        └──────┘
         ...
┌──────┐
│ S0   │  ← Shared base
└──────┘
```

### Forking Process

**Step 1: Encounter Conflict**
```
Input: "if expr then if expr then stmt else stmt"
Position: ─────────────────────────────────^
State: 4
Actions: [Shift(6), Reduce(Prod0)]  ← FORK!
```

**Step 2: Create Parallel Stacks**
```rust
// Before fork (1 stack):
stacks = [Stack { states: [0, 1, 3, 4], pos: 30 }]

// After fork (2 stacks):
stacks = [
    Stack { states: [0, 1, 3, 4, 6], pos: 35 },  // Shifted 'else'
    Stack { states: [0, 1], pos: 30 },           // Reduced to S
]
```

**Step 3: Continue Parsing**
- Each stack processes the next token independently
- Stacks may **merge** if they reach the same state at the same position
- Stacks may **die** if they encounter syntax errors
- Valid stacks produce parse forest nodes

### Merge Conditions

Two stacks **merge** when:
1. Same state ID
2. Same input position
3. Equivalent history (same parse subtree up to this point)

**Example:**
```
Stack A: [0, 1, 3, 5] at position 20
Stack B: [0, 2, 4, 5] at position 20
         └─────────┘    └──────────┘
         Different       Same state
         history         Same position
                         → MERGE!
```

Merged stacks share a single node in the GSS, reducing memory usage.

### Forest Node Creation

When reducing, the parser creates **forest nodes**:

```rust
pub struct ForestNode {
    pub symbol: SymbolId,       // Nonterminal symbol
    pub start: usize,            // Start byte position
    pub end: usize,              // End byte position
    pub children: Vec<NodeRef>,  // Child nodes (may have alternatives)
}
```

**Alternatives**: Multiple children lists represent different parse interpretations.

---

## Forest-to-Tree Conversion

### Parse Forest Structure

A **parse forest** is a compact DAG (Directed Acyclic Graph) representing multiple parse trees:

```
Parse Forest (Ambiguous):
       S
      /|\
     / | \
    /  |  \
   A   B   C    ← Multiple interpretations

Parse Tree (Selected):
       S
      / \
     A   B      ← Deterministic selection
```

### Selection Algorithm

The `Builder` (runtime2/src/builder.rs) converts forests to trees:

1. **Traverse Forest**: DFS traversal from root
2. **Select Alternative**: Choose first valid alternative (deterministic)
3. **Build Tree Nodes**: Create Tree-sitter compatible nodes
4. **Track Positions**: Maintain byte ranges and line/column positions

```rust
pub fn build_tree(forest: Forest, source: &[u8]) -> Result<Tree, ParseError> {
    let mut builder = Builder::new(source);

    // Start from root forest node
    let root_ref = forest.root();

    // Recursively build tree
    let tree_root = builder.build_node(root_ref, &forest)?;

    Ok(Tree::new(tree_root, forest.language()))
}
```

### Performance Monitoring

The builder includes instrumentation for performance analysis:

```bash
export RUST_SITTER_LOG_PERFORMANCE=true
cargo run
```

Output:
```
[PERF] Forest→Tree conversion:
  Nodes: 127
  Depth: 15
  Time: 245 µs
  Average: 1.93 µs/node
```

---

## Performance Characteristics

### Time Complexity

| Operation | LR Parser | GLR Parser | Notes |
|-----------|-----------|------------|-------|
| Best Case | O(n) | O(n) | Unambiguous grammar |
| Average Case | O(n) | O(n²) | Local ambiguities |
| Worst Case | O(n) | O(n³) | Highly ambiguous grammar |

**n** = input length

### Space Complexity

| Component | LR Parser | GLR Parser | Notes |
|-----------|-----------|------------|-------|
| Parse Table | O(states × symbols) | O(states × symbols × conflicts) | Multi-action cells |
| Stack | O(depth) | O(depth × forks) | GSS grows with ambiguity |
| Parse Tree | O(nodes) | O(nodes × alternatives) | Forest may be larger |

### Real-World Performance (Benchmarks)

From `docs/PERFORMANCE_BASELINE.md`:

**Python Grammar (273 symbols, 57 fields):**
```
Parse "def foo(): pass":
  LR:  ~50 µs
  GLR: ~75 µs  (1.5x slower) ✓ Acceptable

Parse 1000-line Python file:
  LR:  ~5 ms
  GLR: ~8 ms   (1.6x slower) ✓ Production-ready
```

**Dangling-Else Grammar:**
```
Parse "if expr then if expr then stmt else stmt":
  GLR: ~120 µs  (includes forking overhead)
  Forks: 2
  Merges: 1
```

### Optimization Strategies

1. **Early Pruning**: Eliminate invalid stacks as soon as possible
2. **Aggressive Merging**: Merge stacks eagerly to reduce memory
3. **Lazy Forest Construction**: Build forest nodes only when needed
4. **Table Compression**: Use Tree-sitter's compression for compact tables

---

## Comparison with LR Parsing

### Capability Comparison

| Feature | LR Parser | GLR Parser |
|---------|-----------|------------|
| **Unambiguous Grammars** | ✅ Excellent | ✅ Excellent |
| **Ambiguous Grammars** | ❌ Fails or requires rewrites | ✅ Handles naturally |
| **Precedence Support** | ✅ Yes | ✅ Yes (plus conflict preservation) |
| **Error Recovery** | ⚠️ Limited | ✅ Better (multiple paths) |
| **Performance** | ✅ O(n) guaranteed | ⚠️ O(n²) typical, O(n³) worst |
| **Memory Usage** | ✅ Minimal | ⚠️ Higher (GSS + forest) |
| **Implementation Complexity** | ⚠️ Moderate | ⚠️⚠️ High |

### When to Use Each

**Use LR Parser when:**
- ✅ Grammar is provably unambiguous
- ✅ Performance is critical (hard real-time systems)
- ✅ Memory is extremely constrained
- ✅ Standard Tree-sitter workflow is sufficient

**Use GLR Parser when:**
- ✅ Grammar has inherent ambiguities
- ✅ Language has context-dependent syntax (C++, Rust)
- ✅ Grammar readability is important
- ✅ Error recovery quality matters
- ✅ Research/experimentation with language design

### Migration Path

rust-sitter supports **both** LR and GLR modes:

```rust
// LR Mode (runtime/)
use rust_sitter::Parser;
let parser = Parser::new();  // Uses Tree-sitter C runtime

// GLR Mode (runtime2/)
use rust_sitter_runtime::Parser;
let parser = Parser::new();  // Uses pure-Rust GLR engine
```

See: [RUNTIME_MODES.md](../specs/RUNTIME_MODES.md) for architectural decision.

---

## Implementation Details

### Key Files and Responsibilities

#### Parse Table Generation

**File**: `tablegen/src/compress.rs`
```rust
pub fn compress_action_table(
    automaton: &LR1Automaton,
    grammar: &Grammar,
) -> Result<ActionTable, TableGenError> {
    // Convert LR(1) automaton to multi-action cells
    let mut cells = vec![vec![vec![]; symbol_count]; state_count];

    for (state_id, state) in automaton.states.iter() {
        for (symbol, actions) in &state.actions {
            // CRITICAL: Store ALL actions (GLR)
            cells[state_id][symbol] = actions.clone();
        }
    }

    // Compress using Tree-sitter algorithm
    compress_cells(cells)
}
```

#### Runtime Forking

**File**: `runtime2/src/engine.rs`
```rust
pub fn parse_step(&mut self, token: Token) -> Result<(), ParseError> {
    let actions = self.get_actions(self.state, token.symbol);

    match actions.len() {
        0 => Err(ParseError::UnexpectedToken),
        1 => self.execute_action(actions[0]),  // Fast path
        _ => self.fork_and_execute(actions),   // GLR fork!
    }
}

fn fork_and_execute(&mut self, actions: &[Action]) -> Result<(), ParseError> {
    // Create parallel stacks for each action
    for action in actions {
        let forked_stack = self.stack.clone();
        self.stacks.push(forked_stack);
        self.execute_action_on_stack(action, self.stacks.last_mut().unwrap())?;
    }

    // Merge identical stacks
    self.merge_stacks();

    Ok(())
}
```

#### Forest-to-Tree Conversion

**File**: `runtime2/src/builder.rs`
```rust
pub fn build_node(
    &mut self,
    node_ref: NodeRef,
    forest: &Forest,
) -> Result<NodeId, BuildError> {
    let forest_node = forest.get_node(node_ref)?;

    // Select first alternative (deterministic)
    let children = &forest_node.children[0];

    // Recursively build children
    let child_ids: Vec<NodeId> = children
        .iter()
        .map(|child_ref| self.build_node(*child_ref, forest))
        .collect::<Result<_, _>>()?;

    // Create tree node
    let tree_node = TreeNode {
        symbol: forest_node.symbol,
        start: forest_node.start,
        end: forest_node.end,
        children: child_ids,
    };

    Ok(self.arena.insert(tree_node))
}
```

### Feature Flags

The GLR implementation is controlled by feature flags:

```toml
[features]
default = []
pure-rust-glr = ["rust-sitter-glr-core", "rust-sitter-tablegen"]
glr-core = ["pure-rust-glr"]  # Alias for GLR engine
serialization = ["serde", "bincode"]  # For .parsetable format
```

### Environment Variables

- `RUST_SITTER_LOG_PERFORMANCE=true`: Enable performance logging
- `RUST_SITTER_EMIT_ARTIFACTS=true`: Output generated tables for debugging
- `RUST_TEST_THREADS=N`: Control test concurrency (default: 2)

---

## References

### Academic Papers

- **Tomita's Algorithm** (1985): Original GLR parsing algorithm
- **Scott & Johnstone** (2006): Right Nulled GLR parsers
- **Aycock & Horspool** (2002): Practical Earley parsing

### Tree-sitter Documentation

- [Tree-sitter GLR Support](https://tree-sitter.github.io/tree-sitter/creating-parsers#parsing-conflicts)
- [LR Parsing Theory](https://tree-sitter.github.io/tree-sitter/creating-parsers#lr-parsing)

### Internal Documentation

- [GLR_V1_COMPLETION_CONTRACT.md](../specs/GLR_V1_COMPLETION_CONTRACT.md) - GLR v1 acceptance criteria
- [TREE_API_COMPATIBILITY_CONTRACT.md](../specs/TREE_API_COMPATIBILITY_CONTRACT.md) - Tree API validation
- [BDD_GLR_CONFLICT_PRESERVATION.md](../plans/BDD_GLR_CONFLICT_PRESERVATION.md) - BDD test scenarios
- [PERFORMANCE_BASELINE.md](../PERFORMANCE_BASELINE.md) - Performance benchmarks
- [RUNTIME_MODES.md](../specs/RUNTIME_MODES.md) - Dual runtime architecture

### Code References

- **GLR Core**: `glr-core/src/lib.rs` - LR(1) automaton construction
- **Table Generation**: `tablegen/src/compress.rs` - Parse table compression
- **Runtime Engine**: `runtime2/src/engine.rs` - GLR fork/merge logic
- **Tree Builder**: `runtime2/src/builder.rs` - Forest-to-tree conversion
- **Tree API**: `runtime2/src/tree.rs`, `runtime2/src/node.rs` - Tree traversal

---

## Appendix: GLR by Example

### Example: Arithmetic Expression

**Grammar:**
```
Expr → Expr + Expr  [prec_left(1)]
Expr → Expr * Expr  [prec_left(2)]
Expr → number
```

**Input:** `1 + 2 * 3`

**Parse Forest** (before selection):
```
       Expr
      /  |  \
     /   |   \
    +    +    *     ← Two interpretations
   / \  / \  / \
  1   * 1  2  +  3
     / \      / \
    2   3    1   2
```

**Selected Tree** (precedence favors `*` over `+`):
```
    Expr(+)
    /     \
   1    Expr(*)
        /     \
       2       3
```

**Result:** `1 + (2 * 3) = 7` ✓

---

**Document Version**: 1.0.0
**Last Updated**: 2025-11-20
**Next Review**: After GLR v1 production deployment
**Owner**: rust-sitter core team

---

END OF ARCHITECTURE DOCUMENT
