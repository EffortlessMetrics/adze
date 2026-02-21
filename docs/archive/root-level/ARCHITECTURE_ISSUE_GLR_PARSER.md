# Known Architectural Issue: GLR Table Generation with Simple LR Runtime

## Status: CRITICAL - Affects Pure-Rust Parser Correctness

## Summary

The pure-Rust parser implementation has a **critical architectural mismatch**:
- **Table Generation** (build-time): Uses GLR-core to generate multi-action parse tables with proper precedence/associativity ordering
- **Runtime Parser** (parse-time): Uses `pure_parser.rs` which is a **simple LR parser** that only takes the FIRST action for each symbol, ignoring the GLR capabilities

## Impact

**Grammar Features That DON'T Work Correctly**:
- ❌ Operator associativity (`#[prec_left(N)]`, `#[prec_right(N)]`)
- ❌ Ambiguous grammars requiring GLR
- ❌ Complex precedence disambiguation
- ❌ Any grammar with unresolved shift/reduce conflicts

**Example Failure**:
```rust
// Grammar:
#[adze::prec_left(2)]
Mul(Box<Expr>, "*", Box<Expr>)

// Input: "1 * 2 * 3"
// Expected: (1 * 2) * 3   ← left-associative
// Actual:   1 * (2 * 3)   ← WRONG! right-associative
```

## Root Cause

### Table Generation (Correct)
`glr-core/src/lib.rs:344` - Fixed to use rule associativity:
```rust
match (tokp.prec.cmp(&rulep.prec), rulep.assoc) {  // ✅ Uses rule assoc
    (Equal, Assoc::Left) => PrecDecision::PreferReduce,
    ...
}
```

### Table Compression (Preserves ordering)
`tablegen/src/compress.rs:404-421` - Stores all actions:
```rust
for action in action_cell {  // Stores ALL actions in order
    entries.push(CompressedActionEntry {
        symbol: symbol_id,
        action: action.clone(),
    });
}
```

### Runtime Parser (BUG HERE)
`runtime/src/pure_parser.rs:1054-1074` - Only uses FIRST action:
```rust
while offset + 1 < end_offset {
    let entry_col = *language.small_parse_table.add(offset);
    let entry_val = *language.small_parse_table.add(offset + 1);
    offset += 2;
    if entry_col == symbol {
        return self.decode_action(language, entry_val as usize);  // ❌ STOPS HERE!
    }
}
```

**Why This Fails**:
1. GLR-core generates `ActionCell` with `[Reduce(2), Shift(5)]` (precedence-ordered for left-assoc)
2. Tablegen compresses to: `[(symbol: 3, Reduce(2)), (symbol: 3, Shift(5))]`
3. `pure_parser` sees symbol 3 and returns `Reduce(2)` **without checking if there are more actions**
4. Result: Parser behavior depends on which action is stored first, not true GLR

## The Fix (Two Options)

### Option A: Wire grammar::parse() to use parser_v4 (RECOMMENDED)

`parser_v4.rs` IS a proper GLR parser with correct associativity handling (line 126):
```rust
let assoc_bias = if (rid.0 as usize) < self.parse_table.rule_assoc_by_rule.len() {
    self.parse_table.rule_assoc_by_rule[rid.0 as usize] as i32  // ✅ Uses rule assoc!
} else {
    0
};
```

**Change needed**:
- Modify `runtime/src/__private.rs:214-218` to use `parser_v4::Parser` instead of `pure_parser::Parser`
- Adapt the API (parser_v4 expects Grammar/ParseTable, pure_parser uses TSLanguage)
- Add feature flag or configuration to select parser implementation

**Files to modify**:
- `runtime/src/__private.rs` - Change parse() implementation
- `tool/src/pure_rust_builder.rs` - Generate parser_v4-compatible structures
- `example/Cargo.toml` - Add feature flag for GLR vs simple LR

### Option B: Make pure_parser GLR-capable

Modify `pure_parser.rs` to:
1. Process ALL actions for a symbol, not just the first
2. Implement fork/merge logic when multiple actions exist
3. Use parser state stack splitting as in `parser_v4.rs`

**Complexity**: HIGH - essentially rewriting pure_parser to be GLR

## Workaround (Current State)

**For grammars WITHOUT conflicts**: Pure-parser works fine
**For grammars WITH conflicts**: Use tree-sitter C backend (default, non-pure-rust)

```toml
# Cargo.toml
[dependencies]
adze = "0.8"  # Uses C backend by default - works correctly

[dependencies]
adze = { version = "0.8", features = ["pure-rust"] }  # ❌ Broken for associativity
```

## Test Impact

**Failing Tests** (example/src/arithmetic.rs):
- `arithmetic::tests::successful_parses` - line 83, 109 fail
- `arithmetic::tests::test_glr_precedence_disambiguation` - line 295 fails
- Expected: left-associative trees
- Actual: right-associative trees

**Passing Tests**:
- `test_empty_input` ✅ - No conflicts
- `test_precedence` ✅ - Different operators (no same-precedence associativity)
- `test_simple` ✅ - Single token

## Related Work

- PR #XYZ: Fixed glr-core to store rule associativity (commit 490c9b5)
- This fix is CORRECT but insufficient - runtime doesn't use it
- Need follow-up PR to wire parser_v4 or fix pure_parser

## References

- `glr-core/src/lib.rs:344` - GLR table generation (FIXED)
- `runtime/src/pure_parser.rs:1054` - Simple LR runtime (BUG)
- `runtime/src/parser_v4.rs:126` - GLR runtime (WORKS)
- `runtime/src/__private.rs:214` - Entry point to fix

## Action Items

1. [ ] Create GitHub issue tracking this architectural mismatch
2. [ ] Add documentation warning about pure-rust limitations
3. [ ] Implement Option A (wire parser_v4) in follow-up PR
4. [ ] Add integration tests that verify associativity works
5. [ ] Consider deprecating pure_parser in favor of parser_v4

## Timeline

- **Discovered**: 2025-11-16
- **Root cause identified**: context-scout agent analysis
- **Fix attempted**: glr-core rule associativity storage (partial)
- **Full fix needed**: Wire parser_v4 or rewrite pure_parser
- **Estimated effort**: 2-4 hours for Option A, 8-16 hours for Option B

---

**Last Updated**: 2025-11-16
**Author**: Claude (debugging session)
**Priority**: HIGH - Affects correctness of pure-Rust parser
