# Policy allowlist conventions

Cross-cutting conventions that apply to every TOML allowlist under
`/policy/`.

## Naming

Each allowlist file is `<surface>.toml`. The schema version is the first
key:

```toml
schema_version = "0.3"
```

`check-no-panic-family`, `check-file-policy`, and `check-lint-policy`
read the schema version and fail loudly if it does not match what they
were built for.

## Identity vs. drift hint

The matching key for an entry is the **logical identity**, not the
location:

| File                            | Identity                                |
| ------------------------------- | --------------------------------------- |
| `no-panic-allowlist.toml`       | `path + family + selector`              |
| `non-rust-allowlist.toml`       | `glob` (or `path` for single-file)      |
| `clippy-debt.toml`              | `name + crate`                          |

Where present, `last_seen` (line + column) is a *drift hint*. The
checker flags it when reality drifts, but never refuses to match an
entry just because the line number moved.

## Required metadata

Every entry must declare:

* `owner` — a component or team name. Single-word slugs preferred.
* a `reason` / `explanation` field — one sentence on **why**.
* `expires` — ISO date. Required for entries classified `tooling` and
  for any debt-style receipt.

`retired = true` keeps an entry in the file for audit history without
requiring it to match. Stale (non-retired, non-matching) entries fail.

## Fallible test helpers

A future PR will introduce small fallible-assertion helpers shared by
the test suite:

```rust
testing::ensure(cond, msg) -> Result<()>
testing::ensure_eq(left, right) -> Result<()>
testing::require_some(opt) -> Result<T>
testing::require_ok(res) -> Result<T>
```

This is *not* part of the initial rollout. Tests today still use the
standard `assert_eq!` family. The no-panic checker does not flag
assertion macros.

## Generated reports

All checkers write to `target/policy/reports/` so that CI can upload a
single artifact:

```
target/policy/reports/no-panic.md
target/policy/reports/no-panic.json
target/policy/reports/no-panic-proposed-allowlist.toml
target/policy/reports/file-policy.md
target/policy/reports/file-policy.json
target/policy/reports/lint-policy.md
target/policy/reports/lint-policy.json
target/policy/reports/policy-summary.md
```

The `.md` reports are human-readable; the `.json` reports are
machine-readable for downstream automation (badges, dashboards).

## Editor's checklist for receipts

Before committing a new receipt:

- [ ] Identity (path + family + selector for no-panic; glob for files)
- [ ] Owner is a real component/team
- [ ] Reason explains *why this is the right shape*, not just "fix later"
- [ ] `expires` is set and is realistic
- [ ] If the entry will be removed by a planned refactor, link the
      tracking issue in `explanation`
- [ ] `covered_by` lists the command that proves the file/code is correct,
      if applicable
