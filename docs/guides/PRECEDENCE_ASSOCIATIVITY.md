# Precedence and Associativity in GLR Parsers

**Version**: 1.0.0
**Date**: 2025-11-20
**Type**: Reference Guide (Diataxis)
**Audience**: Grammar authors using rust-sitter GLR

---

## Table of Contents

1. [Introduction](#introduction)
2. [Precedence Basics](#precedence-basics)
3. [Associativity Basics](#associativity-basics)
4. [GLR-Specific Behavior](#glr-specific-behavior)
5. [Conflict Resolution Rules](#conflict-resolution-rules)
6. [Common Patterns](#common-patterns)
7. [Advanced Topics](#advanced-topics)
8. [Reference Tables](#reference-tables)

---

## Introduction

Precedence and associativity annotations control how rust-sitter's GLR parser resolves **parsing conflicts** when multiple interpretations are valid. This guide explains:

- **What** precedence and associativity mean
- **How** they affect GLR parsing behavior
- **When** to use each type
- **Examples** of common patterns

### Key Concepts

| Concept | Definition | Example |
|---------|------------|---------|
| **Precedence** | Order of operations priority | `*` before `+` in `1 + 2 * 3` |
| **Associativity** | Grouping direction for same precedence | Left: `(1-2)-3`, Right: `1^(2^3)` |
| **Conflict** | Multiple valid actions in parse table | Shift vs Reduce |
| **Multi-Action Cell** | GLR parse table cell with >1 action | `[Shift(6), Reduce(0)]` |

---

## Precedence Basics

### What is Precedence?

**Precedence** determines which operation "binds tighter" when multiple operators are present.

**Example:**
```
Expression: 1 + 2 * 3

Without precedence (ambiguous):
  (1 + 2) * 3 = 9
  1 + (2 * 3) = 7

With precedence (* > +):
  1 + (2 * 3) = 7  ✓ Correct
```

### Precedence Levels

In rust-sitter, precedence is specified with **integer levels**:

```rust
#[derive(Debug)]
pub enum Expr {
    /// Addition: precedence level 1
    #[rust_sitter(prec_left = 1)]
    Add {
        left: Box<Expr>,
        #[rust_sitter(token = "+")]
        _op: (),
        right: Box<Expr>,
    },

    /// Multiplication: precedence level 2
    #[rust_sitter(prec_left = 2)]
    Mul {
        left: Box<Expr>,
        #[rust_sitter(token = "*")]
        _op: (),
        right: Box<Expr>,
    },

    #[rust_sitter(leaf, regex = r"\d+")]
    Number(String),
}
```

**Rules:**
- Higher number = higher precedence (binds tighter)
- `prec_left(2) > prec_left(1)` → Multiply before add
- Same number = same precedence (use associativity)

### Precedence Annotations

rust-sitter provides three precedence annotations:

| Annotation | Meaning | Use Case |
|-----------|---------|----------|
| `prec_left(N)` | Precedence N, left-associative | Most operators: +, -, *, / |
| `prec_right(N)` | Precedence N, right-associative | Exponentiation: ^, Assignment: = |
| `prec(N)` | Precedence N, no associativity | Comparison: <, >, == (non-chainable) |

---

## Associativity Basics

### What is Associativity?

**Associativity** determines how operators of the **same precedence** group when chained.

**Example:**
```
Expression: 10 - 5 - 2

Left-associative:
  (10 - 5) - 2 = 3  ✓ Standard subtraction

Right-associative:
  10 - (5 - 2) = 7  ✗ Incorrect for subtraction
```

### Types of Associativity

#### 1. Left-Associative (`prec_left`)

**Grouping:** Left to right
**Operators:** +, -, *, /, %

```rust
#[rust_sitter(prec_left = 1)]
Sub {
    left: Box<Expr>,
    #[rust_sitter(token = "-")]
    _op: (),
    right: Box<Expr>,
}
```

**Parse Tree for `10 - 5 - 2`:**
```
    Sub
   /   \
  Sub   2
 /   \
10    5

Result: (10 - 5) - 2 = 3
```

#### 2. Right-Associative (`prec_right`)

**Grouping:** Right to left
**Operators:** ^, =, ::

```rust
#[rust_sitter(prec_right = 3)]
Exp {
    left: Box<Expr>,
    #[rust_sitter(token = "^")]
    _op: (),
    right: Box<Expr>,
}
```

**Parse Tree for `2 ^ 3 ^ 4`:**
```
    Exp
   /   \
  2    Exp
      /   \
     3     4

Result: 2 ^ (3 ^ 4) = 2^81
```

#### 3. Non-Associative (`prec`)

**Grouping:** Not allowed to chain
**Operators:** <, >, ==, !=

```rust
#[rust_sitter(prec = 2)]
LessThan {
    left: Box<Expr>,
    #[rust_sitter(token = "<")]
    _op: (),
    right: Box<Expr>,
}
```

**Behavior:**
```
1 < 2 < 3  ← ERROR: Cannot chain non-associative operators
1 < 2      ✓ OK
(1 < 2) && (2 < 3)  ✓ OK (use logical operators)
```

---

## GLR-Specific Behavior

### Key Difference: Conflicts Are Preserved

**Traditional LR Parser:**
- Precedence **eliminates** conflicting actions
- Parse table has single action per cell
- Grammar must be unambiguous after precedence

**GLR Parser (rust-sitter):**
- Precedence **orders** conflicting actions
- Parse table can have multiple actions per cell
- Grammar can remain ambiguous
- **Parser explores all paths at runtime**

### Multi-Action Cells

Example parse table state:

```
State 4, Symbol 'else':
  Actions: [Shift(6), Reduce(Prod0)]
           ^^^^^^^^  ^^^^^^^^^^^^^^
           Priority 2  Priority 1 (lower precedence)

GLR behavior:
1. Try Shift(6) first (higher priority)
2. If Shift fails, backtrack and try Reduce(Prod0)
3. OR fork and explore both paths simultaneously
```

### Precedence as Action Ordering

```rust
// Example: Dangling-else with precedence
#[rust_sitter(prec_left = 2)]  // Higher precedence
IfThenElse { ... }

#[rust_sitter(prec_left = 1)]  // Lower precedence
IfThen { ... }
```

**Effect on parsing `if a then if b then x else y`:**

Without precedence:
```
Actions: [Reduce(IfThen), Shift('else')]  ← Ambiguous
GLR forks and tries both
```

With precedence:
```
Actions: [Shift('else'), Reduce(IfThen)]  ← Ordered by precedence
GLR prefers Shift (matches IfThenElse variant)
```

**Result:** More deterministic parsing, fewer forks, better performance.

---

## Conflict Resolution Rules

### Rule 1: Shift/Reduce Conflicts

**Situation:** Parser can either:
1. Shift: Consume next token
2. Reduce: Complete current production

**Resolution:**
```
if shift_precedence > reduce_precedence:
    prefer shift
elif shift_precedence < reduce_precedence:
    prefer reduce
else:
    use associativity:
        left-assoc  → prefer reduce
        right-assoc → prefer shift
        non-assoc   → error if chained
```

**Example: Expression Parsing**

```rust
// State: Expr + Expr •, lookahead: *
// Actions: Shift(*) [prec 2] or Reduce(Add) [prec 1]

prec(*) = 2 > prec(+) = 1
→ Prefer Shift(*)
→ Result: Expr + (Expr * Expr)  ✓ Correct
```

### Rule 2: Reduce/Reduce Conflicts

**Situation:** Multiple productions can be reduced

**Resolution:**
```
if prod1_precedence > prod2_precedence:
    prefer prod1
elif prod1_precedence < prod2_precedence:
    prefer prod2
else:
    use production order (first declared wins)
```

**Example: Type Disambiguation**

```rust
// Ambiguous: Is "Foo" a type or a value?
#[rust_sitter(prec = 2)]  // Higher precedence
Type { name: String }

#[rust_sitter(prec = 1)]  // Lower precedence
Value { name: String }

// Parser prefers Type variant
```

### Rule 3: GLR Forking Decision

When conflicts remain after precedence/associativity:

```
if runtime_forking_enabled:
    create parallel parse stacks
    explore all valid paths
    merge stacks when they converge
else:
    report ambiguity error
```

---

## Common Patterns

### Pattern 1: Arithmetic Operators

**Standard precedence hierarchy:**

```rust
pub enum Expr {
    // Precedence 1: Additive (lowest)
    #[rust_sitter(prec_left = 1)]
    Add { left: Box<Expr>, right: Box<Expr> },

    #[rust_sitter(prec_left = 1)]
    Sub { left: Box<Expr>, right: Box<Expr> },

    // Precedence 2: Multiplicative
    #[rust_sitter(prec_left = 2)]
    Mul { left: Box<Expr>, right: Box<Expr> },

    #[rust_sitter(prec_left = 2)]
    Div { left: Box<Expr>, right: Box<Expr> },

    // Precedence 3: Exponentiation (highest)
    #[rust_sitter(prec_right = 3)]  // Right-associative!
    Exp { left: Box<Expr>, right: Box<Expr> },

    // Precedence 4: Unary (even higher)
    #[rust_sitter(prec_right = 4)]
    Neg { operand: Box<Expr> },

    #[rust_sitter(leaf, regex = r"\d+")]
    Number(String),
}
```

**Result:**
```
1 + 2 * 3      → 1 + (2 * 3) = 7
10 - 5 - 2     → (10 - 5) - 2 = 3
2 ^ 3 ^ 4      → 2 ^ (3 ^ 4) = 2^81
-5 * 3         → (-5) * 3 = -15
```

### Pattern 2: Comparison Operators

**Non-associative to prevent chaining:**

```rust
pub enum Expr {
    #[rust_sitter(prec = 2)]  // Non-associative
    LessThan { left: Box<Expr>, right: Box<Expr> },

    #[rust_sitter(prec = 2)]
    GreaterThan { left: Box<Expr>, right: Box<Expr> },

    #[rust_sitter(prec = 2)]
    Equal { left: Box<Expr>, right: Box<Expr> },

    // Lower precedence logical operators
    #[rust_sitter(prec_left = 1)]
    And { left: Box<Expr>, right: Box<Expr> },
}
```

**Behavior:**
```
1 < 2 < 3      → ERROR: Cannot chain comparisons
1 < 2 && 2 < 3 → OK: Use logical AND
```

### Pattern 3: Assignment (Right-Associative)

```rust
pub enum Stmt {
    #[rust_sitter(prec_right = 1)]
    Assign {
        target: Box<Expr>,
        #[rust_sitter(token = "=")]
        _eq: (),
        value: Box<Expr>,
    },
}
```

**Result:**
```
a = b = c  →  a = (b = c)  ✓ Right-to-left
```

### Pattern 4: Dangling-Else

**Prefer shift (bind else to nearest if):**

```rust
pub enum Stmt {
    #[rust_sitter(prec_left = 2)]  // Higher precedence
    IfThenElse {
        #[rust_sitter(keyword = "if")]
        _if: (),
        condition: Box<Expr>,
        #[rust_sitter(keyword = "then")]
        _then: (),
        then_body: Box<Stmt>,
        #[rust_sitter(keyword = "else")]
        _else: (),
        else_body: Box<Stmt>,
    },

    #[rust_sitter(prec_left = 1)]  // Lower precedence
    IfThen {
        #[rust_sitter(keyword = "if")]
        _if: (),
        condition: Box<Expr>,
        #[rust_sitter(keyword = "then")]
        _then: (),
        body: Box<Stmt>,
    },
}
```

**Result:**
```
if a then if b then x else y
→ if a then (if b then x else y)  ✓ Else binds to inner if
```

---

## Advanced Topics

### Dynamic Precedence

Some languages (Python, Ruby) have **context-dependent precedence**:

```python
# Python: 'not' has lower precedence than comparisons
not x == y  →  not (x == y)

# But inside expressions:
x and not y  →  x and (not y)
```

rust-sitter GLR handles this via **runtime forking**:

```rust
// Both variants exist in grammar
#[rust_sitter(prec_left = 1)]
NotLowPrec { operand: Box<Expr> },

#[rust_sitter(prec_left = 3)]
NotHighPrec { operand: Box<Expr> },

// GLR explores both at runtime
```

### Precedence Climbing

GLR parser doesn't use traditional **precedence climbing** algorithm. Instead:

1. Build LR(1) automaton with all conflicts preserved
2. Use precedence to **order** actions in multi-action cells
3. Runtime explores actions in priority order
4. Backtrack or fork as needed

**Advantage:** More general than precedence climbing, handles any ambiguity.

### Conflict Inspection

**View conflicts during build:**

```bash
export RUST_LOG=rust_sitter=debug
cargo build 2>&1 | grep "conflict"

# Output:
# DEBUG: State 4 shift/reduce conflict:
#   Shift('else') [prec 2] vs Reduce(IfThen) [prec 1]
#   → Ordered: [Shift, Reduce]
```

**Programmatic inspection:**

```rust
// In build script or test
let automaton = build_lr1_automaton(&grammar)?;

for (state_id, state) in &automaton.states {
    for (symbol, actions) in &state.actions {
        if actions.len() > 1 {
            println!("Conflict in state {}, symbol {}: {:?}",
                     state_id, symbol, actions);
        }
    }
}
```

---

## Reference Tables

### Precedence Levels (Convention)

| Level | Operators | Associativity | Example |
|-------|-----------|---------------|---------|
| 0 | Statement separators | N/A | `;`, newline |
| 1 | Logical OR | Left | `\|\|`, `or` |
| 2 | Logical AND | Left | `&&`, `and` |
| 3 | Equality | Non-assoc | `==`, `!=` |
| 4 | Relational | Non-assoc | `<`, `>`, `<=`, `>=` |
| 5 | Bitwise OR | Left | `\|` |
| 6 | Bitwise XOR | Left | `^` |
| 7 | Bitwise AND | Left | `&` |
| 8 | Shift | Left | `<<`, `>>` |
| 9 | Additive | Left | `+`, `-` |
| 10 | Multiplicative | Left | `*`, `/`, `%` |
| 11 | Exponentiation | Right | `^`, `**` |
| 12 | Unary | Right | `-`, `!`, `~` |
| 13 | Member access | Left | `.`, `->`, `::` |
| 14 | Call/Index | Left | `()`, `[]` |

**Note:** These are conventions. Adjust for your language's semantics.

### Associativity Quick Reference

| Operator Type | Associativity | Annotation | Example |
|--------------|---------------|------------|---------|
| Arithmetic | Left | `prec_left` | `1 - 2 - 3` = `(1 - 2) - 3` |
| Exponentiation | Right | `prec_right` | `2 ^ 3 ^ 4` = `2 ^ (3 ^ 4)` |
| Comparison | Non-assoc | `prec` | `1 < 2 < 3` = ERROR |
| Assignment | Right | `prec_right` | `a = b = c` = `a = (b = c)` |
| Member access | Left | `prec_left` | `a.b.c` = `(a.b).c` |
| Logical AND | Left | `prec_left` | `a && b && c` = `(a && b) && c` |

### Conflict Types

| Conflict | Cause | Resolution Strategy |
|----------|-------|---------------------|
| Shift/Reduce | Operator precedence ambiguity | Compare precedence; use assoc if equal |
| Reduce/Reduce | Multiple productions match | Compare precedence; use order if equal |
| Shift/Shift | Multiple tokens match (tokenizer issue) | Not a parse conflict; fix tokenizer |

---

## Examples

### Example 1: Full Expression Grammar

```rust
#[grammar("expr")]
mod grammar {
    #[derive(Debug, Clone, PartialEq)]
    pub enum Expr {
        // Precedence 1: Logical OR
        #[rust_sitter(prec_left = 1)]
        Or {
            left: Box<Expr>,
            #[rust_sitter(token = "||")]
            _op: (),
            right: Box<Expr>,
        },

        // Precedence 2: Logical AND
        #[rust_sitter(prec_left = 2)]
        And {
            left: Box<Expr>,
            #[rust_sitter(token = "&&")]
            _op: (),
            right: Box<Expr>,
        },

        // Precedence 3: Equality (non-associative)
        #[rust_sitter(prec = 3)]
        Eq {
            left: Box<Expr>,
            #[rust_sitter(token = "==")]
            _op: (),
            right: Box<Expr>,
        },

        // Precedence 4: Additive
        #[rust_sitter(prec_left = 4)]
        Add {
            left: Box<Expr>,
            #[rust_sitter(token = "+")]
            _op: (),
            right: Box<Expr>,
        },

        // Precedence 5: Multiplicative
        #[rust_sitter(prec_left = 5)]
        Mul {
            left: Box<Expr>,
            #[rust_sitter(token = "*")]
            _op: (),
            right: Box<Expr>,
        },

        // Precedence 6: Unary (right-associative)
        #[rust_sitter(prec_right = 6)]
        Not {
            #[rust_sitter(token = "!")]
            _op: (),
            operand: Box<Expr>,
        },

        // Leaf: Number
        #[rust_sitter(leaf, regex = r"\d+")]
        Number(String),

        // Leaf: Identifier
        #[rust_sitter(leaf, regex = r"[a-zA-Z_][a-zA-Z0-9_]*")]
        Ident(String),
    }
}
```

**Test cases:**
```rust
#[test]
fn test_precedence() {
    assert_parse("1 + 2 * 3", "Add(1, Mul(2, 3))");
    assert_parse("a && b || c", "Or(And(a, b), c)");
    assert_parse("!a && b", "And(Not(a), b)");
}
```

### Example 2: Statement Grammar with Dangling-Else

```rust
#[grammar("stmt")]
mod grammar {
    #[derive(Debug)]
    pub enum Stmt {
        // Precedence 2: If-Then-Else (higher)
        #[rust_sitter(prec_left = 2)]
        IfThenElse {
            #[rust_sitter(keyword = "if")]
            _if: (),
            condition: Box<Expr>,
            #[rust_sitter(keyword = "then")]
            _then: (),
            then_body: Box<Stmt>,
            #[rust_sitter(keyword = "else")]
            _else: (),
            else_body: Box<Stmt>,
        },

        // Precedence 1: If-Then (lower)
        #[rust_sitter(prec_left = 1)]
        IfThen {
            #[rust_sitter(keyword = "if")]
            _if: (),
            condition: Box<Expr>,
            #[rust_sitter(keyword = "then")]
            _then: (),
            body: Box<Stmt>,
        },

        #[rust_sitter(leaf)]
        Simple(String),
    }

    #[derive(Debug)]
    #[rust_sitter(leaf)]
    pub struct Expr(String);
}
```

**Parsing `if a then if b then x else y`:**

```
Conflict at 'else':
  Shift('else')   [prec 2]  ← IfThenElse variant
  Reduce(IfThen)  [prec 1]  ← IfThen variant

Resolution: Shift wins (higher precedence)
Result: if a then (if b then x else y)  ✓
```

---

## Best Practices

### 1. Use Conventional Precedence Levels

✅ **Do:** Follow language conventions
```rust
#[rust_sitter(prec_left = 10)]  // Multiplicative
Mul { ... }

#[rust_sitter(prec_left = 9)]   // Additive
Add { ... }
```

❌ **Don't:** Use arbitrary numbers
```rust
#[rust_sitter(prec_left = 42)]  // Unclear priority
Mul { ... }

#[rust_sitter(prec_left = 13)]  // Hard to compare
Add { ... }
```

### 2. Document Ambiguities

✅ **Do:** Explain conflict resolution
```rust
/// Dangling-else is resolved by binding 'else' to nearest 'if'.
/// This matches C, Java, and most other languages.
///
/// Precedence: IfThenElse (2) > IfThen (1)
#[rust_sitter(prec_left = 2)]
IfThenElse { ... }
```

### 3. Test Edge Cases

```rust
#[test]
fn test_associativity() {
    // Left-associative subtraction
    assert_eq!(parse("10 - 5 - 2"), "(10 - 5) - 2");

    // Right-associative exponentiation
    assert_eq!(parse("2 ^ 3 ^ 4"), "2 ^ (3 ^ 4)");

    // Non-associative comparison (should error)
    assert!(parse("1 < 2 < 3").is_err());
}
```

### 4. Use GLR Strengths

✅ **Leverage GLR for inherent ambiguities:**
```rust
// GLR can handle both interpretations
#[rust_sitter(prec_left = 1)]
TypeCast { ... }

#[rust_sitter(prec_left = 1)]
FunctionCall { ... }

// Runtime selects based on context
```

---

**Document Version**: 1.0.0
**Last Updated**: 2025-11-20
**See Also**:
- [GLR_ARCHITECTURE.md](../architecture/GLR_ARCHITECTURE.md)
- [GLR_USER_GUIDE.md](./GLR_USER_GUIDE.md)
- [Tree-sitter Precedence Documentation](https://tree-sitter.github.io/tree-sitter/creating-parsers#precedence)

---

END OF REFERENCE GUIDE
