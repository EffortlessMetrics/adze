# Correctness Push Plan

**Last updated:** 2026-05-06
**Scope:** current parser/runtime, GLR, tablegen ABI, CLI, and product-proof convergence.

This is the execution playbook for moving Adze from "bounded core lane is green" to "the product claims are behavior-proven." It is intentionally narrower than a roadmap: finish the active correctness queue first, then open focused follow-up work for gaps that should not be hidden inside broad implementation PRs.

## Baseline

- Required fast gate stays `just ci-supported`.
- `ci-supported` covers the seven core crates: `adze`, `adze-macro`, `adze-tool`, `adze-common`, `adze-ir`, `adze-glr-core`, and `adze-tablegen`.
- The broader product lane is advisory until each canary proves real behavior instead of only compile/no-run smoke.
- README-stable claims must map to a proof command in `docs/status/SUPPORT_TIERS.md`.
- Runtime2 remains an experimental proving ground unless a later promotion plan gives it required behavior tests and a public support contract.

## Live-State Refresh

Before merging any queued PR, refresh the real GitHub state:

```bash
gh pr list --state open --limit 50 \
  --json number,title,mergeable,isDraft,headRefName,baseRefName,updatedAt,url
```

If GitHub API access is rate-limited, do not guess mergeability. Continue with local rebases and tests, but report that live PR count could not be refreshed.

## Merge Queue

Use the queue below as the current seed order. Re-check live state before each merge because PRs may have landed, closed, or changed base.

| Order | PR | Direction | Required proof beyond `just ci-supported` |
|---:|---|---|---|
| 1 | #300 | Rebase, test, merge | `cargo test -p adze-tablegen language_gen::tests::test_count_symbols_uses_parse_table_count_with_externals -- --exact`; `cargo clippy -p adze-tablegen -- -D warnings` |
| 2 | #306 | Rebase after #300, test, merge | `cargo test -p adze-tablegen --test primary_state_comprehensive lang_gen_primary_state_count_equals_state_count -- --exact`; `cargo test -p adze-tablegen --test primary_state_comprehensive` |
| 3 | #303 | Rebase, test, merge | `cargo check -p adze-tool`; clean downstream/repro build if available |
| 4 | #332 | Merge after ABI tests | `cargo test -p adze-glr-core --lib test_ts_lexer_layout_matches_tree_sitter_abi -- --nocapture`; `cargo test -p adze-glr-core --lib test_grammar_lexer_uses_tree_sitter_callback_abi -- --nocapture` |
| 5 | #335 | Merge after timeout tests | `cargo test -p adze --lib unified_parser::tests:: -- --nocapture` |
| 6 | #348 | Merge after CLI tests | `cargo test -p adze-cli -- --nocapture` |
| 7 | #386 | Manually resolve conflicts, then merge | `cargo test -p adze-tablegen compression -- --nocapture`; `cargo test -p adze-tablegen validation -- --nocapture`; `cargo test -p adze-tablegen --all-features --no-run` |
| 8 | #387 | Manually resolve conflicts, then merge | `cargo test -p adze-glr-core conflict -- --nocapture`; `cargo test -p adze-glr-core ambiguity -- --nocapture`; `cargo test -p adze-glr-core --all-features --no-run` |
| 9 | #376 | Merge only if strict canary proves real parser output | `cargo test -p adze-golden-tests -- --nocapture`; `cargo test -p adze-golden-tests --features javascript-grammar javascript_canary_expression_golden -- --nocapture` |
| 10 | #398 | Merge after correctness PRs | `cargo metadata --format-version 1`; `cargo tree -i criterion`; `cargo tree -i bincode`; benchmark no-run checks; `cargo test -p adze-ir serde -- --nocapture` |
| Last | #321 | Rebase or close if superseded | Keep separate from correctness merges |

For every PR:

```bash
git checkout main
git pull --ff-only
gh pr checkout <PR>
git fetch origin main
git rebase origin/main
cargo fmt --all -- --check
just ci-supported
```

Do not merge #386 or #387 without resolving conflicts manually. Do not weaken #376's canary to make it pass. Do not let #398 change production parse-table format semantics; postcard should remain dev/test-only unless a separate format migration is explicitly designed.

## Post-Queue Issues

After the PR queue is empty, open focused issues instead of broad catch-all implementation PRs:

- GLR product proof: conflict-preserving end-to-end typed extraction.
- Tablegen ABI completeness: conflict encoding/routing, field maps, symbol/state invariants.
- Product-proof behavior lane: replace compile-only canaries with one real behavior per major surface.
- Parse diagnostics: spans, expected token sets, line/column mapping, excerpts.
- CLI clean-room quickstart and parse command truthfulness.
- README/support-tier reconciliation: no Stable claim without a named proof command.
- Benchmark truthfulness: real parser work vs infrastructure-only measurements.

## Green Ladder

Rung 0 is the current required gate:

```bash
just ci-supported
```

Rung 1 is advisory product behavior. Convert `scripts/ci-product.sh` from compile-only smoke to bounded behavior smokes, but keep it non-blocking until stable.

Rung 2 is a stable product lane. Promote only README-stable claims:

```bash
just ci-supported
just ci-product-stable
```

The stable product lane should cover a clean-room README quickstart, typed extraction exact-value test, operator precedence test, GLR ambiguity canary, serialization canary, and one structured parse-error diagnostic test.

Rung 3 remains scheduled/manual: full workspace all-features, fuzzing, Miri, sanitizers, benchmarks, grammar corpus, runtime2, and browser WASM execution.

## Reporting Format

After each merge or failed merge attempt, report:

- PR handled.
- Proof commands run.
- Current open PR count, or why it could not be refreshed.
- Red checks.
- Next blocking PR.
