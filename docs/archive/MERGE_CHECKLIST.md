# Merge-Ready Checklist

## Code Invariants & ABI

- [ ] `ts_format::TSActionTag` constants locked (tests in place)
- [ ] Dense/token-first normalization: terminals `[0..tcols)`, NTs `[tcols..N)`
- [ ] Reduce encodes **rule_id**; `child_count == rules[rule_id].rhs_len`
- [ ] NT goto merged as `Shift(next)` in NT columns (no runtime goto table)
- [ ] `Accept` present at `GOTO(I0,start)` on EOF (shape test added)
- [ ] Externals: `external_token_count ≥ 1`, external columns `(< tcols)`

## Decoder Shape

- [ ] Iterates **all** columns (`0..symbol_count`)
- [ ] `rules` reconstructed from `TSRule[i].rhs_len` + symbol mappings
- [ ] Fail-safe Reduce mapping → `Action::Error` (no silent rule 0)

## Parser Driver

- [ ] Chooser unified: **Accept > Shift > Reduce** (tests import the *same* chooser)
- [ ] `get_goto_state()` uses **NT map** (not token map)

## Lexers

- [ ] JSON `lex_fn` set via `set_lex_fn`
- [ ] INDENT stub uses **lex modes** (no `static mut`), `external_token_count = 1`
- [ ] `lex_modes` pointer lifetime owned or intentionally leaked in tests

## CI

- [ ] Small-table path test job enabled
- [ ] Toolchain matrix (stable + beta) runs core jobs
- [ ] Test connectivity safeguards in place (no `.rs.disabled` files)

## Docs

- [ ] `docs/ts_spec.md` top box: **DO NOT CHANGE** contract
- [ ] Reviewer snapshot in PR description

## Reviewer Snapshot

**Critical invariants that must be maintained:**

* **Tags**: Error=0, Shift=1, Reduce=3, Accept=4
* **Columns**: dense 0..N-1; tokens `[0..tcols)`, NTs `[tcols..N)`
* **Reduce**: `symbol = rule_id`, `child_count = rhs_len`
* **Goto**: NT goto encoded as `Shift(next_state)` in NT columns
* **Accept**: at `GOTO(I0,start)` on EOF
* **Decoder**: iterates **all** columns; reconstructs rules via `TSRule[]`; fail-safe reduce mapping
* **Externals**: `external_token_count ≥ 1`; externals in token band
* **Chooser**: **Accept > Shift > Reduce**; single source (`ts_format::choose_action`)

If any of these change, update encoder + decoder + tests + spec together.
