# GLR Parser Guardrails and Regression Prevention

## Post-Merge Guardrails

* Add a non-blocking **ts-bridge parity** job (JSON & Python): log accept/error status, don't require identical trees yet.
* Keep a tiny **criterion benchmark** (tokens/sec, peak stacks) to watch perf drift.
* Consider turning on safe-dedup only when `new_stacks.len() > N` (currently ~10) and make `N` a const.

## "If this ever breaks again" Cheat-Sheet

| Symptom | Likely Cause | Check |
|---------|--------------|-------|
| Missing accepts near EOF | Re-closure removed or EOF loop order changed | Look for Phase-2 `reduce_until_saturated` with same lookahead |
| Infinite loop on ε | Stamp lost `end` fallback | Check RedStamp uses `(state, rule, end)` tuple |
| Query double-counts | Wrapper-squash or capture dedup missing | Verify `convert_forest_to_query_subtree` squashes same-span nodes |
| Single parse on ambiguous input | Accept aggregation or dedup collapsing distinct roots | Check per-token accept collection and pointer-equality dedup |
| Fork not happening | LR(1) depth issue or state-only dedup | Ensure test inputs are ≥3 tokens for ambiguity |
| Stack explosion | Safe dedup disabled or threshold too high | Check `new_stacks.len() > 10` guard |

## Core Correctness Invariants

These must NEVER be removed:

1. **Phase-2 Reduce → re-close**: After any reduction, re-saturate with same lookahead
2. **Accept Aggregation**: Collect ALL accepts per token, no early returns
3. **EOF Recovery Pattern**: `close → check → (insert|pop)`, never delete at EOF
4. **Epsilon Guard**: RedStamp keyed on `(state, rule, end)` with position tracking
5. **Nonterminal Goto**: Use goto table for nonterminals, not action table
6. **Wrapper Squashing**: Collapse unary nodes with identical byte ranges
7. **Safe Deduplication**: Only remove pointer-equal duplicates, not state-equal

## Regression Test Guards

Each fix has a corresponding test in `test_glr_regression_guards.rs`:

- `test_reduce_reclosure_guard` - Fails if Phase-2 re-closure removed
- `test_eof_recovery_no_delete_guard` - Fails if EOF deletes tokens
- `test_accept_aggregation_guard` - Fails if accepts aren't aggregated
- `test_wrapper_squash_guard` - Validates wrapper squashing concept
- `test_no_state_only_dedup_guard` - Ensures pointer-based dedup

## Performance Monitoring Points

Track these metrics to catch regressions:

- **Tokens/sec** on deterministic input (baseline)
- **Peak stack count** on known ambiguous input
- **Memory usage** for large files (>10K tokens)
- **Fork frequency** on ambiguous grammars
- **Subtree reuse %** in incremental parsing

## Testing Matrix

Always run with these feature combinations:
- Default features
- `--features incremental_glr`
- `--features pure-rust`
- `--all-features`

## Red Flags in Code Review

Watch for these patterns that might break GLR:

- Removing any `reduce_until_saturated` call
- Early `return` on `Action::Accept`
- `break` in EOF recovery loop without accept check
- Dedup based on state alone (not pointer equality)
- Changing EOF loop order from `close → check → recover`
- Removing position from epsilon guards
- Direct returns in Phase-2 processing