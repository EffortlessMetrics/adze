# GLR Engine Contract Specification

**Status**: Phase 3.1 - Core GLR Runtime Implementation
**Component**: GLREngine (glr_engine.rs)
**Dependencies**: ParseTable, Token stream
**Purpose**: Handle GLR parsing with fork/merge on conflicts

---

## Overview

The GLR Engine is the core component that executes GLR parsing using a ParseTable. It handles:
- Multiple parallel parser stacks (Graph Structured Stack - GSS)
- Forking on conflicts (shift/reduce, reduce/reduce)
- Merging identical stacks
- Parse forest construction

**Design Principle**: Simple, correct implementation first; optimize later.

---

## Core Data Structures

### GLREngine

```rust
pub struct GLREngine {
    /// Reference to the parse table (from glr-core)
    parse_table: &'static ParseTable,

    /// Current parser stacks (GSS nodes)
    /// Each stack represents one possible parse path
    stacks: Vec<ParserStack>,

    /// Parse forest accumulator
    /// Stores all parse tree nodes from all paths
    forest: ParseForest,

    /// Configuration limits
    config: GLRConfig,
}
```

**Invariants**:
- `stacks.len() >= 1` (at least one active stack)
- `stacks.len() <= config.max_forks` (fork limit not exceeded)
- All stacks in `stacks` are "active" (not yet completed or failed)

---

### ParserStack

```rust
pub struct ParserStack {
    /// Stack of states (LR parser state stack)
    states: Vec<StateId>,

    /// Stack of parse tree nodes corresponding to states
    nodes: Vec<ForestNodeId>,

    /// Unique ID for this stack (for merging detection)
    id: StackId,
}
```

**Invariants**:
- `states.len() == nodes.len() + 1` (one more state than node, initial state has no node)
- `states.is_empty()` is false (stack always has at least initial state)
- Stack top is `states.last().unwrap()`

---

### ParseForest

```rust
pub struct ParseForest {
    /// All nodes in the forest
    nodes: Vec<ForestNode>,

    /// Root nodes (one per successful parse)
    roots: Vec<ForestNodeId>,
}

pub struct ForestNode {
    /// Symbol produced by this node
    symbol: SymbolId,

    /// Children of this node
    children: Vec<ForestNodeId>,

    /// Byte range in input
    range: Range<usize>,
}
```

**Invariants**:
- `roots` contains IDs of complete parse trees
- All node IDs in `children` are valid indices into `nodes`
- Byte ranges are consistent (children within parent range)

---

### GLRConfig

```rust
pub struct GLRConfig {
    /// Maximum number of parallel stacks
    pub max_forks: usize,  // Default: 1000

    /// Maximum forest nodes
    pub max_forest_nodes: usize,  // Default: 10000
}
```

---

## Core Algorithm

### High-Level Flow

```
1. Initialize: Create initial stack with state 0
2. For each token:
   a. For each active stack:
      - Get actions from parse table
      - If single action: Apply it
      - If multiple actions (conflict): Fork
   b. Apply all actions (shift/reduce)
   c. Merge identical stacks
   d. Check termination (all stacks accept or error)
3. Return parse forest
```

### Pseudocode

```
function parse(tokens: &[Token]) -> Result<ParseForest, ParseError>
    stacks = [ParserStack::new(state_0)]
    forest = ParseForest::new()

    for token in tokens:
        new_stacks = []

        for stack in stacks:
            state = stack.top_state()
            actions = parse_table[state][token.kind]

            if actions.is_empty():
                # Error: no valid action
                continue

            for action in actions:
                match action:
                    Shift(next_state):
                        new_stack = stack.clone()
                        node = forest.add_terminal(token)
                        new_stack.push(next_state, node)
                        new_stacks.push(new_stack)

                    Reduce(rule_id):
                        new_stack = perform_reduce(stack, rule_id, &forest)
                        new_stacks.push(new_stack)

                    Accept:
                        forest.add_root(stack.top_node())

        stacks = merge_identical_stacks(new_stacks)

        if stacks.is_empty():
            return Err(ParseError::SyntaxError)

    return Ok(forest)
```

---

## API Contract

### GLREngine::new()

```rust
pub fn new(
    parse_table: &'static ParseTable,
    config: GLRConfig,
) -> Self
```

**Contract**:
- **Preconditions**:
  - `parse_table` satisfies ParseTable invariants
  - `config.max_forks > 0`
  - `config.max_forest_nodes > 0`

- **Postconditions**:
  - Engine initialized with one empty stack (state 0)
  - Forest is empty
  - Ready to parse

---

### GLREngine::parse()

```rust
pub fn parse(
    &mut self,
    tokens: &[Token],
) -> Result<ParseForest, ParseError>
```

**Contract**:
- **Preconditions**:
  - `tokens` is a valid token stream (ends with EOF token)
  - Engine is in initial state (or reset)

- **Postconditions**:
  - Returns `Ok(forest)` if parsing succeeds
  - `forest.roots.len() >= 1` (at least one parse tree)
  - All parse trees in forest are valid

- **Error Conditions**:
  - `ParseError::SyntaxError`: No valid parse (all stacks failed)
  - `ParseError::TooManyForks`: Fork limit exceeded
  - `ParseError::ForestTooLarge`: Node limit exceeded

**Performance**:
- Time: O(n³) worst case, O(n) average for practical grammars
- Space: O(n * forks) for stacks, O(n²) for forest worst case

---

### GLREngine::reset()

```rust
pub fn reset(&mut self)
```

**Contract**:
- Clears all stacks and forest
- Returns engine to initial state
- Allows reuse of engine for multiple parses

---

## Conflict Handling

### Shift/Reduce Conflict

**Scenario**: Action cell contains `[Shift(s1), Reduce(r1)]`

**Behavior**:
1. Clone current stack
2. Apply Shift to original stack → fork 1
3. Apply Reduce to cloned stack → fork 2
4. Both forks continue independently

**Example**:
```
Input: "1 + 2" on lookahead "+"
State 4: [Shift(3), Reduce(rule: expr → expr + expr)]

Fork 1: Shift to state 3 (continue building "(1 + 2) + ...")
Fork 2: Reduce to expr (complete "1 + 2" first)
```

### Reduce/Reduce Conflict

**Scenario**: Action cell contains `[Reduce(r1), Reduce(r2)]`

**Behavior**:
1. Clone stack for each reduce action
2. Apply each reduce to its clone
3. All reductions continue as separate forks

---

## Stack Merging

### When to Merge

Two stacks are **identical** if:
- Same state stack: `stack1.states == stack2.states`
- Different node stacks are OK (representing different parse paths)

### Merging Algorithm

```rust
fn merge_identical_stacks(stacks: Vec<ParserStack>) -> Vec<ParserStack> {
    let mut merged: HashMap<Vec<StateId>, ParserStack> = HashMap::new();

    for stack in stacks {
        let key = stack.states.clone();
        if let Some(existing) = merged.get_mut(&key) {
            // Merge: combine forest nodes (create packed node)
            existing.merge_with(stack);
        } else {
            merged.insert(key, stack);
        }
    }

    merged.into_values().collect()
}
```

**Benefit**: Reduces exponential explosion of forks

---

## Forest Construction

### Terminal Nodes

**Contract**:
```rust
fn add_terminal(&mut self, token: &Token) -> ForestNodeId
```

- Creates leaf node for token
- Stores byte range from token
- Returns node ID

### Nonterminal Nodes (Reduce)

**Contract**:
```rust
fn add_nonterminal(
    &mut self,
    symbol: SymbolId,
    children: Vec<ForestNodeId>,
    range: Range<usize>,
) -> ForestNodeId
```

- Creates internal node for reduced production
- Children are popped from stack
- Range spans all children

### Packed Nodes (Ambiguity)

When two different parses produce same nonterminal at same position:

```rust
struct PackedNode {
    alternatives: Vec<Vec<ForestNodeId>>,  // Multiple child sequences
}
```

**Example**: `1 + 2 + 3` can be `(1 + 2) + 3` or `1 + (2 + 3)`
- Both produce `expr` spanning bytes 0-5
- Packed node stores both child sequences

---

## Error Handling

### Error Recovery Strategy

**Phase 3.1 (MVP)**: No error recovery
- If all stacks fail → `ParseError::SyntaxError`
- Return first error encountered

**Future (Phase 3.2+)**:
- Panic mode recovery
- Error productions
- Partial parse trees

### Error Contexts

```rust
pub enum ParseError {
    SyntaxError {
        position: usize,
        expected: Vec<SymbolId>,
        found: Option<Token>,
    },
    TooManyForks {
        limit: usize,
        attempted: usize,
    },
    ForestTooLarge {
        node_count: usize,
        limit: usize,
    },
}
```

---

## Testing Strategy

### Unit Tests

```rust
#[test]
fn test_glr_engine_initialization() {
    let engine = GLREngine::new(&PARSE_TABLE, GLRConfig::default());
    assert_eq!(engine.stacks.len(), 1);
    assert_eq!(engine.stacks[0].states, vec![StateId(0)]);
}

#[test]
fn test_fork_on_shift_reduce_conflict() {
    // Setup: ParseTable with S/R conflict in state 4
    let engine = GLREngine::new(&CONFLICT_TABLE, GLRConfig::default());

    // Parse to conflict state
    // Verify: stacks.len() == 2 after fork
}

#[test]
fn test_merge_identical_stacks() {
    // Create two stacks with same state sequence
    // Verify: merged to single stack with combined nodes
}
```

### Integration Tests

```rust
#[test]
fn test_parse_ambiguous_expression() {
    let engine = GLREngine::new(&AMBIGUOUS_EXPR_TABLE, GLRConfig::default());
    let tokens = tokenize("1 + 2 + 3");

    let forest = engine.parse(&tokens).unwrap();

    assert!(forest.roots.len() >= 1);
    // For ambiguous grammar, may have multiple roots or packed nodes
}

#[test]
fn test_parse_unambiguous_expression() {
    let engine = GLREngine::new(&UNAMBIGUOUS_TABLE, GLRConfig::default());
    let tokens = tokenize("1 + 2");

    let forest = engine.parse(&tokens).unwrap();

    assert_eq!(forest.roots.len(), 1);  // Single parse
}
```

---

## Performance Contracts

### Time Complexity

- **Best case**: O(n) - No conflicts, acts like LR parser
- **Average case**: O(n log n) - Few conflicts, efficient merging
- **Worst case**: O(n³) - Highly ambiguous grammar

### Space Complexity

- **Stacks**: O(forks * n) where forks ≤ max_forks
- **Forest**: O(n²) worst case for packed nodes
- **Total**: O(max_forks * n + n²)

### Practical Limits

```rust
impl Default for GLRConfig {
    fn default() -> Self {
        Self {
            max_forks: 1000,           // Allow up to 1000 parallel paths
            max_forest_nodes: 10_000,  // 10K nodes ≈ 1MB
        }
    }
}
```

---

## Implementation Phases

### Phase 1: Basic Structure (Current)

- [x] Define data structures
- [x] Write contract specification
- [ ] Implement GLREngine struct
- [ ] Implement single-token processing (no conflicts)

### Phase 2: Conflict Handling

- [ ] Implement fork logic
- [ ] Implement stack merging
- [ ] Test with shift/reduce conflicts

### Phase 3: Forest Construction

- [ ] Implement forest data structure
- [ ] Handle packed nodes (ambiguity)
- [ ] Convert forest to Tree

### Phase 4: Integration

- [ ] Wire into Parser::parse()
- [ ] End-to-end tests
- [ ] Performance benchmarking

---

## Security Considerations

### Resource Limits

**Fork Bomb Prevention**:
```rust
if self.stacks.len() >= self.config.max_forks {
    return Err(ParseError::TooManyForks {
        limit: self.config.max_forks,
        attempted: self.stacks.len() + new_forks,
    });
}
```

**Memory Exhaustion Prevention**:
```rust
if self.forest.nodes.len() >= self.config.max_forest_nodes {
    return Err(ParseError::ForestTooLarge {
        node_count: self.forest.nodes.len(),
        limit: self.config.max_forest_nodes,
    });
}
```

### Stack Overflow Prevention

- Limit maximum stack depth (implicit via forest node limit)
- Use iterative algorithms (avoid recursion where possible)

---

## Open Questions

1. **Tokenization**: Should GLREngine accept raw bytes or tokens?
   - **Decision**: Accept tokens (separation of concerns)
   - Parser handles tokenization, engine handles parsing

2. **Forest representation**: SPPF vs naive forest?
   - **Decision**: Start with naive (simpler), optimize later
   - SPPF (Shared Packed Parse Forest) is more compact but complex

3. **Error recovery**: Include in Phase 3.1 or defer?
   - **Decision**: Defer to Phase 3.2
   - MVP: fail fast on errors

---

## References

- [Tomita's GLR Algorithm](https://en.wikipedia.org/wiki/GLR_parser)
- [SPPF: Shared Packed Parse Forest](https://doi.org/10.1016/j.scico.2009.12.001)
- [Efficient GLR Parsing](https://doi.org/10.1145/69622.357187)
- [PHASE_3_PURE_RUST_GLR_RUNTIME.md](./PHASE_3_PURE_RUST_GLR_RUNTIME.md)
- [GLR_PARSER_API_CONTRACT.md](./GLR_PARSER_API_CONTRACT.md)

---

**Status**: Contract Specification Complete
**Next**: Implement GLREngine struct and basic parsing logic
**Phase**: 3.1 - Core GLR Runtime Implementation

