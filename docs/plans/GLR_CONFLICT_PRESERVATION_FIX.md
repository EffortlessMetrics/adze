# GLR Conflict Preservation Fix

**Date**: 2025-11-19
**Status**: DESIGN
**Priority**: CRITICAL (Unblocks GLR runtime)
**Effort**: 4-6 hours
**Related**: PARSER_V4_TABLE_LOADING_BLOCKER.md, GLR_RUNTIME_WIRING_PLAN.md

---

## 📊 Problem Statement

**Discovery**: The arithmetic grammar has ZERO GLR conflicts in its parse table.

```
=== Arithmetic Grammar Parse Table Inspection ===
Total states: 11
Total symbols: 12
--- Multi-Action Cells (GLR Conflicts) ---
  No multi-action cells found (no GLR conflicts detected)
```

**Root Cause**: `glr-core/src/lib.rs::resolve_shift_reduce_conflict()` **eliminates** conflicts when precedence/associativity is defined, creating a conflict-free LR grammar instead of preserving conflicts for GLR runtime handling.

---

## 🔍 Current Behavior (Incorrect)

### Code Location: `glr-core/src/lib.rs:2019-2071`

```rust
fn resolve_shift_reduce_conflict(&self, conflict: &mut Conflict, grammar: &Grammar) {
    // ...
    match compare_precedences(shift_prec, reduce_prec) {
        PrecedenceComparison::PreferShift => {
            conflict.actions = vec![shift];  // ❌ WRONG: Eliminates reduce!
        }
        PrecedenceComparison::PreferReduce => {
            conflict.actions = vec![reduce];  // ❌ WRONG: Eliminates shift!
        }
        PrecedenceComparison::Error => {
            conflict.actions = vec![Action::Fork(vec![shift, reduce])];  // ✅ Correct
        }
        PrecedenceComparison::None => {
            conflict.actions = vec![Action::Fork(vec![shift, reduce])];  // ✅ Correct
        }
    }
}
```

**Impact**:
- Precedence annotations eliminate conflicts → Simple LR grammar
- No multi-action cells in parse table
- parser_v4 can't demonstrate GLR behavior
- Arithmetic tests show wrong associativity (because they use pure_parser fallback)

---

## 🎯 Desired Behavior (Correct)

### GLR Conflict Preservation

For **true GLR parsing**, we need to:
1. **Preserve BOTH actions** when a conflict occurs
2. **Order actions by precedence/associativity** for runtime priority
3. Let **parser_v4 explore multiple derivations** using action priorities

### Proposed Fix

```rust
fn resolve_shift_reduce_conflict(&self, conflict: &mut Conflict, grammar: &Grammar) {
    let precedence_resolver = StaticPrecedenceResolver::from_grammar(grammar);

    let mut shift_action = None;
    let mut reduce_action = None;

    for action in &conflict.actions {
        match action {
            Action::Shift(_) => shift_action = Some(action.clone()),
            Action::Reduce(_) => reduce_action = Some(action.clone()),
            _ => {}
        }
    }

    match (shift_action, reduce_action) {
        (Some(shift), Some(reduce)) => {
            let shift_prec = precedence_resolver.token_precedence(conflict.symbol);
            let reduce_prec = if let Action::Reduce(rule_id) = &reduce {
                precedence_resolver.rule_precedence(*rule_id)
            } else {
                None
            };

            match compare_precedences(shift_prec, reduce_prec) {
                PrecedenceComparison::PreferShift => {
                    // ✅ NEW: Preserve both, shift first (higher priority)
                    conflict.actions = vec![shift, reduce];
                }
                PrecedenceComparison::PreferReduce => {
                    // ✅ NEW: Preserve both, reduce first (higher priority)
                    conflict.actions = vec![reduce, shift];
                }
                PrecedenceComparison::Error => {
                    // Non-associative: keep Fork for error reporting
                    conflict.actions = vec![Action::Fork(vec![shift, reduce])];
                }
                PrecedenceComparison::None => {
                    // No precedence: use Fork to explore all paths
                    conflict.actions = vec![Action::Fork(vec![shift, reduce])];
                }
            }
        }
        _ => {
            // Keep original actions
        }
    }
}
```

---

## 📋 Implementation Plan

### Step 1: Update Conflict Resolution (2 hours)

**File**: `glr-core/src/lib.rs`

**Changes**:
1. Modify `resolve_shift_reduce_conflict()` to preserve both actions
2. Order actions by precedence (first action = higher priority)
3. Remove `Fork` for precedence-based ordering (actions directly in cell)

**Tests**:
- Verify arithmetic grammar generates multi-action cells
- Check action ordering matches precedence
- Ensure left/right associativity affects action order

### Step 2: Verify Table Generation (1 hour)

**Test**: Run table inspection test again

```bash
cargo test -p rust-sitter --test test_arithmetic_table_loading --features glr -- --nocapture
```

**Expected Output**:
```
--- Multi-Action Cells (GLR Conflicts) ---
  State X, Symbol - (minus): 2 actions
    Shift(StateId(Y))
    Reduce(RuleId(Z))
  State X, Symbol * (mult): 2 actions
    Shift(StateId(A))
    Reduce(RuleId(B))
```

### Step 3: Update parser_v4 Priority Handling (1-2 hours)

**File**: `runtime/src/parser_v4.rs`

**Current Behavior**: `action_priority()` assigns priorities based on action type and dynamic precedence

**Required Changes**:
- Honor **action order in cell** as primary priority
- First action in cell = highest priority
- Fall back to existing priority logic for ties

```rust
fn choose_action(&self, actions: &[Action], state: StateId) -> Action {
    if actions.is_empty() {
        return Action::Error;
    }

    // Priority 1: Action position in cell (first = highest)
    // Priority 2: action_priority() for ties
    actions
        .iter()
        .enumerate()
        .max_by_key(|(index, action)| {
            let position_priority = -((*index) as i32) * 10_000;  // Earlier = higher
            let action_priority = self.action_priority(action);
            position_priority + action_priority
        })
        .map(|(_, action)| action.clone())
        .unwrap_or(Action::Error)
}
```

### Step 4: Integration Testing (1 hour)

**Tests**:
1. Run arithmetic tests with GLR feature
2. Verify left-associativity: `"1 - 2 - 3"` → `((1 - 2) - 3)`
3. Verify precedence: `"1 - 2 * 3"` → `(1 - (2 * 3))`
4. Compare against expected snapshot tests

---

## 🧪 Test Strategy

### Test 1: Table Inspection
```bash
cargo test -p rust-sitter --test test_arithmetic_table_loading --features glr -- --nocapture
```
**Expected**: Multi-action cells detected

### Test 2: Arithmetic Parsing
```bash
cargo test -p rust-sitter-example --lib --features glr test_glr_precedence
```
**Expected**: All precedence/associativity tests pass

### Test 3: BDD Scenarios
```bash
# After fix, implement BDD scenarios from GLR_RUNTIME_WIRING_PLAN.md Step 5
cargo test -p rust-sitter-example --features glr scenario_left_assoc
cargo test -p rust-sitter-example --features glr scenario_precedence
```

---

## 🎯 Success Criteria

### When Fix is Complete:

1. **Table Generation**:
   - ✅ Arithmetic grammar has multi-action cells
   - ✅ Actions ordered by precedence (shift vs reduce)
   - ✅ Left/right associativity affects action order

2. **Parser Execution**:
   - ✅ parser_v4 successfully parses arithmetic expressions
   - ✅ Precedence honored: `1 - 2 * 3` → `(1 - (2 * 3))`
   - ✅ Left-assoc honored: `1 - 2 - 3` → `((1 - 2) - 3)`

3. **Tests Passing**:
   - ✅ `test_glr_precedence_disambiguation` passes
   - ✅ All arithmetic tests pass with `--features glr`
   - ✅ No regressions in existing tests

---

## 🔗 Related Issues

### This Fix Resolves:
- PARSER_V4_TABLE_LOADING_BLOCKER.md (wasn't table loading, was table generation!)
- GLR_RUNTIME_WIRING_PLAN.md Steps 5-6 (unblocks BDD tests)

### Architectural Note:
This fix changes glr-core from "LR with precedence resolution" to "true GLR with precedence ordering". This is a **fundamental architectural improvement** that enables the full power of GLR parsing.

---

## 📅 Timeline

**Estimated Effort**: 4-6 hours

**Breakdown**:
- Step 1: 2 hours (conflict resolution change)
- Step 2: 1 hour (table verification)
- Step 3: 1-2 hours (parser_v4 priority)
- Step 4: 1 hour (integration testing)

**Target Completion**: Same day (high priority)

---

## 📝 Notes

### Fork vs Multi-Action

The current code uses `Action::Fork([shift, reduce])` for unresolved conflicts. After this fix:

- **With precedence**: `vec![shift, reduce]` or `vec![reduce, shift]` (ordered by priority)
- **Without precedence**: `vec![Action::Fork([shift, reduce])]` (fork for exploration)
- **Non-associative**: `vec![Action::Fork([shift, reduce])]` (fork for error detection)

### Compatibility

This change is **backward compatible**:
- Grammars without conflicts: No change (single action per cell)
- Grammars without precedence: Still use Fork
- New behavior only affects grammars with precedence AND conflicts

---

**Let's implement this fix!** 🚀
