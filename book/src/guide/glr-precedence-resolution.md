# GLR Precedence Resolution: A How-To Guide

This guide explains how to effectively use precedence rules with rust-sitter's GLR parser to handle operator precedence and resolve ambiguous grammars.

## Understanding GLR Precedence

Traditional LR parsers eliminate conflicts by choosing one action (shift or reduce). GLR parsers preserve multiple actions but order them by precedence, enabling both correct disambiguation AND handling of inherently ambiguous grammars.

### Key Concepts

1. **Action Preservation**: GLR keeps both conflicting actions in the action table
2. **Precedence Ordering**: Higher precedence actions are tried first
3. **Graceful Fallback**: If the preferred action fails, alternatives are explored
4. **Conflict Resolution**: Common conflicts (shift/reduce, reduce/reduce) are resolved automatically

## Setting Up Operator Precedence

### Basic Arithmetic Example

```rust
#[rust_sitter::grammar("arithmetic")]
mod grammar {
    #[rust_sitter::language]
    pub enum Expression {
        Number(Number),
        Binary(BinaryOp),
    }

    // Lower precedence (looser binding)
    #[rust_sitter::prec_left(1)]
    pub struct Addition {
        pub left: Box<Expression>,
        #[rust_sitter::leaf(text = "+")]
        _op: (),
        pub right: Box<Expression>,
    }

    #[rust_sitter::prec_left(1)]
    pub struct Subtraction {
        pub left: Box<Expression>,
        #[rust_sitter::leaf(text = "-")]
        _op: (),
        pub right: Box<Expression>,
    }

    // Higher precedence (tighter binding)
    #[rust_sitter::prec_left(2)]
    pub struct Multiplication {
        pub left: Box<Expression>,
        #[rust_sitter::leaf(text = "*")]
        _op: (),
        pub right: Box<Expression>,
    }

    #[rust_sitter::prec_left(2)]
    pub struct Division {
        pub left: Box<Expression>,
        #[rust_sitter::leaf(text = "/")]
        _op: (),
        pub right: Box<Expression>,
    }

    // Highest precedence (tightest binding)
    #[rust_sitter::prec_right(3)]
    pub struct Exponentiation {
        pub base: Box<Expression>,
        #[rust_sitter::leaf(text = "^")]
        _op: (),
        pub exponent: Box<Expression>,
    }

    pub struct Number {
        #[rust_sitter::leaf(pattern = r"\d+")]
        pub value: (),
    }
}
```

## How GLR Resolves Precedence Conflicts

### Example: `1 + 2 * 3`

When parsing this expression, the GLR parser encounters a conflict at the `*` token:

```
1 + 2 * 3
      ^ Conflict: Should we reduce `1 + 2` or shift `*`?
```

**Traditional LR Solution**: Choose one action, eliminate the other
**GLR Solution**: Keep both actions, order by precedence

```rust
// GLR action table for this state/symbol:
action_table[state_add][MULTIPLY] = vec![
    Action::Reduce(multiply_rule),  // Precedence 2 - preferred
    Action::Shift(add_state)        // Precedence 1 - fallback
];
```

**Result**: Parser tries `Reduce(multiply_rule)` first, leading to:
```rust
Add {
    left: Number(1),
    right: Mul {
        left: Number(2),
        right: Number(3)
    }
}
// Represents: 1 + (2 * 3) ✅
```

## Associativity Control

### Left Associativity: `prec_left`

```rust
#[rust_sitter::prec_left(1)]
pub struct Subtraction { /* ... */ }
```

**Effect**: `1 - 2 - 3` parses as `(1 - 2) - 3`

### Right Associativity: `prec_right`

```rust
#[rust_sitter::prec_right(3)]
pub struct Exponentiation { /* ... */ }
```

**Effect**: `2 ^ 3 ^ 4` parses as `2 ^ (3 ^ 4)`

### Non-Associative: `prec`

```rust
#[rust_sitter::prec(5)]
pub struct Comparison { /* ... */ }
```

**Effect**: `a == b == c` produces a parse error (prevents chaining)

## Complex Precedence Scenarios

### Multiple Operator Types

```rust
// Logical operators (lowest precedence)
#[rust_sitter::prec_left(1)]
Or(Box<Expr>, (), Box<Expr>),

#[rust_sitter::prec_left(2)]
And(Box<Expr>, (), Box<Expr>),

// Comparison operators (middle precedence, non-associative)
#[rust_sitter::prec(3)]
Equal(Box<Expr>, (), Box<Expr>),

#[rust_sitter::prec(3)]
LessThan(Box<Expr>, (), Box<Expr>),

// Arithmetic operators (higher precedence)
#[rust_sitter::prec_left(4)]
Add(Box<Expr>, (), Box<Expr>),

#[rust_sitter::prec_left(5)]
Multiply(Box<Expr>, (), Box<Expr>),

// Unary operators (highest precedence)
#[rust_sitter::prec(6)]
Negate((), Box<Expr>),
```

### Expression with Mixed Operators

Input: `!a + b * c == d && e || f`

GLR parsing with precedence produces:
```rust
Or {
    left: And {
        left: Equal {
            left: Add {
                left: Negate((), Identifier("a")),
                right: Mul {
                    left: Identifier("b"),
                    right: Identifier("c")
                }
            },
            right: Identifier("d")
        },
        right: Identifier("e")
    },
    right: Identifier("f")
}
```

## Troubleshooting Precedence Issues

### Common Problem: Ambiguous Precedence

```rust
// ❌ Problem: Same precedence for different operator types
#[rust_sitter::prec_left(1)]
Add(Box<Expr>, (), Box<Expr>),

#[rust_sitter::prec_left(1)]  // Same as addition!
LessThan(Box<Expr>, (), Box<Expr>),
```

**Solution**: Use different precedence levels:
```rust
#[rust_sitter::prec_left(2)]   // Arithmetic first
Add(Box<Expr>, (), Box<Expr>),

#[rust_sitter::prec(1)]        // Comparison second (and non-associative)
LessThan(Box<Expr>, (), Box<Expr>),
```

### Debugging Precedence Conflicts

Enable GLR debugging to see action resolution:

```bash
RUST_LOG=debug cargo build
```

Look for log messages like:
```
GLR: State 42, token PLUS has actions: [Reduce(mul_rule:prec=2), Shift(add_state:prec=1)]
GLR: Choosing Reduce(mul_rule) due to higher precedence
```

## Best Practices

### 1. Use Meaningful Precedence Gaps

```rust
// ✅ Good: Leave room for expansion
#[rust_sitter::prec_left(10)]   // Addition
#[rust_sitter::prec_left(20)]   // Multiplication  
#[rust_sitter::prec_right(30)]  // Exponentiation

// ❌ Bad: No room for new operators
#[rust_sitter::prec_left(1)]
#[rust_sitter::prec_left(2)]
#[rust_sitter::prec_left(3)]
```

### 2. Group Related Operators

```rust
// Comparison operators: precedence 10-19
#[rust_sitter::prec(10)] Equal(/* ... */),
#[rust_sitter::prec(10)] NotEqual(/* ... */),
#[rust_sitter::prec(10)] LessThan(/* ... */),
#[rust_sitter::prec(10)] GreaterThan(/* ... */),

// Arithmetic operators: precedence 20-29
#[rust_sitter::prec_left(20)] Add(/* ... */),
#[rust_sitter::prec_left(20)] Subtract(/* ... */),
#[rust_sitter::prec_left(25)] Multiply(/* ... */),
#[rust_sitter::prec_left(25)] Divide(/* ... */),
```

### 3. Test Precedence Behavior

```rust
#[test]
fn test_operator_precedence() {
    let parser = create_parser();
    
    // Test multiplication binds tighter than addition
    let tree = parser.parse("1 + 2 * 3").unwrap();
    assert_eq!(
        tree,
        Add {
            left: Number(1),
            right: Mul {
                left: Number(2),
                right: Number(3)
            }
        }
    );
    
    // Test right associativity of exponentiation
    let tree = parser.parse("2 ^ 3 ^ 4").unwrap();
    assert_eq!(
        tree,
        Pow {
            base: Number(2),
            exponent: Pow {
                base: Number(3),
                exponent: Number(4)
            }
        }
    );
}
```

### 4. Document Precedence Decisions

```rust
/// Arithmetic expression grammar with C-style operator precedence:
/// 
/// Precedence levels (highest to lowest):
/// - 60: Unary operators (!, -, +)
/// - 50: Multiplicative (*, /, %)
/// - 40: Additive (+, -)  
/// - 30: Relational (<, >, <=, >=)
/// - 20: Equality (==, !=)
/// - 10: Logical AND (&&)
/// - 5:  Logical OR (||)
pub enum Expression {
    // Implementation...
}
```

## Advanced: Dynamic Precedence

For languages with context-sensitive precedence (like C++ templates), GLR's action preservation enables sophisticated disambiguation:

```rust
// Multiple parsing strategies preserved
#[rust_sitter::prec(1)]
TemplateParams(/* ... */),     // Parse < > as template brackets

#[rust_sitter::prec(2)]  
Comparison(/* ... */),         // Parse < > as comparison operators

// GLR explores both interpretations, context determines winner
```

## Migration from Tree-sitter

When converting Tree-sitter grammars with `prec.left()` and `prec.right()`:

```javascript
// Tree-sitter grammar.js
prec.left(1, seq($.expr, '+', $.expr))
prec.right(2, seq($.base, '^', $.exp))
```

Becomes:
```rust
// rust-sitter
#[rust_sitter::prec_left(1)]
Add(Box<Expr>, (), Box<Expr>),

#[rust_sitter::prec_right(2)]
Pow(Box<Expr>, (), Box<Expr>),
```

## Summary

GLR precedence resolution provides:

- **Correct operator precedence** without grammar modification
- **Ambiguity handling** for complex language constructs  
- **Graceful error recovery** when precedence rules are insufficient
- **Debugging transparency** through action table inspection

The key insight: GLR doesn't eliminate conflicts, it orders them intelligently, enabling both correctness and flexibility in grammar design.