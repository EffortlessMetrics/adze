# Precedence Troubleshooting

Adze handles operator precedence and associativity using macro attributes. This guide explains how to resolve common precedence issues in your grammar.

## Precedence Attributes

- **`#[adze::prec_left(n)]`**: Left-associative (e.g., `1 - 2 - 3` is `(1 - 2) - 3`).
- **`#[adze::prec_right(n)]`**: Right-associative (e.g., `x = y = 1` is `x = (y = 1)`).
- **`#[adze::prec(n)]`**: Non-associative (e.g., `x < y < z` is a syntax error).

Higher numbers bind tighter (have higher precedence).

## Common Patterns

### Binary Operators

```rust
#[adze::language]
pub enum Expression {
    Number(#[adze::leaf(pattern = r"\d+")] String),

    #[adze::prec_left(1)]
    Add(Box<Expression>, #[adze::leaf(text = "+")] (), Box<Expression>),

    #[adze::prec_left(2)]
    Multiply(Box<Expression>, #[adze::leaf(text = "*")] (), Box<Expression>),
}
```

In this example, `1 + 2 * 3` correctly parses as `1 + (2 * 3)` because `Multiply` has a higher precedence level (2) than `Add` (1).

## Troubleshooting Steps

### 1. "Ambiguous Parse" or GLR Forks
If your parser is forking unexpectedly, it usually means two rules could match the same input and they have the same precedence.

**Fix**: Ensure all conflicting operators have distinct precedence levels.

### 2. Infinite Recursion
Recursive rules without precedence or terminal tokens can cause the parser generator to fail or the parser to hang.

**Fix**: Ensure recursive calls are wrapped in variants with precedence or are preceded by a unique terminal (like an opening parenthesis).

### 3. Associativity Mismatch
If `1 - 2 - 3` is parsing as `1 - (2 - 3)`, you likely used `prec_right` or a plain `prec` instead of `prec_left`.

**Fix**: Change the attribute to `#[adze::prec_left(n)]`.

## Debugging

To see how Adze is resolving conflicts:

1. Enable artifact emission: `export ADZE_EMIT_ARTIFACTS=true`
2. Build your project: `cargo build`
3. Inspect `adze_debug_{grammar}.log` in your system's temp directory. Look for "Conflict" entries to see which rules are competing.
