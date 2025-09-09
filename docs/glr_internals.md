# GLR Parser Internals: A Technical Deep Dive

## What is GLR Parsing?

GLR (Generalized LR) parsing extends traditional LR parsing to handle **ambiguous grammars** - grammars where a single input can have multiple valid parse trees. This is crucial for parsing real-world programming languages like C++, Rust, and Python, which have inherent ambiguities.

## The Core Innovation: ActionCells

Traditional LR parsers have a single action per (state, symbol) pair:
```
parse_table[state][symbol] = Shift(next_state) | Reduce(rule)
```

GLR parsers support **multiple actions** per cell:
```rust
// Traditional LR
type Action = Shift(StateId) | Reduce(RuleId);
type ParseTable = Vec<Vec<Action>>;

// GLR with ActionCells
type ActionCell = Vec<Action>;  // Multiple actions possible!
type GLRParseTable = Vec<Vec<ActionCell>>;
```

## How GLR Handles Conflicts

### 1. Fork on Conflict
When the parser encounters multiple actions, it **forks** the parse stack:

```
Input: "x = 1 + 2 * 3"
                    ^
State 42: [Shift(s67), Reduce(r12)]  // Conflict!

Fork into:
  Stack A: Shift to state 67
  Stack B: Reduce by rule 12
```

### 2. Graph-Structured Stack (GSS)
Instead of maintaining separate stacks, GLR uses a **shared graph structure**:

```
     [S0]
       |
     [S1]
      / \
   [S2] [S3]  <- Fork point
     \ /
    [S4]      <- Merge point
```

This saves memory and enables efficient merging.

### 3. Merge on Convergence
When forked paths reach the same (state, position), they **merge**:

```rust
if stack1.state == stack2.state && stack1.position == stack2.position {
    // Merge: both paths lead to the same parse state
    merge_stacks(stack1, stack2);
}
```

## rust-sitter's GLR Implementation

### Action Table Structure
```rust
// In tablegen/compress.rs
pub struct CompressedParseTable {
    action_table: Vec<Vec<Vec<Action>>>,  // 3D: state × symbol × actions
    goto_table: HashMap<(StateId, SymbolId), StateId>,
}
```

### Runtime Fork Logic
```rust
// In glr-core/lib.rs
impl GLRParser {
    fn process_token(&mut self, token: Token) {
        for action_cell in self.get_actions(self.state, token.symbol) {
            if action_cell.len() > 1 {
                // Fork! Create new GSS heads
                for action in action_cell {
                    self.fork_and_apply(action);
                }
            }
        }
    }
}
```

### Forest Construction
When ambiguity persists to the end, GLR produces a **parse forest**:

```rust
pub enum ForestNode {
    Leaf { symbol: SymbolId, text: String },
    Branch { 
        symbol: SymbolId,
        alternatives: Vec<Vec<ForestNode>>  // Multiple derivations
    }
}
```

## Advantages of GLR

1. **Handles Real Ambiguity**: Parses C++ templates, Rust macros, Python indentation
2. **No Grammar Restrictions**: No need to refactor grammars to be LALR(1)
3. **Better Error Recovery**: Multiple paths provide fallback options
4. **Grammar Development**: Easier to prototype without resolving all conflicts

## Performance Considerations

- **Worst Case**: O(n³) for highly ambiguous grammars
- **Typical Case**: O(n) for mostly deterministic grammars
- **Memory**: Shared GSS reduces duplication
- **Optimization**: Precedence/associativity prunes unnecessary forks

## Example: Precedence Disambiguation Fixes

GLR parsing now correctly handles precedence conflicts:

```rust
// Expression: 1 + 2 * 3
// Should parse as: 1 + (2 * 3) due to operator precedence
```

**Before Fix**: Ambiguous parse - both `(1 + 2) * 3` and `1 + (2 * 3)` possible
**After Fix**: Precedence rules correctly disambiguate to `1 + (2 * 3)`

```rust
// GLR action table preserves both actions but orders by precedence:
action_table[state][PLUS_TOKEN] = vec![
    Reduce(multiply_rule),  // Higher precedence - preferred
    Shift(plus_state)       // Lower precedence - fallback
];
```

## Example: Error Recovery Improvements

GLR now handles malformed input gracefully:

```rust
// Input: "1 + + 2" (double plus operator)
```

**Before Fix**: Parser would crash or produce unpredictable results
**After Fix**: Parser recovers by inserting error nodes and continuing:

```rust
// Parse tree includes error recovery:
BinaryOp {
    left: Number(1),
    op: Plus,
    right: ErrorNode {
        children: [Plus, Number(2)]  // Recovered invalid sequence
    }
}
```

## Example: EOF Handling Fixes

Fixed `process_eof()` parameter usage for proper end-of-input handling:

```python
# Empty file or file starting with 'def'?
def foo(): pass
```

**Before Fix**: State 0 had single action, couldn't handle both cases.
**After Fix**: Correct EOF processing enables both interpretations:

```rust
action_table[0][DEF_TOKEN] = vec![
    Shift(state_1),    // Start parsing statement
    Reduce(empty_rule) // Empty module
];
```

Now both interpretations are explored, the correct one survives!

## Further Reading

- [Tomita's Algorithm (1985)](https://en.wikipedia.org/wiki/GLR_parser) - Original GLR paper
- [Tree-sitter's Conflict Resolution](https://tree-sitter.github.io/tree-sitter/creating-parsers#conflicts) 
- [rust-sitter's Multi-Path Architecture](/runtime/src/glr_forest.rs)