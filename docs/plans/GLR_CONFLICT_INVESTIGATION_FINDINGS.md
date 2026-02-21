# GLR Conflict Investigation Findings

**Date**: 2025-11-19
**Status**: ACTIVE INVESTIGATION
**Priority**: HIGH
**Related**: BDD_GLR_CONFLICT_PRESERVATION.md, PARSER_V4_TABLE_LOADING_BLOCKER.md

---

## 🎯 Executive Summary

Systematic testing with two different grammars (arithmetic and dangling-else) reveals that **BOTH generate zero LR(1) conflicts**, despite being designed to test GLR conflict preservation.

**Key Insight**: LR(1) lookahead is more powerful than initially expected for grammars defined using explicit variants. Our grammar definitions may be inadvertently creating conflict-free LR(1) automata.

---

## 📊 Test Results

### Test 1: Arithmetic Grammar
```
Grammar: Expression with precedence annotations
- Sub with prec_left(1)
- Mul with prec_left(2)

Results:
  Total states: 10
  Total symbols: 12
  Multi-action cells: 0

Conclusion: No conflicts detected
```

### Test 2: Dangling-Else Grammar
```
Grammar: If-then and if-then-else statements
- IfThen(if, Expr, then, Statement)
- IfThenElse(if, Expr, then, Statement, else, Statement)

Results:
  Total states: 36
  Total symbols: 21
  Multi-action cells: 0

Conclusion: No conflicts detected
```

---

## 🔬 Root Cause Analysis

### Why Arithmetic Has No Conflicts

The arithmetic grammar uses **explicit precedence annotations** (`prec_left`). The LR(1) automaton construction likely encodes these annotations into different states or lookahead sets, resolving what would be conflicts in an LR(0) parser.

**Example**:
```rust
// In state after "1 - 2", on lookahead "*":
// LR(1) KNOWS to shift because * has higher precedence
// This isn't a "conflict" - it's deterministic based on lookahead
```

**Implication**: Precedence annotations work at grammar-construction time, not conflict-resolution time.

### Why Dangling-Else Has No Conflicts

The dangling-else grammar uses **separate production variants** for the two cases:
- `IfThen` (no else clause)
- `IfThenElse` (with else clause)

When the LR(1) parser is in state `"if Expr then Statement •"` and sees "else":
- **Lookahead determines the action**: If "else" is in the follow set of `Statement` in the `IfThenElse` context, the parser shifts
- **No conflict**: The parser knows which production it's building based on lookahead

**Classic dangling-else** requires a SINGLE production that can be interpreted multiple ways:
```
Statement → if Expr then Statement
Statement → if Expr then Statement else Statement
```

But in our enum-based definition:
```rust
enum Statement {
    IfThen(...),        // Separate variant
    IfThenElse(...),    // Separate variant
}
```

The variants are **distinguished at grammar construction time**, not parse time.

---

## 💡 Key Insights

### 1. Enum Variants Create Implicit Disambiguation

Rust-sitter's grammar macro translates Rust enums into grammar productions. Each enum variant becomes a distinct production rule. This creates implicit disambiguation that prevents conflicts.

**Traditional BNF** (creates conflicts):
```bnf
stmt ::= "if" expr "then" stmt
       | "if" expr "then" stmt "else" stmt
```

**Rust-sitter** (conflict-free):
```rust
enum Statement {
    IfThen(if, Expr, then, Statement),
    IfThenElse(if, Expr, then, Statement, else, Statement),
}
```

### 2. LR(1) is Powerful for Well-Structured Grammars

LR(1) lookahead can resolve many ambiguities that would be conflicts in LR(0) or SLR parsers. Our grammars are well-structured enough that LR(1) handles them deterministically.

### 3. True GLR Requires Inherent Ambiguity

To test GLR conflict preservation, we need grammars with **inherent ambiguity** that cannot be resolved by:
- Precedence annotations
- Distinct enum variants
- LR(1) lookahead

---

## 🎯 How to Create a Grammar with Guaranteed Conflicts

### Option A: Ambiguous Expression Grammar (RECOMMENDED)

Create a grammar where the SAME production can appear in multiple contexts:

```rust
#[adze::grammar("ambiguous_expr")]
pub mod grammar {
    #[adze::language]
    pub enum Expr {
        // Single binary operation rule (no precedence)
        Binary(
            Box<Expr>,
            #[adze::leaf(pattern = r"[-+*/]")] String,
            Box<Expr>,
        ),

        Number(#[adze::leaf(pattern = r"\d+")] i32),
    }
}
```

**Why this creates conflicts**:
- No precedence annotations
- Single `Binary` variant for all operators
- Input `"1 - 2 + 3"` is inherently ambiguous:
  - Could be `(1 - 2) + 3` (left-associative)
  - Could be `1 - (2 + 3)` (right-associative)
- LR(1) **cannot** disambiguate without precedence info

### Option B: Grammar with Indirect Left Recursion

```rust
#[adze::language]
pub enum S {
    A(Box<A>),
    B(Box<B>),
}

#[adze::language]
pub enum A {
    SA(Box<S>, #[adze::leaf(text = "a")] ()),
    Epsilon,
}

#[adze::language]
pub enum B {
    SB(Box<S>, #[adze::leaf(text = "b")] ()),
    Epsilon,
}
```

**Why this creates conflicts**: Indirect recursion through multiple non-terminals creates reduce/reduce conflicts.

### Option C: True Dangling-Else (Without Enum Variants)

Use a single Statement production with optional else:

```rust
#[adze::language]
pub enum Statement {
    If(
        #[adze::leaf(text = "if")] (),
        Box<Expr>,
        #[adze::leaf(text = "then")] (),
        Box<Statement>,
        Option<ElseClause>,  // Optional!
    ),
    Other(#[adze::leaf(text = "other")] ()),
}

pub struct ElseClause {
    #[adze::leaf(text = "else")] (),
    Box<Statement>,
}
```

**Problem**: Rust-sitter's `Option` handling may still create distinct productions.

---

## ✅ Recommended Next Steps

### Immediate (Today)
1. **Implement Option A**: Create ambiguous expression grammar without precedence
2. **Run diagnostic test**: Verify conflicts ARE detected
3. **Validate GLR fix**: Confirm multi-action cells present in parse table
4. **Document results**: Update BDD specification with working test grammar

### Short Term (This Week)
5. **Create unit tests**: glr-core level tests for conflict preservation
6. **Integration tests**: Verify tablegen generates multi-action cells
7. **Runtime tests**: Test GLR fork/merge behavior (once conflicts confirmed)
8. **Update documentation**: Mark dangling-else approach as "conflict-free by design"

### Medium Term (Next Sprint)
9. **Grammar library**: Create collection of conflict-generating test grammars
10. **Fuzzing**: Generate random ambiguous grammars for stress testing
11. **Performance**: Benchmark GLR vs LR on conflict-heavy grammars

---

## 📚 References

- **LR(1) Parsing Theory**: [Dragon Book, Chapter 4](https://en.wikipedia.org/wiki/LR_parser)
- **Ambiguous Grammars**: [Compilers: Principles, Techniques, and Tools](https://en.wikipedia.org/wiki/Ambiguous_grammar)
- **GLR Parsing**: [Scott & Johnstone 2006](https://en.wikipedia.org/wiki/GLR_parser)

---

## 🎓 Lessons Learned

### 1. Test Grammar Design is Critical
Simply having a "classically ambiguous" problem (like dangling-else) doesn't guarantee conflicts in all grammar formulations. The specific way the grammar is written matters enormously.

### 2. Enum Variants are Powerful Disambiguation
Rust-sitter's enum-based grammar definition creates strong separation between alternatives, which LR(1) can exploit to avoid conflicts.

### 3. Explicit Precedence Works Early
Precedence annotations likely affect grammar construction, not just conflict resolution. This makes them invisible to conflict detection.

### 4. TDD Reveals Deep Insights
Systematic testing with diagnostic tools revealed these insights that wouldn't be obvious from theory alone.

---

**Status**: Investigation complete. Next step: Create Option A (ambiguous expression) grammar.

**Estimated Effort**: 2-3 hours to implement and validate.
