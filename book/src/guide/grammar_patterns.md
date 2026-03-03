# Grammar Design Patterns

This chapter covers practical design patterns for building grammars with adze.
Each pattern is illustrated with working examples drawn from real grammars in the
repository.

## Operator Precedence

When a grammar has multiple operators, you need precedence levels to control
which operators bind more tightly. Assign higher numbers to operators that
should bind first.

### Standard arithmetic precedence

```rust
#[adze::grammar("calc")]
mod grammar {
    #[adze::language]
    #[derive(PartialEq, Eq, Debug)]
    pub enum Expression {
        Number(#[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())] i32),

        // Level 1 – additive (loosest)
        #[adze::prec_left(1)]
        Add(Box<Expression>, #[adze::leaf(text = "+")] (), Box<Expression>),

        #[adze::prec_left(1)]
        Sub(Box<Expression>, #[adze::leaf(text = "-")] (), Box<Expression>),

        // Level 2 – multiplicative
        #[adze::prec_left(2)]
        Mul(Box<Expression>, #[adze::leaf(text = "*")] (), Box<Expression>),

        #[adze::prec_left(2)]
        Div(Box<Expression>, #[adze::leaf(text = "/")] (), Box<Expression>),

        // Level 3 – exponentiation (tightest)
        #[adze::prec_right(3)]
        Pow(Box<Expression>, #[adze::leaf(text = "^")] (), Box<Expression>),
    }

    #[adze::extra]
    struct Whitespace {
        #[adze::leaf(pattern = r"\s")]
        _whitespace: (),
    }
}
```

With these levels `1 + 2 * 3` parses as `1 + (2 * 3)` because multiplication
(level 2) binds tighter than addition (level 1).

### Tip: leave gaps between levels

Use increments of 10 instead of 1 so you can insert new levels later without
renumbering everything:

```rust
#[adze::prec_left(10)]  // additive
Add(/* ... */),

#[adze::prec_left(20)]  // multiplicative
Mul(/* ... */),

#[adze::prec_right(30)] // exponentiation
Pow(/* ... */),
```

## Associativity

Associativity decides how operators of the **same** precedence level group.

### Left associativity (`prec_left`)

Most arithmetic operators are left-associative. `1 - 2 - 3` becomes `(1 - 2) - 3`:

```rust
#[adze::prec_left(1)]
Sub(Box<Expression>, #[adze::leaf(text = "-")] (), Box<Expression>),
```

### Right associativity (`prec_right`)

Assignment and exponentiation are typically right-associative. `2 ^ 3 ^ 4`
becomes `2 ^ (3 ^ 4)`:

```rust
#[adze::prec_right(3)]
Pow(Box<Expression>, #[adze::leaf(text = "^")] (), Box<Expression>),
```

### Non-associative (`prec`)

Comparison operators are often non-associative. Chaining `a == b == c` should
produce a parse error rather than silently grouping:

```rust
#[adze::prec(5)]
Equal(Box<Expression>, #[adze::leaf(text = "==")] (), Box<Expression>),
```

### Combining multiple tiers

A real-world expression grammar often combines all three:

```rust
// Logical (lowest)
#[adze::prec_left(1)]
Or(Box<Expr>, #[adze::leaf(text = "||")] (), Box<Expr>),

#[adze::prec_left(2)]
And(Box<Expr>, #[adze::leaf(text = "&&")] (), Box<Expr>),

// Comparison (non-associative)
#[adze::prec(3)]
Equal(Box<Expr>, #[adze::leaf(text = "==")] (), Box<Expr>),

// Arithmetic
#[adze::prec_left(4)]
Add(Box<Expr>, #[adze::leaf(text = "+")] (), Box<Expr>),

#[adze::prec_left(5)]
Mul(Box<Expr>, #[adze::leaf(text = "*")] (), Box<Expr>),
```

## Recursive Grammars

Recursive types let you describe nested structures. Use `Box<T>` to break the
size cycle that Rust would otherwise reject.

### Direct recursion through an enum

The arithmetic grammar above is directly recursive — each `Expression` variant
contains `Box<Expression>`.

### Mutual recursion

Statements and expressions often refer to each other:

```rust
#[adze::language]
#[derive(Debug)]
pub enum Statement {
    IfThen(
        #[adze::leaf(text = "if")] (),
        Box<Expr>,
        #[adze::leaf(text = "then")] (),
        Box<Statement>,
    ),
    IfThenElse(
        #[adze::leaf(text = "if")] (),
        Box<Expr>,
        #[adze::leaf(text = "then")] (),
        Box<Statement>,
        #[adze::leaf(text = "else")] (),
        Box<Statement>,
    ),
    ExprStmt(Box<Expr>),
}
```

### Lambda calculus — a compact recursive grammar

This example shows recursive enums with keyword literals and backslash-dot
abstraction syntax:

```rust
#[adze::grammar("lambda")]
mod grammar {
    #[adze::language]
    #[derive(PartialEq, Eq, Debug, Clone)]
    pub enum Expr {
        Var(#[adze::leaf(pattern = r"[a-z][a-z0-9]*")] String),

        Abs(
            #[adze::leaf(text = r"\")] (),
            #[adze::leaf(pattern = r"[a-z][a-z0-9]*")] String,
            #[adze::leaf(text = ".")] (),
            Box<Expr>,
        ),

        #[adze::prec_left(1)]
        App(Box<Expr>, Box<Expr>),

        Paren(
            #[adze::leaf(text = "(")] (),
            Box<Expr>,
            #[adze::leaf(text = ")")] (),
        ),
    }

    #[adze::extra]
    struct Whitespace {
        #[adze::leaf(pattern = r"\s")]
        _whitespace: (),
    }
}
```

## Optional and Vec Fields

### Optional fields with `Option<T>`

Wrap a field in `Option` when the element may be absent:

```rust
pub struct Function {
    pub name: Identifier,
    pub params: Option<ParameterList>,
    pub return_type: Option<Type>,
}
```

You can also make leaf tokens optional:

```rust
pub struct Language {
    #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
    v: Option<i32>,
    #[adze::leaf(text = "_")]
    _sep: (),
    #[adze::leaf(text = ".")]
    _dot: Option<()>,
}
```

With this grammar `_`, `1_`, `_.`, and `1_.` are all valid inputs.

### Repeated fields with `Vec<T>`

Use `Vec` for zero-or-more repetitions:

```rust
pub struct IniFile {
    entries: Vec<Entry>,
}
```

For one-or-more, add `non_empty`:

```rust
#[adze::repeat(non_empty = true)]
#[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
numbers: Vec<i32>,
```

### Delimited lists

Combine `#[adze::delimited]` with `Vec` for separator-separated lists:

```rust
#[adze::repeat(non_empty = true)]
#[adze::delimited(
    #[adze::leaf(text = ",")]
    ()
)]
#[adze::leaf(pattern = r"[a-zA-Z_]\w*", transform = |v| v.to_string())]
items: Vec<String>,
```

This accepts `alpha, beta, gamma` but rejects a trailing comma.

### Optional elements inside a list

When a list element itself is optional, use `Vec<Option<T>>`:

```rust
#[adze::delimited(
    #[adze::leaf(text = ",")]
    ()
)]
#[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
numbers: Vec<Option<i32>>,
```

This lets you parse `1,, 2` where the middle slot is empty.

## Enums for Choice Rules

Enums map naturally to grammar alternatives. Each variant is one possible
production.

### Simple alternatives

```rust
#[derive(Debug)]
pub enum Entry {
    Section(Section),
    Pair(Pair),
    Comment(Comment),
}
```

### Inline leaf alternatives

For simple token choices you can use a regex instead of a full enum:

```rust
pub struct Statement {
    #[adze::leaf(pattern = r"(let|const|var)")]
    keyword: (),
    decl: Declaration,
}
```

### Tagged enums with operator variants

The arithmetic grammar earlier shows this pattern: each enum variant carries its
own literal operator token and precedence annotation.

### Nesting enums

Enums can be nested. For example, separate `Statement` and `Expression` enums
where `Statement` has a variant wrapping `Expression`:

```rust
pub enum Statement {
    Assign(Assignment),
    Expr(Expression),
    Return(ReturnStmt),
}

pub enum Expression {
    Number(Number),
    Binary(BinaryOp),
    Call(FnCall),
}
```

## Error Recovery Patterns

### Whitespace and comments as extras

Defining `#[adze::extra]` for whitespace (and optionally comments) lets the
parser skip over them automatically, which prevents many trivial parse failures:

```rust
#[adze::extra]
struct Whitespace {
    #[adze::leaf(pattern = r"\s")]
    _whitespace: (),
}
```

### Handling parse results gracefully

Always match on `Result` to avoid panicking on invalid input:

```rust
match grammar::parse(input) {
    Ok(tree) => process(tree),
    Err(errs) => {
        for e in &errs {
            eprintln!("parse error: {:?}", e);
        }
    }
}
```

### Defensive span extraction

When working with parsed `Spanned` values, use the safe API to avoid panics on
truncated input:

```rust
match span.try_slice_str(source) {
    Ok(text) => Some(text.to_string()),
    Err(SpanError::OutOfBounds { .. }) => {
        // Extract what we can
        if span.span.0 < source.len() {
            Some(source[span.span.0..].to_string())
        } else {
            None
        }
    }
    Err(_) => None,
}
```

### GLR ambiguity as a recovery tool

With adze's GLR backend, ambiguous grammars do not cause hard failures. The
parser explores multiple interpretations and picks the highest-precedence one.
This means a slightly ambiguous grammar will still parse, giving you time to
refine rather than blocking all progress.

## Performance Tips

### Keep grammars small and focused

Each additional rule increases the parse table size. Split unrelated sub-languages
into separate grammar modules when possible.

### Prefer `Vec` over deep recursion

A grammar like `List → Item List | ε` creates a deeply nested AST.
Using `Vec<Item>` produces a flat list and a smaller parse table:

```rust
// ❌ Deep recursion
pub enum List {
    Cons(Item, Box<List>),
    Nil,
}

// ✅ Flat repetition
pub struct List {
    items: Vec<Item>,
}
```

### Use `#[adze::extra]` for whitespace

Handling whitespace inside every rule bloats the grammar. Declare it once as an
extra token and the parser skips it everywhere:

```rust
#[adze::extra]
struct Whitespace {
    #[adze::leaf(pattern = r"\s")]
    _whitespace: (),
}
```

### Avoid overly broad regex patterns

A pattern like `r".*"` matches everything and causes conflicts. Be as specific
as possible:

```rust
// ❌ Too broad
#[adze::leaf(pattern = r".+")]
token: (),

// ✅ Specific
#[adze::leaf(pattern = r"[a-zA-Z_]\w*")]
identifier: (),
```

### Use snapshot tests to catch regressions

The `insta` crate is used throughout the example grammars for snapshot testing.
This catches accidental changes to parse tree shapes:

```rust
#[test]
fn parse_simple() {
    insta::assert_debug_snapshot!(grammar::parse("1 + 2"));
}
```

### Enable artifact output for debugging

Set `ADZE_EMIT_ARTIFACTS=true` to inspect the generated grammar JSON and
understand what the tool produced:

```bash
ADZE_EMIT_ARTIFACTS=true cargo build 2>&1
# Artifacts appear in target/debug/build/<crate>-<hash>/out/
```

## Common Pitfalls and How to Avoid Them

### 1. Left recursion without `prec_left`

Direct left recursion in an enum without a precedence annotation produces
infinite loops or parse failures. Always annotate recursive binary operators:

```rust
// ❌ Infinite loop risk
pub enum Expr {
    Add(Box<Expr>, Box<Expr>),
}

// ✅ Annotated
#[adze::prec_left(1)]
Add(Box<Expr>, #[adze::leaf(text = "+")] (), Box<Expr>),
```

### 2. Forgetting `Box` for recursive types

Rust requires indirection for recursive types. Omitting `Box` causes a compile
error about infinite size:

```rust
// ❌ Compile error: recursive type has infinite size
pub enum Expr {
    Add(Expr, Expr),
}

// ✅ Box breaks the cycle
pub enum Expr {
    Add(Box<Expr>, Box<Expr>),
}
```

### 3. Multiple precedence attributes on one rule

Only one of `prec`, `prec_left`, or `prec_right` may appear per rule.
The grammar processor will reject duplicates with a clear error message:

```rust
// ❌ Error: multiple precedence attributes
#[adze::prec(1)]
#[adze::prec_left(2)]
pub struct Conflict { /* ... */ }
```

### 4. Non-integer precedence values

Precedence must be a `u32` integer literal. Strings, floats, and constants are
rejected:

```rust
// ❌ Rejected
#[adze::prec("high")]
#[adze::prec(3.14)]

// ✅ Valid
#[adze::prec(10)]
```

### 5. Ambiguous grammars without precedence

If two operators share a precedence level and associativity unintentionally, the
parser may pick an unexpected grouping. Compare operators should usually have
their own distinct level:

```rust
// ❌ Same level — "a + b < c" is ambiguous
#[adze::prec_left(1)]
Add(/* ... */),
#[adze::prec_left(1)]
LessThan(/* ... */),

// ✅ Separate levels
#[adze::prec_left(2)]
Add(/* ... */),
#[adze::prec(1)]
LessThan(/* ... */),
```

### 6. Missing `#[adze::extra]` for whitespace

Without an extra-token declaration the parser requires exact whitespace
placement, which is almost never what you want:

```rust
// ❌ Only parses "1+2", not "1 + 2"
// (no whitespace handling)

// ✅ Parses "1+2", "1 + 2", "1  +  2", etc.
#[adze::extra]
struct Whitespace {
    #[adze::leaf(pattern = r"\s")]
    _whitespace: (),
}
```

### 7. Trailing separators in delimited lists

`#[adze::delimited]` with `#[adze::repeat(non_empty = true)]` rejects trailing
separators. If you need to allow them, use `Vec<Option<T>>` for the element type
so the final empty slot is legal.

### 8. Overly complex leaf transforms

Keep `transform` closures simple. Complex logic should live in a separate
function to keep the grammar readable:

```rust
// ❌ Hard to read inline
#[adze::leaf(
    pattern = r#""([^"\\]|\\.)*""#,
    transform = |s| {
        s[1..s.len()-1]
            .replace("\\n", "\n")
            .replace("\\t", "\t")
            .replace("\\\"", "\"")
    }
)]
pub value: String,

// ✅ Extract into a helper
fn unescape(s: &str) -> String {
    s[1..s.len()-1]
        .replace("\\n", "\n")
        .replace("\\t", "\t")
        .replace("\\\"", "\"")
}

#[adze::leaf(pattern = r#""([^"\\]|\\.)*""#, transform = unescape)]
pub value: String,
```

## Quick Reference

| Pattern | Attribute / Type | Example |
|---|---|---|
| Zero-or-more | `Vec<T>` | `entries: Vec<Entry>` |
| One-or-more | `#[adze::repeat(non_empty = true)]` | `items: Vec<Item>` |
| Optional | `Option<T>` | `ret: Option<Type>` |
| Delimited list | `#[adze::delimited(...)]` | `args: Vec<Expr>` |
| Left-associative op | `#[adze::prec_left(N)]` | `Add(Box<E>, (), Box<E>)` |
| Right-associative op | `#[adze::prec_right(N)]` | `Pow(Box<E>, (), Box<E>)` |
| Non-associative op | `#[adze::prec(N)]` | `Eq(Box<E>, (), Box<E>)` |
| Skip whitespace | `#[adze::extra]` | `struct Whitespace { ... }` |
| Exact token | `#[adze::leaf(text = "...")]` | `_plus: ()` |
| Pattern token | `#[adze::leaf(pattern = r"...")]` | `name: ()` |
| Transform token | `transform = \|v\| ...` | `value: i32` |

## Next Steps

- [Grammar Definition](grammar-definition.md) — full attribute reference
- [GLR Precedence Resolution](glr-precedence-resolution.md) — deep dive on GLR conflict handling
- [Error Recovery](error-recovery.md) — robust parsing of malformed input
- [Performance Optimization](performance.md) — tuning large grammars
